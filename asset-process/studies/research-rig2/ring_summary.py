"""Summarise ring_census.py's JSON: how joints are painted, by joint kind.   usage: ring_summary.py data/ring-census.json"""
import sys, json, collections
import numpy as np
D = json.load(open(sys.argv[1])); J = [dict(j, npc=c['npc'], name=c['name']) for c in D for j in c['joints']]
print(f'{len(D)} creatures, {len(J)} bone-to-child boundaries where both bones own vertices')
kinds = ['hinge', 'ball', 'chain', 'rigid-still', 'rigid-separate']
print('\nkind | joints | creatures | child angle range median (10th-90th) | radius median | bone length / radius median')
for k in kinds:
    r = [j for j in J if j['kind'] == k]; ang = np.array([j['angle'] for j in r])
    rad = np.array([j.get('radius', np.nan) for j in r]); ln = np.array([j.get('bone_len', np.nan) for j in r])
    print(f"{k:15s} | {len(r):4d} | {len({j['npc'] for j in r}):2d} | {np.median(ang):5.0f} ({np.quantile(ang, .1):.0f}-{np.quantile(ang, .9):.0f}) | {np.nanmedian(rad):5.1f} | {np.nanmedian(ln / np.maximum(rad, 1e-6)):4.1f}")
print('\nRINGS (hop distance from the boundary; +k on the child bone\'s side, -k on the parent\'s). Per ring: share of joints where at least half the ring carries BOTH bones | mean share of the bone ACROSS the joint (pooled over vertices) | n vertices')
for k in kinds[:4]:
    r = [j for j in J if j['kind'] == k]; print(f'{k}:')
    for ring in (-4, -3, -2, -1, 1, 2, 3, 4):
        rr = [j['rings'][str(ring)] for j in r if str(ring) in j['rings']]
        if not rr: continue
        n = sum(x['n'] for x in rr); oth = sum(x['other'] * x['n'] for x in rr) / n; two = sum(x['two'] * x['n'] for x in rr) / n; third = sum(x['third'] * x['n'] for x in rr) / n
        vals = collections.Counter()
        for x in rr:
            for v, c in x['vals']: vals[v] += c
        print(f"   ring {ring:+d}: joints with the ring shared {np.mean([x['two'] >= .5 for x in rr]):.2f} | vertices carrying both {two:.2f} | mean cross share {oth:.3f} | third-bone share {third:.3f} | {n} vertices | commonest cross bytes {vals.most_common(6)}")
print('\nHOW MANY RINGS are shared (at least half of the ring\'s vertices on both bones), child side + parent side: share of joints')
for k in kinds[:4]:
    r = [j for j in J if j['kind'] == k]; c = collections.Counter(min(j['rings_shared_c'], 3) + min(j['rings_shared_p'], 3) for j in r)
    cs = collections.Counter((min(j['rings_shared_p'], 2), min(j['rings_shared_c'], 2)) for j in r)
    print(f"  {k:12s}: total rings shared " + ', '.join(f'{n}: {c[n] / len(r):.2f}' for n in range(7) if c[n]) + ' | (parent side, child side) ' + ', '.join(f'{p}{q}: {v / len(r):.2f}' for (p, q), v in sorted(cs.items())))
print('\nFALL-OFF with distance along the limb, x = distance from the joint in RADII of the limb at the joint (negative = parent side). Child bone\'s share of the pair, pooled: median, 25th-75th, n')
edges = [-3, -2, -1.5, -1, -.6, -.3, -.1, .1, .3, .6, 1, 1.5, 2, 3]
for k in kinds[:4]:
    pts = np.array([p for j in J if j['kind'] == k for p in j['pts']]).reshape(-1, 4); print(f'  {k}: {len(pts)} vertices within 5 rings')
    line = []
    for lo, hi in zip(edges[:-1], edges[1:]):
        s = pts[(pts[:, 0] >= lo) & (pts[:, 0] < hi), 1]
        if len(s) >= 10: line.append(f'[{lo:+.1f},{hi:+.1f}) {np.median(s):.2f} ({np.quantile(s, .25):.2f}-{np.quantile(s, .75):.2f}) n={len(s)}')
    print('     ' + '\n     '.join(line))
print('\nWHERE the half-and-half vertices sit (child share 0.4 to 0.6): x median, 10th-90th; and the width of the blend per joint: x-range of vertices with child share 0.1 to 0.9')
for k in kinds[:4]:
    pts = np.array([p for j in J if j['kind'] == k for p in j['pts']]).reshape(-1, 4); h = pts[(pts[:, 1] >= .4) & (pts[:, 1] <= .6), 0]
    wid = []; cen = []; none = 0; nj = 0
    for j in J:
        if j['kind'] != k: continue
        nj += 1; p = np.array(j['pts']).reshape(-1, 4); b = p[(p[:, 1] > .1) & (p[:, 1] < .9)]
        if len(b) < 2: none += 1; continue
        wid.append(np.quantile(b[:, 0], .9) - np.quantile(b[:, 0], .1)); cen.append(np.median(b[:, 0]))
    print(f"  {k:12s}: half-and-half vertices {len(h)}, x median {np.median(h) if len(h) else float('nan'):+.2f} ({np.quantile(h, .1) if len(h) else 0:+.2f} to {np.quantile(h, .9) if len(h) else 0:+.2f}); joints with NO blended vertex {none} of {nj} ({none / max(nj, 1):.2f});"
          f" blend centre x median {np.median(cen):+.2f}; blend width in radii: median {np.median(wid):.2f} (25th-75th {np.quantile(wid, .25):.2f}-{np.quantile(wid, .75):.2f})")
print('\nDOES A JOINT THAT BENDS MORE GET MORE BLEND? joints with a boundary, by the child\'s angle range: share with any ring shared | mean cross share on rings +-1 | blend width median')
for lo, hi in ((0, 8), (8, 20), (20, 45), (45, 90), (90, 999)):
    r = [j for j in J if 'rings' in j and lo <= j['angle'] < hi]
    if not r: continue
    any_ = np.mean([(j['rings_shared_c'] + j['rings_shared_p']) > 0 for j in r])
    cross = []
    for j in r:
        x = [j['rings'][s] for s in ('1', '-1') if s in j['rings']]; n = sum(y['n'] for y in x); cross.append(sum(y['other'] * y['n'] for y in x) / max(n, 1))
    print(f'  {lo:3d} to {hi:3d} degrees: {len(r):4d} joints | any ring shared {any_:.2f} | cross share on rings +-1: mean {np.mean(cross):.3f}, median {np.median(cross):.3f}')
print('\nBY CREATURE: share of jointed boundaries (not rigid) with any ring shared; mean cross share on rings +-1')
for c in D:
    r = [j for j in c['joints'] if 'rings' in j and j['kind'] in ('hinge', 'ball', 'chain')]
    if not r: continue
    cross = []
    for j in r:
        x = [j['rings'][s] for s in ('1', '-1') if s in j['rings']]; n = sum(y['n'] for y in x); cross.append(sum(y['other'] * y['n'] for y in x) / max(n, 1))
    print(f"  {c['npc']:6d} {c['name'][:22]:22s} one-influence share {c['one']:.2f}: {len(r):3d} joints, any ring shared {np.mean([(j['rings_shared_c'] + j['rings_shared_p']) > 0 for j in r]):.2f}, cross share {np.mean(cross):.3f}")
