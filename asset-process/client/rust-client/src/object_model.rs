//! Object shape selection and transforms (GameObjectComposition.ao).
use crate::{disk_cache::DiskCache, model::Model, object::Definition, Error, Result};
use std::{collections::HashMap, sync::Arc};
#[cfg(not(feature="custom-assets"))]
type ModelKey = (u32,u8,u8);
// A functional definition and its authored visual can share the original ID.
// Keep exact recipes (not only a digest), while leaving default Java keys intact.
#[cfg(feature="custom-assets")]
#[derive(Hash,PartialEq,Eq)]
struct ModelKey {
    identity:(u32,u8,u8), models:Vec<u32>, shapes:Option<Vec<u8>>,
    rotated:bool, scale:[u16;3], offset:[i16;3],
    recolor:Vec<(u16,u16)>, retexture:Vec<(u16,u16)>,
}
#[derive(Default)]
pub struct Models {
    raw: HashMap<u32, Arc<Model>>,
    pending: bool,
    transformed: HashMap<ModelKey, Arc<Model>>,
}
fn query_ready(
    def: &Definition,
    shape: u8,
    mut query: impl FnMut(u32) -> Result<bool>,
) -> Result<bool> {
    let mut ready = true;
    for id in def.model_ids(shape) {
        ready &= query(id)?;
    }
    Ok(ready)
}
fn placement_shape(shape: u8) -> u8 {
    match shape {
        11 => 10,
        5..=8 => 4,
        _ => shape,
    }
}
fn group_ready(cache: &mut DiskCache, id: u32) -> Result<bool> {
    match cache.group(7, id, [0; 4]) {
        Ok(_) => Ok(true),
        Err(Error::PendingAsset { .. }) => Ok(false),
        Err(error) => Err(error),
    }
}
/// wq.at/ow.zc: do not mutate a pending placement until its shape's models are ready.
pub fn ready(cache: &mut DiskCache, def: &Definition, shape: u8) -> Result<bool> {
    query_ready(def, placement_shape(shape), |id| group_ready(cache, id))
}
impl Models {
    /// Whether any object lookup in this mesh build encountered pending data.
    pub fn pending(&self) -> bool {
        self.pending
    }
    pub fn object(
        &mut self,
        cache: &mut DiskCache,
        def: &Definition,
        shape: u8,
        rotation: u8,
    ) -> Result<Option<Arc<Model>>> {
        #[cfg(not(feature="custom-assets"))]
        let key = (def.id, shape, rotation);
        #[cfg(feature="custom-assets")]
        let key = ModelKey { identity:(def.id,shape,rotation), models:def.models.clone(),
            shapes:def.model_shapes.clone(),rotated:def.rotated,scale:def.scale,offset:def.offset,
            recolor:def.recolor.clone(),retexture:def.retexture.clone() };
        if let Some(model) = self.transformed.get(&key) {
            return Ok(Some(model.clone()));
        }
        let ids = def.model_ids(shape);
        if ids.is_empty() {
            return Ok(None);
        }
        if !query_ready(def, shape, |id| {
            if self.raw.contains_key(&id) {
                Ok(true)
            } else {
                group_ready(cache, id)
            }
        })? {
            self.pending = true;
            return Ok(None);
        }
        let mirrored = if def.model_shapes.is_some() {
            def.rotated ^ (rotation > 3)
        } else {
            def.rotated ^ (shape == 2 && rotation > 3)
        };
        let mut parts = Vec::new();
        for id in ids {
            let model = if let Some(m) = self.raw.get(&id) {
                m.clone()
            } else {
                let model = Arc::new(Model::decode(&cache.file(7, id, 0, [0; 4])?)?);
                self.raw.insert(id, model.clone());
                model
            };
            let mut m = (*model).clone();
            if mirrored {
                m.mirror();
            }
            parts.push(m);
        }
        let mut model = if parts.len() == 1 {
            parts.remove(0)
        } else {
            Model::combine(&parts)?
        };
        if shape == 4 && rotation > 3 {
            model.rotate(256)?;
            model.translate([45, 0, -45]);
        }
        model.rotate_quarter(rotation);
        for &(a, b) in &def.recolor {
            model.recolor(a, b);
        }
        for &(a, b) in &def.retexture {
            model.retexture(a, b);
        }
        model.scale(def.scale.map(i32::from));
        model.translate(def.offset.map(i32::from));
        let model = Arc::new(model);
        self.transformed.insert(key, model.clone());
        Ok(Some(model))
    }
}

#[cfg(test)]
mod readiness_tests {
    use super::*;
    #[cfg(feature="custom-assets")]
    #[test]
    fn authored_and_original_recipes_with_same_id_do_not_alias() {
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_file(self.0.join("main_file_cache.dat2"));let _=std::fs::remove_dir(&self.0);}}
        let root=std::env::temp_dir().join(format!("gielinor-model-recipes-{}",std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let _cleanup=Cleanup(root.clone());
        std::fs::write(root.join("main_file_cache.dat2"),[]).unwrap();
        let mut cache=DiskCache::open(&root,1024).unwrap();
        let mut raw=Model::decode(&[0;18]).unwrap();
        raw.vertices=vec![[20.,-30.,40.],[0.,0.,0.],[10.,10.,10.]];
        raw.faces=vec![[0,1,2]];raw.colors=vec![1234];raw.face_textures=Some(vec![7]);
        let mut models=Models::default();
        models.raw.insert(1,Arc::new(raw.clone()));
        raw.vertices[0]=[80.,-10.,60.];models.raw.insert(2,Arc::new(raw));
        let mut original=Definition::decode(909,&[0],false,true).unwrap();
        original.models=vec![1];
        let first=models.object(&mut cache,&original,10,0).unwrap().unwrap();
        let mut bound=original.clone();bound.models=vec![2];bound.scale=[256,128,128];
        bound.offset=[5,0,-5];bound.recolor=vec![(1234,4321)];bound.retexture=vec![(7,9)];
        let rendered=models.object(&mut cache,&bound,10,0).unwrap().unwrap();
        assert_eq!(rendered.vertices[0],[165.,-10.,55.]);
        assert_eq!(rendered.colors,vec![4321]);assert_eq!(rendered.face_textures,Some(vec![9]));
        assert!(Arc::ptr_eq(&first,&models.object(&mut cache,&original,10,0).unwrap().unwrap()));
        assert!(Arc::ptr_eq(&rendered,&models.object(&mut cache,&bound,10,0).unwrap().unwrap()));
        let mut irrelevant=bound.clone();irrelevant.interact_type=0;irrelevant.blocks_projectiles=false;
        assert!(Arc::ptr_eq(&rendered,&models.object(&mut cache,&irrelevant,10,0).unwrap().unwrap()));
        for kind in 0..7 {
            let mut variant=original.clone();
            match kind {
                0=>variant.models=vec![2],1=>variant.rotated=true,
                2=>variant.scale=[256,128,128],3=>variant.offset=[5,0,-5],
                4=>variant.recolor=vec![(1234,4321)],5=>variant.retexture=vec![(7,9)],
                _=>variant.model_shapes=Some(vec![4]),
            }
            let mut fresh=Models::default();fresh.raw=models.raw.clone();
            let expected=fresh.object(&mut cache,&variant,10,0).unwrap();
            let actual=models.object(&mut cache,&variant,10,0).unwrap();
            match (expected,actual) {
                (Some(expected),Some(actual))=>{
                    assert_eq!(actual.vertices,expected.vertices,"recipe {kind}");
                    assert_eq!(actual.colors,expected.colors,"recipe {kind}");
                    assert_eq!(actual.face_textures,expected.face_textures,"recipe {kind}");
                    assert!(!Arc::ptr_eq(&first,&actual),"recipe {kind}");
                },(None,None)=>{},_=>panic!("shape admission mismatch {kind}"),
            }
        }
    }
    #[test]
    fn original_object_and_placement_readiness_queries() {
        let mut r = crate::buffer::Reader::new(include_bytes!(
            "../tests/fixtures/object-readiness-jvm.bin"
        ));
        assert_eq!(r.u32().unwrap(), 1840);
        for _ in 0..1840 {
            let variant = r.u8().unwrap();
            let mask = r.u8().unwrap();
            let shape = r.u8().unwrap();
            let placement = r.u8().unwrap() != 0;
            let expected = r.u8().unwrap() != 0;
            let count = r.u32().unwrap();
            let expected_queries: Vec<_> = (0..count).map(|_| r.u32().unwrap()).collect();
            let mut def = Definition::decode(0, &[0], false, true).unwrap();
            def.models = if variant == 1 || variant == 2 {
                vec![1, 3, 5]
            } else {
                vec![]
            };
            def.model_shapes = match variant {
                2 => Some(vec![4, 10, 4]),
                4 => Some(vec![]),
                _ => None,
            };
            let mut queries = Vec::new();
            let result = query_ready(
                &def,
                if placement {
                    placement_shape(shape)
                } else {
                    shape
                },
                |id| {
                    queries.push(id);
                    let bit = match id {
                        1 => 0,
                        3 => 1,
                        5 => 2,
                        _ => panic!("unexpected model"),
                    };
                    Ok(mask & (1 << bit) != 0)
                },
            )
            .unwrap();
            assert_eq!(
                (result, queries),
                (expected, expected_queries),
                "variant{variant} mask{mask} shape{shape} placement{placement}"
            );
        }
        assert_eq!(r.remaining(), 0);
    }
}
