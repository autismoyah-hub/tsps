"""Blind line-up of creatures IN MOTION: real against rebuild, shuffled, the key kept aside.

usage: lineup_bone.py OUTDIR PAIR [PAIR...] [--seed N] [--zoom-game] [--cell WxH] [--yaw N] [--ms N] [--control]
PAIR = REAL_NPC_ID:OURS.json        one real creature and its rebuild (drawn under the real one's sequences)
     | REAL_NPC_ID:REAL_NPC_ID      a control pair: two real creatures (use the same id twice for an identical pair)
OUTDIR must not exist. Writes:
  OUTDIR/lineup.gif    every pair as two neighbouring columns in a shuffled left/right order, numbered 1, 2, 3 ...;
                       all play stand, walk, attack, death together at the sequences' own average frame time, loop
  OUTDIR/lineup.png    stills: one row per sampled frame (bind, then 3 frames of each sequence), the same columns
  OUTDIR/lineup-game.png  the same stills at game size (zoom 1150, pitch 220)
  OUTDIR/KEY.txt       which number is which. KEEP IT FROM THE JUDGE (the review page shows only the pictures).
Every column is drawn by the NPC harness (the client's lighting and painting); a pair shares zoom, aim and frames.
"""
import sys, pathlib, random, json
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import bone_rebuild_lib as BL
L = BL.L
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
out = pathlib.Path(a[1])
if out.exists(): raise SystemExit(f'{out} exists: a line-up is never overwritten')
pairs = [x for x in a[2:] if ':' in x and not x.startswith('--') and x.split(':')[0].isdigit()]
seed = int(opt('--seed', 239)); rng = random.Random(seed); yaw = int(opt('--yaw', 192)); cw, ch = (int(v) for v in opt('--cell', '300x300').split('x'))
out.mkdir(parents=True); work = out / 'defs'
def font(n):
    try: return ImageFont.truetype('segoeuib.ttf', n)
    except Exception: return ImageFont.load_default()
columns = []; key = [f'seed {seed} yaw {yaw}']; number = 1
strips = {'gif': [], 'still': [], 'game': []}
for pair in pairs:
    rid, _, other = pair.partition(':'); rid = int(rid); seqs = L.sequences(rid)
    lo, hi, m, sc = L.size_after_scale(rid); D = float((hi - lo).max()); aim = int((hi[1] - lo[1]) / 2)
    real = L.entry_real(rid, seqs)
    ours = L.entry_real(int(other), seqs) if other.isdigit() else L.entry_ours(pathlib.Path(other).resolve(), seqs, work)
    order = [('REAL', real), ('CONTROL real' if other.isdigit() else 'REBUILD', ours)]
    rng.shuffle(order)
    for what, e in order: key.append(f'{number}: {what}  {e if what == "REAL" or other.isdigit() else other}  (real NPC {rid}, sequences {seqs})'); number += 1
    entries = [e for _, e in order]
    zoom = int(512 * D * 1.75 / cw)
    # animation: every frame of each sequence, held for its average frame time
    rows = []; hold = []
    for name in ('stand', 'walk', 'attack', 'death'):
        if name not in seqs: continue
        n, length, _, _ = L.seq_frames(seqs[name])
        for f in range(n): rows.append(f'{name}:{f}'); hold.append(max(20, int(20 * length / max(n, 1))))
        hold[-1] += 500
    g, _ = L.draw(entries, rows, zoom, yaw, 128, (cw, ch), aim)
    strips['gif'].append((L.cells(g, 2, len(rows)), rows, hold))
    still_rows = L.sample_rows(seqs, 3)
    s, _ = L.draw(entries, still_rows, zoom, yaw, 128, (cw, ch), aim); strips['still'].append((L.cells(s, 2, len(still_rows)), still_rows))
    gs, _ = L.draw(entries, still_rows, 1150, yaw, 220, (max(140, int(D * .445 * 1.6)), max(140, int(D * .445 * 1.6))), aim); strips['game'].append((L.cells(gs, 2, len(still_rows)), still_rows))
ncol = 2 * len(pairs)
def numbered(im, top=26):
    d = ImageDraw.Draw(im)
    w = im.width // ncol
    for k in range(ncol): d.text((k * w + 8, 2), str(k + 1), fill=(245, 245, 245), font=font(20))
    return im
# stills
for tag, name in (('still', 'lineup.png'), ('game', 'lineup-game.png')):
    nrows = max(len(r) for _, r in strips[tag]); cellh = strips[tag][0][0][0][0].shape[0]; widths = [c[0][0].shape[1] for c, _ in strips[tag]]
    W = 2 * sum(widths); im = Image.new('RGB', (W, 26 + nrows * cellh), (20, 18, 16)); x = 0
    for (cells, rws), w in zip(strips[tag], widths):
        for r, row in enumerate(cells):
            for c in range(2): im.paste(Image.fromarray(row[c]), (x + c * w, 26 + r * row[c].shape[0]))
        x += 2 * w
    d = ImageDraw.Draw(im); k = 0; x = 0
    for (cells, rws), w in zip(strips[tag], widths):
        for c in range(2): d.text((x + c * w + 8, 2), str(k + 1), fill=(245, 245, 245), font=font(20)); k += 1
        x += 2 * w
    im.save(out / name)
# animation: a common clock in 20 ms steps; each pair loops its own cycle
frames = []; total = max(sum(h) for _, _, h in strips['gif']); step = int(opt('--ms', 60))
for t in range(0, total, step):
    f = Image.new('RGB', (ncol * cw, ch + 26), (20, 18, 16))
    for p, (cells, rows, hold) in enumerate(strips['gif']):
        tt = t % sum(hold); i = 0
        while tt >= hold[i]: tt -= hold[i]; i += 1
        for c in range(2): f.paste(Image.fromarray(cells[i][c]), ((2 * p + c) * cw, 26))
    frames.append(numbered(f))
frames[0].save(out / 'lineup.gif', save_all=True, append_images=frames[1:], duration=step, loop=0, optimize=False)
(out / 'KEY.txt').write_text('\n'.join(key) + '\n', encoding='utf-8')
import shutil; shutil.rmtree(work, ignore_errors=True)
print(out / 'lineup.gif', f'{len(frames)} frames of {step} ms, {ncol} columns;', out / 'lineup.png', ';', out / 'lineup-game.png', '; key in', out / 'KEY.txt')
