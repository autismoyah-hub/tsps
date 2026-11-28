use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    input::{self, Input, Update},
    interface_state::Interfaces,
    script_runtime::CacheSource,
    vm::{Host, Value, Variable},
    widget::Widget,
    Result,
};
#[path = "../tests/support/typed_character_admission.rs"]
mod support;
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    client.scripts.source = CacheSource::new(DiskCache::open(
        "tests/fixtures/typed-character-admission-vm-cache",
        1 << 20,
    )?);
    client.interfaces = Interfaces::default();
    let mut w = Widget::empty(0, 4);
    w.raw_size = [100, 100];
    client.interfaces.install_group(0, vec![w])?;
    client.interfaces.root = Some(0);
    client
        .interfaces
        .runtime
        .get_mut(&0)
        .unwrap()
        .listeners
        .insert(
            1419,
            (
                vec![
                    Value::Int(1419),
                    Value::Int(-2147483640),
                    Value::Int(-2147483639),
                ],
                None,
            ),
        );
    client.canvas = [200, 200];
    let mut pending = Input::default();
    let mut checked = 0;
    for case in support::cases()?
        .into_iter()
        .filter(|c| c.event.is_some() || c.value % 257 == 0 || (128..160).contains(&c.value))
    {
        client.input.begin_cycle();
        pending.begin_cycle();
        input::dispatch(
            Update::Character(case.value),
            [&mut client.input, &mut pending],
            None,
            |_| None,
            |_| false,
        )?;
        for i in 0..3 {
            client
                .scripts
                .write(Variable::ClientInt, i, Value::Int(-999))?;
        }
        client.tick_interfaces(false)?;
        assert!(
            client.script_failures.is_empty(),
            "character{:04x}: {:?}",
            case.value,
            client.script_failures
        );
        let expected = case
            .event
            .map(|(key, ch)| [1, key, ch])
            .unwrap_or([-999; 3]);
        for (i, want) in expected.into_iter().enumerate() {
            let got = client.scripts.read(Variable::ClientInt, i as i32)?;
            assert!(
                matches!(got,Value::Int(v)if v==want),
                "character{:04x} varc{i}: {got:?}, expected{want}",
                case.value
            );
        }
        checked += 1;
    }
    // U+2020 occurs in both the admitted set and the 257-step sample.
    assert_eq!(checked, 537);
    println!("537 original character admission cases match actual-cache listener count and key/character arguments");
    Ok(())
}
