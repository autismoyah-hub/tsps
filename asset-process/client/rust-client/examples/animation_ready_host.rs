extern crate gielinor_client;
use gielinor_client::animation_readiness;
pub use gielinor_client::{animation, cached_animation, disk_cache, skeletal, Error, Result};
fn main() -> Result<()> {
    let mut cache = disk_cache::DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let sequences = animation::Sequence::load_all(&mut cache)?;
    let mut ready = animation_readiness::Readiness::default();
    let mut ids = sequences
        .values()
        .filter(|s| s.cached_animation.is_some())
        .map(|s| s.id)
        .collect::<Vec<_>>();
    ids.sort();
    assert!(ready.ready(&mut cache, &sequences[&13]));
    for id in ids.into_iter().take(16) {
        assert!(ready.ready(&mut cache, &sequences[&id]), "cached {id}");
        println!("cached sequence {id} ready");
    }
    let mut bad = animation::Sequence::decode(0, &[0])?;
    bad.cached_animation = Some(u32::MAX);
    assert!(!ready.ready(&mut cache, &bad));

    let mut animations = gielinor_client::animation_runtime::Animations::load(&mut cache)?;
    let mut missing = animation::Sequence::decode(60000, &[0])?;
    missing.frames = vec![0xffff0000];
    missing.lengths = vec![1];
    animations.sequences.insert(60000, missing);
    let mut model = gielinor_client::model::Model::combine(&[])?;
    model.vertices = vec![[1., 2., 3.]];
    let before = model.vertices.clone();
    animations.apply(&mut cache, &mut model, 60000, &Default::default())?;
    assert_eq!(model.vertices, before);
    println!("Real legacy/cached readiness and unavailable animation base-model fallback passed");
    Ok(())
}
