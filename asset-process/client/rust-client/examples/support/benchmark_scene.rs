//! Bounded static scene recipes referring only to an admitted package.
use gielinor_client::{asset_package::Package,model::Model};
use sha2::{Digest,Sha256};
use std::{fmt::Write,io::Read,path::Path};
type Result<T>=std::result::Result<T,Box<dyn std::error::Error>>;
#[derive(Debug,PartialEq)]
pub struct Part { pub id:u32, pub offset:[i32;3], pub quarter:u8 }
pub struct Recipe { pub parts:Vec<Part>, pub ambient:i32, pub contrast:i32, identity:String }
pub struct Loaded { pub model:Model, pub owners:Vec<u64>, pub ambient:i32, pub contrast:i32, pub scope:String }
impl Recipe {
    pub fn read(path:&Path)->Result<Self> {
        let file=std::fs::File::open(path)?;
        if !file.metadata()?.is_file(){return Err("scene must be a regular file".into());}
        let mut bytes=Vec::new();file.take(16385).read_to_end(&mut bytes)?;
        if bytes.len()>16384{return Err("scene exceeds 16384 bytes".into());}
        Self::parse(std::str::from_utf8(&bytes)?)
    }
    fn parse(text:&str)->Result<Self> {
        let mut parts=Vec::new();let mut light=None;
        for line in text.lines().map(str::trim).filter(|l|!l.is_empty()&&!l.starts_with('#')) {
            let f:Vec<_>=line.split_whitespace().collect();
            match f.first().copied() {
                Some("light") if f.len()==3 && light.is_none()=>{
                    let ambient=f[1].parse::<i32>()?;let contrast=f[2].parse::<i32>()?;
                    if !(0..=255).contains(&ambient)||!(1..=4096).contains(&contrast){return Err("scene light bounds".into());}
                    light=Some((ambient,contrast));
                },
                Some("model") if f.len()==6 && parts.len()<64=>{
                    let id=f[1].parse()?;
                    let offset=[f[2].parse::<i32>()?,f[3].parse()?,f[4].parse()?];
                    let quarter=f[5].parse::<u8>()?;
                    if offset.iter().any(|v|!(-1_000_000..=1_000_000).contains(v))||quarter>3{return Err("scene transform bounds".into());}
                    parts.push(Part{id,offset,quarter});
                },
                _=>return Err("expected bounded model ID X Y Z QUARTER or one light AMBIENT CONTRAST".into())
            }
        }
        if parts.is_empty(){return Err("empty scene".into());}
        let (ambient,contrast)=light.unwrap_or((64,768));
        Ok(Self{parts,ambient,contrast,identity:format!("{:x}",Sha256::digest(text.as_bytes()))})
    }
    pub fn load(&self,package:&Package)->Result<Loaded> {
        let mut models=Vec::new();let mut owners=Vec::new();let(mut vertices,mut faces)=(0usize,0usize);
        let mut scope=format!("scene_manifest_sha256={}\nscene_parts={}\nambient={}\ncontrast={}\n",self.identity,self.parts.len(),self.ambient,self.contrast);
        for (index,part) in self.parts.iter().enumerate() {
            let bytes=package.model(part.id).ok_or("scene model absent from pinned package")?;
            let mut model=Model::decode(bytes)?;
            if model.vertices.is_empty()||model.faces.is_empty(){return Err("empty scene model".into());}
            if model.face_textures.as_ref().is_some_and(|t|t.iter().any(|&v|v>=0)){return Err("static scene supports untextured models only".into());}
            vertices=vertices.checked_add(model.vertices.len()).ok_or("scene vertex budget")?;
            faces=faces.checked_add(model.faces.len()).ok_or("scene face budget")?;
            if vertices>262144||faces>1_000_000{return Err("scene input geometry budget".into());}
            model.rotate_quarter(part.quarter);model.translate(part.offset);
            owners.extend(std::iter::repeat_n(index as u64+1,model.faces.len()));
            writeln!(scope,"part={index},model={},sha256={:x},offset={:?},quarter={}",part.id,Sha256::digest(bytes),part.offset,part.quarter)?;
            models.push(model);
        }
        // Preserve the source's combination before lighting, including shared normals.
        // Model::combine remains responsible for the native u16 vertex-index limit.
        let model=Model::combine(&models)?;
        if model.faces.len()!=owners.len(){return Err("combined face ownership mismatch".into());}
        Ok(Loaded{model,owners,ambient:self.ambient,contrast:self.contrast,scope})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scene_declarations_preserve_order_transforms_and_lighting() {
        let r=Recipe::parse("# owned package IDs only\nmodel 7 -128 0 256 3\nlight 32 512\nmodel 7 128 -20 0 1\n").unwrap();
        assert_eq!(r.parts,vec![Part{id:7,offset:[-128,0,256],quarter:3},Part{id:7,offset:[128,-20,0],quarter:1}]);
        assert_eq!((r.ambient,r.contrast),(32,512));
        let d=Recipe::parse("model 7 0 0 0 0").unwrap();assert_eq!((d.ambient,d.contrast),(64,768));
        assert_ne!(d.identity,r.identity);
    }
    #[test]
    fn scene_declarations_reject_raw_paths_unbounded_and_ambiguous_input() {
        for s in ["","# empty","owned model.bin 0 0 0 0 -","model 7 0 0 0 4","model 7 1000001 0 0 0","model 7 NaN 0 0 0","model 7 0 0 0 0 extra","light 256 768\nmodel 7 0 0 0 0","light 64 0\nmodel 7 0 0 0 0","light 64 768\nlight 64 768\nmodel 7 0 0 0 0"]{assert!(Recipe::parse(s).is_err(),"{s}");}
        assert!(Recipe::parse(&"model 7 0 0 0 0\n".repeat(65)).is_err());
        assert_eq!(Recipe::parse(&"model 7 0 0 0 0\n".repeat(64)).unwrap().parts.len(),64);
    }
}
