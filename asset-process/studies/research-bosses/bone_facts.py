"""Facts for SKELETAL.md, measured over the cache with bonecodec (read only).   usage: bone_facts.py [SEQ_ID]"""
import sys, json, pathlib, collections, math
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import bonelib as L, bonecodec as B
import numpy as np
c = L.cache()
sid = int(sys.argv[1]) if len(sys.argv) > 1 else 13926
aid, rng = L.anim_id(sid); a = B.decode_anim(L.anim_bytes(aid)); bid = a['base']; base = B.decode_base(L.base_bytes(bid))
print(f'--- sequence {sid}: keyframe file {aid >> 16}:{aid & 65535}, range {rng}, base {bid}; file range {a["range"]} version {a["version"]} pose {a["pose"]}')
s = L.seq_def(sid); print('sequence definition:', {k: v for k, v in s.items() if v not in (None, [], False) and k != 'ops'}, 'opcodes', s['ops'])
print('base: group types', dict(collections.Counter(base['types'])), 'bones', len(base['bones']), 'poses', base['poses'])
kinds = collections.Counter((ch['kind'], ch['channel'], ch['curve']['byte0']) for ch in a['channels']); print('channels (kind, channel, byte0): count', dict(sorted(kinds.items())))
bones_with = collections.Counter(ch['target'] for ch in a['channels'] if ch['kind'] == 1); print('bones with channels', len(bones_with), 'of', len(base['bones']), '; channels per bone', collections.Counter(bones_with.values()))
steps = collections.Counter(); tang = collections.Counter(); const = 0; nk = []
for ch in a['channels']:
    k = ch['curve']['keys']; nk.append(len(k))
    if len({x[1] for x in k}) == 1: const += 1
    for i in range(len(k) - 1):
        steps[k[i + 1][0] - k[i][0]] += 1
        o = (k[i][4], k[i][5]); tang['hold' if o == (0, 0) else 'next' if o[0] > 3e38 else 'spline'] += 1
print('curves', len(a['channels']), 'constant-valued', const, 'keys per curve', collections.Counter(nk).most_common(6), 'first key times', collections.Counter(ch['curve']['keys'][0][0] for ch in a['channels']).most_common(4),
      'last key times', collections.Counter(ch['curve']['keys'][-1][0] for ch in a['channels']).most_common(4))
print('key spacing (cycles): count', steps.most_common(8)); print('segment kinds', dict(tang), 'weighted curves', sum(1 for ch in a['channels'] if ch['curve']['weighted']))
for ch in a['channels']:
    k = ch['curve']['keys']
    if ch['kind'] == 1 and len(k) > 4 and any(x[4] not in (0.0,) and x[4] < 3e38 for x in k[:-1]):
        print('a spline curve: bone', ch['target'], B.CHANNEL_NAMES[ch['channel']], 'weighted', ch['curve']['weighted'], 'keys', [[x[0]] + [round(v, 4) for v in x[1:]] for x in k[:4]]); break
for ch in a['channels']:
    if ch['kind'] != 1:
        print('a non-bone channel: kind', ch['kind'], 'target group', ch['target'], 'of type', base['types'][ch['target']], 'labels', base['labels'][ch['target']][:8], 'channel', ch['channel'], 'keys', [[x[0], round(x[1], 3)] for x in ch['curve']['keys'][:5]]); break
nb = collections.Counter(base['types'][ch['target']] for ch in a['channels'] if ch['kind'] != 1); print('non-bone channel targets by group type', dict(nb))
# ---- the bone section
print('--- bones of base', bid)
depth = {}
def dep(i):
    if i not in depth: p = base['bones'][i]['parent']; depth[i] = 0 if p < 0 else dep(p) + 1
    return depth[i]
ds = [dep(i) for i in range(len(base['bones']))]; print('roots', [i for i, b in enumerate(base['bones']) if b['parent'] < 0], 'deepest chain', max(ds) + 1, 'parent after child (index order)', sum(1 for i, b in enumerate(base['bones']) if b['parent'] > i))
def eul(m):
    M = np.array(m, np.float64).reshape(4, 4).T; inv = np.linalg.inv(M).T.reshape(16)
    x = -math.asin(max(-1, min(1, inv[6])))
    if abs(math.cos(x)) > 0.005: y = math.atan2(inv[2], inv[10]); z = math.atan2(inv[4], inv[5])
    else:
        y = math.atan2(inv[1], inv[0]); y = y if inv[6] < 0 else -y; z = 0.0
    return x, y, z
ex_eq = 0; sc = collections.Counter()
for i, b in enumerate(base['bones'][:6]):
    m = b['matrices'][0]; e = eul(m); s3 = [round(math.sqrt(sum(m[k * 4 + j] ** 2 for j in range(3))), 3) for k in range(3)]
    print(f'bone {i} parent {b["parent"]} local translation {[round(v, 2) for v in m[12:15]]} rest rotation (client formula) {[round(v, 3) for v in e]} scale {s3} extra {[round(v, 3) for v in b["extra"][0]]} m15 {m[15]}')
for b in base['bones']:
    m = b['matrices'][0]; e = eul(m)
    ex_eq += all(abs(x - y) < 1e-3 or abs(abs(x - y) - 2 * math.pi) < 1e-3 for x, y in zip(e, b['extra'][0]))
    sc[tuple(round(math.sqrt(sum(m[k * 4 + j] ** 2 for j in range(3))), 2) for k in range(3))] += 1
print('bones whose 3 extra floats equal the rest rotation the client derives:', ex_eq, 'of', len(base['bones']), '; rest scales', sc.most_common(3))
# ---- do the first keys of a stand equal the rest values?
loc = {}
for i, b in enumerate(base['bones']):
    m = b['matrices'][0]; loc[i] = list(eul(m)) + list(m[12:15]) + [math.sqrt(sum(m[k * 4 + j] ** 2 for j in range(3))) for k in range(3)]
dmax = collections.defaultdict(float)
for ch in a['channels']:
    if ch['kind'] == 1:
        comp = ch['channel'] - 1; grp = 'rot tran scale'.split()[comp // 3]
        for k in ch['curve']['keys']: dmax[grp] = max(dmax[grp], abs(k[1] - loc[ch['target']][comp]))
print('largest departure of any key from the rest value, by component kind:', {k: round(v, 3) for k, v in dmax.items()})
# ---- weights over all weighted models
if '--weights' in sys.argv:
    w = json.loads((L.HERE / 'data' / 'census-models.json').read_text())['weighted']
    infl = collections.Counter(); sums = collections.Counter(); zero = 0; both = 0; order = 0; nvt = 0; maxbone = 0; bonesused = []
    for mid in w:
        m = B.decode_model(L.model_bytes(mid)); both += m['vertex_skins'] is not None; bu = set()
        for ws in m['bone_weights']:
            infl[len(ws)] += 1; sums[sum(x for _, x in ws)] += 1; zero += any(x == 0 for _, x in ws); nvt += 1
            order += [x for _, x in ws] != sorted((x for _, x in ws), reverse=True); bu.update(b for b, _ in ws)
        maxbone = max(maxbone, max(bu) if bu else 0); bonesused.append(len(bu))
    print('--- weights over', len(w), 'weighted models,', nvt, 'vertices')
    print('influences per vertex', dict(sorted(infl.items()))); print('weight sums', sums.most_common(6)); print('vertices with a zero weight', zero, '; not sorted heaviest first', order, '; models that ALSO carry vertex labels', both, '; highest bone index', maxbone)
    print('bones used per model: min', min(bonesused), 'median', sorted(bonesused)[len(bonesused) // 2], 'max', max(bonesused))
