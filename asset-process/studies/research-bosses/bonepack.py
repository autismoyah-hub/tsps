"""Pack a skeletal creature made by OUR encoders into a PRIVATE COPY of the cache under new ids, so the client's
ordinary loaders fetch every part by id (models archive 7, frame base archive 1, keyframe files archive 22, sequences
archive 2 group 12, NPC archive 2 group 9). The source cache is only read; OUT must not exist.

usage: bonepack.py RT_DIR OUT_CACHE [--authored DIR:SEQ_ID ...]
RT_DIR is a bone_rt.py output folder: its `renumbered` files are packed as
  model 62500.., frame base (real id + 10000), one keyframe file per real one in archive 22 groups 3000.., a copy of
  every real keyframe sequence on the base as sequence 21000.. (same definition, opcode 13 naming our file), and
  NPC 16370 (the real definition's values, our models, our stand and walk).
--authored DIR:SEQ packs DIR/anim.dat as a new archive-22 group and DIR/seq.dat (re-pointed) as sequence SEQ.
Writes OUT_CACHE/bonepack.json: the id map (real sequence -> ours).
Uses research-skeleton/skelpack.py's store / write_group / encode_npc and animlib's container and reference code.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, shutil, copy
HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE)); sys.path.insert(1, _AP.fix(r'<ROOT>\studies\research-skeleton'))
import bonelib as L, bonecodec as B, skelpack as SP
A = L.A
a = sys.argv; rt = pathlib.Path(a[1]); out = pathlib.Path(a[2])
if out.exists(): raise SystemExit(f'{out} exists')
rep = json.loads((rt / 'roundtrip.json').read_text()); npc_id = rep['npc']; bid = rep['base']; nbid = bid + 10000
src = A.Cache(); refs = {}; raws = {}
def ref(x):
    if x not in refs:
        refs[x] = copy.deepcopy(src.reference(x)); raws[x] = src.raw(255, x)
        assert A.build_reference(refs[x]) == A.unpack(raws[x]), f'reference table {x} does not re-encode exactly'
    return refs[x]
R = rt / 'renumbered'; d = L.npc_def(npc_id)
models = {62500 + i: p.read_bytes() for i, p in enumerate(sorted(R.glob('part-*.bin'), key=lambda p: int(p.stem.split('-')[1])))}
base = (R / f'base_{nbid}.dat').read_bytes()
g0 = max(max(ref(22)['groups']) + 1, 3000); anim_map = {}; anims = {}
for i, x in enumerate(rep['anims']):
    anims[g0 + i] = (R / f"anim_{x['id']}.dat").read_bytes(); anim_map[x['id']] = (g0 + i) << 16
seq_files = dict(src.files(2, 12)); s0 = max(max(seq_files) + 1, 21000); seq_map = {}; new_seqs = {}
for i, row in enumerate(rep['sequences']):
    s = A.decode_sequence(seq_files[row['seq']]); s['cached_animation'] = anim_map[row['anim']]
    new_seqs[s0 + i] = A.encode_sequence(s); seq_map[row['seq']] = s0 + i
authored = {}
for k, spec in enumerate(a[i + 1] for i, x in enumerate(a) if x == '--authored'):
    dpath, sid = spec.rsplit(':', 1); dpath = pathlib.Path(dpath); sid = int(sid); g = g0 + len(anims) + k
    seq = bytearray((dpath / 'seq.dat').read_bytes()); assert seq[2] == 13; seq[3:7] = (g << 16).to_bytes(4, 'big')
    authored[sid] = (g, (dpath / 'anim.dat').read_bytes(), bytes(seq)); assert sid not in seq_files and sid not in new_seqs
npc_new = 16370
kv = dict(l.split('=', 1) for l in (rt / 'raw' / f'npc-{npc_id}.npcdef').read_text().splitlines() if '=' in l and not l.startswith('#'))
npc = dict(name=d['name'] + ' (ours)', models=sorted(models), recolor=d['recolor'], scale=d['scale'], ambient=d['ambient'], contrast=int(kv['contrast']), size=d['size'], zbuffer=d['zbuffer'],
           stand=seq_map[d['anims'][0]], walk=seq_map.get(d['anims'][3], seq_map[d['anims'][0]]), combat=0, ops=[None, 'Attack', None, None, None])
for mid in models: assert mid not in ref(7)['groups'], f'model {mid} exists'
assert nbid not in ref(1)['groups']; assert all(g not in ref(22)['groups'] for g in anims)
assert npc_new not in src.files(2, 9), 'npc id taken'; ref(2)
out.mkdir(parents=True)
for p in src.root.iterdir():
    if p.name.startswith('main_file_cache.'): shutil.copy2(p, out / p.name)
for mid, b in models.items(): SP.store(out, 7, mid, {0: b}, refs[7], version=1)
SP.store(out, 1, nbid, {0: base}, refs[1], version=1)
for g, b in anims.items(): SP.store(out, 22, g, {0: b}, refs[22], version=1)
for sid, (g, b, seq) in authored.items(): SP.store(out, 22, g, {0: b}, refs[22], version=1); new_seqs[sid] = seq
files = dict(seq_files); files.update(new_seqs); SP.store(out, 2, 12, files, refs[2])
nf = dict(src.files(2, 9)); nf[npc_new] = SP.encode_npc(npc); SP.store(out, 2, 9, nf, refs[2])
for x in sorted(refs):
    r = refs[x]; r['version'] += 1
    SP.write_group(out, 255, x, A.pack_container(A.build_reference(r)) + raws[x][A.container_span(raws[x]):])
chk = A.Cache(out)
for mid, b in models.items(): assert chk.file(7, mid, 0) == b
assert chk.file(1, nbid, 0) == base
for g, b in anims.items(): assert chk.file(22, g, 0) == b
back = chk.files(2, 12); assert all(back[k] == v for k, v in new_seqs.items()) and all(back[k] == v for k, v in seq_files.items())
info = dict(real_npc=npc_id, npc=npc_new, models=sorted(models), base=nbid, sequences={str(k): v for k, v in seq_map.items()}, authored={str(s): g << 16 for s, (g, _, _) in authored.items()})
(out / 'bonepack.json').write_text(json.dumps(info, indent=1))
print(f'packed: NPC {npc_new}, models {sorted(models)}, frame base {nbid}, {len(anims)} keyframe files in archive 22 groups {min(anims)}..{max(anims)}, sequences {min(seq_map.values())}..{max(seq_map.values())}, authored {info["authored"]}; read back OK ->', out)
