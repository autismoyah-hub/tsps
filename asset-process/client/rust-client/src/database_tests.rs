use super::*;
#[test]
fn database_interpreter_state_transitions_match_original_jvm() {
    fn raw<'a>(r: &mut Reader<'a>) -> &'a [u8] {
        let n = r.u32().unwrap() as usize;
        r.bytes(n).unwrap()
    }
    let mut r = Reader::new(include_bytes!("../tests/fixtures/database-script-jvm.bin"));
    let mut db = Database::default();
    let bytes = raw(&mut r);
    db.indexes.insert((7, 1), Index::decode(bytes).unwrap());
    db.indexes.insert((7, 0), Index::decode(bytes).unwrap());
    db.tables
        .insert(7, crate::database_defs::Table::decode(raw(&mut r)).unwrap());
    db.definitions
        .insert(100, crate::database_defs::Row::decode(raw(&mut r)).unwrap());
    db.definitions
        .insert(101, crate::database_defs::Row::decode(&[0]).unwrap());
    let directory =
        std::env::temp_dir().join(format!("rust239-database-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("main_file_cache.dat2"), []).unwrap();
    let mut cache = DiskCache::open(&directory, 1 << 20).unwrap();
    for step in 0..r.u32().unwrap() {
        let op = r.u32().unwrap() as u16;
        let mut s = Stacks::default();
        for _ in 0..r.u32().unwrap() {
            s.push_int(r.u32().unwrap() as i32).unwrap();
        }
        assert!(db.execute(op, &mut s, &mut cache).unwrap());
        let expected = (0..r.u32().unwrap())
            .map(|_| r.u32().unwrap() as i32)
            .collect::<Vec<_>>();
        assert_eq!(s.ints(), expected, "step {step}, opcode {op}");
        assert_eq!(s.objects().len(), r.u32().unwrap() as usize);
        for object in s.objects() {
            assert!(matches!(object,crate::vm::Object::String(v) if v.as_bytes()==raw(&mut r)));
        }
        let expected = (0..r.u32().unwrap())
            .map(|_| r.u64().unwrap() as i64)
            .collect::<Vec<_>>();
        assert_eq!(s.longs(), expected);
    }
    assert_eq!(r.remaining(), 0);
    drop(cache);
    std::fs::remove_file(directory.join("main_file_cache.dat2")).unwrap();
    std::fs::remove_dir(directory).unwrap();
}
