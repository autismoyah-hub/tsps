"""Contact sheet and moving picture: authored keyframe files beside the creature's real sequences, by the client's code.

usage: rig_show.py WORKDIR OUT_STEM ROW [ROW..] [--every 6] [--yaw 192] [--cell 240] [--zoomk 1.5] [--pitch 128] [--model DEF.npcdef] [--base FILE]
ROW = ours:FILE.dat[:LABEL]  (drawn from the loose keyframe file)  |  real:SEQ[:LABEL]  (by sequence id from the cache)
Writes OUT_STEM.png (one row per animation, a cell every N cycles, labelled with the cycle) and OUT_STEM.gif (the rows
side by side, 3 cycles a picture at 60 ms = the game's speed, each looping on its own length).
--model: draw another creature definition (asset_bone def: file) instead of the real NPC (for a new creature).
"""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import riglib as R
L, B = R.L, R.B
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); stem = pathlib.Path(a[2]); stem.parent.mkdir(parents=True, exist_ok=True)
specs = [x for x in a[3:] if x.startswith(('ours:', 'real:'))]
every = int(opt('--every', 6)); yaw = int(opt('--yaw', 192)); cs = int(opt('--cell', 240)); zk = float(opt('--zoomk', 1.5)); pitch = int(opt('--pitch', 128))
entry = f"def:{pathlib.Path(opt('--model')).resolve()}" if opt('--model') else f"npc:{man['npc']}"
M = L.merged(entry, stem.parent / 'rest.tmp.txt'); (stem.parent / 'rest.tmp.txt').unlink(); zoom = int(opt('--zoom', 0)) or int(zk * max(M['V'].max(0) - M['V'].min(0)))
def font(n):
    try: return ImageFont.truetype('segoeuib.ttf', n)
    except Exception: return ImageFont.load_default()
rows = []
for s in specs:
    kind, _, rest = s.partition(':')
    if kind == 'ours':
        path, _, label = rest.rpartition(':') if (':' in rest[2:]) else (rest, '', '')
        if not path: path, label = rest, ''
        n = B.decode_anim(pathlib.Path(path).read_bytes())['range'][1]
        g, _, _ = L.grid(entry, anim=pathlib.Path(path).resolve(), base=opt('--base'), count=n, frames='all', cols=n, zoom=zoom, yaw=yaw, pitch=pitch, cell=f'{cs}x{cs}', aim=opt('--aim'))
        rows.append((f'OURS: {label or pathlib.Path(path).stem} ({n} cycles)', n, g))
    else:
        sid, _, label = rest.partition(':'); sid = int(sid); n = next(x['frames'] for x in man['sequences'] if x['seq'] == sid)
        g, _, _ = L.grid(f"npc:{man['npc']}", seq=sid, frames='all', cols=n, zoom=zoom, yaw=yaw, pitch=pitch, cell=f'{cs}x{cs}', aim=opt('--aim'))
        rows.append((f'real {sid} {label} ({n} cycles)', n, g))
ncell = max(-(-n // every) for _, n, _ in rows)
sheet = Image.new('RGB', (ncell * cs, len(rows) * cs), (0x30, 0x2b, 0x25)); dd = ImageDraw.Draw(sheet)
for r, (name, n, g) in enumerate(rows):
    for k, f in enumerate(range(0, n, every)):
        sheet.paste(Image.fromarray(g[:, f * cs:(f + 1) * cs]), (k * cs, r * cs)); dd.text((k * cs + 4, r * cs + cs - 16), f'cycle {f}', fill=(200, 200, 160), font=font(11))
    dd.text((6, r * cs + 4), name, fill=(255, 255, 255), font=font(14))
wrap = int(opt('--wrap', 0))
if wrap and ncell > wrap:                       # long rows folded: every animation's cells 0..wrap-1, then the next band
    bands = -(-ncell // wrap); w2 = Image.new('RGB', (wrap * cs, bands * len(rows) * cs), (0x30, 0x2b, 0x25))
    for b in range(bands): w2.paste(sheet.crop((b * wrap * cs, 0, min((b + 1) * wrap, ncell) * cs, len(rows) * cs)), (0, b * len(rows) * cs))
    sheet = w2
sheet.save(stem.with_suffix('.png'))
frames = []; total = max(n for _, n, _ in rows) * 2
for t in range(0, total, 3):
    f = Image.new('RGB', (len(rows) * cs, cs + 22), (20, 18, 16)); d2 = ImageDraw.Draw(f)
    for r, (name, n, g) in enumerate(rows):
        k = t % n; f.paste(Image.fromarray(g[:, k * cs:(k + 1) * cs]), (r * cs, 22)); d2.text((r * cs + 4, 3), name[:40], fill=(240, 240, 240), font=font(11))
    frames.append(f)
frames[0].save(stem.with_suffix('.gif'), save_all=True, append_images=frames[1:], duration=60, loop=0)
print(stem.with_suffix('.png'), sheet.size, stem.with_suffix('.gif'), len(frames), 'frames')
