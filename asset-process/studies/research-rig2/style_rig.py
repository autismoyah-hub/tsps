"""style_npc.py's checks for a BONE-WEIGHTED creature (the skeletal equivalent): the same measures (cmesh.measure) and the
same limits (style_npc.RULES), with 'part' = each vertex's heaviest bone where a labelled creature has its label.
usage: style_rig.py DEF.json [--txt OUT]      DEF.json: {"name", "models": [files beside it], "scale", "ambient", "contrast", "size", "zbuffer"}
Working files go under research-rig2/data."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, io, contextlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import riglib as R
sys.path.insert(1, _AP.fix(r'<ROOT>\studies\research-creatures'))
import style_npc
BL = R.BL; cmesh = BL.cmesh
a = sys.argv; d = pathlib.Path(a[1]).resolve(); j = json.loads(d.read_text()); buf = io.StringIO()
class Tee:
    def write(self, s): sys.__stdout__.write(s); buf.write(s)
    def flush(self): sys.__stdout__.flush()
with contextlib.redirect_stdout(Tee()):
    m = BL.with_weights('def:' + str(d)); r = cmesh.measure(m, tuple(j.get('scale', [128, 128])))
    print('==', d.name, f"({len(m['V'])} vertices, {len(m['F'])} faces; part = heaviest bone)")
    o = style_npc.check(r); print(f"OUT of GAME limits: {o['GAME']}   OUT of the RECENT band: {o['RECENT']}")
if '--txt' in a: pathlib.Path(a[a.index('--txt') + 1]).write_text(buf.getvalue())
