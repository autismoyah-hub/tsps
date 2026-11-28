"""Exploration: alternative summaries of work/results/*.json, and the effect of the shift tolerance (explore.py k TAG..)."""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import dlib
R = dlib.WORK / 'results'
def summary(o):
    M = [m for m in o['real'] if not m['translucent'] and m['game']['kept'] is not None]
    k = np.array([m['game']['kept'] for m in M]); px = np.array([m['game']['max_px'] for m in M], float); ef = np.array([m['game']['max_con'] for m in M])
    w = px * ef
    S = [m for m in o['ours'] if not m['translucent'] and m['game']['kept'] is not None]
    sk = np.array([m['game']['kept'] for m in S]) if S else np.array([1.0])
    small = px <= 20
    return dict(n=len(M), recall=k.mean(), missing=(1 - k).sum(), recall_w=(k * w).sum() / w.sum(), recall_small=(k[small].mean() if small.any() else float('nan')), n_small=int(small.sum()),
                spurious=(1 - sk).sum(), n_ours=len(S), f1=2 * k.mean() * sk.mean() / max(k.mean() + sk.mean(), 1e-9))
if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == 'k':
        import detail
        from dlib import tl
        for tag in sys.argv[3:]:
            npc, path = dlib.OURS[tag]; mr, dr, seqs, ar, marks, pr, pr0 = detail.real_set(npc); mo, jo, po = dlib.ours_pictures(tag, npc, path)
            for kk in [int(x) for x in sys.argv[2].split(',')]:
                orig = dlib.cameras
                def cams(n, kk=kk, orig=orig):
                    c, sc = orig(n); c['game']['k'] = kk; return c, sc
                dlib.cameras = cams
                rec = dlib.compare(ar, marks, pr, pr0, po, npc, cams=('game',)); dlib.cameras = orig
                v = [r['game']['kept'] for r, m in zip(rec, marks) if r['game']['kept'] is not None and not m['translucent']]
                print(tag, 'k', kk, 'recall %.3f n %d' % (np.mean(v), len(v)), flush=True)
    else:
        print(f"{'case':18s} {'n':>4s} {'recall':>7s} {'missing':>8s} {'recall_w':>9s} {'small<=20px':>12s} {'ours':>5s} {'spurious':>9s} {'F1':>6s}")
        for f in sorted(R.glob('*.json')):
            o = json.loads(f.read_text()); s = summary(o)
            print(f"{o['tag']:18s} {s['n']:4d} {s['recall']:7.3f} {s['missing']:8.1f} {s['recall_w']:9.3f} {s['recall_small']:7.3f} ({s['n_small']:2d}) {s['n_ours']:5d} {s['spurious']:9.1f} {s['f1']:6.3f}")
