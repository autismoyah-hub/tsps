"""A joint drawn close up through its bend, and measured: ours or a real creature's, by the same code.

usage: bend_view.py OUT.png ROW [ROW ..] [--angles 0,45,90,120,140] [--cell 300] [--upp 0.42]
ROW = LABEL@MODEL.bin@BASE.dat@ANIM.dat[+ANIM2.dat..]@JOINT      JOINT = bone index, or x,height,z of the joint (nearest bone head)
For each row and each angle: the pose (any cycle of the given keyframe files) where the joint's bone is turned against
its parent by the nearest amount to the angle (the angle found is printed in the cell); the skin is posed by the float64
port of the client's skin (rigmath), the PARENT bone is held still, and the limb is drawn from the side (along the hinge's
axis) and from the inner side of the bend, flat colours with the face edges, by a painter's sort. Not the client's
rasteriser: a diagram of exact posed vertices. Only faces whose vertices ride the two bones and their neighbours in the
chain are drawn.
Measured per cell (printed and written under the cell):
  width   the ring of vertices ON the joint (within 0.2 limb radii of the joint plane): the smallest ratio of a vertex's
          distance from the joint to its distance at rest (1.00 = no collapse; a half-and-half vertex in the plane of
          the bend gives cos(angle / 2): linear skinning's loss of volume);
  flipped faces of the drawn region whose normal has turned over against the way its own bones turned it (a pinch or a
          fold through itself);  thin = faces under a fifth of their rest area.
"""
import sys, pathlib, colorsys
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import riglib as R
from ring_census import rotvec
B, RM = R.B, R.RM
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
angles = [float(x) for x in opt('--angles', '0,45,90,120,140').split(',')]; cs = int(opt('--cell', 300)); upp = float(opt('--upp', 0.42))
rows = [x for x in a[2:] if x.count('@') == 4]
def font(n):
    try: return ImageFont.truetype('segoeuib.ttf', n)
    except Exception: return ImageFont.load_default()
def rgb(c):
    h, s, l = (c >> 10) & 63, (c >> 7) & 7, c & 127; return np.array(colorsys.hls_to_rgb(h / 64 + 1 / 128, min(1, l / 127 * 1.25 + .08), s / 8 + 1 / 16))
sheet = Image.new('RGB', (len(angles) * 2 * cs, len(rows) * (cs + 40)), (0x30, 0x2b, 0x25)); dd = ImageDraw.Draw(sheet)
for ri, spec in enumerate(rows):
    label, mf, bf, afs, joint = spec.split('@')
    sk = RM.Skeleton(B.decode_base(pathlib.Path(bf).read_bytes())); m = B.decode_model(pathlib.Path(mf).read_bytes())
    V = np.array(m['vertices'], float); F = np.array(m['faces']); W = RM.weight_matrix(m['bone_weights'], sk.n); H = sk.head()
    if ',' in joint:
        x, h, z = [float(t) for t in joint.split(',')]; c = int(np.argmin(np.linalg.norm(H - np.array([x, -h, z]), axis=1)))
    else: c = int(joint)
    p = sk.parent[c]; tabs = []
    for af in afs.split('+'):
        an = B.decode_anim(pathlib.Path(af).read_bytes()); tabs.append(RM.channel_table(an, sk, range(an['range'][1] + 1)))
    tb = np.concatenate(tabs); rest = sk.local[c, :3, :3]; rest = rest / np.linalg.norm(rest, axis=-1, keepdims=True)
    M = RM.compose(tb[:, c, 0:3], tb[:, c, 3:6], np.ones_like(tb[:, c, 6:9]))[..., :3, :3]; rv = rotvec(rest.T[None] @ M); ang = np.degrees(np.linalg.norm(rv, axis=-1))
    kmax = int(ang.argmax()); Gp = sk.rest[p][:3, :3]; Gp = Gp / np.linalg.norm(Gp, axis=0)
    # hinge axis in bone space: the child's local rotation axis carried by the parent's and the child's rest frames
    Gc = sk.rest[c][:3, :3]; hinge = Gp @ rest @ (rv[kmax] / max(np.linalg.norm(rv[kmax]), 1e-9))
    hinge = hinge / np.linalg.norm(hinge); S = np.diag([1.0, -1.0, -1.0])                 # model <-> bone space
    jp = S @ H[c]; u = jp - S @ H[p]; u = u / np.linalg.norm(u); w = np.cross(hinge, u); w /= np.linalg.norm(w); hinge = np.cross(u, w)
    chain = {p, c} | set(sk.children[c]) | ({sk.parent[p]} if sk.parent[p] >= 0 else set())
    core = (W[:, p] + W[:, c]) > 0; okv = (W[:, sorted(chain)].sum(1) > 0.99); fsel = okv[F].all(1) & core[F].any(1); Fr = F[fsel]; cols = np.array(m['colors'])[fsel]
    dom = W.argmax(1); K_all = sk.skin_matrices(tb)
    # rest geometry of the joint ring
    Pb0 = V @ S.T - jp; x0 = Pb0 @ u; rad = np.median(np.linalg.norm(Pb0[core & (np.abs(x0) < 12)] - np.outer(x0[core & (np.abs(x0) < 12)], u), axis=1)) if (core & (np.abs(x0) < 12)).sum() > 3 else 10.0
    ringv = core & (np.abs(x0) < 0.2 * rad); n0 = np.cross(Pb0[Fr[:, 1]] - Pb0[Fr[:, 0]], Pb0[Fr[:, 2]] - Pb0[Fr[:, 0]]); a0 = np.linalg.norm(n0, axis=1)
    print(f'{label}: joint at bone {c} (parent {p}), limb radius at the joint {rad:.1f}, bones {np.linalg.norm(H[c] - H[p]):.0f} long, {int(fsel.sum())} faces drawn, {int(ringv.sum())} ring vertices; largest turn in these files {ang.max():.0f} degrees')
    for ai, A in enumerate(angles):
        k = int(np.argmin(np.abs(ang - A))); K = K_all[k]; P = RM.skin_fast(V, W, K) @ S.T                      # bone space
        Kp = K[p]; Pp = (np.linalg.inv(Kp) @ np.concatenate([P, np.ones((len(P), 1))], 1).T).T[:, :3] - jp        # parent held still, joint at 0
        # measures
        Rrel = (np.linalg.inv(Kp) @ K[c])[:3, :3]; cdir = Rrel @ u; bis = cdir - u                                # bisector direction of the bend (toward the inner side)
        if np.linalg.norm(bis) < 1e-6: bis = w
        bis = bis / np.linalg.norm(bis); rr = np.linalg.norm(Pp[ringv], axis=1) / np.maximum(np.linalg.norm(Pb0[ringv], axis=1), 1e-9); width = float(rr.min())
        n1 = np.cross(Pp[Fr[:, 1]] - Pp[Fr[:, 0]], Pp[Fr[:, 2]] - Pp[Fr[:, 0]]); a1 = np.linalg.norm(n1, axis=1)
        Kd = (np.linalg.inv(Kp)[None] @ K[dom[Fr[:, 0]]])[:, :3, :3]; exp = np.einsum('fij,fj->fi', Kd, n0); flipped = int(((n1 * exp).sum(1) < 0).sum()); thin = int((a1 < 0.2 * a0).sum())
        print(f'   angle wanted {A:.0f}, found {ang[k]:.0f} (cycle {k}): joint ring kept at least {width:.2f} of its radius (a half-and-half vertex in the bend plane keeps {np.cos(np.radians(ang[k]) / 2):.2f}); faces flipped {flipped}, under a fifth of their area {thin}')
        for vi, (ex, ez) in enumerate(((w, hinge), (hinge * 0.82 + w * 0.57 * (1 if True else -1), None))):
            if ez is None:                                        # the second view: from the inner side of the bend, three-quarter
                inner = bis if A > 5 else w; ez = -(inner * 0.75 + hinge * 0.66); ez /= np.linalg.norm(ez); ex = np.cross(u, ez); ex /= np.linalg.norm(ex)
            X = Pp @ ex; Y = Pp @ u; Z = Pp @ ez
            im = Image.new('RGB', (cs, cs), (0x30, 0x2b, 0x25)); d = ImageDraw.Draw(im); order = np.argsort(Z[Fr].mean(1))
            nview = np.abs((n1 / np.maximum(a1[:, None], 1e-9)) @ ez)
            for f in order:
                pts = [(cs / 2 + X[i] / upp, cs * 0.45 + Y[i] / upp) for i in Fr[f]]; col = rgb(int(cols[f])) * (0.5 + 0.5 * nview[f])
                d.polygon(pts, fill=tuple(int(255 * min(1, x)) for x in col), outline=(25, 22, 20))
            sheet.paste(im, ((ai * 2 + vi) * cs, ri * (cs + 40) + 40))
        dd.text((ai * 2 * cs + 6, ri * (cs + 40) + 22), f'{ang[k]:.0f} deg: width {width:.2f}, flipped {flipped}, thin {thin}', fill=(230, 230, 180), font=font(13))
    dd.text((6, ri * (cs + 40) + 3), f'{label}  (each angle: from the side | from inside the bend)', fill=(255, 255, 255), font=font(15))
pathlib.Path(a[1]).parent.mkdir(parents=True, exist_ok=True); sheet.save(a[1]); print(a[1], sheet.size)
