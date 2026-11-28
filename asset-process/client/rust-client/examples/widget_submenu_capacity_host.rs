use gielinor_client::{
    buffer::Reader, client_state::ClientState, context_menu::Action, disk_cache::DiskCache,
    widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    client.canvas = [200, 100];
    let mut r = Reader::new(include_bytes!("../tests/fixtures/widget-submenu-capacity-jvm.bin"));
    for case in 0..r.u32()? {
        let size = r.u32()?;
        let mut row = Vec::new();
        for i in 0..size {
            row.push(if r.u8()? != 0 {
                format!("Sub{i}")
            } else {
                String::new()
            });
        }
        let expected = (0..r.u32()?).map(|_| r.u32()).collect::<Result<Vec<_>>>()?;
        let mut w = Widget::empty(1, 4);
        w.raw_size = [50, 50];
        w.flags = 2;
        w.actions = vec!["Op".into()];
        w.sub_actions = vec![Some(row)];
        client.interfaces = Default::default();
        client.interfaces.install_group(0, vec![w])?;
        client.interfaces.root = Some(0);
        let menu = client.interface_menu([10, 10])?;
        let actual = menu[0]
            .children
            .iter()
            .map(|entry| match entry.action {
                Action::Interface { sub: Some(sub), .. } => sub as u32,
                _ => panic!("unexpected action"),
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "case {case} size {size}");
    }
    assert_eq!(r.remaining(), 0);
    println!("256 actual ClientState widget submenu capacity traces");
    Ok(())
}
