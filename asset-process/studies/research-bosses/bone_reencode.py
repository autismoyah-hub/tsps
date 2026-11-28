"""The ceiling case, and deliberately degraded copies, for the rebuild test of a BONE-WEIGHTED creature.

usage: bone_reencode.py NPC_ID OUTDIR [degradations] [--name NAME]
       bone_reencode.py NPC_ID --limbs        list the bone subtrees (candidate limbs) with the vertices they carry
Writes the real NPC's model files re-encoded through OUR encoder (bonecodec.encode_model, hints off) as
OUTDIR/NAME-part-N.bin and OUTDIR/NAME.json (recolours, scale, ambient, contrast, stand, walk, attack, death).
With no degradation this is the CEILING: it must score as identical.

Degradations (seeded, repeatable):
  --flatten BONE[:TO]   every vertex that carries any weight on BONE or a bone below it is given ONE influence, weight
                        255, on TO (default: BONE's parent): the limb stops following its own bones and rides its
                        parent rigidly. Identical at rest (weights do nothing at rest); wrong in motion.
  --jitter N            move every vertex position by up to +/-N units on each axis (equal positions move equally)
  --tones Q             round every painted lightness to a multiple of Q
  --lightshift K        add K to every painted lightness
"""
import sys, pathlib, json, random, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import bone_rebuild_lib as BL, bonelib, bonecodec as B
L = BL.L
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
npc_id = int(a[1]); d = L.npc_def(npc_id)
if not BL.is_weighted(npc_id): raise SystemExit('not a bone-weighted NPC: use research-creatures/reencode_npc.py')
seqs = L.sequences(npc_id); bid = bonelib.base_of_seq(seqs['stand']); bones, base = BL.skeleton(bid)
children = collections.defaultdict(list)
for b in bones: children[b['parent']].append(b['bone'])
def subtree(i):
    out = [i]
    for c in children[i]: out += subtree(c)
    return out
if '--limbs' in a:
    m = BL.with_weights(f'npc:{npc_id}'); used = set(int(i) for i in __import__('numpy').unique(m['F'])); n = len(used)
    print(f'NPC {npc_id} {d["name"]}: base {bid}, {len(bones)} bones, {n} vertices; subtrees carrying 2% to 30% of the vertices:')
    for b in bones:
        st = set(subtree(b['bone'])); k = sum(1 for i in used if any(x in st for x, _ in m['W'][i]))
        if .02 * n <= k <= .30 * n: print(f"  bone {b['bone']:3d} parent {b['parent']:3d} depth {b['depth']:2d} subtree {len(st):3d} bones, {k:4d} vertices ({100 * k / n:.0f}%), rest at {[round(v) for v in b['rest']]}")
    raise SystemExit
out = pathlib.Path(a[2]); out.mkdir(parents=True, exist_ok=True)
kv, files = bonelib.dump_npc(npc_id, out / 'raw')
parts = [B.decode_model(f.read_bytes()) for f in files]
degraded = any(x in a for x in ('--flatten', '--jitter', '--tones', '--lightshift'))
rng = random.Random(239); notes = []
if opt('--flatten'):
    spec = opt('--flatten'); bone, _, to = spec.partition(':'); bone = int(bone); to = int(to) if to else bones[bone]['parent']; st = set(subtree(bone)); n = 0
    for p in parts:
        for i, ws in enumerate(p['bone_weights']):
            if any(b in st for b, _ in ws): p['bone_weights'][i] = [(to, 255)]; n += 1
    notes.append(f'weights of bone {bone} and the {len(st) - 1} bones below it flattened to bone {to} ({n} vertices)')
if opt('--jitter'):
    k = int(opt('--jitter')); move = {}
    for p in parts:
        nv = []
        for v in p['vertices']:
            if v not in move: move[v] = tuple(c + rng.randint(-k, k) for c in v)
            nv.append(move[v])
        p['vertices'] = nv
    notes.append(f'jitter +/-{k}')
def recol(fn):
    for p in parts: p['colors'] = [fn(c) if c != 65535 else c for c in p['colors']]
if opt('--tones'):
    q = int(opt('--tones')); recol(lambda c: (c & ~127) | max(2, min(126, int(round((c & 127) / q)) * q))); notes.append(f'tones rounded to {q}')
if opt('--lightshift'):
    k = int(opt('--lightshift')); recol(lambda c: (c & ~127) | max(2, min(126, (c & 127) + k))); notes.append(f'lightness {k:+d}')
name = opt('--name', f'npc-{npc_id}-' + ('degraded' if degraded else 'ceiling'))
names = []; same = []
for i, (p, f) in enumerate(zip(parts, files)):
    data = B.encode_model(p, hints=False)
    back = B.decode_model(data); diff = B.model_equal(p, back)
    if diff: raise SystemExit(f'round trip changed {diff}: not written')
    (out / f'{name}-part-{i}.bin').write_bytes(data); names.append(f'{name}-part-{i}.bin'); same.append(data == f.read_bytes())
j = dict(name=d['name'] + (' (degraded: ' + '; '.join(notes) + ')' if degraded else ' (re-encoded)'), models=names,
         recolor=[[int(x) for x in pair.split(':')] for pair in kv.get('recolor', '').split(',') if pair],
         scale=[int(x) for x in kv['scale'].split(',')], ambient=int(kv['ambient']), contrast=int(kv['contrast']), size=int(kv['size']), zbuffer=kv['zbuffer'] == '1', **seqs)
(out / f'{name}.json').write_text(json.dumps(j, indent=1))
print(out / f'{name}.json', '|', '; '.join(notes) or 'ceiling', '|', len(parts), 'parts; byte-identical to the cache:', same)
