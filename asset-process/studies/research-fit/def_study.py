"""def_study.py: step 2. Definition of seven sets (the five reference sets, our earlier set, our redesigned set), each
with and without its helm, at game size (zoom 300; and zoom 520 as a check), standing and through walk and run, front
(yaw 192) and back (yaw 1216). Writes def-measures.json and contact sheets in render/research-fit-definition/."""
import sys, pathlib, json
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import lib as L, deflib as D

OUT = L.ROOT / 'render' / 'research-fit-definition'
sets = [(r[0], L.ref_outfit(r), L.ref_outfit(r, helm=False)) for r in L.REFS]
sets.append(('earlier', L.HELM_V2A + '+' + L.SET_V2, L.SET_V2))
sets.append(('redesigned', L.HELM_V3A + '+' + L.SET_V3, L.SET_V3))
entries = [s[1] for s in sets] + [s[2] for s in sets]
names = [s[0] for s in sets] + [s[0] + '-nohelm' for s in sets]
allres = {}
for zoom, cell in ((300, (260, 420)), (520, (160, 260))):
    res, cells = D.run_entries(OUT / f'z{zoom}.png', entries, zoom=zoom, cell=cell)
    allres[zoom] = {names[k]: dict(summary=D.summarise(res[k]), frames=res[k]) for k in range(len(entries))}
    if zoom == 300:
        # contact sheet: the figure and beside it what the step measure sees
        try: font = ImageFont.truetype('segoeuib.ttf', 15)
        except Exception: font = ImageFont.load_default()
        views = ['stand-front', 'stand-back', 'run4-front', 'run4-back']
        for tag, ks in (('sets', range(7)), ('nohelm', range(7, 14))):
            W, H = cell
            sheet = Image.new('RGB', (len(ks) * W, len(views) * H * 2), (0, 0, 0))
            for ci, k in enumerate(ks):
                for vi, v in enumerate(views):
                    c, bg = cells[k][v]
                    sheet.paste(Image.fromarray(c.astype(np.uint8)), (ci * W, vi * 2 * H))
                    sheet.paste(Image.fromarray(D.edge_picture(c, bg)), (ci * W, vi * 2 * H + H))
            d = ImageDraw.Draw(sheet)
            for ci, k in enumerate(ks):
                s = allres[zoom][names[k]]['summary']
                d.text((ci * W + 4, 2), names[k], fill=(255, 255, 255), font=font)
                d.text((ci * W + 4, 20), f"step {s['step']} lines {s['lines']}", fill=(255, 230, 150), font=font)
            sheet.save(OUT / f'contact-{tag}.png'); print(OUT / f'contact-{tag}.png', sheet.size)
json.dump(allres, open(pathlib.Path(__file__).parent / 'def-measures.json', 'w'), indent=1)
cols = ['area', 'step16', 'step', 'step40', 'step_len', 'step_mat', 'step_dark', 'step_tone', 'contrast', 'line_dark', 'line_light', 'line_mat', 'lines',
        'regions', 'reg_per1000', 'reg_big', 'reg_cover', 'p05', 'p50', 'p95', 'range', 'dark']
for zoom in allres:
    for suffix, title in (('', 'median of 34 pictures'), ('_stand', 'standing'), ('_move', 'moving (32 frames)'), ('_front', 'front'), ('_back', 'back')):
        print(f'\nzoom {zoom}: {title}')
        print(f"{'set':20}" + ''.join(f'{c[:9]:>10}' for c in cols))
        for n in names:
            s = allres[zoom][n]['summary']
            print(f'{n:20}' + ''.join(f'{s[c + suffix]:>10}' for c in cols))
