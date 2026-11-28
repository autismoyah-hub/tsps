//! research-creatures, lighting: print the client's own lit colours for an NPC's rest mesh, face by face.
//!
//! usage: creature_light CACHE ENTRY OUT.txt [seq=ID:FRAME]
//! ENTRY = npc:ID                         built by npc_model::mesh, lit as world_actors.rs:457 does:
//!                                        light(64 + ambient, 850 + contrast, [-30, -50, -30])
//!       | file:PATH[;ambient=A;contrast=C]   one model file of ours; contrast as STORED (x5 applied here)
//! OUT.txt:  h ambient contrast            (the two numbers given to Model::light)
//!           v x y z label                 rest vertices
//!           f a b c colour render alpha texture c0 c1 c2     c* = Model::light's result for the face
//! with seq=ID:FRAME also, after animation of a COPY (Animations::apply_observed), before scale:
//!           p x y z                       posed vertices
//!           r c0 c1 c2                    what Model::light WOULD give on the posed mesh (the client never does this)
use gielinor_client::{
    actor_sequence::State, animation_runtime::Animations, disk_cache::DiskCache, item_model::Models, model::Model,
    npc::Definition, npc_model,
};
use std::fmt::Write as _;
type R<T> = Result<T, Box<dyn std::error::Error>>;
fn main() -> R<()> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 4 { return Err("CACHE ENTRY OUT.txt [seq=ID:FRAME]".into()); }
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let mut models = Models::default();
    let (model, ambient, contrast, label): (Model, i32, i32, String) = if let Some(id) = a[2].strip_prefix("npc:") {
        let defs = Definition::load_all(&mut cache)?;
        let d = defs.get(&id.parse()?).ok_or("no such NPC")?;
        let m = npc_model::mesh(&mut cache, &mut models, d, false, None)?.ok_or("no models")?;
        (m, 64 + d.ambient, 850 + d.contrast, format!("npc {} {:?} models {:?}", d.id, d.name, d.models))
    } else if let Some(rest) = a[2].strip_prefix("file:") {
        let mut bits = rest.split(';');
        let path = bits.next().unwrap();
        let (mut am, mut co) = (0i32, 0i32);
        for b in bits {
            let (k, v) = b.split_once('=').ok_or("k=v")?;
            match k { "ambient" => am = v.parse()?, "contrast" => co = v.parse::<i32>()? * 5, _ => return Err("ambient= or contrast=".into()) }
        }
        (Model::decode(&std::fs::read(path)?)?, 64 + am, 850 + co, format!("file {path}"))
    } else { return Err("npc:ID or file:PATH".into()); };
    let colors = model.light(ambient, contrast, [-30, -50, -30])?;
    let mut t = String::new();
    writeln!(t, "# {label}: {} vertices {} faces", model.vertices.len(), model.faces.len())?;
    writeln!(t, "h {ambient} {contrast}")?;
    for (i, v) in model.vertices.iter().enumerate() {
        writeln!(t, "v {} {} {} {}", v[0] as i32, v[1] as i32, v[2] as i32, model.vertex_skins.as_ref().map_or(0, |s| s[i]))?;
    }
    for (i, f) in model.faces.iter().enumerate() {
        writeln!(t, "f {} {} {} {} {} {} {} {} {} {}", f[0], f[1], f[2], model.colors[i],
            model.render_types.as_ref().map_or(0, |r| r[i]), model.alpha.as_ref().map_or(0, |r| r[i]),
            model.face_textures.as_ref().map_or(-1, |r| r[i]), colors[i][0], colors[i][1], colors[i][2])?;
    }
    if let Some(s) = a.get(4).and_then(|s| s.strip_prefix("seq=")) {
        let (id, fr) = s.split_once(':').ok_or("seq=ID:FRAME")?;
        let (id, fr): (u32, i32) = (id.parse()?, fr.parse()?);
        let mut animations = Animations::load(&mut cache)?;
        let mut m = model.clone();
        let state = State { id: id as i32, frame: fr, ..Default::default() };
        let applied = animations.apply_observed(&mut cache, &mut m, id, &state.clock())?;
        writeln!(t, "# posed: {applied:?}")?;
        for v in &m.vertices { writeln!(t, "p {} {} {}", v[0] as i32, v[1] as i32, v[2] as i32)?; }
        let relit = m.light(ambient, contrast, [-30, -50, -30])?;
        for c in &relit { writeln!(t, "r {} {} {}", c[0], c[1], c[2])?; }
    }
    std::fs::write(&a[3], t)?;
    println!("{label}: {} vertices {} faces, light({ambient}, {contrast}, [-30,-50,-30])", model.vertices.len(), model.faces.len());
    Ok(())
}
