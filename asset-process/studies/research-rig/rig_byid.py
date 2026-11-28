"""By-id proof for a NEW creature: the packed cache copy (rigpack.py) drawn through the client's ordinary loaders
against the same creature from LOOSE files.
usage: rig_byid.py PACKED_CACHE EXPORTDIR DEF.npcdef [--out DIR]
For every packed sequence: `npc:ID seq=S` on the packed cache against `def:DEF anim=FILE base=FILE` on the real cache,
every frame, two views; differing pixels are counted. --out keeps the by-id grids as pictures."""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import riglib as R
from PIL import Image
L, B = R.L, R.B
a = sys.argv; packed = pathlib.Path(a[1]).resolve(); exp = pathlib.Path(a[2]).resolve(); d = pathlib.Path(a[3]).resolve()
man = json.loads((packed / 'rigpack.json').read_text()); real = L.CACHE; cs = 300; out = pathlib.Path(a[a.index('--out') + 1]) if '--out' in a else None
def g(cache, entry, **o):
    L.CACHE = str(cache)
    try: return L.grid(entry, **o)
    finally: L.CACHE = real
M = L.merged(f'def:{d}', exp / 'rest.tmp.txt'); (exp / 'rest.tmp.txt').unlink(); zoom = int(2.2 * max(M['V'].max(0) - M['V'].min(0)))
tot = [0, 0, 0]
for sid, s in man['sequences'].items():
    n = B.decode_anim((exp / s['file']).read_bytes())['range'][1]; dsum = drawn = cells = 0
    for yaw in (192, 704):
        o = dict(frames='all', cols=20, zoom=zoom, yaw=yaw, pitch=128, cell=f'{cs}x{cs}')
        A, ha, t = g(packed, f"npc:{man['npc']}", seq=int(sid), **o); assert 'NOT APPLIED' not in t, t[-400:]
        Bm, hb, _ = g(real, f'def:{d}', anim=exp / s['file'], base=exp / man['base_file'], count=n, **o)
        df = L.diff_cells(A, Bm, cs, cs)[:len(ha)]; dsum += sum(x[0] for x in df); drawn += sum(x[1] for x in df); cells += len(ha)
        if out is not None: out.mkdir(parents=True, exist_ok=True); Image.fromarray(A).save(out / f'byid-seq{sid}-yaw{yaw}.png')
    print(f"sequence {sid} ({s['file']}, {n} cycles) by id on the packed cache against loose files: {cells} cells, {drawn} figure pixels, {dsum} differing; distinct poses {len(set(ha.values()))}; vertex hashes equal in the last view {sum(ha[k] == hb.get(k) for k in ha)} of {len(ha)}")
    tot[0] += cells; tot[1] += drawn; tot[2] += dsum
print(f"BY ID, NPC {man['npc']} {man['name']!r} on frame base {man['base']}: {tot[0]} cells = {tot[0] * cs * cs} pixels, {tot[1]} figure pixels, {tot[2]} differing")
