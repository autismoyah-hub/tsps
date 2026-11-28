"""stylecheck.py DEF.json PLAN.json [--json OUT] : is a finished creature painted in ONE consistent style?

Two sets of lines. Exit 0 if no line FAILS, 2 otherwise (WARN does not fail).
 PLAN  the creature against its own paint plan (paintplan.py format), part by part:
       colours that are in no ramp of the plan; ramps or tones a part may not use; motifs the plan gives a part that
       it lacks, and motifs that appear on one part only; light edges and dark recesses per part against the plan's
       shares; where the marks are against the plan's focal parts; left against right.
 BAND  the creature against the real band: the 10th to 90th centile of the 39 real high-end creatures measured in
       data/paint.json (measure_paint.py), on the measures PAINT.md names. Outside on the bad side = FAIL for the
       measures that separated our rejected creatures from the real ones, WARN for the rest.
The plan's parts are used for PLAN lines; the eight automatic zones (as for the real band) for BAND lines.
"""
import sys, pathlib, json, collections
import numpy as np
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import paintlib as pl, paintplan, measure_paint as mp

# key, bad side ('low' / 'high' / 'both'), FAIL or WARN, words
BAND = [
    ('mark_single_share', 'high', 'FAIL', 'marks that are a single triangle (splinters)'),
    ('speckle', 'high', 'FAIL', 'surface in one-triangle patches'),
    ('mirror_paint_near', 'low', 'FAIL', 'mirrored surface painted alike, left against right'),
    ('planes_toned', 'low', 'FAIL', 'fold length (30 degrees or more) where the two planes carry different paint'),
    ('form_ratio', 'low', 'FAIL', 'paint changes at folds against inside flat surfaces (ratio)'),
    ('focal_part_mark_share', 'low', 'FAIL', 'share of all marks on the densest of eight zones'),
    ('boundary_density_cv', 'low', 'WARN', 'spread of paint-boundary density over the zones (calm against busy)'),
    ('mat2_share', 'low', 'WARN', 'second material, share of skin'),
    ('materials', 'both', 'WARN', 'materials holding 3% of the skin or more'),
    ('ramp_step_cv_mean', 'low', 'WARN', 'unevenness of ramp steps (0 = a ruler-made ramp)'),
    ('step_med', 'both', 'WARN', 'median lightness step across a tone boundary'),
    ('flat_soft', 'high', 'WARN', 'flat surface cut by soft same-material boundaries (mottle)'),
    ('edge_light', 'low', 'WARN', 'convex fold length carrying a tone 4 or more above its part'),
    ('recess_dark', 'low', 'WARN', 'concave fold length carrying a tone 4 or more below its part'),
    ('motif_recurrence', 'low', 'WARN', 'share of zones that carry each motif the creature uses'),
    ('orphan_colour_share', 'high', 'WARN', 'surface in colours used on one zone only'),
    ('marks', 'both', 'WARN', 'marks (small contrasting patches)'),
    ('mark_faces_med', 'low', 'WARN', 'faces in the median mark'),
]

def check(def_json, plan_path, quiet=False):
    plan = json.loads(pathlib.Path(plan_path).read_text())
    sk = pl.skin('def:' + str(pathlib.Path(def_json).resolve()))
    lines = []; say = lambda kind, level, text: lines.append((kind, level, text))
    # ------------------------------------------------------------ PLAN
    part, names = paintplan.parts_from_plan(sk, plan)
    a = pl.analyse(sk, part, names); idx = paintplan.ramp_index(plan); m = a['m']
    C, area = sk['C'], sk['area']; tot = sk['tot']
    off = collections.Counter(); offa = 0.0
    for c, w in zip(C, area):
        if int(c) not in idx: off[int(c)] += w; offa += w
    say('PLAN', 'FAIL' if offa / tot > 0.005 else 'OK', f"{len(off)} colours in no ramp of the plan, {100 * offa / tot:.1f}% of the skin" + (': ' + ' '.join('%d.%d.%d' % pl.hsl(c) for c, _ in off.most_common(6)) if off else ''))
    total_marks = max(m['marks'], 1)
    motif_parts = collections.defaultdict(list)
    for j, r in enumerate(a['parts']):
        n = r['name']; spec = plan['paint'].get(n)
        if r.get('share', 0) <= 0.005: continue
        if spec is None:
            if r['share'] > 0.02: say('PLAN', 'WARN', f"{n}: {100 * r['share']:.0f}% of the skin lies on no part of the plan")
            continue
        s = part == j; pa = area[s].sum()
        out_tones = collections.Counter(); bad_ramp = collections.Counter(); offp = collections.Counter()
        for c, w in zip(C[s], area[s]):
            if int(c) not in idx: offp[int(c)] += w; continue
            mat, t = idx[int(c)]
            if mat not in spec['ramps']: bad_ramp[mat] += w
            elif not (spec['ramps'][mat][0] <= t <= spec['ramps'][mat][1]): out_tones[(mat, t)] += w
        if offp and sum(offp.values()) / pa > 0.005: say('PLAN', 'FAIL', f"{n} uses {len(offp)} colours outside every ramp ({100 * sum(offp.values()) / pa:.0f}% of the part)")
        for mat, w in bad_ramp.items():
            if w / pa > 0.01: say('PLAN', 'FAIL', f"{n} carries {mat} on {100 * w / pa:.0f}% of its surface; the plan gives it no {mat}")
        by = collections.defaultdict(list)
        for (mat, t), w in out_tones.items():
            if w / pa > 0.003: by[mat].append((t, w))
        for mat, v in by.items():
            w = sum(x[1] for x in v)
            say('PLAN', 'FAIL' if w / pa > 0.03 else 'WARN', f"{n} uses {len(v)} tones outside its range of the {mat} ramp (tones {sorted(t for t, _ in v)}, allowed {spec['ramps'][mat][0]}..{spec['ramps'][mat][1]}; {100 * w / pa:.0f}% of the part)")
        for k in spec.get('motifs', []):
            if r['motifs'].get(k, 0) == 0: say('PLAN', 'WARN', f"{n}: the plan gives it '{k}' and it has none")
        for k, cnt in r['motifs'].items():
            if cnt >= 1: motif_parts[k].append((n, cnt))
        le, dr = r.get('edge_light'), r.get('recess_dark')
        want = plan.get('edges', {})
        if le is not None and le < want.get('light_edge', 0) * 0.6: say('PLAN', 'FAIL', f"no light edge on {100 * (1 - le):.0f} percent of convex fold length on the {n} (plan: light on {100 * want['light_edge']:.0f} percent)")
        if dr is not None and dr < want.get('dark_recess', 0) * 0.6: say('PLAN', 'FAIL', f"no dark tone in {100 * (1 - dr):.0f} percent of concave fold length on the {n} (plan: dark in {100 * want['dark_recess']:.0f} percent)")
        share = r['marks'] / total_marks; dens = share / max(r['share'], 1e-9)
        if spec.get('focal') and dens < 1.0: say('PLAN', 'FAIL', f"{n} is a focal part and holds {100 * share:.0f}% of the marks on {100 * r['share']:.0f}% of the skin: no denser than the average")
        if not spec.get('focal') and dens > 1.5: say('PLAN', 'FAIL', f"{n} is a calm part and holds {100 * share:.0f}% of the marks on {100 * r['share']:.0f}% of the skin: {dens:.1f} times the average density")
    nparts = sum(1 for r in a['parts'] if r.get('share', 0) > 0.03 and r['name'] in plan['paint'])
    for k, v in motif_parts.items():
        tot_k = sum(c for _, c in v)
        if len(v) == 1 and tot_k >= 3 and nparts >= 3: say('PLAN', 'WARN', f"'{k}' marks appear on the {v[0][0]} only ({tot_k} of them)")
    focal = [n for n, p in plan['paint'].items() if p.get('focal')]
    fshare = sum(r['marks'] for r in a['parts'] if r['name'] in focal) / total_marks
    farea = sum(r.get('share', 0) for r in a['parts'] if r['name'] in focal)
    say('PLAN', 'OK' if fshare >= 1.25 * farea else 'FAIL', f"focal parts ({', '.join(focal)}) hold {100 * fshare:.0f}% of the marks on {100 * farea:.0f}% of the skin")
    # left against right
    rows = {r['name']: r for r in a['parts']}
    for n in list(rows):
        if n.startswith('left ') and 'right ' + n[5:] in rows:
            A, Bq = rows[n], rows['right ' + n[5:]]
            if A.get('share', 0) <= 0 or Bq.get('share', 0) <= 0: continue
            ov = pl._hist_inter(A['colours'], Bq['colours'])
            say('PLAN', 'OK' if ov >= 0.75 else 'FAIL', f"{n} and {'right ' + n[5:]} share {100 * ov:.0f}% of their paint (same colours in the same shares)")
    sym = plan.get('symmetry')
    if sym is not None and m['mirror_geometry'] >= 0.6:
        say('PLAN', 'OK' if m['mirror_paint_near'] >= sym else 'FAIL', f"mirrored surface painted alike: {m['mirror_paint_near']:.2f} (plan: at least {sym})")
    # ------------------------------------------------------------ BAND
    d = mp.load(); ab = pl.analyse(sk)['m']; band_rows = []
    for key, side, level, words in BAND:
        b = mp.band(d, key); v = ab.get(key)
        if b is None or v is None: continue
        if key == 'mirror_paint_near' and ab['mirror_geometry'] < 0.6: continue
        bad = (side in ('low', 'both') and v < b['p10'] - 1e-9) or (side in ('high', 'both') and v > b['p90'] + 1e-9)
        band_rows.append(dict(key=key, value=v, p10=b['p10'], p50=b['p50'], p90=b['p90'], verdict=level if bad else 'OK'))
        say('BAND', level if bad else 'OK', f"{words} [{key}] = {v:.3f} | real {b['p10']:.3f} .. {b['p90']:.3f}, median {b['p50']:.3f}")
    pf = sum(1 for k, l, _ in lines if l == 'FAIL' and k == 'PLAN'); bf = sum(1 for k, l, _ in lines if l == 'FAIL' and k == 'BAND')
    warns = sum(1 for _, l, _ in lines if l == 'WARN')
    # Verdict: no PLAN line may fail; at most ONE BAND line may (calibrate.py: 24 of the 39 real creatures break
    # none, 10 break one, 5 break two or three; our rejected creatures broke three to five).
    fails = pf + max(bf - 1, 0)
    if not quiet:
        print(f"== stylecheck {def_json}\n   plan {plan_path}")
        for kind, level, text in lines: print(f"{kind:4s} {level:4s} {text}")
        print(f"PLAN FAIL {pf}   BAND FAIL {bf} (one allowed: 34 of 39 real creatures break at most one)   WARN {warns}   -> {'PASS' if fails == 0 else 'NOT PASSED'}" + ('  (strict: a BAND line still fails)' if fails == 0 and bf else ''))
    return fails, warns, lines, band_rows

if __name__ == '__main__':
    a = sys.argv[1:]
    fails, warns, lines, rows = check(a[0], a[1])
    if '--json' in a: pathlib.Path(a[a.index('--json') + 1]).write_text(json.dumps(pl.jsonable(dict(fails=fails, warns=warns, lines=lines, band=rows)), indent=1))
    sys.exit(0 if fails == 0 else 2)
