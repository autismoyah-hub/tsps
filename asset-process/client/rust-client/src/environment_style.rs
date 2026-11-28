//! Immutable render bindings for one explicitly admitted authored package.
//! Gameplay definitions and the authentic cache remain the source of operations.
use crate::{asset_package::Package, object::Definition, Error, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::Path,
};
pub const VISUAL_ONLY_PARAM: u32 = 0xffff00;
pub const MINIMAP_GEOMETRY_PARAM: u32 = 0xffff01;
pub struct Bindings {
    identity: [u8; 32],
    package: [u8; 32],
    bindings: Vec<(u32, u32, u8)>,
}
impl Bindings {
    pub fn read(path: &Path, expected: [u8; 32], package: &Package) -> Result<Self> {
        let meta = std::fs::symlink_metadata(path)?;
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 65536 {
            return Err(Error::Invalid("visual binding bounds/type"));
        }
        let mut bytes = Vec::new();
        File::open(path)?.take(65537).read_to_end(&mut bytes)?;
        Self::decode(&bytes, expected, package)
    }
    pub fn decode(bytes: &[u8], expected: [u8; 32], package: &Package) -> Result<Self> {
        if bytes.len() > 65536 || <[u8; 32]>::from(Sha256::digest(bytes)) != expected {
            return Err(Error::Invalid("visual binding admission"));
        }
        let text = std::str::from_utf8(bytes).map_err(|_| Error::Invalid("visual binding UTF8"))?;
        let mut lines = text.lines();
        if lines.next() != Some("CINDERWORKS_VISUAL_BINDINGS 1") {
            return Err(Error::Invalid("visual binding version"));
        }
        let mut seen = HashSet::new();
        let mut bindings = Vec::new();
        for line in lines.filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let v = line
                .split_whitespace()
                .map(str::parse::<u32>)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|_| Error::Invalid("visual binding number"))?;
            if v.len() != 3
                || !matches!(
                    v[0],
                    409 | 909
                        | 18491
                        | 57694
                        | 57692
                        | 37947
                        | 2097
                        | 9736
                        | 30124
                        | 16810
                        | 16811
                        | 18511
                        | 18512
                        | 18525
                        | 18526
                )
                || !(1241000..1242000).contains(&v[1])
                || !matches!(v[2], 0 | 4 | 10)
                || !seen.insert(v[0])
                || package.model(v[1]).is_none()
            {
                return Err(Error::Invalid("visual binding identity/model/shape"));
            }
            bindings.push((v[0], v[1], v[2] as u8));
        }
        Ok(Self {
            identity: expected,
            package: package.identity(),
            bindings,
        })
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn package_identity(&self) -> [u8; 32] {
        self.package
    }
    pub fn len(&self) -> usize {
        self.bindings.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
    /// Produce a separate rendering map. Original operations, sizes, collision
    /// flags and runtime object IDs are retained; the caller's map is untouched.
    pub fn for_render(
        &self,
        original: &HashMap<u32, Definition>,
    ) -> Result<HashMap<u32, Definition>> {
        self.into_render(original.clone())
    }
    /// Consume a private render-definition map without cloning all cache definitions.
    pub fn into_render(
        &self,
        mut defs: HashMap<u32, Definition>,
    ) -> Result<HashMap<u32, Definition>> {
        self.validate_definitions(&defs)?;
        for &(id, model, shape) in &self.bindings {
            Self::apply(defs.get_mut(&id).unwrap(),model,shape);
        }
        Ok(defs)
    }
    pub(crate) fn validate_definitions(&self,defs:&HashMap<u32,Definition>)->Result<()> {
        if self
            .bindings
            .iter()
            .any(|(id, _, _)| !defs.contains_key(id))
        {
            return Err(Error::Invalid("visual binding original definition missing"));
        }
        Ok(())
    }
    pub(crate) fn definition<'a>(&self,original:&'a Definition)->std::borrow::Cow<'a,Definition> {
        let Some(&(_,model,shape))=self.bindings.iter().find(|(id,_,_)|*id==original.id) else {
            return std::borrow::Cow::Borrowed(original);
        };
        let mut definition=original.clone();Self::apply(&mut definition,model,shape);
        std::borrow::Cow::Owned(definition)
    }
    fn apply(d:&mut Definition,model:u32,shape:u8) {
            d.models = vec![model];
            d.model_shapes = Some(vec![shape]);
            d.recolor.clear();
            d.retexture.clear();
            d.scale = [128; 3];
            d.offset = [0; 3];
            d.rotated = false;
            d.contour = -1;
            d.animation = None;
            d.transforms = None;
            d.transform_varbit = None;
            d.transform_varp = None;
    }
}
pub fn visual_only(def: &Definition, package: &Package) -> bool {
    package.object(def.id).is_some()
        && (64700..65300).contains(&def.id)
        && matches!(
            def.params.get(&VISUAL_ONLY_PARAM),
            Some(crate::object::Param::Int(1))
        )
}
pub fn minimap_geometry(def: &Definition, package: &Package) -> bool {
    package.object(def.id).is_some()
        && (64700..65300).contains(&def.id)
        && matches!(
            def.params.get(&MINIMAP_GEOMETRY_PARAM),
            Some(crate::object::Param::Int(1))
        )
}
