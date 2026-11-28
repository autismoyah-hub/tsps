"""How features are CONSTRUCTED: every small contrasting thing on a creature (a one-colour patch inside the skin, or a
separate small shell) with the numbers that say how it is built.

    feats = find(c)      list of dicts (see below)
usage: features.py ENTRY [--box upLo,upHi[,zLo,zHi]] [--pic OUT.png] [--yaw a,b] [--min-contrast N]
   prints the table; the picture shows the boxed region lit, plain and with each feature tinted and numbered.

Kinds:
  patch  a connected one-colour region of the main skin (or of any shell of more than SHELL_MAX faces), up to 1.5% of
         the skin, that contrasts with its surround (8 lit lightness, or another material)
  shell  a separate piece: connected by shared vertices to nothing else, at most SHELL_MAX faces
Numbers per feature:
  faces, area share of skin, size (longest extent) / creature height
  welded      share of its border length shared with skin faces (1 = cut into the skin's own topology; 0 = separate)
  closed      shell only: no open edge (a capped lump); open shells are cones / cards open where they meet the skin
  sunk        shell only: share of its vertices that lie inside the main skin (interpenetration)
  relief      height of the feature above (+) or below (-) the plane of its border, as a share of its own width
              (patch: border ring; shell: the skin under it): 0 = paint on the surface, negative = inset, positive = lump
  relief_h    the same height as a share of the creature's height
  border_fold mean fold in degrees between the feature's faces and the skin across its border (patch only)
  contrast    lit grey-level difference (0..255) between the feature and what surrounds it
"""
import sys, pathlib, json, math, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import cleanlib as cl
from explore import same_mat
SHELL_MAX = 0.06        # a separate shell up to this share of the creature's faces is a 'piece'
PATCH_MAX = 0.015

def tri_dist(P, A, B, C):
    """distance from points P [n,3] to triangles (A, B, C) [m,3]: returns [n] min distance and index"""
    out = np.full(len(P), np.inf); idx = np.zeros(len(P), int)
    AB, AC = B - A, C - A
    N = np.cross(AB, AC); nn = np.maximum((N * N).sum(1), 1e-12)
    for i, p in enumerate(P):
        AP = p - A
        d1, d2 = (AB * AP).sum(1), (AC * AP).sum(1)
        d00, d01, d11 = (AB * AB).sum(1), (AB * AC).sum(1), (AC * AC).sum(1)
        den = np.maximum(d00 * d11 - d01 * d01, 1e-12)
        v = (d11 * d1 - d01 * d2) / den; w = (d00 * d2 - d01 * d1) / den
        inside = (v >= 0) & (w >= 0) & (v + w <= 1)
        dist_plane = np.abs((AP * N).sum(1)) / np.sqrt(nn)
        # edge distances
        def seg(P0, D, L2):
            t = np.clip(((p - P0) * D).sum(1) / np.maximum(L2, 1e-12), 0, 1); q = P0 + D * t[:, None]; return np.linalg.norm(p - q, axis=1)
        de = np.minimum(seg(A, AB, d00), np.minimum(seg(A, AC, d11), seg(B, C - B, ((C - B) ** 2).sum(1))))
        d = np.where(inside, dist_plane, de)
        k = int(np.argmin(d)); out[i] = d[k]; idx[i] = k
    return out, idx

def inside_mesh(P, A, B, C):
    """ray casting along a skewed direction; True where the crossing count is odd"""
    d = np.array([0.2672612, 0.5345225, 0.8017837]); res = np.zeros(len(P), bool)
    e1, e2 = B - A, C - A
    pv = np.cross(d, e2); det = (e1 * pv).sum(1); good = np.abs(det) > 1e-9; inv = np.where(good, 1 / np.where(good, det, 1), 0)
    for i, p in enumerate(P):
        t0 = p - A; u = (t0 * pv).sum(1) * inv
        q = np.cross(t0, e1); v = (q @ d) * inv; t = (e2 * q).sum(1) * inv
        res[i] = (int((good & (u >= 0) & (v >= 0) & (u + v <= 1) & (t > 1e-6)).sum()) % 2) == 1
    return res

def find(c, min_contrast=8.0):
    cl.topology(c)
    F, V, C, area, LC = c['F'], c['V'], c['C'], c['area'], c['LC']
    skin = c['drawn'] & (c['A'] == 0)
    idx = np.nonzero(skin)[0]; nf = len(idx); tot = area[skin].sum()
    used = np.unique(F[skin]); hgt = float(np.ptp(V[used][:, 1]))
    litg = np.array([cl.grey(cl.TABLE[(int(C[i]) & 0xFF80) + int(round(LC[i].mean()))]) for i in range(len(F))])
    shell_of = c['shell']; sizes = collections.Counter(shell_of[int(i)] for i in idx)
    main = max(sizes, key=sizes.get)
    bigshell = {r for r, n in sizes.items() if n > SHELL_MAX * nf}
    ea, eb, el, dih, ev = c['e_a'], c['e_b'], c['e_len'], c['e_dih'], c['e_v']
    both = skin[ea] & skin[eb]; ea, eb, el, dih, ev = ea[both], eb[both], el[both], dih[both], ev[both]
    mainf = np.array([i for i in idx if shell_of[int(i)] in bigshell])
    MA, MB, MC = V[F[mainf, 0]], V[F[mainf, 1]], V[F[mainf, 2]]
    feats = []
    # ---- separate shells
    groups = collections.defaultdict(list)
    for i in idx: groups[shell_of[int(i)]].append(int(i))
    open_by = collections.Counter()
    for (a, b) in c['open']:
        f0 = c['vf'][a][0]
        if skin[f0]: open_by[shell_of[f0]] += 1
    for r, fs in groups.items():
        if r in bigshell: continue
        fs = np.array(fs); vs = np.unique(F[fs]); P = V[vs]
        ins = inside_mesh(P, MA, MB, MC); dist, near = tri_dist(P, MA, MB, MC)
        out_d = np.where(ins, -dist, dist)
        ext = np.ptp(P, axis=0); width = float(np.sort(ext)[1]) if len(vs) > 2 else float(ext.max())
        # surround: main-skin faces nearest to the piece's vertices
        nb = np.unique(mainf[near]); sur = float((litg[nb] * area[nb]).sum() / max(area[nb].sum(), 1e-9))
        me = float((litg[fs] * area[fs]).sum() / area[fs].sum())
        col = collections.Counter(int(x) for x in C[fs]).most_common(1)[0][0]
        feats.append(dict(kind='shell', faces=int(len(fs)), fs=fs, area=float(area[fs].sum() / tot), size_h=float(ext.max() / hgt), welded=0.0, closed=open_by.get(r, 0) == 0,
                          sunk=float(ins.mean()), touching=bool(dist.min() <= 0.02 * hgt), gap=float(dist.min() / hgt), relief=float(out_d.max() / max(width, 1e-9)), relief_h=float(out_d.max() / hgt),
                          border_fold=float('nan'), contrast=abs(me - sur), colour=cl.hsl(col), cen=V[vs].mean(0), surround_same=same_mat(col, int(C[nb[0]])) if len(nb) else False))
    # ---- one-colour patches in the big shells
    parent = {int(i): int(i) for i in mainf}
    def findr(x):
        while parent[x] != x: parent[x] = parent[parent[x]]; x = parent[x]
        return x
    inbig = np.array([shell_of[int(a)] in bigshell and shell_of[int(b)] in bigshell for a, b in zip(ea, eb)])
    for a, b, ok in zip(ea, eb, inbig):
        if ok and C[a] == C[b]: parent[findr(int(a))] = findr(int(b))
    pf = collections.defaultdict(list)
    for i in mainf: pf[findr(int(i))].append(int(i))
    border = collections.defaultdict(list)
    for k, (a, b, ok) in enumerate(zip(ea, eb, inbig)):
        if not ok: continue
        ra, rb = findr(int(a)), findr(int(b))
        if ra != rb: border[ra].append((k, int(b))); border[rb].append((k, int(a)))
    openset = collections.Counter()
    for (a, b) in c['open']:
        for f in c['vf'][a]:
            if b in F[f] and f in parent: openset[findr(int(f))] += float(np.linalg.norm(V[a] - V[b]))
    for r, fs in pf.items():
        fs = np.array(fs); a_ = float(area[fs].sum())
        if a_ > PATCH_MAX * tot or not border[r]: continue
        ks = np.array([k for k, _ in border[r]]); others = np.array([o for _, o in border[r]])
        bl = el[ks]; sur = float((litg[others] * bl).sum() / bl.sum()); me = float((litg[fs] * area[fs]).sum() / a_)
        col = int(C[fs[0]]); other_mat = float(bl[[not same_mat(col, int(C[o])) for o in others]].sum() / bl.sum())
        con = abs(me - sur)
        if con < min_contrast and other_mat < 0.7: continue
        bv = np.unique(ev[ks]); Pb = V[bv]; ctr = Pb.mean(0)
        nrm = (c['unit'][fs] * area[fs][:, None]).sum(0); nrm /= max(np.linalg.norm(nrm), 1e-9)
        if len(bv) >= 3:
            _, _, vt = np.linalg.svd(Pb - ctr); pn = vt[2]
            if pn @ nrm < 0: pn = -pn
        else: pn = nrm
        vs = np.unique(F[fs]); hts = (V[vs] - ctr) @ pn
        inner = np.setdiff1d(vs, bv)
        ext = np.ptp(V[vs], axis=0); width = math.sqrt(a_)
        h = float(hts[np.argmax(np.abs(hts))]) if len(hts) else 0.0
        feats.append(dict(kind='patch', faces=int(len(fs)), fs=fs, area=a_ / tot, size_h=float(ext.max() / hgt), welded=float(bl.sum() / (bl.sum() + openset.get(r, 0.0))), closed=None, sunk=0.0,
                          touching=True, gap=0.0, relief=h / max(width, 1e-9), relief_h=h / hgt, border_fold=float((np.abs(dih[ks]) * bl).sum() / bl.sum()),
                          border_concave=float(bl[dih[ks] < 0].sum() / bl.sum()), contrast=con, colour=cl.hsl(col), cen=V[vs].mean(0), other_mat=other_mat, inner_vertices=int(len(inner))))
    for f in feats:
        f['up'] = float((V[used][:, 1].max() - f['cen'][1]) / hgt)
        f['construction'] = classify(f)
    return feats, hgt

def classify(f):
    """One of: paint (flat on the skin), inset (cut in), relief (raised, part of the skin), piece-open (separate, open
    where it meets the skin), piece-closed (a separate capped lump), floating (separate and not touching)."""
    if f['kind'] == 'shell':
        if not f['touching']: return 'floating'
        return 'piece-closed' if f['closed'] else 'piece-open'
    if f['relief'] <= -0.12: return 'inset'
    if f['relief'] >= 0.20: return 'relief'
    return 'paint'

def in_box(f, c, box):
    V = c['V']; used = np.unique(c['F'][c['drawn'] & (c['A'] == 0)]); hi = V[used][:, 1].max(); h = np.ptp(V[used][:, 1])
    if not (box[0] <= f['up'] <= box[1]): return False
    if len(box) >= 4 and not (box[2] <= f['cen'][2] <= box[3]): return False
    return True

def picture(c, feats, box, out, yaws=(0, 40, 90), size=440):
    from PIL import Image, ImageDraw, ImageFont
    try: fnt = ImageFont.truetype('segoeui.ttf', 13)
    except Exception: fnt = ImageFont.load_default()
    V = c['V']; skin = c['drawn'] & (c['A'] == 0); used = np.unique(c['F'][skin]); Vu = V[used]
    hi = Vu[:, 1].max(); h = np.ptp(Vu[:, 1])
    y0, y1 = hi - box[1] * h, hi - box[0] * h
    sel = Vu[(Vu[:, 1] >= y0) & (Vu[:, 1] <= y1)]
    if len(box) >= 4: sel = sel[(sel[:, 2] >= box[2]) & (sel[:, 2] <= box[3])]
    b = (sel.min(0), sel.max(0))
    palette = [(255, 60, 60), (60, 255, 90), (80, 150, 255), (255, 220, 60), (255, 90, 240), (70, 240, 240), (255, 150, 50), (190, 190, 255)]
    rows = []
    for tinted in (False, True):
        cells = []
        for y in yaws:
            tint = {int(i): palette[n % 8] for n, f in enumerate(feats) for i in f['fs']} if tinted else None
            img, ids = cl.view(c, y, 8, size=size, box=b, tint=tint)
            canvas = np.zeros((size, size, 3), np.uint8); canvas[:] = cl.BG
            hh, ww = min(img.shape[0], size), min(img.shape[1], size); canvas[:hh, :ww] = img[:hh, :ww]
            im = Image.fromarray(canvas)
            if tinted:
                d = ImageDraw.Draw(im)
                for n, f in enumerate(feats):
                    m = np.isin(ids[:hh, :ww], f['fs'])
                    if m.sum() >= 3:
                        ys, xs = np.nonzero(m); d.text((int(xs.mean()) + 3, int(ys.mean()) - 7), str(n), fill=(255, 255, 255), font=fnt, stroke_width=2, stroke_fill=(0, 0, 0))
            cells.append(np.array(im))
        rows.append(np.concatenate(cells, 1))
    Image.fromarray(np.concatenate(rows, 0)).save(out)

def table(feats):
    print(f"{'#':>3s} {'kind':5s} {'construction':13s} {'faces':>5s} {'area%':>6s} {'size/h':>6s} {'weld':>5s} {'closed':>6s} {'sunk':>5s} {'relief':>6s} {'rel/h%':>6s} {'fold':>5s} {'contr':>5s} {'up':>5s} {'x':>5s} {'z':>5s}  colour")
    for n, f in enumerate(feats):
        print(f"{n:3d} {f['kind']:5s} {f['construction']:13s} {f['faces']:5d} {100 * f['area']:6.2f} {f['size_h']:6.3f} {f['welded']:5.2f} {str(f['closed']) if f['closed'] is not None else '-':>6s} {f['sunk']:5.2f} {f['relief']:6.2f} {100 * f['relief_h']:6.2f} "
              f"{f['border_fold']:5.0f} {f['contrast']:5.0f} {f['up']:5.2f} {f['cen'][0]:5.0f} {f['cen'][2]:5.0f}  {f['colour']}")

if __name__ == '__main__':
    a = sys.argv[1:]
    c = cl.load(a[0]); mc = float(a[a.index('--min-contrast') + 1]) if '--min-contrast' in a else 8.0
    feats, hgt = find(c, mc)
    box = [float(x) for x in a[a.index('--box') + 1].split(',')] if '--box' in a else [0, 1]
    if '--zfrac' in a:      # fractions of the body length from the front (-z) end
        zl, zh = [float(x) for x in a[a.index('--zfrac') + 1].split(',')]
        Vu = c['V'][np.unique(c['F'][c['drawn'] & (c['A'] == 0)])]; z0, z1 = Vu[:, 2].min(), Vu[:, 2].max()
        box = box[:2] + [z0 + zl * (z1 - z0), z0 + zh * (z1 - z0)]
    feats = [f for f in feats if in_box(f, c, box)]
    feats.sort(key=lambda f: (-f['up'], f['cen'][0]))
    print(f"== {c['name']}  height {hgt:.0f}  features in box: {len(feats)}")
    table(feats)
    if '--pic' in a:
        yaws = [float(x) for x in a[a.index('--yaw') + 1].split(',')] if '--yaw' in a else (0, 40, 90)
        picture(c, feats, box, a[a.index('--pic') + 1], yaws)
