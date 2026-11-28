//! research-live: one frame of the WORLD as the live client assembles it, without a server: a map location loaded by
//! the client's map loader, plus the state a server would have sent, put into ClientState by hand.
//!
//! usage: live_world CACHE OUT.bmp WORLD_X WORLD_Y YAW DISTANCE ELEVATION TARGET_UP [THING...]
//! THING = object=ID:X:Y:SHAPE:ROT                 an object the server spawned (added after the map is loaded)
//!       | npc=ID:X:Y:ANGLE[:SEQ:FRAME]            an NPC by id at a world tile; SEQ is the action animation
//!       | player=X:Y:GENDER:ANGLE:ITEM,ITEM,..[:SEQ:FRAME[:STAND:WALK]]   a player; the items are equipped by id; SEQ -1 =
//!                                                 no action; STAND and WALK are the movement sequences the server puts
//!                                                 in the appearance block for the held weapon (default 808 and 819)
//! Camera as in authored_scene_snapshot: the target is the world tile (may be fractional), yaw in 16384ths.
//!
//! What is the client's own code here:
//!   Scene::load_normal + WorldMesh::build            terrain and map objects (archive 5; objects by id, archive 2/6, 7)
//!   player_appearance::decode                         the appearance block BYTES are built here exactly as a server
//!                                                     writes them (gender, twelve equipment values: 0, kit + 256 or
//!                                                     item + 2048; colours; seven movement sequences; name ...) and
//!                                                     decoded by the client's packet decoder
//!   actor_sequence::State::set                        the action animation, as an animation update sets it
//!   world_actors::Actors::triangles                   NPCs and players assembled, lit, posed and submitted as in
//!                                                     the live frame (npc_triangles, player_triangles)
//!   render3d::Frame::draw_iter                        the classic rasteriser
//! What is NOT the live frame: live_runtime.rs additionally registers a traversal order of terrain, objects and actors
//! (object_submission) and draws interfaces; here scene and actors are drawn in one pass with the depth buffer and
//! world_actors::register_root_order. No packets are parsed except the appearance block; no network is opened.
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, identity_kit::Kit, npc_info::{Appearance as NpcAppearance, Npc},
    player_appearance, player_info::{Player, State}, player_model::Appearance, rebuild::Normal, render3d::Frame,
    scene::{PlacedObject, Scene}, texture, world_actors::Actors, world_mesh::WorldMesh,
};
use std::collections::HashMap;
type R<T> = Result<T, Box<dyn std::error::Error>>;

fn appearance_block(base: &Appearance, items: &gielinor_client::item::Definitions, ids: &[u32], name: &str, stand: u16, walk: u16) -> R<Vec<u8>> {
    // widget_appearance.rs Composition::equip: equipment[slot] = item + 2048; the item's 2nd and 3rd wear positions = 0
    let mut eq = base.equipment;
    for &id in ids {
        let d = items.items.get(&id).ok_or(format!("item {id} is not in the cache"))?;
        let slot = d.wear_positions[0];
        if slot < 0 { return Err(format!("item {id} declares no wear position").into()); }
        eq[slot as usize] = id + 2048;
        for &h in &d.wear_positions[1..] { if h >= 0 { eq[h as usize] = 0; } }
    }
    let mut b = vec![base.gender, 255, 255];
    let put = |b: &mut Vec<u8>, v: u32| { if v == 0 { b.push(0) } else { b.extend(((v as u16).to_be_bytes())) } };
    for v in eq { put(&mut b, v); }
    for v in base.equipment { put(&mut b, v); }
    b.extend(base.colors);
    for s in [stand, 823, walk, 820, 821, 822, 824] { b.extend(s.to_be_bytes()); }
    b.extend(name.bytes()); b.push(0);
    b.push(3); b.extend(0u16.to_be_bytes()); b.push(0); b.extend(0u16.to_be_bytes());
    b.extend([0, 0, 0]); b.push(0);
    Ok(b)
}

fn main() -> R<()> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 9 { return Err("CACHE OUT.bmp WORLD_X WORLD_Y YAW DISTANCE ELEVATION TARGET_UP [object=..|npc=..|player=..]".into()); }
    let p: Vec<f32> = a[3..9].iter().map(|v| v.parse()).collect::<Result<_, _>>()?;
    let mut client = ClientState::load(DiskCache::open(&a[1], 64 << 20)?)?;
    let rebuild = Normal { world_area: -1, zone_x: (p[0] as u16) / 8, zone_y: (p[1] as u16) / 8, players: None };
    let mut scene = Scene::load_normal(&mut client.scripts.source.cache, &rebuild)?;
    let (bx, by) = (scene.base_x, scene.base_y);
    let kits = Kit::load_all(&mut client.scripts.source.cache)?;
    let sequences = gielinor_client::animation::Sequence::load_all(&mut client.scripts.source.cache)?;
    let (mut players, _) = State::initialize(&[0; 4608], 1)?;
    let mut high = Vec::new();
    let mut spawned = 0;
    for thing in &a[9..] {
        let (kind, v) = thing.split_once('=').ok_or("THING")?;
        let f = v.split(':').collect::<Vec<_>>();
        match kind {
            "object" => {
                let n = f.iter().map(|x| x.parse::<i32>()).collect::<Result<Vec<_>, _>>()?;
                scene.objects.push(PlacedObject { initial_index: None, rejected: false, runtime: true, id: n[0] as u32, plane: 0,
                    x: (n[1] - bx) as u8, y: (n[2] - by) as u8, shape: n[3] as u8, rotation: n[4] as u8 });
                spawned += 1;
            }
            "npc" => {
                let id: u32 = f[0].parse()?;
                let def = client.npcs.get(&id).ok_or(format!("NPC {id} is not in the cache"))?.clone();
                let mut path = gielinor_client::actor_path::Path::default();
                path.teleport([f[1].parse()?, f[2].parse()?], def.size as i32);
                let mut npc = Npc { visual: Default::default(), path, sequence: Default::default(), pose: Default::default(),
                    idle_fallback: Default::default(), spots: Default::default(), overlays: Default::default(), turn: Default::default(),
                    facing: None, animations: def.animations, freezes: Default::default(), exact: None, id: id as u16,
                    position: [f[1].parse()?, f[2].parse()?], angle: f[3].parse()?, spawn_cycle: None, steps: vec![], jump: false,
                    blocks: vec![], appearance: NpcAppearance::default() };
                npc.turn.angle = f[3].parse()?; npc.turn.target = f[3].parse()?;
                if f.len() >= 6 { npc.sequence.set(f[4].parse()?, 0, 0, &sequences)?; npc.sequence.frame = f[5].parse()?; }
                let index = 1 + client.npc_state.npcs.len() as u16;
                client.npc_state.npcs.insert(index, npc);
                client.npc_state.indices.push(index);
                println!("npc {id} {:?}: models {:?}, stand {}, walk {}", def.name, def.models, def.animations[0], def.animations[3]);
            }
            "player" => {
                let (x, y, gender): (i32, i32, u8) = (f[0].parse()?, f[1].parse()?, f[2].parse()?);
                let ids = f[4].split(',').filter(|s| !s.is_empty()).map(|s| s.parse::<u32>()).collect::<Result<Vec<_>, _>>()?;
                let index = 1 + high.len();
                let bytes = appearance_block(&Appearance::standard(&kits, gender), &client.items, &ids, &format!("Player{index}"),
                    f.get(7).map_or(Ok(808), |v| v.parse())?, f.get(8).map_or(Ok(819), |v| v.parse())?)?;
                let update = player_appearance::decode(&bytes, &client.items)?;
                println!("player {index} appearance block {} bytes -> equipment {:?}", bytes.len(), update.appearance.equipment);
                let mut pl = Player::at([0, x, y], Some(update), 1, None);
                pl.turn.angle = f[3].parse()?; pl.turn.target = f[3].parse()?;
                if f.len() >= 7 && f[5] != "-1" { pl.sequence.set(f[5].parse()?, 0, 0, &sequences)?; pl.sequence.frame = f[6].parse()?; }
                if index == 1 { client.data.position = [0, x, y]; }
                players.players[index] = Some(pl);
                high.push(index as u16);
            }
            _ => return Err("THING".into()),
        }
    }
    players.high = high;
    client.player_state = Some(players);
    client.preferences.render_self = true;
    let definitions = gielinor_client::object::Definition::load_all(&mut client.scripts.source.cache)?;
    if spawned > 0 { scene.add_object_collision(&definitions)?; }
    let mesh = WorldMesh::build(&scene, &mut client.scripts.source.cache, 0)?;
    if mesh.pending_assets { return Err("assets pending".into()); }
    let mut actors = Actors::load(&mut client.scripts.source.cache)?;
    let (mut tris, pending) = actors.triangles(&mut client, &scene, &mesh.item_support)?;
    for w in &pending { println!("pending: {w}"); }
    gielinor_client::model_order::rebase(&mut tris, mesh.triangles.len());
    let local = [((p[0] - bx as f32) * 128.) as i32, ((p[1] - by as f32) * 128.) as i32];
    let floor_height = scene.height_fine(0, local[0], local[1]).ok_or("target height missing")? as f32;
    let yaw = p[2] * std::f32::consts::TAU / 16384.0;
    let (x, z) = (local[0] as f32, local[1] as f32);
    let target = [x, floor_height - p[5], z];
    let mut camera = gielinor_client::render3d::Camera::look_at([x + yaw.sin() * p[3], target[1] - p[4], z - yaw.cos() * p[3]], target, 1000.)?;
    camera.far = 30000;
    let mut textures = HashMap::new();
    for (id, d) in texture::Definition::load_all(&mut client.scripts.source.cache)? {
        textures.insert(id, texture::Texture::load(&mut client.scripts.source.cache, d, 128, 0.8)?);
    }
    let palette = gielinor_client::floor::palette(0.8)?;
    let mut frame = Frame::new(1280, 900)?;
    gielinor_client::world_actors::register_root_order(&mut frame, &camera, &client, &scene, &tris);
    frame.draw_iter(&camera, mesh.visible_triangles(0).chain(tris.iter()), &palette, &textures);
    std::fs::write(&a[2], frame.raster()?.bmp()?)?;
    println!("live_world: base ({bx}, {by}); {} map + spawned objects, {} scene faces, {} actor faces; missing map squares {:?}",
        scene.objects.len(), mesh.triangles.len(), tris.len(), scene.missing_regions);
    Ok(())
}
