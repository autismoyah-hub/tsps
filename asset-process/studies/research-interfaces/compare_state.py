"""compare_state.py [GROUPS...|all]: let the tooling client open each group and run its onLoad scripts (iface_dump),
then draw the same laid-out tree with iface.py; count differing pixels in total and outside the rectangles of the
components iface.py does not draw (models, item icons). Writes harness/compare-state.json."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, subprocess
from pathlib import Path
import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import ifcache as C, iface

EXE = _AP.fix(r'<CLIENT_BIN>\iface_dump.exe')
OUT = HERE / 'state'
OUT.mkdir(exist_ok=True)


def one(cache, g, extra=(), tag=None, keep=False):
    tag = tag or str(g)
    ref, js = OUT / f'rust-{tag}.png', OUT / f'{tag}.jsonl'
    p = subprocess.run([EXE, str(C.CACHE_239), str(g), str(ref), str(js), *extra], capture_output=True, text=True,
                       timeout=180)
    if p.returncode:
        return dict(group=g, rust_error=(p.stderr or p.stdout).strip()[-300:])
    info = json.loads(p.stdout.strip().splitlines()[-1])
    ras, rep = iface.render_state(cache, js)
    A = ras.rgb().astype(int)
    B = np.asarray(Image.open(ref).convert('RGB')).astype(int)
    d = (A != B).any(-1)
    order, lay, byid = iface.load_state(js)
    mask = np.zeros_like(d)
    for i, why in rep['not_drawn']:
        c = lay[i]['clip']
        mask[c[1]:c[3], c[0]:c[2]] = True
    out = dict(group=g, tag=tag, widgets=info['widgets'], rust_drawn=info['drawn'], script_failures=info['failed'][:200],
               py_drawn=len(rep['drawn']), not_drawn=len(rep['not_drawn']), diff=int(d.sum()),
               diff_outside_undrawn=int((d & ~mask).sum()),
               nonbg=int((A != np.array([0x29, 0x25, 0x1f])).any(-1).sum()))
    if keep:
        Image.fromarray(A.astype(np.uint8)).save(OUT / f'py-{tag}.png')
    else:
        ref.unlink()
    return out


if __name__ == '__main__':
    cache = C.Cache()
    args = sys.argv[1:]
    groups = sorted(cache.reference(3)) if args == ['all'] else [int(a) for a in args]
    res = []
    for g in groups:
        try:
            r = one(cache, g, keep=len(groups) < 20)
        except Exception as e:
            r = dict(group=g, py_error=repr(e))
        res.append(r)
        print(json.dumps(r), flush=True)
    (HERE / 'harness' / ('compare-state.json' if args == ['all'] else 'compare-state-some.json')).write_text(
        json.dumps(res, indent=0))
