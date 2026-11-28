"""Boards for the LOOK step (no measure is computed here).

    P board.py contact          REAL at close size, one cell each (to pick the sentinel's and ray's nearest)
    P board.py pairs            each BEST beside its nearest REAL: game size (x3 nearest-neighbour), close, silhouette
    P board.py ours             all eleven of ours at close size (for the review questions)
Pictures by research-clean/cleanlib.view (the client's light, once at rest). Game size = 0.445 px per unit x scale/128
(the NPC harness's zoom 1150), pitch 39.
"""
import sys, pathlib, json
import numpy as np, cv2
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gaplib as g
cl = g.cl

def scale_of(name):
    e = g.ALL[name]
    if e.startswith('npc:'): return g.cl.census()[int(e[4:])]['scale'][0]
    return json.loads(pathlib.Path(e[4:]).read_text()).get('scale', [128, 128])[0]

def game_ppu(name): return 0.445 * scale_of(name) / 128

def label(img, text, h=22):
    bar = np.full((h, img.shape[1], 3), 20, np.uint8)
    cv2.putText(bar, text, (4, h - 7), cv2.FONT_HERSHEY_SIMPLEX, 0.45, (235, 235, 235), 1, cv2.LINE_AA)
    return np.vstack([bar, img])

def pad_to(img, w, h, bg=cl.BG):
    out = np.zeros((h, w, 3), np.uint8); out[:] = bg
    y = (h - img.shape[0]) // 2; x = (w - img.shape[1]) // 2
    out[max(y, 0):max(y, 0) + min(img.shape[0], h), max(x, 0):max(x, 0) + min(img.shape[1], w)] = img[:h, :w]
    return out

def rgb2bgr(a): return np.ascontiguousarray(a[:, :, ::-1])

def close(c, yaw, size=420, pitch=10): return cl.view(c, yaw=yaw, pitch=pitch, size=size)[0]

def game(c, name, yaw, mag=3):
    img, _ = cl.view(c, yaw=yaw, pitch=39, ppu=game_ppu(name), pad=4)
    return cv2.resize(img, (img.shape[1] * mag, img.shape[0] * mag), interpolation=cv2.INTER_NEAREST)

def game_common(cs, names, yaw, mag=3):
    """game-size pictures of several creatures at one shared pixel scale (each at its own true game size)"""
    return [game(c, n, yaw, mag) for c, n in zip(cs, names)]

def silhouette(c, yaw, size=300):
    nf = len(c['F']); rgb = np.zeros((nf, 3), np.uint8)
    return cl.view(c, yaw=yaw, pitch=10, size=size, face_rgb=rgb, bg=(235, 235, 235))[0]

def contact():
    cells = []
    for n in g.REAL:
        c = g.load(g.ALL[n]); cells.append(label(pad_to(close(c, 30, 220), 220, 220), n))
    rows = [np.hstack(cells[i:i + 6] + [np.zeros_like(cells[0])] * (6 - len(cells[i:i + 6]))) for i in range(0, len(cells), 6)]
    out = HERE / 'boards' / 'contact-real.png'; cv2.imwrite(str(out), rgb2bgr(np.vstack(rows))); print(out)
    cells = []
    for n in list(g.BEST) + list(g.REJECTED):
        c = g.load(g.ALL[n]); cells.append(label(pad_to(close(c, 30, 220), 220, 220), n))
    rows = [np.hstack(cells[i:i + 6] + [np.zeros_like(cells[0])] * (6 - len(cells[i:i + 6]))) for i in range(0, len(cells), 6)]
    out = HERE / 'boards' / 'contact-ours.png'; cv2.imwrite(str(out), rgb2bgr(np.vstack(rows))); print(out)

def pair(ours):
    real = g.NEAREST[ours]
    co, cr = g.load(g.ALL[ours]), g.load(g.ALL[real])
    yaws = (0, 45, 90)
    blocks = []
    # game size: both at their true game size, magnified x3 nearest-neighbour, in one strip
    gs = []
    for y in yaws:
        a, b = game(cr, real, y), game(co, ours, y)
        h = max(a.shape[0], b.shape[0]); w = max(a.shape[1], b.shape[1])
        gs.append(np.hstack([pad_to(a, w, h), np.full((h, 6, 3), 0, np.uint8), pad_to(b, w, h)]))
    W = max(x.shape[1] for x in gs)
    blocks.append(label(np.vstack([pad_to(x, W, x.shape[0]) for x in gs]), f'GAME SIZE x3: left {real} (real), right {ours} (ours); yaw 0/45/90'))
    cs = []
    for y in yaws:
        cs.append(np.hstack([pad_to(close(cr, y), 420, 420), np.full((420, 6, 3), 0, np.uint8), pad_to(close(co, y), 420, 420)]))
    blocks.append(label(np.vstack([pad_to(x, 846, 420) for x in cs]), f'CLOSE: left {real}, right {ours}'))
    sq = lambda im: pad_to(im, 300, 300, (235, 235, 235))
    sil = np.hstack([sq(silhouette(cr, 0)), sq(silhouette(cr, 90)), np.full((300, 6, 3), 0, np.uint8), sq(silhouette(co, 0)), sq(silhouette(co, 90))])
    blocks.append(label(sil, f'SILHOUETTE front/side: left {real}, right {ours}'))
    W = max(b.shape[1] for b in blocks)
    img = np.vstack([pad_to(b, W, b.shape[0]) for b in blocks])
    out = HERE / 'boards' / f"pair-{ours.replace(' ', '-').lower()}.png"
    cv2.imwrite(str(out), rgb2bgr(img)); print(out)
    # close-up crops of the head / front at double size for reading features
    for tag, c, n in (('real', cr, real), ('ours', co, ours)):
        img = close(c, 20, 900, 5)
        cv2.imwrite(str(HERE / 'boards' / f"big-{ours.replace(' ', '-').lower()}-{tag}.png"), rgb2bgr(img))

if __name__ == '__main__':
    what = sys.argv[1]
    if what == 'contact': contact()
    elif what == 'pairs':
        for o in (sys.argv[2:] or g.BEST): pair(o)
