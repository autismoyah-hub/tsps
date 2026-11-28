"""look.py: one pose from several directions, entries side by side (the first look at a build, and the close reading).

usage: look.py OUT.png ENTRY[;SOURCE[;FRAME]] ... [--yaws 192,256,0,64] [--zoom N] [--pitch N] [--cell WxH] [--aim H]
ENTRY as skelview's (npc:ID | def:PATH); SOURCE bind | seq=ID | anim=DIR; one row per entry, one column per yaw.
Yaw is in 1/2048 turns of the client's camera (192 = the usual three-quarter front; 1216 = from behind).
"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import skelview as S
from PIL import Image, ImageDraw
rows, o = S.parse(sys.argv[2:])
yaws = [int(y) for y in o.get('yaws', '192,704,1216,1728').split(',')]
cw, ch = map(int, o.get('cell', '360x400').split('x'))
im = Image.new('RGB', (cw * len(yaws), ch * len(rows)), S.BG); d = ImageDraw.Draw(im)
font, small = S.fonts()
for r, row in enumerate(rows):
    p = row.split(';'); entry = p[0]; source = p[1] if len(p) > 1 else 'bind'; frame = [int(p[2])] if len(p) > 2 else ([0] if source != 'bind' else None)
    for c, yaw in enumerate(yaws):
        cells, _ = S.strip(entry, source, frame, zoom=int(o.get('zoom', 900)), yaw=yaw, pitch=int(o.get('pitch', 0)), cell=(cw, ch),
                           aim=int(o['aim']) if 'aim' in o else None)
        im.paste(cells[0], (c * cw, r * ch))
        d.text((c * cw + 5, r * ch + 4), f'{pathlib.Path(entry).name if entry.startswith("def:") else entry} yaw {yaw}' + (f' {source} f{frame[0]}' if frame else ''), fill=(220, 220, 180), font=small)
pathlib.Path(sys.argv[1]).parent.mkdir(parents=True, exist_ok=True)
im.save(sys.argv[1]); print(sys.argv[1], im.size)
