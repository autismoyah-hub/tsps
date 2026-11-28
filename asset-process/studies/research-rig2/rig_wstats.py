"""How a real skeletal creature is WEIGHTED: the facts a weight tool has to reproduce.

usage: rig_wstats.py WORKDIR [--json OUT] [--bones]
WORKDIR is rig_fetch.py's. Prints: influences per vertex; the weight values in use; which bones carry weight and which
are only joints of the tree; whether a vertex's bones are neighbours in the tree; how the mesh's connected shells map
to bones (a rigid shell = every vertex on one bone at 255); how far a vertex sits from its bone's segment against the
nearest segment of any bone (is "nearest bone" the real rule?). --bones lists every bone.
"""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R, wlib
B, RM = R.B, R.RM
a = sys.argv; work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); src = work / man['src']
sk = RM.Skeleton(B.decode_base((src / man['base_file']).read_bytes()))
out = dict(npc=man['npc'], name=man['name'], bones=sk.n, parts=[])
for f in man['models']:
    m = B.decode_model((src / f).read_bytes()); W = m['bone_weights']
    if W is None: continue
    V = np.array(m['vertices'], float); F = np.array(m['faces']); nv = len(V)
    inf = collections.Counter(len(w) for w in W); dom = np.array([max(w, key=lambda bw: bw[1])[0] if w else -1 for w in W])
    used = sorted({b for w in W for b, _ in w}); domset = sorted(set(dom.tolist()))
    vals = collections.Counter(x for w in W for _, x in w if len(w) > 1)
    # tree relation of the pairs of a multi-influence vertex
    rel = collections.Counter()
    def related(x, y):
        if sk.parent[x] == y or sk.parent[y] == x: return 'parent-child'
        if sk.parent[x] == sk.parent[y]: return 'siblings'
        ax = set(); q = x
        while q >= 0: ax.add(q); q = sk.parent[q]
        if y in ax: return 'ancestor'
        q = y
        while q >= 0:
            if q == x: return 'ancestor'
            q = sk.parent[q]
        return 'unrelated'
    for w in W:
        if len(w) > 1:
            w2 = sorted(w, key=lambda bw: -bw[1]); rel[related(w2[0][0], w2[1][0])] += 1
    shells = wlib.shells(nv, F); ns = int(shells.max()) + 1
    rigid = 0; rigid_v = 0; sizes = []
    for s in range(ns):
        idx = np.nonzero(shells == s)[0]; sizes.append(len(idx))
        if all(len(W[i]) == 1 for i in idx) and len({W[i][0][0] for i in idx}) == 1: rigid += 1; rigid_v += len(idx)
    seg = wlib.segments(sk); D = wlib.vertex_bone_distance(V, seg, sk.n)                # (nv, nb) distance to each bone's segments
    near = D.argmin(1); own = D[np.arange(nv), dom]; best = D.min(1)
    part = dict(file=f, vertices=nv, faces=len(F), influences={int(k): int(v) for k, v in sorted(inf.items())}, one_share=round(inf[1] / nv, 3),
                bones_weighted=len(used), bones_dominant=len(domset), bones_never_weighted=sk.n - len(used),
                second_bone_relation=dict(rel), shells=ns, shells_rigid=rigid, vertices_in_rigid_shells=rigid_v, largest_shell=max(sizes),
                dominant_is_nearest_segment=round(float((near == dom).mean()), 3), own_distance_median=round(float(np.median(own)), 1), nearest_distance_median=round(float(np.median(best)), 1),
                own_over_nearest_p90=round(float(np.quantile(own - best, .9)), 1),
                weights_multi_most_common=[(int(k), int(v)) for k, v in vals.most_common(8)], sums_not_255=sum(1 for w in W if sum(x for _, x in w) != 255))
    # is the dominant bone the nearest JOINT (head), or the nearest joint's parent?
    H = sk.head(); dj = np.linalg.norm(V[:, None] - H[None], axis=2); nj = dj.argmin(1)
    part['dominant_is_nearest_joint'] = round(float((nj == dom).mean()), 3)
    part['dominant_is_parent_of_nearest_joint'] = round(float((np.array(sk.parent)[nj] == dom).mean()), 3)
    leaf = [i for i in range(sk.n) if not sk.children[i]]
    mk = wlib.markers(sk); part['marker_bones'] = mk; part['never_weighted'] = [i for i in range(sk.n) if i not in set(used)]
    dfm = np.ones(sk.n, bool); dfm[mk] = False; D2 = wlib.vertex_bone_distance(V, wlib.segments_deform(sk, dfm), sk.n)
    part['dominant_is_nearest_segment_without_markers'] = round(float((D2.argmin(1) == dom).mean()), 3)
    part['leaf_bones'] = len(leaf); part['leaf_bones_weighted'] = sum(1 for i in leaf if i in set(used))
    out['parts'].append(part)
    print(f"{man['name']} {f}: {nv} vertices, {len(F)} faces, {sk.n} bones")
    for k, v in part.items():
        if k not in ('file', 'vertices', 'faces'): print(f'  {k}: {v}')
    if '--bones' in a:
        cnt = collections.Counter(dom.tolist())
        for i in range(sk.n):
            print(f"  bone {i:3d} parent {sk.parent[i]:3d} depth {sk.depth[i]:2d} children {len(sk.children[i])} head {np.round(sk.head()[i]).astype(int).tolist()} dominates {cnt.get(i, 0)}")
if '--json' in a: pathlib.Path(a[a.index('--json') + 1]).write_text(json.dumps(out, indent=1))
