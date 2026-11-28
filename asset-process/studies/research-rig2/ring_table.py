"""The cross share real creatures paint at a place near a joint, over ALL jointed boundaries (blended or not):
by the side the vertex is on, its hop ring, and |x| (radii from the joint plane), within 2 radii of the axis.
By creature softness (share of one-influence vertices): soft under 0.4, medium 0.4 to 0.65, hard over 0.65.
usage: ring_table.py data/ring-census.json [--json OUT] [--skip NPC,NPC]   (--skip: leave creatures out, e.g. the test bosses)"""
import sys, json
import numpy as np
a = sys.argv; D = json.load(open(a[1])); skip = {int(x) for x in a[a.index('--skip') + 1].split(',')} if '--skip' in a else set()
def cls(one): return 'soft' if one < .4 else 'medium' if one < .65 else 'hard'
xb = [0, .1, .25, .45, .7, 1.0, 1.5, 9]; out = {}
for style in ('soft', 'medium', 'hard', 'all'):
    cs = [c for c in D if c['npc'] not in skip and (style == 'all' or cls(c['one']) == style)]
    print(f"\n=== {style}: {len(cs)} creatures")
    for kind in ('hinge', 'ball', 'chain'):
        P = np.array([p for c in cs for j in c['joints'] if j.get('kind') == kind and 'pts' in j for p in j['pts']]).reshape(-1, 4)
        if not len(P): continue
        P = P[P[:, 3] <= 2.0]; side = np.sign(P[:, 2]); cross = np.where(side > 0, 1 - P[:, 1], P[:, 1]); hop = np.abs(P[:, 2]); ax = np.abs(P[:, 0])
        # a vertex whose x disagrees with its side (the boundary is not at the joint) is kept: |x| is still its distance from the joint plane
        print(f'  {kind}: {len(P)} vertices. cross share MEAN / MEDIAN / share over 0.1 / n')
        for sd, nm in ((-1, 'parent side'), (1, 'child side ')):
            for h in (1, 2, 3):
                line = []
                for lo, hi in zip(xb[:-1], xb[1:]):
                    s = cross[(side == sd) & (hop == h) & (ax >= lo) & (ax < hi)]
                    if len(s) >= 15:
                        line.append(f'|x|<{hi}: {s.mean():.2f}/{np.median(s):.2f}/{np.mean(s > .1):.2f}/{len(s)}'); out[f'{style}|{kind}|{sd}|{h}|{hi}'] = (float(s.mean()), float(np.median(s)), int(len(s)))
                print(f'    {nm} hop {h}: ' + '  '.join(line))
if '--json' in a: open(a[a.index('--json') + 1], 'w').write(json.dumps(out, indent=1))
