"""Reference points for the weight test: the REAL weights, simplified, so the gate's motion marks can be read.
usage: rig_wsimplify.py WORKDIR OUT.bin --max N [--steps K]
Keeps each vertex's N heaviest real influences (renormalised to 255; --steps K rounds shares to 1/K first)."""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R, wlib
B = R.B
a = sys.argv; work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); src = work / man['src']
m = B.decode_model((src / man['models'][0]).read_bytes()); nb = 256; W = wlib.dense(m['bone_weights'], nb); k = int(a[a.index('--max') + 1])
steps = int(a[a.index('--steps') + 1]) if '--steps' in a else 0
if '--blend-from-dominant' in a:
    # "labels, then blend": keep ONLY each vertex's dominant real bone (what an author assigns part by part), then let
    # the rule make the blends: R rounds of averaging with the mesh neighbours, kept to the k heaviest
    R_ = int(a[a.index('--blend-from-dominant') + 1]); keep = float(a[a.index('--keep') + 1]) if '--keep' in a else 0.5
    D = np.zeros_like(W); D[np.arange(len(W)), W.argmax(1)] = 1
    W = wlib.smooth(D, wlib.adjacency(len(W), m['faces']), R_, keep)
if steps:
    for i, row in enumerate(W):
        idx = np.argsort(-row)[:k]; w = row[idx] / row[idx].sum(); q = np.round(w * steps).astype(int); q[0] += steps - q.sum(); W[i] = 0; W[i, idx] = np.maximum(q, 0) / steps
m['bone_weights'] = wlib.to_bytes(W, k, 0.0, 0.0); pathlib.Path(a[2]).write_bytes(B.encode_model(m, hints=False)); print(a[2])
