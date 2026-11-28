"""Which joints are blended at all? Shape of the boundary against the answer (ring_census.py's JSON).  usage: ring_fit2.py JSON"""
import sys, json
import numpy as np
D = json.load(open(sys.argv[1])); J = [dict(j, npc=c['npc'], one=c['one']) for c in D for j in c['joints'] if 'pts' in j and j['kind'] in ('hinge', 'ball', 'chain')]
def blended(j):
    p = np.array(j['pts']).reshape(-1, 4); return ((p[:, 1] > .1) & (p[:, 1] < .9)).sum() >= 2
def table(name, key, bins, rows=J):
    out = []
    for lo, hi in bins:
        s = [blended(j) for j in rows if lo <= key(j) < hi]
        if len(s) >= 8: out.append(f'{lo}-{hi}: {np.mean(s):.2f} (n={len(s)})')
    print(f'  {name}: ' + ', '.join(out))
print(f'{len(J)} jointed boundaries (hinge, ball, chain); blended {np.mean([blended(j) for j in J]):.2f}')
table('largest angular gap of the boundary round the axis, degrees (a closed ring: under 120)', lambda j: j['gap'], ((0, 60), (60, 120), (120, 180), (180, 240), (240, 361)))
table('boundary centre off the axis, radii', lambda j: j['off_axis'], ((0, .2), (.2, .5), (.5, .8), (.8, 1.01)))
table('child part flatness (third extent / first)', lambda j: j['flat_c'], ((0, .05), (.05, .15), (.15, .3), (.3, 1.01)))
table('boundary vertices', lambda j: j['boundary'], ((3, 6), (6, 10), (10, 16), (16, 30), (30, 999)))
table('angle range of the child, degrees', lambda j: j['angle'], ((8, 20), (20, 45), (45, 90), (90, 999)))
table('creature\'s share of one-influence vertices', lambda j: j['one'], ((0, .2), (.2, .4), (.4, .6), (.6, .8), (.8, 1.01)))
print('closed ring (gap under 120) AND boundary of 6 or more:')
r = [j for j in J if j['gap'] < 120 and j['boundary'] >= 6]; print(f'  {len(r)} joints, blended {np.mean([blended(j) for j in r]):.2f}'); table('    by creature one-influence share', lambda j: j['one'], ((0, .2), (.2, .4), (.4, .6), (.6, .8), (.8, 1.01)), r)
r = [j for j in J if not (j['gap'] < 120 and j['boundary'] >= 6)]; print(f'  the rest: {len(r)} joints, blended {np.mean([blended(j) for j in r]):.2f}')
