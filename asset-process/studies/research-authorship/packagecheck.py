"""packagecheck.py: a boss PACKAGE (boss model, skeleton, NPC family, sequences, effects, arena objects, fight) against the
eight newest real end-game bosses measured in AUTHORSHIP.md (data/packages.json, made by pkg_measure.py).

usage:
  packagecheck.py PACKAGE.json [--quiet]      each rule: holds | OUTSIDE, our value, the real band (min-max of the 8), the median
  packagecheck.py --real KEY OUT.json         write a real package in this format (sol araxxor amox huey titans yama doom maggot)
  packagecheck.py --calibrate                 each real package against bands made from the OTHER seven (leave one out)

It is a GUARD, not a judge: it says whether a package has the parts and budgets the real ones have. It cannot see whether a
part is good. Bands come from eight bosses only. No owner grade has been tied to it.

THE FORMAT (JSON; every list may be empty; missing numbers are measured where a file is given, else the rule says n/a)
{
 "name": str,
 "boss": {                          the main fought form
    "def": PATH,                    optional: an NPC definition JSON as cmesh/npc.py read it ({"models": [...], "scale", "size", ...});
                                    when given, every number below is MEASURED from it (numbers written here are then ignored)
    "tiles": N, "models": N, "faces": N, "colours": N, "colours90": N, "hue_families": N, "main_family_share": 0..1,
    "translucent_area": 0..1, "textured_faces": N, "islands": N, "largest_shell": 0..1 },
 "skeleton": {"kind": "weighted" | "classic", "own": true|false, "built_for": str, "groups": N, "bones": N},
              own = made for this boss (its minions and pet may share it); false = borrowed from an older creature
 "npcs": [ {"name": str, "role": "boss" | "add" | "pet" | "state" | "helper", "scale": N, "on_boss_skeleton": bool} ],
 "sequences": [ {"name": str, "role": "idle" | "walk" | "attack" | "death" | "spawn" | "phase" | "hit/block" | "other",
                 "cycles": N, "own": bool} ]        the boss's OWN-skeleton sequences (what it plays)
 "effects": [ {"name": str, "role": "projectile" | "impact" | "telegraph" | "ground" | "cast/aura" | "other", "own": bool,
               "model": PATH (optional .bin: faces, size, textured and colour relation are measured), "faces": N, "size": N,
               "textured": N, "in_boss_colours": 0..1} ]
 "objects": [ {"name": str, "role": "mechanic" | "dressing" | "entry/barrier" | "utility/memorial" | "other", "animated": bool} ],
 "fight": PATH (optional: a fight file for research-fights/fightcheck.py; its hold count is reported)
}
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, re, pathlib, statistics as st, collections, subprocess
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
B = pathlib.Path(_AP.fix(r'<ROOT>\builds'))
PY = _AP.fix(r'<PYTHON>')
D = json.loads((HERE / 'data' / 'packages.json').read_text())

def hsl(c): return (c >> 10) & 63, (c >> 7) & 7, c & 127
def sector(h, s, l): return 'n-dark' if (s <= 1 and l < 40) else 'n-light' if (s <= 1 and l >= 80) else 'n-mid' if s <= 1 else f'h{h // 8}'

# ---------------------------------------------------------------- real packages in this format
def real(key):
    p = D[key]; main = str(p['mains'][0]); bm = p['boss_measure'][main]
    fams = p['colours'][main] or []
    sk = p['skeleton'][0]
    boss_npc = next(n for n in p['npcs'] if n['id'] == p['mains'][0])
    own_skel = not [o for o in sk['older_creatures']] and key != 'amox'      # Amoxliatl stands on the 2023 Nagua / Moons skeleton
    secs = {x['sector']: x['share'] for x in p['sectors'][main]}
    return dict(
        name=p['label'],
        boss=dict(tiles=boss_npc['size'], models=len(boss_npc['models']), faces=bm['faces'], colours=bm['colours'], colours90=bm['colours_for_90'],
                  hue_families=len([f for f in fams if f['share'] >= .03]), main_family_share=fams[0]['share'] if fams else None,
                  translucent_area=p['alpha_area'][main], textured_faces=sum(m['textured'] for m in p['models']), islands=bm['detail_islands'],
                  largest_shell=bm['largest_shell_face_share']),
        skeleton=dict(kind='weighted' if sk['bones'] else 'classic', own=own_skel, groups=sk['groups'], bones=sk['bones'], base=sk['base']),
        npcs=[dict(name=n['gv'], role=n['role'], scale=n['scale'][0], on_boss_skeleton=n['skeleton'] in p['bases']) for n in p['npcs']],
        sequences=[dict(name=s['gv'], role=s['role'], cycles=s['cycles'], own=True) for s in p['seqs'] if s['on_base'] and (own_skel or s['named'])],
        effects=[dict(name=s['gv'], role=s['role'], own=True, faces=s['faces'], size=s['size'], textured=s['textured'],
                      in_boss_colours=round(sum(v for k, v in (s['sector_relation'] or {}).items() if k in ('main', 'body', 'accent')), 3))
                 for s in p['spots'] if s['named_for_boss']],
        objects=[dict(name=l['gv'], role=l['role'], animated=bool(l.get('animated'))) for l in p['locs']])

# ---------------------------------------------------------------- measuring our files
def measure_boss(defpath):
    for q in (B / 'research-creatures', B / 'research-npc', B / 'research-animation', B.parent / 'tools'): sys.path.insert(0, str(q))
    import cmesh
    cmesh.DATA = HERE / 'data'
    m = cmesh.load('def:' + str(defpath)); r = cmesh.measure(m)
    area, unit, cen = cmesh.face_geometry(m); keep = m['A'] < 254
    by = collections.Counter()
    for c, a in zip(m['C'][keep], area[keep]): by[int(c)] += float(a)
    tot = sum(by.values()) or 1
    cmesh.HMAP.clear(); cmesh.HMAP.update(cmesh.hue_clusters([c for c, a in by.items() if a >= .004 * tot]))
    fam = collections.Counter()
    for c, a in by.items(): fam[cmesh.family(c)] += a
    fams = sorted(fam.values(), reverse=True)
    sec = collections.Counter()
    for c, a in by.items(): sec[sector(*hsl(c))] += a / tot
    d = json.loads(pathlib.Path(defpath).read_text())
    tex = 0
    import osrscache as oc
    for mp in d['models']:
        mm = oc.decode_model((pathlib.Path(defpath).parent / mp).read_bytes())
        tex += sum(1 for t in mm['face_textures'] if t != -1)
    out = dict(tiles=d.get('size'), models=len(d['models']), faces=r['faces'], colours=r['colours'], colours90=r['colours_for_90'],
               hue_families=len([v for v in fams if v >= .03 * tot]), main_family_share=round(fams[0] / tot, 4) if fams else None,
               translucent_area=round(float(area[(m['A'] > 0) & (m['A'] < 254)].sum() / max(area.sum(), 1e-9)), 4), textured_faces=tex,
               islands=r['detail_islands'], largest_shell=round(r['largest_shell_face_share'], 4))
    return out, dict(sec)

def measure_effect(path, boss_sectors):
    sys.path.insert(0, str(B.parent / 'tools'))
    import osrscache as oc
    m = oc.decode_model(pathlib.Path(path).read_bytes())
    v = m['vertices']; f = m['faces']
    lo = [min(p[i] for p in v) for i in range(3)]; hi = [max(p[i] for p in v) for i in range(3)]
    col = collections.Counter()
    for i, (a, b, c) in enumerate(f):
        ax, ay, az = v[a]; bx, by_, bz = v[b]; cx, cy, cz = v[c]
        ux, uy, uz = bx - ax, by_ - ay, bz - az; wx, wy, wz = cx - ax, cy - ay, cz - az
        ar = 0.5 * ((uy * wz - uz * wy) ** 2 + (uz * wx - ux * wz) ** 2 + (ux * wy - uy * wx) ** 2) ** .5
        if m['render_types'][i] & 2 == 0: col[m['colors'][i]] += ar
    tot = sum(col.values()) or 1
    rel = None
    if boss_sectors:
        cand = [k for k, x in boss_sectors.items() if .01 <= x <= .15 and not k.startswith('n')]
        big = {k for k, x in boss_sectors.items() if x >= .03} | ({max(cand, key=lambda k: boss_sectors[k])} if cand else set())
        rel = round(sum(a for c, a in col.items() if sector(*hsl(c)) in big) / tot, 3)
    return dict(faces=len(f), size=max(hi[i] - lo[i] for i in range(3)), textured=sum(1 for t in m['face_textures'] if t != -1), in_boss_colours=rel)

def complete(pk):
    pk = json.loads(json.dumps(pk)); secs = None
    b = pk.get('boss', {})
    if b.get('def'):
        num, secs = measure_boss(b['def']); b.update(num)
    for e in pk.get('effects', []):
        if e.get('model'): e.update({k: v for k, v in measure_effect(e['model'], secs).items() if v is not None})
    return pk

# ---------------------------------------------------------------- the rules
def med_of(x): x = [v for v in x if v is not None]; return st.median(x) if x else None
def nb(pk): return pk.get('boss', {})
def seqs(pk): return pk.get('sequences', [])
def own_fx(pk): return [e for e in pk.get('effects', []) if e.get('own', True)]
def roles(xs): return collections.Counter(x.get('role') for x in xs)

RULES = [  # id, words, value(pk), kind ('band' | 'atleast' | 'bool'), group
 ('M1', 'tiles a side', lambda p: nb(p).get('tiles'), 'band', 'model'),
 ('M2', 'faces of the main form', lambda p: nb(p).get('faces'), 'band', 'model'),
 ('M3', 'models in its definition', lambda p: nb(p).get('models'), 'band', 'model'),
 ('M4', 'colours', lambda p: nb(p).get('colours'), 'band', 'model'),
 ('M5', 'colours covering 90% of the surface', lambda p: nb(p).get('colours90'), 'band', 'model'),
 ('M6', 'hue families holding 3% or more', lambda p: nb(p).get('hue_families'), 'band', 'model'),
 ('M7', 'share of the main hue family', lambda p: nb(p).get('main_family_share'), 'band', 'model'),
 ('M8', 'translucent share of the surface', lambda p: nb(p).get('translucent_area'), 'band', 'model'),
 ('M9', 'textured faces', lambda p: nb(p).get('textured_faces'), 'band', 'model'),
 ('M10', 'small contrasting islands inlaid in the skin', lambda p: nb(p).get('islands'), 'band', 'model'),
 ('M11', 'largest welded shell, share of faces', lambda p: nb(p).get('largest_shell'), 'band', 'model'),
 ('S1', 'a skeleton of its own (not borrowed from an older creature)', lambda p: p.get('skeleton', {}).get('own'), 'bool', 'skeleton'),
 ('S2', 'bones (weighted) or groups (classic) in the skeleton', lambda p: (p.get('skeleton', {}).get('bones') or p.get('skeleton', {}).get('groups')), 'band', 'skeleton'),
 ('S3', 'a scaled pet on the boss skeleton', lambda p: any(n.get('role') == 'pet' for n in p.get('npcs', [])), 'bool', 'family'),
 ('S4', 'NPC ids in the package (forms, states, adds, pets, helpers)', lambda p: len(p.get('npcs', [])), 'band', 'family'),
 ('A1', 'sequences on its own skeleton', lambda p: len(seqs(p)), 'band', 'animation'),
 ('A2', 'attack sequences', lambda p: roles(seqs(p)).get('attack', 0), 'band', 'animation'),
 ('A3', 'idle, attack and death all present', lambda p: all(roles(seqs(p)).get(r) for r in ('idle', 'attack', 'death')), 'bool', 'animation'),
 ('A4', 'median sequence length, cycles', lambda p: med_of([s.get('cycles') for s in seqs(p)]), 'band', 'animation'),
 ('A5', 'all sequences together, cycles', lambda p: sum(s.get('cycles') or 0 for s in seqs(p)) if seqs(p) else None, 'band', 'animation'),
 ('A6', 'share of its sequences made for it (not borrowed)', lambda p: (sum(1 for s in seqs(p) if s.get('own')) / len(seqs(p))) if seqs(p) else None, 'band', 'animation'),
 ('E1', 'effects of its own', lambda p: len(own_fx(p)), 'band', 'effects'),
 ('E2', 'projectiles of its own', lambda p: roles(own_fx(p)).get('projectile', 0), 'band', 'effects'),
 ('E3', 'distinct effect jobs (projectile, impact, telegraph, ground, cast/aura)', lambda p: len({e.get('role') for e in own_fx(p)} - {'other'}), 'band', 'effects'),
 ('E4', 'median faces of its effects', lambda p: med_of([e.get('faces') for e in own_fx(p)]), 'band', 'effects'),
 ('E5', 'median largest dimension of its effects, units', lambda p: med_of([e.get('size') for e in own_fx(p)]), 'band', 'effects'),
 ('E6', "median share of an effect's colour in the boss's own colour sectors", lambda p: med_of([e.get('in_boss_colours') for e in own_fx(p)]), 'band', 'effects'),
 ('E7', 'textured effects', lambda p: sum(1 for e in own_fx(p) if e.get('textured')), 'band', 'effects'),
 ('O1', 'objects of its own (hazards, dressing, entries)', lambda p: len(p.get('objects', [])), 'band', 'arena'),
 ('O2', 'animated objects of its own', lambda p: sum(1 for o in p.get('objects', []) if o.get('animated')), 'band', 'arena'),
 ('O3', 'at least one mechanic object (pool, tile, hazard)', lambda p: any(o.get('role') == 'mechanic' for o in p.get('objects', [])), 'bool', 'arena'),
]
EXCLUDE = {'O1': {'sol'}, 'O2': {'sol'}, 'O3': {'sol'}}     # Sol Heredit's arena is the Colosseum's, named for it, not for him

def bands(keys):
    out = {}
    pks = {k: real(k) for k in keys}
    for rid, words, f, kind, grp in RULES:
        vals = {k: f(p) for k, p in pks.items() if k not in EXCLUDE.get(rid, ())}
        if kind == 'band':
            xs = {k: v for k, v in vals.items() if v is not None}
            if xs:
                lo_k = min(xs, key=xs.get); hi_k = max(xs, key=xs.get)
                out[rid] = dict(lo=xs[lo_k], hi=xs[hi_k], med=st.median(xs.values()), n=len(xs), lo_by=lo_k, hi_by=hi_k)
        else:
            out[rid] = dict(n_true=sum(1 for v in vals.values() if v), n=len(vals), false_by=[k for k, v in vals.items() if not v])
    return out

def fmt(v):
    if isinstance(v, float): return f'{v:.3g}' if abs(v) < 10 else f'{v:,.0f}'
    if isinstance(v, int) and not isinstance(v, bool): return f'{v:,}'
    return str(v)

def check(pk, bd, quiet=False):
    held = n = 0; lines = []
    for rid, words, f, kind, grp in RULES:
        v = f(pk); b = bd.get(rid)
        if v is None or b is None: lines.append(f'{rid:4s} n/a     {words}'); continue
        n += 1
        if kind == 'band':
            ok = b['lo'] <= v <= b['hi']
            lines.append(f"{rid:4s} {'holds  ' if ok else 'OUTSIDE'} {words}: {fmt(v)}  (real {fmt(b['lo'])} to {fmt(b['hi'])}, median {fmt(b['med'])}; "
                         f"lowest {b['lo_by']}, highest {b['hi_by']}; n={b['n']})")
        else:
            ok = bool(v)
            lines.append(f"{rid:4s} {'holds  ' if ok else 'OUTSIDE'} {words}: {v}  (real: {b['n_true']} of {b['n']}" + (f"; not: {', '.join(b['false_by'])})" if b['false_by'] else ')'))
        held += ok
    fight = pk.get('fight')
    if fight:
        r = subprocess.run([PY, '-I', str(B / 'research-fights' / 'fightcheck.py'), fight, '--quiet'], capture_output=True, text=True)
        last = [l for l in r.stdout.splitlines() if 'rules hold' in l]
        lines.append(f"F    fight (fightcheck.py): {last[-1] if last else 'not run: ' + (r.stderr.strip()[-200:] or 'no output')}")
    if not quiet: print('\n'.join(lines))
    print(f'{held} of {n} package rules hold' + (f'  [{pk.get("name")}]' if quiet else ''))
    return held, n

if __name__ == '__main__':
    a = sys.argv[1:]
    if not a: print(__doc__); sys.exit(0)
    if a[0] == '--real':
        pathlib.Path(a[2]).write_text(json.dumps(real(a[1]), indent=1), encoding='utf-8'); print('wrote', a[2])
    elif a[0] == '--calibrate':
        keys = list(D)
        for k in keys:
            check(real(k), bands([x for x in keys if x != k]), quiet=True)
    else:
        pk = complete(json.loads(pathlib.Path(a[0]).read_text(encoding='utf-8')))
        check(pk, bands(list(D)), quiet='--quiet' in a)
