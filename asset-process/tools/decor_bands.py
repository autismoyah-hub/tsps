"""Measure every real wall decoration (archive folder shape-4) and floor decoration (shape-22) as the game mounts
it, and write the bands gate.py uses for `original-object --shape 4 | 22`.

usage: decor_bands.py     writes measure/decor-real.json (one row per real object) and tools/decor_bands.json
Each object is taken once per distinct model list, with its definition's recolours and scale applied
(compare.real_model). Objects whose definition carries a placement offset (opcodes 70-72) are left out and counted.
Frame: the stored model frame, which is the mounted frame at rotation 0. Heights are positive up.
  shape 4:  the wall plane is x = -64 (west edge of the tile), the room is on the +x side, the wall runs along z.
            low / high = hanging height band; proud = max x + 64 (front of the piece from the plane);
            gap = min x + 64 (back of the piece from the plane; negative = sunk behind it); thick = x extent;
            width = z extent; zc = middle of the z extent.
  shape 22: low / high = lowest / highest point; ext = x and z extents (pooled: a rotation swaps them);
            over = how far the piece passes its tile's edge on the worst side, 0 when inside.
            Bands come from the pieces that lie ON the floor (low >= 0). The rest go below ground (trapdoors,
            pits, bridges over a drop); our flat floor has no hole for them.
Two rules, written beside every limit in the JSON:
  A  PASS inside the real p10..p90, BORDERLINE inside p2.5..p97.5, FAIL outside. Used for where the piece sits on
     its mount (wall: back of the piece from the wall plane; floor: lowest point, overhang past the tile) and for
     the two quality measures the furniture gate already limits at p90 / p97.5 (main materials, largest triangle).
  B  PASS inside p2.5..p97.5, BORDERLINE inside p0.5..p99.5, FAIL outside. Used for what the piece is (hanging
     heights, how far proud, thickness, width, face count, colours, footprint). With rule A on all of these,
     28% of the real wall pieces themselves failed at least one band; a gate must not fail a quarter of the real game.
One exception, stated in the file: the lower limit of a floor piece's highest point is 1, because a real piece
whose surface is all at height 0 draws nothing on our terrain (tested: object 17648, "empty snapshot").
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, collections
sys.dont_write_bytecode = True
sys.path.insert(0, _AP.fix('<ROOT>/tools'))
import io, contextlib
import osrscache as o, scene, compare
from measure import area

ROOT = pathlib.Path(_AP.fix('<ROOT>'))


def one(j, shape):
    with contextlib.redirect_stdout(io.StringIO()):
        m = compare.real_model(j['id'], shape)
    V, F = m['vertices'], m['faces']
    xs, ys, zs = zip(*V)
    ar = [area(V, f) for f in F]
    tot = sum(ar) or 1
    mats = collections.Counter()
    for c, a in zip(m['colors'], ar):
        mats[c >> 7] += a
    L = sorted((o.hsl_split(c)[2], a) for c, a in zip(m['colors'], ar))
    acc = 0; l50 = L[-1][0]
    for l, a in L:
        acc += a
        if acc >= tot / 2:
            l50 = l; break
    st = j.get('size_tiles') or [1, 1]
    r = dict(id=j['id'], name=j.get('name'), models=j['shape_model_ids'] if 'shape_model_ids' in j else m['object']['shape_model_ids'],
             tiles=st, verts=len(V), faces=len(F),
             low=-max(ys), high=-min(ys), min_x=min(xs), max_x=max(xs), min_z=min(zs), max_z=max(zs),
             colours=len(set(m['colors'])), hue_sat=len(mats),
             main=sum(1 for a in mats.values() if a > 0 and a >= .03 * tot),
             L50=l50, Lmin=min(c & 127 for c in m['colors']), Lmax=max(c & 127 for c in m['colors']),
             dup=len(V) - len(set(V)), zero_area=ar.count(0), flat=sum(1 for t in m['render_types'] if t == 1) / len(F),
             largest=max(ar) / tot, alpha=sum(1 for a in m['alpha'] if a) / len(F),
             textured=sum(1 for t in m['face_textures'] if t != -1) / len(F),
             depth_faces=sum(1 for d in m['depth_bias'] if d), depth_values=sorted(set(m['depth_bias'])),
             priorities=sorted(set(m['priorities'])), animated=bool(j.get('animations')))
    r.update(thick=r['max_x'] - r['min_x'], width=r['max_z'] - r['min_z'], tall=r['high'] - r['low'],
             proud=r['max_x'] + 64, behind=-64 - r['min_x'], zc=(r['min_z'] + r['max_z']) / 2,
             ext_x=r['max_x'] - r['min_x'], ext_z=r['max_z'] - r['min_z'],
             over=max(0, -64 * st[0] - r['min_x'], r['max_x'] - 64 * st[0], -64 * st[1] - r['min_z'], r['max_z'] - 64 * st[1]))
    return r


def pct(xs, q):
    xs = sorted(xs)
    k = (len(xs) - 1) * q / 100
    f = int(k); c = min(f + 1, len(xs) - 1)
    return xs[f] + (xs[c] - xs[f]) * (k - f)


def table(rows, keys, title):
    print(f'\n{title}: n = {len(rows)}')
    print(f'{"measure":10s}' + ''.join(f'{p:>9s}' for p in ('min', 'p2.5', 'p5', 'p10', 'p25', 'p50', 'p75', 'p90', 'p95', 'p97.5', 'max')))
    out = {}
    for k in keys:
        xs = [r[k] for r in rows]
        q = [pct(xs, p) for p in (0, 2.5, 5, 10, 25, 50, 75, 90, 95, 97.5, 100)]
        out[k] = q
        print(f'{k:10s}' + ''.join(f'{v:9.2f}' for v in q))
    return out


if __name__ == '__main__':
    res = {}
    for shape in (4, 22):
        rows, skipped, seen = [], collections.Counter(), set()
        for p in sorted((scene.ARCH / 'objects' / f'shape-{shape}').glob('*.json'), key=lambda p: int(p.stem)):
            j = json.loads(p.read_text(encoding='utf-8'))
            if any(j.get('offset') or []):
                skipped['definition offset'] += 1; continue
            try:
                with contextlib.redirect_stdout(io.StringIO()):
                    jj = scene.object_json(j['id'], shape, quiet=True)
                key = (tuple(jj['shape_model_ids']), tuple(map(tuple, j.get('recolors') or [])), tuple(j.get('scale') or []))
                if key in seen:
                    skipped['same models as an earlier object'] += 1; continue
                seen.add(key)
                rows.append(one(jj, shape))
            except (Exception, SystemExit) as e:
                skipped[f'error {type(e).__name__}'] += 1
        res[str(shape)] = rows
        print(f'\n===== shape {shape}: {len(rows)} distinct real objects measured; left out: {dict(skipped)}')
    (ROOT / 'measure').mkdir(exist_ok=True)
    (ROOT / 'measure' / 'decor-real.json').write_text(json.dumps(res))
    w = res['4']; allf = res['22']
    f = [r for r in allf if r['low'] >= 0]
    flat = [r for r in f if r['high'] <= 8]
    for r in w: r['gap'] = -r['behind']
    keys = ['faces', 'colours', 'hue_sat', 'main', 'L50', 'largest', 'flat', 'dup', 'zero_area']
    tw = table(w, ['low', 'high', 'tall', 'gap', 'proud', 'thick', 'width', 'zc'] + keys, 'shape 4, all')
    table(allf, ['low', 'high', 'tall', 'ext_x', 'ext_z', 'over'] + keys, 'shape 22, all')
    tf = table(f, ['low', 'high', 'tall', 'ext_x', 'ext_z', 'over'] + keys, 'shape 22, lying on the floor (low >= 0)')
    tflat = table(flat, ['low', 'high', 'tall', 'ext_x', 'ext_z', 'over'] + keys, 'shape 22, flat pieces on the floor (high <= 8)')
    ext = [r[k] for r in f for k in ('ext_x', 'ext_z')]
    for name, rows in (('4', w), ('22', allf)):
        n = len(rows)
        print(f'\nshape {name}: with alpha faces {sum(1 for r in rows if r["alpha"])}/{n}; textured {sum(1 for r in rows if r["textured"])}/{n}; '
              f'with depth offsets {sum(1 for r in rows if r["depth_faces"])}/{n}; negative depth offsets {sum(1 for r in rows if any(d < 0 for d in r["depth_values"]))}/{n}; '
              f'animated {sum(1 for r in rows if r["animated"])}/{n}; more than one tile {sum(1 for r in rows if r["tiles"] != [1, 1])}/{n}')
        print('  depth values:', collections.Counter(d for r in rows for d in r['depth_values']).most_common(12))

    A = ((2.5, 10, 90, 97.5), 'PASS p10..p90, BORDERLINE p2.5..p97.5')        # where the piece sits; the furniture gate's own rule
    B = ((0.5, 2.5, 97.5, 99.5), 'PASS p2.5..p97.5, BORDERLINE p0.5..p99.5')  # what the piece is

    def both(values, rule):
        return dict(limits=[round(pct(values, q), 2) for q in rule[0]], pct=rule[1])

    def upper(values, rule):
        return dict(limits=[None, None] + [round(pct(values, q), 2) for q in rule[0][2:]], pct=rule[1].replace('p10..', '<= ').replace('p2.5..', '<= ').replace('p0.5..', '<= '))

    col = lambda rows, k: [r[k] for r in rows]
    high22 = both(col(f, 'high'), B)
    measured_high = list(high22['limits'])
    high22['limits'][0] = high22['limits'][1] = 1.0
    bands = {
        'rules': {'A': A[1] + ' (where the piece sits on its mount, and the two quality measures the furniture gate already limits this way)',
                  'B': B[1] + ' (what the piece is: heights, sizes, counts)'},
        'made_by': 'tools/decor_bands.py from the revision-239 cache; one row per real piece in measure/decor-real.json',
        '4': dict(n=len(w), population='wall decorations',
                  measures=dict(low=both(col(w, 'low'), B), high=both(col(w, 'high'), B), gap=both(col(w, 'gap'), A),
                                proud=both(col(w, 'proud'), B), thick=both(col(w, 'thick'), B), width=both(col(w, 'width'), B),
                                zc=both(col(w, 'zc'), B), faces=both(col(w, 'faces'), B), colours=upper(col(w, 'colours'), B),
                                main=upper(col(w, 'main'), A), largest=upper(col(w, 'largest'), A))),
        '22': dict(n=len(f), n_all=len(allf), n_flat=len(flat), population='floor decorations lying on the ground',
                   measures=dict(low=both(col(f, 'low'), A), high=high22, ext=both(ext, B), over=upper(col(f, 'over'), A),
                                 faces=both(col(f, 'faces'), B), colours=upper(col(f, 'colours'), B),
                                 main=upper(col(f, 'main'), A), largest=upper(col(f, 'largest'), A)),
                   note=f'high: both lower limits set to 1; measured they were {measured_high[0]} and {measured_high[1]}. p10 of the '
                        f'{len(flat)} flat pieces (no higher than 8) is {pct(col(flat, "high"), 10):g}, and a surface at height 0 is not drawn on our terrain'),
    }
    (ROOT / 'tools' / 'decor_bands.json').write_text(json.dumps(bands, indent=1) + '\n')
    print('\nwrote', ROOT / 'tools' / 'decor_bands.json')
    for shape in ('4', '22'):
        print(f'shape {shape}, n = {bands[shape]["n"]} {bands[shape]["population"]}')
        for k, v in bands[shape]['measures'].items():
            print(f'  {k:8s} {str(v["limits"]):40s} {v["pct"]}')
