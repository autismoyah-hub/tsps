"""First measurements: real game assets against our exported models. Private output."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, collections, pathlib, statistics as st, math
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o

ARCH = pathlib.Path(_AP.fix(r'<REFERENCE_ARCHIVE>'))
E114 = pathlib.Path(_AP.fix(r'<E114_SCENE>\package'))


def area(v, f):
    a, b, c = (v[i] for i in f)
    ux, uy, uz = b[0] - a[0], b[1] - a[1], b[2] - a[2]
    vx, vy, vz = c[0] - a[0], c[1] - a[1], c[2] - a[2]
    return 0.5 * math.sqrt((uy * vz - uz * vy) ** 2 + (uz * vx - ux * vz) ** 2 + (ux * vy - uy * vx) ** 2)


def wquant(pairs, q):
    pairs = sorted(pairs)
    tot = sum(w for _, w in pairs)
    acc = 0
    for x, w in pairs:
        acc += w
        if acc >= q * tot:
            return x
    return pairs[-1][0]


def stats(m):
    v, F = m['vertices'], m['faces']
    ar = [area(v, f) for f in F]
    tot = sum(ar) or 1
    ext = sorted(max(p[i] for p in v) - min(p[i] for p in v) for i in range(3))
    L, S = [], []
    colarea = collections.Counter()
    for c, a, t in zip(m['colors'], ar, m['face_textures']):
        if t != -1:
            continue
        h, s, l = o.hsl_split(c)
        L.append((l, a)); S.append((s, a)); colarea[c] += a
    top = colarea.most_common()
    # number of colours needed to cover 90% of the surface
    acc = 0; n90 = 0
    for _, a in top:
        acc += a; n90 += 1
        if acc >= 0.9 * sum(colarea.values()):
            break
    return dict(
        faces=len(F), verts=len(v), ext=ext,
        smooth=sum(a for a, r in zip(ar, m['render_types']) if r == 0) / tot,
        smooth_faces=sum(1 for r in m['render_types'] if r == 0) / len(F),
        ncol=len(colarea), n90=n90,
        L50=wquant(L, .5) if L else None, L90=wquant(L, .9) if L else None,
        S50=wquant(S, .5) if S else None, S90=wquant(S, .9) if S else None,
        textured=sum(1 for t in m['face_textures'] if t != -1) / len(F),
        thin=ext[0] / ext[2] if ext[2] else 0, mid=ext[1] / ext[2] if ext[2] else 0,
        medarea=st.median(ar),
    )


def newest(kind, sub, n, pred=lambda j: True):
    d = ARCH / kind / sub
    ids = sorted((int(p.stem) for p in d.glob('*.json')), reverse=True)
    out = []
    seen = set()
    for i in ids:
        j = json.loads((d / f'{i}.json').read_text())
        key = tuple(j.get('model_ids') or [])
        if not key or key in seen or not pred(j):
            continue
        seen.add(key)
        out.append(j)
        if len(out) >= n:
            break
    return out


def summarise(name, rows):
    def med(k):
        xs = [r[k] for r in rows if r[k] is not None]
        return (round(st.median(xs), 3), round(min(xs), 3), round(max(xs), 3)) if xs else None
    print(f'== {name}: n={len(rows)}')
    for k in ('faces', 'smooth', 'ncol', 'n90', 'L50', 'L90', 'S50', 'S90', 'textured', 'thin', 'mid', 'medarea'):
        print(f'   {k:9s} median/min/max {med(k)}')


if __name__ == '__main__':
    res = {}
    # real weapons: inventory model only (first model id), newest distinct
    rows = []
    for j in newest('items', 'slot-3', 60, lambda j: j.get('inventory_model', 0) > 0):
        try:
            s = stats(o.model(j['inventory_model'])); s['name'] = j['name']; s['id'] = j['id']; rows.append(s)
        except Exception as e:
            print('skip', j['id'], e)
    res['real_weapons'] = rows; summarise('real weapons (newest 60, ground model)', rows)
    rows = []
    for j in newest('objects', 'shape-10', 150, lambda j: (j.get('faces') or 0) >= 100):
        try:
            s = stats(o.merge([o.model(i) for i in j['model_ids']])); s['name'] = j['name']; s['id'] = j['id']; rows.append(s)
        except Exception as e:
            print('skip', j['id'], e)
    res['real_objects'] = rows; summarise('real objects (newest 150 distinct, shape 10, >=100 faces)', rows)
    rows = []
    for j in newest('npcs', 'combat-000-049', 80):
        try:
            s = stats(o.merge([o.model(i) for i in j['model_ids']])); s['name'] = j['name']; s['id'] = j['id']; rows.append(s)
        except Exception as e:
            print('skip', j['id'], e)
    res['real_npcs'] = rows; summarise('real NPCs (newest 80 distinct, low combat)', rows)
    rows = []
    for f in sorted(E114.glob('m*.bin')):
        s = stats(o.decode_model(f.read_bytes())); s['name'] = f.name; rows.append(s)
    res['e114'] = rows; summarise('E114 exported models (99 chunks of the room)', rows)
    out = pathlib.Path(_AP.fix(r'<ROOT>\measure\first-measure.json'))
    out.parent.mkdir(exist_ok=True)
    out.write_text(json.dumps(res, indent=1))
