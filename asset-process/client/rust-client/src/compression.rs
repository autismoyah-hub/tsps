use crate::{crypto::crc32, Error, Result};

/// JS5 omits the four-byte BZh1 header from bzip2 payloads.
pub fn bzip2(payload: &[u8], expected: usize) -> Result<Vec<u8>> {
    if expected > u32::MAX as usize || payload.len() > u32::MAX as usize - 4 {
        return Err(Error::Invalid("bzip2 size"));
    }
    let mut source = b"BZh1".to_vec();
    source.extend_from_slice(payload);
    let mut output = vec![0u8; expected.max(1)];
    let mut length = output.len() as u32;
    // SAFETY: Both pointers address exclusively owned allocations of the supplied
    // lengths, live throughout the call. The implementation is vendored Rust;
    // no JVM, external process or system libbz2 is involved.
    let status = unsafe {
        libbz2_rs_sys::BZ2_bzBuffToBuffDecompress(
            output.as_mut_ptr().cast(),
            &mut length,
            source.as_mut_ptr().cast(),
            source.len() as u32,
            0,
            0,
        )
    };
    if status != 0 || length as usize != expected {
        return Err(Error::Invalid("bzip2 data or length"));
    }
    output.truncate(expected);
    Ok(output)
}

pub fn gzip(data: &[u8], expected: usize) -> Result<Vec<u8>> {
    if data.len() < 18 || data[..3] != [0x1f, 0x8b, 8] {
        return Err(Error::Invalid("gzip header"));
    }
    let flags = data[3];
    if flags & 0xe0 != 0 {
        return Err(Error::Invalid("gzip reserved flags"));
    }
    let end = data.len() - 8;
    let mut pos = 10usize;
    if flags & 4 != 0 {
        let b = data.get(pos..pos + 2).ok_or(Error::Truncated)?;
        pos = pos
            .checked_add(2 + u16::from_le_bytes([b[0], b[1]]) as usize)
            .ok_or(Error::Truncated)?;
    }
    for flag in [8, 16] {
        if flags & flag != 0 {
            let tail = data.get(pos..end).ok_or(Error::Truncated)?;
            pos += tail.iter().position(|&b| b == 0).ok_or(Error::Truncated)? + 1;
        }
    }
    if flags & 2 != 0 {
        let b = data.get(pos..pos + 2).ok_or(Error::Truncated)?;
        if crc32(&data[..pos]) as u16 != u16::from_le_bytes([b[0], b[1]]) {
            return Err(Error::Invalid("gzip header checksum"));
        }
        pos += 2;
    }
    let compressed = data.get(pos..end).ok_or(Error::Truncated)?;
    let output = miniz_oxide::inflate::decompress_to_vec_with_limit(compressed, expected)
        .map_err(|_| Error::Invalid("gzip deflate data"))?;
    let checksum = u32::from_le_bytes(data[end..end + 4].try_into().unwrap());
    let size = u32::from_le_bytes(data[end + 4..].try_into().unwrap());
    if output.len() != expected || output.len() as u32 != size || crc32(&output) != checksum {
        return Err(Error::Invalid("gzip length or checksum"));
    }
    Ok(output)
}
