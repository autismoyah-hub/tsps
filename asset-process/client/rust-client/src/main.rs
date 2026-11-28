use gielinor_client::{cache, crypto, disk_cache::DiskCache, js5, protocol239};
use std::{env, error::Error, fs};
#[global_allocator]
static ALLOCATOR: gielinor_client::client_metrics::TrackedAllocator = gielinor_client::client_metrics::TrackedAllocator;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        #[cfg(windows)]
        Some("live-view"|"live-token-view")=>{if args.len()!=6&&!(args.len()==7&&(args[6]=="--smoke"||args[6]=="--smoke-actions"||args[6]=="--smoke-inventory")){return Err("expected live-view CACHE_DIRECTORY ADDRESS USERNAME PASSWORD_FILE_OR_TOKEN_CONFIG PUBLIC_KEY [--smoke|--smoke-actions|--smoke-inventory]".into());}(if args[0]=="live-token-view" { gielinor_client::viewport::live_token_view } else { gielinor_client::viewport::live_view })(&args[1],&args[2],&args[3],&args[4],&args[5],args.len()==7,args.get(6).is_some_and(|v|v=="--smoke-actions"||v=="--smoke-inventory"),args.get(6).is_some_and(|v|v=="--smoke-inventory"))?;},
        #[cfg(windows)]
        Some("interface-view")=>{if args.len()!=3&&!(args.len()==4&&args[3]=="--smoke"){return Err("expected interface-view CACHE_DIRECTORY GROUP [--smoke]".into());}gielinor_client::viewport::interface_view(&args[1],args[2].parse()?,args.len()==4)?;},
        Some("verify-animations")=>{
            if args.len()!=2{return Err("expected verify-animations CACHE_DIRECTORY".into());}
            let mut cache=DiskCache::open(&args[1],64<<20)?;let sequences=gielinor_client::animation::Sequence::load_all(&mut cache)?;
            let ids=cache.reference(1)?.groups.iter().map(|g|g.id).collect::<Vec<_>>();let mut skeletons=std::collections::HashMap::new();let mut bones=0;
            let mut rejected_skeletons=0;for id in ids{match gielinor_client::animation::Skeleton::decode(id.try_into()?,&cache.file(1,id,0,[0;4])?){Ok(skeleton)=>{bones+=skeleton.bones.len();skeletons.insert(id as u16,skeleton);},Err(e)=>{rejected_skeletons+=1;if rejected_skeletons<=20{eprintln!("skeleton {id}: {e}");}}}}
            let groups=cache.reference(0)?.groups.iter().map(|g|g.id).collect::<Vec<_>>();let(mut frames,mut rejected_frames)=(0,0);for id in groups{for(file,data)in cache.group(0,id,[0;4])?{let mut r=gielinor_client::buffer::Reader::new(&data);let skeleton=r.u16()?;let result=if let Some(skeleton)=skeletons.get(&skeleton){gielinor_client::animation::Frame::decode(&data,skeleton).map(|_|())}else{Err(gielinor_client::Error::Invalid("missing/rejected skeleton"))};match result{Ok(())=>frames+=1,Err(e)=>{rejected_frames+=1;if rejected_frames<=20{eprintln!("frame {id}:{file}: {e}");}}}}}
            println!("Decoded {} sequences ({} cached), {} skeletons with {bones} bones, {frames} legacy frames; rejected {rejected_skeletons} skeletons and {rejected_frames} frames",sequences.len(),sequences.values().filter(|s|s.cached_animation.is_some()).count(),skeletons.len());
            if rejected_skeletons+rejected_frames!=0{return Err("animation asset audit has rejected inputs".into());}
        }
        #[cfg(windows)]
        Some("scene-view") => {
            if args.len() != 5 && !(args.len()==6 && args[5]=="--smoke") {
                return Err("expected scene-view CACHE_DIRECTORY ZONE_X ZONE_Y MAX_PLANE [--smoke]".into());
            }
            gielinor_client::viewport::run(
                &args[1],
                args[2].parse()?,
                args[3].parse()?,
                args[4].parse()?,
                args.len()==6,
            )?;
        }
        Some("scene-render") => {
            if args.len() != 6 {
                return Err(
                    "expected scene-render CACHE_DIRECTORY ZONE_X ZONE_Y MAX_PLANE OUTPUT.bmp"
                        .into(),
                );
            }
            let mut cache = DiskCache::open(&args[1], 64 << 20)?;
            let rebuild = gielinor_client::rebuild::Normal {
                world_area: -1,
                zone_x: args[2].parse()?,
                zone_y: args[3].parse()?,
                players: None,
            };
            let scene = gielinor_client::scene::Scene::load_normal(&mut cache, &rebuild)?;
            let mesh = gielinor_client::world_mesh::WorldMesh::build(
                &scene,
                &mut cache,
                args[4].parse()?,
            )?;
            let mut textures = std::collections::HashMap::new();
            for (id, def) in gielinor_client::texture::Definition::load_all(&mut cache)? {
                textures.insert(
                    id,
                    gielinor_client::texture::Texture::load(&mut cache, def, 128, 0.8)?,
                );
            }
            let palette = gielinor_client::floor::palette(0.8)?;
            let centre = [
                52.0 * 128.0,
                scene.height_fine(0, 52 * 128, 52 * 128).unwrap_or(0) as f32,
                52.0 * 128.0,
            ];
            let camera = gielinor_client::render3d::Camera::look_at(
                [centre[0] + 1600.0, centre[1] - 2200.0, centre[2] - 1800.0],
                centre,
                760.0,
            )?;
            let mut frame = gielinor_client::render3d::Frame::new(1280, 900)?;
            frame.draw(&camera, &mesh.triangles, &palette, &textures);
            save_new(&args[5], &frame.raster()?.bmp()?)?;
            println!(
                "Rendered {} object instances, {} scene triangles ({} projected triangles).",
                mesh.object_instances,
                mesh.triangles.len(),
                frame.triangles
            );
        }
        Some("verify-objects") => {
            if args.len() != 2 {
                return Err("expected verify-objects CACHE_DIRECTORY".into());
            }
            let mut cache = DiskCache::open(&args[1], 64 << 20)?;
            let definitions = gielinor_client::object::Definition::load_all(&mut cache)?;
            println!(
                "Decoded {} object definitions, {} model references, {} actions",
                definitions.len(),
                definitions.values().map(|d| d.models.len()).sum::<usize>(),
                definitions
                    .values()
                    .map(|d| d.operations.len())
                    .sum::<usize>()
            );
        }
        Some("verify-models") => {
            if args.len() != 2 {
                return Err("expected verify-models CACHE_DIRECTORY".into());
            }
            let mut cache = DiskCache::open(&args[1], 64 << 20)?;
            let ids = cache
                .reference(7)?
                .groups
                .iter()
                .map(|g| g.id)
                .collect::<Vec<_>>();
            let (mut models, mut vertices, mut faces, mut failures) = (0, 0, 0, 0);
            let mut formats = std::collections::BTreeMap::new();
            for id in ids {
                for (file, data) in cache.group(7, id, [0; 4])? {
                    match gielinor_client::model::Model::decode(&data) {
                        Ok(model) => {
                            models += 1;
                            vertices += model.vertices.len();
                            faces += model.faces.len();
                            *formats.entry(format!("{:?}", model.format)).or_insert(0) += 1;
                        }
                        Err(error) => {
                            failures += 1;
                            if failures <= 20 {
                                eprintln!("model {id}:{file}: {error}");
                            }
                        }
                    }
                }
            }
            println!("Decoded {models} models, {vertices} vertices, {faces} faces; formats {formats:?}; {failures} failures.");
            if failures != 0 {
                return Err("model decode failures".into());
            }
        }
        Some("scene-map") => {
            if args.len() != 6 {
                return Err(
                    "expected scene-map CACHE_DIRECTORY ZONE_X ZONE_Y PLANE OUTPUT.bmp".into(),
                );
            }
            let mut cache = DiskCache::open(&args[1], 64 << 20)?;
            let rebuild = gielinor_client::rebuild::Normal {
                world_area: -1,
                zone_x: args[2].parse()?,
                zone_y: args[3].parse()?,
                players: None,
            };
            let scene = gielinor_client::scene::Scene::load_normal(&mut cache, &rebuild)?;
            let floors = gielinor_client::floor::Floors::load(&mut cache)?;
            let textures = gielinor_client::texture::Definition::load_all(&mut cache)?;
            let image = gielinor_client::minimap::terrain(
                &scene,
                &floors,
                &textures,
                args[4].parse()?,
                0,
                0,
            )?;
            save_new(&args[5], &image.bmp()?)?;
            println!("Scene base {},{}; {} objects; {} underlays, {} overlays, {} textures; missing regions {:?}",scene.base_x,scene.base_y,scene.objects.len(),floors.underlays.len(),floors.overlays.len(),textures.len(),scene.missing_regions);
        }
        Some("verify-textures") => {
            if args.len() != 2 {
                return Err("expected verify-textures CACHE_DIRECTORY".into());
            }
            let mut cache = DiskCache::open(&args[1], 64 << 20)?;
            let definitions = gielinor_client::texture::Definition::load_all(&mut cache)?;
            let mut pixels = 0;
            for definition in definitions.values() {
                let texture = gielinor_client::texture::Texture::load(
                    &mut cache,
                    definition.clone(),
                    128,
                    0.8,
                )?;
                pixels += texture.pixels.len();
            }
            println!(
                "Decoded {} texture definitions and {pixels} pixels.",
                definitions.len()
            );
        }
        Some("script-run") | Some("script-info") => {
            use gielinor_client::{
                script_runtime::{CacheSource, Runtime},
                vm::{Host, Status, Vm},
            };
            if args.len() < 3 {
                return Err(
                    "expected script-run|script-info CACHE_DIRECTORY ID [INTEGER_ARGUMENTS...]"
                        .into(),
                );
            }
            let mut runtime =
                Runtime::new(CacheSource::new(DiskCache::open(&args[1], 64 << 20)?), 239);
            let script = runtime.script(args[2].parse()?)?;
            if args[0] == "script-info" {
                println!(
                    "locals=({},{},{}) arguments=({},{},{})",
                    script.local_ints,
                    script.local_strings,
                    script.local_longs,
                    script.int_arguments,
                    script.string_arguments,
                    script.long_arguments
                );
                for (pc, instruction) in script.instructions.iter().enumerate() {
                    println!("{pc}: {} {:?}", instruction.opcode, instruction.operand);
                }
            } else {
                let ints = args[3..]
                    .iter()
                    .map(|s| s.parse())
                    .collect::<Result<Vec<i32>, _>>()?;
                if ints.len() != script.int_arguments as usize
                    || script.string_arguments != 0
                    || script.long_arguments != 0
                {
                    return Err("CLI script-run requires exactly the integer arguments and no object/long arguments".into());
                }
                let mut vm = Vm::new(script, ints, vec![], vec![], 500000)?;
                while vm.run(&mut runtime, 10000)? == Status::Yielded {}
                println!("Executed {} instructions; {:?}", vm.executed(), vm.stacks);
            }
        }
        Some("verify-varbits") => {
            if args.len() != 2 {
                return Err("expected verify-varbits CACHE_DIRECTORY".into());
            }
            let mut cache = DiskCache::open(&args[1], 64 << 20)?;
            let files = cache.group(2, 14, [0; 4])?;
            for (_, bytes) in &files {
                gielinor_client::script_runtime::Varbit::decode(bytes)?;
            }
            println!("Decoded {} varbit definitions.", files.len());
            let crcs = cache.login_crcs()?;
            println!("Login reference CRCs: {crcs:08x?}");
        }
        Some("verify-maps") => {
            if args.len() != 2 {
                return Err("expected verify-maps CACHE_DIRECTORY".into());
            }
            let mut store = DiskCache::open(&args[1], 64 << 20)?;
            let groups = store.reference(5)?.groups.clone();
            let (mut maps, mut objects, mut skipped, mut failures) =
                (0usize, 0usize, 0usize, 0usize);
            for group in groups {
                let files = store.group(5, group.id, [0; 4])?;
                let Some((_, terrain)) = files.iter().find(|(id, _)| *id == 0) else {
                    skipped += 1;
                    continue;
                };
                if terrain.len() < 32768 {
                    skipped += 1;
                    continue;
                }
                let result = (|| -> gielinor_client::Result<usize> {
                    gielinor_client::terrain::Region::decode(
                        terrain,
                        ((group.id >> 8) * 64) as i32,
                        ((group.id & 255) * 64) as i32,
                    )?;
                    let (_, locations) = files
                        .iter()
                        .find(|(id, _)| *id == 1)
                        .ok_or(gielinor_client::Error::Invalid("map location file missing"))?;
                    Ok(gielinor_client::terrain::locations(locations, 1_000_000)?.len())
                })();
                match result {
                    Ok(count) => {
                        maps += 1;
                        objects += count;
                    }
                    Err(error) => {
                        failures += 1;
                        if failures <= 20 {
                            eprintln!("region={}: {error}", group.id);
                        }
                    }
                }
            }
            println!("Decoded {maps} map regions, {} tiles, {objects} object placements; {skipped} short/non-terrain groups skipped.", maps * 16384);
            if failures != 0 {
                return Err(format!("{failures} map decode failures").into());
            }
        }
        Some("cache-index") => {
            if args.len() != 3 {
                return Err("expected cache-index CACHE_DIRECTORY ARCHIVE".into());
            }
            let mut store = DiskCache::open(&args[1], 64 << 20)?;
            for group in &store.reference(args[2].parse()?)?.groups {
                println!(
                    "group={} name={:?} files={} crc={:08x} version={}",
                    group.id,
                    group.name_hash,
                    group.files.len(),
                    group.crc,
                    group.version
                );
            }
        }
        Some("cache-named") => {
            if args.len() != 6 {
                return Err(
                    "expected cache-named CACHE_DIRECTORY ARCHIVE GROUP_NAME FILE_NAME OUTPUT"
                        .into(),
                );
            }
            let mut store = DiskCache::open(&args[1], 64 << 20)?;
            save_new(
                &args[5],
                &store.named_file(args[2].parse()?, &args[3], &args[4])?,
            )?;
        }
        Some("sprite-bmp") => {
            if args.len() != 4 {
                return Err("expected sprite-bmp SPRITE_FILE INDEX OUTPUT.bmp".into());
            }
            let sheet =
                gielinor_client::sprite::SpriteSheet::decode(&fs::read(&args[1])?, 16 << 20)?;
            let mut raster =
                gielinor_client::raster::Raster::new(sheet.width, sheet.height, 16 << 20)?;
            raster.clear(0xff202020);
            raster.draw_indexed(&sheet, args[2].parse()?, 0, 0)?;
            save_new(&args[3], &raster.bmp()?)?;
        }
        Some("verify-assets") => {
            if args.len() != 2 {
                return Err("expected verify-assets CACHE_DIRECTORY".into());
            }
            let mut store = DiskCache::open(&args[1], 64 << 20)?;
            let mut failures = 0;
            for archive in [8, 12] {
                let groups = store.reference(archive)?.groups.clone();
                let mut decoded = 0usize;
                let mut total = 0usize;
                let mut short_entries = 0usize;
                for group in groups {
                    for (file, bytes) in store.group(archive, group.id, [0; 4])? {
                        if archive == 12 && bytes.len() < 19 {
                            short_entries += 1;
                            continue;
                        }
                        let result = if archive == 8 {
                            gielinor_client::sprite::SpriteSheet::decode(&bytes, 16 << 20)
                                .map(|s| s.sprites.len())
                        } else {
                            gielinor_client::script::Script::decode(&bytes, 1_000_000)
                                .map(|s| s.instructions.len())
                        };
                        match result {
                            Ok(count) => {
                                decoded += 1;
                                total += count;
                            }
                            Err(error) => {
                                failures += 1;
                                if failures <= 20 {
                                    eprintln!(
                                        "archive={archive} group={} file={file}: {error}",
                                        group.id
                                    );
                                }
                            }
                        }
                    }
                }
                println!("archive={archive} decoded_files={decoded} {}={total} short_entries_not_decoded={short_entries}", if archive == 8 { "sprites" } else { "instructions" });
            }
            if failures != 0 {
                return Err(format!("{failures} asset decode failures").into());
            }
        }
        Some("js5-fetch") => {
            if args.len() != 5 && args.len() != 9 {
                return Err(
                    "expected js5-fetch IP:PORT ARCHIVE GROUP OUTPUT [KEY0 KEY1 KEY2 KEY3]".into(),
                );
            }
            let address: std::net::SocketAddr = args[1].parse()?;
            if address.port() == 43594 {
                return Err("43594 is reserved; use the project's 43597 or 43595 target".into());
            }
            let mut key = [0; 4];
            if args.len() == 9 {
                for (slot, text) in key.iter_mut().zip(&args[5..]) {
                    *slot = text.parse()?;
                }
            }
            let mut client = js5::Client::connect(
                address,
                protocol239::REVISION,
                key,
                std::time::Duration::from_secs(10),
                64 << 20,
            )?;
            let bytes = client.fetch(args[2].parse()?, args[3].parse()?)?;
            cache::Container::parse(&bytes, 64 << 20)?.decompress()?;
            save_new(&args[4], &bytes)?;
        }
        Some("cache-file") => {
            if args.len() != 6 {
                return Err("expected cache-file CACHE_DIRECTORY ARCHIVE GROUP FILE OUTPUT".into());
            }
            let mut cache = DiskCache::open(&args[1], 64 << 20)?;
            let bytes = cache.file(args[2].parse()?, args[3].parse()?, args[4].parse()?, [0; 4])?;
            save_new(&args[5], &bytes)?;
        }
        Some("verify-cache") => {
            if args.len() != 2 {
                return Err("expected verify-cache CACHE_DIRECTORY".into());
            }
            let mut store = DiskCache::open(&args[1], 64 << 20)?;
            let entries = fs::read(std::path::Path::new(&args[1]).join("main_file_cache.idx255"))?;
            let mut total = 0usize;
            for (archive, entry) in entries.chunks_exact(6).enumerate() {
                if entry == [0; 6] {
                    continue;
                }
                let archive = u8::try_from(archive)?;
                let groups = store.reference(archive)?.groups.clone();
                let mut bytes_read = 0usize;
                for info in &groups {
                    let bytes = store.raw_group(archive, info.id)?;
                    if bytes.len() < 7 {
                        return Err(format!("truncated archive {archive} group {}", info.id).into());
                    }
                    let end = bytes.len() - 2;
                    let version = u16::from_be_bytes(bytes[end..].try_into().unwrap());
                    if crypto::crc32(&bytes[..end]) != info.crc || version != info.version as u16 {
                        return Err(format!(
                            "checksum/version mismatch archive {archive} group {}",
                            info.id
                        )
                        .into());
                    }
                    bytes_read += bytes.len();
                }
                println!(
                    "archive={archive} verified={} bytes={bytes_read}",
                    groups.len()
                );
                total += groups.len();
            }
            println!("Verified CRC and version for {total} groups; encrypted payloads were not decrypted.");
        }
        Some("protocol") => {
            let packets = match args.get(1).map(String::as_str) {
                Some("client") => protocol239::CLIENT_PACKETS,
                Some("server") => protocol239::SERVER_PACKETS,
                _ => return Err("expected protocol client|server".into()),
            };
            for p in packets {
                println!("{:3} {:4} {}", p.opcode, p.size, p.name);
            }
        }
        Some("cache-group") => {
            if args.len() != 6 {
                return Err("expected cache-group DAT2 IDX ARCHIVE GROUP OUTPUT".into());
            }
            let group = cache::read_group(
                &fs::read(&args[1])?,
                &fs::read(&args[2])?,
                args[3].parse()?,
                args[4].parse()?,
            )?;
            // Never silently replace an existing export.
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[5])?;
            file.write_all(&group)?;
            println!(
                "Exported {} bytes; CRC32 {:08x}",
                group.len(),
                crypto::crc32(&group)
            );
        }
        Some("container") => {
            let path = args.get(1).ok_or("expected container FILE")?;
            let bytes = fs::read(path)?;
            let c = cache::Container::parse(&bytes, 64 << 20)?;
            println!(
                "compression={} compressed={} uncompressed={} version={:?}",
                c.compression,
                c.payload.len(),
                c.uncompressed_size,
                c.version
            );
            let decoded = c.decompress()?;
            println!(
                "Decoded {} bytes; CRC32 {:08x}",
                decoded.len(),
                crypto::crc32(&decoded)
            );
        }
        Some("scan-cache") => {
            let path =
                std::path::Path::new(args.get(1).ok_or("expected scan-cache CACHE_DIRECTORY")?);
            let dat = fs::read(path.join("main_file_cache.dat2"))?;
            let idx = fs::read(path.join("main_file_cache.idx255"))?;
            let mut archives = 0;
            let mut total_groups = 0;
            for id in 0..idx.len() / 6 {
                if idx[id * 6..id * 6 + 6] == [0; 6] {
                    continue;
                }
                let bytes = cache::read_group(&dat, &idx, 255, id as u32)?;
                let decoded = cache::Container::parse(&bytes, 64 << 20)?.decompress()?;
                let index = cache::ArchiveIndex::parse(&decoded)?;
                println!(
                    "archive={id} format={} version={} groups={} files={}",
                    index.format,
                    index.version,
                    index.groups.len(),
                    index.groups.iter().map(|g| g.files.len()).sum::<usize>()
                );
                archives += 1;
                total_groups += index.groups.len();
            }
            println!("Validated {archives} archive indexes, {total_groups} groups (index metadata only).");
        }
        None | Some("status") => {
            println!(
                "OSRS {} Rust port — INCOMPLETE, not a playable client",
                protocol239::REVISION
            );
            println!("Native read/write codecs, Huffman, ISAAC/XTEA/CRC32, packet framing, JS5 streaming, verified disk cache and file extraction.");
            println!(
                "{} client and {} server packet definitions.",
                protocol239::CLIENT_PACKETS.len(),
                protocol239::SERVER_PACKETS.len()
            );
            println!("Native login transport, RSA, proof of work, core script VM and arithmetic have fixture coverage. Scene, UI, audio and gameplay integration remain under construction.");
            println!("Commands: status | protocol client|server | cache-group DAT2 IDX ARCHIVE GROUP OUTPUT | container FILE | scan-cache CACHE_DIRECTORY | verify-cache CACHE_DIRECTORY | cache-file CACHE_DIRECTORY ARCHIVE GROUP FILE OUTPUT | js5-fetch IP:PORT ARCHIVE GROUP OUTPUT [KEY0 KEY1 KEY2 KEY3]");
            println!(
                "Assets: verify-assets CACHE_DIRECTORY | sprite-bmp SPRITE_FILE INDEX OUTPUT.bmp"
            );
            println!("Maps: verify-maps CACHE_DIRECTORY | cache-index CACHE_DIRECTORY ARCHIVE | cache-named CACHE_DIRECTORY ARCHIVE GROUP_NAME FILE_NAME OUTPUT");
            println!("Scripts: script-info CACHE_DIRECTORY ID | script-run CACHE_DIRECTORY ID [INTEGER_ARGUMENTS...] | verify-varbits CACHE_DIRECTORY");
        }
        _ => return Err("unknown command; run status for usage".into()),
    }
    Ok(())
}

fn save_new(path: &str, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    println!(
        "Exported {} bytes; CRC32 {:08x}",
        bytes.len(),
        crypto::crc32(bytes)
    );
    Ok(())
}
