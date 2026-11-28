"""The creature rebuild library (research-creatures/rebuild_lib.py) pointed at this folder and taught about bones.

Everything rebuild_lib does is kept (definitions, sequences, the motion grid drawn by asset_npc.exe, picture
metrics): asset_npc already decodes bone weights from a model FILE and plays keyframe sequences by id, so a rebuild
given as files is drawn under the real creature's sequences exactly as a labelled one is. Added here:
  - working files go under research-bosses/data (nothing is written into research-creatures);
  - attack / death sequences the server tables lack (data/inferred-seqs.json, each looked at once on a sheet);
  - bones(entry): the rest mesh WITH weights (asset_bone.exe merged), each vertex's dominant bone;
  - skeleton(base id): bones, parents, rest positions in model space.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, json
import numpy as np
HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE)); sys.path.insert(1, _AP.fix(r'<ROOT>\studies\research-creatures')); sys.path.insert(2, _AP.fix(r'<ROOT>\studies\research-npc'))
import cmesh, rebuild_lib as L, bonelib, bonecodec
cmesh.DATA = HERE / 'data'; L.HERE = HERE
(HERE / 'data').mkdir(exist_ok=True)
_inf = HERE / 'data' / 'inferred-seqs.json'
if _inf.exists():
    for k, v in json.loads(_inf.read_text()).items(): L.KNOWN_SEQS.setdefault(int(k), {}).update(v)

def is_weighted(npc_id):
    models = L.census('models'); d = L.npc_def(npc_id)
    return any(models[m]['weights'] for m in d['models'] if m in models)
def with_weights(entry):
    """The assembled rest mesh with weights, in the same vertex order as cmesh.load(entry). Adds m['W'] and m['DOM']
    (dominant bone per vertex, -1 when a vertex has no weights) and sets m['L'] = DOM + 1 so that the measures that
    group by label group by bone."""
    m = cmesh.load(entry)
    work = HERE / 'data' / 'merged-w'; work.mkdir(parents=True, exist_ok=True)
    import hashlib
    out = work / (hashlib.md5(entry.encode()).hexdigest()[:10] + '.txt')
    w = bonelib.merged(cmesh.entry_to_exe(entry, HERE / 'data' / 'merged' / 'defs'), out)
    assert len(w['W']) == len(m['V']) and (w['V'] == m['V']).all(), 'the two harnesses disagree on the rest mesh'
    m['W'] = w['W']; m['DOM'] = np.array([max(ws, key=lambda bw: bw[1])[0] if ws else -1 for ws in w['W']], int)
    m['L_labels'] = m['L'].copy(); m['L'] = m['DOM'] + 1
    out.unlink()
    return m
def skeleton(base_id):
    """Bones of a frame base: parent, rest position in MODEL space as the model stores it (x, y down, z), depth."""
    b = bonecodec.decode_base(bonelib.base_bytes(int(base_id))); n = len(b['bones']); G = [None] * n
    def glob(i):
        if G[i] is None:
            M = np.array(b['bones'][i]['matrices'][0], float).reshape(4, 4).T; p = b['bones'][i]['parent']
            G[i] = glob(p) @ M if p >= 0 else M
        return G[i]
    out = []
    for i in range(n):
        g = glob(i); x, y, z = g[:3, 3]; p = b['bones'][i]['parent']; d = 0; q = p
        while q >= 0: d += 1; q = b['bones'][q]['parent']
        out.append(dict(bone=i, parent=p, depth=d, rest=[float(x), float(-y), float(-z)]))        # the skin step negates y and z
    return out, b
def bone_table(m):
    """{bone: (vertices it dominates, weighted centroid, total weight / 255)} over the vertices used by faces."""
    used = np.unique(m['F']); acc = {}
    for i in used:
        for bone, w in m['W'][i]:
            a = acc.setdefault(bone, [0, np.zeros(3), 0.0]); a[1] += m['V'][i] * w; a[2] += w
        if m['DOM'][i] >= 0: acc.setdefault(int(m['DOM'][i]), [0, np.zeros(3), 0.0])[0] += 1
    return {b: (a[0], a[1] / a[2] if a[2] else a[1], a[2] / 255) for b, a in acc.items()}
