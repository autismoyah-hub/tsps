"""make_boards.py: the boards for the owner (PNG), from files the comparisons already wrote.

board-01-five-reference.png : the five reference interfaces, as TSPS drew them | as iface.py draws the same state
board-02-more-panels.png    : eight more panels, TSPS | iface.py (each 0 differing pixels)
board-03-fixed-frame.png    : iface.py drawing the fixed-mode game frame with Pet list (210) open, beside the tooling client
board-04-proof.png          : the proof: real 148 in TSPS | our rebuild delivered to TSPS | our rebuild drawn by iface.py
"""
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
GAP = 6


def label(im, text):
    w = max(im.width, 8 * len(text) + 8)
    out = Image.new('RGB', (w, im.height + 16), (20, 20, 20))
    out.paste(im, (0, 16))
    ImageDraw.Draw(out).text((4, 2), text, fill=(230, 230, 230))
    return out


def row(images):
    h = max(i.height for i in images)
    w = sum(i.width for i in images) + GAP * (len(images) - 1)
    out = Image.new('RGB', (w, h), (20, 20, 20)); x = 0
    for i in images:
        out.paste(i, (x, 0)); x += i.width + GAP
    return out


def col(images):
    w = max(i.width for i in images)
    h = sum(i.height for i in images) + GAP * (len(images) - 1)
    out = Image.new('RGB', (w, h), (20, 20, 20)); y = 0
    for i in images:
        out.paste(i, (0, y)); y += i.height + GAP
    return out


def pair(prefix, title, scale=1):
    a = Image.open(f'{prefix}-tsps.png'); b = Image.open(f'{prefix}-ours.png')
    if scale != 1:
        a = a.resize((a.width * scale, a.height * scale), Image.NEAREST)
        b = b.resize((b.width * scale, b.height * scale), Image.NEAREST)
    return label(row([label(a, 'TSPS client'), label(b, 'iface.py')]), title)


def main():
    t = HERE / 'tsps'
    five = [
        pair(t / 'hud2' / 'cmp-303-hud5', 'HPBAR_HUD 303 (boss health overlay), TSPS live state: 0 px differ (x3)', 3),
        pair(t / 'modal' / 'cmp' / '210', 'POH_PETLIST 210 (framed panel): 0 px differ'),
        pair(t / 'modal' / 'cmp' / '155', 'BARROWS_REWARD 155 (reward chest): 16 px differ (slot area), chest model not drawn'),
        pair(t / 'hud' / 'cmp-231-final', 'CHAT_LEFT 231 (dialogue): text 0 px differ; chat head model and chatbox parchment not compared'),
        pair(t / 'hud' / 'cmp-160', 'ORBS 160: frames and icons match; 546 px differ where scripts recoloured numbers and filled orbs'),
    ]
    col(five).save(HERE / 'board-01-five-reference.png')
    more = [pair(t / 'modal' / 'cmp' / str(g), f'{g}: 0 px differ') for g in (539, 868, 648, 156, 476, 4, 383, 887)]
    col([row(more[0:2]), row(more[2:4]), row(more[4:6]), row(more[6:8])]).save(HERE / 'board-02-more-panels.png')
    a = Image.open(HERE / 'state' / 'py-fixed-210.png'); b = Image.open(HERE / 'state' / 'rust-fixed-210.png')
    row([label(a, 'iface.py: fixed frame 548 + chatbox 162 + orbs 160 + Pet list 210'),
         label(b, 'tooling client (same state): 0 of 384,795 px differ')]).save(HERE / 'board-03-fixed-frame.png')
    p = HERE / 'proof'
    full = np.asarray(Image.open(p / 'board-delivered.png'))
    real, deliv, ours = full[:, :400], full[:, 406:806], full[:, 812:1212]
    row([label(Image.fromarray(real), 'real PET_INSURANCE 148 in TSPS'),
         label(Image.fromarray(deliv), 'our rebuild, served to TSPS as 30900'),
         label(Image.fromarray(ours), 'our rebuild drawn by iface.py')]).save(HERE / 'board-04-proof.png')
    print('boards written')


if __name__ == '__main__':
    main()
