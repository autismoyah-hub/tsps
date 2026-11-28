"""Venv-side shared parts of the rig route (research-rig): the research-bosses tools, with every working file they
write redirected into THIS folder's data/ (nothing is written into another build folder).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, json
HERE = pathlib.Path(__file__).resolve().parent
BOSSES = pathlib.Path(_AP.fix(r'<ROOT>\studies\research-bosses'))
sys.path.insert(0, str(HERE)); sys.path.insert(1, str(BOSSES))
(HERE / 'data').mkdir(exist_ok=True)
import bone_rebuild_lib as BL                      # loads research-bosses/data/inferred-seqs.json (read only)
import bonelib, bonecodec as B, rigmath as RM
BL.HERE = HERE; BL.cmesh.DATA = HERE / 'data'; BL.L.HERE = HERE
L = bonelib; RL = BL.L
PY = sys.executable
BLENDER = _AP.fix(r'<BLENDER>')

def blender(script, *args, blend=None, log=None):
    """Run a script in background Blender; returns its text. Raises on a non-zero exit."""
    import subprocess
    cmd = [BLENDER, '-b'] + ([str(blend)] if blend else []) + ['--python-exit-code', '1', '-P', str(script), '--'] + [str(a) for a in args]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if log: pathlib.Path(log).write_text(r.stdout + r.stderr)
    if r.returncode: raise SystemExit('BLENDER FAILED: ' + ' '.join(cmd) + '\n' + (r.stdout + r.stderr)[-3000:])
    return r.stdout
