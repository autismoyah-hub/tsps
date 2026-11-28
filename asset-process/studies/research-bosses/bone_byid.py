"""By-id proof: the packed copy (bonepack.py) drawn through the client's ordinary loaders against the real cache.

usage: bone_byid.py PACKED_CACHE [--every K]
For every real keyframe sequence S of the real NPC and its packed copy S' : asset_bone grid npc:REAL seq=S on the
real cache against npc:OURS seq=S' on the packed cache, every frame, two views; differing pixels are counted.
Authored sequences: by id on the packed cache against their loose keyframe file on the real cache.
"""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import bonelib as L
a = sys.argv; packed = pathlib.Path(a[1]).resolve(); every = int(a[a.index('--every') + 1]) if '--every' in a else 1
info = json.loads((packed / 'bonepack.json').read_text()); real = L.CACHE; cs = 300
def g(cache, entry, **o):
    L.CACHE = str(cache)
    try: return L.grid(entry, **o)
    finally: L.CACHE = real
tot = [0, 0, 0, 0]
frames = dict(L.seqs_on_base(L.base_of_seq(int(next(iter(info['sequences']))))))
for s, s2 in info['sequences'].items():
    s = int(s); n = frames[s]; dsum = drawn = cells = 0
    for yaw in (192, 704):
        o = dict(frames=f'every:{every}' if every > 1 else 'all', cols=20, zoom=1250, yaw=yaw, pitch=128, cell=f'{cs}x{cs}')
        A, _, _ = g(real, f"npc:{info['real_npc']}", seq=s, **o); B, _, t = g(packed, f"npc:{info['npc']}", seq=s2, **o)
        assert 'NOT APPLIED' not in t, t[-400:]
        d = L.diff_cells(A, B, cs, cs); k = -(-n // every); dsum += sum(x[0] for x in d); drawn += sum(x[1] for x in d); cells += k
    print(f'real seq {s} / packed seq {s2}: {n} frames, {cells} cells, {drawn} figure pixels, {dsum} differing', flush=True)
    tot[0] += cells; tot[1] += drawn; tot[2] += dsum; tot[3] += n
print(f'BY ID, renumbered creature (NPC {info["npc"]}, base {info["base"]}): {tot[3]} frames, {tot[0]} cells = {tot[0] * cs * cs} pixels, {tot[1]} figure pixels, {tot[2]} differing')
for sid, asset in info.get('authored', {}).items():
    for d in pathlib.Path(__file__).parent.glob('authored/*/seq.dat'):
        pass
for sid, loose in [x.split('=') for x in a[2:] if '=' in x and x.split('=')[0].isdigit()]:
    o = dict(frames='all', cols=20, zoom=1250, yaw=192, pitch=128, cell=f'{cs}x{cs}')
    A, _, t = g(packed, f"npc:{info['real_npc']}", seq=int(sid), **o); B, _, _ = g(real, f"npc:{info['real_npc']}", anim=loose, **o)
    d = L.diff_cells(A, B, cs, cs); print(f'authored sequence {sid} by id (packed cache) against loose file {loose}: {len(d)} cells, {sum(x[1] for x in d)} figure pixels, {sum(x[0] for x in d)} differing')
