"""Find arm values that put a hand at a point (a small inverse-kinematics helper for key poses).

usage: reach.py MESH.txt r|l X H Z [--rest SEQ:FRAME] [--pose JSON]
   MESH.txt   asset_anim dump= of the figure;  X H Z = target in worn space (x negative = the character's right,
   H height above the feet, z negative = front).  --pose gives other channels of the key first, e.g.
   '{"torso": [3, 0, 0]}', because the torso moves the shoulder. Prints shoulder and elbow values for the key.
The search is a coordinate descent on the six stored numbers with a small cost on distance from the rest values,
posing the mesh with animlib.apply_frame (the port of the client's deform.rs).
"""
import sys, json, math, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import animlib as A, author, rigs
argv = sys.argv
verts, labels = author.load_mesh(argv[1])
side = argv[2]; target = [float(argv[3]), -float(argv[4]), float(argv[5])]
rs, rf = (argv[argv.index('--rest') + 1].split(':') if '--rest' in argv else ('808', '0'))
_, bid, base, frames = A.sequence_frames(int(rs))
f = dict(base=bid, groups={g: list(v) for g, v in frames[int(rf)]['groups'].items()})
if '--pose' in argv:
    for k, v in json.loads(argv[argv.index('--pose') + 1]).items():
        f['groups'][author.group_of(bid, k)] = list(v)
GS, GE = rigs.PLAYER[side + '_shoulder'][0], rigs.PLAYER[side + '_elbow'][0]
lab = 27 if side == 'r' else 28
idx = [i for i, l in enumerate(labels) if l == lab]
ref = [list(f['groups'].get(GS, [0, 0, 0])), list(f['groups'].get(GE, [0, 0, 0]))]


def cost(c):
    f['groups'][GS], f['groups'][GE] = list(c[0]), list(c[1])
    v = A.apply_frame([list(x) for x in verts], labels, f, base)
    cen = [sum(v[i][a] for i in idx) / len(idx) for a in range(3)]
    d = math.dist(cen, target)
    return d + 0.02 * sum(abs(c[j][i] - ref[j][i]) for j in range(2) for i in range(3)), d, cen


cur = [list(ref[0]), list(ref[1])]; best = cost(cur)
for step in (16, 8, 4, 2, 1):
    improved = True
    while improved:
        improved = False
        for j in range(2):
            for i in range(3):
                for sgn in (1, -1):
                    t = [list(cur[0]), list(cur[1])]; t[j][i] += sgn * step
                    if abs(t[j][i]) > 110: continue
                    c = cost(t)
                    if c[0] < best[0] - 1e-6: cur, best, improved = t, c, True
print(f'"{side}_shoulder": {cur[0]}, "{side}_elbow": {cur[1]}   hand at x {best[2][0]:.1f} h {-best[2][1]:.1f} z {best[2][2]:.1f}, {best[1]:.1f} units from the target')
