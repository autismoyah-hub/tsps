"""Style gate for an ORIGINAL creature: does the mesh sit inside what real creatures measure (CREATURES.md)?

usage: style_npc.py ENTRY [ENTRY...]         ENTRY = def:PATH.json | npc:ID
       style_npc.py --calibrate              how many of the 111 measured real creatures each rule passes, per era
Two sets of checks, all PROVISIONAL (set 2026-11-27 from 28 + 30 + 53 real creatures; none yet confirmed by the owner's eye):
  GAME   limits that every era keeps (10th / 90th centile of all 111): outside them a mesh is unlike ANY real creature.
  RECENT the 10th-90th centile band of the 53 creatures from 2019 on, for the measures that separate 2004-2005
         creatures from recent ones (separation 0.72 or more): outside them, on the early side, a mesh reads as dated.
A verdict line is: rule | value | limit | OK / OUT.  The last line counts the OUTs of each kind.
"""
import sys, pathlib, json
sys.path.insert(0, str(pathlib.Path(__file__).parent))
HERE = pathlib.Path(__file__).parent
# key, description, low, high (None = no bound), kind
RULES = [
    ('split_vertex_share', 'welded: vertices on a shared position', None, 0.0, 'GAME'),
    ('valence_6_share', 'hand-split, not a grid: vertices with exactly six faces', None, 0.45, 'GAME'),
    ('valence_sd', 'hand-split, not a grid: spread of faces per vertex', 1.30, None, 'GAME'),
    ('tone_follows_up', 'tone NOT painted by facing: correlation lightness~up', None, 0.45, 'GAME'),
    ('tone_by_part', 'tone painted by part: lightness variance explained by label', 0.15, None, 'GAME'),
    ('tone_step_med', 'soft tone steps: median lightness step across a tone edge', None, 16, 'GAME'),
    ('flat_face_share', 'flat-shaded faces (more only for ice, crystal, bone shards)', None, 0.10, 'GAME'),
    ('shells', 'one skin: separate shells (a kit-built humanoid or armoured boss may exceed)', None, 40, 'GAME'),
    ('faces', 'faces', 888, None, 'RECENT'),
    ('face_med_share_e4', 'median face, ten-thousandths of the surface', None, 8.3, 'RECENT'),
    ('colours', 'colours', 11, None, 'RECENT'),
    ('colours_for_90', 'colours covering 90% of the surface', 4, None, 'RECENT'),
    ('tones_per_hue', 'tones per hue family', 2.9, None, 'RECENT'),
    ('insurface_colour_edge_share', 'paint boundaries inside a surface (share of in-surface edge length)', 0.107, None, 'RECENT'),
    ('largest_region_share', 'largest one-colour patch, share of surface', None, 0.45, 'RECENT'),
    ('detail_inlaid', 'small contrasting islands inlaid in the skin (eyes, teeth, claws, marks)', 16, None, 'RECENT'),
    ('fold_med_deg', 'median fold between neighbouring faces, degrees', None, 28.4, 'RECENT'),
    ('thin_area_share', 'surface on thin parts (limbs, horns, tail)', 0.31, None, 'RECENT'),
]
def check(r, quiet=False):
    outs = dict(GAME=0, RECENT=0)
    for key, desc, lo, hi, kind in RULES:
        x = r.get(key)
        if x is None: continue
        ok = (lo is None or x >= lo) and (hi is None or x <= hi)
        lim = (f'>= {lo}' if lo is not None else '') + (' and ' if lo is not None and hi is not None else '') + (f'<= {hi}' if hi is not None else '')
        if not ok: outs[kind] += 1
        if not quiet: print(f"{kind:6s} {desc} [{key}] | {x:.3f} | {lim} | {'OK' if ok else 'OUT'}")
    return outs
if __name__ == '__main__':
    a = sys.argv[1:]
    if '--calibrate' in a:
        rows = json.loads((HERE / 'data' / 'era.json').read_text())
        for g in ('E1', 'E2', 'R', 'OURS'):
            rs = [(k, r) for k, r in rows.items() if r['group'] == g]; res = [(k, check(r, True)) for k, r in rs]
            print(f"{g}: {len(rs)} creatures; no GAME rule broken: {sum(o['GAME'] == 0 for _, o in res)}; at most one RECENT rule broken: {sum(o['RECENT'] <= 1 for _, o in res)}; "
                  f"median RECENT rules broken {sorted(o['RECENT'] for _, o in res)[len(res) // 2]}; median GAME rules broken {sorted(o['GAME'] for _, o in res)[len(res) // 2]}")
            if g == 'OURS':
                for k, o in res: print('   ', k, o)
        for key, desc, lo, hi, kind in RULES:
            def passed(g):
                v = [r[key] for r in rows.values() if r['group'] == g and r.get(key) is not None]
                return f"{sum((lo is None or x >= lo) and (hi is None or x <= hi) for x in v)}/{len(v)}"
            print(f'{kind:6s} {key:30s} pass E1 {passed("E1")}  E2 {passed("E2")}  R {passed("R")}')
    else:
        import cmesh
        for e in a:
            sc = tuple(json.loads(pathlib.Path(e[4:]).read_text()).get('scale', [128, 128])) if e.startswith('def:') else (128, 128)
            print('==', e); o = check(cmesh.measure(cmesh.load(e), sc)); print(f"OUT of GAME limits: {o['GAME']}   OUT of the RECENT band: {o['RECENT']}")
