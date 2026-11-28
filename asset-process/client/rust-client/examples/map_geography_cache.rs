use gielinor_client::map_composite;
use gielinor_client::map_geography;
use gielinor_client::{disk_cache::DiskCache, Result};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let maps = cache.group(19, 1, [0; 4])?;
    let (mut files, mut tiles, mut objects) = (0, 0, 0);
    for (id, bytes) in maps {
        let map = map_composite::Composite::decode(&bytes, true)?;
        let destinations: std::collections::BTreeSet<_> = map
            .regions
            .keys()
            .copied()
            .chain(map.chunks.iter().map(|c| c.region.destination))
            .collect();
        for dest in destinations {
            let group = ((dest[0] as u32) << 8) | dest[1] as u32;
            let bytes = cache.file(18, group, id, [0; 4])?;
            let geography = map_geography::Geography::decode(&bytes, &map, dest).map_err(|e| {
                eprintln!("map {id} region {dest:?} bytes {}", bytes.len());
                e
            })?;
            files += 1;
            for tile in geography.tiles {
                tiles += usize::from(tile.underlay != 0);
                objects += tile
                    .layers
                    .iter()
                    .map(|p| p.decorations.len())
                    .sum::<usize>();
            }
        }
    }
    println!("{files} geography files, {tiles} nonempty underlays, {objects} decorations");
    Ok(())
}
