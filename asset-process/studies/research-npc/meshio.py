"""Read and write the game's model files for the NPC work (research-npc).

load(path) -> dict   vertices [(x, y, z)] in CACHE coordinates (y down), faces, colors (16-bit HSL), priorities,
                     render_types (1 = flat), alpha, depth (per-face depth offset as stored, signed), vertex_skins,
                     face_skins
save(m, path)        through the project's encoder (native_codec), checked by decoding the result again
col(h, s, l)         the 16-bit colour
usage as a tool: meshio.py FILE.bin [...]   prints counts, bounds, label / priority / depth-offset histograms, colours
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, collections
sys.path.insert(0, _AP.fix(r'<ROOT>\tools'))
sys.path.insert(1, _AP.fix(r'<NATIVE_ASSETS>'))
import osrscache, native_codec

def col(h, s, l): return (h << 10) | (s << 7) | l
def hsl(c): return (c >> 10) & 63, (c >> 7) & 7, c & 127
def load(path):
    m = osrscache.decode_model(pathlib.Path(path).read_bytes())
    nf = len(m['faces'])
    return dict(vertices=[tuple(int(c) for c in v) for v in m['vertices']], faces=[tuple(f) for f in m['faces']], colors=list(m['colors']),
                priorities=list(m['priorities']), render_types=list(m['render_types']) or [0] * nf, alpha=list(m['alpha']) or [0] * nf,
                depth=[b // 2 for b in m['depth_bias']] if m.get('depth_bias') else [0] * nf,
                vertex_skins=list(m['vertex_skins'] or [0] * len(m['vertices'])), face_skins=list(m.get('face_skins') or [0] * nf))
def save(m, path):
    nf = len(m['faces'])
    g = dict(vertices=[list(p) for p in m['vertices']], faces=[tuple(f) for f in m['faces']], colors=list(m['colors']),
             vertex_skins=[int(l) for l in m['vertex_skins']], priorities=list(m.get('priorities') or [0] * nf))
    if any(m.get('face_skins') or []): g['face_skins'] = [int(l) for l in m['face_skins']]
    if any(m.get('render_types') or []): g['render_types'] = list(m['render_types'])
    if any(m.get('alpha') or []): g['alpha'] = list(m['alpha'])
    if any(m.get('depth') or []): g['face_flags'] = [d & 255 for d in m['depth']]
    data = native_codec.model(g)
    back = load_bytes(data)
    for k in ('vertices', 'faces', 'colors', 'priorities', 'vertex_skins'):
        if [tuple(v) if isinstance(v, (list, tuple)) else v for v in back[k]] != [tuple(v) if isinstance(v, (list, tuple)) else v for v in m[k]]:
            raise SystemExit(f'round trip changed {k}: not written')
    if any(m.get('depth') or []) and back['depth'] != list(m['depth']): raise SystemExit('round trip changed depth offsets')
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    pathlib.Path(path).write_bytes(data)
    return data
def load_bytes(data):
    m = osrscache.decode_model(data); nf = len(m['faces'])
    return dict(vertices=[tuple(int(c) for c in v) for v in m['vertices']], faces=[tuple(f) for f in m['faces']], colors=list(m['colors']),
                priorities=list(m['priorities']), render_types=list(m['render_types']) or [0] * nf, alpha=list(m['alpha']) or [0] * nf,
                depth=[b // 2 for b in m['depth_bias']] if m.get('depth_bias') else [0] * nf,
                vertex_skins=list(m['vertex_skins'] or [0] * len(m['vertices'])), face_skins=list(m.get('face_skins') or [0] * nf))
def stat(path):
    m = load(path); V = m['vertices']
    lo = [min(v[i] for v in V) for i in range(3)]; hi = [max(v[i] for v in V) for i in range(3)]
    print(f'{path}: {len(V)} vertices {len(m["faces"])} faces; x {lo[0]}..{hi[0]} height {-hi[1]}..{-lo[1]} z {lo[2]}..{hi[2]}')
    print('   vertex labels', dict(sorted(collections.Counter(m['vertex_skins']).items())))
    print('   priorities', dict(sorted(collections.Counter(m['priorities']).items())), ' depth offsets', dict(sorted(collections.Counter(m['depth']).items())),
          ' flat faces', sum(1 for r in m['render_types'] if r & 1), ' alpha faces', sum(1 for a in m['alpha'] if a))
    print('   colours (h,s,l: faces)', ', '.join(f'{hsl(c)}: {n}' for c, n in collections.Counter(m['colors']).most_common(14)))
if __name__ == '__main__':
    for p in sys.argv[1:]: stat(p)
