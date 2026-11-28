"""dumpctl.py GROUP [--state]: print the Teacher's control component list compactly (abs positions)."""
import sys, json
from pathlib import Path
HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import ifref as R

g = int(sys.argv[1])
p = HERE / 'sheets-teacher-only' / str(g) / 'control.json'
if p.exists():
    d = json.loads(p.read_text())
    comps = d['components']
else:
    S = R.real_scene(g)
    comps = R.scene_to_components(S, g)
byf = {c['id']: c for c in comps}
absl = {}
for c in comps:
    pa = c['parent']
    base = absl[pa] if pa is not None else [0, 0]
    absl[c['id']] = [base[0] + c['raw_position'][0], base[1] + c['raw_position'][1]]
root = comps[0]
ra = absl[root['id']]
for c in comps:
    a = absl[c['id']]
    pa = c['parent']
    rel = [a[0] - ra[0], a[1] - ra[1]]
    k = c['kind']
    s = f"{c['file']:3d} p{(pa & 0xffff) if pa is not None else '-':>3} k{k} {'H' if c['hidden'] else ' '} @{rel} sz{c['raw_size']}"
    if k == 5:
        s += f" spr {c['sprites'][0]} tile {int(c['sprite_tiling'])} tr {c['transparency']} fh {int(c['flip_h'])} fv {int(c['flip_v'])}"
    elif k == 3:
        s += f" rect {c['colors'][0]:06x} fill {int(c['filled'])} tr {c['transparency']} fm {c.get('fill_mode', 0)}"
    elif k == 4:
        s += (f" text f{c['font']} {c['colors'][0]:06x} sh {int(c['text_shadow'])} al {c['text_alignment']} lh {c['line_height']}"
              f" {c['text']!r}")
    elif k == 9:
        s += f" line {c['colors'][0]:06x} w {c['line_width']}"
    print(s)
