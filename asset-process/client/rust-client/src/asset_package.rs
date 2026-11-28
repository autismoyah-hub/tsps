//! Explicit, immutable packages for original custom assets. Reading a package
//! never opens or changes an authentic cache and grants no network authority.
use crate::{Error, Result};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs::File, io::Read, path::Path};

const MAX_REGISTRY: usize = 65_536;
const MAX_FILE: usize = 16 << 20;
const MAX_PACKAGE: usize = 256 << 20;

pub struct Package {
    models: BTreeMap<u32, Vec<u8>>,
    objects: BTreeMap<u32, Vec<u8>>,
    terrain: BTreeMap<u32, Vec<u8>>,
    locations: BTreeMap<u32, Vec<u8>>,
    underlays: BTreeMap<u32, Vec<u8>>,
    overlays: BTreeMap<u32, Vec<u8>>,
    identity: [u8; 32],
}

fn bounded_read(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > limit as u64 {
        return Err(Error::Invalid("asset file bounds/type"));
    }
    // Limit the actual read too: metadata alone cannot bound a growing file.
    let mut bytes = Vec::new();
    File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(Error::Invalid("asset file bounds"));
    }
    Ok(bytes)
}

impl Package {
    /// Read an explicitly selected local package. CRCs detect damage; they do
    /// not authenticate a publisher. A mounting caller must separately admit
    /// the full content identity and the package's owned ID namespace.
    pub fn read(root: &Path) -> Result<Self> {
        let index = bounded_read(&root.join("registry.txt"), MAX_REGISTRY)?;
        let text =
            std::str::from_utf8(&index).map_err(|_| Error::Invalid("asset registry UTF8"))?;
        let mut lines = text.lines();
        if lines.next() != Some("TOWER_ASSET_PACKAGE 1") {
            return Err(Error::Invalid("asset registry version"));
        }
        let mut package = Self {
            models: BTreeMap::new(),
            objects: BTreeMap::new(),
            terrain: BTreeMap::new(),
            locations: BTreeMap::new(),
            underlays: BTreeMap::new(),
            overlays: BTreeMap::new(),
            identity: [0; 32],
        };
        // Versioned identity: exact registry, followed by length-delimited
        // payload bytes in registry order. No paths outside the package enter it.
        let mut hash = Sha256::new();
        hash.update(b"GIELINOR_ASSET_PACKAGE_CONTENT_V1\0");
        hash.update((index.len() as u64).to_be_bytes());
        hash.update(&index);
        let mut total = index.len();
        for line in lines {
            let row: Vec<_> = line.split_whitespace().collect();
            if row.len() != 4
                || row[2].is_empty()
                || !row[2]
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
                || row[2].contains("..")
            {
                return Err(Error::Invalid("asset registry row/path"));
            }
            let id: u32 = row[1].parse().map_err(|_| Error::Invalid("asset ID"))?;
            let crc = u32::from_str_radix(row[3], 16).map_err(|_| Error::Invalid("asset CRC"))?;
            let map = match row[0] {
                "model" => &mut package.models,
                "object" if id <= 65_535 => &mut package.objects,
                "terrain" if id <= 65_535 => &mut package.terrain,
                "locations" if id <= 65_535 => &mut package.locations,
                "underlay" if (5801..=5832).contains(&id) => &mut package.underlays,
                "overlay" if (5801..=5832).contains(&id) => &mut package.overlays,
                _ => return Err(Error::Invalid("asset kind/ID")),
            };
            if map.contains_key(&id) {
                return Err(Error::Invalid("duplicate asset ID"));
            }
            let bytes = bounded_read(&root.join(row[2]), MAX_FILE.min(MAX_PACKAGE - total))?;
            total += bytes.len();
            if crate::crypto::crc32(&bytes) != crc {
                return Err(Error::Invalid("asset checksum"));
            }
            match row[0] {
                "model" => {
                    crate::model::Model::decode(&bytes)?;
                }
                "object" => {
                    crate::object::Definition::decode_local(id, &bytes)?;
                }
                "terrain" => {
                    crate::terrain::Region::decode(&bytes, ((id>>8)*64) as i32, ((id&255)*64) as i32)?;
                }
                "underlay" => {crate::floor::Underlay::decode(&bytes)?;}
                "overlay" => {crate::floor::Overlay::decode(&bytes)?;}
                "locations" => {crate::terrain::locations(&bytes,65536)?;}
                _ => unreachable!(),
            }
            hash.update((bytes.len() as u64).to_be_bytes());
            hash.update(&bytes);
            map.insert(id, bytes);
        }
        if package.terrain.is_empty() && (package.models.is_empty() || package.objects.is_empty()) {
            return Err(Error::Invalid("empty asset package"));
        }
        if package.locations.keys().any(|id|!package.terrain.contains_key(id)){return Err(Error::Invalid("locations require admitted terrain"));}
        for bytes in package.locations.values(){
            if crate::terrain::locations(bytes,65536)?.iter().any(|p|!package.objects.contains_key(&p.object)){return Err(Error::Invalid("locations require owned objects"));}
        }
        package.identity = hash.finalize().into();
        Ok(package)
    }

    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn model(&self, id: u32) -> Option<&[u8]> {
        self.models.get(&id).map(Vec::as_slice)
    }
    pub fn object(&self, id: u32) -> Option<&[u8]> {
        self.objects.get(&id).map(Vec::as_slice)
    }
    pub fn terrain(&self, id: u32) -> Option<&[u8]> {
        self.terrain.get(&id).map(Vec::as_slice)
    }
    pub fn model_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.models.keys().copied()
    }
    pub fn object_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.objects.keys().copied()
    }
    pub fn terrain_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.terrain.keys().copied()
    }
    pub fn locations(&self,id:u32)->Option<&[u8]>{self.locations.get(&id).map(Vec::as_slice)}
    pub fn underlay(&self,id:u32)->Option<&[u8]>{self.underlays.get(&id).map(Vec::as_slice)}
    pub fn underlay_ids(&self)->impl Iterator<Item=u32>+'_ {self.underlays.keys().copied()}
    pub fn overlay(&self,id:u32)->Option<&[u8]>{self.overlays.get(&id).map(Vec::as_slice)}
    pub fn overlay_ids(&self)->impl Iterator<Item=u32>+'_ {self.overlays.keys().copied()}
}
