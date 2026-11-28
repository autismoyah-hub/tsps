"""compare_rust.py [GROUPS...|all]: draw each group with the tooling client's interface_preview and with iface.py, count
differing pixels in total and outside the rectangles of components iface.py does not draw (models).
Writes harness/compare-rust.json."""
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

EXE = _AP.fix(r'<CLIENT_BIN>\interface_preview.exe')
TMP = HERE / 'harness' / 'tmp'
TMP.mkdir(parents=True, exist_ok=True)


def one(cache, g, keep=False):
    ref = TMP / f'rust-{g}.png'
    p = subprocess.run([EXE, str(C.CACHE_239), str(g), str(ref)], capture_output=True, text=True, timeout=120)
    if p.returncode:
        return dict(group=g, rust_error=(p.stderr or p.stdout).strip()[-300:])
    widgets = C.load_interface(cache, g)
    ras, rep = iface.render(cache, widgets)
    A = ras.rgb().astype(int)
    B = np.asarray(Image.open(ref).convert('RGB')).astype(int)
    d = (A != B).any(-1)
    lay, _ = iface.layout(widgets)
    mask = np.zeros_like(d)
    for i, why in rep['not_drawn']:
        c = lay[i]['clip']
        mask[c[1]:c[3], c[0]:c[2]] = True
    out = dict(group=g, rust=p.stdout.strip(), drawn=len(rep['drawn']), not_drawn=len(rep['not_drawn']),
               diff=int(d.sum()), diff_outside_undrawn=int((d & ~mask).sum()), changed_by_draw=int(((A != 0x29) .any(-1)).sum()))
    bg = np.array([0x29, 0x25, 0x1f])
    out['nonbg_pixels'] = int((A != bg).any(-1).sum())
    if keep:
        Image.fromarray(A.astype(np.uint8)).save(TMP / f'py-{g}.png')
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
    name = 'compare-rust.json' if args == ['all'] else 'compare-rust-some.json'
    (HERE / 'harness' / name).write_text(json.dumps(res, indent=0))
