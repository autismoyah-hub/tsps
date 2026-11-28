"""fxpack.py: research-skeleton/skelpack.py (itself research-live/pack.py + frame bases) copied and extended with SPOT ANIMATION
definitions (archive 2, group 13, file ID): "spots": {"4100": {"model": 62510, "sequence": 20110, "scale": [128, 128],
"rotation": 0, "ambient": 60, "contrast": 60, "recolor": [[from, to]..]}} or {"def": "path/effect.def", "model": ID, "sequence": ID}
(the harness's loose definition: its scale, rotation, ambient, contrast and recolours are taken; model and sequence are the
packed ids). A spot id must fit 16 bits (every packet names it in two bytes); the model and the sequence it names must be in
the result. Encoded by fxlib.encode_spot and read back through fxlib.decode_spot (a port of spot_animation.rs).

skelpack.py: research-live/pack.py copied and extended with FRAME BASES (archive 1), for a creature with its own skeleton.

pack.py: copy a revision-239 cache and add our assets to the copy under new ids.

usage: fxpack.py SOURCE_CACHE OUT_CACHE MANIFEST.json

SOURCE_CACHE is only read. OUT_CACHE must not exist; it becomes a full copy (dat2 and every idx) and then receives:
  archive 7, group ID, file 0        one model per id                              "models"
  archive 2, group 1,  file ID       floor underlays (a colour)                    "underlays"  {"300": {"rgb": "3e4444"}}
  archive 2, group 6,  file ID       object (scenery) definitions                  "objects"
  archive 2, group 9,  file ID       NPC definitions                               "npcs"
  archive 2, group 10, file ID       item definitions                              "items"
  archive 2, group 12, file ID       sequence definitions                          "sequences"
  archive 0, group G,  file F        the classic frames a new sequence names       (from the sequence's loose directory)
  archive 1, group B,  file 0        a NEW frame base (skeleton)                   "bases": {"2700": "path/base_2700.dat"}
                                     every loose frame's first two bytes must name a base that is in the cache or in
                                     "bases"; a base_B.dat lying in a sequence directory is NOT packed by itself
  archive 5, group REGION, files 0,1 one map square: terrain and object placements "maps"
  archive 255, group A               the reference table of every archive touched (group list, file lists, CRC-32,
                                     version, sizes)
Every id in the manifest must be FREE in the source cache (model group, definition file, frame group, map group);
an id that exists stops the run before anything is written. Nothing real is ever replaced.

MANIFEST.json (all ids are the final ids; every section optional):
{ "models":    {"62000": "path/model.bin", ...},
  "objects":   {"63000": {"hex": "definition bytes as hex"}                      as the tools already write them, or
                "63001": {"name": "Desk", "models": [62000], "shapes": [10] or null, "size": [2, 1], "ambient": 30,
                          "contrast": 0, "solid": true, "merge_normals": false, "interactive": 0}},
  "items":     {"60000": {a SPEC as research-items/itemcodec.py takes it: model, wear_models, zoom, ...}},
  "npcs":      {"16350": {"name": "..", "models": [ids], "recolor": [[from, to]..], "scale": [xz, y], "ambient": 0,
                          "contrast": 0 (as stored), "size": 1, "zbuffer": true, "stand": SEQ, "walk": SEQ,
                          "combat": 0, "ops": [null, "Attack", null, null, null]}},
  "sequences": {"20001": "loose directory with seq.dat and frame_G_F.dat (research-animation layout)"},
  "maps":      {"REGION": {"terrain": "file", "locations": "file"}} }

What the client checks on a group (disk_cache.rs validate_container): the CRC-32 of the stored container and the
version, against the reference table; a four-byte version trailer follows each container on disk (as in the source
cache). Sector chains: 520-byte sectors; the header is 8 bytes for group ids up to 65535 and 10 bytes above
(cache.rs read_group_from), both written here.
After writing, everything is read back with this file's own reader and compared with the inputs byte for byte;
the client-side read-back is examples/live_readback.rs (see LIVE.md).
"""
import sys, shutil, zlib, json, copy, pathlib

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'research-animation')); sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent / 'research-items'))
import animlib as A
import fxlib

CONFIG = {'underlays': 1, 'objects': 6, 'npcs': 9, 'items': 10, 'sequences': 12, 'spots': 13}


def encode_spot(d):
    d = dict(d)
    if 'def' in d:
        kv = dict(l.split('=', 1) for l in pathlib.Path(d['def']).read_text(encoding='utf-8').splitlines() if '=' in l and not l.strip().startswith('#'))
        d.setdefault('scale', [int(x) for x in kv.get('scale', '128,128').split(',')]); d.setdefault('rotation', int(kv.get('rotation', 0)))
        d.setdefault('ambient', int(kv.get('ambient', 0))); d.setdefault('contrast', int(kv.get('contrast', 0)))
        if kv.get('recolor'): d.setdefault('recolor', [[int(x) for x in p.split(':')] for p in kv['recolor'].split(',')])
        if kv.get('noflag') == '1': d['flag'] = False
    spec = dict(model=d['model'], sequence=d.get('sequence', -1), scale=d.get('scale', [128, 128]), rotation=d.get('rotation', 0), ambient=d.get('ambient', 0),
                contrast=d.get('contrast', 0), flag=d.get('flag', True), recolor=[tuple(p) for p in d.get('recolor', [])], retexture=[])
    data = fxlib.encode_spot(spec)
    back = fxlib.decode_spot(data)
    assert all(back[k] == spec[k] for k in ('model', 'sequence', 'rotation', 'ambient', 'contrast', 'flag')) and back['scale'] == list(spec['scale']) and back['recolor'] == spec['recolor'], 'spot definition does not read back'
    return data


# ------------------------------------------------------------------------------------------------- encoders
def encode_underlay(d):
    """Floor underlay: opcode 1 + rgb, 0 end (the client's floor decoder; the same bytes authored floors use)."""
    return bytes([1]) + bytes.fromhex(d['rgb']) + bytes(1)


def encode_object(d):
    if 'hex' in d:
        return bytes.fromhex(d['hex'])
    b = bytearray()
    if d.get('contrast', 0): b += bytes([39, d['contrast'] & 255])
    if d.get('ambient', 0): b += bytes([29, d['ambient'] & 255])
    if d.get('merge_normals'): b += bytes([22])
    shapes = d.get('shapes')
    b += bytes([6 if shapes else 7, len(d['models'])])
    for i, m in enumerate(d['models']):
        b += m.to_bytes(4, 'big')
        if shapes: b += bytes([shapes[i]])
    b += bytes([2]) + d.get('name', 'null').encode('cp1252') + b'\0'
    sx, sz = d.get('size', [1, 1])
    b += bytes([14, sx, 15, sz])
    if not d.get('solid', True): b += bytes([17])
    if 'interactive' in d: b += bytes([19, d['interactive']])
    for i, t in enumerate(d.get('ops', [])):
        if t: b += bytes([30 + i]) + t.encode('cp1252') + b'\0'
    return bytes(b + b'\0')


def encode_npc(d):
    """npc.rs decode_mode: 61 models (u8 n, u32 each), 2 name, 12 size, 13 stand, 14 walk, 30-34 options, 40 recolours,
    95 combat level, 97 / 98 scale, 100 ambient, 101 contrast (stored; the client multiplies by 5), 147 = painted by
    priority like a player, 0 end."""
    b = bytearray()
    b += bytes([61, len(d['models'])]) + b''.join(int(m).to_bytes(4, 'big') for m in d['models'])
    b += bytes([2]) + d['name'].encode('cp1252') + b'\0'
    if d.get('size', 1) != 1: b += bytes([12, d['size']])
    if d.get('stand') is not None: b += bytes([13]) + int(d['stand']).to_bytes(2, 'big')
    if d.get('walk') is not None: b += bytes([14]) + int(d['walk']).to_bytes(2, 'big')
    for i, t in enumerate(d.get('ops', [])):
        if t: b += bytes([30 + i]) + t.encode('cp1252') + b'\0'
    if d.get('recolor'):
        b += bytes([40, len(d['recolor'])])
        for a, c in d['recolor']: b += int(a).to_bytes(2, 'big') + int(c).to_bytes(2, 'big')
    if d.get('combat') is not None: b += bytes([95]) + int(d['combat']).to_bytes(2, 'big')
    sxz, sy = d.get('scale', [128, 128])
    if sxz != 128: b += bytes([97]) + sxz.to_bytes(2, 'big')
    if sy != 128: b += bytes([98]) + sy.to_bytes(2, 'big')
    if d.get('ambient', 0): b += bytes([100, d['ambient'] & 255])
    if d.get('contrast', 0): b += bytes([101, d['contrast'] & 255])
    if not d.get('zbuffer', True): b += bytes([147])
    return bytes(b + b'\0')


def encode_item(spec):
    import itemcodec
    return bytes(itemcodec.encode(spec))


# ------------------------------------------------------------------------------------------------- map square
def encode_terrain(tiles=None, underlay=0, height=None):
    """One 64 x 64 square, four planes, as terrain.rs Region::decode reads it: per tile a run of u16 opcodes,
    0 = end with the procedural height, 1 + u8 = end with height 8 * value below the plane beneath, 2..49 + u16 =
    overlay (shape and rotation in the opcode), 50..81 = flags + 49, above 81 = underlay id + 81.
    tiles: {(plane, x, y): {"underlay": id, "height": units of 8 or None, "overlay": id, "flags": n}}; every other
    tile of plane 0 takes `underlay` and `height`; planes 1 to 3 are empty."""
    tiles = tiles or {}
    b = bytearray()
    for plane in range(4):
        for x in range(64):
            for y in range(64):
                t = tiles.get((plane, x, y), {'underlay': underlay, 'height': height} if plane == 0 else {})
                if t.get('overlay'): b += (2).to_bytes(2, 'big') + int(t['overlay']).to_bytes(2, 'big')
                if t.get('flags'): b += (49 + t['flags']).to_bytes(2, 'big')
                if t.get('underlay'): b += (81 + t['underlay']).to_bytes(2, 'big')
                h = t.get('height')
                if h is None: b += (0).to_bytes(2, 'big')
                else: b += (1).to_bytes(2, 'big') + bytes([h])
    return bytes(b)


def _smart(v):
    if v < 128: return bytes([v])
    assert v < 32768
    return (v + 32768).to_bytes(2, 'big')


def encode_locations(rows):
    """rows: (object id, plane, x, y, shape, rotation). terrain.rs locations: object ids ascending as smart deltas
    (first delta is id + 1; 32767 chains), then per placement a smart delta of the packed position plane<<12 | x<<6 | y
    plus 1 and one byte shape<<2 | rotation; 0 ends each list."""
    by = {}
    for oid, plane, x, y, shape, rot in rows:
        by.setdefault(oid, []).append(((plane << 12) | (x << 6) | y, (shape << 2) | rot))
    b = bytearray(); last = -1
    for oid in sorted(by):
        d = oid - last; last = oid
        while d >= 32767:
            b += _smart(32767); d -= 32767
        b += _smart(d)
        prev = 0
        for pos, attr in sorted(by[oid]):       # equal positions (two shapes on one tile) give a delta of 1 + 0
            b += _smart(pos - prev + 1) + bytes([attr]); prev = pos
        b += _smart(0)
    b += _smart(0)
    return bytes(b)


# ------------------------------------------------------------------------------------------------- cache writing
def write_group(root, archive, gid, stored):
    """Append `stored` (container + trailer) to dat2 as a new sector chain and point the index entry at it."""
    dat = root / 'main_file_cache.dat2'
    idx = root / f'main_file_cache.idx{archive}'
    big = gid > 65535
    room = 510 if big else 512
    with dat.open('r+b') as d:
        d.seek(0, 2)
        sector = max(1, (d.tell() + 519) // 520)
        d.seek(sector * 520)
        first = sector
        pos = 0; chunk = 0
        while pos < len(stored):
            part = stored[pos:pos + room]; pos += len(part)
            nxt = sector + 1 if pos < len(stored) else 0
            head = (gid.to_bytes(4, 'big') if big else gid.to_bytes(2, 'big')) + chunk.to_bytes(2, 'big') + nxt.to_bytes(3, 'big') + bytes([archive])
            d.write(head + part.ljust(room, b'\0'))
            sector += 1; chunk += 1
        assert sector < 1 << 24
    with idx.open('r+b') as f:
        f.seek(0, 2)
        if f.tell() < gid * 6:
            f.write(bytes(gid * 6 - f.tell()))
        f.seek(gid * 6)
        f.write(len(stored).to_bytes(3, 'big') + first.to_bytes(3, 'big'))


def store(root, archive, gid, files, ref, version=None):
    """files: {file id: bytes}. Writes the group and updates the in-memory reference table `ref`."""
    ids = sorted(files)
    data = A.join_files([files[i] for i in ids])
    container = A.pack_container(data)
    old = ref['groups'].get(gid, {})
    version = version if version is not None else old.get('version', 0) + 1
    entry = dict(old, crc=zlib.crc32(container) & 0xffffffff, version=version, files=ids)
    if ref['flags'] & 4: entry['sizes'] = (len(container), len(data))
    assert not ref['flags'] & 0b1011, 'named or digested reference tables are not written by this packer'
    ref['groups'][gid] = entry
    write_group(root, archive, gid, container + (version & 0xffffffff).to_bytes(4, 'big'))


def main():
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    src_dir, out, man_path = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3])
    man = json.loads(man_path.read_text(encoding='utf-8'))
    if out.exists():
        raise SystemExit(f'{out} exists; give a new directory (this packer never overwrites)')
    if out.resolve() == src_dir.resolve():
        raise SystemExit('OUT_CACHE is the source cache')
    src = A.Cache(src_dir)
    rel = lambda p: (man_path.parent / p) if not pathlib.Path(p).is_absolute() else pathlib.Path(p)

    # ---- gather everything and check every id free, before a byte is written
    refs, raws = {}, {}
    def ref(a):
        if a not in refs:
            refs[a] = copy.deepcopy(src.reference(a)); raws[a] = src.raw(255, a)
            assert A.build_reference(refs[a]) == A.unpack(raws[a]), f'reference table {a} does not re-encode exactly'
        return refs[a]
    clashes = []
    models = {int(k): rel(v).read_bytes() for k, v in man.get('models', {}).items()}
    for mid in models:
        if mid in ref(7)['groups']: clashes.append(f'model {mid}')
    configs = {}
    for kind, enc in (('underlays', encode_underlay), ('objects', encode_object), ('npcs', encode_npc), ('items', encode_item), ('spots', encode_spot)):
        add = {int(k): enc(v) for k, v in man.get(kind, {}).items()}
        if add:
            have = set(ref(2)['groups'][CONFIG[kind]]['files'])
            clashes += [f'{kind[:-1]} {i}' for i in add if i in have]
            configs[kind] = add
    bases = {int(k): rel(v).read_bytes() for k, v in man.get('bases', {}).items()}
    for b in bases:
        if b in ref(1)['groups']: clashes.append(f'frame base {b}')
        assert b <= 65535, 'a frame names its base in two bytes'
        A.decode_base(bases[b])
    frames = {}
    seqs = {}
    for k, d in man.get('sequences', {}).items():
        d = rel(d); sid = int(k)
        seq = (d / 'seq.dat').read_bytes()
        seqs[sid] = seq
        loose = {}
        for p in d.glob('frame_*.dat'):
            _, g, f = p.stem.split('_')
            loose.setdefault(int(g), {})[int(f)] = p.read_bytes()
        for fid in A.decode_sequence(seq)['frames']:
            g, f = fid >> 16, fid & 65535
            if g in loose: assert f in loose[g], f'{d}: seq.dat names frame {g}:{f} but there is no file for it'
            else: assert g in ref(0)['groups'] and f in ref(0)['groups'][g]['files'], f'{d}: frame {g}:{f} is neither loose nor in the cache'
        for g, files in loose.items():
            for f, data in files.items():
                b = int.from_bytes(data[:2], 'big')
                if b not in bases and b not in ref(1)['groups']: raise SystemExit(f'{d}: frame {g}:{f} names frame base {b}, which is neither in the cache nor in the manifest')
                A.decode_frame(data, A.decode_base(bases[b]) if b in bases else A.decode_base(src.file(1, b, 0)))
            if g in ref(0)['groups'] or g in frames: clashes.append(f'frame group {g}')
            frames[g] = files
    if seqs:
        have = set(ref(2)['groups'][12]['files'])
        clashes += [f'sequence {i}' for i in seqs if i in have]
        configs['sequences'] = seqs
    maps = {}
    for k, m in man.get('maps', {}).items():
        rid = int(k)
        if rid in ref(5)['groups']: clashes.append(f'map square {rid}')
        maps[rid] = {0: rel(m['terrain']).read_bytes(), 1: rel(m['locations']).read_bytes()}
    if clashes:
        raise SystemExit('ids already in the source cache, nothing written: ' + ', '.join(clashes))
    # every model id a definition names must exist in the result
    known = set(ref(7)['groups']) | set(models) if (models or configs) else set()
    for kind in ('objects', 'npcs', 'items'):
        for i, v in man.get(kind, {}).items():
            named = []
            if kind == 'items': named = [v.get('model', 0)] + sum(v.get('wear_models', []), []) + sum(v.get('head_models', []), [])
            elif 'models' in v: named = v['models']
            for m in named:
                if m not in (-1, 0) and m not in known: raise SystemExit(f'{kind[:-1]} {i} names model {m}, which is neither in the cache nor in the manifest')

    for i, v in man.get('spots', {}).items():
        assert int(i) <= 65535, 'a spot animation id is two bytes in every packet'
        if v['model'] not in set(ref(7)['groups']) | set(models): raise SystemExit(f'spot {i} names model {v["model"]}, which is neither in the cache nor in the manifest')
        if v.get('sequence', -1) >= 0 and v['sequence'] not in set(ref(2)['groups'][12]['files']) | set(seqs): raise SystemExit(f'spot {i} names sequence {v["sequence"]}, which is neither in the cache nor in the manifest')

    # ---- copy, then write
    out.mkdir(parents=True)
    for p in src.root.iterdir():
        if p.name.startswith('main_file_cache.'):
            shutil.copy2(p, out / p.name)
    print('free-id check passed; reference tables', sorted(refs), 're-encode byte-identical; copied', src.root, '->', out)
    for mid in sorted(models):
        store(out, 7, mid, {0: models[mid]}, refs[7], version=1)
    if models: print(f'archive 7: {len(models)} models, ids {min(models)}..{max(models)}')
    for b in sorted(bases):
        store(out, 1, b, {0: bases[b]}, refs[1], version=1)
        print(f'archive 1 group {b}: frame base, {len(bases[b])} bytes, {bases[b][0]} groups')
    for g in sorted(frames):
        store(out, 0, g, frames[g], refs[0], version=1)
        print(f'archive 0 group {g}: {len(frames[g])} frame files')
    for kind, add in configs.items():
        gid = CONFIG[kind]
        files = dict(src.files(2, gid)); n = len(files)
        files.update(add)
        store(out, 2, gid, files, refs[2])
        print(f'archive 2 group {gid} ({kind}): {n} -> {len(files)} files; added {sorted(add)}')
    for rid in sorted(maps):
        store(out, 5, rid, maps[rid], refs[5], version=1)
        print(f'archive 5 group {rid}: map square ({rid >> 8}, {rid & 255}), world tiles x {(rid >> 8) * 64}.. y {(rid & 255) * 64}..; '
              f'terrain {len(maps[rid][0])} bytes, placements {len(maps[rid][1])} bytes')
    for a in sorted(refs):
        r = refs[a]; r['version'] += 1
        container = A.pack_container(A.build_reference(r))
        write_group(out, 255, a, container + raws[a][A.container_span(raws[a]):])

    # ---- read back with this file's reader
    chk = A.Cache(out); n = 0
    for mid, b in models.items():
        assert chk.file(7, mid, 0) == b; n += 1
    for b_, data in bases.items():
        assert chk.file(1, b_, 0) == data; n += 1
    for g, files in frames.items():
        for f, b in files.items():
            assert chk.file(0, g, f) == b; n += 1
    for kind, add in configs.items():
        back = chk.files(2, CONFIG[kind]); orig = src.files(2, CONFIG[kind])
        for i, b in add.items():
            assert back[i] == b; n += 1
        assert all(back[i] == v for i, v in orig.items()), f'real {kind} changed'
        assert len(back) == len(orig) + len(add)
    for rid, files in maps.items():
        for f, b in files.items():
            assert chk.file(5, rid, f) == b; n += 1
    for a in refs:                                   # untouched groups keep CRC and version
        old = src.reference(a)['groups']; new = chk.reference(a)['groups']
        same = sum(1 for g in old if old[g]['crc'] == new[g]['crc'] and old[g]['version'] == new[g]['version'])
        print(f'archive {a}: {len(old)} -> {len(new)} groups; {same} unchanged, {len(old) - same} rewritten, {len(new) - len(old)} new')
    print(f'read back {n} added files byte for byte: OK. packed cache at {out}')


if __name__ == '__main__':
    main()
