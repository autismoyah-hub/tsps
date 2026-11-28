"""The PAINT PLAN: a style sheet an author writes BEFORE painting, and the tool that reads one off a real creature.

usage: paintplan.py read ENTRY OUT.json [--parts humanoid|auto]   write the paint plan a real creature follows
       paintplan.py show PLAN.json                                  print it as a readable sheet

ENTRY = npc:ID | def:PATH.json.  --parts humanoid (labelled bipeds): head, trunk, left arm, right arm, left leg,
right leg from the centres of the vertex labels;  --parts auto (default): eight zones from the face centres.

Format (JSON; every colour is [hue, saturation, lightness] of the game's 16-bit colour):
{
 "name": ..., "source": ...,
 "materials": {                              # one named RAMP per painted substance, dark to light, exact colours
   "stone": {"ramp": [[35,2,10], ...], "share": 0.80, "job": "the body"} , ...},
 "parts": {"head": {"labels": [..]} | {"seed": [x, y, z]}, ...},     # label lists, or zone seeds (nearest seed wins)
 "paint": {                                  # per part: which ramps (and which tones of each), which motifs, focal or calm
   "head": {"ramps": {"stone": [2, 8], "bronze": [0, 5]},   # allowed tone indices lo..hi of each ramp on this part
            "motifs": ["dark line", "accent spot"],          # motif kinds this part carries (paintlib.MOTIFS)
            "focal": true, "marks": 0.30},                   # share of the creature's marks meant to sit here
   ...},
 "edges": {"light_edge": 0.35, "dark_recess": 0.45},   # least share of convex fold length (30 degrees or more) that carries
                                                        # a tone at least 4 lighter than the part's median / of concave fold length 4 darker
 "symmetry": 0.88,                           # least share of mirrored surface painted alike (within 4 lightness)
 "numbers": {...}                            # what the source creature measures (reader only; for study)
}
"""
import sys, pathlib, json, collections
import numpy as np
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import paintlib as pl

def humanoid_labels(sk):
    """Label lists for head, trunk, arms, legs of an upright biped, from the centres of its vertex labels."""
    used = np.unique(sk['F']); V = sk['V'][used]; L = sk['L'][used]
    ground = V[:, 1].max(); H = ground - V[:, 1].min()
    cen = {int(l): V[L == l].mean(0) for l in np.unique(L)}
    xmid = float(np.median([c[0] for c in cen.values()]))
    xr = max(abs(c[0] - xmid) for c in cen.values()) or 1.0
    out = collections.defaultdict(list)
    for l, c in cen.items():
        h = (ground - c[1]) / H; x = (c[0] - xmid) / xr; side = 'left' if x < 0 else 'right'       # the creature's own -x is called left
        if h > 0.72 and abs(x) < 0.5: out['head'].append(l)
        elif abs(x) >= 0.5: out[side + ' arm'].append(l)
        elif h < 0.5: out[side + ' leg'].append(l)
        else: out['trunk'].append(l)
    return {k: sorted(out[k]) for k in ('head', 'trunk', 'left arm', 'right arm', 'left leg', 'right leg') if out[k]}

def parts_from_plan(sk, plan):
    spec = plan['parts']
    if all('labels' in v for v in spec.values()):
        return pl.label_parts(sk, {k: v['labels'] for k, v in spec.items()})
    names = list(spec); seeds = np.array([spec[n]['seed'] for n in names], float)
    return np.argmin(((sk['cen'][:, None, :] - seeds[None]) ** 2).sum(2), axis=1), names

def ramp_index(plan):
    """colour -> (material name, tone index) for every colour of the plan's ramps."""
    out = {}
    for name, mt in plan['materials'].items():
        for i, c in enumerate(mt['ramp']): out[pl.pack(*c)] = (name, i)
    return out

def read(entry, parts='auto'):
    sk = pl.skin(entry)
    if parts == 'humanoid':
        labs = humanoid_labels(sk); part, names = pl.label_parts(sk, labs); spec = {k: dict(labels=v) for k, v in labs.items()}
        if (part == len(labs)).any(): spec['other'] = dict(labels=[])
    else:
        part, cen = pl.auto_parts(sk); names = pl.zone_names(sk, part, cen); spec = {n: dict(seed=[round(float(x), 1) for x in c]) for n, c in zip(names, cen)}
    a = pl.analyse(sk, part, names); m = a['m']
    mats = {}; used = collections.Counter()
    for d in a['mats']:
        if d['share'] < pl.SIG: continue
        n = pl.mat_name(d); used[n] += 1
        if used[n] > 1: n += f' {used[n]}'
        d['plan_name'] = n
        ramp = [list(pl.hsl(c)) for c in d['colours'] if d['shares'][c] >= 0.001]
        mats[n] = dict(ramp=ramp, share=round(d['share'], 3), tones=d['tones'], step=d['step_med'], steps_uneven=round(d['step_cv'], 2), hues=d['hues'], saturations=d['sats'])
    plan = dict(name=entry, source=entry, materials=mats, parts=spec, paint={})
    idx = ramp_index(plan)
    for j, r in enumerate(a['parts']):
        if r.get('share', 0) <= 0: continue
        ramps = {}
        for c, v in r['colours'].items():
            if v < 0.01 or c not in idx: continue
            n, i = idx[c]; lo, hi = ramps.get(n, (i, i)); ramps[n] = (min(lo, i), max(hi, i))
        plan['paint'][r['name']] = dict(ramps={k: list(v) for k, v in ramps.items()}, motifs=[k for k, n in r['motifs'].items() if n >= 1],
                                        focal=bool(r['marks'] / max(m['marks'], 1) >= 1.5 * r['share'] and r['marks'] >= 5), marks=round(r['marks'] / max(m['marks'], 1), 2),
                                        share=round(r['share'], 3), step=r.get('step_med'), light_edge=None if r.get('edge_light') is None else round(r['edge_light'], 2),
                                        dark_recess=None if r.get('recess_dark') is None else round(r['recess_dark'], 2),
                                        planes_toned=None if r.get('planes_toned') is None else round(r['planes_toned'], 2))
    plan['edges'] = dict(light_edge=round(m['edge_light'] or 0, 2), dark_recess=round(m['recess_dark'] or 0, 2))
    plan['symmetry'] = round(m['mirror_paint_near'], 2)
    keep = ('materials', 'mat2_share', 'mat3_share', 'ramp_step_cv_mean', 'step_med', 'planes_toned', 'form_ratio', 'flat_soft', 'speckle', 'marks', 'mark_single_share', 'mark_faces_med',
            'motifs_used', 'motif_recurrence', 'focal_part_mark_share', 'boundary_density_cv', 'palette_overlap', 'orphan_colour_share', 'mirror_geometry', 'mirror_paint_near')
    plan['numbers'] = {k: (round(m[k], 3) if isinstance(m.get(k), float) else m.get(k)) for k in keep}
    return plan

def show(plan):
    print(plan['name'])
    for n, mt in plan['materials'].items():
        print(f"  ramp {n:18s} {100 * mt.get('share', 0):5.1f}%  " + ' '.join('%d.%d.%d' % tuple(c) for c in mt['ramp']) + (f"   ({mt.get('job')})" if mt.get('job') else ''))
    for n, p in plan['paint'].items():
        print(f"  part {n:14s} {'FOCAL' if p.get('focal') else 'calm '} marks {100 * p.get('marks', 0):3.0f}%  ramps " + ', '.join(f'{k}[{v[0]}..{v[1]}]' for k, v in p['ramps'].items()) + '  motifs: ' + ', '.join(p.get('motifs', [])))
    print('  edges', plan.get('edges'), ' symmetry', plan.get('symmetry'))
    if plan.get('numbers'): print('  numbers', plan['numbers'])

if __name__ == '__main__':
    a = sys.argv[1:]
    if a[0] == 'read':
        plan = read(a[1], a[a.index('--parts') + 1] if '--parts' in a else 'auto')
        pathlib.Path(a[2]).parent.mkdir(parents=True, exist_ok=True); pathlib.Path(a[2]).write_text(json.dumps(pl.jsonable(plan), indent=1)); show(plan); print('->', a[2])
    elif a[0] == 'show': show(json.loads(pathlib.Path(a[1]).read_text()))
