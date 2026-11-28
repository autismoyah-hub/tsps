"""Step 1: decompose the gate's TONE error of a rebuild by cause, per region, per view, standing and in motion.

usage: decomp.py NAME [--per N]        NAME in tl.BOSSES / tl.MIDS, or  NAME=NPC_ID:PATH.json
writes work/decomp/NAME.json, prints the tables, pictures to render/research-tone-decomp/NAME-*.png

Method (all pictures by the client's harness under the gate's own cameras; see tl.py):
  per pixel of the shared silhouette we know, for the REAL mesh and for OURS: the front face (ID pictures), hence its
  paint, alpha, region, whether it lies in a small contrasting island; and the client's light number (LIGHT picture).
  With the game's colour table we synthesise four pictures: paint x light for (real, ours) x (real, ours).
    D   pixels where either front face is translucent, or the code picture is unreadable (alpha animation): cause (d),
        measured on the real pictures, not modelled.
    S   the solid rest. There:
        PAINT alone  = |synth(our paint, L) - synth(real paint, L)|, averaged over L = real light and L = our light
        LIGHT alone  = |synth(P, our light) - synth(P, real light)|, averaged over P = real paint and P = our paint
        each then split by ALIGNMENT: the same quantity after letting every pixel find its value within K pixels of the
        other picture (both directions, the worse of the two): what vanishes is cause (f), a detail a few pixels off;
        aligned PAINT error is split by where it lies: in a small contrasting island of the real mesh (detail missing
        or wrong, cause c), in an island only ours has (spurious detail, cause c reversed), elsewhere (broad paint, a);
        aligned LIGHT error is cause (b), light from shape.
  The causes alone do not add up to the total (absolute differences; and the authors fitted paint UNDER their light to
  cancel light errors), so the gate's number is ALLOCATED in proportion to the causes' own magnitudes; both are printed.
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections
import numpy as np
from PIL import Image, ImageDraw
import tl
from tl import L

def islands(a):
    """Per face: True if it lies in a small contrasting one-colour region (the cmesh idea, simplified):
    region area <= 1.2% of the drawn surface and its paint differs by >= 20 grey levels from its largest neighbour."""
    V, F, C = a['V'], a['F'], a['C']; dr = tl.drawn(a)
    n = np.cross(V[F[:, 1]] - V[F[:, 0]], V[F[:, 2]] - V[F[:, 0]]); area = np.linalg.norm(n, axis=1) / 2
    tot = area[dr].sum(); parent = list(range(len(F)))
    def find(i):
        while parent[i] != i: parent[i] = parent[parent[i]]; i = parent[i]
        return i
    edges = collections.defaultdict(list)
    for i, f in enumerate(F):
        if not dr[i]: continue
        for x, y in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): edges[(min(x, y), max(x, y))].append(i)
    for v in edges.values():
        for j in v[1:]:
            if C[v[0]] == C[j]: parent[find(j)] = find(v[0])
    root = np.array([find(i) for i in range(len(F))]); ra = collections.Counter()
    for i in np.nonzero(dr)[0]: ra[root[i]] += area[i]
    neigh = collections.defaultdict(collections.Counter)
    for (x, y), v in edges.items():
        el = np.linalg.norm(V[x] - V[y])
        for i in v:
            for j in v:
                if root[i] != root[j]: neigh[root[i]][root[j]] += el
    g = lambda c: float(tl.TABLE[int(c)] @ tl.GREY)
    isl = np.zeros(len(F), bool); count = 0
    for r, ar in ra.items():
        if ar > .012 * tot: continue
        if neigh[r]:
            big = max(neigh[r].items(), key=lambda kv: kv[1])[0]
            if abs(g(C[r]) - g(C[big])) < 20: continue
        isl[root == r] = True; count += 1
    return isl & dr, count, area

def regions_for(npc):
    """Geometric regions on the REAL mesh (face centroid; x, height = -y, z)."""
    if npc == 7541:
        names = ['rings', 'helmet/spire', 'blade arm (-x)', 'hammer arm (+x)', 'chest/apron', 'leg -x', 'leg +x']
        def f(a):
            cen = a['V'][a['F']].mean(1); x, h = cen[:, 0], -cen[:, 1]
            lab = a['L'][a['F']]; ring = ((lab == 0) | (lab == 1)).all(1) | ((a['A'] > 0) & (h < 40) & (np.abs(a['V'][a['F']][:, :, 0]).max(1) > 150))
            r = np.full(len(x), 4)
            r[h < 330] = np.where(x[h < 330] < 0, 5, 6)
            arm = (np.abs(x) > 105) & (h > 120) & (h < 520); r[arm & (x < 0)] = 2; r[arm & (x > 0)] = 3
            r[h > 545] = 1; r[ring] = 0
            return r
        return names, f
    if npc == 11278:
        names = ['head/neck', 'arm+feathers -x', 'arm+feathers +x', 'trunk/plates/tabard', 'legs/talons']
        def f(a):
            cen = a['V'][a['F']].mean(1); x, h = cen[:, 0], -cen[:, 1]
            r = np.full(len(x), 3); r[h < 95] = 4; r[(np.abs(x) > 38) & (h > 110)] = 1; r[(x > 38) & (h > 110)] = 2; r[(h > 255) & (np.abs(x) <= 38)] = 0
            return r
        return names, f
    names = ['upper half', 'lower half']
    def f(a):
        cen = a['V'][a['F']].mean(1); h = -cen[:, 1]; mid = (h.max() + h.min()) / 2
        return np.where(h > mid, 0, 1)
    return names, f

def shift_min(a, b, k):
    """Per pixel: min over |dx|,|dy| <= k of |a(p) - b(p + d)| (nan in b = not available)."""
    best = np.full(a.shape, np.inf); H, W = a.shape
    bp = np.full((H + 2 * k, W + 2 * k), np.nan); bp[k:k + H, k:k + W] = b
    for dy in range(-k, k + 1):
        for dx in range(-k, k + 1):
            d = np.abs(a - bp[k + dy:k + dy + H, k + dx:k + dx + W]); d[np.isnan(d)] = np.inf
            best = np.minimum(best, d)
    return best
def aligned(a, b, k):
    d0 = np.abs(a - b); d = np.maximum(shift_min(a, b, k), shift_min(b, a, k))
    return np.where(np.isfinite(d), np.minimum(d, d0), d0)

def run(name, npc, ours_json, per=6, pictures=True, rows=None, quiet=False):
    mr, kv = tl.real_mesh(npc); dr = tl.real_def(npc); mo, jo = tl.ours_mesh(ours_json)
    seqs = L.sequences(npc); cam = tl.camera(npc); rows = rows or L.sample_rows(seqs, per)
    ar, ao = tl.arrays(mr), tl.arrays(mo)
    if dr['recolor']: raise SystemExit('recoloured NPC: not handled')
    isl_r, nr, _ = islands(ar); isl_o, no, _ = islands(ao)
    rnames, rf = regions_for(npc); reg_r = rf(ar)
    ids_r = tl.id_variants(f'real-{npc}', mr, dr); ids_o = tl.id_variants(f'ours-{name}', mo, jo)
    ents = [L.entry_real(npc, seqs), tl.entry(ours_json, seqs)] + [tl.entry(p, seqs) for p in ids_r + ids_o]
    lo, hi, _, _ = L.size_after_scale(npc); upp = cam['zoom'] / 512.0      # rough units per pixel at the aim distance
    upp = float((hi - lo).max()) * 1.7 / cam['cell'][0]; K = 2      # pixels of the gate picture (every creature fills the same cell)
    out = dict(name=name, npc=npc, rows=rows, K=K, units_per_pixel=upp, islands_real=nr, islands_ours=no, regions=rnames, cells=[])
    picdir = tl.RENDER / 'research-tone-decomp'; picdir.mkdir(parents=True, exist_ok=True)
    for yaw in (192, 1216):
        cells, text = tl.draw(ents, rows, cam, yaw)
        for row, cs in zip(rows, cells):
            real, ours, r0, r1, rl, o0, o1, ol = cs
            def faces(i0, i1, a):
                c0, c1 = tl.decode_codes(i0), tl.decode_codes(i1); f = np.where((c0 >= 0) & (c1 >= 0), c0 + tl.NCODE * c1, -1); f[f >= len(a['F'])] = -1
                return f
            fr, fo = faces(r0, r1, ar), faces(o0, o1, ao)
            I = tl.mask(real) & tl.mask(ours); nI = int(I.sum())
            if nI == 0: continue
            act = np.abs(tl.grey(real) - tl.grey(ours))
            lnr, lno = tl.decode_light(rl), tl.decode_light(ol)
            okf = (fr >= 0) & (fo >= 0) & np.isfinite(lnr) & np.isfinite(lno)
            trans = np.zeros(I.shape, bool); trans[fr >= 0] |= ar['A'][fr[fr >= 0]] > 0; trans[fo >= 0] |= ao['A'][fo[fo >= 0]] > 0
            Dm = I & (trans | ~okf); S = I & ~Dm
            pr = np.where(fr >= 0, ar['C'][np.clip(fr, 0, None)], -1); po = np.where(fo >= 0, ao['C'][np.clip(fo, 0, None)], -1)
            Grr, Goo, Gor, Gro = tl.synth(pr, lnr), tl.synth(po, lno), tl.synth(po, lnr), tl.synth(pr, lno)
            for G in (Grr, Goo, Gor, Gro): G[~S] = np.nan
            tot_model = np.abs(Goo - Grr)
            pa1, pa2 = np.abs(Gor - Grr), np.abs(Goo - Gro); la1, la2 = np.abs(Gro - Grr), np.abs(Goo - Gor)
            paA1, paA2 = aligned(Gor, Grr, K), aligned(Goo, Gro, K); laA1, laA2 = aligned(Gro, Grr, K), aligned(Goo, Gor, K)
            pa, la, paA, laA = (pa1 + pa2) / 2, (la1 + la2) / 2, (paA1 + paA2) / 2, (laA1 + laA2) / 2
            totA = aligned(Goo, Grr, K)
            ir = np.zeros(I.shape, bool); ir[fr >= 0] = isl_r[fr[fr >= 0]]; io = np.zeros(I.shape, bool); io[fo >= 0] = isl_o[fo[fo >= 0]]
            regpix = np.full(I.shape, -1); regpix[fr >= 0] = reg_r[fr[fr >= 0]]
            def stats(M):
                n = int(M.sum()); Ms = M & S; ns = int(Ms.sum())
                if n == 0: return None
                s = lambda x, m=Ms: float(np.nansum(x[m]))
                d = dict(pixels=n, solid=ns, tone=float(act[M].sum() / n), contrib=float(act[M].sum() / nI),
                         contrib_D=float(act[M & Dm].sum() / nI), contrib_S=float(act[Ms].sum() / nI))
                if ns:
                    d.update(tone_D=float(act[M & Dm].mean()) if (M & Dm).any() else 0.0, tone_S=float(act[Ms].mean()), model_S=s(tot_model) / ns, model_S_aligned=s(totA) / ns,
                             paint_alone=s(pa) / ns, light_alone=s(la) / ns, paint_real_light=s(pa1) / ns, light_real_paint=s(la1) / ns,
                             paint_aligned=s(paA) / ns, light_aligned=s(laA) / ns,
                             paint_al_real_island=s(paA, Ms & ir) / ns, paint_al_our_island=s(paA, Ms & io & ~ir) / ns, paint_al_broad=s(paA, Ms & ~ir & ~io) / ns,
                             ln_absdiff=float(np.abs(lno - lnr)[Ms].mean()), island_px_real=int((Ms & ir).sum()), island_px_ours=int((Ms & io & ~ir).sum()),
                             paint_differs_px=float((pr != po)[Ms].mean()))
                    parts = dict(f_misalign=max(d['paint_alone'] - d['paint_aligned'], 0) + max(d['light_alone'] - d['light_aligned'], 0),
                                 a_broad_paint=d['paint_al_broad'], c_detail_missing=d['paint_al_real_island'], c_detail_spurious=d['paint_al_our_island'], b_shape_light=d['light_aligned'])
                    tsum = sum(parts.values()) or 1.0
                    d['alloc'] = {k: v / tsum * d['contrib_S'] for k, v in parts.items()}; d['alloc']['d_translucent'] = d['contrib_D']
                else: d['alloc'] = dict(d_translucent=d['contrib_D'])
                return d
            cell = dict(row=row, yaw=yaw, gate_tone=float(act[I].mean()), all=stats(I), regions={rnames[k]: stats(I & (regpix == k)) for k in range(len(rnames))})
            cell['regions']['(unassigned)'] = stats(I & (regpix < 0))
            out['cells'].append(cell)
            if pictures and row == 'bind':
                def heat(x, scale=60):
                    v = np.clip(np.nan_to_num(x) / scale, 0, 1); im = np.zeros(x.shape + (3,), np.uint8); im[:] = (24, 24, 28)
                    im[..., 0] = np.where(S, 40 + 215 * v, 24); im[..., 1] = np.where(S, 40 + 150 * v * v, 24); im[..., 2] = np.where(S, 40, 28)
                    im[Dm] = (40, 60, 110); return im
                cls = np.zeros(I.shape + (3,), np.uint8); cls[:] = (24, 24, 28); cls[S] = (70, 70, 70); cls[S & ir] = (255, 200, 0); cls[S & io & ~ir] = (255, 0, 200); cls[Dm] = (40, 60, 110)
                un = (tl.mask(real) ^ tl.mask(ours)); cls[un] = (0, 160, 0)
                def sy(G):
                    im = np.zeros(I.shape + (3,), np.uint8); im[:] = tl.BG; g = np.clip(np.nan_to_num(G), 0, 255).astype(np.uint8)
                    for c in range(3): im[..., c] = np.where(S, g, im[..., c])
                    return im
                panels = [('real (game)', real), ('ours (game)', ours), ('|difference| (gate scores this)', heat(act)), ('PAINT alone: our paint, real light', heat(pa1)),
                          ('LIGHT alone: real paint, our light', heat(la1)), ('real paint under OUR light (grey)', sy(Gro)), ('our paint under REAL light (grey)', sy(Gor)),
                          ('classes: yellow real island, magenta ours only, blue translucent, green outline', cls)]
                ys, xs = np.nonzero(tl.mask(real) | tl.mask(ours)); y0, y1, x0, x1 = max(ys.min() - 6, 0), ys.max() + 7, max(xs.min() - 6, 0), xs.max() + 7; z = 3
                tiles = []
                for cap, im in panels:
                    t = Image.fromarray(im[y0:y1, x0:x1]).resize(((x1 - x0) * z, (y1 - y0) * z), Image.NEAREST); c = Image.new('RGB', (t.width, t.height + 18), (18, 22, 28)); c.paste(t, (0, 18))
                    ImageDraw.Draw(c).text((3, 3), cap[:int(t.width / 6.2)], fill=(235, 235, 235)); tiles.append(c)
                W = sum(t.width for t in tiles[:4]) + 12; Hh = tiles[0].height; board = Image.new('RGB', (W, Hh * 2 + 4), (18, 22, 28))
                for i, t in enumerate(tiles): board.paste(t, ((i % 4) * (tiles[0].width + 4), (i // 4) * (Hh + 4)))
                board.save(picdir / f'{name}-bind-yaw{yaw}.png')
    (tl.WORK / 'decomp').mkdir(exist_ok=True); (tl.WORK / 'decomp' / f'{name}.json').write_text(json.dumps(out, indent=1))
    if not quiet: report(out)
    return out

KEYS = ['a_broad_paint', 'c_detail_missing', 'c_detail_spurious', 'b_shape_light', 'f_misalign', 'd_translucent']
def agg(cells, pick):
    """Mean over cells of each statistic (cells weigh equally, as in the gate)."""
    ds = [pick(c) for c in cells]; ds = [d for d in ds if d]
    if not ds: return None
    o = {}
    for k in ds[0]:
        if k == 'alloc': o[k] = {q: float(np.mean([d['alloc'].get(q, 0.0) for d in ds])) for q in KEYS}
        else: o[k] = float(np.mean([d.get(k, 0.0) for d in ds]))
    return o
def report(out):
    P = print; cells = out['cells']
    P(f"\n=== {out['name']} (NPC {out['npc']})  K = {out['K']} px alignment window, {out['units_per_pixel']:.2f} units per pixel; small contrasting islands real {out['islands_real']}, ours {out['islands_ours']}")
    groups = [('bind front', lambda c: c['row'] == 'bind' and c['yaw'] == 192), ('bind rear', lambda c: c['row'] == 'bind' and c['yaw'] == 1216), ('bind', lambda c: c['row'] == 'bind')]
    for s in ('stand', 'walk', 'attack', 'death'): groups.append((s, lambda c, s=s: c['row'].startswith(s + ':')))
    P('ALLOCATION of the gate tone (grey levels; rows add to the gate number)')
    P(f"{'':12s} {'gate':>6s} | {'a paint':>8s} {'c detail':>8s} {'c spur.':>8s} {'b light':>8s} {'f align':>8s} {'d transl':>8s} | share of pixels translucent")
    for gname, sel in groups:
        cs = [c for c in cells if sel(c)]
        if not cs: continue
        a = agg(cs, lambda c: c['all']); al = a['alloc']
        P(f"{gname:12s} {np.mean([c['gate_tone'] for c in cs]):6.2f} | " + ' '.join(f'{al[k]:8.2f}' for k in KEYS) + f" | {1 - a['solid'] / a['pixels']:.2f}")
    P('CAUSES ALONE on solid pixels (grey levels over solid pixels; they do not add)')
    P(f"{'':12s} {'actual':>7s} {'model':>6s} {'paint':>6s} {'light':>6s} {'p.algn':>6s} {'l.algn':>6s} {'tot.algn':>8s} | paint@realL light@realP | LN diff | paint differs px")
    for gname, sel in groups:
        cs = [c for c in cells if sel(c)]
        if not cs: continue
        a = agg(cs, lambda c: c['all'])
        if 'tone_S' not in a:  # no solid (opaque) pixels in this group, e.g. a wholly translucent boss (Eldric); 2026-11-28
            P(f"{gname:12s} no solid pixels to decompose; translucent px tone {a.get('tone_D', a.get('tone', 0)):.2f}"); continue
        P(f"{gname:12s} {a['tone_S']:7.2f} {a['model_S']:6.2f} {a['paint_alone']:6.2f} {a['light_alone']:6.2f} {a['paint_aligned']:6.2f} {a['light_aligned']:6.2f} {a['model_S_aligned']:8.2f} | {a['paint_real_light']:6.2f} {a['light_real_paint']:6.2f} | {a['ln_absdiff']:5.1f} | {a['paint_differs_px']:.2f}   translucent px tone {a.get('tone_D', 0):.2f}")
    P('BY REGION, bind (both views): local tone, contribution to the gate number, allocation')
    P(f"{'region':22s} {'px':>6s} {'local':>6s} {'contrib':>7s} | " + ' '.join(f'{k[:8]:>8s}' for k in KEYS) + ' | paint alone, light alone (local, solid)')
    for rn in out['regions'] + ['(unassigned)']:
        cs = [c for c in cells if c['row'] == 'bind']; a = agg(cs, lambda c: c['regions'].get(rn))
        if not a: continue
        P(f"{rn:22s} {a['pixels']:6.0f} {a['tone']:6.2f} {a['contrib']:7.2f} | " + ' '.join(f"{a['alloc'][k]:8.2f}" for k in KEYS) + f" | {a.get('paint_alone', 0):6.2f} {a.get('light_alone', 0):6.2f}")
    P('BY REGION, all motion cells')
    for rn in out['regions'] + ['(unassigned)']:
        cs = [c for c in cells if c['row'] != 'bind']; a = agg(cs, lambda c: c['regions'].get(rn))
        if not a: continue
        P(f"{rn:22s} {a['pixels']:6.0f} {a['tone']:6.2f} {a['contrib']:7.2f} | " + ' '.join(f"{a['alloc'][k]:8.2f}" for k in KEYS) + f" | {a.get('paint_alone', 0):6.2f} {a.get('light_alone', 0):6.2f}")

if __name__ == '__main__':
    a = sys.argv; per = int(a[a.index('--per') + 1]) if '--per' in a else 6
    for nm in [x for x in a[1:] if not x.startswith('--') and not x.isdigit()]:
        if '=' in nm:
            nm, rest = nm.split('=', 1); npc, path = rest.split(':', 1); run(nm, int(npc), path, per)
        else:
            b = {**tl.BOSSES, **tl.MIDS}[nm]; run(nm, b['npc'], b['ours'], per)
