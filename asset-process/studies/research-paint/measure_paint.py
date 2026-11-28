"""Step 1: measure how paint is organised on the game's best creatures and on ours.

usage: measure_paint.py verify            print each real id with the name the cache's census gives it (must match)
       measure_paint.py run               analyse all -> data/paint.json
       measure_paint.py table [--all]     real band (10th-90th centile, median) beside ours; measures ranked by how far
                                          the second watcher sits outside the real band, in units of the real spread
       measure_paint.py identity          within-creature against between-creature distance of part statistics
"""
import sys, pathlib, json
import numpy as np
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import paintlib as pl
B = HERE.parent
# id: (expected name, set). 'owner' = the owner's named references (raids, Nex, Nightmare, Doom, Maggot King)
REAL = {
    7541: ('Tekton', 'owner'), 7566: ('Vasa Nistirio', 'owner'), 7530: ('Vespula', 'owner'), 7527: ('Vanguard', 'owner'), 7563: ('Muttadile', 'owner'),
    8360: ('The Maiden of Sugadinti', 'owner'), 8359: ('Pestilent Bloat', 'owner'), 8355: ('Nylocas Vasilias', 'owner'), 8388: ('Sotetseg', 'owner'),
    8340: ('Xarpus', 'owner'), 8370: ('Verzik Vitur', 'owner'), 8372: ('Verzik Vitur', 'owner'), 8374: ('Verzik Vitur', 'owner'),
    11789: ('Akkha', 'owner'), 11778: ('Ba-Ba', 'owner'), 11719: ('Kephri', 'owner'), 11730: ('Zebak', 'owner'),
    11753: ("[quoted words withheld]", 'owner'), 11756: ("[quoted words withheld]", 'owner'),
    11278: ('Nex', 'owner'), 9425: ('The Nightmare', 'owner'), 14707: ('Doom of Mokhaiotl', 'owner'), 15742: ('Maggot King', 'owner'),
    8061: ('Vorkath', 'other'), 8615: ('Alchemical Hydra', 'other'), 5862: ('Cerberus', 'other'), 12191: ('Duke Sucellus', 'other'),
    12214: ('The Leviathan', 'other'), 12204: ('The Whisperer', 'other'), 12223: ('Vardorvis', 'other'), 12821: ('Sol Heredit', 'other'),
    13668: ('Araxxor', 'other'), 14009: ('The Hueycoatl', 'other'), 14176: ('Yama', 'other'), 319: ('Corporeal Beast', 'other'), 2042: ('Zulrah', 'other'),
    12812: ('Minotaur', 'mid'), 13599: ('Tormented Demon', 'mid'), 9293: ('Basilisk Knight', 'mid'),
}
OURS = {
    'watcher-1': 'def:' + str(B / 'creature-stone-watcher' / 'creature-stone-watcher.json'),
    'watcher-2': 'def:' + str(B / 'creature-stone-watcher-2' / 'creature-stone-watcher-2.json'),
    'star-hound': 'def:' + str(B / 'creature-star-hound-3' / 'creature-star-hound-3.json'),
    'zodiac-ray': 'def:' + str(B / 'research-rig2' / 'ray' / 'work' / 'r8' / 'export' / 'zodiac-ray.json'),
    'warden-general-2': 'def:' + str(B / 'boss-warden-general-2' / 'eclipse.json'),
}
EXTRA = {}       # filled by later steps (repaints) through data/extra.json
def extra():
    p = HERE / 'data' / 'extra.json'
    return json.loads(p.read_text()) if p.exists() else {}

def census_names():
    out = {}
    for line in (B / 'research-npc' / 'census' / 'npcs.jsonl').read_text(encoding='utf-8').splitlines():
        d = json.loads(line); out[d['id']] = d
    return out

def verify():
    cn = census_names(); bad = 0
    for i, (name, grp) in REAL.items():
        d = cn.get(i); got = d['name'] if d else None
        ok = got is not None and (got == name or name in got or got in name)
        bad += not ok
        print(f"{i:6d} {grp:6s} expected {name!r:30s} census {got!r:30s} models {d['models'] if d else None} size {d['size'] if d else None} {'OK' if ok else 'MISMATCH'}")
    print('mismatches:', bad)

def run(keys=None):
    out = {}
    p = HERE / 'data' / 'paint.json'
    if p.exists() and keys: out = json.loads(p.read_text())
    todo = [(str(i), f'npc:{i}', REAL[i][0] + (f' {i}' if REAL[i][0] == 'Verzik Vitur' else ''), REAL[i][1]) for i in REAL] + [(k, e, k, 'ours') for k, e in OURS.items()] + [(k, e, k, 'ours') for k, e in extra().items()]
    for key, entry, name, grp in todo:
        if keys and key not in keys: continue
        sk = pl.skin(entry); a = pl.analyse(sk)
        out[key] = dict(entry=entry, name=name, group=grp, m=a['m'], parts=[{k: v for k, v in r.items() if k != 'colours'} for r in a['parts']],
                        mats=[{k: v for k, v in d.items() if k != 'shares'} for d in a['mats'] if d['share'] >= pl.SIG])
        print(f"{name:26s} faces {a['m']['skin_faces']:5d} translucent {a['m']['translucent_share']:.2f} materials {a['m']['materials']} marks {a['m']['marks']} overlap {a['m']['palette_overlap']:.2f}", flush=True)
    (HERE / 'data').mkdir(exist_ok=True); p.write_text(json.dumps(pl.jsonable(out), indent=1))

def load(): return json.loads((HERE / 'data' / 'paint.json').read_text())

def band(d, key, groups=('owner', 'other', 'mid')):
    x = np.array([v['m'][key] for v in d.values() if v['group'] in groups and isinstance(v['m'].get(key), (int, float))], float)
    if len(x) < 5: return None
    return dict(p10=float(np.quantile(x, .1)), p50=float(np.median(x)), p90=float(np.quantile(x, .9)), lo=float(x.min()), hi=float(x.max()), n=len(x), sd=float(x.std()))

def out_by(b, v):
    """How far v lies outside the band's 10th-90th centile, in units of that band's width (0 = inside)."""
    if v is None or b is None: return None
    wdt = max(b['p90'] - b['p10'], 1e-9)
    return (v - b['p90']) / wdt if v > b['p90'] else (v - b['p10']) / wdt if v < b['p10'] else 0.0

def table(show_all=False, focus='watcher-2'):
    d = load(); ours = [k for k, v in d.items() if v['group'] == 'ours']
    keys = [k for k, v in d['7541']['m'].items() if isinstance(v, (int, float))]
    rows = []
    for k in keys:
        b = band(d, k)
        if b is None: continue
        o = {n: d[n]['m'].get(k) for n in ours}
        rows.append((k, b, o, out_by(b, o.get(focus))))
    rows.sort(key=lambda r: -abs(r[3] or 0))
    print(f"{'measure':34s} {'real p10':>8s} {'median':>8s} {'p90':>8s} {'min':>7s} {'max':>7s} | " + ' '.join(f'{n[:11]:>11s}' for n in ours) + f' | {focus} outside by (band widths)')
    for k, b, o, z in rows:
        if not show_all and abs(z or 0) < 0.01 and not any(abs(out_by(b, v) or 0) > 0 for v in o.values()): continue
        cell = lambda n: (f"{o[n]:10.3f}" + ('*' if out_by(b, o[n]) else ' ')) if o[n] is not None else '          -'
        print(f"{k:34s} {b['p10']:8.3f} {b['p50']:8.3f} {b['p90']:8.3f} {b['lo']:7.2f} {b['hi']:7.2f} | " + ' '.join(cell(n) for n in ours) + f" | {z if z is None else round(z, 2)}")
    print('* = outside the real 10th-90th centile band')

PV = ['light_mean', 'light_sd_main', 'step_med', 'patch_area_med_e4', 'boundary_share', 'speckle', 'edge_light', 'fold_paint', 'n_colours', 'boundary_on_fold']
def identity():
    """For each creature: mean distance between its own parts' paint statistics (z-scored over all parts of all real
    creatures) against the mean distance from its parts to other creatures' parts. Under 1 = its parts are more
    like each other than like other creatures (a style of its own, held across the body)."""
    d = load(); X, own = [], []
    for k, v in d.items():
        for r in v['parts']:
            if r.get('share', 0) < 0.03: continue
            x = [r.get(q) for q in PV]
            if any(q is None for q in x): continue
            x[3] = np.log10(max(x[3], 1e-3)); X.append(x); own.append(k)
    X = np.array(X, float); own = np.array(own)
    real = np.array([d[k]['group'] != 'ours' for k in own])
    mu, sd = X[real].mean(0), X[real].std(0) + 1e-9; Z = (X - mu) / sd
    res = {}
    for k in d:
        s = own == k
        if s.sum() < 3: continue
        A = Z[s]; within = np.mean([np.linalg.norm(a - b) for i, a in enumerate(A) for b in A[i + 1:]])
        O = Z[real & ~s]; between = float(np.mean(np.linalg.norm(A[:, None, :] - O[None], axis=2)))
        res[k] = (within, between, within / between, int(s.sum()))
    rr = np.array([v[2] for k, v in res.items() if d[k]['group'] != 'ours']); rw = np.array([v[0] for k, v in res.items() if d[k]['group'] != 'ours'])
    print(f'real: within/between ratio median {np.median(rr):.2f} (10th-90th {np.quantile(rr, .1):.2f}-{np.quantile(rr, .9):.2f}); within-distance median {np.median(rw):.2f} ({np.quantile(rw, .1):.2f}-{np.quantile(rw, .9):.2f})')
    for k, v in sorted(res.items(), key=lambda kv: kv[1][2]):
        print(f"  {d[k]['name']:26s} {d[k]['group']:6s} within {v[0]:.2f} between {v[1]:.2f} ratio {v[2]:.2f} parts {v[3]}")
    return res

if __name__ == '__main__':
    a = sys.argv[1:]
    if a[0] == 'verify': verify()
    elif a[0] == 'run': run(a[1:] or None)
    elif a[0] == 'table': table('--all' in a, focus=a[a.index('--focus') + 1] if '--focus' in a else 'watcher-2')
    elif a[0] == 'identity': identity()
