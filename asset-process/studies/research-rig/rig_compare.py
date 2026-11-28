"""An exported creature (rig_export.py) against the real one, drawn by the client's own code (asset_bone.exe).

usage: rig_compare.py WORKDIR EXPORTDIR [--seqs A,B,C] [--every K] [--views 192,704] [--cell 300] [--json OUT] [--keep DIR]
WORKDIR is rig_fetch.py's (manifest.json, src/). For every exported action that came from a real sequence:
  bytes   model, base and keyframe files against the cache's own;
  drawn   every frame (or every K-th) of the sequence, two views: `npc:ID seq=S` through the cache loaders against
          `def:` our model files with anim= our keyframe file and base= our frame base as LOOSE files; differing
          pixels, figure pixels, cells whose posed vertices are bit-equal (hash of the float bits), and the worst cell.
--keep DIR writes real | ours | difference strips of the worst cell of each sequence.
"""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import riglib as R
import numpy as np
from PIL import Image
L = R.L
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
work = pathlib.Path(a[1]).resolve(); exp = pathlib.Path(a[2]).resolve(); man = json.loads((work / 'manifest.json').read_text()); rep = json.loads((exp / 'export.json').read_text())
every = int(opt('--every', 1)); views = [int(v) for v in opt('--views', '192,704').split(',')]; cs = int(opt('--cell', 300)); npc = man['npc']
want = {int(x) for x in opt('--seqs').split(',')} if opt('--seqs') else None
src = work / man['src']
same_models = [(exp / m['file']).read_bytes() == (src / f).read_bytes() for m, f in zip(rep['models'], man['models'])]
same_base = (exp / rep['base_file']).read_bytes() == (src / man['base_file']).read_bytes()
kv = dict(l.split('=', 1) for l in (src / man['npcdef']).read_text().splitlines() if '=' in l and not l.startswith('#'))
odef = L.write_npcdef(exp / 'ours.npcdef', man['name'] + ' (through Blender)', [exp / m['file'] for m in rep['models']], kv)
print(f"{man['name']} {npc}: model files byte-identical {same_models}; frame base byte-identical {same_base}")
M = L.merged(f'npc:{npc}', exp / 'rest.tmp.txt'); (exp / 'rest.tmp.txt').unlink(); sc = man['scale']
ext = (M['V'].max(0) - M['V'].min(0)) * [sc[0] / 128, sc[1] / 128, sc[0] / 128]; zoom = max(300, int(1.75 * max(ext[0], ext[2], ext[1] * 1.1)))
tot = dict(frames=0, cells=0, drawn=0, differ=0, hash=0, bytes=0, files=0); rows = []
keep = pathlib.Path(opt('--keep')) if opt('--keep') else None
for x in rep['actions']:
    sid = x.get('seq')
    if sid is None or (want and sid not in want): continue
    n = x['length']; same = (exp / x['file']).read_bytes() == (src / x['source']).read_bytes(); row = dict(seq=sid, frames=n, byte_identical=same, verbatim_bones=x['verbatim_bones'], rewritten_bones=x['rewritten_bones'], views={})
    worst = (0, None)
    for yaw in views:
        o = dict(frames=f'every:{every}' if every > 1 else 'all', cols=20, zoom=zoom, yaw=yaw, pitch=128, cell=f'{cs}x{cs}')
        real, hr, _ = L.grid(f'npc:{npc}', seq=sid, **o)
        ours, ho, _ = L.grid(f'def:{odef}', anim=exp / x['file'], base=exp / rep['base_file'], count=n, **o)
        d = L.diff_cells(real, ours, cs, cs)[:len(hr)]
        row['views'][yaw] = dict(cells=len(hr), drawn=sum(v[1] for v in d), differ=sum(v[0] for v in d), cells_differing=sum(1 for v in d if v[0]), worst_cell=max(v[0] for v in d), hash_equal=sum(hr[k] == ho.get(k) for k in hr))
        k = int(np.argmax([v[0] for v in d]))
        if d[k][0] > worst[0]:
            r_, c_ = divmod(k, 20); A = real[r_ * cs:(r_ + 1) * cs, c_ * cs:(c_ + 1) * cs]; Bc = ours[r_ * cs:(r_ + 1) * cs, c_ * cs:(c_ + 1) * cs]
            D = A.copy(); D[(A != Bc).any(2)] = (255, 0, 255); worst = (d[k][0], np.concatenate([A, Bc, D], 1), k * every, yaw)
    v = row['views']; cells = sum(q['cells'] for q in v.values()); drawn = sum(q['drawn'] for q in v.values()); dif = sum(q['differ'] for q in v.values())
    row['worst_cell_pixels'] = worst[0]
    tot['frames'] += v[views[0]]['cells']; tot['cells'] += cells; tot['drawn'] += drawn; tot['differ'] += dif; tot['hash'] += sum(q['hash_equal'] for q in v.values()); tot['bytes'] += same; tot['files'] += 1
    rows.append(row)
    print(f"seq {sid} ({n} cycles, {v[views[0]]['cells']} drawn x {len(views)} views): keyframe file byte-identical {same} ({x['verbatim_bones']} bones verbatim, {x['rewritten_bones']} re-derived); figure pixels {drawn}; differing {dif}"
          f" in {sum(q['cells_differing'] for q in v.values())} cells (worst cell {worst[0]}{'' if worst[1] is None else f', frame {worst[2]} yaw {worst[3]}'}); posed vertices bit-equal in {sum(q['hash_equal'] for q in v.values())} of {cells} cells", flush=True)
    if keep is not None and worst[1] is not None:
        keep.mkdir(parents=True, exist_ok=True); Image.fromarray(worst[1]).save(keep / f'worst-seq{sid}-frame{worst[2]}-yaw{worst[3]}.png')
print(f"TOTAL: {tot['files']} sequences ({tot['bytes']} keyframe files byte-identical), {tot['frames']} frames, {tot['cells']} cells of {cs}x{cs} = {tot['cells'] * cs * cs} pixels, {tot['drawn']} figure pixels, {tot['differ']} differing"
      f" ({100 * tot['differ'] / max(tot['drawn'], 1):.5f}% of figure pixels); posed vertices bit-equal in {tot['hash']} of {tot['cells']} cells")
if opt('--json'): pathlib.Path(opt('--json')).write_text(json.dumps(dict(npc=npc, models_identical=same_models, base_identical=same_base, total=tot, rows=rows, zoom=zoom, cell=cs, every=every), indent=1))
