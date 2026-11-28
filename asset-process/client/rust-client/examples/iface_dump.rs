// Research example (the working folder, private): open a group as the top interface, optionally set player
// variables and run scripts, run the onLoad scripts, then write the client's own render (PNG) and the laid-out
// widget tree in draw order (JSON lines) so a second renderer can draw exactly the same state.
// iface_dump CACHE GROUP OUT.png OUT.jsonl [p<varp>=<v>] [b<varbit>=<v>] [s<script>:<int|'text>:...] [t<ticks>]
// Variable settings apply before the onLoad scripts; s-entries run after them, in order.
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, interface_packet::Update,
    interface_render::Renderer, interface_state::ScriptEvent, raster::Raster, script_runtime::Source,
    vm::{Object, Value},
};
use std::io::Write;
fn esc(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 32 || (c as u32) > 126 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() < 5 {
        return Err("iface_dump CACHE GROUP OUT.png OUT.jsonl [p..=..] [b..=..] [s..] [t..]".into());
    }
    let cache = DiskCache::open(&a[1], 64 << 20)?;
    let mut client = ClientState::load(cache)?;
    let group: u16 = a[2].parse()?;
    let mut scripts = Vec::new();
    let mut ticks = 0;
    for arg in &a[5..] {
        let (k, rest) = arg.split_at(1);
        match k {
            "p" => {
                let (id, v) = rest.split_once('=').ok_or("p<id>=<v>")?;
                client.scripts.variables.set_player(id.parse()?, v.parse()?);
            }
            "b" => {
                let (id, v) = rest.split_once('=').ok_or("b<id>=<v>")?;
                let bit = client.scripts.source.varbit(id.parse()?)?;
                let base = client.scripts.variables.player(bit.base);
                client.scripts.variables.set_player(bit.base, bit.set(base, v.parse()?));
            }
            "s" => scripts.push(rest.to_string()),
            "t" => ticks = rest.parse()?,
            "m" => {}
            _ => return Err(format!("unknown argument {arg}").into()),
        }
    }
    client.interface(Update::OpenTop(group))?;
    let mut failed = Vec::new();
    while let Some(event) = client.next_interface_event()? {
        if let Err(e) = client.run_event(event) { failed.push(format!("{e}")); }
    }
    // m<group>.<component>=<sub group>[/kind]: if_opensub, in argument order, each followed by its scripts.
    for arg in &a[5..] {
        if let Some(rest) = arg.strip_prefix('m') {
            let (host, sub) = rest.split_once('=').ok_or("m<g>.<c>=<sub>")?;
            let (g, c) = host.split_once('.').ok_or("m<g>.<c>=<sub>")?;
            let (sub, kind) = sub.split_once('/').unwrap_or((sub, "1"));
            let parent = (g.parse::<u32>()? << 16) | c.parse::<u32>()?;
            client.interface(Update::OpenSub(gielinor_client::interface_packet::SubInterface { parent, group: sub.parse()?, kind: kind.parse()? }))?;
            while let Some(event) = client.next_interface_event()? {
                if let Err(e) = client.run_event(event) { failed.push(format!("{e}")); }
            }
        }
    }
    for s in &scripts {
        let mut values = Vec::new();
        for part in s.split(':') {
            if let Some(t) = part.strip_prefix('\'') {
                values.push(Value::Object(Object::String(t.to_string())));
            } else if let Some((g, c)) = part.split_once('.') {
                values.push(Value::Int(((g.parse::<i32>()?) << 16) | c.parse::<i32>()?));
            } else {
                values.push(Value::Int(part.parse()?));
            }
        }
        let event = ScriptEvent { widget: (group as u32) << 16, context: Default::default(), arguments: values };
        if let Err(e) = client.run_event(event) { failed.push(format!("script {s}: {e}")); }
        while let Some(event) = client.next_interface_event()? {
            if let Err(e) = client.run_event(event) { failed.push(format!("{e}")); }
        }
    }
    for _ in 0..ticks {
        client.tick_interfaces(false)?;
        while let Some(e) = client.script_failures.pop_front() { failed.push(e.to_string()); }
        client.cycle += 1;
        client.input.begin_cycle();
    }
    let layout = client.interfaces.layout(client.canvas)?;
    let mut renderer = Renderer::load(&mut client.scripts.source.cache, 0.8)?;
    let mut raster = Raster::new(765, 503, 1 << 20)?;
    raster.clear(0xff29251f);
    let report = renderer.draw(&mut client.scripts.source.cache, &client.items, &client.interfaces, &layout, &mut raster)?;
    std::fs::write(&a[3], raster.png()?)?;
    let mut out = std::fs::File::create(&a[4])?;
    for id in &layout.order {
        let p = &layout.widgets[id];
        let w = &client.interfaces.widgets[id];
        let r = &client.interfaces.runtime[id];
        writeln!(out, "{{\"id\":{},\"logical\":{},\"child_index\":{},\"parent\":{},\"abs\":[{},{}],\"size\":[{},{}],\"clip\":[{},{},{},{}],\"hidden\":{},\"if3\":{},\"kind\":{},\"content\":{},\"colors\":[{},{},{},{}],\"filled\":{},\"transparency\":{},\"transparency_bottom\":{},\"fill_mode\":{},\"line_width\":{},\"line_reversed\":{},\"sprites\":[{},{}],\"sprite_angle\":{},\"sprite_tiling\":{},\"outline\":{},\"sprite_shadow\":{},\"flip_v\":{},\"flip_h\":{},\"font\":{},\"text\":{},\"alternate_text\":{},\"text_alignment\":[{},{}],\"line_height\":{},\"text_shadow\":{},\"models\":[{},{}],\"item\":{},\"count\":{},\"condition\":{},\"scrollbar\":{},\"raw_position\":[{},{}],\"raw_size\":[{},{}],\"size_mode\":[{},{}],\"position_mode\":[{},{}],\"scroll_size\":[{},{}]}}",
            id, r.logical_id, r.child_index, p.parent.map_or(-1, |v| v as i64), p.absolute[0], p.absolute[1], p.geometry.size[0], p.geometry.size[1],
            p.clip[0], p.clip[1], p.clip[2], p.clip[3], p.hidden, w.if3, w.kind, w.content,
            w.colors[0], w.colors[1], w.colors[2], w.colors[3], w.filled, w.transparency, w.transparency_bottom, w.fill_mode, w.line_width, w.line_reversed,
            w.sprites[0], w.sprites[1], w.sprite_angle, w.sprite_tiling, w.outline, w.sprite_shadow, w.flip_vertical, w.flip_horizontal,
            w.font, esc(&w.text), esc(&w.alternate_text), w.text_alignment[0], w.text_alignment[1], w.line_height, w.text_shadow,
            w.models[0], w.models[1], r.item, r.count, r.condition, w.scrollbar,
            w.raw_position[0], w.raw_position[1], w.raw_size[0], w.raw_size[1], w.size_alignment[0], w.size_alignment[1],
            w.position_alignment[0], w.position_alignment[1], w.scroll_size[0], w.scroll_size[1])?;
    }
    println!("{{\"group\":{group},\"widgets\":{},\"drawn\":{},\"pending\":{},\"failed\":{}}}", layout.order.len(), report.drawn, report.pending.len(), esc(&failed.join(" | ")));
    Ok(())
}
