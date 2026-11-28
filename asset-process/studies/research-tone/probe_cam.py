"""Probe the harness camera: large cells, pitch range, and posed-vertex dumps."""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np, time, subprocess
from PIL import Image
import tl
from tl import L
npc = 7541; seqs = L.sequences(npc); print('sequences', seqs, {k: L.seq_frames(v) for k, v in seqs.items()})
out = tl.RENDER / 'research-tone-work'; out.mkdir(exist_ok=True)
for cell in ((380, 380), (1000, 1000)):
    cam = tl.camera(npc, cell)
    for pitch in (128, 400, 1920, 0):
        cam['pitch'] = pitch; t = time.time()
        try:
            cells, text = tl.draw([L.entry_real(npc, seqs)], ['bind', 'stand:0'], cam, 192, out / f'cam-{cell[0]}-p{pitch}.png')
            print(cell, 'zoom', cam['zoom'], 'pitch', pitch, 'drawn px', [int(tl.mask(c[0]).sum()) for c in cells], 'seconds', round(time.time() - t, 1), text.strip()[:200].replace('\n', ' | '))
        except SystemExit as e: print(cell, pitch, 'FAILED', str(e)[:300])
import light_npc
p = tl.WORK / 'probe-light.txt'
r = subprocess.run([light_npc.EXE, light_npc.CACHE, f'npc:{npc}', str(p), f'seq={seqs["stand"]}:0'], capture_output=True, text=True); print(r.stdout[:300], r.stderr[:300])
V, Lb, F, amb, con, posed, relit = light_npc.read(p); m, kv = tl.real_mesh(npc)
print('vertices', len(V), len(m['vertices']), 'same order:', [tuple(v) for v in V] == [tuple(v) for v in m['vertices']], 'faces same:', [f[:3] for f in F] == [tuple(f) for f in m['faces']], 'posed', len(posed), 'amb con', amb, con)
