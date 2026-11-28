"""extra.py: the second batch of measures, merged into data/measures.json.   usage: extra.py [key ...]
  x_*      does the SIGNATURE read at game size? pixels of the helm farther than 5 px (3 units) from the bare figure,
           and how many of them stand 12+ / 24+ grey from the ground colour; the same for the helm's whole rim
  bk_*     baked shade: paint lightness of the main steel by the way the face looks (front / side / back / top)
  fold*    share of shell edge length folded more than 35 / 45 / 55 degrees; by zone (skull dome, face, signature)
  cl_*     the creature study's cleancheck lines on the helm mesh (research-clean/cleancheck.py)
  br_*     the brief's own table (meridian-helm-v5-astra/measure.py: eye relief, painted hard folds, ...)
  pr_*     proportion: helm height over the shell's width at the skull, crest rise over skull width
"""
import sys, pathlib, json, collections, io, contextlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from scipy import ndimage
import h2
from h2 import L
import measures as M
import osrscache as o

DATA = h2.HERE / 'data'


def sig_pixels(keys):
    es = []
    for k in keys: es += [h2.entry(k), h2.entry(k, helm=False)]
    es.append('none')
    out = {k: {} for k in keys}
    for yaw in M.YAWS:
        a = h2.draw(f'x-{yaw}', es, yaw=yaw, zoom=300, cell=(170, 170), aim='head')
        cs = h2.cells(a, len(es)); bare = cs[-1]; bg = bare[2, 2]; bgG = float(bg.sum() / 3)
        bfig = np.abs(bare - bg).sum(2) > 0
        near = ndimage.binary_dilation(bfig, iterations=5)
        for i, k in enumerate(keys):
            c, Mk, _ = M.helm_cell(cs[2 * i], cs[2 * i + 1])
            G = h2.grey(cs[2 * i]); v = M.VN[yaw]
            sig = Mk & ~near
            r = out[k]
            n = int(sig.sum()); r[f'x_{v}_sig_px'] = n
            d = np.abs(G - bgG)
            r[f'x_{v}_sig_read12'] = int((sig & (d >= 12)).sum()); r[f'x_{v}_sig_read24'] = int((sig & (d >= 24)).sum())
            r[f'x_{v}_sig_read_share'] = round(float((sig & (d >= 12)).sum() / n), 3) if n >= 8 else None
            r[f'x_{v}_sig_strong_share'] = round(float((sig & (d >= 24)).sum() / n), 3) if n >= 8 else None
            r[f'x_{v}_sig_contrast'] = round(float(np.median(d[sig])), 1) if n >= 8 else None
            # thickness of the part that reads: mean width of the 12+ mask
            rd = sig & (d >= 12)
            if rd.sum() >= 8:
                per = (rd & ~ndimage.binary_erosion(rd)).sum()
                r[f'x_{v}_sig_read_thick'] = round(float(2 * rd.sum() / per), 2)
                op = ndimage.binary_opening(rd, np.ones((2, 2)))
                r[f'x_{v}_sig_read_2px'] = int(op.sum())
            else:
                r[f'x_{v}_sig_read_thick'] = 0.0; r[f'x_{v}_sig_read_2px'] = 0
            # whole helm: pixels that read against the ground
            r[f'x_{v}_helm_read_share'] = round(float((Mk & (d >= 12)).sum() / max(Mk.sum(), 1)), 3)
            r[f'x_{v}_helm_strong_share'] = round(float((Mk & (d >= 24)).sum() / max(Mk.sum(), 1)), 3)
    for k in keys:
        r = out[k]
        for q in ('sig_read12', 'sig_read24', 'sig_read_2px', 'sig_px'):
            vals = [r[f'x_{M.VN[y]}_{q}'] for y in M.YAWS]
            r[f'x_min_{q}'] = int(min(vals)); r[f'x_mean_{q}'] = round(float(np.mean(vals)), 1)
        r['x_frontside_min_read24'] = int(min(r['x_front_sig_read24'], r['x_side_sig_read24'], r['x_litside_sig_read24']))
        r['x_frontside_min_read12'] = int(min(r['x_front_sig_read12'], r['x_side_sig_read12'], r['x_litside_sig_read12']))
        for q in ('helm_read_share', 'helm_strong_share'):
            vals = [r[f'x_{M.VN[y]}_{q}'] for y in M.YAWS]
            r[f'x_min_{q}'] = round(float(min(vals)), 3); r[f'x_mean_{q}'] = round(float(np.mean(vals)), 3)
    return out


def baked(key, mh):
    r = {}
    T, n, area, ed, col, idx = M.edges_of(mh)
    col = np.array(col); fam = np.where(col[:, 1] <= 1, -1, col[:, 0])
    shell = np.array([not M.fitlib.wearer_face(mh['col'][i], mh['raw'][i]) for i in idx])
    w = collections.Counter()
    for f, a, s in zip(fam.tolist(), area.tolist(), shell.tolist()):
        if s: w[f] += a
    main = w.most_common(1)[0][0]; ism = (fam == main) & shell
    # outward orientation: flip normals that point to the head's centre
    cen = T.mean(1); out = cen - np.array([0, 182.0, -8.0]); flip = (n * out).sum(1) < 0
    nn = np.where(flip[:, None], -n, n)
    def wmean(mask):
        m = mask & ism
        return float((col[m][:, 2] * area[m]).sum() / area[m].sum()) if area[m].sum() > 0 else None
    zones = dict(front=nn[:, 2] < -0.6, side=np.abs(nn[:, 0]) > 0.6, back=nn[:, 2] > 0.6, top=nn[:, 1] > 0.6)
    for z, m in zones.items():
        v = wmean(m); r[f'bk_{z}_L'] = round(v, 1) if v is not None else None
        r[f'bk_{z}_area'] = round(float(area[m & shell].sum() / area[shell].sum()), 3)
    if r['bk_front_L'] and r['bk_side_L']:
        r['bk_side_minus_front'] = round(r['bk_side_L'] - r['bk_front_L'], 1)
        r['bk_side_over_front'] = round(r['bk_side_L'] / r['bk_front_L'], 2)
    if r['bk_front_L'] and r['bk_back_L']:
        r['bk_back_minus_front'] = round(r['bk_back_L'] - r['bk_front_L'], 1)
    # surface of side- and back-looking steel with paint under lightness 20 (it will draw near black on the unlit side)
    sb = (zones['side'] | zones['back']) & shell
    r['bk_sideback_under20'] = round(float(area[sb & (col[:, 2] < 20)].sum() / max(area[sb].sum(), 1e-9)), 3)
    r['bk_sideback_under16'] = round(float(area[sb & (col[:, 2] < 16)].sum() / max(area[sb].sum(), 1e-9)), 3)
    r['bk_sideback_L50'] = float(np.median(np.repeat(col[sb][:, 2], np.maximum(1, (area[sb] * 4).astype(int))))) if sb.any() else None
    # folds by threshold and by zone
    HT = None
    tot = collections.Counter(); hit = collections.Counter()
    for e, fs in ed.items():
        if len(fs) != 2: continue
        (i, ln), (j, _) = fs
        if not (shell[i] and shell[j]): continue
        ang = np.degrees(np.arccos(np.clip(n[i] @ n[j], -1, 1)))
        c = (cen[i] + cen[j]) / 2
        # zone: signature = farther than 3.5 units outside the bare head box; face = front below the brow; dome = the rest
        outside = max(abs(c[0]) - 11, c[1] - 196, -20 - c[2], c[2] - 4, 0)
        zone = 'sig' if outside > 3.5 else ('face' if c[2] < -10 and c[1] < 190 else 'dome')
        for zz in ('all', zone):
            tot[zz] += ln
            for t in (35, 45, 55):
                if ang > t: hit[zz, t] += ln
    for zz in ('all', 'sig', 'face', 'dome'):
        for t in (35, 45, 55):
            r[f'fold{t}_{zz}'] = round(float(hit[zz, t] / tot[zz]), 3) if tot[zz] > 0 else None
        r[f'edge_share_{zz}'] = round(float(tot[zz] / tot['all']), 3)
    r['fold45_len_per_sqrt'] = round(float(hit['all', 45] / np.sqrt(area[shell].sum())), 1)
    return r


def clean(key):
    sys.path.insert(0, str(h2.ROOT / 'builds' / 'research-clean'))
    import cleanlib as cl, cleancheck as cc
    kind, a = h2.H[key]['mesh']
    if kind == 'real':
        e = L.idx()[a]
        m = o.merge([o.model(k) for k in e['wm'][0] if k not in (-1, None)])
        rec = e.get('rec')
        if rec:
            mp = {int(x): int(y) for x, y in rec}; m['colors'] = [mp.get(c, c) for c in m['colors']]
    else:
        m = o.decode_model(pathlib.Path(a).read_bytes())
    nf = len(m['faces']); nv = len(m['vertices'])
    mm = dict(m)
    mm.setdefault('vertex_skins', [0] * nv); mm['vertex_skins'] = mm.get('vertex_skins') or [0] * nv
    mm['depth'] = mm.get('depth') or [0] * nf
    mm['alpha'] = mm.get('alpha') or [0] * nf
    mm['face_skins'] = mm.get('face_skins') or [0] * nf
    # the wearer redrawn inside an open helm is not the helm's skin: make it undrawn for this measure
    al = list(mm['alpha'])
    for i, c in enumerate(mm['colors']):
        h, s, l = o.hsl_split(c)
        if c in (6798, 4550) or (h == 4 and s == 3): al[i] = 255
    mm['alpha'] = al
    c = cl.from_mesh(mm, name=key)
    with contextlib.redirect_stdout(io.StringIO()):
        mres, off = cc.measure(c)
    r = {}
    for k2 in ('splotch', 'hard_over_firm', 'firm_share', 'largest_piece', 'stuck_faces', 'small_lumps', 'features_as_lumps', 'deliberate', 'marks', 'facets',
               'mushy_unpainted_over_hard', 'closed_pieces', 'shells'):
        v = mres.get(k2)
        if v is not None: r['cl_' + k2] = round(float(v), 3)
    return r


def brief_row(key, mh):
    sys.path.insert(0, str(h2.ROOT / 'builds' / 'meridian-helm-v5-astra'))
    import measure as BM
    buf = io.StringIO(); err = io.StringIO()
    try:
        with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(err):
            BM.row(key, mh)
    except Exception as ex:
        return {'br_error': str(ex)[:80]}
    p = [x.strip() for x in buf.getvalue().strip().strip('|').split('|')]
    names = ['name', 'faces', 'upper_fold', 'aft_tip', 'edge_thick', 'hard_painted', 'shallow_joins', 'eye_relief', 'step_p95', 'min_L', 'front_L', 'face_back', 'hs_groups', 'paints']
    r = {}
    for nme, v in zip(names, p):
        if nme in ('name', 'front_L', 'face_back'):
            if nme == 'front_L':
                a, b = v.replace('–', '-').split('-'); r['br_front_Lmin'] = float(a); r['br_front_Lmax'] = float(b); r['br_front_Lrange'] = float(b) - float(a)
            continue
        try: r['br_' + nme] = float(v)
        except ValueError: r['br_' + nme] = None
    return r


if __name__ == '__main__':
    keys = sys.argv[1:] or [k for k in h2.ORDER]
    f = DATA / 'measures.json'; D = json.load(open(f))
    X = sig_pixels(keys)
    for k in keys:
        mh = h2.mesh(k)
        r = D.setdefault(k, dict(group=h2.H[k]['group'], name=h2.H[k]['name']))
        r.update(X[k]); r.update(baked(k, mh)); r.update(brief_row(k, mh))
        try: r.update(clean(k))
        except Exception as ex: print('clean failed', k, repr(ex)[:200])
        if r.get('fit_w_skull'):
            r['pr_h_over_wskull'] = round(r['sil_front_h'] / r['fit_w_skull'], 2)
            r['pr_above_over_wskull'] = round(r['fit_above'] / r['fit_w_skull'], 2)
            r['pr_sigarea_over_sil'] = round(r['sig_front_area'] / r['sil_front_area'], 3)
        print(k, 'ok')
    json.dump(D, open(f, 'w'), indent=1)
