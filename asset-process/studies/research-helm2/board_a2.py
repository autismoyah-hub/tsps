"""board_a2.py [A2-labelled.bin] -> boards/board-A2-test.png
The left half of board-67 again (whole figure front and back at game size, then the head close from four sides), with
the test helm A2 put beside A. Columns: A2 on the matched body, A2 on A's stand-in body, A as judged, Meridian wardens
(the helm the owner passed), Torva, Oathplate, Justiciar."""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import h2

D = h2.HERE / 'umbral-helm-a2'
a2 = pathlib.Path(sys.argv[1]).as_posix() if len(sys.argv) > 1 else (D / 'umbral-helm-a2-labelled.bin').as_posix()
match = '+'.join(h2.m((D / f'{k}-match.bin').as_posix(), s) for k, s in (('cuirass', 'body+arms'), ('legs', 'legs'), ('gauntlets', 'hands'), ('boots', 'feet')))
cols = [('A2 · matched body', h2.m(a2, 'head+hair+jaw') + '+' + match),
        ('A2 · A\'s stand-in body', h2.m(a2, 'head+hair+jaw') + '+' + h2.H['umbralA']['rest']),
        ('Umbral A (judged: No)', h2.entry('umbralA')),
        ('Meridian wardens (passed)', h2.entry('v5')),
        ('Torva', h2.entry('torva')), ('Oathplate', h2.entry('oathplate')), ('Justiciar', h2.entry('justiciar'))]
es = [e for _, e in cols]
font = lambda s, b=False: ImageFont.truetype('segoeui' + ('b' if b else '') + '.ttf', s)
W = 240
c = Image.new('RGB', (W * len(cols), 2700), '#1c2226'); d = ImageDraw.Draw(c)
d.text((22, 12), 'Umbral Watch A2: the test of HELM2.md', font=font(30, True), fill='#f2f0e9')
d.text((23, 56), 'Same layout as the left half of board-67. A2 keeps A\'s fitted shell, crescent and palette; it changes how the surface is built and painted.', font=font(16), fill='#b6c6c7')
for i, (n, _) in enumerate(cols): d.text((i * W + 10, 100), n, font=font(16, True), fill='#f2f0e9')
top = 141
for yaw, game, cap in [(0, True, 'Front'), (1024, True, 'Back'), (0, False, 'Head · front'), (192, False, 'Head · three-quarter'), (512, False, 'Head · side'), (1024, False, 'Head · back')]:
    if game: a = h2.draw(f'a2-game-{yaw}', es, yaw=yaw, zoom=300, cell=(240, 450))
    else: a = h2.draw(f'a2-head-{yaw}', es, yaw=yaw, zoom=130, cell=(360, 340), aim='head')
    im = Image.fromarray(a.astype(np.uint8))
    d.text((15, top), cap, font=font(16), fill='#b6c6c7'); top += 27
    for i in range(len(cols)):
        cell = im.crop((i * 240, 0, i * 240 + 240, 430)) if game else im.crop((i * 360 + 55, 0, i * 360 + 295, 249))
        c.paste(cell, (i * W, top))
    top += cell.height + 9
out = h2.HERE / 'boards'; out.mkdir(exist_ok=True)
c.crop((0, 0, c.width, top)).save(out / 'board-A2-test.png'); print(out / 'board-A2-test.png', c.width, top)
