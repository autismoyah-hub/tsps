"""The cross-joint weight profile on joints that ARE blended (ring_census.py's JSON), by kind, side, hop ring and x.
usage: ring_profile.py data/ring-census.json [--json OUT]
A joint counts as blended when at least 2 of its vertices carry both bones with the smaller share over 0.1."""
import sys, json, collections
import numpy as np
D = json.load(open(sys.argv[1])); J = [dict(j, npc=c['npc']) for c in D for j in c['joints'] if 'pts' in j]
kinds = ['hinge', 'ball', 'chain', 'rigid-still']; out = {}
def blended(j):
    p = np.array(j['pts']).reshape(-1, 4); return ((p[:, 1] > .1) & (p[:, 1] < .9)).sum() >= 2
print('share of joints blended, by kind and by the child\'s angle range (degrees):')
for k in kinds:
    r = [j for j in J if j['kind'] == k]; line = []
    for lo, hi in ((0, 20), (20, 45), (45, 90), (90, 999)):
        s = [blended(j) for j in r if lo <= j['angle'] < hi]
        if s: line.append(f'{lo}-{hi}: {np.mean(s):.2f} (n={len(s)})')
    print(f'  {k:12s} all {np.mean([blended(j) for j in r]):.2f} | ' + ', '.join(line))
edges = [-9, -2, -1.5, -1, -.7, -.45, -.25, -.08, .08, .25, .45, .7, 1, 1.5, 2, 9]
for k in kinds:
    r = [j for j in J if j['kind'] == k and blended(j)]; pts = np.array([p for j in r for p in j['pts']]).reshape(-1, 4)
    print(f'\n{k}: {len(r)} blended joints, {len(pts)} vertices. x of ring -1: median {np.median(pts[pts[:, 2] == -1, 0]):+.2f} (25th-75th {np.quantile(pts[pts[:, 2] == -1, 0], .25):+.2f} to {np.quantile(pts[pts[:, 2] == -1, 0], .75):+.2f}); ring +1: {np.median(pts[pts[:, 2] == 1, 0]):+.2f} ({np.quantile(pts[pts[:, 2] == 1, 0], .25):+.2f} to {np.quantile(pts[pts[:, 2] == 1, 0], .75):+.2f}); ring -2: {np.median(pts[pts[:, 2] == -2, 0]):+.2f}; ring +2: {np.median(pts[pts[:, 2] == 2, 0]):+.2f}')
    print('  by x (radii from the joint; - = parent side): child share mean | median (25th-75th) | n')
    prof = []
    for lo, hi in zip(edges[:-1], edges[1:]):
        s = pts[(pts[:, 0] >= lo) & (pts[:, 0] < hi), 1]
        if len(s) >= 8: print(f'    [{lo:+.2f},{hi:+.2f}) {s.mean():.2f} | {np.median(s):.2f} ({np.quantile(s, .25):.2f}-{np.quantile(s, .75):.2f}) | {len(s)}'); prof.append((lo, hi, float(s.mean()), float(np.median(s))))
    print('  by hop ring: cross share (the bone across the joint) mean | median (25th-75th) | share of vertices with any cross weight | commonest bytes')
    ring = {}
    for h in (-4, -3, -2, -1, 1, 2, 3, 4):
        s = pts[pts[:, 2] == h, 1]; s = s if h < 0 else 1 - s
        if len(s) < 8: continue
        b = collections.Counter(int(round(x * 255)) for x in s if 0 < x < 1)
        print(f'    ring {h:+d}: {s.mean():.3f} | {np.median(s):.2f} ({np.quantile(s, .25):.2f}-{np.quantile(s, .75):.2f}) | {np.mean(s > 0):.2f} | {b.most_common(5)} | n={len(s)}'); ring[h] = (float(s.mean()), float(np.median(s)))
    out[k] = dict(joints=len(r), by_x=prof, by_ring=ring)
if '--json' in sys.argv: open(sys.argv[sys.argv.index('--json') + 1], 'w').write(json.dumps(out, indent=1))
