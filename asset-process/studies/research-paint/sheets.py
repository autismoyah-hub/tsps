"""Paint-only (unlit) contact sheets and per-part swatch sheets.

usage: sheets.py contact OUT.png KEY[,KEY..] [--cols N] [--tile PX] [--title T]    front and back, paint only, no light
       sheets.py swatch  OUT.png KEY[,KEY..] [--plan PLAN.json]                    per creature: paint front/back with its
                         parts tinted beside them, then one palette strip per part (colours ordered material by
                         material, dark to light, width = share of the part) and the part's numbers
       sheets.py density OUT.png KEY[,KEY..]                                       where the paint boundaries are (front, back)
KEY = a real NPC id, one of ours (watcher-1, watcher-2, star-hound, zodiac-ray, warden-general-2, or a key of
data/extra.json), npc:ID or def:PATH.json.
"""
import sys, pathlib, json, math
import numpy as np
from PIL import Image, ImageDraw, ImageFont
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import paintlib as pl, measure_paint as mp
BG = (48, 43, 37)
def font(n):
    try: return ImageFont.truetype('segoeui.ttf', n)
    except Exception: return ImageFont.load_default()
def entry_of(key):
    if key.startswith(('npc:', 'def:')): return key, key[-28:]
    if key in mp.OURS: return mp.OURS[key], key
    ex = mp.extra()
    if key in ex: return ex[key], key
    i = int(key); return f'npc:{i}', mp.REAL.get(i, (str(i),))[0] + f' {i}'
def fit(img, w, h):
    im = Image.fromarray(img); s = min(w / im.width, h / im.height, 1.0) if max(im.width, im.height) > 0 else 1
    if s < 1: im = im.resize((max(1, int(im.width * s)), max(1, int(im.height * s))), Image.LANCZOS)
    out = Image.new('RGB', (w, h), BG); out.paste(im, ((w - im.width) // 2, (h - im.height) // 2)); return out

def contact(out, keys, cols=6, tile=260, title=None, pitch=12):
    tiles = []
    for k in keys:
        e, name = entry_of(k); sk = pl.skin(e)
        t = Image.new('RGB', (tile * 2, tile + 22), BG)
        for i, yaw in enumerate((25, 205)):
            t.paste(fit(pl.paint_view(sk, yaw, pitch, size=tile * 2), tile, tile), (i * tile, 0))
        ImageDraw.Draw(t).text((4, tile + 2), name, fill=(235, 235, 235), font=font(14)); tiles.append(t)
    cols = min(cols, len(tiles)); n = -(-len(tiles) // cols); tw, th = tiles[0].size; top = 30 if title else 0
    im = Image.new('RGB', (cols * tw, n * th + top), (20, 18, 16))
    if title: ImageDraw.Draw(im).text((8, 4), title, fill=(240, 240, 240), font=font(17))
    for i, t in enumerate(tiles): im.paste(t, ((i % cols) * tw, top + (i // cols) * th))
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True); im.save(out); print(out, im.size)

def strip(colours, mats, w, h):
    """One part's palette: colours grouped by material (largest material first), dark to light, width = share."""
    im = Image.new('RGB', (w, h), BG); d = ImageDraw.Draw(im)
    order = []
    for mi, mt in enumerate(mats):
        cs = sorted([c for c in colours if c in mt['shares']], key=lambda c: (pl.hsl(c)[2], c))
        if cs: order.append((mi, cs))
    x = 0.0; tot = sum(colours.values()) or 1
    for mi, cs in order:
        for c in cs:
            ww = colours[c] / tot * (w - 2 * len(order)); d.rectangle([x, 0, x + ww, h - 1], fill=pl.rgb(c)); x += ww
        x += 2
    return im

def swatch(out, keys, plan=None):
    panels = []
    for k in keys:
        e, name = entry_of(k); sk = pl.skin(e)
        part = names = None
        if plan:
            import paintplan
            part, names = paintplan.parts_from_plan(sk, json.loads(pathlib.Path(plan).read_text()))
        a = pl.analyse(sk, part, names); rows = [r for r in a['parts'] if r.get('share', 0) > 0]
        W = 1500; rowh = 30; H = max(330, 50 + rowh * len(rows) + 60)
        im = Image.new('RGB', (W, H), BG); d = ImageDraw.Draw(im)
        d.text((8, 4), name, fill=(240, 240, 240), font=font(18))
        x = 4
        for yaw in (25, 205):
            im.paste(fit(pl.paint_view(sk, yaw, 12, size=560), 250, H - 34), (x, 30)); x += 252
        tint = np.array([pl.PART_TINT[p % len(pl.PART_TINT)] for p in a['part']], np.uint8)
        im.paste(fit(pl.paint_view(sk, 25, 12, size=560, face_rgb=tint), 200, H - 34), (x, 30)); x += 206
        m = a['m']
        for j, r in enumerate([r for r in a['parts']]):
            if r.get('share', 0) <= 0: continue
            y = 34 + rowh * j
            d.rectangle([x, y + 4, x + 14, y + rowh - 6], fill=pl.PART_TINT[j % len(pl.PART_TINT)])
            d.text((x + 20, y + 4), f"{r['name'][:17]}", fill=(230, 230, 230), font=font(13))
            im.paste(strip(r['colours'], a['mats'], 380, rowh - 6), (x + 150, y + 3))
            st = r.get('step_med'); el_ = r.get('edge_light'); fp = r.get('fold_paint')
            d.text((x + 540, y + 4), f"{100 * r['share']:4.0f}%  {r['n_colours']:2d} col  step {('%2.0f' % st) if st is not None else ' -'}  patch {r['patch_area_med_e4']:5.0f}  marks {r['marks']:3d}  edge-light {('%.2f' % el_) if el_ is not None else '  - '}  fold-paint {('%.2f' % fp) if fp is not None else '  - '}",
                   fill=(200, 200, 185), font=font(12))
        y = 40 + rowh * len(a['parts'])
        d.text((x, y), f"materials {m['materials']}   palette overlap between parts {m['palette_overlap']:.2f}   left-right paint match {m['mirror_paint_near']:.2f}   marks {m['marks']}   densest part holds {100 * m['focal_part_mark_share']:.0f}% of marks", fill=(235, 220, 160), font=font(13))
        d.text((x, y + 18), 'ramps: ' + '   '.join(f"{pl.mat_name(mt)} {100 * mt['share']:.0f}% {mt['tones']}t" for mt in a['mats'] if mt['share'] >= 0.03), fill=(200, 200, 185), font=font(12))
        panels.append(im)
    im = Image.new('RGB', (panels[0].width, sum(p.height + 4 for p in panels)), (20, 18, 16)); y = 0
    for p in panels: im.paste(p, (0, y)); y += p.height + 4
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True); im.save(out); print(out, im.size)

def density(out, keys, tile=300):
    """Paint boundaries drawn as light lines on the dark figure: where the detail is, where the calm is."""
    import cv2
    tiles = []
    for k in keys:
        e, name = entry_of(k); sk = pl.skin(e)
        t = Image.new('RGB', (tile * 2, tile + 22), (0, 0, 0))
        for i, yaw in enumerate((25, 205)):
            img, ids = pl.paint_view(sk, yaw, 12, size=tile * 2, return_ids=True, all_faces=False)
            C = sk['C']; idc = np.where(ids >= 0, C[np.clip(ids, 0, len(C) - 1)], -1)
            edge = np.zeros(ids.shape, bool)
            edge[:, 1:] |= (idc[:, 1:] != idc[:, :-1]) & (idc[:, 1:] >= 0) & (idc[:, :-1] >= 0)
            edge[1:, :] |= (idc[1:, :] != idc[:-1, :]) & (idc[1:, :] >= 0) & (idc[:-1, :] >= 0)
            heat = cv2.GaussianBlur(edge.astype(np.float32), (0, 0), ids.shape[0] / 28.0)
            heat = heat / max(heat.max(), 1e-6)
            rgbm = cv2.applyColorMap((heat * 255).astype(np.uint8), cv2.COLORMAP_INFERNO)[:, :, ::-1].copy()
            rgbm[ids < 0] = 0; rgbm[edge] = (255, 255, 255)
            t.paste(fit(rgbm, tile, tile), (i * tile, 0))
        ImageDraw.Draw(t).text((4, tile + 2), name, fill=(235, 235, 235), font=font(14)); tiles.append(t)
    cols = min(4, len(tiles)); n = -(-len(tiles) // cols); tw, th = tiles[0].size
    im = Image.new('RGB', (cols * tw, n * th), (0, 0, 0))
    for i, t in enumerate(tiles): im.paste(t, ((i % cols) * tw, (i // cols) * th))
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True); im.save(out); print(out, im.size)

if __name__ == '__main__':
    a = sys.argv[1:]
    def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
    keys = a[2].split(',')
    if a[0] == 'contact': contact(a[1], keys, int(opt('--cols', 4)), int(opt('--tile', 260)), opt('--title'))
    elif a[0] == 'swatch': swatch(a[1], keys, opt('--plan'))
    elif a[0] == 'density': density(a[1], keys)
