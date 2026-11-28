use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    script_date,
    vm::{Host, Object, Stacks},
    Result,
};
fn text(r: &mut Reader<'_>) -> Result<String> {
    let len = r.u16()? as usize;
    Ok(String::from_utf8(r.bytes(len)?.to_vec()).unwrap())
}
fn main() -> Result<()> {
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/script-date-local-jvm.bin"
    ));
    let zone = text(&mut r)?;
    let locale = text(&mut r)?;
    let calendar = text(&mut r)?;
    assert_eq!(zone, "America/Los_Angeles");
    assert_eq!(locale, "en-US");
    assert_eq!(calendar, "java.util.GregorianCalendar");
    assert_eq!(
        script_date::local_identity()?,
        ("Pacific Standard Time".into(), false),
        "This held local fixture requires the recorded Windows timezone and calendar"
    );
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let count = r.u32()?;
    for _ in 0..count {
        let day = r.u32()? as i32;
        let expected = text(&mut r)?;
        let mut stacks = Stacks::default();
        stacks.push_int(day)?;
        client.extended(4104, false, &mut stacks)?;
        match stacks.pop_object()? {
            Object::String(actual) => assert_eq!(actual, expected),
            _ => panic!("date string output"),
        }
        assert!(stacks.ints().is_empty());
        assert!(stacks.objects().is_empty());
    }
    assert_eq!(r.remaining(), 0);
    println!("Actual-cache ClientState opcode4104: {count} JVM local date cases passed; {zone}, {locale}, {calendar}");
    Ok(())
}
