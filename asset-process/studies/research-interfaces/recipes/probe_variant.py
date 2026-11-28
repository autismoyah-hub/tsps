"""probe_variant.py FAMILY SPRITE [SPRITE...]: split a family's instances by the relative rectangle forms of the
given piece sprites; list groups, box sizes and what else the instance holds, per variant."""
import sys, collections
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import common as K
import census_frames as F

name = sys.argv[1]
keys = [int(s) for s in sys.argv[2:]]
fam = F.FAMILIES[name]
V = collections.defaultdict(lambda: dict(n=0, groups=collections.Counter(), sizes=collections.Counter(), extra=collections.Counter()))
for g, rows, by in K.all_states():
    for inst in F.find_instances(rows, by, fam):
        W, H = inst['box'][2:]
        sig = []
        for s in keys:
            rs = [F.rel(inst, r) for r in inst['members'] if r['sprites'][0] == s]
            for x, y, w, h in rs:
                sig.append((s, x if x < W / 2 else f'W{x - W:+d}', y if y < H / 2 else f'H{y - H:+d}',
                            w if w < W / 2 else f'W{w - W:+d}', h if h < H / 2 else f'H{h - H:+d}'))
        e = V[tuple(sig)]
        e['n'] += 1; e['groups'][g] += 1; e['sizes'][(W, H)] += 1
        for r in inst['members']:
            s = r['sprites'][0]
            if s not in keys and s not in fam['corners']:
                e['extra'][(s, tuple(F.rel(inst, r)) if s in F.BACKS else '')] += 1
for sig, e in sorted(V.items(), key=lambda kv: -kv[1]['n']):
    print(e['n'], sig)
    print('    groups', dict(e['groups']), ' sizes', e['sizes'].most_common(5))
    print('    other pieces', e['extra'].most_common(8))
