use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet::Update,
    world_entity::Entity,
    zone::{Item, Packet, Record, State},
};
fn packet(plane: i32) -> Packet {
    Packet {
        absolute: Some([plane, 4, 4]),
        origin: None,
        clear: false,
        records: vec![
            Record::Add {
                at: 0,
                item: Item {
                    id: 995,
                    quantity: 1,
                    operations: 31,
                    visible: 0,
                    despawn: i32::MAX,
                    ownership: 0,
                    private: false,
                    custom: None,
                },
            },
            Record::Remove {
                at: 0,
                id: 995,
                quantity: 1,
            },
        ],
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    for plane in 0..4 {
        client
            .zone
            .apply(packet(plane), [0, 0], plane, 0, &client.items)?;
        assert!(client.zone.piles.is_empty());
        assert!(client.zone.scene_allocations.contains(&[plane, 4, 4]));
    }
    let mut entity = Entity::default();
    entity.dimensions = [16, 16];
    entity.zone = client.zone.clone();
    client
        .world_entities
        .entry(0)
        .or_default()
        .entities
        .insert(9, entity);
    client.world_base((0, 0));
    assert!(client.zone.scene_allocations.is_empty());
    assert_eq!(
        client.world_entities[&0].entities[&9]
            .zone
            .scene_allocations
            .len(),
        4,
        "root rebuild must not clear child history"
    );
    client.zone.apply(packet(2), [0, 0], 2, 0, &client.items)?;
    client.active_world = 9;
    // Empty 16x16 rebuild: six header bytes and sixteen zero chunk-presence bits.
    client.apply(Update::WorldEntityRebuild(vec![0; 8]))?;
    let child = &client.world_entities[&0].entities[&9];
    assert!(child.zone.scene_allocations.is_empty());
    assert!(child.rebuild.is_some());
    assert!(
        client.zone.scene_allocations.contains(&[2, 4, 4]),
        "child rebuild must not clear root history"
    );
    let mut child_zone = State::default();
    child_zone.apply_in_view(packet(3), [0, 0], 3, 0, &client.items, [16, 16])?;
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&9)
        .unwrap()
        .zone = child_zone;
    assert!(
        client.world_entities[&0].entities[&9]
            .zone
            .scene_allocations
            .contains(&[3, 4, 4]),
        "events after rebuild survive"
    );
    println!("Actual-cache root/child item allocation histories survive removal, reset independently on rebuild and retain subsequent events");
    Ok(())
}
