"""build_inputs.py: gather the project's assets into one MANIFEST.json for pack.py, and the scene files for step 3a.

usage: build_inputs.py OUTDIR
Writes OUTDIR/manifest.json, OUTDIR/ids.json (what each id is), OUTDIR/scene-CUT/ (loose and by-id scene inputs),
OUTDIR/map/ (terrain and placements of the test map square).

ID RANGES (all checked free by pack.py against the source cache before it writes; the source cache's highest ids are
model 61605, object 62200, NPC 16300, item 65534 with 60000..60007 unused, sequence 14428, frame group 14521):
  models     62000..62099  scenery (kit pieces and furniture), in order of first use
             62100..62174  item models: the research-items id 1000100 + k becomes 62100 + k
             62200..62299  NPC-only models
             1000101       one copy of the longsword's worn model under a LARGE id (tests the 10-byte sector header)
  objects    63000..63099
  items      60000..60007  (as research-items chose them)
  NPCs       16350 Meridian warden, 16351 Star hound. NPC ids MUST stay below 16384: the NPC_INFO spawn record carries the
             id in 14 bits (npc_info.rs: bits.bits(14)); the source cache uses up to 16300, so 16301..16383 are the
             83 free ids a server can spawn. (A first pack used 17000 / 17001: they loaded and drew from the cache, but no
             server could have spawned them.)
  sequences  20001..20005, frame groups 20001..20005 (as research-animation chose them)
  map square 4374 = (17, 22): world tiles x 1088..1151, y 1408..1471; squares (16..18, 21..23) are all absent from the
             source cache, so the client's map loader shows only this square.
             4378 = (17, 26): the same room on a floor of the scene tool's colour (new underlay 300), for the pixel proof.
  underlay   300 (source cache's highest: 241)
Model ids stay at or below 65535 on purpose: the client's JS5 request carries a 16-bit group id (js5.rs request), so a
model above 65535 can be read from disk but cannot be fetched from a server.
"""
import sys, json, pathlib, hashlib, zlib, importlib.util
HERE = pathlib.Path(__file__).resolve().parent
B = HERE.parent
sys.path.insert(0, str(HERE)); sys.path.insert(0, str(B.parent / 'tools'))
sys.dont_write_bytecode = True
import pack, scene

OUT = pathlib.Path(sys.argv[1]).resolve(); OUT.mkdir(parents=True, exist_ok=True)
man = {'underlays': {}, 'models': {}, 'objects': {}, 'items': {}, 'npcs': {}, 'sequences': {}, 'maps': {}}
ids = {'models': {}, 'objects': {}}
by_hash = {}


def model(path, base):
    """Register a model file once (by content); returns its id."""
    path = pathlib.Path(path); h = hashlib.sha256(path.read_bytes()).hexdigest()
    if h in by_hash: return by_hash[h]
    mid = base
    while str(mid) in man['models']: mid += 1
    man['models'][str(mid)] = str(path); ids['models'][str(mid)] = str(path.relative_to(B)) if B in path.parents else str(path)
    by_hash[h] = mid
    return mid


# ---------------------------------------------------------------- scenery: the furnished observatory + the scribe's desk
spec = importlib.util.spec_from_file_location('compose', B / 'room-observatory-furnished' / 'compose.py')
sys.path.insert(0, str(B / 'room-observatory-furnished'))
compose = importlib.util.module_from_spec(spec); spec.loader.exec_module(compose)
DESK = B / 'original-scribe-desk' / 'desk.bin'
objects = {}            # definition bytes -> object id
obj_model = {}


def scene_rows(cut):
    """The room as (object id, x, z, shape, rotation) rows, registering models and object definitions."""
    room = compose.chamber(cut)
    room.kit(DESK, 44, 42, 0, 10, sx=2, sz=1, ambient=30, contrast=0)       # the scribe's desk, as roomlib.furn places it
    paths = {mid: path for (path, shape, flags), (mid, data, d) in room._models.items()}
    rows = []
    for mid, data, d, x, z, rot, shape in room.extra:
        new = model(paths[mid], 62000)
        old = mid.to_bytes(4, 'big'); assert d.count(old) == 1
        d2 = d.replace(old, new.to_bytes(4, 'big'))
        if d2 not in objects:
            oid = 63000 + len(objects); objects[d2] = oid; obj_model[oid] = new
            man['objects'][str(oid)] = {'hex': d2.hex()}
            ids['objects'][str(oid)] = f'model {new} shape {shape} ({pathlib.Path(paths[mid]).name})'
        rows.append((objects[d2], x, z, shape, rot))
    return rows


def write_scene(cut, rows):
    """Two scene folders for one cut: loose/ (models and definitions as a mounted package: today's route) and
    byid/ (the same floor, a package holding one unplaced dummy, and placements.txt naming cache object ids)."""
    root = OUT / f'scene-{cut or "full"}'
    sha = lambda b: hashlib.sha256(b).hexdigest()
    vertices = ['BROKEN_WATER_RETURN 1'] + [f'V {x} {z} 0 5803 1' for x in range(128) for z in range(128)]
    bind = b'CINDERWORKS_VISUAL_BINDINGS 1\n'
    def content(folder, pkg_rows, floor_lines):
        d = root / folder; (d / 'package').mkdir(parents=True, exist_ok=True)
        reg = ('TOWER_ASSET_PACKAGE 1\n' + ''.join(f'{k} {i} {n} {zlib.crc32(b):08x}\n' for k, i, n, b in pkg_rows)).encode()
        (d / 'package' / 'registry.txt').write_bytes(reg)
        for k, i, n, b in pkg_rows: (d / 'package' / n).write_bytes(b)
        floor = ('\n'.join(floor_lines) + '\n').encode()
        (d / 'floor.bwr').write_bytes(floor); (d / 'bindings.txt').write_bytes(bind)
        (d / 'content.txt').write_text('GIELINOR_AUTHORED_CONTENT 1\n'
            f'package=package\npackage_sha256={scene.package_identity(reg, [b for *_, b in pkg_rows])}\n'
            f'floor=floor.bwr\nfloor_sha256={sha(floor)}\nbindings=bindings.txt\nbindings_sha256={sha(bind)}\nmotion=off\n', newline='\n')
    used_o = sorted({r[0] for r in rows})
    defs = {oid: d for d, oid in objects.items()}
    used_m = sorted({obj_model[o] for o in used_o})
    pkg = [('model', m, f'm{m}.bin', pathlib.Path(man['models'][str(m)]).read_bytes()) for m in used_m]
    pkg += [('object', o, f'o{o}.bin', defs[o]) for o in used_o]
    content('loose', pkg, vertices + [f'O {o} {x} {z} {s} {r}' for o, x, z, s, r in rows])
    dummy_model = pathlib.Path(man['models'][str(used_m[0])]).read_bytes()
    dummy = [('model', 64990, 'dummy.bin', dummy_model), ('object', 64990, 'dummy-object.bin', bytes([7, 1]) + (64990).to_bytes(4, 'big') + bytes([2]) + b'Unplaced\0' + bytes([0]))]
    content('byid', dummy, vertices)
    (root / 'byid' / 'placements.txt').write_text(''.join(f'{o} {x} {z} {s} {r}\n' for o, x, z, s, r in rows))
    return len(rows), len(used_o), len(used_m)


scenes = {}
for cut in ('sw', 's', ''):
    scenes[cut] = scene_rows(cut)
for cut, rows in scenes.items():
    print('scene', cut or 'full', 'placements, object ids, models:', write_scene(cut, rows))

# ---------------------------------------------------------------- items 60000..60007
ITEMS = B / 'research-items' / 'completed'
remap = lambda v: 62100 + (v - 1000100) if isinstance(v, int) and v >= 1000100 else v
deep = lambda v: [deep(x) for x in v] if isinstance(v, list) else remap(v)
for name in ('longsword', 'glaive', 'staff', 'cuirass', 'gauntlets', 'legs', 'boots', 'barbute'):
    d = ITEMS / name
    iid = json.loads((d / 'decoded.json').read_text())['id']
    for k, p in json.loads((d / 'models.json').read_text()).items():
        mid = remap(int(k)); assert str(mid) not in man['models']
        man['models'][str(mid)] = p; ids['models'][str(mid)] = f'item {iid} {name}: {pathlib.Path(p).name} (research-items id {k})'
        by_hash.setdefault(hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest(), mid)
    spec_ = json.loads((d / 'spec.json').read_text())
    man['items'][str(iid)] = {k: deep(v) if k in ('model', 'wear_models', 'head_models') else v for k, v in spec_.items()}
man['models']['1000101'] = man['models']['62101']; ids['models']['1000101'] = 'large-id copy of 62101 (longsword, male worn)'

# ---------------------------------------------------------------- NPCs
def npc(nid, path, extra):
    d = json.loads(path.read_text())
    models = []
    for m in d['models']:
        if m.startswith('cache:'): models.append(int(m[6:]))
        else:
            p = pathlib.Path(m) if pathlib.Path(m).is_absolute() else path.parent / m
            before = len(man['models']); mid = model(p, 62200); models.append(mid)
            if len(man['models']) > before: ids['models'][str(mid)] = f'npc {nid}: {p.name}'
    man['npcs'][str(nid)] = dict(name=d['name'], models=models, recolor=d.get('recolor', []), scale=d.get('scale', [128, 128]),
                                 ambient=d.get('ambient', 0), contrast=d.get('contrast', 0), size=d.get('size', 1),
                                 zbuffer=d.get('zbuffer', True), stand=d.get('stand'), walk=d.get('walk'), **extra)
    return {k: d[k] for k in ('attack', 'death', 'jab') if k in d}
server = {}
server['16350'] = npc(16350, B / 'research-npc' / 'warden' / 'warden.json', dict(combat=60, ops=[None, 'Attack', None, None, None]))
server['16351'] = npc(16351, B / 'creature-star-hound-2' / 'star-hound.json', dict(combat=40, ops=[None, 'Attack', None, None, None]))
ids['npc_server_sequences'] = server       # attack / death are not in an NPC definition: the server sends them

# ---------------------------------------------------------------- sequences
V = B / 'research-animation' / 'verification'
for sid, name in ((20001, 'meridian_crosscut'), (20002, 'meridian_fall-grip'), (20003, 'warden_salute'), (20004, 'cow_flyshake'), (20005, 'goblin')):
    man['sequences'][str(sid)] = str(V / name)

# ---------------------------------------------------------------- one map square: the full room at local tiles 32..40
REGION = (17 << 8) | 22
(OUT / 'map').mkdir(exist_ok=True)
UNDER = 48                                                 # a real underlay id, chosen in LIVE.md (step 5)
tiles = {}
for x in range(64):
    for y in range(64):
        tiles[(0, x, y)] = {'underlay': UNDER, 'height': 0}
(OUT / 'map' / 'terrain.bin').write_bytes(pack.encode_terrain(tiles))
rows = [(o, 0, x - 8, z - 8, s, r) for o, x, z, s, r in scenes['']]         # room tiles 40..48 -> local 32..40
REAL = [(25702, 0, 27, 34, 10, 0), (2677, 0, 27, 36, 10, 0), (1276, 0, 44, 40, 10, 0), (1276, 0, 30, 44, 10, 1)]   # real Drawers, Chest, two Trees
rows += REAL
(OUT / 'map' / 'locations.bin').write_bytes(pack.encode_locations(rows))
(OUT / 'map' / 'locations.json').write_text(json.dumps(rows))
man['maps'][str(REGION)] = {'terrain': str(OUT / 'map' / 'terrain.bin'), 'locations': str(OUT / 'map' / 'locations.bin')}
# a second square, (17, 26) = 4378, for the pixel proof of the map route: the room alone on a floor of the authored
# floor's own colour (the scene tool's tile value 5803 is underlay file 5802 = authored_floor.rs COLORS[1] = 0x55564e), packed as a new underlay 300
man['underlays'] = {'300': {'rgb': '55564e'}}
REGION2 = (17 << 8) | 26
(OUT / 'map' / 'terrain2.bin').write_bytes(pack.encode_terrain({(0, x, y): {'underlay': 301, 'height': 0} for x in range(64) for y in range(64)}))
rows2 = [(o, 0, x - 8, z - 8, s, r) for o, x, z, s, r in scenes['']]
(OUT / 'map' / 'locations2.bin').write_bytes(pack.encode_locations(rows2))
(OUT / 'map' / 'locations2.json').write_text(json.dumps(rows2))
man['maps'][str(REGION2)] = {'terrain': str(OUT / 'map' / 'terrain2.bin'), 'locations': str(OUT / 'map' / 'locations2.bin')}
ids['map'] = {'region': REGION, 'placements': len(rows), 'real_objects': REAL}

(OUT / 'manifest.json').write_text(json.dumps(man, indent=1))
(OUT / 'ids.json').write_text(json.dumps(ids, indent=1))
print('models', len(man['models']), 'objects', len(man['objects']), 'items', len(man['items']), 'npcs', len(man['npcs']),
      'sequences', len(man['sequences']), 'map placements', len(rows))
