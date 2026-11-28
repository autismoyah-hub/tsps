"""readback.py: read everything in MANIFEST back from the packed cache THROUGH THE CLIENT (examples/live_readback.rs and
research_items.exe defs) and compare with the inputs: raw files byte for byte, decoded fields field for field.

usage: readback.py PACKED_CACHE MANIFEST.json WORKDIR
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, re, subprocess, pathlib, shutil
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE)); sys.dont_write_bytecode = True
import pack
EX = _AP.bindir()
cache, man_path, work = sys.argv[1], pathlib.Path(sys.argv[2]), pathlib.Path(sys.argv[3])
man = json.loads(man_path.read_text())
if work.exists(): shutil.rmtree(work)
work.mkdir(parents=True)
what = ['counts'] + [f'model:{i}' for i in man['models']] + [f'object:{i}' for i in man['objects']] + [f'npc:{i}' for i in man['npcs']] \
    + [f'item:{i}' for i in man['items']] + [f'seq:{i}' for i in man['sequences']] + [f'map:{i}' for i in man['maps']]
cmd = [str(_AP.exe('live_readback.exe', EX)), cache, str(work)] + what
print('$', ' '.join(cmd[:3]), f'counts + {len(what) - 1} ids')
r = subprocess.run(cmd, capture_output=True, text=True)
if r.returncode: raise SystemExit(r.stdout + r.stderr)
(work / 'client-lines.txt').write_text(r.stdout)
lines = {}
for l in r.stdout.splitlines():
    k = l.split()
    lines[(k[0], k[1])] = l
print(lines[('counts', lines and next(k[1] for k in lines if k[0] == 'counts'))])
nbytes = nfields = 0
def same(name, a, b):
    global nbytes
    if a != b: raise SystemExit(f'DIFFERENT: {name}')
    nbytes += 1
def field(name, got, want):
    global nfields
    if str(got) != str(want): raise SystemExit(f'FIELD DIFFERENT: {name}: client {got!r}, input {want!r}')
    nfields += 1
get = lambda line, key: re.search(rf'\b{key}=(\[.*?\]|Some\(\[.*?\]\)|"[^"]*"|\S+)', line).group(1)
for i, p in man['models'].items():
    same(f'model {i}', (work / f'model-{i}.bin').read_bytes(), pathlib.Path(p).read_bytes())
for i, d in man['objects'].items():
    b = pack.encode_object(d); same(f'object {i}', (work / f'object-{i}.bin').read_bytes(), b)
    l = lines[('object', i)]
    # independent parse of the input bytes: lighting, the one model with or without shape, name, size
    p = 0; amb = con = 0; shape = None; merge = False
    while b[p] != 0:
        op = b[p]
        if op == 39: con = int.from_bytes(b[p + 1:p + 2], 'big', signed=True) * 5; p += 2
        elif op == 29: amb = int.from_bytes(b[p + 1:p + 2], 'big', signed=True); p += 2
        elif op == 22: merge = True; p += 1
        elif op in (6, 7):
            assert b[p + 1] == 1; mid = int.from_bytes(b[p + 2:p + 6], 'big'); p += 6
            if op == 6: shape = b[p]; p += 1
        elif op == 2:
            e = b.index(0, p + 1); name = b[p + 1:e].decode(); p = e + 1
        elif op == 14: sx = b[p + 1]; p += 2
        elif op == 15: sz = b[p + 1]; p += 2
        elif op == 17: p += 1
        elif op == 19: p += 2
        else: raise SystemExit(f'object {i}: opcode {op} not expected in our definitions')
    field(f'object {i} models', get(l, 'models'), f'[{mid}]')
    field(f'object {i} shapes', get(l, 'shapes'), f'Some([{shape}])' if shape is not None else 'None')
    field(f'object {i} name', get(l, 'name'), f'"{name}"')
    field(f'object {i} size', get(l, 'size').replace(' ', ''), f'[{sx},{sz}]')
    field(f'object {i} ambient', get(l, 'ambient'), amb); field(f'object {i} contrast', get(l, 'contrast'), con)
    field(f'object {i} merge', get(l, 'non_flat_shading'), str(merge).lower())
for i, d in man['npcs'].items():
    same(f'npc {i}', (work / f'npc-{i}.bin').read_bytes(), pack.encode_npc(d))
    l = lines[('npc', i)]
    field(f'npc {i} name', get(l, 'name'), f'"{d["name"]}"')
    field(f'npc {i} models', get(l, 'models').replace(' ', ''), json.dumps(d['models']).replace(' ', ''))
    field(f'npc {i} size', get(l, 'size'), d['size']); field(f'npc {i} stand', get(l, 'stand'), d['stand']); field(f'npc {i} walk', get(l, 'walk'), d['walk'])
    field(f'npc {i} recolor', get(l, 'recolor').replace(' ', '').replace('(', '[').replace(')', ']'), json.dumps(d['recolor']).replace(' ', ''))
    field(f'npc {i} scale', get(l, 'scale').replace(' ', ''), json.dumps(d['scale']).replace(' ', ''))
    field(f'npc {i} ambient', get(l, 'ambient'), d['ambient']); field(f'npc {i} contrast', get(l, 'contrast'), d['contrast'] * 5)
    field(f'npc {i} z_buffer', get(l, 'z_buffer'), str(d['zbuffer']).lower()); field(f'npc {i} combat', get(l, 'combat'), d['combat'])
for i, d in man['items'].items():
    same(f'item {i}', (work / f'item-{i}.bin').read_bytes(), pack.encode_item(d))
for i, d in man['sequences'].items():
    d = pathlib.Path(d); same(f'seq {i}', (work / f'seq-{i}.bin').read_bytes(), (d / 'seq.dat').read_bytes())
    for p in d.glob('frame_*.dat'):
        _, g, f = p.stem.split('_')
        if (work / f'frame-{g}-{f}.bin').exists(): same(f'frame {g}:{f}', (work / f'frame-{g}-{f}.bin').read_bytes(), p.read_bytes())
    l = lines[('seq', i)]
    field(f'seq {i} frames decoded', get(l, 'decoded_frames'), len(json.loads(get(l, 'frames'))))
for i, d in man['maps'].items():
    same(f'map {i} terrain', (work / f'map-{i}-0.bin').read_bytes(), pathlib.Path(d['terrain']).read_bytes())
    same(f'map {i} locations', (work / f'map-{i}-1.bin').read_bytes(), pathlib.Path(d['locations']).read_bytes())
    want = len(json.loads(pathlib.Path(d['locations']).with_suffix('.json').read_text()))
    field(f'map {i} placements', get(lines[('map', i)], 'placements'), want)
    field(f'map {i} underlay tiles', get(lines[('map', i)], 'tiles_with_underlay'), 4096)
# items, every field, by the client's Definitions loader (research_items defs = Definition::decode_local on each record)
import itemcodec
out = work / 'items.jsonl'
subprocess.run([str(_AP.exe('research_items.exe', EX)), cache, 'defs', str(out)], check=True, capture_output=True)
rows = {}
for l in out.read_text(encoding='utf-8').splitlines():
    j = json.loads(l)
    if str(j['id']) in man['items']: rows[str(j['id'])] = j
for i, spec in man['items'].items():
    want = dict(itemcodec.DEFAULT); want.update(spec)
    j = rows[i]
    for k, v in want.items():
        if k in ('trailing', 'searchable'): continue
        if k not in j: raise SystemExit(f'item {i}: the client row has no field {k}')
        field(f'item {i} {k}', json.dumps(j[k], sort_keys=True), json.dumps(v, sort_keys=True))
print(f'client read-back: {nbytes} files equal to the inputs byte for byte; {nfields} decoded fields equal; 0 different')
