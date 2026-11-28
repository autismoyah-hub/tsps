"""skelcard.py: the axis card of a skeleton of ours: every turn channel given +AMOUNT on x, then y, then z, one cell each,
so an animator SEES which way a positive number turns each joint before keying anything.

usage: skelcard.py RIG.json DEF.npcdef OUT.png [--amount 24] [--zoom N] [--yaw N] [--cell WxH] [--aim H] [--joints a,b,c]
Rows: joints; columns: rest, x+, y+, z+ (turn channels), and move y-AMOUNT (up) where the joint has a move channel.
"""
import sys, json, pathlib, tempfile
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE)); sys.path.insert(0, str(HERE.parent / 'research-animation'))
import animlib as A, skelanim, skelview as S
from PIL import Image, ImageDraw
rows, o = S.parse(sys.argv[1:])
rigpath, defpath, out = rows[0], rows[1], rows[2]
rig, base, chan = skelanim.load_rig(rigpath)
amount = int(o.get('amount', 24))
joints = o['joints'].split(',') if 'joints' in o else [j for j, e in rig['joints'].items() if 'turn' in e['channels']]
cw, ch = map(int, o.get('cell', '220x260').split('x'))
tmp = pathlib.Path(tempfile.mkdtemp(prefix='skelcard-'))
frames = []; caps = []
for j in joints:
    g = chan[f'{j}.turn']
    frames.append({}); caps.append(f'{j}: rest')
    for a, n in enumerate('xyz'):
        v = [0, 0, 0]; v[a] = amount; frames.append({g: v}); caps.append(f'{j}.turn {n}+{amount}')
bid = rig['base_id']
ids = []
for k, f in enumerate(frames):
    (tmp / f'frame_30000_{k + 1}.dat').write_bytes(A.encode_frame(dict(base=bid, groups=f), base)); ids.append((30000 << 16) | (k + 1))
s = dict(A.SEQ_DEFAULTS); s['frames'] = ids; s['lengths'] = [1] * len(ids)
(tmp / 'seq.dat').write_bytes(A.encode_sequence(s)); (tmp / f'base_{bid}.dat').write_bytes(A.encode_base(base))
cells, _ = S.strip(f'def:{defpath}', f'anim={tmp}', None, zoom=int(o.get('zoom', 1100)), yaw=int(o.get('yaw', 192)), cell=(cw, ch), aim=int(o['aim']) if 'aim' in o else None)
im = Image.new('RGB', (cw * 4, ch * len(joints)), S.BG); d = ImageDraw.Draw(im); font, small = S.fonts()
for k, c in enumerate(cells):
    x, y = (k % 4) * cw, (k // 4) * ch
    im.paste(c, (x, y)); d.text((x + 4, y + 3), caps[k], fill=(230, 225, 190), font=small)
pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True); im.save(out); print(out, im.size)
