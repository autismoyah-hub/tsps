//! View a translated mesh in the local client's classic software rasterizer.
use gielinor_client::{model::Model, model_bounds::Bounds, model_order::{Mode, Submission}, floor,
    render3d::{Camera, Frame, Triangle, TextureMapping, texture_uv}, texture::{Definition, Texture}};
use std::{collections::HashMap, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let root=PathBuf::from(&args[1]);
    let m=Model::decode(&std::fs::read(root.join("model.bin"))?)?;
    let light=m.light(64,850,[-30,-50,-30])?;
    let projected=std::fs::read(root.join("projected.uv")).unwrap_or_default();
    if !projected.is_empty() && projected.len()!=24*m.faces.len(){return Err("invalid UV stream".into());}
    let mut textures=HashMap::new();
    for id in m.face_textures.iter().flatten().filter(|&&t| t>=0) {
        if textures.contains_key(&(*id as u16)){continue;}
        let raw=std::fs::read(root.join(format!("t{}.rgba",id)))?;
        if raw.len()!=128*128*4 {return Err("invalid image".into());}
        let opaque=raw.chunks_exact(4).all(|p|p[3]==255);
        let pixels=raw.chunks_exact(4).map(|p|if p[3]<128 {0} else {
            let gamma=|c:u8| ((c as f64/256.).powf(0.8)*256.) as u32;
            ((gamma(p[0])<<16)|(gamma(p[1])<<8)|gamma(p[2])).max(1)
        }).collect();
        textures.insert(*id as u16,Texture{definition:Definition{sprite:0,average_hsl:0,opaque,direction:0,speed:0},size:128,pixels});
    }
    let mid:[f32;3]=std::array::from_fn(|i| {
        let axis=||m.vertices.iter().map(|p|p[i] as i32);
        (axis().min().unwrap_or(0)+axis().max().unwrap_or(0)) as f32/2.
    });
    let radius=m.vertices.iter().map(|p|(0..3).map(|i|(p[i] as f32-mid[i]).powi(2)).sum::<f32>().sqrt()).fold(1.,f32::max);
    let scale=(100./radius).min(16.);
    let palette=floor::palette(0.8)?;
    let camera=Camera::look_at([0.;3],[0.,0.,1.],512.)?;
    for (j,(yaw,elev)) in [(0_f32,0_f32),(90.,0.),(180.,0.),(270.,0.),(0.,89.9),(35.,22.)].into_iter().enumerate(){
        let (s,c)=yaw.to_radians().sin_cos();
        let (t,b)=elev.to_radians().sin_cos();
        let v:Vec<_>=m.vertices.iter().map(|p|{
            let [x,y,z]=std::array::from_fn(|i|(p[i] as f32-mid[i])*scale);
            let z=z*c-x*s;
            [x*c+(p[2] as f32-mid[2])*scale*s,y*b+z*t,475.+z*b-y*t]
        }).collect();
        let mut tris=Vec::new();
        // Camera coordinates are normalized to a 100-unit sphere for repeatable framing.
        let bounds=Bounds::new(&[[-101.0;3],[101.0;3]],[0,0,475]);
        for (i,f) in m.faces.iter().enumerate(){
            if light[i][2]==-2{continue;}
            let points=f.map(|k|v[k as usize]);
            let coord=m.texture_coords.as_ref().map_or(-1,|l|l[i]);
            let basis=if coord != -1 && m.texture_types[coord as u8 as usize]==0 {
                m.texture_triangles[coord as u8 as usize]
            }else{*f};
            let map=basis.map(|k|v[k as usize]);
            let complex=coord!= -1 && m.texture_types[coord as u8 as usize]!=0 && !projected.is_empty();
            let uv=if complex {
                std::array::from_fn(|k|std::array::from_fn(|j|{
                    let p=24*i+8*k+4*j; f32::from_le_bytes(projected[p..p+4].try_into().unwrap())
                }))
            }else{texture_uv(map,points)};
            tris.push(Triangle{texture_mapping:(!complex).then_some(TextureMapping{vertices:map,local:None}),
                local_vertices:None,model_bounds:Some(bounds),model_order:Some(Submission{mode:Mode::Priority,..Submission::new(&m,i,0)}),
                depth_bias:m.face_depth_bias(i),view:0,vertices:points,
                colors:if light[i][2]==-1{[light[i][0];3]}else{light[i]},flat:light[i][2]==-1,
                uv,texture:m.face_textures.as_ref().and_then(|l|(l[i]>=0).then_some(l[i] as u16)),
                alpha:m.face_alpha(i),near_alpha:Some(m.alpha.as_ref().map_or(0,|l|l[i])),tag:1});
        }
        let mut frame=Frame::new(256,256)?;
        frame.clear(0x262a30);
        frame.draw(&camera,&tris,&palette,&textures);
        let rgb:Vec<u8>=frame.pixels.iter().flat_map(|p|[(p>>16) as u8,(p>>8) as u8,*p as u8]).collect();
        std::fs::write(root.join(format!("{}.rgb",j)),rgb)?;
    }
    Ok(())
}
