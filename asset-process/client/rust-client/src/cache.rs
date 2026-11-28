use crate::{buffer::Reader, Error, Result};

/// Read-only dat2/idx sector chain reader. Supports large group IDs.
pub fn read_group(dat: &[u8], idx: &[u8], archive: u8, group: u32) -> Result<Vec<u8>> {
    read_group_from(
        &mut std::io::Cursor::new(dat),
        &mut std::io::Cursor::new(idx),
        archive,
        group,
    )
}

pub fn read_group_from<D: std::io::Read + std::io::Seek, I: std::io::Read + std::io::Seek>(
    dat: &mut D,
    idx: &mut I,
    archive: u8,
    group: u32,
) -> Result<Vec<u8>> {
    use std::io::SeekFrom;
    fn exact(reader: &mut impl std::io::Read, bytes: &mut [u8]) -> Result<()> {
        reader.read_exact(bytes).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::Truncated
            } else {
                e.into()
            }
        })
    }
    idx.seek(SeekFrom::Start(u64::from(group) * 6))?;
    let mut entry = [0; 6];
    exact(idx, &mut entry)?;
    let mut r = Reader::new(&entry);
    let length = r.u24()? as usize;
    let mut sector = r.u24()? as usize;
    if length == 0 {
        return Ok(Vec::new());
    }
    let mut output = Vec::with_capacity(length.min(1 << 20));
    let mut chunk = 0u16;
    while output.len() < length {
        if sector == 0 {
            return Err(Error::Invalid("zero sector in chain"));
        }
        let header = if group > 65535 { 10 } else { 8 };
        let count = (length - output.len()).min(520 - header);
        dat.seek(SeekFrom::Start(sector as u64 * 520))?;
        let mut bytes = [0; 520];
        exact(dat, &mut bytes[..header + count])?;
        let mut s = Reader::new(&bytes[..header + count]);
        let actual_group = if group > 65535 {
            s.u32()?
        } else {
            s.u16()? as u32
        };
        let actual_chunk = s.u16()?;
        let next = s.u24()? as usize;
        let actual_archive = s.u8()?;
        if actual_group != group || actual_chunk != chunk || actual_archive != archive {
            return Err(Error::Invalid("cache sector header mismatch"));
        }
        output.extend_from_slice(s.bytes(count)?);
        sector = next;
        if output.len() < length {
            chunk = chunk
                .checked_add(1)
                .ok_or(Error::Invalid("too many sectors"))?;
        }
    }
    Ok(output)
}

#[derive(Debug)]
pub struct Container<'a> {
    pub compression: u8,
    pub uncompressed_size: usize,
    pub payload: &'a [u8],
    pub version: Option<u32>,
}
impl<'a> Container<'a> {
    pub fn parse(data: &'a [u8], max_size: usize) -> Result<Self> {
        let mut r = Reader::new(data);
        let compression = r.u8()?;
        let len = r.u32()? as usize;
        if compression > 2 {
            return Err(Error::Unsupported("cache compression type"));
        }
        if len > max_size {
            return Err(Error::Invalid("compressed size limit"));
        }
        let uncompressed_size = if compression == 0 {
            len
        } else {
            r.u32()? as usize
        };
        if uncompressed_size > max_size {
            return Err(Error::Invalid("uncompressed size limit"));
        }
        let payload = r.bytes(len)?;
        let version = match r.remaining() {
            0 => None,
            2 => Some(u32::from(r.u16()?)),
            4 => Some(r.u32()?),
            _ => return Err(Error::Invalid("container trailing bytes")),
        };
        Ok(Self {
            compression,
            uncompressed_size,
            payload,
            version,
        })
    }
    pub fn uncompressed(&self) -> Result<&'a [u8]> {
        if self.compression != 0 {
            return Err(Error::Invalid("compressed container; use decompress"));
        }
        Ok(self.payload)
    }
    pub fn decompress(&self) -> Result<Vec<u8>> {
        match self.compression {
            0 => Ok(self.payload.to_vec()),
            1 => crate::compression::bzip2(self.payload, self.uncompressed_size),
            2 => crate::compression::gzip(self.payload, self.uncompressed_size),
            _ => Err(Error::Unsupported("cache compression")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub id: u32,
    pub name_hash: Option<u32>,
}
#[derive(Debug, Clone)]
pub struct GroupInfo {
    pub id: u32,
    pub name_hash: Option<u32>,
    pub crc: u32,
    pub version: u32,
    pub sizes: Option<(u32, u32)>,
    pub files: Vec<FileInfo>,
}
#[derive(Debug)]
pub struct ArchiveIndex {
    pub format: u8,
    pub version: u32,
    pub groups: Vec<GroupInfo>,
}
impl ArchiveIndex {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let format = r.u8()?;
        if !(5..=7).contains(&format) {
            return Err(Error::Unsupported("archive index format"));
        }
        let version = if format >= 6 { r.u32()? } else { 0 };
        let flags = r.u8()?;
        // Original GameIndexDataBase.bx rejects bits 1/3 and ignores higher bits.
        if flags & 0b1010 != 0 {
            return Err(Error::Unsupported(
                "archive digest/uncompressed checksum flags",
            ));
        }
        let count = |r: &mut Reader<'_>| -> Result<u32> {
            if format == 7 {
                r.large_smart()
            } else {
                Ok(r.u16()? as u32)
            }
        };
        let groups_count = count(&mut r)? as usize;
        if groups_count > 1_000_000 || groups_count > r.remaining() / 2 {
            return Err(Error::Invalid("archive group count"));
        }
        let mut groups = Vec::with_capacity(groups_count);
        let mut id = 0u32;
        for n in 0..groups_count {
            let delta = count(&mut r)?;
            if n > 0 && delta == 0 {
                return Err(Error::Invalid("duplicate group id"));
            }
            id = id
                .checked_add(delta)
                .ok_or(Error::Invalid("group id overflow"))?;
            groups.push(GroupInfo {
                id,
                name_hash: None,
                crc: 0,
                version: 0,
                sizes: None,
                files: Vec::new(),
            });
        }
        if flags & 1 != 0 {
            for group in &mut groups {
                group.name_hash = Some(r.u32()?);
            }
        }
        for group in &mut groups {
            group.crc = r.u32()?;
        }
        if flags & 4 != 0 {
            for group in &mut groups {
                group.sizes = Some((r.u32()?, r.u32()?));
            }
        }
        for group in &mut groups {
            group.version = r.u32()?;
        }
        let mut file_counts = Vec::with_capacity(groups_count);
        for _ in 0..groups_count {
            file_counts.push(count(&mut r)? as usize);
        }
        for (group, n) in groups.iter_mut().zip(file_counts) {
            if n > r.remaining() / 2 {
                return Err(Error::Truncated);
            }
            let mut id = 0u32;
            for i in 0..n {
                let delta = count(&mut r)?;
                if i > 0 && delta == 0 {
                    return Err(Error::Invalid("duplicate file id"));
                }
                id = id
                    .checked_add(delta)
                    .ok_or(Error::Invalid("file id overflow"))?;
                group.files.push(FileInfo {
                    id,
                    name_hash: None,
                });
            }
        }
        if flags & 1 != 0 {
            for group in &mut groups {
                for file in &mut group.files {
                    file.name_hash = Some(r.u32()?);
                }
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("archive index trailing bytes"));
        }
        Ok(Self {
            format,
            version,
            groups,
        })
    }
}

/// Split an already decompressed JS5 group into its component files.
pub fn split_files(data: &[u8], file_count: usize) -> Result<Vec<Vec<u8>>> {
    if file_count == 0 || file_count > 65535 {
        return Err(Error::Invalid("file count"));
    }
    if file_count == 1 {
        return Ok(vec![data.to_vec()]);
    }
    let chunks = *data.last().ok_or(Error::Truncated)? as usize;
    if chunks == 0 {
        return Err(Error::Invalid("zero chunks"));
    }
    let table_size = chunks
        .checked_mul(file_count)
        .and_then(|n| n.checked_mul(4))
        .ok_or(Error::Truncated)?;
    let table_start = data
        .len()
        .checked_sub(table_size + 1)
        .ok_or(Error::Truncated)?;
    let mut table = Reader::new(&data[table_start..data.len() - 1]);
    let mut files = vec![Vec::new(); file_count];
    let mut offset = 0usize;
    for _ in 0..chunks {
        let mut size = 0i64;
        for file in &mut files {
            size += table.u32()? as i32 as i64;
            if size < 0 {
                return Err(Error::Invalid("negative file chunk size"));
            }
            let end = offset.checked_add(size as usize).ok_or(Error::Truncated)?;
            file.extend_from_slice(
                data.get(offset..end)
                    .filter(|_| end <= table_start)
                    .ok_or(Error::Truncated)?,
            );
            offset = end;
        }
    }
    if offset != table_start {
        return Err(Error::Invalid("unused group data"));
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn two_sector_chain_and_corruption() {
        let mut dat = vec![0; 1560];
        let idx = [0, 2, 1, 0, 0, 1];
        dat[520..528].copy_from_slice(&[0, 0, 0, 0, 0, 0, 2, 7]);
        dat[528..1040].fill(42);
        dat[1040..1048].copy_from_slice(&[0, 0, 0, 1, 0, 0, 0, 7]);
        dat[1048] = 99;
        let out = read_group(&dat, &idx, 7, 0).unwrap();
        assert_eq!(out.len(), 513);
        assert_eq!(out[512], 99);
        dat[1043] = 2;
        assert!(read_group(&dat, &idx, 7, 0).is_err());
        dat[1043] = 1;
        dat[527] = 6;
        assert!(read_group(&dat, &idx, 7, 0).is_err());
    }
    #[test]
    fn group_chunk_table() {
        let mut data = b"abcDEfgHIJK".to_vec();
        for n in [3i32, -1, 2, 2] {
            data.extend_from_slice(&n.to_be_bytes());
        }
        data.push(2);
        assert_eq!(
            split_files(&data, 2).unwrap(),
            vec![b"abcfg".to_vec(), b"DEHIJK".to_vec()]
        );
        data.pop();
        assert!(split_files(&data, 2).is_err());
    }
    #[test]
    fn container_boundaries() {
        let data = [0, 0, 0, 0, 3, 1, 2, 3, 0, 7];
        let c = Container::parse(&data, 20).unwrap();
        assert_eq!(c.version, Some(7));
        assert_eq!(c.uncompressed().unwrap(), &[1, 2, 3]);
        assert!(Container::parse(&data, 2).is_err());
        assert!(Container::parse(&data[..7], 20).is_err());
    }
}
