"""beyond.py: where the newest bosses go past the older end-game style, measured.
usage: beyond.py   -> prints; data/beyond.json
Eras (release years are my knowledge): A = 2017-2019 (Chambers of Xeric, Theatre of Blood, Vorkath, Hydra, Nightmare 2020),
B = 2022-2023 (Nex 2022 classic, Tombs of Amascut, Desert Treasure II, Phantom Muspah), C = the eight new packages (2024-2026).
Boss meshes from research-bosses/data/bosses.json (BOSSES.md's measures); effects from research-effects/data/spots.jsonl by
gameval name prefix (each era's boss-named effects only; generic or reused names excluded).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import json, re, pathlib, statistics as st, collections
HERE = pathlib.Path(__file__).resolve().parent
B = pathlib.Path(_AP.fix(r'<ROOT>\builds'))
BJ = json.loads((B / 'research-bosses/data/bosses.json').read_text())
SP = [json.loads(l) for l in (B / 'research-effects/data/spots.jsonl').read_text(encoding='utf-8').splitlines()]
PK = json.loads((HERE / 'data' / 'packages.json').read_text())

ERA_A = [7541, 7566, 7530, 7527, 7563, 8360, 8359, 8355, 8388, 8340, 8370, 8372, 8374, 8061, 8615, 9425]
ERA_B = [11278, 11789, 11778, 11719, 11730, 11753, 11756, 11761, 11762, 12077, 12223, 12191, 12214, 12204]
ERA_C = [12821, 13668, 13685, 14009, 12596, 14147, 14176, 14707, 15742]
FX_A = r'^(olm_|raids_|tekton|vasa|vespula|muttadile|vanguard|tob_|verzik|vorkath|hydraboss|nightmare_)'
FX_B = r'^(nex_|toa_|wardens|kephri|akkha|zebak|baba_|tumekens|elidinis|spotanim_wardens|muspah|vfx_duke|spotanim_duke|vfx_leviathan|leviathan_|vfx_whisperer|whisperer_|spotanim_whisperer|vardorvis|vfx_vardorvis)'

def q(x):
    x = sorted(v for v in x if v is not None)
    if not x: return None
    k = lambda p: x[min(len(x) - 1, int(p * (len(x) - 1) + .5))]
    return [round(k(.1), 3), round(st.median(x), 3), round(k(.9), 3), len(x)]

def mesh(ids):
    rows = [BJ[str(i)] for i in ids if str(i) in BJ]
    g = lambda f: q([f(r) for r in rows])
    return dict(n=len(rows), faces=g(lambda r: r['measure']['faces']), verts=g(lambda r: r['verts_files']),
                colours=g(lambda r: r['measure']['colours']), colours90=g(lambda r: r['measure']['colours_for_90']),
                alpha_face_share=g(lambda r: r['measure']['alpha_face_share']), islands=g(lambda r: r['measure']['detail_islands']),
                shells=g(lambda r: r['measure']['shells']), main_hue_share=g(lambda r: r['measure']['main_hue_share']),
                tones_per_hue=g(lambda r: r['measure']['tones_per_hue']), seqs=g(lambda r: r['seqs']),
                weighted=sum(1 for r in rows if r['weighted']), textured=sum(1 for r in rows if r['textured_faces']),
                models_in_def=g(lambda r: len(r['models'])), scaled=sum(1 for r in rows if r['scale'] != [128, 128]))

def fx(rows):
    m = lambda f: q([f(s) for s in rows if s.get('m') and s['m'].get('faces')])
    return dict(n=len(rows), faces=m(lambda s: s['m']['faces']), colours90=m(lambda s: s['m']['colours90']), colours=m(lambda s: s['m']['colours']),
                alpha_share=m(lambda s: s['m']['alpha_share']), any_alpha=round(sum(1 for s in rows if s.get('m', {}).get('alpha_faces')) / max(1, len(rows)), 3),
                size=m(lambda s: max(s['m']['box'])), cycles=q([s['s'].get('cycles') for s in rows if s.get('s')]),
                skeletal=sum(1 for s in rows if s.get('s', {}).get('kind') == 'cached'), weights=sum(1 for s in rows if s.get('m', {}).get('weights')),
                textured=sum(1 for s in rows if s.get('m', {}).get('textured')), flat_any=round(sum(1 for s in rows if s.get('m', {}).get('flat')) / max(1, len(rows)), 3),
                distinct_models=len({s['model'] for s in rows}))

own_c = [next(s for s in SP if s['id'] == x['id']) for p in PK.values() for x in p['spots'] if x['named_for_boss']]
res = dict(mesh=dict(A=mesh(ERA_A), B=mesh(ERA_B), C=mesh(ERA_C)),
           fx=dict(A=fx([s for s in SP if re.search(FX_A, s['name'] or '')]), B=fx([s for s in SP if re.search(FX_B, s['name'] or '')]), C=fx(own_c)))
for k in ('mesh', 'fx'):
    print('==', k)
    keys = list(res[k]['A'])
    for f in keys: print(f'  {f:18s} A {res[k]["A"][f]}   B {res[k]["B"][f]}   C {res[k]["C"][f]}')
(HERE / 'data' / 'beyond.json').write_text(json.dumps(res, indent=1))
