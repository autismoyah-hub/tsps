"""What sets the WIDTH of a painted joint, and where it stops sideways (ring_census.py's JSON).  usage: ring_fit.py data/ring-census.json"""
import sys, json
import numpy as np
D = json.load(open(sys.argv[1])); J = [dict(j, npc=c['npc']) for c in D for j in c['joints'] if 'pts' in j]
def blended(j):
    p = np.array(j['pts']).reshape(-1, 4); return ((p[:, 1] > .1) & (p[:, 1] < .9)).sum() >= 2
kinds = ['hinge', 'ball', 'chain']
print('1. Reach of the blend on each side, per blended joint: the farthest vertex (90th centile) still carrying over 0.1 of the bone across,\n   in RADII and as a share of that side\'s BONE LENGTH. Which unit is the steadier (interquartile range / median)?')
for k in kinds:
    rb, ra, lb, la = [], [], [], []
    for j in J:
        if j['kind'] != k or not blended(j): continue
        p = np.array(j['pts']).reshape(-1, 4); rad = j['radius']
        b = p[(p[:, 0] < 0) & (p[:, 1] > .1)]; a_ = p[(p[:, 0] > 0) & (p[:, 1] < .9)]
        if len(b) >= 2: x = -np.quantile(b[:, 0], .1); rb.append(x); lb.append(x * rad / max(j['bone_len'], 1))
        if len(a_) >= 2: x = np.quantile(a_[:, 0], .9); ra.append(x); la.append(x * rad / max(j['len_c'], 1))
    q = lambda v: f'{np.median(v):.2f} ({np.quantile(v, .25):.2f}-{np.quantile(v, .75):.2f}; spread {(np.quantile(v, .75) - np.quantile(v, .25)) / np.median(v):.2f})'
    print(f'  {k:6s} before the joint: radii {q(rb)} | share of the parent bone {q(lb)} || after: radii {q(ra)} | share of the child bone {q(la)}')
print('\n2. Sideways: vertices within 0.35 radii of the joint plane, by their distance from the limb axis (radii): share carrying both bones over 0.1 | n   (blended joints only)')
for k in kinds:
    pts = np.array([p for j in J if j['kind'] == k and blended(j) for p in j['pts']]).reshape(-1, 4); s = pts[np.abs(pts[:, 0]) < .35]; line = []
    for lo, hi in ((0, .6), (.6, 1.4), (1.4, 2.5), (2.5, 4), (4, 99)):
        t = s[(s[:, 3] >= lo) & (s[:, 3] < hi)]
        if len(t) > 10: line.append(f'{lo}-{hi}: {np.mean((t[:, 1] > .1) & (t[:, 1] < .9)):.2f} (n={len(t)})')
    print(f'  {k:6s} ' + ', '.join(line))
print('\n3. Is a joint blended at all? share of joints blended by: bone length / radius of the shorter side | vertices owned by the smaller side | boundary vertices')
for k in kinds:
    r = [j for j in J if j['kind'] == k]
    def by(key, bins):
        out = []
        for lo, hi in bins:
            s = [blended(j) for j in r if lo <= key(j) < hi]
            if len(s) >= 8: out.append(f'{lo}-{hi}: {np.mean(s):.2f} (n={len(s)})')
        return ', '.join(out)
    print(f'  {k:6s} shorter bone / radius: ' + by(lambda j: min(j['bone_len'], j['len_c']) / max(j['radius'], 1e-6), ((0, .5), (.5, 1), (1, 2), (2, 4), (4, 99))))
    print(f'         smaller side\'s vertices: ' + by(lambda j: min(j['verts_c'], j['verts_p']), ((1, 3), (3, 6), (6, 12), (12, 30), (30, 999))))
    print(f'         boundary vertices: ' + by(lambda j: j['boundary'], ((3, 6), (6, 10), (10, 16), (16, 30), (30, 999))))
print('\n4. The boundary between the two one-bone regions against the joint: median x of the boundary vertices (radii), blended joints: median, 10th-90th')
for k in kinds:
    v = np.array([j['bnd_x_med'] for j in J if j['kind'] == k and blended(j)]); print(f'  {k:6s} {np.median(v):+.2f} ({np.quantile(v, .1):+.2f} to {np.quantile(v, .9):+.2f})')
