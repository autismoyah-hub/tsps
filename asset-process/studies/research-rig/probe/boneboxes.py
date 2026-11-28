import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))
import numpy as np
import riglib as R
B, RM = R.B, R.RM
work = pathlib.Path(sys.argv[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); src = work / man['src']
m = B.decode_model((src / man['models'][0]).read_bytes()); V = np.array(m['vertices'], float); V[:, 1] *= -1
dom = np.array([max(w, key=lambda bw: bw[1])[0] for w in m['bone_weights']])
for b in sorted(set(dom.tolist())):
    P = V[dom == b]; print(f'bone {b}: {len(P)} vertices, box (x, height, z) {P.min(0).astype(int).tolist()}..{P.max(0).astype(int).tolist()}')
