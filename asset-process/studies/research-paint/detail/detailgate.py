"""The proposed gate line for small inlaid details of a creature rebuild.

usage: detailgate.py REAL_NPC_ID OURS.json [--tag NAME] [--json OUT.json]
       detailgate.py --results [TAG..]        the same line from results already in work/results (no drawing)
exit 0 PASS, 1 BORDERLINE, 2 FAIL

LINE  "small marks missing at game size"
  A MARK of the real creature = a connected one-colour region of its drawn opaque skin, at most 1.2% of the drawn
  surface, that contrasts with its surroundings (every neighbouring region differs by >= 12 paint lightness or is of
  another hue family; or it differs by >= 20 grey levels from its largest neighbour).
  It is VISIBLE in a game-size cell (zoom 1150, pitch 220; yaw 192, 704, 1216, 1728; rows bind, stand:0, walk:3,
  attack:4: 16 cells) if it is the front, opaque face on >= 2 pixels and painting every mark out would change those
  pixels by >= 12 grey levels (0-255) on average.
  It is KEPT in that cell if the rebuild's picture, at that footprint, shows at least half of the mark's effect
  (projection of [rebuild - real with marks out, less its mean over the ring of unmarked pixels within 2 px] on
  [real - real with marks out]), allowing a shift of up to 2 pixels (less where a neighbouring mark would answer).
  MISSING = sum over visible marks of (1 - share of its visible cells in which it is kept).
  PASS  MISSING <= 24      BORDERLINE  MISSING <= 30      FAIL  above.
  Printed with it, not gating: recall (1 - MISSING / visible marks) with a bootstrap standard error, recall by footprint
  class, and the rebuild's own visible marks that the real picture does not show (spurious).
Anchors of the thresholds (owner's blind line-ups): Minotaur rebuild 22.3 missing, taken for the real one; Nex rebuild
31.9, found ("less definition"); Tekton plate-first 45.2, found ("very minor detail missing"); first Tekton 93.1.
"""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import dlib
PASS, BORDER = 24.0, 30.0
def line(o):
    M = [m for m in o['real'] if not m['translucent'] and m['game']['kept'] is not None]
    k = np.array([m['game']['kept'] for m in M]); px = np.array([m['game']['max_px'] for m in M])
    S = [m for m in o['ours'] if not m['translucent'] and m['game']['kept'] is not None]
    sk = np.array([m['game']['kept'] for m in S])
    rng = np.random.default_rng(239); bs = [k[rng.integers(0, len(k), len(k))].mean() for _ in range(2000)] if len(k) else [0]
    missing = float((1 - k).sum()); verdict = 'PASS' if missing <= PASS else 'BORDERLINE' if missing <= BORDER else 'FAIL'
    by = {}
    for lo, hi, nm in dlib.SIZE_PX[1:]:
        s = (px >= lo) & (px <= hi)
        if s.any(): by[nm] = (float(k[s].mean()), int(s.sum()))
    return dict(tag=o['tag'], npc=o['npc'], verdict=verdict, missing=missing, visible=len(M), recall=float(k.mean()) if len(k) else 1.0, recall_se=float(np.std(bs)),
                by_footprint=by, ours_visible=len(S), spurious=float((1 - sk).sum()) if len(S) else 0.0)
def show(r):
    print(f"{r['verdict']:10s} small marks missing at game size: {r['missing']:.1f} of {r['visible']} visible marks of real NPC {r['npc']}   (PASS <= {PASS:.0f}, BORDERLINE <= {BORDER:.0f})   [{r['tag']}]")
    print(f"           recall {r['recall']:.3f} +/- {r['recall_se']:.3f}; by footprint " + ', '.join(f"{k} {v[0]:.2f} (n {v[1]})" for k, v in r['by_footprint'].items()))
    print(f"           the rebuild's own visible marks: {r['ours_visible']}, of which the real picture does not show {r['spurious']:.1f}")
if __name__ == '__main__':
    a = sys.argv[1:]
    if a and a[0] == '--results':
        fs = [dlib.WORK / 'results' / f'{t}.json' for t in a[1:]] or sorted((dlib.WORK / 'results').glob('*.json'))
        for f in fs: show(line(json.loads(f.read_text())))
        sys.exit(0)
    npc, path = int(a[0]), pathlib.Path(a[1]).resolve()
    tag = a[a.index('--tag') + 1] if '--tag' in a else 'gate-' + path.stem
    for f in (dlib.WORK / 'px').glob(f'ours*-{tag}.npz'): f.unlink()          # a rebuild's pictures are never reused
    import detail
    r = line(detail.run(f'{tag}={npc}:{path}')); show(r)
    if '--json' in a: pathlib.Path(a[a.index('--json') + 1]).write_text(json.dumps(r, indent=1))
    sys.exit(dict(PASS=0, BORDERLINE=1, FAIL=2)[r['verdict']])
