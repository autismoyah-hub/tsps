"""Boss reference sheet, version 2 (Teacher-side: reads the real mesh; the OUTPUT is what an author may see).

usage: sheet_boss.py NPC_ID OUTDIR [--px N] [--old OLD_SHEET.json]

Writes OUTDIR/sheet.json and, for each of 18 unmirrored orthographic views (6 axis views, 4 level diagonals, 4 from
above, 4 from below; ortho.VIEWS), in OUTDIR/views/:
  NAME-paint.png   front-most OPAQUE face in its PAINTED colour (exact colour-table RGB; alpha 0 = nothing there)
  NAME-lit.png     the same pixel as the game draws it at rest: the client's light (light_npc.py's rule), Gouraud
  NAME-light.png   the light number alone (grey level = light number; 128 = paint as painted)
  NAME-ink.png     white = paint boundary, blue = a depth step inside one paint, grey = outline
  NAME-tpaint.png  front-most TRANSLUCENT face lying in front of the opaque layer: its paint, PNG alpha = its opacity
  NAME-tlit.png    the same, lit
and boards for people: board-paint.png, board-lit.png, board-ink.png, and enlargements/ (the windows with the most
paint boundary in the six axis views, at twice the resolution, paint | lit | ink).

What it reveals beyond the first sheet: the lit tone and the light number at every visible point (an author can already
draw the real boss lit from any angle with npc.py; this only registers it to the paint map), twelve more views, a higher
resolution, which paint is translucent and how much. What it still hides: vertex positions, the triangulation, depth,
where faces are spent, hidden faces, how joints are built. Known leak, as before: where every face has its own
paint, the paint map outlines the faces.
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json
import numpy as np
from PIL import Image, ImageDraw
import tl, ortho
from oracle import vertex_ln

def corner_lit(a, ambient, contrast):
    ln = vertex_ln(a, ambient, contrast); cl = ln[a['F']]                       # [nf,3] light number at corners
    lit = np.clip(((a['C'] & 127)[:, None] * np.nan_to_num(cl)).astype(np.int64) >> 7, 2, 126).astype(float)
    return cl, lit
def layers(a, c, fr, px):
    """Opaque layer and the translucent layer in front of it: (ids, bary) each."""
    dr = tl.drawn(a); op = np.nonzero(dr & (a['A'] == 0))[0]; tr = np.nonzero(dr & (a['A'] > 0))[0]
    bias = a['D'].astype(float) * 0.5
    ido, bo, zo = ortho.raster(a['V'], a['F'], c, fr, px, op, bias)
    if len(tr):
        idt, bt, zt = ortho.raster(a['V'], a['F'], c, fr, px, tr, bias); idt[zt <= zo] = -1
    else: idt, bt = np.full_like(ido, -1), np.zeros_like(bo)
    return (ido, bo, zo), (idt, bt)
def pictures(a, cl, lit, c, fr, px):
    (ido, bo, zo), (idt, bt) = layers(a, c, fr, px); T = tl.TABLE; out = {}
    def rgba(ids, ba, lit_mode, opacity=None):
        ok = ids >= 0; col = a['C'][np.clip(ids, 0, None)]
        if lit_mode:
            l = np.clip(np.rint(np.nan_to_num(ortho.interp(ids, ba, lit))), 2, 126).astype(np.int64); col = (col & ~127) | l
        im = np.zeros(ids.shape + (4,), np.uint8); im[..., :3] = np.where(ok[..., None], T[col], 0)
        im[..., 3] = np.where(ok, 255 if opacity is None else opacity, 0); return im
    out['paint'] = rgba(ido, bo, False); out['lit'] = rgba(ido, bo, True)
    ln = ortho.interp(ido, bo, cl); L8 = np.zeros(ido.shape + (2,), np.uint8); L8[..., 0] = np.clip(np.rint(np.nan_to_num(ln)), 0, 255); L8[..., 1] = np.where(ido >= 0, 255, 0); out['light'] = L8
    opac = 255 - a['A'][np.clip(idt, 0, None)]
    out['tpaint'] = rgba(idt, bt, False, opac); out['tlit'] = rgba(idt, bt, True, opac)
    # ink
    ok = ido >= 0; col = np.where(ok, a['C'][np.clip(ido, 0, None)], -1); ink = np.zeros(ido.shape + (3,), np.uint8); ink[:] = (16, 18, 22); ink[ok] = (44, 48, 54)
    for ax in (0, 1):
        c1, c2 = (col[:, :-1], col[:, 1:]) if ax else (col[:-1], col[1:]); z1, z2 = (zo[:, :-1], zo[:, 1:]) if ax else (zo[:-1], zo[1:])
        both = (c1 >= 0) & (c2 >= 0); edge = both & (c1 != c2); step = both & (c1 == c2) & (np.abs(z1 - z2) > 6); sil = (c1 >= 0) != (c2 >= 0)
        tgt = ink[:, :-1] if ax else ink[:-1]
        tgt[sil] = (120, 120, 120); tgt[step] = (60, 110, 230); tgt[edge] = (255, 255, 255)
    out['ink'] = ink; out['_edge'] = (ink == 255).all(2); out['_ids'] = ido
    return out
def save(arr, path):
    mode = {2: 'LA', 3: 'RGB', 4: 'RGBA'}[arr.shape[2]]; Image.fromarray(arr, mode).save(path)

def build(npc, out, px=None, old=None):
    out = pathlib.Path(out); (out / 'views').mkdir(parents=True, exist_ok=True); (out / 'enlargements').mkdir(exist_ok=True)
    m, kv = tl.real_mesh(npc); a = tl.arrays(m); d = tl.real_def(npc)
    if d['recolor']: raise SystemExit('recoloured NPC: apply recolours first')
    dr = tl.drawn(a); used = np.unique(a['F'][dr]); lo, hi = a['V'][used].min(0), a['V'][used].max(0); ext = hi - lo
    px = px or float(np.clip(1900 / ext.max(), 2.0, 6.0))
    cl, lit = corner_lit(a, d['ambient'], d['contrast'])
    info = json.loads(pathlib.Path(old).read_text()) if old else {}
    info.update(sheet_version=2, npc_id=npc, pixels_per_unit=px, bounds_min=[float(x) for x in lo], bounds_max=[float(x) for x in hi], ambient=d['ambient'], contrast=d['contrast'],
                projection='pixel_x = (dot(p, right) - origin[0]) * pixels_per_unit; pixel_y = (dot(p, down) - origin[1]) * pixels_per_unit; p in cache coordinates (y DOWN); faces wound away from the viewer are not drawn',
                light_rule='lit lightness = clamp((paint lightness * light number) >> 7, 2, 126); light picture grey level = light number',
                palette_rgb={str(int(c)): [int(x) for x in tl.TABLE[int(c)]] for c in sorted(set(a['C'][dr].tolist()))}, views=[])
    rgbs = {}
    for c, rgb in info['palette_rgb'].items(): rgbs.setdefault(tuple(rgb), []).append(c)
    info['palette_rgb_collisions'] = [v for v in rgbs.values() if len(v) > 1]
    thumbs = {k: [] for k in ('paint', 'lit', 'ink')}; cand = []
    for name, c in ortho.VIEWS:
        fr = ortho.frame(lo, hi, c, px); pics = pictures(a, cl, lit, c, fr, px); r, dn, t = ortho.basis(c)
        for k in ('paint', 'lit', 'light', 'ink', 'tpaint', 'tlit'): save(pics[k], out / 'views' / f'{name}-{k}.png')
        info['views'].append(dict(name=name, toward_viewer=[float(x) for x in t], right=[float(x) for x in r], down=[float(x) for x in dn], origin=[fr[0], fr[1]], size=[fr[2], fr[3]]))
        for k in thumbs:
            im = pics[k] if k == 'ink' else None
            if im is None:
                base = np.zeros(pics[k].shape[:2] + (3,), np.uint8); base[:] = (30, 36, 46); al = pics[k][..., 3:] / 255.0; base = (base * (1 - al) + pics[k][..., :3] * al).astype(np.uint8)
                tp = pics['tpaint' if k == 'paint' else 'tlit']; al = tp[..., 3:] / 255.0; im = (base * (1 - al) + tp[..., :3] * al).astype(np.uint8)
            thumbs[k].append((name, Image.fromarray(im)))
        if name in ('front', 'back', 'left', 'right', 'top', 'bottom'): cand.append((name, c, fr, pics['_edge']))
        print(name, fr[2], 'x', fr[3], flush=True)
    # boards
    for k, lst in thumbs.items():
        sc = min(1.0, 430 / max(max(im.width, im.height) for _, im in lst)); tiles = []
        for name, im in lst:
            t = im.resize((max(1, int(im.width * sc)), max(1, int(im.height * sc))), Image.LANCZOS if k != 'ink' else Image.BOX); cnv = Image.new('RGB', (440, 456), (18, 22, 28)); cnv.paste(t, ((440 - t.width) // 2, 20 + (436 - t.height) // 2))
            ImageDraw.Draw(cnv).text((4, 4), f'{name}  ({k})', fill=(235, 235, 235)); tiles.append(cnv)
        board = Image.new('RGB', (440 * 6, 456 * 3), (18, 22, 28))
        for i, t in enumerate(tiles): board.paste(t, ((i % 6) * 440, (i // 6) * 456))
        board.save(out / f'board-{k}.png')
    # enlargements: per axis view, the two windows with the most paint boundary, at twice the resolution
    win_u = max(60.0, float(ext.max()) / 6); enl = []
    for name, c, fr, edge in cand:
        w = int(win_u * px); H, W = edge.shape
        if H < w or W < w: w = min(H, W)
        ii = np.pad(edge.astype(np.int64).cumsum(0).cumsum(1), ((1, 0), (1, 0))); score = ii[w:, w:] - ii[:-w, w:] - ii[w:, :-w] + ii[:-w, :-w]
        taken = []
        for _ in range(2):
            y, x = np.unravel_index(np.argmax(score), score.shape)
            if score[y, x] <= 0: break
            taken.append((x, y)); score[max(0, y - w):y + w, max(0, x - w):x + w] = -1
        for n, (x, y) in enumerate(taken):
            px2 = px * 2; fr2 = (fr[0] + x / px, fr[1] + y / px, int(w * 2), int(w * 2)); p2 = pictures(a, cl, lit, c, fr2, px2)
            strip = np.concatenate([np.where(p2['paint'][..., 3:] > 0, p2['paint'][..., :3], 30), np.where(p2['lit'][..., 3:] > 0, p2['lit'][..., :3], 30), p2['ink']], 1).astype(np.uint8)
            fn = f'{name}-{n}.png'; Image.fromarray(strip).save(out / 'enlargements' / fn)
            enl.append(dict(file=fn, view=name, origin=[fr2[0], fr2[1]], pixels_per_unit=px2, size=[fr2[2], fr2[3]], panels='paint | lit | ink'))
    info['enlargements'] = enl
    (out / 'sheet.json').write_text(json.dumps(info, indent=1))
    print(out / 'sheet.json', f'{px:.2f} px per unit, {len(info["views"])} views, {len(enl)} enlargements; palette RGB collisions: {info["palette_rgb_collisions"]}')

# ---------------------------------------------------------------- author-side reader (no mesh involved)
class Sheet:
    def __init__(self, folder):
        self.folder = pathlib.Path(folder); self.info = json.loads((self.folder / 'sheet.json').read_text()); self.px = self.info['pixels_per_unit']
        self.views = {v['name']: v for v in self.info['views']}
        self.rgb2col = {tuple(v): int(k) for k, v in self.info['palette_rgb'].items()}
        self._cache = {}
    def frame(self, name): v = self.views[name]; return (v['origin'][0], v['origin'][1], v['size'][0], v['size'][1])
    def toward(self, name): return tuple(self.views[name]['toward_viewer'])
    def img(self, name, kind):
        k = (name, kind)
        if k not in self._cache: self._cache[k] = np.asarray(Image.open(self.folder / 'views' / f'{name}-{kind}.png'))
        return self._cache[k]
    def paint(self, name, kind='paint'):
        """-> colour per pixel (16-bit game colour, -1 = nothing), and opacity (0..255)."""
        im = self.img(name, kind); flat = (im[..., 0].astype(np.int64) << 16) | (im[..., 1].astype(np.int64) << 8) | im[..., 2]
        lut = {(r << 16) | (g << 8) | b: c for (r, g, b), c in self.rgb2col.items()}; u, inv = np.unique(flat, return_inverse=True)
        col = np.array([lut.get(int(x), -1) for x in u])[inv].reshape(flat.shape); col[im[..., 3] == 0] = -1
        return col, im[..., 3]
    def light(self, name):
        im = self.img(name, 'light'); return np.where(im[..., 1] > 0, im[..., 0].astype(float), np.nan)
    def lit_grey(self, name, kind='lit'):
        im = self.img(name, kind); return np.where(im[..., 3] > 0, im[..., :3].astype(float) @ tl.GREY, np.nan)

if __name__ == '__main__':
    a = sys.argv
    def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
    build(int(a[1]), a[2], float(opt('--px')) if opt('--px') else None, opt('--old'))
