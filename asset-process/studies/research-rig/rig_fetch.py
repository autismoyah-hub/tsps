"""Fetch one real skeletal creature out of the cache (read only) as loose files for the Blender importer.

usage: rig_fetch.py NPC_ID OUTDIR [--check]
Writes OUTDIR/src/: the NPC's model files as stored (asset_npc dump), base_ID.dat, anim_ID.dat for every keyframe
sequence on the creature's frame base, and OUTDIR/manifest.json (definition values, the files, the sequences with
their lengths, and which sequence is stand / walk / attack / death).
--check: this folder's float64 port of the client's pose and skin (rigmath) against the client itself: the bounding
box asset_bone prints for six frames of each named sequence must equal the port's after truncation.
"""
import sys, json, pathlib, re
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import riglib as R
import numpy as np
L, B, RM = R.L, R.B, R.RM
a = sys.argv; npc = int(a[1]); out = pathlib.Path(a[2]).resolve(); src = out / 'src'; src.mkdir(parents=True, exist_ok=True)
d = L.npc_def(npc); kv, files = L.dump_npc(npc, src)
named = R.RL.sequences(npc); bid = L.base_of_seq(named['stand'])
(src / f'base_{bid}.dat').write_bytes(L.base_bytes(bid))
seqs = []
for sid, n in L.seqs_on_base(bid):
    aid, rng = L.anim_id(sid); p = src / f'anim_{aid}.dat'
    if not p.exists(): p.write_bytes(L.anim_bytes(aid))
    seqs.append(dict(seq=sid, frames=n, anim=aid, file=p.name, range=list(rng)))
man = dict(npc=npc, name=d['name'], src='src', models=[f.name for f in files], base=bid, base_file=f'base_{bid}.dat', sequences=seqs, named=named,
           scale=d['scale'], ambient=d['ambient'], contrast_stored=int(kv['contrast']), size=d['size'], zbuffer=d['zbuffer'], recolor=kv.get('recolor', ''), npcdef=f'npc-{npc}.npcdef')
(out / 'manifest.json').write_text(json.dumps(man, indent=1))
print(f"NPC {npc} {d['name']}: {len(files)} model files, base {bid}, {len(seqs)} keyframe sequences ({len({s['anim'] for s in seqs})} files), named {named} -> {out / 'manifest.json'}")
if '--check' in a:
    models = [B.decode_model(f.read_bytes()) for f in files]
    if len(models) != 1: raise SystemExit('--check handles one-model creatures (the client welds several models first)')
    m = models[0]; sk = RM.Skeleton(B.decode_base((src / f'base_{bid}.dat').read_bytes())); W = RM.weight_matrix(m['bone_weights'], sk.n); worst = 0; cells = 0
    for name, sid in named.items():
        s = next(x for x in seqs if x['seq'] == sid); an = B.decode_anim((src / s['file']).read_bytes()); n = s['frames']
        fr = sorted({int(round(k * (n - 1) / 5)) for k in range(6)})
        _, _, text = L.grid(f'npc:{npc}', seq=sid, frames=','.join(map(str, fr)), cell='64x64')
        boxes = [(eval(m_.group(1)), eval(m_.group(2))) for m_ in re.finditer(r'box min (\[.*?\]) max (\[.*?\])', text)]
        pos = RM.skin_fast(m['vertices'], W, sk.skin_matrices(RM.channel_table(an, sk, fr))) * np.array([d['scale'][0], d['scale'][1], d['scale'][0]]) / 128
        for k, f in enumerate(fr):
            lo, hi = np.trunc(pos[k].min(0)).astype(int), np.trunc(pos[k].max(0)).astype(int); cells += 1
            worst = max(worst, int(np.abs(lo - boxes[k][0]).max()), int(np.abs(hi - boxes[k][1]).max()))
    print(f'port against client: {cells} posed frames, worst bounding-box difference {worst} units (0 = the port poses as the client does)')
