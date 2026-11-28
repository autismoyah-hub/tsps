"""A weighted creature is lit ONCE, in its bind pose. How far is what the player sees from that pose, and what does it
do to tone?

usage: rig_light.py WORKDIR [--model PART.bin --base BASE.dat --anim FILE.dat[,FILE..] --ambient A --contrast C(stored)] [--json OUT]
For the creature's stand, walk, attack and death (or the given keyframe files), at every key pose:
  turn      the angle between each vertex's normal in the bind pose and in the pose shown (degrees);
  light     the client's light number per vertex (LIGHTING.md: ambient' + dot(sum of the smooth faces' 256-length
            normals, (-30, -50, -30)) / (D x votes); 128 = paint as painted) in the bind pose = WHAT IS DRAWN, against
            what the same formula gives on the posed mesh = what a re-light would draw;
  wrong-way the share of the surface that is lit as if it faced UP (bind normal within 45 degrees of up) but is shown
            facing sideways or down, and the share shown facing up that was lit as a side or an underside.
Shares are by vertex over vertices used by drawn faces. The formula is this folder's float port (integer truncation
of the client is not reproduced: differences of a unit of light number).
"""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R
B, RM = R.B, R.RM
a = sys.argv; work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); src = work / man.get('src', 'src')
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
sk = RM.Skeleton(B.decode_base(pathlib.Path(opt('--base') or (src / man['base_file'])).read_bytes()))
m = B.decode_model(pathlib.Path(opt('--model') or (src / man['models'][0])).read_bytes()); V = np.array(m['vertices'], float); W = RM.weight_matrix(m['bone_weights'], sk.n); F = np.array(m['faces'])
amb = 64 + int(opt('--ambient', man.get('ambient', 0))); D = ((850 + 5 * int(opt('--contrast', man.get('contrast_stored', 0)))) * 65) >> 8
rt = np.array(m['render_types'] or [0] * len(F)); smooth = rt == 0; DIR = np.array([-30.0, -50.0, -30.0])
def vertex_normals(P):
    n = np.cross(P[F[:, 1]] - P[F[:, 0]], P[F[:, 2]] - P[F[:, 0]]); ln = np.linalg.norm(n, axis=1, keepdims=True); n = np.where(ln > 0, n / np.maximum(ln, 1e-12), 0) * 256
    S = np.zeros_like(P); votes = np.zeros(len(P))
    for k in range(3): np.add.at(S, F[smooth, k], n[smooth]); np.add.at(votes, F[smooth, k], 1)
    return S, votes
def light(S, votes): return amb + (S @ DIR) / (D * np.maximum(votes, 1))
S0, votes = vertex_normals(V); used = votes > 0; L0 = light(S0, votes); N0 = S0 / np.maximum(np.linalg.norm(S0, axis=1, keepdims=True), 1e-9)
# the model's stored winding against the light: which sign of the normal is "outward" is the client's; the up test uses
# the same normals the light uses, with the light direction's own convention (a surface the light favours has dot > 0)
up_axis = np.array([0.0, -1.0, 0.0])                      # model y is DOWN; the light's y component is -50: light favours normals with y < 0
sign = 1.0 if np.mean((S0[used] @ DIR)) != 0 else 1.0
jobs = [(pathlib.Path(f).stem, B.decode_anim(pathlib.Path(f).read_bytes()), None) for f in opt('--anim').split(',')] if opt('--anim') else \
    [(f"{k} {v}", B.decode_anim((src / next(s['file'] for s in man['sequences'] if s['seq'] == v)).read_bytes()), next(s['frames'] for s in man['sequences'] if s['seq'] == v)) for k, v in man['named'].items()]
upb = (N0 @ up_axis); out = []
# orientation check: on a closed creature the top of the bind pose must be bright. If the brightest tenth faces "down" by this axis, flip it.
bright = L0[used] >= np.quantile(L0[used], .9)
if (upb[used][bright]).mean() < 0: up_axis = -up_axis; upb = -upb
print(f"{man['name']}: {int(used.sum())} lit vertices; ambient' {amb}, D {D}; bind pose light number: median {np.median(L0[used]):.0f}, 10th to 90th centile {np.quantile(L0[used], .1):.0f} to {np.quantile(L0[used], .9):.0f}; bind surface facing up (within 45 degrees) {100 * (upb[used] > .707).mean():.0f}%, down {100 * (upb[used] < -.707).mean():.0f}%")
for name, an, n in jobs:
    n = n or an['range'][1]; tb = RM.channel_table(an, sk, range(n + 1)); t = RM.step_times(tb); P = RM.skin_fast(V, W, sk.skin_matrices(tb[t]))
    ang = []; dl = []; wrong_up = []; wrong_dn = []
    for k in range(len(t)):
        S, _ = vertex_normals(P[k]); N = S / np.maximum(np.linalg.norm(S, axis=1, keepdims=True), 1e-9)
        ang.append(np.degrees(np.arccos(np.clip((N * N0).sum(1), -1, 1)))[used]); dl.append((light(S, votes) - L0)[used]); up = (N @ up_axis)[used]
        wrong_up.append(((upb[used] > .707) & (up < .5)).mean()); wrong_dn.append(((up > .707) & (upb[used] < .5)).mean())
    ang = np.array(ang); dl = np.array(dl); a0 = ang[0]; d0 = dl[0]
    row = dict(name=name, poses=len(t), turn_median=float(np.median(ang)), turn_p90=float(np.quantile(ang, .9)), turned_over_45=float((ang > 45).mean()), turned_over_90=float((ang > 90).mean()),
               light_change_median=float(np.median(np.abs(dl))), light_change_p90=float(np.quantile(np.abs(dl), .9)), light_over_16=float((np.abs(dl) >= 16).mean()), light_over_32=float((np.abs(dl) >= 32).mean()),
               lit_up_shown_sideways=float(np.mean(wrong_up)), shown_up_lit_side=float(np.mean(wrong_dn)), first_pose_turn_median=float(np.median(a0)), first_pose_light_over_16=float((np.abs(d0) >= 16).mean()),
               drawn_too_bright=float((dl <= -16).mean()), drawn_too_dark=float((dl >= 16).mean()))
    out.append(row)
    print(f"  {name}: {len(t)} poses. Normals turned from the bind pose: median {row['turn_median']:.0f} degrees, 90th centile {row['turn_p90']:.0f}; over 45 degrees {100 * row['turned_over_45']:.0f}% of the surface, over 90: {100 * row['turned_over_90']:.0f}%."
          f" Light number a re-light would change: median {row['light_change_median']:.0f}, 90th centile {row['light_change_p90']:.0f} (of 128); by 16 or more on {100 * row['light_over_16']:.0f}% (drawn too bright {100 * row['drawn_too_bright']:.0f}%, too dark {100 * row['drawn_too_dark']:.0f}%), by 32 or more on {100 * row['light_over_32']:.0f}%."
          f" Lit as facing up but shown sideways or down: {100 * row['lit_up_shown_sideways']:.0f}%; shown facing up but lit as a side or underside: {100 * row['shown_up_lit_side']:.0f}%.")
if opt('--json'): pathlib.Path(opt('--json')).write_text(json.dumps(out, indent=1))
