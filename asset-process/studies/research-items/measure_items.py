"""measure_items.py: what real items consist of, measured over every item record of the cache.

usage: measure_items.py            (prints the tables; writes data/measure-items.json)

1. which definition fields real items use (all plain items; wearables by slot);
2. the inventory model against the male worn model: the same model id, a re-posed copy (same topology, a rigid
   move fits), or a different mesh; face counts, sizes, where the origin sits;
3. the female worn model against the male, by slot: same ids, same mesh moved, same mesh scaled per axis, the same
   topology reshaped, or a different mesh;
4. chat-head models against the worn helm.
Fits are least squares over vertices paired by index (only tried when vertex and face counts agree and the face
index lists are equal). "rigid" = rotation + translation (Kabsch); "axis" = per-axis scale + translation.
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections, statistics as st
import numpy as np
import itemlib as L

IT = L.items()
plain = [d for d in IT.values() if L.plain(d)]
out = {}
med = lambda v: round(float(st.median(v)), 2) if len(v) else None
q = lambda v, p: round(float(np.percentile(v, p)), 2) if len(v) else None


def section(t):
    print('\n' + '=' * 100 + '\n' + t + '\n' + '=' * 100)


# ---------------------------------------------------------------- 1. fields
section('1. Definition fields in use')
print(f'{len(IT)} records; {len(plain)} plain named items; '
      f'{sum(d["note"][1] != -1 for d in IT.values())} note templates, {sum(d["bought"][1] != -1 for d in IT.values())} bought, '
      f'{sum(d["placeholder"][1] != -1 for d in IT.values())} placeholders')
ops = collections.Counter()
for d in IT.values():
    raw = bytes.fromhex(d['raw'])
    if raw: ops[raw[0]] += 0
wear = collections.defaultdict(list)
for d in plain:
    wear[d['wear_positions'][0]].append(d)
print('plain items by first wear position:', {L.SLOT_NAMES.get(k, k): len(v) for k, v in sorted(wear.items())})


def usage(ds, label):
    n = len(ds)
    if not n: return
    f = lambda pred: f'{sum(1 for d in ds if pred(d)) * 100 / n:5.1f}%'
    print(f'{label:22s} n={n:5d} | recolour {f(lambda d: d["recolor"])} retexture {f(lambda d: d["retexture"])} '
          f'zan!=0 {f(lambda d: d["angles"][2])} offset!=0 {f(lambda d: any(d["offset"]))} scale!=128 {f(lambda d: d["scale"] != [128] * 3)} '
          f'ambient {f(lambda d: d["ambient"])} contrast {f(lambda d: d["contrast"])} stackable {f(lambda d: d["stackable"])} '
          f'male2 {f(lambda d: d["wear_models"][0][1] != -1)} male3 {f(lambda d: d["wear_models"][0][2] != -1)} '
          f'female {f(lambda d: d["wear_models"][1][0] != -1)} maleoff {f(lambda d: d["wear_offset"][0])} femoff {f(lambda d: d["wear_offset"][1])} '
          f'head {f(lambda d: d["head_models"][0][0] != -1)} head2 {f(lambda d: d["head_models"][0][1] != -1)}')


usage(plain, 'all plain')
for k in sorted(wear):
    if k != -1: usage(wear[k], f'slot {k} {L.SLOT_NAMES.get(k, "?")}')
wp = collections.defaultdict(collections.Counter)
for d in plain:
    if d['wear_positions'][0] != -1:
        wp[d['wear_positions'][0]][tuple(d['wear_positions'][1:])] += 1
print('second and third wear positions (slots blanked), by slot:')
for k in sorted(wp):
    print(f'  {L.SLOT_NAMES.get(k, k)}: ' + ', '.join(f'{list(a)} x{n}' for a, n in wp[k].most_common(6)))
fo = collections.Counter((d['wear_offset'][0], d['wear_offset'][1]) for d in plain if d['wear_models'][0][0] != -1)
print('wear offsets (male, female) among wearables:', fo.most_common(8))
zooms = {L.SLOT_NAMES.get(k, k): (med([d['zoom'] for d in v]), q([d['zoom'] for d in v], 10), q([d['zoom'] for d in v], 90)) for k, v in wear.items()}
print('zoom median (10th, 90th) by slot:', zooms)
out['counts'] = {str(k): len(v) for k, v in wear.items()}


# ---------------------------------------------------------------- fits
def same_topology(a, b):
    return a is not None and b is not None and len(a['V']) == len(b['V']) and len(a['F']) == len(b['F']) and np.array_equal(a['F'], b['F'])


def rigid_fit(A, B):
    """Best rotation R (det +1 or -1 allowed: mirror reported) and t with B ~ A R + t; returns rms, max, R, t, uniform scale."""
    A = A.astype(float); B = B.astype(float)
    ca, cb = A.mean(0), B.mean(0)
    H = (A - ca).T @ (B - cb)
    U, S, Vt = np.linalg.svd(H)
    R = U @ Vt
    sa = ((A - ca) ** 2).sum()
    scale = S.sum() / sa if sa > 0 else 1.0
    best = None
    for s in (1.0, scale):
        P = (A - ca) @ R * s + cb
        e = np.sqrt(((P - B) ** 2).sum(1))
        r = (float(np.sqrt((e ** 2).mean())), float(e.max()), R, cb - ca @ R * s, s)
        if best is None or r[0] < best[0] - 0.05: best = r
    return best


def axis_fit(A, B):
    """Per-axis scale and shift: B[:,k] ~ a_k A[:,k] + b_k. Returns rms, max, scales, shifts."""
    A = A.astype(float); B = B.astype(float)
    P = np.empty_like(B); sc = []; sh = []
    for k in range(3):
        va = A[:, k].var()
        a = ((A[:, k] - A[:, k].mean()) * (B[:, k] - B[:, k].mean())).mean() / va if va > 1e-9 else 1.0
        b = B[:, k].mean() - a * A[:, k].mean()
        P[:, k] = a * A[:, k] + b; sc.append(a); sh.append(b)
    e = np.sqrt(((P - B) ** 2).sum(1))
    return float(np.sqrt((e ** 2).mean())), float(e.max()), sc, sh


def euler_deg(R):
    """Describe a rotation matrix (row-vector convention) loosely: angle of rotation in degrees."""
    c = (np.trace(R) - 1) / 2
    return float(np.degrees(np.arccos(max(-1, min(1, c)))))


# ---------------------------------------------------------------- 2. inventory vs male worn
section('2. Inventory model against the male worn model')
rows2 = collections.defaultdict(list)
for k in sorted(wear):
    if k == -1: continue
    for d in wear[k]:
        ids = L.worn_ids(d, 0)
        if not ids: continue
        inv = L.model(d['model'])
        if inv is None or len(inv["V"]) == 0: continue
        worn = L.concat(ids)
        if worn is None or len(worn["V"]) == 0: continue
        r = dict(id=d['id'], name=d['name'], inv_faces=len(inv['F']), worn_faces=len(worn['F']), inv_verts=len(inv['V']), worn_verts=len(worn['V']),
                 n_worn_models=len(ids))
        if ids == [d['model']]:
            r['kind'] = 'same id'
        elif same_topology(inv, worn):
            rms, mx, R, t, s = rigid_fit(worn['V'], inv['V'])
            r.update(rigid_rms=rms, rigid_max=mx, angle=euler_deg(R), det=float(np.linalg.det(R)), shift=[float(x) for x in t], scale=s)
            if np.array_equal(inv['V'], worn['V']): r['kind'] = 'identical copy'
            elif mx <= 2.0 and abs(s - 1) < 0.02 and euler_deg(R) < 0.5: r['kind'] = 'moved copy'
            elif mx <= 2.5: r['kind'] = 'reposed copy'
            else: r['kind'] = 'same topology, reshaped'
        else:
            r['kind'] = 'different mesh'
        Vi = inv['V'] * (np.array(d['scale']) / 128.0)
        lo, hi = Vi.min(0), Vi.max(0)
        r['inv_extent'] = [float(x) for x in (hi - lo)]
        r['inv_centre'] = [float(x) for x in (hi + lo) / 2]
        r['origin_inside'] = bool(np.all(lo <= 0) and np.all(hi >= 0))
        lw, hw = worn['V'].min(0), worn['V'].max(0)
        r['worn_extent'] = [float(x) for x in (hw - lw)]
        r['inv_labels'] = bool(inv.get('vertex_skins') is not None and any(inv['vertex_skins']))
        r['inv_priorities'] = sorted(set(inv['priorities'])) if inv.get('priorities') is not None else None
        r['zoom'] = d['zoom']; r['angles'] = d['angles']; r['offset'] = d['offset']
        rows2[k].append(r)
L.save_models()
print('slot      n    | same id  identical  moved  reposed  reshaped  different | faces inv/worn median (different only) | longest extent inv/worn median')
for k, rs in rows2.items():
    c = collections.Counter(r['kind'] for r in rs); n = len(rs)
    diff = [r for r in rs if r['kind'] == 'different mesh']
    ratio = med([r['inv_faces'] / r['worn_faces'] for r in diff if r['worn_faces']])
    ext = med([max(r['inv_extent']) / max(r['worn_extent']) for r in rs if max(r['worn_extent']) > 0])
    print(f'{L.SLOT_NAMES.get(k, k):8s} {n:5d} | {c["same id"] * 100 / n:6.1f}% {c["identical copy"] * 100 / n:8.1f}% {c["moved copy"] * 100 / n:6.1f}% '
          f'{c["reposed copy"] * 100 / n:7.1f}% {c["same topology, reshaped"] * 100 / n:8.1f}% {c["different mesh"] * 100 / n:9.1f}% | {ratio} | {ext}')
print('\nreposed copies: rotation angle (degrees) and scale, by slot')
for k, rs in rows2.items():
    rp = [r for r in rs if r['kind'] in ('reposed copy', 'moved copy')]
    if rp:
        print(f'  {L.SLOT_NAMES.get(k, k):8s} n={len(rp)} angle median {med([r["angle"] for r in rp])} (10th {q([r["angle"] for r in rp], 10)}, 90th {q([r["angle"] for r in rp], 90)}), '
              f'scale median {med([r["scale"] for r in rp])}, mirrored {sum(r["det"] < 0 for r in rp)}')
print('\nwhere the inventory model sits (after the definition scale), by slot: extent x, height, z medians; centre medians; origin inside the box')
for k, rs in rows2.items():
    E = np.array([r['inv_extent'] for r in rs]); C = np.array([r['inv_centre'] for r in rs])
    print(f'  {L.SLOT_NAMES.get(k, k):8s} extent {[med(E[:, i]) for i in range(3)]} centre {[med(C[:, i]) for i in range(3)]} (|centre| / half-diagonal median '
          f'{med(np.linalg.norm(C, axis=1) / (np.linalg.norm(E, axis=1) / 2 + 1e-9))}); origin inside box {sum(r["origin_inside"] for r in rs) * 100 / len(rs):.0f}%; '
          f'carries labels {sum(r["inv_labels"] for r in rs) * 100 / len(rs):.0f}%')
out['inventory_vs_worn'] = {str(k): v for k, v in rows2.items()}

# ---------------------------------------------------------------- 3. female vs male
section('3. Female worn model against the male worn model')
rows3 = collections.defaultdict(list)
for k in sorted(wear):
    if k == -1: continue
    for d in wear[k]:
        mi, fi = L.worn_ids(d, 0), L.worn_ids(d, 1)
        if not mi: continue
        r = dict(id=d['id'], name=d['name'], male=mi, female=fi, off=d['wear_offset'])
        if not fi:
            r['kind'] = 'no female model'
        elif mi == fi:
            r['kind'] = 'same ids' if d['wear_offset'][0] == d['wear_offset'][1] else 'same ids, offset differs'
        else:
            a, b = L.concat(mi), L.concat(fi)
            if a is None or b is None: continue
            r.update(male_faces=len(a['F']), female_faces=len(b['F']))
            if same_topology(a, b):
                A = a['V'] + [0, d['wear_offset'][0], 0]; B = b['V'] + [0, d['wear_offset'][1], 0]
                dlt = B - A
                r['labels_equal'] = list(a['vertex_skins']) == list(b['vertex_skins'])
                if not dlt.any(): r['kind'] = 'identical copy'
                elif (dlt == dlt[0]).all(): r['kind'] = 'moved copy'; r['shift'] = [int(x) for x in dlt[0]]
                else:
                    rms, mx, sc, sh = axis_fit(A, B)
                    r.update(axis_rms=rms, axis_max=mx, scales=sc, shifts=sh, moved_share=float((np.abs(dlt).sum(1) > 0).mean()),
                             disp_med=float(np.median(np.linalg.norm(dlt, axis=1))), disp_max=float(np.linalg.norm(dlt, axis=1).max()))
                    r['kind'] = 'axis-scaled copy' if mx <= 1.5 else 'same topology, reshaped'
            else:
                r['kind'] = 'different mesh'
                la, ha, lb, hb = a['V'].min(0), a['V'].max(0), b['V'].min(0), b['V'].max(0)
                r['extent_ratio'] = [float(x) for x in (hb - lb) / np.maximum(ha - la, 1)]
        rows3[k].append(r)
L.save_models()
kinds = ['same ids', 'same ids, offset differs', 'identical copy', 'moved copy', 'axis-scaled copy', 'same topology, reshaped', 'different mesh', 'no female model']
print(f'{"slot":8s} {"n":>5s} | ' + ' | '.join(kinds))
for k, rs in rows3.items():
    c = collections.Counter(r['kind'] for r in rs); n = len(rs)
    print(f'{L.SLOT_NAMES.get(k, k):8s} {n:5d} | ' + ' | '.join(f'{c[x] * 100 / n:{len(x)}.1f}' for x in kinds))
print('\nfor the remodelled ones (different mesh): female faces / male faces, and female extent / male extent (x, height, z), medians')
for k, rs in rows3.items():
    dm = [r for r in rs if r['kind'] == 'different mesh']
    if dm:
        E = np.array([r['extent_ratio'] for r in dm])
        print(f'  {L.SLOT_NAMES.get(k, k):8s} n={len(dm)} faces ratio {med([r["female_faces"] / r["male_faces"] for r in dm])} '
              f'(10th {q([r["female_faces"] / r["male_faces"] for r in dm], 10)}, 90th {q([r["female_faces"] / r["male_faces"] for r in dm], 90)}), extent ratio {[med(E[:, i]) for i in range(3)]}')
print('\nsame topology, reshaped / axis-scaled: per-axis scale and shift medians (x, y-down, z), share of vertices moved, displacement median and max')
for k, rs in rows3.items():
    rr = [r for r in rs if r['kind'] in ('same topology, reshaped', 'axis-scaled copy')]
    if rr:
        S = np.array([r['scales'] for r in rr]); T = np.array([r['shifts'] for r in rr])
        print(f'  {L.SLOT_NAMES.get(k, k):8s} n={len(rr)} scale {[med(S[:, i]) for i in range(3)]} shift {[med(T[:, i]) for i in range(3)]} moved share {med([r["moved_share"] for r in rr])} '
              f'displacement median {med([r["disp_med"] for r in rr])} max {med([r["disp_max"] for r in rr])}; residual after the axis fit: rms {med([r["axis_rms"] for r in rr])} max {med([r["axis_max"] for r in rr])}; '
              f'labels equal {sum(r["labels_equal"] for r in rr)}/{len(rr)}')
print('\nmoved copies: the shift (x, y-down, z)')
for k, rs in rows3.items():
    mv = collections.Counter(tuple(r['shift']) for r in rs if r['kind'] == 'moved copy')
    if mv: print(f'  {L.SLOT_NAMES.get(k, k):8s} {mv.most_common(6)}')
out['female_vs_male'] = {str(k): v for k, v in rows3.items()}

# ---------------------------------------------------------------- 4. head models
section('4. Chat-head models against the worn helm (slot 0), male')
rows4 = []
for d in wear[0]:
    hi, wi = L.head_ids(d, 0), L.worn_ids(d, 0)
    if not wi: continue
    r = dict(id=d['id'], name=d['name'], worn=wi, head=hi, hides=d['wear_positions'][1:])
    if not hi:
        r['kind'] = 'no head model'
    else:
        h, w = L.concat(hi), L.concat(wi)
        if h is None or w is None: continue
        r.update(head_faces=len(h['F']), worn_faces=len(w['F']))
        if hi == wi: r['kind'] = 'same ids'
        elif same_topology(h, w):
            dlt = h['V'] - w['V']
            r['kind'] = 'identical copy' if not dlt.any() else ('moved copy' if (dlt == dlt[0]).all() else 'same topology, reshaped')
            r['shift'] = [int(x) for x in np.median(dlt, axis=0)]
        else:
            r['kind'] = 'different mesh'
        # most common vertex-to-vertex difference: is the head model the worn one (or part of it) moved?
        cnt = collections.Counter()
        W = w['V']; H = h['V']
        for v in H[:: max(1, len(H) // 60)]:
            for dv in (W - v): cnt[tuple(int(x) for x in dv)] += 1
        (shift, _), = cnt.most_common(1)
        ws = {tuple(int(x) for x in v) for v in W}
        r['best_shift_worn_minus_head'] = list(shift)
        r['head_vertices_on_worn'] = float(np.mean([tuple(int(x) for x in (v + shift)) in ws for v in H]))
        lo, hi_ = H.min(0), H.max(0)
        r['head_box'] = [[int(x) for x in lo], [int(x) for x in hi_]]
        r['head_labels'] = sorted(set(h['vertex_skins']))
        r['head_priorities'] = sorted(set(h['priorities']))
    rows4.append(r)
L.save_models()
c = collections.Counter(r['kind'] for r in rows4); n = len(rows4)
print(f'{n} head-slot items with a male worn model: ' + ', '.join(f'{k} {v} ({v * 100 / n:.1f}%)' for k, v in c.most_common()))
hh = [r for r in rows4 if 'head_faces' in r]
print(f'head faces / worn faces: median {med([r["head_faces"] / r["worn_faces"] for r in hh])} (10th {q([r["head_faces"] / r["worn_faces"] for r in hh], 10)}, 90th {q([r["head_faces"] / r["worn_faces"] for r in hh], 90)})')
sh = collections.Counter(tuple(r['best_shift_worn_minus_head']) for r in hh)
print('most common (worn - head) vertex shift:', sh.most_common(6))
on = [r['head_vertices_on_worn'] for r in hh]
print(f'share of head-model vertices that land on a worn-model vertex after that shift: median {med(on)}, 10th {q(on, 10)}; '
      f'items with 95% or more: {sum(x >= 0.95 for x in on)} of {len(on)}; under 50%: {sum(x < 0.5 for x in on)}')
byhide = collections.defaultdict(list)
for r in rows4: byhide[tuple(r['hides'])].append(r)
for hk, rs in sorted(byhide.items(), key=lambda kv: -len(kv[1]))[:6]:
    cc = collections.Counter(r['kind'] for r in rs)
    print(f'  helms that blank {list(hk)}: n={len(rs)} ' + ', '.join(f'{k} {v}' for k, v in cc.most_common()))
B = np.array([r['head_box'] for r in hh])
print('head model box medians: min', [med(B[:, 0, i]) for i in range(3)], 'max', [med(B[:, 1, i]) for i in range(3)])
print('head model labels (most common sets):', collections.Counter(tuple(r['head_labels']) for r in hh).most_common(5))
print('head model priorities:', collections.Counter(tuple(r['head_priorities']) for r in hh).most_common(5))
fh = [d for d in wear[0] if L.head_ids(d, 1)]
print(f'female head models: {len(fh)} items have one; same ids as the male head model in {sum(L.head_ids(d, 0) == L.head_ids(d, 1) for d in fh)}')
nonhelm = collections.Counter(L.SLOT_NAMES.get(k, k) for k, v in wear.items() for d in v if d['head_models'][0][0] != -1)
print('items with a head model, by slot:', dict(nonhelm))
out['head_vs_worn'] = rows4
(L.HERE / 'data' / 'measure-items.json').write_text(json.dumps(out, default=float))
