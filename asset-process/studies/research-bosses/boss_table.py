"""Tables for BOSSES.md from data/bosses.json and the era study's recent mid-sized creatures. usage: boss_table.py > data/bosses-table.txt"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import json, pathlib, statistics as st
HERE = pathlib.Path(__file__).parent
B = json.loads((HERE / 'data' / 'bosses.json').read_text())
E = json.loads(pathlib.Path(_AP.fix(r'<ROOT>\studies\research-creatures\data\era.json')).read_text())
mid = [r for r in E.values() if r['group'] == 'R' and r['plan'] in 'qsh']
eb = [r for r in E.values() if r['group'] == 'R' and r['plan'] == 'b']
rows = list(B.values())
owner = [r for r in rows if r['set'] != 'other']
def q(v, p):
    v = sorted(v); return v[min(len(v) - 1, int(p * (len(v) - 1) + 0.5))]
def band(v): return f'{st.median(v):.3g} ({q(v, .1):.3g}-{q(v, .9):.3g})'
print('== per boss')
print('id | name | set | tiles | models | faces | verts | rig | base | groups/bones | NPC defs on base (model lists) | seqs (skeletal) | forms ids/model lists | colours (90%) | tones/hue | islands | alpha share | undrawn | textured | flat | amb/con | scale | style out G/R | stand walk attack death')
for r in rows:
    m = r.get('measure', {})
    print(' | '.join(str(x) for x in (r['id'], r['label'], r['set'], r['size'], len(r['models']), r['faces_files'], r['verts_files'],
        'WEIGHTS' if r['weighted'] else f"labels {r['labels_files']}", r['skeleton'], f"{r.get('skel_groups')}/{r.get('skel_bones')}", f"{r.get('skel_npcs')} ({r.get('skel_model_lists')})",
        f"{r.get('seqs')} ({r.get('seqs_skeletal')})", f"{r['forms_ids']}/{r['forms_model_lists']}", f"{m.get('colours')} ({m.get('colours_for_90')})", round(m.get('tones_per_hue', 0), 1),
        m.get('detail_inlaid'), round(r.get('alpha_face_share_any', 0), 3), r.get('undrawn_faces'), r.get('textured_faces'), round(m.get('flat_face_share', 0), 3),
        f"{r['ambient']}/{r['contrast']}", r['scale'][0], f"{r.get('style_game_out')}/{r.get('style_recent_out')}", r['stand'], r['walk'], r.get('attack'), r.get('death'))))
print('\n== medians (10th-90th): recent mid-sized (era study R: quadrupeds, spiders, humanoids, n=%d) | era-study recent bosses (n=%d) | all bosses here (n=%d) | owner-named (raids, Nex, Nightmare, Doom, Maggot King; n=%d)' % (len(mid), len(eb), len(rows), len(owner)))
KEYS = ['faces', 'vertices', 'face_med_share_e4', 'colours', 'colours_for_90', 'tones_per_hue', 'main_hue_light_span', 'tone_step_med', 'largest_region_share', 'insurface_colour_edge_share',
        'detail_inlaid', 'detail_area_share', 'thin_area_share', 'fold_med_deg', 'tone_follows_up', 'tone_by_part', 'valence_6_share', 'valence_sd', 'shells', 'largest_shell_face_share',
        'flat_face_share', 'alpha_face_share', 'split_vertex_share', 'size_x', 'size_y', 'size_z', 'ragged', 'solidity']
for k in KEYS:
    f = lambda L: band([(r.get('measure', r))[k] for r in L if (r.get('measure', r)).get(k) is not None])
    print(f'{k} | {f(mid)} | {f(eb)} | {f(rows)} | {f(owner)}')
def cnt(L, fn): return f'{sum(1 for r in L if fn(r))} of {len(L)}'
print('\n== counts: all bosses here | owner-named')
for name, fn in (('bone weights', lambda r: r['weighted']), ('stand is skeletal (keyframe)', lambda r: r.get('stand_kind') == 'cached'), ('ambient or contrast set', lambda r: r['ambient'] or r['contrast']),
                 ('some translucent faces', lambda r: r.get('alpha_face_share_any', 0) > 0), ('undrawn faces', lambda r: r.get('undrawn_faces', 0) > 0), ('textured faces', lambda r: (r.get('textured_faces') or 0) > 0),
                 ('flat faces > 10%', lambda r: r.get('measure', {}).get('flat_face_share', 0) > .1), ('several models', lambda r: len(r['models']) > 1), ('definition scale not 128', lambda r: r['scale'][0] != 128),
                 ('more than one NPC id of the name', lambda r: r['forms_ids'] > 1), ('more than one model list of the name', lambda r: r['forms_model_lists'] > 1),
                 ('base serves one model list only', lambda r: r.get('skel_model_lists') == 1), ('base serves no other NAME', lambda r: len(r.get('skel_names', [])) <= 1),
                 ('spot animations on its base', lambda r: r.get('skel_spots')), ('objects on its base', lambda r: r.get('skel_objects')),
                 ('size 5 tiles or more', lambda r: r['size'] >= 5), ('breaks no GAME style limit', lambda r: r.get('style_game_out') == 0), ('breaks at most one RECENT band', lambda r: (r.get('style_recent_out') or 0) <= 1)):
    print(f'{name} | {cnt(rows, fn)} | {cnt(owner, fn)}')
print('mid-sized era-study: weights', sum(1 for r in mid if r.get('weights')), 'of', len(mid), '; ambient/contrast set', sum(1 for r in mid if r.get('ambient') or r.get('contrast')), '; size tiles', band([r['size_tiles'] for r in mid]))
print('size tiles: bosses', band([r['size'] for r in rows]), 'owner', band([r['size'] for r in owner]))
print('seqs on base: bosses', band([r.get('seqs', 0) for r in rows]), '; NPC defs on base', band([r.get('skel_npcs', 0) for r in rows]), '; groups', band([r.get('skel_groups') or 0 for r in rows]), '; bones (weighted only)', band([r['skel_bones'] for r in rows if r['weighted']]))
print('weighted vs labelled faces:', band([r['faces_files'] for r in rows if r['weighted']]), '|', band([r['faces_files'] for r in rows if not r['weighted']]))
print('weighted vs labelled colours:', band([r['measure']['colours'] for r in rows if r['weighted']]), '|', band([r['measure']['colours'] for r in rows if not r['weighted']]))
print('\n== style lines broken')
for r in rows:
    if r.get('style_lines'): print(r['label'], '|', '; '.join(r['style_lines']))
print('\n== skeletal sequence ids of the weighted bosses')
for r in rows:
    if r['weighted']: print(r['id'], r['label'], 'base', r['skeleton'], 'bones', r.get('skel_bones'), 'models', r['models'], 'stand', r['stand'], 'walk', r['walk'], 'attack', r.get('attack'), 'death', r.get('death'), 'all', r.get('seq_ids_skeletal'))
print('\n== relatives on the base')
for r in rows:
    print(r['label'], '| names:', ', '.join(r.get('skel_names', [])[:12]), '| spots:', len(r.get('skel_spots') or []), '| objects:', r.get('skel_objects'))
