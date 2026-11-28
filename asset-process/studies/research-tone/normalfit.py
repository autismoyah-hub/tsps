"""Normal aid and normal fit: where an author's mesh is LIT differently from the real boss for reasons of SHAPE, and a
re-split / reshape that removes it. Author-side: reads the author's mesh and the sheet (version 2), no real mesh.

usage: normalfit.py aid DEF.json SHEET_DIR OUT_PREFIX
       normalfit.py fit DEF.json SHEET_DIR OUT_DIR [--name NAME] [--radius R] [--sweeps N] [--no-flips] [--no-moves]
                    [--labels A,B,..] [--box x0,x1,h0,h1,z0,z1] [--stiff MU]

The light of a creature is decided once, at rest, per VERTEX: light number = ambient' + (256 / D) x the mean, over the
smooth faces using the vertex, of (face unit normal . light direction)  (LIGHTING.md; light_npc.py is the exact port).
The sheet's light pictures give that number for the real boss at every visible point of the rest pose. So:
  TARGET  each vertex of the author's mesh gets the sheet's light number around it: every sheet pixel that falls on one
          of the vertex's faces (drawn with visibility in the sheet's own orthographic frames) contributes with the
          weight the vertex has at that pixel. Paint plays no part: this is light from shape alone.
  aid     writes OUT_PREFIX-VIEW.png for the six axis views: the sheet's light | the mesh's light (port) | the
          difference (red = the mesh is lit brighter than the real one, blue = darker; one step = one light number,
          about paint lightness / 64 grey levels), and prints the error by vertex label, worst first.
  fit     greedy descent on  sum_v W_v (light_v - target_v)^2 + MU x sum_v |moved|^2  with two kinds of step:
          FLIP an inner diagonal (re-split: no vertex moves, the outline cannot change), and MOVE a vertex by one unit
          (at most R units from where the author put it). Faces may not turn over. Paint is untouched: run paintfit after.
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections, math
import numpy as np
from PIL import Image, ImageDraw
import tl, ortho
from sheet_boss import Sheet
DIR = np.array([-30.0, -50.0, -30.0])
def consts(ambient, contrast): return 64 + ambient, 256.0 / ((((850 + contrast * 5) * 65) >> 8))
def face_s(P):
    """P [k,3,3] -> unit normal . light direction (0 for a degenerate face), and the raw normals"""
    n = np.cross(P[:, 1] - P[:, 0], P[:, 2] - P[:, 0]); ln = np.linalg.norm(n, axis=1); ok = ln >= 1.0
    return np.where(ok, (n @ DIR) / np.maximum(ln, 1e-9), 0.0), n
def targets(a, sheet, views=None, ambient=0, contrast=0):
    """-> t [nv] target light number, W [nv] weight (surface the vertex answers for), per-view pictures for the aid."""
    views = views or list(sheet.views); V, F = a['V'], a['F']; nv = len(V); dr = tl.drawn(a); op = np.nonzero(dr & (a['A'] == 0))[0]
    num = np.zeros(nv); den = np.zeros(nv); px = sheet.px; pics = {}
    amb, k = consts(ambient, contrast); s, _ = face_s(V[F]); smooth = a['T'] == 0
    S = np.zeros(nv); cnt = np.zeros(nv); np.add.at(S, F[smooth].ravel(), np.repeat(s[smooth], 3)); np.add.at(cnt, F[smooth].ravel(), 1)
    ln_v = amb + k * S / np.maximum(cnt, 1); cl = ln_v[F]
    for name in views:
        fr = sheet.frame(name); c = sheet.toward(name); ids, ba, zb = ortho.raster(V, F, c, fr, px, op, a['D'] * 0.5)
        tgt = sheet.light(name); M = (ids >= 0) & np.isfinite(tgt)
        if name in ('front', 'back', 'left', 'right', 'top', 'bottom'): pics[name] = (tgt, ortho.interp(ids, ba, cl), M)
        if not M.any(): continue
        f = ids[M]; t = tgt[M]; fc = np.clip(ortho.facing(V, F, c), 0, 1)[f]; b0, b1 = ba[M][:, 0], ba[M][:, 1]; b2 = 1 - b0 - b1
        mine = b0 * cl[f, 0] + b1 * cl[f, 1] + b2 * cl[f, 2]; t = mine + np.clip(t - mine, -70, 70)          # a pixel that fell on another part of the real boss
        for j, b in enumerate((b0, b1, b2)):
            w = np.clip(b, 0, 1) * fc; np.add.at(num, F[f, j], w * t); np.add.at(den, F[f, j], w)
    t = np.where(den > 0, num / np.maximum(den, 1e-12), np.nan); W = den / (px * px)
    return t, W, ln_v, pics

class Fit:
    def __init__(self, m, ambient, contrast, t, W, radius=6, stiff=0.02, active=None):
        self.m = m; self.V = np.array(m['vertices'], float); self.V0 = self.V.copy(); self.F = [list(f) for f in m['faces']]; self.nv = len(self.V)
        self.amb, self.k = consts(ambient, contrast); self.t = np.nan_to_num(t); self.W = np.where(np.isfinite(t), W, 0.0); self.W = np.minimum(self.W / max(np.mean(self.W[self.W > 0]), 1e-9), 4.0)
        self.radius = radius; self.stiff = stiff; self.active = np.ones(self.nv, bool) if active is None else active
        self.free = outline_locks(self.V, np.array(self.F), np.array(m['alpha']))      # [nv,3] axes a vertex may move along
        self.smooth = [r == 0 for r in m['render_types']]
        self.vf = collections.defaultdict(set)
        for i, f in enumerate(self.F):
            for v in f: self.vf[v].add(i)
        self.s = np.zeros(len(self.F)); self.n0 = np.zeros((len(self.F), 3))
        s, n = face_s(self.V[np.array(self.F)]); self.s[:] = s; self.n0[:] = n / np.maximum(np.linalg.norm(n, axis=1), 1e-9)[:, None]
        self.S = np.zeros(self.nv); self.cnt = np.zeros(self.nv)
        for i, f in enumerate(self.F):
            if self.smooth[i]:
                for v in f: self.S[v] += self.s[i]; self.cnt[v] += 1
        self.moves = self.flips = 0
    def ln(self, idx=None):
        idx = slice(None) if idx is None else idx; return self.amb + self.k * self.S[idx] / np.maximum(self.cnt[idx], 1)
    def err(self): return self.W * (self.ln() - self.t) ** 2
    def report(self):
        d = np.abs(self.ln() - self.t)[self.W > 0]; w = self.W[self.W > 0]
        return dict(mean_abs=float(np.average(d, weights=w)), rms=float(np.sqrt(np.average(d * d, weights=w))), over8=float(np.average(d > 8, weights=w)))
    def try_move(self, v, delta):
        p = self.V[v] + delta
        if np.abs(p - self.V0[v]).max() > self.radius or (np.abs(delta) > 0)[~self.free[v]].any(): return 0.0, None
        fs = [i for i in self.vf[v]]; P = self.V[np.array([self.F[i] for i in fs])].copy()
        for r, i in enumerate(fs): P[r, self.F[i].index(v)] = p
        s2, n2 = face_s(P); ln2 = np.linalg.norm(n2, axis=1)
        if (ln2 < 2.0).any() or ((n2 * self.n0[fs]).sum(1) <= 0.15 * ln2).any(): return 0.0, None          # no face may collapse or turn over
        dS = collections.defaultdict(float)
        for r, i in enumerate(fs):
            if self.smooth[i]:
                for u in self.F[i]: dS[u] += s2[r] - self.s[i]
        us = np.array(list(dS)); d = np.array([dS[u] for u in us]); old = self.ln(us); new = old + self.k * d / np.maximum(self.cnt[us], 1)
        dE = float((self.W[us] * ((new - self.t[us]) ** 2 - (old - self.t[us]) ** 2)).sum()) + self.stiff * float(((p - self.V0[v]) ** 2).sum() - ((self.V[v] - self.V0[v]) ** 2).sum())
        return dE, (p, fs, s2, us, d)
    def do_move(self, v, pack):
        p, fs, s2, us, d = pack; self.V[v] = p
        for r, i in enumerate(fs): self.s[i] = s2[r]
        self.S[us] += d; self.moves += 1
    def edges(self):
        e = collections.defaultdict(list)
        for i, f in enumerate(self.F):
            for j in range(3): e[(f[j], f[(j + 1) % 3])].append(i)
        return e
    def sweep_flips(self):
        m = self.m; e = self.edges(); done = 0; dirty = set()
        for (a_, b_), fl in list(e.items()):
            if a_ > b_ or len(fl) != 1 or len(e.get((b_, a_), [])) != 1: continue
            f1, f2 = fl[0], e[(b_, a_)][0]
            if f1 in dirty or f2 in dirty or not (self.active[a_] and self.active[b_]): continue
            if any(m[k_][f1] != m[k_][f2] for k_ in ('alpha', 'render_types', 'face_skins', 'depth', 'priorities') if m.get(k_)): continue
            if not self.smooth[f1]: continue
            c_ = [v for v in self.F[f1] if v not in (a_, b_)]; d_ = [v for v in self.F[f2] if v not in (a_, b_)]
            if len(c_) != 1 or len(d_) != 1 or c_[0] == d_[0]: continue
            c_, d_ = c_[0], d_[0]
            if (c_, d_) in e or (d_, c_) in e or self.cnt[a_] <= 3 or self.cnt[b_] <= 3: continue
            P = self.V[np.array([[a_, d_, c_], [d_, b_, c_]])]; s2, n2 = face_s(P); ln2 = np.linalg.norm(n2, axis=1)
            base = self.n0[f1] + self.n0[f2]; base /= max(np.linalg.norm(base), 1e-9)
            if (ln2 < 2.0).any() or ((n2 @ base) <= 0.3 * ln2).any(): continue
            dS = {a_: s2[0] - self.s[f1] - self.s[f2], b_: s2[1] - self.s[f1] - self.s[f2], c_: s2[0] + s2[1] - self.s[f1], d_: s2[0] + s2[1] - self.s[f2]}; dc = {a_: -1, b_: -1, c_: 1, d_: 1}
            us = np.array(list(dS)); old = self.ln(us); new = self.amb + self.k * (self.S[us] + np.array([dS[u] for u in us])) / np.maximum(self.cnt[us] + np.array([dc[u] for u in us]), 1)
            dE = float((self.W[us] * ((new - self.t[us]) ** 2 - (old - self.t[us]) ** 2)).sum())
            if dE < -0.5:
                self.F[f1] = [a_, d_, c_]; self.F[f2] = [d_, b_, c_]; self.s[f1], self.s[f2] = s2
                for u in us: self.S[u] += dS[u]; self.cnt[u] += dc[u]
                self.vf[b_].discard(f1); self.vf[a_].discard(f2); self.vf[d_].add(f1); self.vf[c_].add(f2)
                nn = n2 / ln2[:, None]; self.n0[f1], self.n0[f2] = nn; dirty.update((f1, f2)); done += 1; self.flips += 1
        return done
    def sweep_moves(self):
        done = 0; order = np.argsort(-self.err())
        steps = [np.array(x, float) for x in ((1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1))]
        for v in order:
            if self.W[v] <= 0 and not any(self.W[u] > 0 for i in self.vf[v] for u in self.F[i]): continue
            if not self.active[v] or not self.vf[v]: continue
            best = (-0.05, None)
            for st in steps:
                dE, pack = self.try_move(v, st)
                if pack is not None and dE < best[0]: best = (dE, pack)
            if best[1] is not None: self.do_move(v, best[1]); done += 1
        return done
    def mesh(self):
        m = tl.clone(self.m); m['vertices'] = [tuple(int(x) for x in v) for v in self.V]; m['faces'] = [tuple(f) for f in self.F]; return m

def outline_locks(V, F, alpha, near=2.0):
    """A vertex lying on the OUTLINE of the mesh in the front, side or top view may not move across that view:
    it keeps only the axis along which that view looks. -> free [nv,3] bool."""
    import cv2
    free = np.ones((len(V), 3), bool); lo = V.min(0) - 4; Fd = F[alpha != 255]
    for look, (ax, ay) in ((2, (0, 1)), (0, (2, 1)), (1, (0, 2))):
        w, h = int(V[:, ax].max() - lo[ax]) + 9, int(V[:, ay].max() - lo[ay]) + 9; im = np.zeros((h, w), np.uint8)
        pts = np.stack([V[:, ax] - lo[ax], V[:, ay] - lo[ay]], 1)
        for t in np.round(pts[Fd] * 16).astype(np.int32): cv2.fillConvexPoly(im, t, 255, lineType=cv2.LINE_8, shift=4)
        dist = cv2.distanceTransform(im, cv2.DIST_L2, 3); xi = np.clip(np.rint(pts[:, 0]).astype(int), 0, w - 1); yi = np.clip(np.rint(pts[:, 1]).astype(int), 0, h - 1)
        on = dist[yi, xi] <= near
        for k in range(3):
            if k != look: free[on, k] = False
    return free
def active_mask(m, labels=None, box=None):
    V = np.array(m['vertices'], float); act = np.ones(len(V), bool)
    if labels is not None: act &= np.isin(np.array(m['vertex_skins']), list(labels))
    if box is not None: x0, x1, h0, h1, z0, z1 = box; act &= (V[:, 0] >= x0) & (V[:, 0] <= x1) & (-V[:, 1] >= h0) & (-V[:, 1] <= h1) & (V[:, 2] >= z0) & (V[:, 2] <= z1)
    return act
def fit(def_json, sheet_dir, out_dir, name=None, radius=6, sweeps=12, flips=True, moves=True, labels=None, box=None, stiff=0.02, quiet=False):
    sheet = Sheet(sheet_dir); m, j = tl.ours_mesh(def_json); a = tl.arrays(m); amb, con = j.get('ambient', 0), j.get('contrast', 0)
    t, W, ln_v, pics = targets(a, sheet, None, amb, con); fx = Fit(m, amb, con, t, W, radius, stiff, active_mask(m, labels, box)); before = fx.report()
    for sw in range(sweeps):
        nf_ = fx.sweep_flips() if flips else 0; nm = fx.sweep_moves() if moves else 0
        if not quiet: print(f'   sweep {sw + 1}: {nf_} flips, {nm} moves, light error {fx.report()["mean_abs"]:.2f}', flush=True)
        if nf_ + nm == 0: break
    out = fx.mesh(); pos = collections.Counter(out['vertices']); merged = sum(n for n in pos.values() if n > 1)
    name = name or (pathlib.Path(def_json).stem + '-normalfit'); out_dir = pathlib.Path(out_dir); out_dir.mkdir(parents=True, exist_ok=True)
    tl.meshio.save(out, out_dir / f'{name}.bin'); jj = dict(j); jj['models'] = [f'{name}.bin']; (out_dir / f'{name}.json').write_text(json.dumps(jj, indent=1))
    after = fx.report(); disp = np.abs(fx.V - fx.V0).max(1)
    rep = dict(before=before, after=after, flips=fx.flips, moves=fx.moves, vertices_moved=int((disp > 0).sum()), max_move=float(disp.max()), mean_move=float(disp[disp > 0].mean()) if (disp > 0).any() else 0.0, vertices_now_sharing_a_position=merged)
    (out_dir / f'{name}-normalfit.json').write_text(json.dumps(rep, indent=1))
    print(f"{name}: light number error (mean abs, weighted) {before['mean_abs']:.2f} -> {after['mean_abs']:.2f}; vertices more than 8 off {before['over8']:.2f} -> {after['over8']:.2f}; {fx.flips} flips, {rep['vertices_moved']} vertices moved (mean {rep['mean_move']:.1f}, max {rep['max_move']:.0f} units); shared positions {merged}", flush=True)
    return out_dir / f'{name}.json', rep

def aid(def_json, sheet_dir, prefix):
    sheet = Sheet(sheet_dir); m, j = tl.ours_mesh(def_json); a = tl.arrays(m); amb, con = j.get('ambient', 0), j.get('contrast', 0)
    t, W, ln_v, pics = targets(a, sheet, None, amb, con); prefix = pathlib.Path(prefix); prefix.parent.mkdir(parents=True, exist_ok=True)
    for name, (tgt, mine, M) in pics.items():
        d = np.where(M, np.nan_to_num(mine) - np.nan_to_num(tgt), 0.0); h, w = d.shape
        def greyim(x, ok):
            im = np.zeros((h, w, 3), np.uint8); im[:] = (24, 26, 32); g = np.clip(np.nan_to_num(x), 0, 255).astype(np.uint8)
            for c in range(3): im[..., c] = np.where(ok, g, im[..., c])
            return im
        dif = np.zeros((h, w, 3), np.uint8); dif[:] = (24, 26, 32); v = np.clip(np.abs(d) / 40, 0, 1)
        dif[M] = np.stack([np.where(d > 0, 60 + 195 * v, 60 - 30 * v), 60 - 30 * v, np.where(d < 0, 60 + 195 * v, 60 - 30 * v)], -1)[M].astype(np.uint8)
        strip = np.concatenate([greyim(tgt, np.isfinite(tgt)), greyim(mine, np.isfinite(mine)), dif], 1); im = Image.fromarray(strip); sc = min(1.0, 1500 / im.width)
        im = im.resize((int(im.width * sc), int(im.height * sc)), Image.LANCZOS); ImageDraw.Draw(im).text((4, 4), f'{name}: sheet light | this mesh | difference (red brighter, blue darker; full colour = 40 light numbers)', fill=(255, 255, 255))
        im.save(f'{prefix}-{name}.png')
    lab = np.array(m['vertex_skins']); err = np.abs(ln_v - t); ok = np.isfinite(t) & (W > 0); rows = []
    for l in np.unique(lab[ok]):
        s = ok & (lab == l); rows.append((float((err[s] * W[s]).sum()), int(l), int(s.sum()), float(np.average(err[s], weights=W[s])), float(np.average((ln_v - t)[s], weights=W[s])), float(W[s].sum())))
    rows.sort(reverse=True); tot = sum(r[0] for r in rows); allw = sum(r[5] for r in rows)
    print(f'{def_json}: light number error against the sheet, weighted by surface: mean abs {tot / allw:.2f} (one light number = paint lightness / 64 grey levels: about 0.5 on dark iron, 1 on mid tones)')
    print('label  vertices  mean abs  signed (+ = this mesh brighter)  share of all error')
    for e, l, n, ma, sg, w in rows[:25]: print(f'{l:5d} {n:8d} {ma:9.1f} {sg:+9.1f} {100 * e / tot:22.1f}%')
    return rows
if __name__ == '__main__':
    g = sys.argv
    def opt(n, d=None): return g[g.index(n) + 1] if n in g else d
    if len(g) < 5: raise SystemExit(__doc__)
    if g[1] == 'aid': aid(g[2], g[3], g[4])
    else:
        fit(g[2], g[3], g[4], opt('--name'), float(opt('--radius', 6)), int(opt('--sweeps', 12)), '--no-flips' not in g, '--no-moves' not in g,
            [int(x) for x in opt('--labels').split(',')] if opt('--labels') else None, [float(x) for x in opt('--box').split(',')] if opt('--box') else None, float(opt('--stiff', 0.02)))
