"""probe_family.py FAMILY: per instance of a frame family: group, box, the building script, and each piece's
rectangle relative to the box (to split variants)."""
import sys, collections
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import common as K
import census_frames as F

name = sys.argv[1]
fam = F.FAMILIES[name]
var = collections.Counter()
for g, rows, by in K.all_states():
    for inst in F.find_instances(rows, by, fam):
        a = inst['members'][0]
        scr = K.host_script(g, by, by[inst['parent']]) if inst['parent'] in by else None
        pieces = tuple((r['sprites'][0], tuple(F.rel(inst, r))) for r in inst['members'] if r['sprites'][0] not in fam['corners'])
        W, H = inst['box'][2:]
        sig = []
        for s, (x, y, w, h) in pieces:
            sig.append((s, x if x < W / 2 else f'W{x - W:+d}', y if y < H / 2 else f'H{y - H:+d}',
                        w if w < W / 2 else f'W{w - W:+d}', h if h < H / 2 else f'H{h - H:+d}'))
        var[(scr, tuple(sorted(sig, key=str)))] += 1
        if len(sys.argv) > 2:
            print(g, inst['box'], 'script', scr, sig)
for (scr, sig), n in var.most_common(40):
    print(n, 'script', scr, sig)
