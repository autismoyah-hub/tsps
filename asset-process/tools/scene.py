"""Build a renderer scene: a base authored package plus real game objects, then capture it.

usage: scene.py OUT_DIR BASE_ADMITTED_DIR|- "id:x:z:rot,..." [cam: X Z YAW DIST ELEV UP]...
Real objects are mounted with their own definition bytes, so their own models,
recolours, scale and lighting values are used. Private output only.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, hashlib, zlib, shutil, subprocess, pathlib, os
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o

BIN = _AP.fix(r'<RENDERER_BIN>')
# The same example built from the client copy whose source we read (examples/authored_scene_snapshot.rs).
# Set the environment variable SCENE_SNAPSHOT_BIN to that path (or any other build) to capture with it instead
# of the older prebuilt binary above; the default is unchanged.
BIN_CLIENT_COPY = _AP.fix(r'<CLIENT_BIN>\authored_scene_snapshot.exe')
BIN = os.environ.get('SCENE_SNAPSHOT_BIN', BIN)
ARCH = pathlib.Path(_AP.fix(r'<REFERENCE_ARCHIVE>'))
_defs = {}
TAIL = bytes([2]) + b'Yardstick' + bytes([0, 14, 1, 15, 1, 17, 64, 19, 0, 0])


def config_files(group_id, kind):
    """All files of a config group (archive 2), keyed by definition id."""
    if kind in _defs:
        return _defs[kind]
    ids = json.loads((ARCH / 'catalog-inventory.json').read_text())['types'][kind]['ids']
    data = o.unpack(o.group(2, group_id))
    n = data[-1]
    start = len(data) - 1 - n * len(ids) * 4
    p = start; pos = 0
    out = [bytearray() for _ in ids]
    for _ in range(n):
        size = 0
        for t in out:
            size += int.from_bytes(data[p:p + 4], 'big', signed=True); p += 4
            t += data[pos:pos + size]; pos += size
    assert pos == start
    _defs[kind] = dict(zip(ids, map(bytes, out)))
    return _defs[kind]


def object_def(i):
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


def npc_as_object(npc_id):
    """A real NPC's models as a static object: a human yardstick. Negative ids on the command line."""
    for sub in (ARCH / 'npcs').iterdir():
        f = sub / f'{npc_id}.json'
        if f.exists():
            ids = json.loads(f.read_text())['model_ids']
            return (bytes([7, len(ids)]) + b''.join(i.to_bytes(4, 'big') for i in ids)
                    + TAIL)
    raise KeyError(npc_id)


def package_identity(registry_bytes, payloads):
    h = hashlib.sha256()
    h.update(b'GIELINOR_ASSET_PACKAGE_CONTENT_V1\0')
    h.update(len(registry_bytes).to_bytes(8, 'big')); h.update(registry_bytes)
    for b in payloads:
        h.update(len(b).to_bytes(8, 'big')); h.update(b)
    return h.hexdigest()


def build(out, base, real, extra_models=(), underlay=5803):
    out = pathlib.Path(out)
    pkg = out / 'package'
    pkg.mkdir(parents=True)
    rows = []          # (kind, id, filename, bytes)
    floor_lines = []
    if base:
        base = pathlib.Path(base)
        for line in (base / 'package' / 'registry.txt').read_text().splitlines()[1:]:
            kind, i, name, _ = line.split()
            rows.append((kind, int(i), name, (base / 'package' / name).read_bytes()))
        floor_lines = (base / 'floor.bwr').read_text().splitlines()
    else:
        floor_lines = ['BROKEN_WATER_RETURN 1'] + [f'V {x} {z} 0 {underlay} {1 if underlay else 0}' for x in range(128) for z in range(128)]
    used = {i for k, i, _, _ in rows if k == 'object'}
    nid = 65000
    for (oid, x, z, rot) in real:
        while nid in used:
            nid += 1
        used.add(nid)
        rows.append(('object', nid, f'r{nid}.bin', npc_as_object(-oid) if oid < 0 else object_def(oid)))
        floor_lines.append(f'O {nid} {x} {z} 10 {rot}')
    for entry in extra_models:
        (mid, mbytes, obytes, x, z, rot), shape = entry[:6], (entry[6] if len(entry) > 6 else 10)
        while nid in used:
            nid += 1
        used.add(nid)
        if not any(k == 'model' and i == mid for k, i, _, _ in rows):
            rows.append(('model', mid, f'x{mid}.bin', mbytes))
        rows.append(('object', nid, f'x{nid}.bin', obytes))
        floor_lines.append(f'O {nid} {x} {z} {shape} {rot}')     # shape 22 is a ground decoration: drawn under what stands on the tile
    reg = 'TOWER_ASSET_PACKAGE 1\n' + ''.join(f'{k} {i} {n} {zlib.crc32(b):08x}\n' for k, i, n, b in rows)
    regb = reg.encode()
    (pkg / 'registry.txt').write_bytes(regb)
    for k, i, n, b in rows:
        (pkg / n).write_bytes(b)
    floor = ('\n'.join(floor_lines) + '\n').encode()
    (out / 'floor.bwr').write_bytes(floor)
    bind = b'CINDERWORKS_VISUAL_BINDINGS 1\n'
    (out / 'bindings.txt').write_bytes(bind)
    sha = lambda b: hashlib.sha256(b).hexdigest()
    (out / 'content.txt').write_text(
        'GIELINOR_AUTHORED_CONTENT 1\n'
        f'package=package\npackage_sha256={package_identity(regb, [b for *_, b in rows])}\n'
        f'floor=floor.bwr\nfloor_sha256={sha(floor)}\n'
        f'bindings=bindings.txt\nbindings_sha256={sha(bind)}\nmotion=off\n', newline='\n')
    return out / 'content.txt'


def capture(content, name, pose):
    from PIL import Image
    target = pathlib.Path(content).parent / name
    if target.exists():
        shutil.rmtree(target)
    r = subprocess.run([BIN, str(content), str(o.CACHE), *map(str, pose), 'classic', 'auto', str(target)],
                       capture_output=True, text=True, env={**os.environ, 'RAYON_NUM_THREADS': '4'})
    if r.returncode:
        raise SystemExit(f'renderer failed for {name}: {r.stdout}{r.stderr}')
    Image.open(target / 'view.bmp').save(target.with_suffix('.png'))
    return target.with_suffix('.png')


if __name__ == '__main__':
    out, base, real = sys.argv[1], sys.argv[2], sys.argv[3]
    real = [tuple(map(int, r.split(':'))) for r in real.split(',') if r]
    content = build(out, None if base == '-' else base, real)
    for k, cam in enumerate(sys.argv[4:]):
        print(capture(content, f'cam{k}', cam.split()))
