"""Dump a real asset's mesh to JSON for Blender. usage: real_dump.py KIND ID OUT.json [--shape N]
KIND: object (recolours and scale from its definition applied; found in whichever archive folder holds it:
        shape-10 ordinary objects, shape-0 / 3 / 9 walls, shape-4 wall decorations, shape-22 floors and ground
        decorations. A wall lists a model per placement shape; the folder's shape is dumped unless --shape N
        chooses another, and the line printed says which. The JSON gains `shape`, and `depth_offsets` (one per
        face) when any face carries one),
      model (one raw model id),
      npc or item (all model ids from the archive entry merged; recolours not applied yet);
      worn (an item's male WORN model, in body space: what is seen in the hand).
Coordinates are written y-up in game units. Private: cache-derived."""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o, scene

args = sys.argv[1:]
shape = None
if '--shape' in args:
    k = args.index('--shape'); shape = int(args[k + 1]); del args[k:k + 2]
kind, i, out = args[0], int(args[1]), args[2]
more = {}
if kind == 'object':
    import compare
    m = compare.real_model(i, shape, '--faithful' in args)
    j = m['object']
    more['shape'] = j['shape_used']
    if any(m['depth_bias']): more['depth_offsets'] = m['depth_bias']
    print(f'object {i} {j.get("name")!r}: archive folder shape-{j["shape_folder"]}, dumping its shape {j["shape_used"]} model(s) {j["shape_model_ids"]}'
          + (f'; it lists models for shapes {j["shapes_listed"]}' if j['shapes_listed'] else '; untyped models'))
elif kind == 'model':
    m = o.model(i)
else:
    sub = {'npc': 'npcs', 'item': 'items', 'worn': 'items'}[kind]
    j = next(json.loads(p.read_text()) for p in (scene.ARCH / sub).glob(f'*/{i}.json'))
    if kind == 'worn':
        ids = [k for k in j['wear_models'][0] if k not in (-1, None)]
    else:
        ids = [j['inventory_model']] if kind == 'item' and j.get('inventory_model') else j['model_ids']
    import compare
    m = compare.assembled([o.model(k) for k in ids], '--faithful' in args)
if '--faithful' in args or any(m['alpha']) or any(t>=0 for t in m['face_textures']):
    if kind == 'worn':
        for a,b in j.get('recolors') or []:
            m['colors'] = [b if c == a else c for c in m['colors']]
        dy = j['wear_offsets'][0]
        m['vertices'] = [(x,y+dy,z) for x,y,z in m['vertices']]
        compare.apply_textures(m,j)
    more.update({k:m[k] for k in ('texture_coords','texture_types','texture_triangles','complex_sections',
                                 'bone_weights','face_textures','alpha','render_types','vertex_skins',
                                 'uniform_priority','has_alpha')})
    more.update(depth_offsets=m['depth_bias'],face_skins=m['face_skins'] if m['has_face_skin'] else None)
json.dump(dict(kind=kind, id=i,
               vertices=[[p[0], -p[1], p[2]] for p in m['vertices']], faces=m['faces'],
               colours=[list(o.hsl_split(c)) for c in m['colors']], flat=[1 if r == 1 else 0 for r in m['render_types']],
               priorities=m['priorities'], **more), open(out, 'w'))
print(out, len(m['vertices']), 'vertices', len(m['faces']), 'faces')
