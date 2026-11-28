"""Round trip of the three skeletal encoders on one real creature, judged by the client's own drawing.

usage: bone_rt.py NPC_ID OUTDIR [--seqs A,B,..] [--every K] [--views 192,704] [--cell 300]

1. The NPC's model files, its frame base and the keyframe file of every keyframe sequence on that base are read
   from the cache, DECODED by bonecodec and RE-ENCODED twice:
     plain       encode_model(hints=False), encode_base, encode_anim: our encoder's own choices, same numbers;
     renumbered  bonelib.renumber: a new base id (BASE + 10000), the bones in a shuffled order, every model's vertices
                 in reverse order, every keyframe file's channels in reverse order (weights and targets remapped).
   Byte identity with the cache is reported for `plain`; `renumbered` cannot be byte-identical by construction.
2. Every frame (or every K-th) of every sequence is drawn three ways by examples/asset_bone.rs, from two sides:
     real        npc:ID seq=S                 the client's own loaders (cache) and Animations::apply_observed
     plain       def:files anim=FILE base=FILE the loose-file path: Skeleton::decode, CachedAnimation::decode, Rig, skin
     renumbered  the same with the renumbered files
   and compared pixel for pixel and by the hash of the posed vertices' float bits.
Writes OUTDIR/plain/*, OUTDIR/renumbered/*, OUTDIR/roundtrip.json and prints one line per sequence.
"""
import sys, pathlib, json
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import bonelib as L, bonecodec as B
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
npc_id = int(a[1]); out = pathlib.Path(a[2]); out.mkdir(parents=True, exist_ok=True)
every = int(opt('--every', 1)); views = [int(v) for v in opt('--views', '192,704').split(',')]; cs = int(opt('--cell', 300))
d = L.npc_def(npc_id)
kv, files = L.dump_npc(npc_id, out / 'raw')
stand = d['anims'][0]
bid = L.base_of_seq(stand)
if bid is None: raise SystemExit(f'NPC {npc_id}: stand sequence {stand} is not a keyframe sequence')
seqs = L.seqs_on_base(bid)
if opt('--seqs'): want = {int(x) for x in opt('--seqs').split(',')}; seqs = [s for s in seqs if s[0] in want]
report = dict(npc=npc_id, name=d['name'], base=bid, models=[], anims=[], sequences=[])
# ---- decode, encode
raw_models = [f.read_bytes() for f in files]; models = [B.decode_model(x) for x in raw_models]
raw_base = L.base_bytes(bid); base = B.decode_base(raw_base)
aids = {}; anims = {}
for sid, n in seqs:
    aid, rng = L.anim_id(sid); aids[sid] = aid
    if aid not in anims: anims[aid] = L.anim_bytes(aid)
dec = {aid: B.decode_anim(x) for aid, x in anims.items()}
dups = 0
for x in dec.values():
    seen = set()
    for c in x['channels']:
        k = (c['kind'], c['target'], c['channel']); dups += k in seen; seen.add(k)
if dups: raise SystemExit(f'{dups} repeated channel slots: reversing the channel order would change which one wins')
P = out / 'plain'; R = out / 'renumbered'; P.mkdir(exist_ok=True); R.mkdir(exist_ok=True)
pm = []
for i, (m, x, f) in enumerate(zip(models, raw_models, files)):
    e = B.encode_model(m, hints=False); p = P / f'part-{i}.bin'; p.write_bytes(e); pm.append(p)
    report['models'].append(dict(file=f.name, bytes=len(x), vertices=len(m['vertices']), faces=len(m['faces']), weighted=m['bone_weights'] is not None,
                                 influences=sum(len(w) for w in (m['bone_weights'] or [])), byte_identical=e == x, with_hints_identical=B.encode_model(m) == x))
eb = B.encode_base(base); (P / f'base_{bid}.dat').write_bytes(eb); report['base_byte_identical'] = eb == raw_base
report['base_groups'] = len(base['types']); report['bones'] = len(base['bones'])
for aid, x in anims.items():
    e = B.encode_anim(dec[aid]); (P / f'anim_{aid}.dat').write_bytes(e)
    report['anims'].append(dict(id=aid, group=aid >> 16, file=aid & 65535, bytes=len(x), channels=len(dec[aid]['channels']), keys=sum(len(c['curve']['keys']) for c in dec[aid]['channels']), byte_identical=e == x))
nbid = bid + 10000
rm, rb, ra = L.renumber(models, base, [dec[k] for k in anims], nbid)
rmf = []
for i, m in enumerate(rm):
    p = R / f'part-{i}.bin'; p.write_bytes(B.encode_model(m, hints=False)); rmf.append(p)
(R / f'base_{nbid}.dat').write_bytes(B.encode_base(rb))
for aid, x in zip(anims, ra): (R / f'anim_{aid}.dat').write_bytes(B.encode_anim(x))
report['renumbered_differs_from_cache'] = dict(models=[p.read_bytes() != x for p, x in zip(rmf, raw_models)], base=(R / f'base_{nbid}.dat').read_bytes() != raw_base,
                                               anims=all((R / f'anim_{aid}.dat').read_bytes() != anims[aid] for aid in anims))
pdef = L.write_npcdef(P / 'ours.npcdef', d['name'] + ' (re-encoded)', pm, kv)
rdef = L.write_npcdef(R / 'ours.npcdef', d['name'] + ' (renumbered)', rmf, kv)
print(f"NPC {npc_id} {d['name']}: base {bid} ({len(base['types'])} groups, {len(base['bones'])} bones); {len(models)} models; {len(seqs)} keyframe sequences, {len(anims)} keyframe files")
print('models byte-identical (no hints):', [m['byte_identical'] for m in report['models']], '| base byte-identical:', report['base_byte_identical'],
      '| keyframe files byte-identical:', sum(x['byte_identical'] for x in report['anims']), 'of', len(report['anims']))
# ---- draw
M = L.merged(f'npc:{npc_id}', out / 'rest.txt'); sc = d['scale']
ext = (M['V'].max(0) - M['V'].min(0)) * [sc[0] / 128, sc[1] / 128, sc[0] / 128]
zoom = max(300, int(1.75 * max(ext[0], ext[2], ext[1] * 1.1)))
tot = dict(frames=0, cells=0, drawn=0, plain=0, ren=0, hp=0, hr=0)
for sid, n in seqs:
    aid = aids[sid]; row = dict(seq=sid, anim=aid, frames=n, views={})
    for yaw in views:
        o = dict(frames=f'every:{every}' if every > 1 else 'all', cols=20, zoom=zoom, yaw=yaw, pitch=128, cell=f'{cs}x{cs}')
        real, hr, _ = L.grid(f'npc:{npc_id}', seq=sid, **o)
        pl, hp, _ = L.grid(f'def:{pdef}', anim=P / f'anim_{aid}.dat', base=P / f'base_{bid}.dat', count=n, **o)
        rn, hn, _ = L.grid(f'def:{rdef}', anim=R / f'anim_{aid}.dat', base=R / f'base_{nbid}.dat', count=n, **o)
        dp = L.diff_cells(real, pl, cs, cs); dr = L.diff_cells(real, rn, cs, cs); cells = len(hr)
        row['views'][yaw] = dict(cells=cells, drawn=sum(x[1] for x in dp[:cells]), pixels=cells * cs * cs, plain_differ=sum(x[0] for x in dp), renumbered_differ=sum(x[0] for x in dr),
                                 plain_vertex_hash_equal=sum(hr[k] == hp.get(k) for k in hr), renumbered_cells_with_pixels_differing=sum(1 for x in dr if x[0]),
                                 distinct_poses=len(set(hr.values())))
        v = row['views'][yaw]
        tot['cells'] += cells; tot['drawn'] += v['drawn']; tot['plain'] += v['plain_differ']; tot['ren'] += v['renumbered_differ']; tot['hp'] += v['plain_vertex_hash_equal']
    tot['frames'] += len(hr)
    report['sequences'].append(row)
    print(f"seq {sid} ({n} frames, {len(hr)} drawn x {len(views)} views, {row['views'][views[0]]['distinct_poses']} distinct poses): figure pixels {sum(v['drawn'] for v in row['views'].values())}; "
          f"plain differ {sum(v['plain_differ'] for v in row['views'].values())}; renumbered differ {sum(v['renumbered_differ'] for v in row['views'].values())}; "
          f"posed vertices bit-equal (plain) {sum(v['plain_vertex_hash_equal'] for v in row['views'].values())} of {sum(v['cells'] for v in row['views'].values())}", flush=True)
report['total'] = tot; report['zoom'] = zoom; report['cell'] = cs; report['views'] = views; report['every'] = every
(out / 'roundtrip.json').write_text(json.dumps(report, indent=1))
print(f"TOTAL: {tot['frames']} frames, {tot['cells']} cells of {cs}x{cs} = {tot['cells'] * cs * cs} pixels, {tot['drawn']} figure pixels: plain {tot['plain']} differing, renumbered {tot['ren']} differing; posed vertices bit-equal in {tot['hp']} of {tot['cells']} cells (plain)")
