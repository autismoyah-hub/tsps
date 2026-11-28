//! Asset system tool: one standard figure per entry, wearing real items and our models, drawn as the game
//! lights worn gear.
//! usage: asset_figure CACHE OUT.bmp [zoom=N] [draw=actor|depth|auto|widget] ENTRY [ENTRY...]
//! draw=actor (default) paints the figure as the live client paints a player in the world (priority order, no
//! depth test between the figure's own faces); draw=depth is the old per-pixel depth picture; see asset_pose.rs
//! and support/actor_draw.rs.
//! Each entry is drawn from three angles, one column per entry. Entries are parsed and assembled by the code
//! in asset_pose.rs (see its header for the full grammar and for how the client hides the body):
//! ENTRY = PIECE[+PIECE...]   all pieces on one figure
//! PIECE = none | item:ID | item:ID@SLOT[+HIDE...|+nohide] | model:PATH | model:PATH@SLOT[+HIDE...]
//! SLOT  = 0 head, 1 cape, 2 amulet, 3 weapon, 4 body, 5 shield, 6 arms, 7 legs, 8 hair, 9 hands, 10 feet, 11 jaw
//! item:ID alone uses the slot and hidden slots the item definition declares (weapon slot when it declares
//! none); model:PATH alone is merged after the body and hides nothing, as before.
#[allow(dead_code)]
#[path = "asset_pose.rs"]
mod pose;
use gielinor_client::{
    disk_cache::DiskCache, identity_kit::Kit, interface_model, item::Definitions, item_icon::Icons,
    item_model::Models, raster::Raster,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 4 {
        return Err("CACHE OUT.bmp [zoom=N] [draw=actor|depth|auto|widget] ENTRY [ENTRY...]".into());
    }
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let kits = Kit::load_all(&mut cache)?;
    let items = Definitions::load(&mut cache)?;
    let mut models = Models::default();
    let icons = Icons::load(&mut cache, 0.8)?;
    // an entry of the form zoom=N sets the camera (smaller is closer; default 520); cells grow to match
    let mut zoom = 520i32;
    let mut list = Vec::new();
    let mut draw = pose::actor_draw::Draw::Actor;
    let mut painter = pose::actor_draw::Painter::load(&mut cache)?;
    for e in &a[3..] {
        if let Some(z) = e.strip_prefix("zoom=") { zoom = z.parse()?; }
        else if let Some(v) = e.strip_prefix("draw=") { draw = pose::actor_draw::Draw::parse(v)?; } else { list.push(e.clone()); }
    }
    let entries = &list;
    println!("draw={draw:?}");
    let (cw, ch) = if zoom < 400 { (600usize, 800usize) } else { (300usize, 400usize) };
    let yaws = [192, 704, 1728];
    let mut raster = Raster::new(cw * entries.len(), ch * yaws.len(), 1 << 25)?;
    raster.clear(0xff302b25);
    let standard = pose::standard_figure(&kits);
    for (k, e) in entries.iter().enumerate() {
        let o = pose::outfit(e, &standard, &items, &mut models, &mut cache)?;
        for n in &o.notes { println!("{e}: {n}"); }
        let mut player = standard.clone();
        player.equipment = o.equipment;
        let m = pose::assemble(&o, &player, &mut cache, &mut models, &kits, &items)?;
        let colors = m.light(64, 850, [-30, -50, -30])?;
        for (r, yaw) in yaws.iter().enumerate() {
            let vertices = interface_model::vertices(&m, [0, *yaw, 0], [0, 0], zoom, true)?;
            // height centring shifts the model down by half its height; at pitch 0 the origin's view depth is zoom
            let origin = interface_model::depth_origin(&m, 0, [0, 0], zoom, true);
            let (sprite, warning) = painter.draw(&mut cache, &icons, &m, &colors, &vertices, origin, [cw, ch], draw)?;
            if let Some(w) = warning { if r == 0 { println!("{e}: WARNING {w}"); } }
            sprite.draw(&mut raster, (k * cw) as i32, (r * ch) as i32, 256)?;
        }
        println!("{e}: {} vertices {} faces", m.vertices.len(), m.faces.len());
    }
    std::fs::write(&a[2], raster.bmp()?)?;
    Ok(())
}
