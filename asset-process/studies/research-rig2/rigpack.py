"""Pack a NEW skeletal creature (model, frame base with bones, keyframe files, sequences, NPC) into a PRIVATE COPY of
the cache under free ids, so the client's ordinary loaders fetch every part by id.

usage: rigpack.py EXPORTDIR OUT_CACHE MANIFEST.json
EXPORTDIR is rig_export.py's. MANIFEST.json:
{ "name": "Worm", "npc": 16380, "models": {"62600": "part-0.bin"}, "base": 12700, "base_file": "base_12700.dat",
  "sequences": {"21200": {"file": "anim_sway.dat", "group": 3100, "priority": 5}, ...},
  "stand": 21200, "walk": 21200, "scale": [128, 128], "ambient": 30, "contrast": 20, "size": 1 }
research-bosses/bonepack.py does this for a RENUMBERED REAL creature (it copies the real NPC's and sequences'
definitions); a creature that never existed has none to copy, so this tool writes them, with the same underlying
writers (research-skeleton/skelpack.py store / write_group / encode_npc, animlib's containers and reference tables).
The source cache is only read; OUT_CACHE must not exist; every id must be free. Everything is read back and compared.
A keyframe file's own base id (bytes 1..2) must equal "base". Sequence definition written: opcode 5 (priority),
opcode 13 (keyframe file id = group << 16), opcode 15 (range 0..length), end.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, shutil, copy
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE)); sys.path.insert(1, _AP.fix(r'<ROOT>\studies\research-skeleton'))
import riglib as R
import skelpack as SP
A = R.L.A; B = R.B
a = sys.argv; exp = pathlib.Path(a[1]).resolve(); out = pathlib.Path(a[2]); man = json.loads(pathlib.Path(a[3]).read_text())
if out.exists(): raise SystemExit(f'{out} exists')
src = A.Cache(); refs = {}; raws = {}
def ref(x):
    if x not in refs:
        refs[x] = copy.deepcopy(src.reference(x)); raws[x] = src.raw(255, x)
        assert A.build_reference(refs[x]) == A.unpack(raws[x]), f'reference table {x} does not re-encode exactly'
    return refs[x]
models = {int(k): (exp / v).read_bytes() for k, v in man['models'].items()}; bid = int(man['base']); base = (exp / man['base_file']).read_bytes()
B.decode_base(base); seq_files = dict(src.files(2, 12)); new_seqs = {}; anims = {}
for sid, s in man['sequences'].items():
    sid = int(sid); data = (exp / s['file']).read_bytes(); an = B.decode_anim(data); assert an['base'] == bid, f"{s['file']} names base {an['base']}, not {bid}"
    g = int(s['group']); anims[g] = data; n = an['range'][1]
    new_seqs[sid] = bytes([5, int(s.get('priority', 5)), 13]) + (g << 16).to_bytes(4, 'big') + bytes([15]) + (0).to_bytes(2, 'big') + n.to_bytes(2, 'big') + b'\0'
    assert sid not in seq_files, f'sequence {sid} exists'
for mid in models: assert mid not in ref(7)['groups'], f'model {mid} exists'
assert bid not in ref(1)['groups'], f'frame base {bid} exists'; assert all(g not in ref(22)['groups'] for g in anims), 'keyframe group exists'
npc_id = int(man['npc']); assert npc_id not in src.files(2, 9), 'npc id taken'; ref(2)
npc = dict(name=man['name'], models=sorted(models), recolor=[], scale=man.get('scale', [128, 128]), ambient=man.get('ambient', 0), contrast=man.get('contrast', 0), size=man.get('size', 1), zbuffer=True,
           stand=int(man['stand']), walk=int(man.get('walk', man['stand'])), combat=0, ops=[None, 'Attack', None, None, None])
out.mkdir(parents=True)
for p in src.root.iterdir():
    if p.name.startswith('main_file_cache.'): shutil.copy2(p, out / p.name)
for mid, b in models.items(): SP.store(out, 7, mid, {0: b}, refs[7], version=1)
SP.store(out, 1, bid, {0: base}, refs[1], version=1)
for g, b in anims.items(): SP.store(out, 22, g, {0: b}, refs[22], version=1)
files = dict(seq_files); files.update(new_seqs); SP.store(out, 2, 12, files, refs[2])
nf = dict(src.files(2, 9)); nf[npc_id] = SP.encode_npc(npc); SP.store(out, 2, 9, nf, refs[2])
for x in sorted(refs):
    r = refs[x]; r['version'] += 1
    SP.write_group(out, 255, x, A.pack_container(A.build_reference(r)) + raws[x][A.container_span(raws[x]):])
chk = A.Cache(out)
for mid, b in models.items(): assert chk.file(7, mid, 0) == b
assert chk.file(1, bid, 0) == base
for g, b in anims.items(): assert chk.file(22, g, 0) == b
back = chk.files(2, 12); assert all(back[k] == v for k, v in new_seqs.items()) and all(back[k] == v for k, v in seq_files.items())
(out / 'rigpack.json').write_text(json.dumps(man, indent=1))
print(f"packed: NPC {npc_id} {man['name']!r}, models {sorted(models)}, frame base {bid} ({len(base)} bytes), keyframe groups {sorted(anims)}, sequences {sorted(new_seqs)}; read back OK -> {out}")
