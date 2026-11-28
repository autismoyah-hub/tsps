"""Export a real item's worn model through OUR encoder, labels kept, to test the armour harness.

usage: reencode_item.py ITEM_ID OUT.bin [--female] [--raw] [--no-depth]

Reads the item's worn model ids from the private archive JSON (items/slot-*/ID.json, field `wear_models`,
first row = male, second = female; -1 = absent), decodes each from the cache with osrscache.model, and builds
the worn mesh the way the client does (rust-client src/item_model.rs `wearable`):
  1. the third model is used only when a second exists;
  2. several models are combined as Model::combine does (src/model_merge.rs): faces in order, vertices
     de-duplicated by coordinate, the FIRST vertex at a coordinate keeping its label;
  3. the mesh is moved down by the definition's wear offset (`wear_offsets`, y only);
  4. the definition's recolours (`recolors`, pairs from -> to) are applied in order.
Then re-encodes with native_codec.model keeping vertex_skins, face_skins, per-face priorities, render types,
alpha, and the per-face depth offsets (decode_model's depth_bias, the encoder's face_flags = depth_bias / 2;
the Rune full helm's eight visor faces carry one, and without it they sort differently against the helm:
--no-depth drops them to show the difference). --raw skips steps 3 and 4 (the cache model as stored).
Prints counts and label histograms.
The result is in worn space: draw it with  pose.py OUT.png "item:ID,model:OUT.bin@SLOT+HIDE"  and the two
columns should be the same picture.
Retextures are not applied (the encoder keeps face textures only when the model has no complex texture
mapping; the script stops if the item needs one).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, glob, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
sys.path.insert(1, _AP.fix(r'<NATIVE_ASSETS>'))
import osrscache, native_codec

ARCHIVE = _AP.fix(r'<REFERENCE_ARCHIVE>/items')


def definition(item_id):
    found = glob.glob(f'{ARCHIVE}/slot-*/{item_id}.json')
    if not found:
        raise SystemExit(f'item {item_id}: no JSON under {ARCHIVE}/slot-*/')
    return json.loads(pathlib.Path(found[0]).read_text(encoding='utf-8'))


def combine(parts):
    """Model::combine for the channels our encoder writes: coordinate de-duplication, first label wins."""
    out = dict(vertices=[], faces=[], colors=[], render_types=[], priorities=[], alpha=[], face_skins=[], vertex_skins=[], depth_bias=[])
    index = {}
    for m in parts:
        vs = m['vertex_skins']

        def vert(i, m=m, vs=vs):
            key = tuple(m['vertices'][i])
            if key not in index:
                index[key] = len(out['vertices'])
                out['vertices'].append(key)
                out['vertex_skins'].append(vs[i] if vs is not None else 0)
            return index[key]
        for i, face in enumerate(m['faces']):
            out['faces'].append(tuple(vert(v) for v in face))
            for k in ('colors', 'render_types', 'priorities', 'alpha', 'face_skins', 'depth_bias'):
                out[k].append(m[k][i])
    return out


def worn(item_id, female=False, raw=False):
    d = definition(item_id)
    ids = d['wear_models'][1 if female else 0]
    if ids[0] == -1:
        raise SystemExit(f'item {item_id}: no worn model for this body type')
    ids = ids[:1] if ids[1] == -1 else [i for i in ids if i != -1]
    parts = [osrscache.model(i) for i in ids]
    for i, m in zip(ids, parts):
        if any(t != -1 for t in m['face_textures']):
            raise SystemExit(f'item {item_id}: worn model {i} is textured; this script does not carry textures')
    g = combine(parts)
    if not raw:
        dy = d.get('wear_offsets', [0, 0])[1 if female else 0]
        g['vertices'] = [(x, y + dy, z) for x, y, z in g['vertices']]
        for a, b in d.get('recolors', []):
            g['colors'] = [b if c == a else c for c in g['colors']]
    return d, ids, parts, g


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    if len(args) != 2:
        raise SystemExit(__doc__)
    item_id, out = int(args[0]), pathlib.Path(args[1])
    d, ids, parts, g = worn(item_id, '--female' in sys.argv, '--raw' in sys.argv)
    enc = dict(vertices=[list(v) for v in g['vertices']], faces=g['faces'], colors=g['colors'],
               vertex_skins=g['vertex_skins'], face_skins=g['face_skins'], priorities=g['priorities'])
    if any(g['render_types']):
        enc['render_types'] = g['render_types']
    if any(g['alpha']):
        enc['alpha'] = g['alpha']
    if '--no-depth' in sys.argv:
        g['depth_bias'] = [0] * len(g['faces'])
    if any(g['depth_bias']):
        enc['face_flags'] = [(b // 2) & 255 for b in g['depth_bias']]
    data = native_codec.model(enc)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(data)
    back = osrscache.decode_model(data)
    for k in ('vertices', 'faces', 'colors', 'vertex_skins', 'face_skins', 'priorities', 'render_types', 'alpha', 'depth_bias'):
        got = [tuple(v) for v in back[k]] if k in ('vertices', 'faces') else back[k]
        want = [tuple(v) for v in g[k]] if k in ('vertices', 'faces') else g[k]
        assert got == want, f'round trip changed {k}'
    hist = lambda v: dict(sorted(collections.Counter(v).items()))
    print(f'item {item_id} {d["name"]!r}: wear_positions {d.get("wear_positions")} worn models {ids} '
          f'(source formats {[m["format"] for m in parts]}; source counts {[(len(m["vertices"]), len(m["faces"])) for m in parts]})')
    print(f'  wrote {out} ({len(data)} bytes): {len(g["vertices"])} vertices, {len(g["faces"])} faces; round trip through decode_model identical')
    print(f'  vertex labels {hist(g["vertex_skins"])}')
    print(f'  face labels   {hist(g["face_skins"])}')
    print(f'  priorities    {hist(g["priorities"])}  render types {hist(g["render_types"])}  depth offsets {hist(g["depth_bias"])}')
    print(f'  recolours applied {d.get("recolors", []) if "--raw" not in sys.argv else "none (--raw)"}; colours now {hist(g["colors"])}')


if __name__ == '__main__':
    main()
