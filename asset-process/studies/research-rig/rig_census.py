"""Bone counts and weight habits of every bone-weighted NPC in the cache, by creature size (tiles).
usage: rig_census.py [--json OUT]
For each NPC whose stand is a keyframe sequence: the frame base's bone count (decoded here), the vertices and the
influences per vertex of its model files (decoded here). One row per distinct (base, model list). Read-only."""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R
L, B = R.L, R.B
npcs = L.census('npcs'); seqs = L.census('sequences'); seen = {}; bones = {}; models = {}
for i, d in npcs.items():
    st = d['anims'][0]
    if st < 0 or st not in seqs or seqs[st]['kind'] != 'cached': continue
    key = (seqs[st]['skeleton'], tuple(d['models']))
    if key in seen: continue
    bid = key[0]
    try:
        if bid not in bones: bones[bid] = len(B.decode_base(L.base_bytes(bid))['bones'])
        inf = collections.Counter(); nv = 0
        for mid in d['models']:
            if mid not in models:
                m = B.decode_model(L.model_bytes(mid)); models[mid] = (len(m['vertices']), collections.Counter(len(w) for w in (m['bone_weights'] or [])))
            nv += models[mid][0]; inf.update(models[mid][1])
    except Exception as e:
        continue
    if not inf: continue
    tot = sum(inf.values()); seen[key] = dict(npc=i, name=d['name'], size=d['size'], bones=bones[bid], vertices=nv, one=inf[1] / tot, two=inf[2] / tot, more=sum(v for k, v in inf.items() if k > 2) / tot)
rows = list(seen.values()); print(f'{len(rows)} bone-weighted creatures (distinct base and model list) with a keyframe stand')
print('tiles | n | bones 10th / median / 90th | vertices median | vertices per bone median | share of vertices with one influence, median | creatures 100% one influence')
for lo, hi in ((1, 1), (2, 2), (3, 3), (4, 5), (6, 99)):
    r = [x for x in rows if lo <= x['size'] <= hi]
    if not r: continue
    b = np.array([x['bones'] for x in r]); v = np.array([x['vertices'] for x in r]); o = np.array([x['one'] for x in r])
    print(f"{lo if lo == hi else f'{lo}-{hi}' if hi < 99 else f'{lo}+'} | {len(r)} | {np.quantile(b, .1):.0f} / {np.median(b):.0f} / {np.quantile(b, .9):.0f} | {np.median(v):.0f} | {np.median(v / b):.1f} | {np.median(o):.2f} | {int((o == 1).sum())}")
o = np.array([x['one'] for x in rows]); print(f"all: one influence median {np.median(o):.2f}, 10th centile {np.quantile(o, .1):.2f}; creatures wholly rigid (every vertex one bone): {int((o == 1).sum())} of {len(rows)}; with any vertex of 3 or more influences: {sum(1 for x in rows if x['more'] > 0)}")
if '--json' in sys.argv: pathlib.Path(sys.argv[sys.argv.index('--json') + 1]).write_text(json.dumps(rows, indent=1))
