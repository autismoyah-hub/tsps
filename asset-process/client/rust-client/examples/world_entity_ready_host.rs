use gielinor_client::{
    animation, animation_runtime::Animations, disk_cache::DiskCache, world_entity::Entity, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let mut animations = Animations::load(&mut cache)?;
    let id = *animations
        .sequences
        .iter()
        .filter(|(_, s)| s.cached_animation.is_some())
        .map(|(id, _)| id)
        .min()
        .unwrap();
    let mut entity = Entity::default();
    entity.default_animation.id = id as i32;
    entity.default_animation.frame = 0;
    let expected = animations.world_entity_matrix(&mut cache, &entity)?;
    assert!(expected.is_some());
    let mut missing = animation::Sequence::decode(60000, &[0])?;
    missing.cached_animation = Some(u32::MAX - 1);
    animations.sequences.insert(60000, missing);
    entity.animation.id = 60000;
    entity.animation.frame = 0;
    assert_eq!(
        animations.world_entity_matrix(&mut cache, &entity)?,
        expected,
        "unavailable action must use ready default scene matrix"
    );
    entity.default_animation.id = 60000;
    assert!(animations
        .world_entity_matrix(&mut cache, &entity)?
        .is_none());
    entity.animation.id = id as i32;
    assert_eq!(
        animations.world_entity_matrix(&mut cache, &entity)?,
        expected
    );
    entity.animation.delay = 1;
    assert!(animations
        .world_entity_matrix(&mut cache, &entity)?
        .is_none());
    println!("Child world scene animation: unavailable action falls back, unavailable default gives identity, ready action wins, delayed action waits");
    Ok(())
}
