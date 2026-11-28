"""patch_arena_view.py: writes client/rust-client/examples/arena_view.rs = examples/asset_arena.rs + a `plane N` script line.
asset_arena draws plane 0 only; boss arenas stand on planes 0 to 3 (the Nightmare's is plane 3, Tekton's room plane 1).
`plane N`: the scene mesh is built and shown up to plane N, the camera target's ground height, actor heights and the
actors' own plane (client.data.position[0]) are plane N. Nothing else is changed. Regenerate: run this, then
  cargo +stable-x86_64-pc-windows-gnu build --release --example arena_view     (in client/rust-client)
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import pathlib, re
ROOT = pathlib.Path(_AP.fix('<CLIENT_SRC>/examples'))
s = (ROOT / 'asset_arena.rs').read_text(encoding='utf-8')


def rep(a, b, n=1):
    global s
    assert s.count(a) == n, (s.count(a), a[:70])
    s = s.replace(a, b)


rep('    let cam = lines.iter().find(|l| l[0] == "camera").ok_or("script needs a camera line")?;',
    '    let cam = lines.iter().find(|l| l[0] == "camera").ok_or("script needs a camera line")?;\n'
    '    let pl: usize = match lines.iter().find(|l| l[0] == "plane") { Some(l) => l[1].parse()?, None => 0 };')
rep('            "camera" => {}', '            "camera" => {}\n            "plane" => {}')
rep('WorldMesh::build(&scene, &mut client.scripts.source.cache, 0)', 'WorldMesh::build(&scene, &mut client.scripts.source.cache, pl)', 2)
rep('mesh.visible_triangles(0)', 'mesh.visible_triangles(pl as u8)')
rep('scene.height_fine(0, ', 'scene.height_fine(pl, ')
n = s.count('scene.elevation(0, '); s = s.replace('scene.elevation(0, ', 'scene.elevation(pl, ')
m = s.count('scene.height(0, '); s = s.replace('scene.height(0, ', 'scene.height(pl, ')
rep('    client.preferences.render_self = true;', '    client.preferences.render_self = true;\n    client.data.position[0] = pl as _;')
s = '//! research-arenas: asset_arena.rs + `plane N` (see studies/research-arenas/rust/patch_arena_view.py).\n' + s
(ROOT / 'arena_view.rs').write_text(s, encoding='utf-8', newline='\n')
print('written; elevation', n, 'height', m)
