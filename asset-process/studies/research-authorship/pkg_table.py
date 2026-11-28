"""pkg_table.py: tables and bands from data/packages.json.
usage: pkg_table.py [KEY]   -> prints; pkg_table.py > data/packages.txt
"""
import sys, json, pathlib, statistics as st, collections
HERE = pathlib.Path(__file__).resolve().parent
D = json.loads((HERE / 'data' / 'packages.json').read_text())
only = sys.argv[1] if len(sys.argv) > 1 else None

def med(x): x = [v for v in x if v is not None]; return st.median(x) if x else None
def band(x):
    x = sorted(v for v in x if v is not None)
    return (x[0], st.median(x), x[-1]) if x else (None, None, None)

def summary(p):
    bosses = [n for n in p['npcs'] if n['role'] == 'boss']
    roles = collections.Counter(n['role'] for n in p['npcs'])
    seq_b = [s for s in p['seqs'] if s['on_base']]
    seq_roles = collections.Counter(s['role'] for s in seq_b)
    named_spots = [s for s in p['spots'] if s['named_for_boss']]
    reused = [s for s in p['spots'] if s['server_uses'] and not s['named_for_boss']]
    sroles = collections.Counter(s['role'] for s in named_spots)
    rel = collections.Counter()
    for s in named_spots:
        for k, v in (s['relation'] or {}).items(): rel[k] += v
    nrel = sum(rel.values()) or 1
    lroles = collections.Counter(l['role'] for l in p['locs'])
    fams = p['colours'][str(p['mains'][0])] or []
    bm = p['boss_measure'].get(str(p['mains'][0]), {})
    sk = p['skeleton'][0] if p['skeleton'] else {}
    return dict(
        boss=p['label'], release=p['release'], forms=len(bosses), npcs=len(p['npcs']), roles=dict(roles), model_lists=len(p['model_lists']),
        models_main=len(next(n for n in bosses if n['id'] == p['mains'][0])['models']), faces=bm.get('faces'), verts=bm.get('verts_files'),
        all_models_faces=sum(m['faces'] for m in p['models']), textured_faces=sum(m['textured'] for m in p['models']),
        alpha_area=p['alpha_area'].get(str(p['mains'][0])), colours=bm.get('colours'), colours90=bm.get('colours_for_90'),
        families=len([f for f in fams if f['share'] >= .03]), main_share=round(fams[0]['share'], 3) if fams else None,
        second_share=round(fams[1]['share'], 3) if len(fams) > 1 else None, accent=bm.get('accent_share'), islands=bm.get('detail_islands'),
        shells=bm.get('shells'), largest_shell=bm.get('largest_shell_face_share'), height=bm.get('size_y'),
        base=sk.get('base'), groups=sk.get('groups'), bones=sk.get('bones'), bases=len(p['bases']), on_base=sk.get('npcs_on'),
        outside_name=sk.get('n_outside'), older=[o['name'] for o in sk.get('older_creatures', [])][:4],
        seqs_on_base=len(seq_b), seq_skeletal=sum(1 for s in seq_b if s['kind'] == 'cached'), seq_roles=dict(seq_roles),
        seq_cycles=band([s['cycles'] for s in seq_b]), seq_total_cycles=sum(s['cycles'] or 0 for s in seq_b),
        spots_named=len(named_spots), spots_server_reused=len(reused), spot_roles=dict(sroles),
        spot_faces=band([s['faces'] for s in named_spots]), spot_size=band([s['size'] for s in named_spots]),
        spot_alpha=band([s['alpha_share'] for s in named_spots]), spot_colours90=band([s['colours90'] for s in named_spots]),
        spot_textured=sum(1 for s in named_spots if s['textured']), spot_skeletal=sum(1 for s in named_spots if s['kind'] == 'cached'),
        spot_weights=sum(1 for s in named_spots if s['weights']),
        spot_colour_relation={k: round(v / nrel, 2) for k, v in rel.most_common()},
        projectiles=sum(1 for s in named_spots if s['role'] == 'projectile'),
        locs=len(p['locs']), loc_roles=dict(lroles), locs_animated=sum(1 for l in p['locs'] if l.get('animated')),
        loc_faces=band([l.get('faces') for l in p['locs'] if l['role'] == 'mechanic']),
        reused_names=[s['gv'] for s in reused][:8])

if __name__ == '__main__':
    S = {k: summary(p) for k, p in D.items() if not only or k == only}
    for k, s in S.items():
        print('==', k)
        for a, b in s.items(): print(f'  {a}: {b}')
    if not only:
        (HERE / 'data' / 'summary.json').write_text(json.dumps(S, indent=1), encoding='utf-8')
