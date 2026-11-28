"""Contact sheets of the study rooms.

usage: sheet.py VIEW OUT.png ID [ID ...]     VIEW = overview | played | threeq | plan
Each cell: the picture, then a line with id, kind, place, era and the key numbers
(standing pieces per 10 tiles, share against a wall, wall decorations per 10 wall edges, distinct names).
"""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rlib as R
from render import plan
from PIL import Image, ImageDraw

REAL = R.TF / 'render' / 'research-rooms-real'


def cell_image(r, view, cw, ch):
    if view == 'plan':
        im = plan(r['norm'], R.HERE / 'cache' / f'plan-{r["id"]}.png', '')
    else:
        im = Image.open(REAL / str(r['id']) / f'{view}.png').convert('RGB')
        w, h = im.size
        im = im.crop((int(w * .12), int(h * .08), int(w * .88), int(h * .92)))
    im.thumbnail((cw, ch))
    return im


def main():
    view, out = sys.argv[1], pathlib.Path(sys.argv[2])
    ids = [int(a) for a in sys.argv[3:]]
    study = {r['id']: r for r in json.loads((R.HERE / 'study.json').read_text())}
    (R.HERE / 'cache').mkdir(exist_ok=True)
    cols = 5; cw, ch = (300, 250) if view != 'plan' else (300, 300)
    rows = (len(ids) + cols - 1) // cols
    sheet = Image.new('RGB', (cols * cw, rows * (ch + 34)), (20, 20, 22)); d = ImageDraw.Draw(sheet)
    for i, rid in enumerate(ids):
        r = study[rid]; m = r['m']
        im = cell_image(r, view, cw, ch)
        x, y = (i % cols) * cw, (i // cols) * (ch + 34)
        sheet.paste(im, (x + (cw - im.size[0]) // 2, y + (ch - im.size[1]) // 2))
        t1 = f"#{rid} {r['kind']}{' tower' if r['tower'] else ''} | {r['place'][:20]} ({r['era'][0]})"
        ws = '-' if m['wall_share'] is None else f"{m['wall_share']:.2f}"
        t2 = f"{r['w']}x{r['h']} a{m['area']} per10 {m['per10']:.1f} wall {ws} dec/10w {m['decor_per10w']:.1f} var {m['variety_all']}"
        d.text((x + 4, y + ch + 3), t1, fill=(255, 255, 255)); d.text((x + 4, y + ch + 17), t2, fill=(200, 200, 160))
    sheet.save(out)
    print(out, sheet.size)


if __name__ == '__main__':
    main()
