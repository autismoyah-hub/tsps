"""fitlib.py: how a helm sits on the bare head, as numbers. Used by fit_study.py (the survey) and fit.py (the tool).

All numbers are from the worn mesh in the bind pose, in whole game units, against the bare head (hair + jaw kits:
x -11..11, height 164..196, face at z -20, back of the skull z 4) and, where a body piece is given, against it.

  stand-off  rays from the head's centre (0, 182, -8), every 10 degrees of bearing and elevation. On each ray:
             the farthest hit on the helm's SHELL minus the farthest hit on the bare head. Shell = every face of
             the helm that is not the wearer redrawn inside it (skin hue 4 sat 3, the hair base colour 6798,
             eye black). Positive = the helm stands off the head; negative = the helm is inside the bare hair.
             Regions (bearing 0 = straight ahead, 90 = the side, 180 = behind; elevation 0 = level, 90 = up):
               crown   elevation 60..90            brow    bearing 0..30,  elevation 10..40
               temple  bearing 60..120, el 0..40   nape    bearing 140..180, el -50..-10
               cheek   bearing 30..70, el -40..-10 jaw     bearing 0..20,  el -60..-30
  profile    the helm's front silhouette half-width and the side silhouette's front and back reach, per unit of
             height; `w_*` are full widths at crown-side 194, brow 188, cheek 178, jaw 171 and neck 166.
"""
import math
import numpy as np
from PIL import Image, ImageDraw
import lib as L

CENTRE = np.array([0.0, 182.0, -8.0])
HEAD_W, HEAD_H, HEAD_D = 22.0, 32.0, 24.0          # bare head: x -11..11, 164..196, z -20..4
REGIONS = dict(crown=((0, 180), (60, 90)), brow=((0, 30), (10, 40)), temple=((60, 120), (0, 40)),
               nape=((140, 180), (-50, -10)), cheek=((30, 70), (-40, -10)), jaw=((0, 20), (-60, -30)))
_HEAD = None


def head_tris():
    global _HEAD
    if _HEAD is None:
        hv, hf = L.kit('hair'); jv, jf = L.kit('jaw')
        _HEAD = np.vstack([L.tris(hv, hf), L.tris(jv, jf)])
    return _HEAD


def wearer_face(col, raw):
    h, s, l = col
    return raw in (6798, 4550) or (h == 4 and s == 3) or (raw == 0)


def shell_tris(m):
    keep = [i for i, f in enumerate(m['F']) if len(set(f)) == 3 and not wearer_face(m['col'][i], m['raw'][i])]
    V = m['V']
    return np.array([[V[a], V[b], V[c]] for a, b, c in (m['F'][i] for i in keep)], float).reshape(-1, 3, 3)


def all_tris(m):
    return L.tris(m['V'], m['F'])


def ray_far(orig, dirs, T):
    """farthest hit distance of each ray on triangles T (nan = no hit). Moller-Trumbore, both sides."""
    if len(T) == 0:
        return np.full(len(dirs), np.nan)
    v0, e1, e2 = T[:, 0], T[:, 1] - T[:, 0], T[:, 2] - T[:, 0]
    out = np.full(len(dirs), np.nan)
    for k, d in enumerate(dirs):
        p = np.cross(d, e2); det = (e1 * p).sum(1)
        ok = np.abs(det) > 1e-9
        inv = np.where(ok, 1.0 / np.where(ok, det, 1), 0)
        t0 = orig - v0
        u = (t0 * p).sum(1) * inv
        q = np.cross(t0, e1)
        v = (q @ d) * inv
        t = (e2 * q).sum(1) * inv
        hit = ok & (u >= -1e-6) & (v >= -1e-6) & (u + v <= 1 + 1e-6) & (t > 0.5)
        if hit.any():
            out[k] = t[hit].max()
    return out


def directions():
    rows = []
    for el in range(-60, 91, 10):
        for az in range(0, 360, 10):
            e, a = math.radians(el), math.radians(az)
            # bearing 0 = straight ahead = -z ; 90 = +x
            rows.append((az, el, (math.cos(e) * math.sin(a), math.sin(e), -math.cos(e) * math.cos(a))))
    return rows


_DIRS = directions()
_HEAD_R = None


def standoff(m):
    """per-ray stand-off of the shell from the bare head; returns dict of region medians and the distribution"""
    global _HEAD_R
    D = np.array([d for _, _, d in _DIRS])
    if _HEAD_R is None:
        _HEAD_R = ray_far(CENTRE, D, head_tris())
    rs = ray_far(CENTRE, D, shell_tris(m))
    so = rs - _HEAD_R
    az = np.array([a if a <= 180 else 360 - a for a, _, _ in _DIRS]); el = np.array([e for _, e, _ in _DIRS])
    both = ~np.isnan(so)
    headhit = ~np.isnan(_HEAD_R)
    upper = el >= -30                                   # the skull and face, not the neck hole
    out = {}
    cov = (both & upper).sum() / max((headhit & upper).sum(), 1)
    out['cover'] = round(float(cov), 2)
    v = so[both & upper]
    if len(v):
        for q, name in ((10, 'so_p10'), (50, 'so_med'), (90, 'so_p90')):
            out[name] = round(float(np.percentile(v, q)), 1)
        out['so_iqr'] = round(float(np.percentile(v, 75) - np.percentile(v, 25)), 1)
    for name, ((a0, a1), (e0, e1)) in REGIONS.items():
        sel = both & (az >= a0) & (az <= a1) & (el >= e0) & (el <= e1)
        tot = headhit & (az >= a0) & (az <= a1) & (el >= e0) & (el <= e1)
        out['so_' + name] = round(float(np.median(so[sel])), 1) if sel.sum() >= max(1, 0.5 * tot.sum()) else None
    out['_rays'] = [(a, e, None if np.isnan(s) else round(float(s), 1)) for (a, e, _), s in zip(_DIRS, so)]
    return out


# ------------------------------------------------------------------------------------------------ silhouettes
S = 4                      # raster pixels per unit
X0, X1, H0, H1 = -48, 48, 120, 228


def raster(T, view):
    """view 'f': x right, height up. view 's': z right (front of the head to the LEFT), height up."""
    im = Image.new('L', ((X1 - X0) * S, (H1 - H0) * S), 0); d = ImageDraw.Draw(im)
    ax = 0 if view == 'f' else 2
    for t in T:
        pts = [((p[ax] - X0) * S, (H1 - p[1]) * S) for p in t]
        d.polygon(pts, fill=255); d.line(pts + [pts[0]], fill=255)
    return np.asarray(im) > 0


def row_of(h):
    return int(round((H1 - h) * S))


def extent(mask, h, band=1.0):
    """(min, max) coordinate of the mask in the rows of height h +- band/2, in units; None if empty"""
    r0, r1 = row_of(h + band / 2), row_of(h - band / 2)
    cols = np.nonzero(mask[r0:max(r1, r0 + 1)].any(0))[0]
    if not len(cols):
        return None
    return cols.min() / S + X0, (cols.max() + 1) / S + X0


def profile(T):
    f, s = raster(T, 'f'), raster(T, 's')
    P = {}
    for h in range(160, 216):
        ef, es = extent(f, h), extent(s, h)
        P[h] = dict(w=(ef[1] - ef[0]) if ef else 0.0, zf=es[0] if es else None, zb=es[1] if es else None)
    return P, f, s


def measure(m, body=None, hides=(True, True)):
    """m: the helm mesh (lib dict). body: the body piece mesh it is worn with, or None. hides: (hair, jaw) hidden."""
    V = m['V']; A = all_tris(m)
    out = dict(faces=len(m['F']))
    out.update(standoff(m))
    worn = [A]
    hv, hf = L.kit('hair'); jv, jf = L.kit('jaw')
    if not hides[0]: worn.append(L.tris(hv, hf))
    if not hides[1]: worn.append(L.tris(jv, jf))
    W = np.vstack(worn)                                  # what is seen above the collar: the helm and any kit left
    P, f, s = profile(W)
    HP, hf_, hs_ = profile(head_tris())
    top = max(h for h in P if P[h]['w'] > 0)
    xs = V[:, 0]; ys = V[:, 1]; zs = V[:, 2]
    out['wid'] = round(float((xs.max() - xs.min()) / HEAD_W), 2)
    out['hgt'] = round(float((ys.max() - 164.0) / HEAD_H), 2)
    out['dep'] = round(float((zs.max() - zs.min()) / HEAD_D), 2)
    out['above'] = float(ys.max() - 196); out['below'] = float(164 - ys.min())
    out['fore'] = float(-20 - zs.min()); out['aft'] = float(zs.max() - 4)
    for name, h in (('w_crown', 194), ('w_brow', 188), ('w_cheek', 178), ('w_jaw', 171), ('w_neck', 166)):
        out[name] = round(P[h]['w'], 1)
    for name, h in (('d_brow', 188), ('d_cheek', 178), ('d_jaw', 171), ('d_neck', 166)):
        out[name] = round((P[h]['zb'] - P[h]['zf']), 1) if P[h]['zf'] is not None else 0.0
    # the body of the helm: rows 166..196 only, so a crest or horn above the skull does not count
    wb = {h: P[h]['w'] for h in range(166, 197)}
    hmax = max(wb, key=lambda h: (wb[h], -h)); out['w_max'] = round(wb[hmax], 1); out['h_wmax'] = hmax
    # median-width "shell" of the skull band 180..194, which ignores horns standing out at one height
    out['w_skull'] = round(float(np.median([wb[h] for h in range(180, 195)])), 1)
    out['taper_jaw'] = round(out['w_jaw'] / max(out['w_skull'], 1), 2)      # 1 = a bucket or ball, low = a V face
    out['taper_neck'] = round(out['w_neck'] / max(out['w_skull'], 1), 2)
    d_sk = [P[h]['zb'] - P[h]['zf'] for h in range(180, 195) if P[h]['zf'] is not None]
    out['d_skull'] = round(float(np.median(d_sk)), 1) if d_sk else 0.0
    # roundness of the front outline over the head band 166..196: silhouette area / area of the ellipse on its box
    band = f[row_of(196):row_of(166)]
    cols = np.nonzero(band.any(0))[0]
    bw = (cols.max() - cols.min() + 1) / S if len(cols) else 1
    out['box_fill'] = round(float(band.sum() / S / S / (bw * 30.0)), 2)     # 1 = a box, 0.785 = an ellipse
    # the front-view chin: lowest point of the helm in front of the face, and the lowest point at the back
    front = V[(zs <= -12)]; back = V[(zs >= 0)]; side = V[np.abs(xs) >= 9]
    out['low_front'] = float(front[:, 1].min()) if len(front) else None
    out['low_back'] = float(back[:, 1].min()) if len(back) else None
    out['low_side'] = float(side[:, 1].min()) if len(side) else None
    if body is not None:
        BT = L.tris(body['V'], body['F'])
        bf_, bs_ = raster(BT, 'f'), raster(BT, 's')
        both_f = f | bf_; both_s = s | bs_
        # shoulders: the body piece's greatest width between heights 140 and 168
        sh = max((extent(bf_, h) or (0, 0))[1] - (extent(bf_, h) or (0, 0))[0] for h in range(140, 169))
        out['shoulder_w'] = round(sh, 1)
        out['helm_to_shoulder'] = round(out['w_skull'] / max(sh, 1), 2)
        out['helmmax_to_shoulder'] = round(float(xs.max() - xs.min()) / max(sh, 1), 2)
        # collar: the body's top, at the neck (|x| <= 12) and anywhere
        bv = body['V']
        out['collar_top'] = float(bv[np.abs(bv[:, 0]) <= 12][:, 1].max())
        out['body_top'] = float(bv[:, 1].max())
        # the pinch: narrowest width of helm + body between heights 160 and 174, front and side
        wf = {h: (extent(both_f, h) or (0, 0)) for h in range(158, 176)}
        ws = {h: (extent(both_s, h) or (0, 0)) for h in range(158, 176)}
        nf = min(wf, key=lambda h: wf[h][1] - wf[h][0]); ns = min(ws, key=lambda h: ws[h][1] - ws[h][0])
        out['pinch_f'] = round(wf[nf][1] - wf[nf][0], 1); out['pinch_f_h'] = nf
        out['pinch_s'] = round(ws[ns][1] - ws[ns][0], 1); out['pinch_s_h'] = ns
        out['pinch_f_ratio'] = round(out['pinch_f'] / max(out['w_skull'], 1), 2)
        out['pinch_s_ratio'] = round(out['pinch_s'] / max(out['d_skull'], 1), 2)
        # undercut under the chin (side view): how far the helm's front at cheek height stands in front of the
        # front of helm + body at the pinch height: an overhang reads as a head pushed forward on a stalk
        zf_cheek = min(P[h]['zf'] for h in range(172, 182) if P[h]['zf'] is not None)
        out['chin_overhang'] = round(ws[ns][0] - zf_cheek, 1)
        zb_sk = max(P[h]['zb'] for h in range(172, 196) if P[h]['zb'] is not None)
        out['nape_overhang'] = round(zb_sk - ws[ns][1], 1)
        # overlap between the helm's lowest edge and the body's top, by bearing (units; + = they overlap in height)
        out['lap_front'] = round(out['collar_top'] - (out['low_front'] if out['low_front'] is not None else 200), 1)
    out['_profile'] = {h: (round(P[h]['w'], 1), P[h]['zf'], P[h]['zb']) for h in P}
    out['_top'] = top
    return out


def head_measure():
    HP, f, s = profile(head_tris())
    return {h: (round(HP[h]['w'], 1), HP[h]['zf'], HP[h]['zb']) for h in HP}
