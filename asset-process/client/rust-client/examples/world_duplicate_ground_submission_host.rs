use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet,
    protocol::Packet, scene::Scene, world_actors,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let mut actors = world_actors::Actors::load(&mut c.scripts.source.cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-ground-submission-jvm.bin"
    ));
    let count = r.u32()?;
    for case in 0..count {
        c.world_entities = Default::default();
        c.detached_world_entities.clear();
        c.active_world = 0;
        c.view_registry = Default::default();
        c.player_state = None;
        let id = r.u32()? as u16;
        let _order = r.u32()?;
        for _ in 0..r.u32()? {
            let opcode = r.u32()? as u16;
            let n = r.u32()? as usize;
            let payload = r.bytes(n)?.to_vec();
            c.apply(game_packet::decode(&Packet { opcode, payload })?.unwrap())?;
        }
        let _oldest_size = r.u32()?;
        let direct_size = r.u32()? as usize;
        let _nested_size = r.u32()?;
        let absent = r.u32()?;
        let plane_case = r.u32()?;
        for (slot, parent) in [0, 7].into_iter().enumerate() {
            let e = c
                .world_entities
                .get_mut(&parent)
                .unwrap()
                .entities
                .get_mut(&id)
                .unwrap();
            e.plane = if plane_case == 0 { 0 } else { slot as u8 + 1 };
            if (slot == 0 && absent == 0) || (slot == 1 && absent == 1) {
                e.zone.piles.insert(
                    [e.plane as i32, 2, 2],
                    vec![gielinor_client::zone::Item {
                        id: 995,
                        quantity: 1,
                        operations: 31,
                        visible: 0,
                        despawn: i32::MAX,
                        ownership: 0,
                        private: false,
                        custom: None,
                    }],
                );
            }
        }
        let expected = (0..r.u32()?)
            .map(|_| r.u32().map(|id| id as u16))
            .collect::<Result<Vec<_>, _>>()?;
        let _nested = (0..r.u32()?)
            .map(|_| r.u32())
            .collect::<Result<Vec<_>, _>>()?;
        let scene = Scene::empty_sized(0, 0, direct_size, direct_size)?;
        let (triangles, pending) =
            actors.ground_triangles(&mut c, &scene, &Default::default(), Some(id))?;
        let actual = triangles
            .iter()
            .map(|t| ((t.tag >> 32) & 65535) as u16)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            actual,
            expected.iter().copied().collect(),
            "case{case} root-owned ground items"
        );
        assert!(pending.is_empty(), "case{case} pending {pending:?}");
        assert_eq!(
            triangles.is_empty(),
            expected.is_empty(),
            "case{case} root-owned ground visibility"
        );
        assert!(
            triangles.iter().all(|t| t.tag
                == (0x1000000000000000
                    | (995u64 << 32)
                    | ((if plane_case == 0 { 0u64 } else { 1u64 }) << 28)
                    | (2 << 14)
                    | 2)),
            "case{case} ground tag"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!(
        "{count} original duplicate ground-pile submissions match actual-cache geometry and tags"
    );
    Ok(())
}
