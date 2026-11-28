"""An ORACLE parts file: which coarse PART each vertex belongs to, read from the real weights (the dominant bone's
subtree), and nothing finer. It answers one question: if an author states part membership correctly at this
granularity, how far does the rule-based weighting get? It is not an authoring tool.

usage: rig_oracle_parts.py WORKDIR OUT.parts.json ROOT[,ROOT..] [--cut A,B]
ROOT = the bone at the top of a part (its whole subtree is the part); every vertex whose real dominant bone is in no
listed subtree forms the last part ("trunk": every bone outside the listed subtrees).
"""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R
B, RM = R.B, R.RM
a = sys.argv; work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); src = work / man['src']
sk = RM.Skeleton(B.decode_base((src / man['base_file']).read_bytes())); m = B.decode_model((src / man['models'][0]).read_bytes())
dom = [max(w, key=lambda bw: bw[1])[0] for w in m['bone_weights']]; roots = [int(x) for x in a[3].split(',')]
sub = lambda b: [b] + [y for c in sk.children[b] for y in sub(c)]
parts = []; taken = set(); used = set()
for r in roots:
    s = set(sub(r)); idx = [i for i, d in enumerate(dom) if d in s]; taken |= set(idx); used |= s
    parts.append(dict(name=f'subtree of bone {r}', select=dict(vertices=idx), bones=dict(subtree=r)))
rest = [i for i in range(len(dom)) if i not in taken]
parts.insert(0, dict(name='trunk (everything else)', select=dict(vertices=rest), bones=dict(list=[b for b in range(sk.n) if b not in used])))
if '--trunk-exact' in a:
    # the trunk assigned BY HAND: each trunk vertex to its real dominant bone (one influence). Stands for an author
    # painting the dense middle of the body bone by bone; the listed subtrees stay automatic.
    for b in sorted({dom[i] for i in rest}):
        parts.append(dict(name=f'trunk bone {b} by hand', select=dict(vertices=[i for i in rest if dom[i] == b]), bones=dict(list=[b]), rigid=True))
cut = [int(x) for x in a[a.index('--cut') + 1].split(',')] if '--cut' in a else []
pathlib.Path(a[2]).write_text(json.dumps(dict(_comment='ORACLE: part membership from the real weights at subtree granularity ' + a[3], cut_segments=cut, parts=parts)))
print(a[2], [(p['name'], len(p['select']['vertices'])) for p in parts])
