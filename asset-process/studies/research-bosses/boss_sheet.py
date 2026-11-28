"""Contact sheets of the bosses: one strip per boss = game size standing | close standing | three frames of an attack.

usage: boss_sheet.py OUTDIR [SET ...]     SET = cox tob toa own other (default all); needs data/bosses.json
Drawn by asset_npc.exe draw (the client's combine, light-once-at-rest, animate, scale, depth-tested painting).
Game size = zoom 1150, pitch 220 (a person is about 90 pixels tall); a boss wider than the cell is cropped.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, subprocess, hashlib
import numpy as np
from PIL import Image, ImageDraw, ImageFont
HERE = pathlib.Path(__file__).parent
EXE = _AP.fix(r'<CLIENT_BIN>\asset_npc.exe')
CACHE = _AP.fix(r'<OSRS_CACHE_239>')
TILES = HERE / 'data' / 'tiles'; TILES.mkdir(parents=True, exist_ok=True)
BG = (0x30, 0x2b, 0x25)
SEQS = {j['id']: j for j in (json.loads(l) for l in open(_AP.fix(r'<ROOT>\studies\research-npc\census\sequences.jsonl')))}
def font(n):
    try: return ImageFont.truetype('segoeui.ttf', n)
    except Exception: return ImageFont.load_default()
def render(entry, rows, zoom, pitch, yaw, cell):
    name = hashlib.md5(f'{entry}|{rows}|{zoom}|{pitch}|{yaw}|{cell}'.encode()).hexdigest()[:16]
    png = TILES / (name + '.png')
    if not png.exists():
        bmp = TILES / (name + '.bmp')
        r = subprocess.run([EXE, CACHE, 'draw', str(bmp), 'rows=' + ','.join(rows), f'zoom={zoom}', f'pitch={pitch}', f'yaw={yaw}', f'cell={cell[0]}x{cell[1]}', 'aim=own', entry], capture_output=True, text=True)
        if r.returncode: raise SystemExit(r.stdout + r.stderr)
        (TILES / (name + '.txt')).write_text(r.stdout)
        Image.open(bmp).convert('RGB').save(png); bmp.unlink()
    im = Image.open(png).convert('RGB')
    return [im.crop((0, k * cell[1], cell[0], (k + 1) * cell[1])) for k in range(len(rows))]
def crop_to(im, w, h):
    a = np.asarray(im); mask = (a != np.array(BG, dtype=a.dtype)).any(2)
    out = Image.new('RGB', (w, h), BG)
    if not mask.any(): return out
    ys, xs = np.nonzero(mask); cx, cy = (xs.min() + xs.max()) // 2, (ys.min() + ys.max()) // 2
    big = Image.new('RGB', (im.width + 2 * w, im.height + 2 * h), BG); big.paste(im, (w, h))
    out.paste(big.crop((cx + w - w // 2, cy + h - h // 2, cx + w - w // 2 + w, cy + h - h // 2 + h)), (0, 0)); return out
def strip(b, GW=420, CW=300, H=320, yaw=192):
    entry = b.get('entry') or f"npc:{b['id']}"
    if entry.startswith('def:'):
        sys.path.insert(0, _AP.fix(r'<ROOT>\studies\research-npc')); import npc
        (TILES / 'defs').mkdir(exist_ok=True); entry = 'def:' + str(npc.to_npcdef(pathlib.Path(entry[4:]), TILES / 'defs'))
    me = b.get('measure', {}); D = max(me.get('size_x', 200), me.get('size_y', 200), me.get('size_z', 200))
    seqname = None; rows = ['stand:0' if b['stand'] >= 0 else 'bind']
    for key in ('attack', 'death'):
        s = b.get(key)
        if s and s in SEQS and SEQS[s]['frames'] > 1:
            n = SEQS[s]['frames']; seqname = key; rows += [f'seq:{s}:{int(n * f)}' for f in (0.25, 0.5, 0.75)]; break
    if seqname is None and b['stand'] >= 0:
        n = SEQS[b['stand']]['frames']; seqname = 'stand'; rows += [f"seq:{b['stand']}:{int(n * f)}" for f in (0.25, 0.5, 0.75)]
    zoom = max(120, int(512 * D * 0.72 / (CW * 0.74)))
    close = [crop_to(c, CW, H) for c in render(entry, rows, zoom, 128, yaw, (CW + 300, H + 300))]
    g = max(GW, int(D * 0.445 * 1.6) + 60)
    game = crop_to(render(entry, rows[:1], 1150, 220, yaw, (g, g))[0], GW, H)
    im = Image.new('RGB', (GW + CW * 4, H + 36), BG); im.paste(game, (0, 0))
    for k, c in enumerate(close): im.paste(c, (GW + k * CW, 0))
    d = ImageDraw.Draw(im)
    d.text((4, H + 1), f"{b['label']}  (npc {b['id']})  size {b['size']} tiles  {b.get('faces_files', me.get('faces'))} faces  {me.get('colours', '?')} colours  "
           f"{'BONE WEIGHTS' if b.get('weighted') else 'labels'}  base {b.get('skeleton')}  ambient {b.get('ambient')} contrast {b.get('contrast')}", fill=(235, 235, 235), font=font(14))
    src = b.get('anims_from', '')
    d.text((4, H + 18), 'game size, standing | close, standing | ' + (f"{seqname} {b.get(seqname)} at 1/4, 1/2, 3/4" if seqname else 'no sequence') +
           (f"   [{'INFERRED' if 'INFERRED ' + str(seqname) in src else src}]" if seqname in ('attack', 'death') else ''), fill=(170, 170, 150), font=font(11))
    for k in range(4): d.line([(GW + k * CW, 0), (GW + k * CW, H)], fill=(70, 64, 56))
    return im
def main():
    out = pathlib.Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
    B = json.loads((HERE / 'data' / 'bosses.json').read_text()); sets = sys.argv[2:] or ['cox', 'tob', 'toa', 'own', 'other']
    title = dict(cox='Chambers of Xeric', tob='Theatre of Blood', toa='Tombs of Amascut', own="The owner's other named bosses: Nex, Nightmare, Doom, Maggot King", other='Other end-game bosses')
    for s in sets:
        strips = [strip(b) for b in B.values() if b['set'] == s]
        for part in range(0, len(strips), 6):
            chunk = strips[part:part + 6]; w, h = chunk[0].size
            im = Image.new('RGB', (w, h * len(chunk) + 30), (20, 18, 16)); d = ImageDraw.Draw(im)
            d.text((8, 5), f'{title[s]} ({part // 6 + 1})', fill=(240, 240, 240), font=font(17))
            for i, t in enumerate(chunk): im.paste(t, (0, 30 + i * h))
            f = out / f'bosses-{s}-{part // 6 + 1}.png'; im.save(f); print(f, im.size)
if __name__ == '__main__':
    main()
