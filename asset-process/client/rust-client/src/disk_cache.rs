//! Read-only cache access, reference-table checksums, XTEA and file extraction.
use crate::{
    cache::{self, ArchiveIndex, Container, GroupInfo},
    crypto, Error, Result,
};
use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

fn scan_archive(groups: &[u32], mut load: impl FnMut(u32) -> Result<bool>) -> Result<bool> {
    let mut ready = true;
    for &group in groups {
        ready &= load(group)?;
    }
    Ok(ready)
}

pub struct DiskCache {
    root: PathBuf,
    data: Option<File>,
    indexes: HashMap<u8, File>,
    references: HashMap<u8, ArchiveIndex>,
    reference_containers: HashMap<u8, std::sync::Arc<Vec<u8>>>,
    master: Option<crate::cache_recovery::MasterIndex>,
    max_size: usize,
    recovery: Option<crate::cache_recovery::Recovery>,
    recovery_wait: std::time::Duration,
    #[cfg(feature = "custom-assets")]
    package: Option<std::sync::Arc<crate::asset_package::Package>>,
    #[cfg(feature = "custom-assets")]
    package_only: bool,
    #[cfg(feature = "custom-assets")]
    package_checked: bool,
    #[cfg(feature = "custom-assets")]
    floor: Option<std::sync::Arc<crate::authored_floor::Floor>>,
}
impl DiskCache {
    pub fn open(root: impl AsRef<Path>, max_size: usize) -> Result<Self> {
        Self::open_inner(root.as_ref(), max_size, false)
    }
    /// Read-only live cache, including an absent directory. Recovery remains in memory.
    pub fn open_remote(
        root: impl AsRef<Path>,
        max_size: usize,
        address: std::net::SocketAddr,
    ) -> Result<Self> {
        let mut cache = Self::open_inner(root.as_ref(), max_size, true)?;
        cache.enable_recovery(address)?;
        Ok(cache)
    }
    fn open_inner(root: &Path, max_size: usize, allow_missing: bool) -> Result<Self> {
        let root = root.to_owned();
        let data = match File::open(root.join("main_file_cache.dat2")) {
            Ok(file) => Some(file),
            Err(error) if allow_missing && error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        Ok(Self {
            root,
            data,
            indexes: HashMap::new(),
            references: HashMap::new(),
            reference_containers: HashMap::new(),
            master: None,
            max_size,
            recovery: None,
            recovery_wait: std::time::Duration::ZERO,
            #[cfg(feature = "custom-assets")]
            package: None,
            #[cfg(feature = "custom-assets")]
            package_only: false,
            #[cfg(feature = "custom-assets")]
            package_checked: false,
            #[cfg(feature = "custom-assets")]
            floor: None,
        })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn reopen(&self) -> Result<Self> {
        #[cfg(feature = "custom-assets")]
        if self.package_only {
            return Ok(Self {
                root: self.root.clone(), data: None, indexes: HashMap::new(),
                references: HashMap::new(), reference_containers: HashMap::new(),
                master: None, max_size: self.max_size, recovery: None,
                recovery_wait: std::time::Duration::ZERO,
                package: self.package.clone(), package_only: true, package_checked: true, floor: self.floor.clone(),
            });
        }
        let allow_missing = self.recovery.is_some();
        let mut cache = Self::open_inner(&self.root, self.max_size, allow_missing)?;
        #[cfg(feature = "custom-assets")]
        {
            cache.package = self.package.clone();
            cache.package_only = self.package_only;
            cache.floor = self.floor.clone();
        }
        cache.recovery = self.recovery.clone();
        cache.recovery_wait = self.recovery_wait;
        cache.master = self.master.clone();
        Ok(cache)
    }
    pub fn enable_recovery(&mut self, address: std::net::SocketAddr) -> Result<()> {
        #[cfg(feature = "custom-assets")]
        if self.package.is_some() {
            return Err(Error::Invalid("open a new cache and re-admit custom content before changing sources"));
        }
        self.master = None;
        self.references.clear();
        self.reference_containers.clear();
        self.recovery = Some(crate::cache_recovery::Recovery::connect(
            address,
            crate::protocol239::REVISION,
            [0; 4],
            self.max_size,
        )?);
        Ok(())
    }
    pub fn wait_for_recovery(&mut self, wait: bool) {
        self.recovery_wait = if wait {
            std::time::Duration::from_secs(12)
        } else {
            std::time::Duration::ZERO
        };
    }
    pub fn replace_recovery_wait(&mut self, wait: bool) -> bool {
        let previous = !self.recovery_wait.is_zero();
        self.wait_for_recovery(wait);
        previous
    }
    pub fn recovery_stats(&self) -> Option<(u64, u64, u64, usize)> {
        self.recovery.as_ref().map(|r| r.stats())
    }
    pub fn raw_group(&mut self, archive: u8, group: u32) -> Result<Vec<u8>> {
        #[cfg(feature = "custom-assets")]
        if self.package_only { return Err(Error::Invalid("package-only cache cannot request original assets")); }
        if let std::collections::hash_map::Entry::Vacant(e) = self.indexes.entry(archive) {
            e.insert(File::open(
                self.root.join(format!("main_file_cache.idx{archive}")),
            )?);
        }
        let data = cache::read_group_from(
            self.data
                .as_mut()
                .ok_or(Error::Invalid("cache data file unavailable"))?,
            self.indexes.get_mut(&archive).unwrap(),
            archive,
            group,
        )?;
        if data.len() > self.max_size.saturating_add(13) {
            return Err(Error::Invalid("cache group size limit"));
        }
        Ok(data)
    }
    fn master_index(&mut self) -> Result<&crate::cache_recovery::MasterIndex> {
        if self.master.is_none() {
            let recovery = self
                .recovery
                .as_ref()
                .ok_or(Error::Invalid("no live cache source"))?;
            let bytes = recovery
                .master(self.recovery_wait)?
                .ok_or(Error::PendingAsset {
                    archive: 255,
                    group: 255,
                })?;
            self.master = Some(crate::cache_recovery::MasterIndex::parse(
                &bytes,
                self.max_size,
            )?);
        }
        Ok(self.master.as_ref().unwrap())
    }
    pub fn has_archive(&mut self, archive: u8) -> Result<bool> {
        Ok(self.master_index()?.entry(archive).is_some())
    }
    pub fn reference(&mut self, archive: u8) -> Result<&ArchiveIndex> {
        if !self.references.contains_key(&archive) {
            let expected = if self.recovery.is_some() {
                Some(
                    self.master_index()?
                        .entry(archive)
                        .ok_or(Error::Invalid("archive absent from master index"))?,
                )
            } else {
                None
            };
            let disk = self.raw_group(255, u32::from(archive)).and_then(|bytes| {
                if let Some((crc, version)) = expected {
                    crate::cache_recovery::validate_reference(&bytes, crc, version, self.max_size)?;
                }
                let decoded = Container::parse(&bytes, self.max_size)?.decompress()?;
                Ok((ArchiveIndex::parse(&decoded)?, std::sync::Arc::new(bytes)))
            });
            let (index, bytes) = match disk {
                Ok(result) => result,
                Err(error) => {
                    let Some((crc, version)) = expected else {
                        return Err(error);
                    };
                    let bytes = self
                        .recovery
                        .as_ref()
                        .unwrap()
                        .reference(archive, crc, version, self.recovery_wait)?
                        .ok_or(Error::PendingAsset {
                            archive: 255,
                            group: u32::from(archive),
                        })?;
                    let decoded = Container::parse(&bytes, self.max_size)?.decompress()?;
                    (ArchiveIndex::parse(&decoded)?, bytes)
                }
            };
            self.references.insert(archive, index);
            self.reference_containers.insert(archive, bytes);
        }
        Ok(self.references.get(&archive).unwrap())
    }
    /// Login CRCs use the same validated references as group consumers.
    pub fn login_crcs(&mut self) -> Result<[u32; 23]> {
        let mut crcs = [0; 23];
        for archive in 0u8..23 {
            if archive == 16 {
                continue;
            }
            // GameIntProjection creates archive 22 with ui.ao (optional); an
            // absent master entry leaves its login checksum at zero.
            if archive == 22 && self.recovery.is_some() && !self.has_archive(archive)? {
                continue;
            }
            self.reference(archive)?;
            let bytes = self.reference_containers.get(&archive).unwrap();
            let container = Container::parse(bytes, self.max_size)?;
            let length = container.payload.len() + if container.compression == 0 { 5 } else { 9 };
            crcs[archive as usize] = crypto::crc32(&bytes[..length]);
        }
        Ok(crcs)
    }
    /// Admit one explicitly pinned, immutable package. The expected identity is
    /// supplied by the caller's content policy, never taken from the package.
    /// Original archive/file IDs cannot be replaced. Failure leaves no mount.
    #[cfg(feature = "custom-assets")]
    pub fn mount_package(&mut self, package: crate::asset_package::Package, expected: [u8; 32]) -> Result<()> {
        if self.package.is_some() || package.identity() != expected {
            return Err(Error::Invalid("asset package admission"));
        }
        self.check_package_ids(&package)?;
        self.package = Some(std::sync::Arc::new(package));
        self.package_checked = true;
        Ok(())
    }

    #[cfg(feature = "custom-assets")]
    fn check_package_ids(&mut self, package: &crate::asset_package::Package) -> Result<()> {
        if self.reference(7)?.groups.iter().any(|g| package.model(g.id).is_some()) {
            return Err(Error::Invalid("custom model replaces original ID"));
        }
        let definitions = self.reference(2)?.groups.iter().find(|g| g.id == 6)
            .ok_or(Error::Invalid("original object group absent"))?;
        if definitions.files.iter().any(|f| package.object(f.id).is_some()) {
            return Err(Error::Invalid("custom object replaces original ID"));
        }
        if package.terrain_ids().next().is_some()
            && self.reference(5)?.groups.iter().any(|g| package.terrain(g.id).is_some()) {
            return Err(Error::Invalid("custom terrain replaces original ID"));
        }
        if package.overlay_ids().next().is_some(){
            let originals=self.reference(2)?.groups.iter().find(|g|g.id==4).ok_or(Error::Invalid("original overlays absent"))?;
            if originals.files.iter().any(|f|package.overlay(f.id).is_some()){return Err(Error::Invalid("custom overlay replaces original ID"));}
        }
        if self.floor.is_some() { self.check_floor_ids()?; }
        if package.underlay_ids().next().is_some(){
            let originals=self.reference(2)?.groups.iter().find(|g|g.id==1).ok_or(Error::Invalid("original underlays absent"))?;
            if originals.files.iter().any(|f|package.underlay(f.id).is_some()) || (self.floor.is_some()&&package.underlay_ids().any(|id|(5801..=5807).contains(&id))){return Err(Error::Invalid("custom underlay replaces admitted ID"));}
        }
        Ok(())
    }

    #[cfg(feature = "custom-assets")]
    fn check_floor_ids(&mut self) -> Result<()> {
        if self.package_only { return Ok(()); }
        let underlays=self.reference(2)?.groups.iter().find(|g|g.id==1)
            .ok_or(Error::Invalid("original underlay group absent"))?;
        if underlays.files.iter().any(|f|(5801..=5807).contains(&f.id)) {
            return Err(Error::Invalid("authored material replaces original ID"));
        }
        Ok(())
    }

    /// Attach pinned geography only to an explicitly admitted package.
    #[cfg(feature = "custom-assets")]
    pub fn mount_floor(&mut self, mut floor: crate::authored_floor::Floor, expected: [u8;32]) -> Result<()> {
        if self.floor.is_some() || floor.identity()!=expected {return Err(Error::Invalid("authored floor admission"));}
        let package=self.package.clone().ok_or(Error::Invalid("authored floor requires package"))?;
        if package.underlay_ids().any(|id|(5801..=5807).contains(&id)){return Err(Error::Invalid("floor material conflicts with package underlay"));}
        floor.admit_package(&package)?;
        floor.validate_maps(self)?;
        if !self.package_only {self.check_package_ids(&package)?;}
        self.check_floor_ids()?;
        self.floor=Some(std::sync::Arc::new(floor));
        Ok(())
    }
    #[cfg(feature = "custom-assets")]
    pub(crate) fn authored_floor(&self) -> Option<std::sync::Arc<crate::authored_floor::Floor>> {self.floor.clone()}
    #[cfg(feature = "custom-assets")]
    fn authored_underlays() -> Vec<(u32,Vec<u8>)> {
        crate::authored_floor::COLORS.into_iter().enumerate().map(|(i,c)|
            (5801+i as u32,vec![1,(c>>16)as u8,(c>>8)as u8,c as u8,0])).collect()
    }

    /// Offline inspection only: no authentic cache is opened, and recovery
    /// cannot be enabled. A package never supplies login reference checksums.
    #[cfg(feature = "custom-assets")]
    pub fn open_package_only(root: impl AsRef<Path>, max_size: usize, expected: [u8; 32]) -> Result<Self> {
        let package = crate::asset_package::Package::read(root.as_ref())?;
        if package.identity() != expected { return Err(Error::Invalid("asset package admission")); }
        // Construct directly so even a dat2 file next to the package is ignored.
        Ok(Self {
            root: root.as_ref().to_owned(), data: None, indexes: HashMap::new(),
            references: HashMap::new(), reference_containers: HashMap::new(),
            master: None, max_size, recovery: None,
            recovery_wait: std::time::Duration::ZERO,
            package: Some(std::sync::Arc::new(package)), package_only: true, package_checked: true, floor: None,
        })
    }

    #[cfg(feature = "custom-assets")]
    pub fn package_identity(&self) -> Option<[u8; 32]> {
        self.package.as_ref().map(|p| p.identity())
    }
    #[cfg(feature="custom-assets")]
    pub fn admitted_terrain_ids(&self)->Vec<u32>{self.package.as_ref().map_or(Vec::new(),|p|p.terrain_ids().collect())}
    #[cfg(feature="custom-assets")]
    pub(crate) fn admitted_map_visuals(&self)->Result<Vec<(u32,bool)>>{
        let Some(p)=&self.package else{return Ok(vec![]);};let mut out=vec![];
        for id in p.object_ids(){let d=crate::object::Definition::decode_local(id,p.object(id).unwrap())?;
            if crate::environment_style::visual_only(&d,p){
                if d.interact_type!=0||d.clipped||d.animation.is_some()||d.transforms.is_some()||!d.operations.is_empty(){return Err(Error::Invalid("native map visual is not static noninteractive art"));}
                out.push((id,crate::environment_style::minimap_geometry(&d,p)));
            }
        }Ok(out)
    }

    pub fn group(&mut self, archive: u8, group: u32, key: [u32; 4]) -> Result<Vec<(u32, Vec<u8>)>> {
        #[cfg(feature = "custom-assets")]
        if !self.package_checked {
            if let Some(package) = self.package.clone() {
                self.check_package_ids(&package)?;
                self.package_checked = true;
            }
        }
        #[cfg(feature = "custom-assets")]
        if let Some(package) = &self.package {
            if archive == 7 {
                if let Some(bytes) = package.model(group) { return Ok(vec![(0, bytes.to_vec())]); }
            }
            if archive == 5 {
                if let Some(bytes) = package.terrain(group) { return Ok(vec![(0, bytes.to_vec()), (1, package.locations(group).unwrap_or(&[0]).to_vec())]); }
            }
            if self.package_only {
                return if archive == 2 && group == 6 {
                    Ok(package.object_ids().map(|id| (id, package.object(id).unwrap().to_vec())).collect())
                } else if archive == 2 && group == 1 && self.floor.is_some() {
                    Ok(Self::authored_underlays())
                } else if archive == 2 && group == 1 && package.underlay_ids().next().is_some() {
                    Ok(package.underlay_ids().map(|id|(id,package.underlay(id).unwrap().to_vec())).collect())
                } else if archive == 2 && group == 4 && self.floor.is_some() {
                    Ok(Vec::new())
                } else if archive==2 && group==4 && package.overlay_ids().next().is_some(){
                    Ok(package.overlay_ids().map(|id|(id,package.overlay(id).unwrap().to_vec())).collect())
                } else { Err(Error::Invalid("package-only cache cannot request original assets")) };
            }
        }
        let files = self.base_group(archive, group, key)?;
        #[cfg(feature = "custom-assets")]
        let files = {
            let mut files = files;
            if archive==2 && group==1 && self.floor.is_some() {files.extend(Self::authored_underlays());}
            if archive == 2 && group == 6 {
                if let Some(package) = &self.package {
                    files.extend(package.object_ids().map(|id| (id, package.object(id).unwrap().to_vec())));
                }
            }
            if archive==2 && group==1 {if let Some(package)=&self.package{files.extend(package.underlay_ids().map(|id|(id,package.underlay(id).unwrap().to_vec())));}}
            if archive==2 && group==4 {if let Some(package)=&self.package{files.extend(package.overlay_ids().map(|id|(id,package.overlay(id).unwrap().to_vec())));}}
            files
        };
        Ok(files)
    }

    fn base_group(&mut self, archive: u8, group: u32, key: [u32; 4]) -> Result<Vec<(u32, Vec<u8>)>> {
        let info = self
            .reference(archive)?
            .groups
            .iter()
            .find(|g| g.id == group)
            .ok_or(Error::Invalid("group not in reference table"))?
            .clone();
        let disk = self
            .raw_group(archive, group)
            .and_then(|bytes| decode_group(&bytes, &info, key, self.max_size, true));
        match (disk, &self.recovery) {
            (Ok(files), _) => Ok(files),
            (Err(error), None) => Err(error),
            (Err(_), Some(recovery)) => {
                let bytes = recovery
                    .group(archive, &info, self.recovery_wait)?
                    .ok_or(Error::PendingAsset { archive, group })?;
                decode_group(&bytes, &info, key, self.max_size, true)
            }
        }
    }
    /// GameIndexDataBase.bl requests every missing group before reporting readiness.
    /// Readiness validates the stored container; decoding belongs to file consumers.
    pub fn archive_ready(&mut self, archive: u8) -> Result<bool> {
        let groups = self.reference(archive)?.groups.clone();
        let mut ready = true;
        for wave in groups.chunks(crate::js5::MAX_BATCH) {
            let mut missing = Vec::new();
            for info in wave {
                let disk = self
                    .raw_group(archive, info.id)
                    .and_then(|bytes| validate_container(&bytes, info, true).map(|_| ()));
                if let Err(error) = disk {
                    if self.recovery.is_none() {
                        return Err(error);
                    }
                    missing.push(info.clone());
                }
            }
            if let Some(recovery) = &self.recovery {
                recovery.prefetch_groups(archive, &missing)?;
                let ids = missing.iter().map(|g| g.id).collect::<Vec<_>>();
                ready &= scan_archive(&ids, |group| {
                    let info = missing.iter().find(|g| g.id == group).unwrap();
                    Ok(recovery.group(archive, info, self.recovery_wait)?.is_some())
                })?;
            }
        }
        Ok(ready)
    }
    pub fn file(&mut self, archive: u8, group: u32, file: u32, key: [u32; 4]) -> Result<Vec<u8>> {
        self.group(archive, group, key)?
            .into_iter()
            .find(|(id, _)| *id == file)
            .map(|(_, bytes)| bytes)
            .ok_or(Error::Invalid("file not in group"))
    }
    /// ASCII names cover standard asset names without locale-dependent casing.
    pub fn named_file(
        &mut self,
        archive: u8,
        group_name: &str,
        file_name: &str,
    ) -> Result<Vec<u8>> {
        if !group_name.is_ascii() || !file_name.is_ascii() {
            return Err(Error::Unsupported("non-ASCII cache lookup name"));
        }
        let reference = self.reference(archive)?;
        let group = reference
            .groups
            .iter()
            .find(|g| g.name_hash == Some(name_hash(&group_name.to_ascii_lowercase())))
            .ok_or(Error::Invalid("unknown group name"))?;
        let file = group
            .files
            .iter()
            .find(|f| f.name_hash == Some(name_hash(&file_name.to_ascii_lowercase())))
            .ok_or(Error::Invalid("unknown file name"))?;
        let ids = (group.id, file.id);
        self.file(archive, ids.0, ids.1, [0; 4])
    }
}

fn validate_container<'a>(bytes: &'a [u8], info: &GroupInfo, disk: bool) -> Result<&'a [u8]> {
    let container = if disk {
        if bytes.len() < 7 {
            return Err(Error::Truncated);
        }
        // ui.fa tries the full revision first, then the legacy unsigned short.
        // The latter must equal the full reference revision, without truncation.
        // A two-byte trailer holding only the low 16 bits of a larger revision
        // (the OpenRS2/server layout) is therefore refused, as in the original
        // client, and the group is recovered over JS5 when that is enabled.
        let end4 = bytes.len() - 4;
        let revision4 = u32::from_be_bytes(bytes[end4..].try_into().unwrap());
        let end = if revision4 == info.version && crypto::crc32(&bytes[..end4]) == info.crc {
            end4
        } else {
            bytes.len() - 2
        };
        let revision = if end == end4 {
            revision4
        } else {
            u32::from(u16::from_be_bytes(bytes[end..].try_into().unwrap()))
        };
        if revision != info.version {
            return Err(Error::Invalid("cache group version mismatch"));
        }
        &bytes[..end]
    } else {
        bytes
    };
    if crypto::crc32(container) != info.crc {
        return Err(Error::Invalid("cache group CRC mismatch"));
    }
    Ok(container)
}

/// CRC covers the encrypted container; decrypt only after checking it. On disk,
/// a four-byte (or legacy two-byte) revision suffix is excluded from CRC.
pub fn decode_group(
    bytes: &[u8],
    info: &GroupInfo,
    key: [u32; 4],
    max_size: usize,
    disk: bool,
) -> Result<Vec<(u32, Vec<u8>)>> {
    let container = validate_container(bytes, info, disk)?;
    let mut decrypted = container.to_vec();
    if key != [0; 4] {
        if decrypted.len() < 5 {
            return Err(Error::Truncated);
        }
        crypto::xtea(&mut decrypted[5..], key, true);
    }
    let decoded = Container::parse(&decrypted, max_size)?.decompress()?;
    let files = cache::split_files(&decoded, info.files.len())?;
    Ok(info.files.iter().map(|f| f.id).zip(files).collect())
}

/// Gamepack lc.ao: Java signed CP1252 bytes with wrapping 31*h+byte.
pub fn name_hash(name: &str) -> u32 {
    crate::writer::encode_cp1252(name)
        .into_iter()
        .fold(0u32, |hash, b| {
            hash.wrapping_mul(31).wrapping_add(b as i8 as i32 as u32)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::FileInfo;
    #[test]
    fn encrypted_payload_checks_before_decrypting() {
        let key = [1, 2, 3, 4];
        let mut bytes = vec![0, 0, 0, 0, 11];
        bytes.extend_from_slice(b"hello world");
        crypto::xtea(&mut bytes[5..], key, false);
        let info = GroupInfo {
            id: 7,
            name_hash: None,
            crc: crypto::crc32(&bytes),
            version: 123,
            sizes: None,
            files: vec![FileInfo {
                id: 42,
                name_hash: None,
            }],
        };
        assert_eq!(
            decode_group(&bytes, &info, key, 100, false).unwrap(),
            vec![(42, b"hello world".to_vec())]
        );
        bytes.extend_from_slice(&123u16.to_be_bytes());
        assert_eq!(
            decode_group(&bytes, &info, key, 100, true).unwrap()[0].1,
            b"hello world"
        );
        bytes[6] ^= 1;
        assert_eq!(
            decode_group(&bytes, &info, key, 100, true),
            Err(Error::Invalid("cache group CRC mismatch"))
        );
        bytes[17] ^= 1;
        assert_eq!(
            decode_group(&bytes, &info, key, 100, true),
            Err(Error::Invalid("cache group version mismatch"))
        );
    }
}

#[cfg(test)]
mod readiness_tests {
    use super::*;
    #[test]
    fn original_archive_readiness_scans_every_missing_group() {
        let mut input = crate::buffer::Reader::new(include_bytes!(
            "../tests/fixtures/archive-readiness-jvm.bin"
        ));
        assert_eq!(input.u32().unwrap(), 64);
        let groups = [0, 2, 5];
        for _ in 0..64 {
            let mut cached = input.u32().unwrap();
            let complete = input.u32().unwrap();
            let expected_requests = input.u32().unwrap();
            let expected_ready = input.u8().unwrap() != 0;
            let expected_next_ready = input.u8().unwrap() != 0;
            let expected_next_requests = input.u32().unwrap();
            for (completion, requests, readiness) in [
                (complete, expected_requests, expected_ready),
                (7, expected_next_requests, expected_next_ready),
            ] {
                let mut requested = 0;
                let ready = scan_archive(&groups, |group| {
                    let bit = 1 << groups.iter().position(|&id| id == group).unwrap();
                    if cached & bit == 0 {
                        requested |= bit;
                        cached |= completion & bit;
                    }
                    Ok(cached & bit != 0)
                })
                .unwrap();
                assert_eq!((requested, ready), (requests, readiness));
            }
        }
        assert_eq!(input.remaining(), 0);
        assert!(scan_archive(&[], |_| panic!("empty archive requested data")).unwrap());
    }
    #[test]
    fn archive_readiness_preserves_real_errors() {
        let mut visited = Vec::new();
        let result = scan_archive(&[1, 2, 3], |group| {
            visited.push(group);
            if group == 2 {
                Err(Error::Invalid("malformed archive"))
            } else {
                Ok(false)
            }
        });
        assert_eq!(result, Err(Error::Invalid("malformed archive")));
        assert_eq!(visited, [1, 2]);
    }
}
