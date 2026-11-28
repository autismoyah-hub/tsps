import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from patchlib import Patch

# ---- scene.py: object_json -------------------------------------------------------------------------------------
P = Patch(_AP.fix(r'<ROOT>\tools\scene.py'))
P.rep('''def object_def(i):
    return config_files(6, 'objects')[i]
''', '''def object_def(i):
    return config_files(6, 'objects')[i]


def object_shape_folders():
    """Shape numbers of the archive's objects/shape-N folders, shape 10 (ordinary objects) first, then ascending."""
    root = ARCH / 'objects'
    return sorted((int(d.name[6:]) for d in root.glob('shape-*') if d.name[6:].isdigit()), key=lambda s: (s != 10, s))


def object_json(oid, shape=None, quiet=False):
    """The archive JSON of a real object, from whichever objects/shape-N folder holds it (walls and wall corners
    are in shape-0 / shape-3 / shape-9, wall decorations in shape-4, ground decorations and floors in shape-22,
    everything else in shape-10). The folder is the shape the archive drew the object as; `model_shapes` in the
    JSON, when present, lists the placement shape of each entry of `model_ids` (a wall often has a model each for
    shapes 0, 1, 2, 3 and 9; an object with no `model_shapes` has untyped models, which the game uses for shape 10).
    shape=None  the folder's own shape (if the object is in several folders the first is taken, shape-10 first,
                and a line is printed saying which);
    shape=N     the folder shape-N if the object is there, otherwise any folder whose JSON lists a model for N.
    Added to the returned dict:
      shape_folder     the folder's shape number
      shape_used       the placement shape chosen
      shapes_listed    sorted shapes the object has models for, or None when its models are untyped
      shape_model_ids  the model ids for shape_used (all of model_ids when the models are untyped)."""
    root = ARCH / 'objects'
    found = [s for s in object_shape_folders() if (root / f'shape-{s}' / f'{oid}.json').exists()]
    if not found:
        raise SystemExit(f'object {oid}: no JSON in any of {[f"shape-{s}" for s in object_shape_folders()]} under {root}')
    folder = shape if shape in found else found[0]
    j = json.loads((root / f'shape-{folder}' / f'{oid}.json').read_text(encoding='utf-8'))
    listed = j.get('model_shapes')
    if shape is not None and shape != folder and listed and shape not in listed:
        for s in found:                      # another folder may hold a JSON that lists this shape
            k = json.loads((root / f'shape-{s}' / f'{oid}.json').read_text(encoding='utf-8'))
            if shape in (k.get('model_shapes') or []):
                folder, j, listed = s, k, k['model_shapes']; break
    use = folder if shape is None else shape
    if listed and use not in listed:
        raise SystemExit(f'object {oid} (folder shape-{folder}) lists no model for shape {use}; it has models for shapes {sorted(set(listed))}')
    if not listed and use not in (10, 11) and not quiet:
        print(f'note: object {oid} has untyped models (the game uses them for shape 10); taking all of them for shape {use}')
    if len(found) > 1 and not quiet:
        print(f'object {oid} is in folders {[f"shape-{s}" for s in found]}: using shape-{folder} (choose with --shape N)')
    j['shape_folder'] = folder; j['shape_used'] = use
    j['shapes_listed'] = sorted(set(listed)) if listed else None
    j['shape_model_ids'] = [m for m, s in zip(j['model_ids'], listed) if s == use] if listed else list(j['model_ids'])
    return j
''')
P.save()

# ---- compare.py: real_model ------------------------------------------------------------------------------------
P = Patch(_AP.fix(r'<ROOT>\tools\compare.py'))
P.rep('''def real_model(oid):
    j = json.loads((scene.ARCH / 'objects' / 'shape-10' / f'{oid}.json').read_text())
    m = o.merge([o.model(i) for i in j['model_ids']])
''', '''def real_model(oid, shape=None):
    """A real object's mesh with its definition's recolours and scale applied. The object is found in whichever
    archive shape folder holds it (scene.object_json); shape=N picks the model the object lists for that placement
    shape (default: the folder's own shape). The result also carries `depth_bias` (per-face depth offsets) and
    `object` (the archive JSON with shape_folder / shape_used / shapes_listed / shape_model_ids)."""
    j = scene.object_json(oid, shape)
    parts = [o.model(i) for i in j['shape_model_ids']]
    m = o.merge(parts)
    m['depth_bias'] = [b for p in parts for b in p['depth_bias']]
    m['object'] = j
''')
P.save()

# ---- real_dump.py ----------------------------------------------------------------------------------------------
P = Patch(_AP.fix(r'<ROOT>\tools\real_dump.py'))
P.rep('''"""Dump a real asset's mesh to JSON for Blender. usage: real_dump.py KIND ID OUT.json
KIND: object (recolours and scale from its definition applied), model (one raw model id),
''', '''"""Dump a real asset's mesh to JSON for Blender. usage: real_dump.py KIND ID OUT.json [--shape N]
KIND: object (recolours and scale from its definition applied; found in whichever archive folder holds it:
        shape-10 ordinary objects, shape-0 / 3 / 9 walls, shape-4 wall decorations, shape-22 floors and ground
        decorations. A wall lists a model per placement shape; the folder's shape is dumped unless --shape N
        chooses another, and the line printed says which. The JSON gains `shape`, and `depth_offsets` (one per
        face) when any face carries one),
      model (one raw model id),
''')
P.rep('''kind, i, out = sys.argv[1], int(sys.argv[2]), sys.argv[3]
if kind == 'object':
    import compare
    m = compare.real_model(i)
''', '''args = sys.argv[1:]
shape = None
if '--shape' in args:
    k = args.index('--shape'); shape = int(args[k + 1]); del args[k:k + 2]
kind, i, out = args[0], int(args[1]), args[2]
more = {}
if kind == 'object':
    import compare
    m = compare.real_model(i, shape)
    j = m['object']
    more['shape'] = j['shape_used']
    if any(m['depth_bias']): more['depth_offsets'] = m['depth_bias']
    print(f'object {i} {j.get("name")!r}: archive folder shape-{j["shape_folder"]}, dumping its shape {j["shape_used"]} model(s) {j["shape_model_ids"]}'
          + (f'; it lists models for shapes {j["shapes_listed"]}' if j['shapes_listed'] else '; untyped models'))
''')
P.rep('''               priorities=m['priorities']), open(out, 'w'))''', '''               priorities=m['priorities'], **more), open(out, 'w'))''')
P.save()
