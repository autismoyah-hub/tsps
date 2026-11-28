#[path = "../tests/support/retained_size.rs"]
mod fixture;
use gielinor_client::{
    crypto::crc32, disk_cache::DiskCache, interface_render::Renderer, item::Definitions,
    raster::Raster,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let items = Definitions::load(&mut cache)?;
    let mut renderer = Renderer::load(&mut cache, 0.8)?;
    let cases = fixture::cases()?;
    for case in &cases {
        let mut state = fixture::prepare(case, &mut cache, &items)?;
        assert_eq!(
            fixture::queries(&mut state, &mut cache, &items)?,
            case.geometry,
            "before draw {case:?}"
        );
        for frame in 0..2 {
            let layout = state.layout(fixture::CANVAS)?;
            let mut raster = Raster::new(160, 96, 160 * 96)?;
            raster.clear(0xff302b25);
            let report = renderer.draw(&mut cache, &items, &state, &layout, &mut raster)?;
            assert!(report.pending.is_empty());
            let pixels: Vec<_> = raster
                .pixels()
                .iter()
                .flat_map(|p| p.to_be_bytes())
                .collect();
            assert_eq!(crc32(&pixels), case.crc, "frame {frame} {case:?}");
            assert_eq!(
                fixture::queries(&mut state, &mut cache, &items)?,
                case.geometry,
                "after draw {case:?}"
            );
        }
    }
    println!("{} original sizing transitions, twice-rendered rectangle CRCs and script geometry queries pass actual-cache rendering",cases.len());
    Ok(())
}
