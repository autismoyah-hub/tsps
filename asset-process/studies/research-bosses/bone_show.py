"""Contact sheet and moving picture of an AUTHORED keyframe animation beside the creature's real ones.

usage: bone_show.py NPC_ID OUT_STEM AUTHORED_DIR[,AUTHORED_DIR..] REAL_SEQ[,REAL_SEQ..] [--every 6] [--yaw 192] [--cell 260] [--zoomk 1.6] [--base FILE]
Writes OUT_STEM.png (one row per animation: authored rows first, labelled; one cell every N cycles) and OUT_STEM.gif
(all rows side by side at 20 ms a cycle, 3 cycles a picture, each looping on its own length).
Authored rows are drawn from the loose keyframe file (asset_bone grid anim=..), real rows by sequence id from the cache.
"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import bonelib as L, bonecodec as B
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
npc = int(a[1]); stem = pathlib.Path(a[2]); authored = [pathlib.Path(x) for x in a[3].split(',') if x]; real = [int(x) for x in a[4].split(',') if x]
every = int(opt('--every', 6)); yaw = int(opt('--yaw', 192)); cs = int(opt('--cell', 260)); zk = float(opt('--zoomk', 1.6))
stem.parent.mkdir(parents=True, exist_ok=True)
M = L.merged(f'npc:{npc}', stem.parent / 'rest.tmp.txt'); (stem.parent / 'rest.tmp.txt').unlink(); zoom = int(zk * max(M['V'].max(0) - M['V'].min(0)))
def font(n):
    try: return ImageFont.truetype('segoeuib.ttf', n)
    except Exception: return ImageFont.load_default()
rows = []
for d in authored:
    an = B.decode_anim((d / 'anim.dat').read_bytes()); n = an['range'][1]
    g, _, _ = L.grid(f'npc:{npc}', anim=d / 'anim.dat', base=opt('--base'), count=n, frames='all', cols=n, zoom=zoom, yaw=yaw, pitch=128, cell=f'{cs}x{cs}')
    rows.append((f'OURS: {d.name} ({n} cycles)', n, g))
for s in real:
    n = dict(L.seqs_on_base(L.base_of_seq(s)))[s]
    g, _, _ = L.grid(f'npc:{npc}', seq=s, frames='all', cols=n, zoom=zoom, yaw=yaw, pitch=128, cell=f'{cs}x{cs}')
    rows.append((f'real sequence {s} ({n} cycles)', n, g))
ncell = max(-(-n // every) for _, n, _ in rows)
sheet = Image.new('RGB', (ncell * cs, len(rows) * cs), (0x30, 0x2b, 0x25)); dd = ImageDraw.Draw(sheet)
for r, (name, n, g) in enumerate(rows):
    for k, f in enumerate(range(0, n, every)):
        sheet.paste(Image.fromarray(g[:, f * cs:(f + 1) * cs]), (k * cs, r * cs)); dd.text((k * cs + 4, r * cs + cs - 16), f'cycle {f}', fill=(200, 200, 160), font=font(11))
    dd.text((6, r * cs + 4), name, fill=(255, 255, 255), font=font(15))
sheet.save(stem.with_suffix('.png'))
frames = []; total = max(n for _, n, _ in rows) * 2
for t in range(0, total, 3):
    f = Image.new('RGB', (len(rows) * cs, cs + 22), (20, 18, 16)); d2 = ImageDraw.Draw(f)
    for r, (name, n, g) in enumerate(rows):
        k = t % n; f.paste(Image.fromarray(g[:, k * cs:(k + 1) * cs]), (r * cs, 22)); d2.text((r * cs + 4, 3), name, fill=(240, 240, 240), font=font(12))
    frames.append(f)
frames[0].save(stem.with_suffix('.gif'), save_all=True, append_images=frames[1:], duration=60, loop=0)
print(stem.with_suffix('.png'), sheet.size, stem.with_suffix('.gif'), len(frames), 'frames')
