"""overlay.py: cross-section overlays of helm on bare head, front and side, all at ONE scale (6 pixels a unit).
usage: overlay.py            -> render/research-fit-overlay/{key}.png, sheet-refs.png, sheet-ours.png, outlines.png
Per panel: grey = the helm's silhouette; pale blue = the body piece it is worn with; white line = the helm cut by
the middle plane (side view: x = 0; front view: z = -8, through the middle of the skull); red = the bare head (hair +
jaw kits), outline and the same cut; grid every 10 units, heights labelled; collar 164, crown 196."""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
from scipy import ndimage
import lib as L, fitlib as F

OUT = L.ROOT / 'render' / 'research-fit-overlay'; OUT.mkdir(parents=True, exist_ok=True)
SC = 6; XA, XB, HA, HB = -42, 42, 148, 222
PW, PH = (XB - XA) * SC, (HB - HA) * SC
try:
    FONT = ImageFont.truetype('segoeui.ttf', 14); BOLD = ImageFont.truetype('segoeuib.ttf', 17)
except Exception:
    FONT = BOLD = ImageFont.load_default()


def px(u, h):
    return ((u - XA) * SC, (HB - h) * SC)


def mask(T, view):
    im = Image.new('L', (PW, PH), 0); d = ImageDraw.Draw(im); ax = 0 if view == 'f' else 2
    for t in T:
        pts = [px(p[ax], p[1]) for p in t]
        d.polygon(pts, fill=255); d.line(pts + [pts[0]], fill=255)
    return np.asarray(im) > 0


def outline(m, width=2):
    return m & ~ndimage.binary_erosion(m, iterations=width)


def cut(T, view):
    """segments where the triangles cross the middle plane; returned in panel pixels"""
    ax, c = (2, -8.0) if view == 'f' else (0, 0.0)
    keep = 0 if view == 'f' else 2
    segs = []
    for t in T:
        pts = []
        for i in range(3):
            a, b = t[i], t[(i + 1) % 3]
            da, db = a[ax] - c, b[ax] - c
            if da == 0 and db == 0:
                pts += [a, b]
            elif (da <= 0 <= db) or (db <= 0 <= da):
                if da != db:
                    k = da / (da - db); pts.append(a + (b - a) * k)
        if len(pts) >= 2:
            segs.append((px(pts[0][keep], pts[0][1]), px(pts[1][keep], pts[1][1])))
    return segs


def panel(helm, body, hides, view, title, notes):
    A = F.all_tris(helm); H = F.head_tris()
    img = np.zeros((PH, PW, 3), np.uint8); img[:] = (30, 28, 26)
    if body is not None:
        img[mask(L.tris(body['V'], body['F']), view)] = (58, 74, 96)
    img[mask(A, view)] = (150, 150, 150)
    hm = mask(H, view)
    im = Image.fromarray(img); d = ImageDraw.Draw(im)
    for u in range(XA + 2, XB, 10):
        x = px(u, 0)[0]; d.line([(x, 0), (x, PH)], fill=(70, 66, 60) if u else (110, 104, 96))
    for h in range(150, HB, 10):
        y = px(0, h)[1]; d.line([(0, y), (PW, y)], fill=(70, 66, 60)); d.text((3, y - 16), str(h), fill=(150, 144, 130), font=FONT)
    for h, c in ((164, (90, 140, 200)), (196, (200, 160, 60))):
        y = px(0, h)[1]; d.line([(0, y), (PW, y)], fill=c)
    arr = np.asarray(im).copy()
    arr[outline(hm)] = (235, 60, 50)
    im = Image.fromarray(arr); d = ImageDraw.Draw(im)
    for a, b in cut(H, view): d.line([a, b], fill=(235, 60, 50), width=1)
    for a, b in cut(A, view): d.line([a, b], fill=(255, 255, 255), width=2)
    d.text((6, 4), title, fill=(245, 245, 245), font=BOLD)
    d.text((PW - 70, 4), 'FRONT' if view == 'f' else 'SIDE', fill=(200, 200, 160), font=FONT)
    if view == 's':
        d.text((6, PH - 20), '< face', fill=(200, 200, 160), font=FONT)
    for k, n in enumerate(notes):
        d.text((6, 26 + 16 * k), n, fill=(225, 225, 205), font=FONT)
    return im


def both(key, name, helm, body, hides, d):
    n1 = [f"stand-off median {d['so_med']}  (10th pct {d['so_p10']})", f"brow {d['so_brow']}  temple {d['so_temple']}  cheek {d['so_cheek']}",
          f"width: skull {d['w_skull']}  jaw {d['w_jaw']}  neck {d['w_neck']}"]
    n2 = [f"depth: skull {d['d_skull']}  cheek {d['d_cheek']}  jaw {d['d_jaw']}", f"crown {d['so_crown']}  nape {d['so_nape']}  jaw {d['so_jaw']}",
          f"chin overhang {d['chin_overhang']}  pinch {d['pinch_s']} ({d['pinch_s_ratio']} of skull)"]
    a = panel(helm, body, hides, 'f', name, n1); b = panel(helm, body, hides, 's', name, n2)
    im = Image.new('RGB', (PW * 2 + 6, PH), (0, 0, 0)); im.paste(a, (0, 0)); im.paste(b, (PW + 6, 0))
    im.save(OUT / f'{key}.png')
    return im


import json
M = {d['key']: d for d in json.load(open(pathlib.Path(__file__).parent / 'fit-measures.json'))['helms']}
refs, ours = [], []
meshes = {}
for key, hid, name, bid, lid, note in L.REFS:
    wp = L.wear_positions(hid); h = L.mesh_real(hid); b = L.mesh_real(bid)
    meshes[key] = ('R', h)
    refs.append(both(key, f'{name} {hid}', h, b, (8 in wp[1:], 11 in wp[1:]), M[key]))
for key, path, spec, name, bpath, rnd in L.OURS:
    h = L.mesh_ours(path); b = L.mesh_ours(bpath)
    meshes[key] = ('O', h)
    ours.append(both(key, f'OURS {key}: {name} (round {rnd})', h, b, ('hair' in spec, 'jaw' in spec), M[key]))


def sheet(ims, cols, path):
    rows = (len(ims) + cols - 1) // cols
    w, h = ims[0].size
    S_ = Image.new('RGB', (cols * (w + 10), rows * (h + 10)), (0, 0, 0))
    for k, im in enumerate(ims): S_.paste(im, ((k % cols) * (w + 10), (k // cols) * (h + 10)))
    S_.save(path); print(path, S_.size)


sheet(refs, 1, OUT / 'sheet-refs.png'); sheet(ours, 1, OUT / 'sheet-ours.png')
sheet(refs + ours, 2, OUT / 'sheet-all.png')

# every outline on one head: the five references in green, ours in orange, the bare head in red
for sub, sel in (('all', [k for k in meshes if k in M]), ('v3', ['torva', 'justiciar', 'serpentine', 'faceguard', 'oathplate', 'v3a', 'v3b', 'v3c'])):
    ims = []
    for view in 'fs':
        img = np.zeros((PH, PW, 3), np.uint8); img[:] = (30, 28, 26)
        im = Image.fromarray(img); d = ImageDraw.Draw(im)
        for h in range(150, HB, 10):
            y = px(0, h)[1]; d.line([(0, y), (PW, y)], fill=(70, 66, 60)); d.text((3, y - 16), str(h), fill=(150, 144, 130), font=FONT)
        for u in range(XA + 2, XB, 10):
            x = px(u, 0)[0]; d.line([(x, 0), (x, PH)], fill=(70, 66, 60))
        arr = np.asarray(im).copy()
        for g, colr in (('R', (70, 220, 110)), ('O', (255, 150, 40))):
            for k in sel:
                if meshes[k][0] == g:
                    arr[outline(mask(F.all_tris(meshes[k][1]), view), 1)] = colr
        arr[outline(mask(F.head_tris(), view), 2)] = (235, 60, 50)
        im = Image.fromarray(arr); d = ImageDraw.Draw(im)
        d.text((6, 4), ('FRONT' if view == 'f' else 'SIDE (face to the left)') + ': green = the five references, orange = ours' + (' (round 3 only)' if sub == 'v3' else ' (all seven)') + ', red = bare head', fill=(240, 240, 240), font=FONT)
        ims.append(im)
    S_ = Image.new('RGB', (PW * 2 + 6, PH)); S_.paste(ims[0], (0, 0)); S_.paste(ims[1], (PW + 6, 0)); S_.save(OUT / f'outlines-{sub}.png')
    print(OUT / f'outlines-{sub}.png')
