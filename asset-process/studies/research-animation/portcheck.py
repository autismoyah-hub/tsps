"""Check animlib.apply_frame (the Python port of deform.rs) against the client: label centroids of the posed figure
from the port and from asset_anim.exe track= must agree in every frame.

usage: portcheck.py SEQ MESH.txt ENTRY     (MESH.txt = asset_anim dump= of the same ENTRY)
"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import animlib as A, author, track
sid = int(sys.argv[1])
verts, labels = author.load_mesh(sys.argv[2])
s, pos = track.run(f'seq={sid}', sys.argv[3], labels='27,28,50,1,45')
_, bid, base, frames = A.sequence_frames(sid)
worst = 0.0; n = 0
for k, f in enumerate(frames):
    v = A.apply_frame([list(x) for x in verts], labels, f, base)
    for lab, p in pos[k].items():
        idx = [i for i, l in enumerate(labels) if l == lab]
        c = [sum(v[i][a] for i in idx) / len(idx) for a in range(3)]
        mine = (c[0], -c[1], c[2])
        d = max(abs(mine[a] - p[a]) for a in range(3))
        worst = max(worst, d); n += 1
print(f'seq {sid}: {len(frames)} frames, {n} label centroids compared, largest difference {worst:.2f} units (the client prints one decimal)')
