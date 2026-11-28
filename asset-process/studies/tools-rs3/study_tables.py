"""Print markdown tables from study-mesh.json for STUDY.md."""
import json, pathlib
ROOT = pathlib.Path(__file__).resolve().parent
s = json.loads((ROOT/'study-mesh.json').read_text())
for cat in ('Creatures', 'Weapons', 'Armour', 'Effects'):
    print(f'\n### {cat}\n')
    print('| 951 definition | RS3 faces | RS3 vertices (stored / welded) | Materials | Textures | Alpha faces | Max bones (multi-bone share) | 239 control | OSRS faces | OSRS vertices | OSRS alpha faces | OSRS max bones |')
    print('|---|---:|---:|---:|---:|---:|---:|---|---:|---:|---:|---:|')
    for r in s['rows']:
        if r['category'] != cat: continue
        n, o = r['rs3'], r['osrs']
        print(f"| {r['name']} ({r['id']}) | {n['faces']:,} | {n['vertices']:,} / {n['welded_positions']:,} | {n['materials']} | {n['textures']} | {n['alpha_faces']:,} | {n['max_bones']} ({n['multi_bone_vertex_share']:.0%}) | {r['control_name']} ({r['control']}) | {o['faces']:,} | {o['vertices']:,} | {o['alpha_faces']} | {o['max_bones']} |")
print('\n### Armour, male worn forms\n')
print('| 951 definition | Faces | Vertices (stored / welded) | Materials | Textures | Alpha faces | Max bones (multi-bone share) |')
print('|---|---:|---:|---:|---:|---:|---:|')
names = {r['id']: r['name'] for r in s['rows']}
for k, n in s['armour_worn_male'].items():
    if 'error' in n: print(f'| {names[int(k)]} ({k}) | {n["error"]} |'); continue
    print(f"| {names[int(k)]} ({k}) | {n['faces']:,} | {n['vertices']:,} / {n['welded_positions']:,} | {n['materials']} | {n['textures']} | {n['alpha_faces']} | {n['max_bones']} ({n['multi_bone_vertex_share']:.0%}) |")
