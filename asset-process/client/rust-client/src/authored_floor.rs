//! Explicit immutable authored geography; never reads environment variables.
//! Adapted from Cinderworks a31c18d2 scene.rs, preserving BROKEN_WATER_RETURN 1.
use crate::{Error, Result};
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::Path};

pub const ORIGIN: [i32; 2] = [10368, 64];
pub const SIDE: usize = 128;
pub const COLORS: [u32; 7] = [
    0x68685a, 0x55564e, 0x3e4444, 0x655b48, 0x495c4b, 0x2c505b, 0x282d2d,
];
const MAX_BYTES: usize = 2_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vertex {
    pub height: u16,
    pub underlay: u16,
    pub walkable: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub id: u32,
    pub tile: [u8; 2],
    pub shape: u8,
    pub rotation: u8,
}
pub struct Floor {
    motion: Option<crate::environment_motion::Mode>,
    identity: [u8; 32],
    vertices: Vec<Vertex>,
    placements: Vec<Placement>,
    bindings: Option<std::sync::Arc<crate::environment_style::Bindings>>,
    visual_ids: Vec<u32>,
    minimap_ids: Vec<u32>,
    cutaway: Option<std::sync::Arc<crate::environment_cutaway::Metadata>>,
    map_functions: Option<std::sync::Arc<crate::environment_minimap::MapFunctions>>,
}
impl Floor {
    pub fn with_motion(mut self,mode:crate::environment_motion::Mode)->Result<Self> {
        if self.motion.is_some(){return Err(Error::Invalid("floor motion already configured"));}
        self.motion=Some(mode);Ok(self)
    }
    pub(crate) fn motion(&self)->Option<(crate::environment_motion::Mode,[[u8;32];2])> {
        self.motion.zip(self.bindings.as_ref()).map(|(mode,b)|(mode,[self.identity,b.package_identity()]))
    }

    pub fn with_map_functions(mut self,functions:crate::environment_minimap::MapFunctions)->Result<Self> {
        if self.map_functions.is_some(){return Err(Error::Invalid("floor map functions already configured"));}
        self.map_functions=Some(std::sync::Arc::new(functions));Ok(self)
    }
    pub(crate) fn map_functions(&self)->Option<std::sync::Arc<crate::environment_minimap::MapFunctions>> {self.map_functions.clone()}
    pub(crate) fn validate_maps(&self,cache:&mut crate::disk_cache::DiskCache)->Result<()> {
        if let Some(functions)=&self.map_functions {functions.validate(cache)?;}Ok(())
    }

    pub fn with_cutaway(mut self,metadata:crate::environment_cutaway::Metadata)->Result<Self> {
        if self.cutaway.is_some(){return Err(Error::Invalid("floor cutaway already configured"));}
        self.cutaway=Some(std::sync::Arc::new(metadata));Ok(self)
    }
    pub(crate) fn cutaway(&self,base:[i32;2])->Result<crate::environment_cutaway::Cutaway> {
        self.cutaway.as_ref().map_or_else(||Ok(Default::default()),|m|m.for_scene(base))
    }

    pub fn with_bindings(mut self, bindings:crate::environment_style::Bindings)->Result<Self> {
        if self.bindings.is_some() {return Err(Error::Invalid("floor bindings already configured"));}
        self.bindings=Some(std::sync::Arc::new(bindings));Ok(self)
    }
    pub(crate) fn bindings(&self)->Option<std::sync::Arc<crate::environment_style::Bindings>> {self.bindings.clone()}
    pub(crate) fn visual_only_id(&self,id:u32)->bool {self.visual_ids.binary_search(&id).is_ok()}
    pub(crate) fn minimap_geometry_id(&self,id:u32)->bool {self.minimap_ids.binary_search(&id).is_ok()}
    pub(crate) fn admit_package(&mut self, package:&crate::asset_package::Package)->Result<()> {
        if self.map_functions.as_ref().is_some_and(|m|m.package_identity()!=package.identity()) || (self.map_functions.is_some()&&self.bindings.is_none()) {
            return Err(Error::Invalid("map functions require matching package and floor bindings"));
        }
        if self.cutaway.as_ref().is_some_and(|m|m.package_identity()!=package.identity()) || (self.cutaway.is_some()&&self.bindings.is_none()) {
            return Err(Error::Invalid("cutaway requires matching package and floor bindings"));
        }
        if self.placements.iter().any(|p|package.object(p.id).is_none()) {return Err(Error::Invalid("authored placement absent from admitted package"));}
        if let Some(bindings)=&self.bindings {
            if bindings.package_identity()!=package.identity() {return Err(Error::Invalid("floor bindings package identity"));}
            let mut visual=Vec::new();let mut minimap=Vec::new();
            for id in package.object_ids() {
                let def=crate::object::Definition::decode_local(id,package.object(id).unwrap())?;
                if crate::environment_style::visual_only(&def,package) {
                    if def.interact_type!=0 || def.blocks_projectiles || def.clipped || def.animation.is_some() || def.transforms.is_some()
                        || def.operations.iter().any(|o|!o.text.is_empty()) {return Err(Error::Invalid("visual-only object has gameplay state"));}
                    visual.push(id);
                }
                if crate::environment_style::minimap_geometry(&def,package) {minimap.push(id);}
            }
            self.visual_ids=visual;self.minimap_ids=minimap;
        }
        if self.motion.is_some() {
            if self.bindings.is_none(){return Err(Error::Invalid("motion requires admitted visual bindings"));}
            for p in self.placements.iter().filter(|p|(64950..=64956).contains(&p.id)) {
                let def=crate::object::Definition::decode_local(p.id,package.object(p.id).unwrap())?;
                if p.shape!=10||p.rotation!=0||def.size!=[1,1]||def.interact_type!=0||def.blocks_projectiles||def.animation.is_some()||def.transforms.is_some()||def.operations.iter().any(|o|!o.text.is_empty()) {
                    return Err(Error::Invalid("motion requires owned noninteractive static shape10"));
                }
            }
        }
        Ok(())
    }

    pub fn read(path: &Path, expected: [u8; 32]) -> Result<Self> {
        let metadata = std::fs::symlink_metadata(path)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > MAX_BYTES as u64
        {
            return Err(Error::Invalid("authored floor bounds/type"));
        }
        let mut bytes = Vec::new();
        File::open(path)?
            .take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        Self::decode(&bytes, expected)
    }
    pub fn decode(bytes: &[u8], expected: [u8; 32]) -> Result<Self> {
        if bytes.len() > MAX_BYTES || <[u8; 32]>::from(Sha256::digest(bytes)) != expected {
            return Err(Error::Invalid("authored floor admission"));
        }
        let text = std::str::from_utf8(bytes).map_err(|_| Error::Invalid("authored floor UTF8"))?;
        let mut lines = text.lines();
        if lines.next() != Some("BROKEN_WATER_RETURN 1") {
            return Err(Error::Invalid("authored floor version"));
        }
        let mut vertices = vec![None; SIDE * SIDE];
        let mut placements = Vec::new();
        for line in lines {
            let mut row = line.split_whitespace();
            let kind = row.next().ok_or(Error::Invalid("authored floor record"))?;
            let values = row
                .map(str::parse::<i32>)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|_| Error::Invalid("authored floor number"))?;
            if values.len() != 5 {
                return Err(Error::Invalid("authored floor row length"));
            }
            let v = values;
            match kind {
                "V" => {
                    if !(0..128).contains(&v[0])
                        || !(0..128).contains(&v[1])
                        || !(0..=1024).contains(&v[2])
                        || !(0..=1).contains(&v[4])
                        || (!(5802..=5808).contains(&v[3]) && !(v[3] == 0 && v[4] == 0))
                    {
                        return Err(Error::Invalid("authored floor vertex bounds"));
                    }
                    let index = v[0] as usize * SIDE + v[1] as usize;
                    if vertices[index].is_some() {
                        return Err(Error::Invalid("duplicate authored vertex"));
                    }
                    vertices[index] = Some(Vertex {
                        height: v[2] as u16,
                        underlay: v[3] as u16,
                        walkable: v[4] == 1,
                    });
                }
                "O" => {
                    if !(0..=65535).contains(&v[0])
                        || !(0..128).contains(&v[1])
                        || !(0..128).contains(&v[2])
                        || !(0..=22).contains(&v[3])
                        || !(0..4).contains(&v[4])
                    {
                        return Err(Error::Invalid("authored placement bounds"));
                    }
                    placements.push(Placement {
                        id: v[0] as u32,
                        tile: [v[1] as u8, v[2] as u8],
                        shape: v[3] as u8,
                        rotation: v[4] as u8,
                    });
                }
                _ => return Err(Error::Invalid("authored floor record kind")),
            }
        }
        let vertices = vertices
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or(Error::Invalid("incomplete authored floor"))?;
        Ok(Self {
            motion: None,
            identity: expected,
            vertices,
            placements,
            bindings: None, visual_ids: Vec::new(), minimap_ids: Vec::new(), cutaway: None, map_functions: None,
        })
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn vertex(&self, x: usize, z: usize) -> Option<Vertex> {
        if x < SIDE && z < SIDE {
            Some(self.vertices[x * SIDE + z])
        } else {
            None
        }
    }
    pub fn placements(&self) -> &[Placement] {
        &self.placements
    }
}
