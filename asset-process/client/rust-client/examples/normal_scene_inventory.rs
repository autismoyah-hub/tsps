//! Read-only reference-cache static scene inventory. No authored package or game login.
use gielinor_client::{disk_cache::DiskCache, object::Definition, rebuild::Normal, scene::Scene};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Write,
    path::Path,
};

fn reports(
    scene: &Scene,
    definitions: &HashMap<u32, Definition>,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut placements = String::from(
        "world_x\tworld_z\tplane\tobject_id\tshape\trotation\trejected\tname_escaped\tmodels\n",
    );
    let mut counts = BTreeMap::<u32, usize>::new();
    for object in &scene.objects {
        let d = definitions
            .get(&object.id)
            .ok_or("placement definition missing")?;
        *counts.entry(object.id).or_default() += 1;
        writeln!(
            placements,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\t{:?}",
            scene.base_x + i32::from(object.x),
            scene.base_y + i32::from(object.y),
            object.plane,
            object.id,
            object.shape,
            object.rotation,
            object.rejected,
            d.name,
            d.models
        )?;
    }
    let mut inventory =
        String::from("object_id\tcount\tname_escaped\tmodels\tanimation\ttransforms\n");
    for (id, count) in counts {
        let d = &definitions[&id];
        writeln!(
            inventory,
            "{id}\t{count}\t{:?}\t{:?}\t{:?}\t{:?}",
            d.name, d.models, d.animation, d.transforms
        )?;
    }
    Ok((placements, inventory))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};
    let a = std::env::args().skip(1).collect::<Vec<_>>();
    if !(a.len() == 4 || (a.len() == 5 && a[4].starts_with("--js5="))) {
        return Err("REFERENCE_CACHE ZONE_X ZONE_Z NEW_OUTPUT_DIRECTORY [--js5=ADDRESS]".into());
    }
    let zone = [a[1].parse::<u16>()?, a[2].parse::<u16>()?];
    if zone.iter().any(|v| !(6..=2041).contains(v)) {
        return Err("zone coordinates must keep the complete scene within the 14-bit world".into());
    }
    let output = Path::new(&a[3]);
    if output.exists() {
        return Err("output directory must be new".into());
    }
    let mut cache = if a.len() == 5 {
        let mut c = DiskCache::open_remote(&a[0], 64 << 20, a[4][6..].parse()?)?;
        c.wait_for_recovery(true);
        c
    } else {
        DiskCache::open(&a[0], 64 << 20)?
    };
    let crcs = cache.login_crcs()?;
    let rebuild = Normal {
        world_area: -1,
        zone_x: zone[0],
        zone_y: zone[1],
        players: None,
    };
    let scene = Scene::load_normal(&mut cache, &rebuild)?;
    let definitions = Definition::load_all(&mut cache)?;
    let (placements, inventory) = reports(&scene, &definitions)?;
    let mut groups = String::from("region\tfile\tcrc\tversion\tdecoded_sha256\n");
    let mut referenced = 0;
    for id in rebuild.regions() {
        if let Some(info) = cache
            .reference(5)?
            .groups
            .iter()
            .find(|g| g.id == id)
            .cloned()
        {
            for (file, bytes) in cache.group(5, id, [0; 4])? {
                writeln!(
                    groups,
                    "{id}\t{file}\t{:08x}\t{}\t{:x}",
                    info.crc,
                    info.version,
                    Sha256::digest(bytes)
                )?;
            }
            referenced += 1;
        }
    }
    if referenced == 0 {
        return Err("no referenced regions in requested scene".into());
    }
    if crcs != cache.login_crcs()? {
        return Err("reference CRCs changed during inspection".into());
    }
    let scope=format!("REFERENCE-CACHE STATIC NORMAL SCENE INVENTORY; NOT LIVE GAMEPLAY\nzone={zone:?}\nscene_base={},{}\nregions={:?}\nmissing_regions={:?}\nplacements={}\nreference_crcs={crcs:?}\nexplicit_js5_recovery={}\nactors=false\nui=false\nanimation=false\ntransforms_resolved=false\n",scene.base_x,scene.base_y,rebuild.regions(),scene.missing_regions,scene.objects.len(),a.len()==5);
    std::fs::create_dir(output)?;
    for (name, text) in [
        ("placements.tsv", placements),
        ("inventory.tsv", inventory),
        ("region-identities.tsv", groups),
        ("scope.txt", scope),
    ] {
        std::fs::write(output.join(name), text)?;
    }
    println!("Recorded {} static placements from {referenced} referenced regions; {} absent regions reported",scene.objects.len(),scene.missing_regions.len());
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_preserves_order_coordinates_and_escapes_names() {
        let mut scene = Scene::empty_sized(1000, 2000, 12, 12).unwrap();
        for (id, plane, x, y) in [(9, 2, 3, 4), (2, 0, 5, 6), (9, 1, 7, 8)] {
            scene.objects.push(gielinor_client::scene::PlacedObject {
                initial_index: None,
                rejected: false,
                runtime: false,
                id,
                plane,
                x,
                y,
                shape: 10,
                rotation: 3,
            });
        }
        let mut first = Definition::decode(9, &[0], false, true).unwrap();
        first.name = "A\tB\nC".into();
        first.models = vec![17];
        let second = Definition::decode(2, &[0], false, true).unwrap();
        let defs = HashMap::from([(9, first), (2, second)]);
        let a = reports(&scene, &defs).unwrap();
        assert_eq!(a, reports(&scene, &defs).unwrap());
        assert_eq!(a.0.lines().count(), 4);
        assert_eq!(a.1.lines().count(), 3);
        assert_eq!(
            a.0.lines().nth(1).unwrap(),
            "1003\t2004\t2\t9\t10\t3\tfalse\t\"A\\tB\\nC\"\t[17]"
        );
        assert!(a.1.lines().nth(1).unwrap().starts_with("2\t1\t"));
        assert!(a.1.lines().nth(2).unwrap().starts_with("9\t2\t"));
        assert!(reports(&scene, &HashMap::new()).is_err());
        assert_eq!(scene.objects.len(), 3);
    }
}
