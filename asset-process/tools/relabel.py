"""Give one of our worn models the animation labels of a real item of the same slot, by nearest vertex.

usage: relabel.py OURS.bin TEMPLATE_ITEM_ID OUT.bin        (labels from the template's male worn model)
       relabel.py --check ITEM_A TEMPLATE_ITEM_B            (how well B's labels predict A's true ones)
Every vertex of ours takes the vertex label of the nearest template vertex; every face takes the face label of the
template face whose centroid is nearest. Geometry and colours are not changed. Check the result in pose.py: a
wrongly labelled vertex shows as a spike or a tear when the limb moves.
"""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np, osrscache as o, scene, author
from author import native_codec


def worn(item_id):
    j = next(json.loads(p.read_text()) for p in (scene.ARCH / 'items').glob(f'*/{item_id}.json'))
    V, VL, FC, FL = [], [], [], []
    for k in j['wear_models'][0]:
        if k in (-1, None): continue
        m = o.model(k); v = m['vertices']
        V += v; VL += (m['vertex_skins'] or [0] * len(v))
        for f, l in zip(m['faces'], m['face_skins']):
            FC.append([sum(v[i][a] for i in f) / 3 for a in range(3)]); FL.append(l)
    return np.array(V, float), np.array(VL), np.array(FC, float), np.array(FL)


def nearest(q, pts):
    out = np.empty(len(q), int)
    for s in range(0, len(q), 512):
        d = ((q[s:s + 512, None, :] - pts[None, :, :]) ** 2).sum(-1)
        out[s:s + 512] = d.argmin(1)
    return out


def transfer(verts, faces, template):
    TV, TVL, TFC, TFL = worn(template)
    v = np.array(verts, float)
    vl = TVL[nearest(v, TV)]
    fc = np.array([[sum(verts[i][a] for i in f) / 3 for a in range(3)] for f in faces], float)
    fl = TFL[nearest(fc, TFC)]
    return vl.tolist(), fl.tolist()


if __name__ == '__main__':
    if sys.argv[1] == '--check':
        a, b = int(sys.argv[2]), int(sys.argv[3])
        AV, AVL, AFC, AFL = worn(a)
        vl, _ = transfer([tuple(p) for p in AV], [], b)
        same = float((np.array(vl) == AVL).mean())
        print(f'item {a} labelled from item {b}: {100 * same:.1f}% of vertices get their true label; '
              f'true labels {sorted(set(AVL.tolist()))}; wrong pairs (true->given) {collections.Counter((int(t), int(g)) for t, g in zip(AVL, vl) if t != g).most_common(6)}')
    else:
        ours, template, out = sys.argv[1], int(sys.argv[2]), sys.argv[3]
        m = o.decode_model(pathlib.Path(ours).read_bytes())
        vl, fl = transfer(m['vertices'], m['faces'], template)
        g = dict(vertices=[list(p) for p in m['vertices']], faces=m['faces'], colors=m['colors'], vertex_skins=vl, face_skins=fl)
        if len(set(m['priorities'])) > 1: g['priorities'] = m['priorities']
        else: g['priority'] = m['priorities'][0]
        if any(m['render_types']): g['render_types'] = m['render_types']
        pathlib.Path(out).write_bytes(native_codec.model(g))
        print(out, 'vertex labels', dict(collections.Counter(vl)), 'face labels', dict(collections.Counter(fl)))
