//! Read-only acceptance of legacy/full-width disk revisions and live JS5 groups.
use gielinor_client::{
    disk_cache::{decode_group, DiskCache},
    js5,
};
use std::time::Duration;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("expected CACHE_DIRECTORY IP:PORT".into());
    }
    let mut cache = DiskCache::open(&args[0], 64 << 20)?;
    let mut network = js5::Client::connect(
        args[1].parse()?,
        239,
        [0; 4],
        Duration::from_secs(10),
        64 << 20,
    )?;
    let mut count = 0;
    let mut stale = 0;
    for archive in [0, 1, 2, 7, 12, 14, 15, 22] {
        let groups: Vec<_> = cache
            .reference(archive)?
            .groups
            .iter()
            .filter(|g| !g.files.is_empty())
            .take(4)
            .cloned()
            .collect();
        for info in groups {
            let disk = cache.raw_group(archive, info.id)?;
            let downloaded = network.fetch(archive, u16::try_from(info.id)?)?;
            let expected = decode_group(&downloaded, &info, [0; 4], 64 << 20, false)?;
            match decode_group(&disk, &info, [0; 4], 64 << 20, true) {
                Ok(files) => assert_eq!(files, expected),
                Err(error) => {
                    stale += 1;
                    println!("archive {archive} group {} disk rejected: {error:?}; expected revision {} tail {:?}", info.id, info.version, &disk[disk.len().saturating_sub(4)..]);
                }
            }
            let mut modern = downloaded;
            modern.extend_from_slice(&info.version.to_be_bytes());
            assert_eq!(
                decode_group(&modern, &info, [0; 4], 64 << 20, true)?,
                expected
            );
            println!(
                "archive {archive} group {} revision {} files {} modern/live agree",
                info.id,
                info.version,
                expected.len()
            );
            count += 1;
        }
    }
    println!("{count} live JS5 groups validated against actual reference tables and modern suffix; {stale} disk groups rejected; no cache files written");
    Ok(())
}
