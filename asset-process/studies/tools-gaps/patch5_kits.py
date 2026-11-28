import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from patchlib import Patch
P = Patch(_AP.fix(r'<CLIENT_SRC>\examples\asset_pose.rs'))
rep = P.rep
rep("""//!        asset_pose CACHE info SEQ|item:ID|skeleton:ID [...]   (print what each is; draws nothing)
""", """//!        asset_pose CACHE info SEQ|item:ID|skeleton:ID|kits [...]   (print what each is; draws nothing)
//!        `info kits` lists the body kits of the standard figure this tool draws (Appearance::standard: for each
//!        body part the lowest-numbered enabled kit of the gender), slot by slot: kit id, model ids, the kit's own
//!        recolours, the size and label histogram of the kit mesh, and the player recolours the figure applies.
""")
rep("""    model::Model, player_colors, player_model::Appearance, raster::Raster,""",
    """    model::Model, player_colors, player_model::{Appearance, KIT_SLOTS}, raster::Raster,""")
rep("""    for id in ids {
        if let Some(item) = id.strip_prefix("item:") {""", """    for id in ids {
        if id == "kits" {
            let kits = Kit::load_all(cache)?;
            let standard = standard_figure(&kits);
            println!("standard figure: gender {} colors {:?} equipment {:?}", standard.gender, standard.colors, standard.equipment);
            for i in 0..5 {
                let index = standard.colors[i] as usize;
                if let Some(&to) = player_colors::COLORS[i].get(index) {
                    println!("player_recolor part={i} from={} to={to}", player_colors::BASE[i]);
                }
                if let Some(&to) = player_colors::SECONDARY_COLORS[i].get(index) {
                    println!("player_recolor part={i} from={} to={to}", player_colors::SECONDARY_BASE[i]);
                }
            }
            for (part, &slot) in KIT_SLOTS.iter().enumerate() {
                let value = standard.equipment[slot];
                if !(256..2048).contains(&value) {
                    println!("kit slot={slot} name={} part={part} id=none", SLOT_NAMES[slot]);
                    continue;
                }
                let kit = kits.get(&(value - 256)).ok_or("standard kit")?;
                let m = kit.mesh(cache, &mut models, false)?.ok_or("standard kit has no mesh")?;
                println!("kit slot={slot} name={} part={part} id={} models={:?} recolor={:?} retexture={:?} vertices={} faces={}",
                    SLOT_NAMES[slot], kit.id, kit.models, kit.recolor, kit.retexture, m.vertices.len(), m.faces.len());
                println!("    vertex labels {}", histogram(m.vertex_skins.as_ref(), m.vertices.len()));
                println!("    face labels   {}", histogram(m.face_skins.as_ref(), m.faces.len()));
                match &m.priorities {
                    Some(p) => println!("    face priorities {}", histogram(Some(p), m.faces.len())),
                    None => println!("    face priorities uniform {}", m.priority),
                }
            }
            continue;
        }
        if let Some(item) = id.strip_prefix("item:") {""")
P.save()
