"""Boards for step 2: the real boss's small marks outlined on the real picture and on each rebuild's, kept or missing.

usage: board.py tekton | nex | table
  tekton  real 7541 | first rebuild | second | plate-first      nex  real 11278 | rebuild
  Writes render/research-paint-detail/NAME-game.png (true size above, x3 nearest-neighbour below, yaw 192 and 1216,
  rest pose) and NAME-close.png (zoom 575, x1). On the real: white outline = a mark visible by the gate's rule, grey =
  seen but under 12 grey levels of effect. On a rebuild: green = kept in this cell, red = missing.
  table   render/research-paint-detail/recall-table.png from work/results.
"""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np, cv2
from PIL import Image, ImageDraw, ImageFont
import dlib, detail, detailgate
from dlib import tl
SETS = dict(tekton=(7541, ['tekton1', 'tekton2', 'tekton3'], ['first rebuild ("[quoted words withheld]")', 'second rebuild (not shown to the owner)', 'plate-first ("[quoted words withheld]")']),
            nex=(11278, ['nex'], ['rebuild ("less definition, still very good")']))
def font(n):
    try: return ImageFont.truetype('segoeui.ttf', n)
    except Exception: return ImageFont.load_default()
def outlined(img, insts, colours, f):
    big = cv2.resize(img, None, fx=f, fy=f, interpolation=cv2.INTER_NEAREST)
    for inst, col in zip(insts, colours):
        if col is None: continue
        y0, y1, x0, x1 = inst['box']; m = cv2.resize(inst['fp'].astype(np.uint8), None, fx=f, fy=f, interpolation=cv2.INTER_NEAREST)
        cs, _ = cv2.findContours(m, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_NONE)
        cv2.drawContours(big, cs, -1, col, 1, offset=(x0 * f, y0 * f))
    return big
def board(name):
    npc, tags, titles = SETS[name]
    mr, dr, seqs, ar, marks, pr, pr0 = detail.real_set(npc); allc, _ = dlib.cameras(npc)
    ours = [dlib.ours_pictures(t, npc, dlib.OURS[t][1])[2] for t in tags]
    for cn, f, yaws in (('game', 3, (192, 1216)), ('close', 1, (192,))):
        k = allc[cn]['k']; strips = []
        for yaw in yaws:
            key = f'{cn}|{yaw}|bind'; lit, lit0, face = pr[key + '|lit'], pr0[key + '|lit'], pr[key + '|face']
            insts = [i for i in dlib.instances(ar, marks, face, lit, lit0, k) if not marks[i['mark']]['translucent'] and i['px'] >= dlib.PX_MIN]
            vis = [i['effect'] >= dlib.C_MIN for i in insts]
            ys, xs = np.nonzero(tl.mask(lit)); pad = 6; y0, y1, x0, x1 = max(ys.min() - pad, 0), ys.max() + pad, max(xs.min() - pad, 0), xs.max() + pad
            cols = [(lit, [(255, 255, 255) if v else (120, 120, 120) for v in vis], f'REAL: {sum(vis)} visible marks in this view')]
            for o, title in zip(ours, titles):
                ol = o[key + '|lit']; cc = []; kept = 0
                for i, v in zip(insts, vis):
                    if not v: cc.append(None); continue
                    ku = 0
                    for kk in range(k, 0, -1):
                        if dlib.presence(lit0, lit0, i, kk) < 0.25: ku = kk; break
                    ok = dlib.presence(ol, lit0, i, ku) >= dlib.HALF; kept += ok; cc.append((0, 230, 0) if ok else (255, 40, 40))
                cols.append((ol, cc, f'{title}: kept {kept} of {sum(vis)}'))
            tiles = []
            for img, cc, title in cols:
                parts = []
                if f > 1: plain = img[y0:y1, x0:x1]; parts.append(plain)
                parts.append(cv2.resize(img, None, fx=f, fy=f, interpolation=cv2.INTER_NEAREST)[y0 * f:y1 * f, x0 * f:x1 * f] if f > 1 else img[y0:y1, x0:x1])
                parts.append(outlined(img, insts, cc, f)[y0 * f:y1 * f, x0 * f:x1 * f])
                w = max(p.shape[1] for p in parts); h = sum(p.shape[0] for p in parts) + 26
                t = Image.new('RGB', (w, h), (48, 43, 37)); y = 26
                ImageDraw.Draw(t).text((4, 4), title[:int(w / 6.5)], fill=(240, 240, 240), font=font(13))
                for p in parts: t.paste(Image.fromarray(p), (0, y)); y += p.shape[0]
                tiles.append(t)
            s = Image.new('RGB', (sum(t.width for t in tiles) + 8 * (len(tiles) - 1), max(t.height for t in tiles)), (20, 18, 16)); x = 0
            for t in tiles: s.paste(t, (x, 0)); x += t.width + 8
            strips.append(s)
        out = Image.new('RGB', (max(s.width for s in strips), sum(s.height for s in strips) + 30), (20, 18, 16)); y = 30
        ImageDraw.Draw(out).text((6, 6), f'{name}: small marks of the real boss at {"game size (true size, x3, x3 with marks)" if f > 1 else "close (zoom 575), plain and with marks"}; rest pose; green kept, red missing, white = the real mark', fill=(255, 255, 255), font=font(14))
        for s in strips: out.paste(s, (0, y)); y += s.height
        p = dlib.RENDER / f'{name}-{cn}.png'; out.save(p); print(p, out.size)
def table():
    order = ['ceiling-7541', 'islands10-7541', 'under-9-7541', 'under-21-7541', 'islands50-7541', 'islands100-7541', 'allout-7541', 'tekton1', 'tekton2', 'tekton3', 'ceiling-11278', 'under-9-11278', 'under-21-11278', 'allout-11278', 'nex', 'minotaur', 'lynx', 'araxyte']
    owner = dict(tekton1='found: "[quoted words withheld]"', tekton2='not shown', tekton3='found: "[quoted words withheld]"', nex='found: "[quoted words withheld]"', minotaur='taken for the real one', lynx='not shown blind (as far as recorded)', araxyte='found, by leg colour')
    rows = [detailgate.line(json.loads((dlib.WORK / 'results' / f'{t}.json').read_text())) for t in order]
    heads = ['case', 'visible', 'MISSING', 'verdict', 'recall', '2-3 px', '4-8 px', '9-20 px', '21+ px', 'spurious', 'owner']
    xs = [8, 150, 215, 290, 385, 450, 540, 630, 725, 820, 900]
    im = Image.new('RGB', (1240, 30 + 22 * (len(rows) + 1)), (255, 255, 255)); d = ImageDraw.Draw(im); f = font(14)
    for x, h in zip(xs, heads): d.text((x, 6), h, fill=(0, 0, 0), font=f)
    for j, r in enumerate(rows):
        y = 30 + 22 * j; col = dict(PASS=(0, 120, 0), BORDERLINE=(170, 110, 0), FAIL=(190, 0, 0))[r['verdict']]
        bf = r['by_footprint']; cell = lambda n: f"{bf[n][0]:.2f} ({bf[n][1]})" if n in bf else '-'
        vals = [r['tag'], str(r['visible']), f"{r['missing']:.1f}", r['verdict'], f"{r['recall']:.2f}", cell('2-3 px'), cell('4-8 px'), cell('9-20 px'), cell('21+ px'), f"{r['spurious']:.0f} of {r['ours_visible']}", owner.get(r['tag'], '')]
        for x, v in zip(xs, vals): d.text((x, y), v, fill=col if x == 290 else (0, 0, 0), font=f)
    p = dlib.RENDER / 'recall-table.png'; im.save(p); print(p)
if __name__ == '__main__':
    for a in sys.argv[1:]: table() if a == 'table' else board(a)
