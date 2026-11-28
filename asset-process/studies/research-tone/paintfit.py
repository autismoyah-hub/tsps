"""Paint-fit: paint an author's OWN mesh from the boss sheet (version 2). Author-side: reads no real mesh.

usage: paintfit.py DEF.json SHEET_DIR OUT_DIR [--mode paint|lit|both] [--source both|pose|ortho|bind] [--splits N]
                   [--alpha] [--fill] [--views all|six] [--stride N] [--name NAME]

Pass 1 (paint). The sheet's pictures are projected onto the mesh WITH VISIBILITY, two ways:
  ortho  each of the sheet's orthographic rest views: the mesh is drawn in the view's own frame with a depth buffer and
         the client's winding cull (ortho.py), so each sheet pixel lands on the face that is in front there;
  pose   each cell of the sheet's pose set (rest pose and frames of stand, walk, attack, death from eight directions):
         the mesh is drawn by the client's harness under the same camera with every face in a code colour, so each
         sheet pixel lands on the face the GAME would show there, in that frame (poseset.py);
  ('bind' = pose cells of the rest pose only, plus ortho.)
  Opaque faces read the sheet's opaque paint layer, translucent faces its translucent layer. Each pixel votes for its
  sheet colour, weighted by the surface it stands for. Conflicts between views are resolved by the vote; the report
  gives each face's winning share and lists the faces NO view sees.
  --mode paint  the winning PAINT colour (what the real one is painted, wherever the light falls);
  --mode lit    the palette colour whose LIT result under THIS mesh's own light is nearest the sheet's lit picture over
                the face's pixels (paint compensating for wrong light: tone right, paint wrong);
  --mode both   as lit, but only among colours of the winning paint's hue and saturation (default).
Pass 2 (lit, splits). Where one face covers two tones (the second colour has >= 25% of the face's vote and the two differ
  by >= 12 grey levels when lit), a cut is proposed: the straight line that best separates the two colours in the
  face's best orthographic view, as points on two of its edges. --splits N applies the N most valuable cuts (each
  edge cut also cuts the neighbour across that edge, so the skin stays welded: about four faces per cut), then pass 1
  runs again.
--alpha  set each translucent face's alpha to the sheet's opacity vote.
--fill   faces no view sees take the colour of their seen neighbours instead of keeping their own.
Writes OUT_DIR/NAME.bin, NAME.json, NAME-paintfit.json (per-face report) and prints a summary.
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections
import numpy as np
import tl, ortho, poseset
from sheet_boss import Sheet
from oracle import vertex_ln

class Acc:
    def __init__(self, a, sheet, ambient, contrast):
        self.a = a; self.pal = np.array(sorted(int(c) for c in sheet.info['palette_rgb'])); P = len(self.pal); nf = len(a['F']); self.nf = nf
        self.lut = np.full(65536, -1); self.lut[self.pal] = np.arange(P)
        self.votes = np.zeros((nf, P)); self.pix = np.zeros(nf); self.cost = np.zeros((nf, P)); self.wsum = np.zeros(nf); self.opv = np.zeros(nf); self.opw = np.zeros(nf)
        self.best_view = np.full(nf, -1); self.best_px = np.zeros(nf); self.views = []; self.store = {}
        self.ln_v = vertex_ln(a, ambient, contrast); self.cl = np.nan_to_num(self.ln_v[a['F']], nan=64 + ambient); self.ambient = ambient
        self.T = tl.TABLE.astype(np.float32); self.G = tl.GREY.astype(np.float32)
    def vote(self, f, col, w):
        ci = self.lut[col]; ok = ci >= 0; np.add.at(self.votes, (f[ok], ci[ok]), w[ok] if np.ndim(w) else w); self.pix += np.bincount(f, minlength=self.nf)
    def opacity(self, f, opac, w): np.add.at(self.opv, f, w * opac); np.add.at(self.opw, f, w)
    def lit(self, f, ln, lit_rgb, w):
        pal = self.pal; lit_rgb = lit_rgb.astype(np.float32); w = np.broadcast_to(np.asarray(w, np.float32), f.shape)
        for s0 in range(0, len(f), 150000):
            sl = slice(s0, s0 + 150000); l = np.clip((((pal & 127)[None, :] * ln[sl, None]).astype(np.int64)) >> 7, 2, 126)
            d = (np.abs(self.T[(pal & ~127)[None, :] | l] - lit_rgb[sl, None, :]) @ self.G) * w[sl, None]
            np.add.at(self.cost, f[sl], d)
        np.add.at(self.wsum, f, w)

def feed_ortho(acc, sheet, views, stride=1, mode='both', keep=False):
    a = acc.a; px = sheet.px / stride; dr = tl.drawn(a); op = np.nonzero(dr & (a['A'] == 0))[0]; tr = np.nonzero(dr & (a['A'] > 0))[0]; bias = a['D'].astype(float) * 0.5
    sub = lambda img, shape: img[::stride, ::stride][:shape[0], :shape[1]]
    for name in views:
        ox, oy, w, h = sheet.frame(name); fr = (ox, oy, -(-w // stride), -(-h // stride)); c = sheet.toward(name)
        ido, bo, zo = ortho.raster(a['V'], a['F'], c, fr, px, op, bias)
        if len(tr): idt, bt, zt = ortho.raster(a['V'], a['F'], c, fr, px, tr, bias); idt[zt <= zo] = -1
        else: idt = np.full_like(ido, -1)
        fc = np.clip(ortho.facing(a['V'], a['F'], c), 0, 1); vi = len(acc.views); acc.views.append(name); unit = 1.0 / (px * px)
        col, _ = sheet.paint(name, 'paint'); col = sub(col, ido.shape); M = (ido >= 0) & (col >= 0)
        if M.any():
            f = ido[M]; w = (fc[f] + 0.05) * unit; acc.vote(f, col[M], w)
            cnt = np.bincount(f, minlength=acc.nf); better = cnt > acc.best_px; acc.best_view[better] = vi; acc.best_px[better] = cnt[better]
            if mode != 'paint':
                lit = sub(sheet.img(name, 'lit'), ido.shape)[M][:, :3]; acc.lit(f, ortho.interp(ido, bo, acc.cl)[M], lit, w)
        tcol, opac = sheet.paint(name, 'tpaint'); tcol = sub(tcol, ido.shape); opac = sub(opac, ido.shape); tcol2 = np.where(tcol >= 0, tcol, col); M = (idt >= 0) & (tcol2 >= 0)
        if M.any():
            f = idt[M]; w = (fc[f] + 0.05) * unit; acc.vote(f, tcol2[M], w); Mt = (idt >= 0) & (tcol >= 0)
            if Mt.any(): acc.opacity(idt[Mt], opac[Mt].astype(float), (fc[idt[Mt]] + 0.05) * unit)
        if keep: acc.store[name] = (ido, col)
def feed_pose(acc, sheet_dir, def_json, mode='both', bind_only=False, sub='posefit-ids', death_weight=0.05):
    a = acc.a; info = json.loads((pathlib.Path(sheet_dir) / 'pose.json').read_text())
    lo, hi = np.array(acc_bounds(a)); upp = 1.7 * float((hi - lo).max()) / info['cell'][0]; unit0 = upp * upp
    alpha = a['A']
    for c in poseset.cells(sheet_dir, def_json, sub, (lambda r: r == 'bind') if bind_only else None):
        unit = unit0 * (death_weight if c['row'].startswith('death') else 1.0)      # a dying boss flies apart: its frames place paint least reliably
        fO, fA = c['fO'], c['fA']; M = (fO >= 0) & (c['paint'] >= 0)
        if M.any():
            acc.vote(fO[M], c['paint'][M], unit)
            if mode != 'paint':
                Ml = M & (fA == fO) & (c['tpaint'] < 0) & np.isfinite(c['ln'])
                if Ml.any(): acc.lit(fO[Ml], c['ln'][Ml], c['lit'][Ml], unit)
        tr = (fA >= 0) & (alpha[np.clip(fA, 0, None)] > 0); col = np.where(c['tpaint'] >= 0, c['tpaint'], c['paint']); M = tr & (col >= 0)
        if M.any():
            acc.vote(fA[M], col[M], unit); Mt = tr & (c['tpaint'] >= 0)
            if Mt.any(): acc.opacity(fA[Mt], c['topac'][Mt].astype(float), unit)
def acc_bounds(a):
    used = np.unique(a['F'][tl.drawn(a)]); return a['V'][used].min(0), a['V'][used].max(0)

def decide(acc, mode):
    a = acc.a; nf = acc.nf; votes = acc.votes; tot = votes.sum(1); order = np.argsort(-votes, axis=1); top, sec = order[:, 0], order[:, 1]; pal = acc.pal
    seen = tot > 0; share = np.where(seen, votes[np.arange(nf), top] / np.maximum(tot, 1e-12), 0); share2 = np.where(seen, votes[np.arange(nf), sec] / np.maximum(tot, 1e-12), 0)
    colour = pal[top].copy()
    if mode != 'paint':
        cm = acc.cost / np.maximum(acc.wsum, 1e-12)[:, None]; ok = (acc.wsum > 0) & (a['A'] == 0)
        if mode == 'both': cm = np.where((pal >> 7)[None, :] == (pal[top] >> 7)[:, None], cm, np.inf)
        colour = np.where(ok, pal[cm.argmin(1)], colour)
    return dict(colour=colour, seen=seen, share=share, pixels=acc.pix, top=pal[top], second=pal[sec], second_share=share2,
                alpha_vote=np.where(acc.opw > 0, 255 - acc.opv / np.maximum(acc.opw, 1e-12), -1), best_view=acc.best_view, best_px=acc.best_px, views=acc.views, store=acc.store, ln_v=acc.ln_v)

def propose_splits(a, r, sheet, ambient, stride=1, min_px=14, min_share=0.25, min_grey=12):
    """Faces covering two tones -> list of dict(face, colours, gain, cut=[(edge (va, vb), t), (edge, t)])."""
    F, V = a['F'], a['V']; cl = np.nan_to_num(r['ln_v'][F], nan=64 + ambient).mean(1)
    def litgrey(c, ln): return float(tl.TABLE[(int(c) & ~127) | int(np.clip(((int(c) & 127) * ln) / 128, 2, 126))] @ tl.GREY)
    byview = collections.defaultdict(list)
    for i in np.nonzero(r['seen'] & (a['A'] == 0) & (r['second_share'] >= min_share) & (r['best_px'] >= min_px) & (r['best_view'] >= 0))[0]:
        c1, c2 = int(r['top'][i]), int(r['second'][i]); dg = abs(litgrey(c1, cl[i]) - litgrey(c2, cl[i]))
        if dg >= min_grey: byview[r['views'][r['best_view'][i]]].append((dg * r['second_share'][i] * r['best_px'][i], i, c1, c2, dg))
    out = []
    for name, lst in byview.items():
        if name not in r['store']: continue
        ids, col = r['store'][name]; c = sheet.toward(name); rt, dn, _ = ortho.basis(c); ox, oy, w, h = sheet.frame(name); px = sheet.px / stride
        for g, i, c1, c2, dg in lst:
            ys, xs = np.nonzero(ids == i); cc = col[ys, xs]; pts = np.stack([xs, ys], 1).astype(float); A, Bp = pts[cc == c1], pts[cc == c2]
            if len(A) < 3 or len(Bp) < 3: continue
            nrm = Bp.mean(0) - A.mean(0)
            if np.linalg.norm(nrm) < 1.0: continue
            nrm /= np.linalg.norm(nrm); sa, sb = A @ nrm, Bp @ nrm
            cand = np.unique(np.concatenate([sa, sb])); err = [(sa > t).sum() + (sb <= t).sum() for t in cand]; k = int(np.argmin(err)); thr = cand[k] + 0.5
            if err[k] > 0.3 * min(len(sa), len(sb)): continue                  # not separable by one straight line: left to the author
            tri = np.stack([(V[F[i]] @ rt - ox) * px, (V[F[i]] @ dn - oy) * px], 1); s = tri @ nrm - thr; cut = []
            for e in range(3):
                p, q = e, (e + 1) % 3
                if (s[p] > 0) != (s[q] > 0):
                    t = s[p] / (s[p] - s[q])
                    if 0.12 < t < 0.88: cut.append(((int(F[i][p]), int(F[i][q])), float(t)))
            if len(cut) == 2: out.append(dict(face=int(i), colours=[c1, c2], lit_grey_step=round(dg, 1), gain=float(g), view=name, cut=cut))
    out.sort(key=lambda d: -d['gain']); return out

def apply_splits(m, splits, limit):
    """Cut edges (a new welded vertex on each; every face on a cut edge is divided). -> new meshio dict, cuts applied."""
    m = tl.clone(m); V = [tuple(v) for v in m['vertices']]; done = 0; edge_new = {}; pos = set(V); touched = set()
    per_face_keys = [k for k in ('colors', 'priorities', 'render_types', 'alpha', 'depth', 'face_skins') if m.get(k)]
    for sp in splits:
        if done >= limit: break
        keys = [tuple(sorted(e)) for e, t in sp['cut']]
        if any(k in edge_new for k in keys) or any(v in touched for e, t in sp['cut'] for v in e): continue
        pts = [tuple(int(x) for x in np.rint(np.array(V[e[0]], float) + (np.array(V[e[1]], float) - np.array(V[e[0]], float)) * t)) for e, t in sp['cut']]
        if any(p in pos for p in pts) or pts[0] == pts[1]: continue
        for (e, t), k, p in zip(sp['cut'], keys, pts):
            V.append(p); pos.add(p); m['vertex_skins'].append(m['vertex_skins'][e[0] if t < 0.5 else e[1]]); edge_new[k] = len(V) - 1
        touched.update(v for e, t in sp['cut'] for v in e); done += 1
    out = {k: [] for k in per_face_keys}; nfaces = []
    for i, f in enumerate(m['faces']):
        ring = []
        for e in range(3):
            p, q = f[e], f[(e + 1) % 3]; ring.append((p, False)); k = tuple(sorted((p, q)))
            if k in edge_new: ring.append((edge_new[k], True))
        if len(ring) == 3: tris = [tuple(f)]
        elif len(ring) == 4:
            j = [x for x, (v, new) in enumerate(ring) if new][0]; r = [ring[(j + x) % 4][0] for x in range(4)]; tris = [(r[0], r[1], r[2]), (r[0], r[2], r[3])]
        elif len(ring) == 5:
            news = [x for x, (v, new) in enumerate(ring) if new]; j = news[0] if (news[1] - news[0]) == 2 else news[1]; r = [ring[(j + x) % 5][0] for x in range(5)]
            tris = [(r[0], r[1], r[2]), (r[0], r[2], r[3]), (r[0], r[3], r[4])]       # corner between the two cut points, then the quad
        else:
            r = [v for v, new in ring]; tris = [(r[0], r[x], r[x + 1]) for x in range(1, len(r) - 1)]
        for t in tris:
            nfaces.append(tuple(t))
            for k in per_face_keys: out[k].append(m[k][i])
    m['vertices'] = V; m['faces'] = nfaces
    for k in per_face_keys: m[k] = out[k]
    return m, done

def run(def_json, sheet_dir, out_dir, mode='both', splits=0, alpha=False, fill=False, views='all', stride=1, name=None, source='both', quiet=False, death_weight=0.05):
    sheet = Sheet(sheet_dir); m, j = tl.ours_mesh(def_json); amb, con = j.get('ambient', 0), j.get('contrast', 0)
    vs = [v for v in sheet.views] if views == 'all' else [v for v, _ in ortho.SIX]
    name = name or (pathlib.Path(def_json).stem + '-paintfit'); out_dir = pathlib.Path(out_dir); out_dir.mkdir(parents=True, exist_ok=True)
    def one(m, src_json, keep):
        a = tl.arrays(m); acc = Acc(a, sheet, amb, con)
        if source in ('both', 'ortho', 'bind'): feed_ortho(acc, sheet, vs, stride, mode, keep)
        if source in ('both', 'pose', 'bind'): feed_pose(acc, sheet_dir, src_json, mode, bind_only=(source == 'bind'), death_weight=death_weight)
        return a, decide(acc, mode)
    a, r = one(m, def_json, True); proposals = propose_splits(a, r, sheet, amb, stride); applied = 0
    if splits and proposals:
        m, applied = apply_splits(m, proposals, splits); tmp = tl.write_variant(name + '-cut', m, j, 'paintfit-tmp'); a, r = one(m, tmp, False)
    dr = tl.drawn(a); area = np.linalg.norm(np.cross(a['V'][a['F'][:, 1]] - a['V'][a['F'][:, 0]], a['V'][a['F'][:, 2]] - a['V'][a['F'][:, 0]]), axis=1) / 2
    old = a['C'].copy(); new = old.copy(); s = r['seen'] & dr; new[s] = r['colour'][s]; unseen = dr & ~r['seen']; unseen0 = unseen.copy()
    if fill and unseen.any():
        edges = collections.defaultdict(list)
        for i, f in enumerate(a['F']):
            for x, y in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): edges[(min(x, y), max(x, y))].append(i)
        nb = collections.defaultdict(set)
        for v in edges.values():
            for p in v: nb[p].update(q for q in v if q != p)
        for _ in range(6):
            for i in np.nonzero(unseen)[0]:
                cs = [int(new[q]) for q in nb[i] if not unseen[q] and dr[q]]
                if cs: new[i] = collections.Counter(cs).most_common(1)[0][0]; unseen[i] = False
    m['colors'] = [int(c) for c in new]
    if alpha: m['alpha'] = [int(np.clip(round(v / 5.0) * 5, 1, 254)) if (0 < al < 255 and v >= 0) else int(al) for al, v in zip(a['A'], r['alpha_vote'])]
    tl.meshio.save(m, out_dir / f'{name}.bin'); jj = dict(j); jj['models'] = [f'{name}.bin']; (out_dir / f'{name}.json').write_text(json.dumps(jj, indent=1))
    tot = area[dr].sum()
    rep = dict(mode=mode, source=source, views=vs, stride=stride, faces=int(dr.sum()), faces_seen=int(s.sum()), faces_unseen=int(unseen0.sum()), unseen_area_share=float(area[unseen0].sum() / tot),
               repainted=int((new != old)[dr].sum()), repainted_area_share=float(area[dr & (new != old)].sum() / tot),
               contested_faces=int((s & (r['share'] < 0.6)).sum()), contested_area_share=float(area[s & (r['share'] < 0.6)].sum() / tot),
               mean_winning_share=float(np.average(r['share'][s], weights=np.maximum(area[s], 1e-9))), split_proposals=len(proposals), splits_applied=applied,
               unseen_faces=[int(i) for i in np.nonzero(unseen0)[0]], proposals=proposals[:400],
               per_face=[dict(face=int(i), colour=int(new[i]), winning_share=round(float(r['share'][i]), 3), second=int(r['second'][i]), second_share=round(float(r['second_share'][i]), 3), pixels=int(r['pixels'][i])) for i in np.nonzero(s)[0]])
    (out_dir / f'{name}-paintfit.json').write_text(json.dumps(rep, indent=0))
    if not quiet:
        print(f"{name}: mode {mode}, source {source}, {len(vs)} ortho views, stride {stride}: {rep['faces']} drawn faces, {rep['faces_seen']} seen, {rep['faces_unseen']} seen by NO view ({100 * rep['unseen_area_share']:.1f}% of surface); "
              f"repainted {rep['repainted']} ({100 * rep['repainted_area_share']:.1f}% of surface); contested (winner < 60% of vote) {rep['contested_faces']} ({100 * rep['contested_area_share']:.1f}%); "
              f"mean winning share {rep['mean_winning_share']:.2f}; faces covering two tones: {rep['split_proposals']} cuts proposed, {applied} applied", flush=True)
    return out_dir / f'{name}.json', rep

if __name__ == '__main__':
    g = sys.argv
    def opt(n, d=None): return g[g.index(n) + 1] if n in g else d
    if len(g) < 4: raise SystemExit(__doc__)
    run(g[1], g[2], g[3], opt('--mode', 'both'), int(opt('--splits', 0)), '--alpha' in g, '--fill' in g, opt('--views', 'all'), int(opt('--stride', 1)), opt('--name'), opt('--source', 'both'), False, float(opt('--death-weight', 0.05)))
