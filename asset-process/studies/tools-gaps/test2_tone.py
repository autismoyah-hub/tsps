"""Test of pose.py --tone: a real item against (2) its re-encoded copy (must be 0) and (3) a copy with every face
one lightness step darker (must be small and non-zero, signed negative = column 3 darker). usage: test2_tone.py"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib
T = pathlib.Path(_AP.fix(r'<ROOT>\tools'))
sys.path.insert(0, str(T))
import osrscache, author
PY = [_AP.fix(r'<PYTHON>'), '-I']
OUT = pathlib.Path(__file__).parent / 't2'


def darker(src, dst, steps=1):
    m = osrscache.decode_model(pathlib.Path(src).read_bytes())
    g = dict(vertices=[list(v) for v in m['vertices']], faces=m['faces'], vertex_skins=m['vertex_skins'], face_skins=m['face_skins'],
             priorities=m['priorities'], colors=[(c & ~127) | max(0, (c & 127) - steps) for c in m['colors']])
    if any(m['render_types']): g['render_types'] = m['render_types']
    if any(m['alpha']): g['alpha'] = m['alpha']
    if any(m['depth_bias']): g['face_flags'] = [(b // 2) & 255 for b in m['depth_bias']]
    pathlib.Path(dst).write_bytes(author.native_codec.model(g))
    changed = sum(a != b for a, b in zip(m['colors'], g['colors']))
    print(f'{dst.name}: {changed} of {len(m["colors"])} faces one lightness step darker')


for item, slot, extra in ((1163, 'head+hair+jaw', ['--aim', 'head', '--zoom', '130', '--cell', '400x400']), (1101, 'body', []),
                          (11840, 'feet', ['--aim', 'feet', '--zoom', '130', '--cell', '400x400'])):
    copy = OUT / f'{item}-copy.bin'; dark = OUT / f'{item}-dark1.bin'
    subprocess.run([*PY, str(T / 'reencode_item.py'), str(item), str(copy)], check=True, capture_output=True)
    darker(copy, dark)
    r = subprocess.run([*PY, str(T / 'pose.py'), str(OUT / f'tone-{item}.png'),
                        f'item:{item},model:{copy.as_posix()}@{slot},model:{dark.as_posix()}@{slot},none',
                        '--seq', '819', '--frames', 'bind,2,5', '--tone', '1:2,1:3', '--diff', '1:2,1:3', *extra], capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    print(f'--- item {item}')
    print('\n'.join(l for l in r.stdout.splitlines() if l.startswith(('tone', 'diff'))))
