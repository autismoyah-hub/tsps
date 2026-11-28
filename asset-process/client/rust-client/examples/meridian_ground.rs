//! Flat-tile ground comparison. CACHE OUT.bmp item:ID def:FILE+model:FILE item:ID
use gielinor_client::{disk_cache::DiskCache, item::{Definition,Definitions}, item_model::Models,
    model::Model, render3d::{Frame,Triangle,Camera,texture_uv},texture,floor};
use std::collections::HashMap;
type R<T> = Result<T,Box<dyn std::error::Error>>;
fn place(
    out: &mut Vec<Triangle>,
    model: &Model,
    colors: &[[i32; 3]],
    position: [i32; 3],
    angle: u16,
    tag: u64,
    single_tile: bool,
) -> R<()> {
    // Ground-pile slope alignment has already run, but its output is still
    // quantized by GameModel.by just like animated actor vertices.
    let start = out.len();
    let vertices = gielinor_client::model_draw::vertices(&model.vertices, angle, position)?;
    let bounds = gielinor_client::model_bounds::Bounds::new(&model.vertices, position).with_selection(&model.vertices, angle, single_tile);
    for (i, face) in model.faces.iter().enumerate() {
        if colors[i][2] == -2 {
            continue;
        }
        let color = if colors[i][2] == -1 {
            [colors[i][0]; 3]
        } else {
            colors[i]
        };
        let positions = face.map(|v| vertices[v as usize]);
        let texture = model
            .face_textures
            .as_ref()
            .and_then(|t| (t[i] != -1).then_some(t[i] as u16));
        let map = model
            .texture_coords
            .as_ref()
            .and_then(|t| {
                if t[i] == -1 {
                    None
                } else {
                    model.texture_triangles.get(t[i] as u8 as usize)
                }
            })
            .unwrap_or(face);
        out.push(Triangle {
            #[cfg(feature="enhanced-renderer")] material_hsl:Some(model.colors[i]), texture_mapping: Some(gielinor_client::render3d::TextureMapping { vertices: map.map(|v| vertices[v as usize]), local: Some(map.map(|v| model.vertices[v as usize])) }),
            local_vertices: Some(face.map(|v| model.vertices[v as usize])),
            model_bounds: Some(bounds),
                model_order: Some(gielinor_client::model_order::Submission::new(model,i,start)), depth_bias: model.face_depth_bias(i),
            view: 0,
            vertices: positions,
            colors: color,
                        flat: colors[i][2] == -1,
            uv: texture_uv(map.map(|v| vertices[v as usize]), positions),
            texture,
            alpha: model.face_alpha(i),
            near_alpha: Some(model.alpha.as_ref().map_or(0, |a| a[i])),
            tag,
        });
    }
    if out.len() == start && single_tile && tag != 0 {
        let mut marker = Triangle::selection_only(bounds, tag);
        marker.model_order = Some(gielinor_client::model_order::Submission {
            actor_spot: None, source_ordinal: None, temporary_id: None, scene_height_offset: 0,
            mode: gielinor_client::model_order::Mode::Auto, priority: model.priority,
            has_priorities: false, has_alpha: model.alpha.is_some(), batch: start,
        });
        out.push(marker);
    }
    Ok(())
}

fn bmp(w: usize, h: usize, px: &[u32]) -> Vec<u8> {
    let row = (w * 3 + 3) / 4 * 4;
    let mut o = Vec::with_capacity(54 + row * h);
    o.extend(b"BM"); o.extend(((54 + row * h) as u32).to_le_bytes()); o.extend([0u8; 4]); o.extend(54u32.to_le_bytes());
    o.extend(40u32.to_le_bytes()); o.extend((w as i32).to_le_bytes()); o.extend((h as i32).to_le_bytes());
    o.extend(1u16.to_le_bytes()); o.extend(24u16.to_le_bytes()); o.extend([0u8; 24]);
    for y in (0..h).rev() {
        for x in 0..w { let p = px[y * w + x]; o.extend([(p & 255) as u8, (p >> 8 & 255) as u8, (p >> 16 & 255) as u8]); }
        o.extend(std::iter::repeat(0u8).take(row - w * 3));
    }
    o
}

fn main()->R<()>{
    let args=std::env::args().collect::<Vec<_>>();
    if args.len()!=6 {return Err("CACHE OUT.bmp SPEC SPEC SPEC".into());}
    let mut cache=DiskCache::open(&args[1],64<<20)?;
    let defs=Definitions::load(&mut cache)?;
    let mut models=Models::default();
    let palette=floor::palette(0.8)?;
    let mut textures=HashMap::new();
    for (id,d) in texture::Definition::load_all(&mut cache)? {
        textures.insert(id,texture::Texture::load(&mut cache,d,128,0.8)?);
    }
    let mut widest=0.0_f32;
    if let Some((_,path))=args[4].split_once("+model:"){
        for v in Model::decode(&std::fs::read(path)?)?.vertices {
            widest=widest.max(v[0].abs()).max(v[2].abs());
        }
    }
    let focal=if widest>65. {495.} else {700.};
    let camera=Camera::look_at([28.,-278.,-340.],[0.,-4.,0.],focal)?;
    let mut frame=Frame::new(766,210)?;
    frame.clear(0x282c29);
    let mut tris=Vec::new();
    for x in -4..=4 {for z in -3..=3 {
        let mut tile=Model::combine(&[])?;
        tile.vertices=vec![[-64.,0.,-64.],[64.,0.,-64.],[64.,0.,64.],[-64.,0.,64.]];
        tile.faces=vec![[0,1,2],[0,2,3]]; tile.colors=vec![0;2];
        let h=18*1024+2*128+38+(x+z+8)%2*2;
        place(&mut tris,&tile,&[[h,h,-1];2],[128*x,0,128*z],0,0,false)?;
    }}
    frame.draw(&camera,&tris,&palette,&textures);
    let mut before=frame.pixels.clone();
    for (i,s) in args[3..].iter().enumerate(){
        let (d,m)=if let Some(id)=s.strip_prefix("item:"){
            let id=id.parse()?;
            (defs.items[&id].clone(),models.inventory(&mut cache,&defs,id,1,None)?)
        }else{
            let (d,m)=s.strip_prefix("def:").and_then(|a|a.split_once("+model:")).ok_or("bad SPEC")?;
            let d=Definition::decode_local(60000,&std::fs::read(d)?)?;
            let mut m=Model::decode(&std::fs::read(m)?)?;
            m.scale(d.scale);
            for &(a,b) in &d.recolor {m.recolor(a,b);}
            for &(a,b) in &d.retexture {m.retexture(a,b);}
            (d,m)
        };
        let colors=m.light(64+d.ambient,768+d.contrast,[-50,-10,-50])?;
        let mut obj=Vec::new();
        place(&mut obj,&m,&colors,[(i as i32-1)*128,0,0],0,i as u64+1,true)?;
        frame.draw(&camera,&obj,&palette,&textures);
        let changed=frame.pixels.iter().zip(&before).filter(|(a,b)|a!=b).count();
        println!("{s}: {changed} changed ground pixels");
        if changed==0 {return Err("invisible ground item".into());}
        before.clone_from(&frame.pixels);
    }
    std::fs::write(&args[2],bmp(766,210,&frame.pixels))?;
    Ok(())
}

