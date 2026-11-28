"""Contact sheets of creatures, each drawn by the NPC harness (the client's own lighting and painting) twice:
CLOSE (the figure fills the tile) and at GAME SIZE (zoom 1150, pitch 220: a person is about 90 pixels tall).

usage: contact.py OUT.png KEY[,KEY...] [--cols N] [--tile PX] [--game-box H] [--title TEXT] [--yaw N] [--no-labels]
       contact.py --era OUTDIR          the standard era sheets (E1, E2, R by body plan) and the like-for-like sheets
KEY = a key of data/era.json (an NPC id as text, or hound / hound-1 / spider / boss-1 / boss-2), or npc:ID, or
      def:PATH.json. Tiles are cached under data/tiles/.
"""
import sys, pathlib, subprocess, json, hashlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from PIL import Image, ImageDraw, ImageFont
import cmesh
HERE = pathlib.Path(__file__).parent
TILES = HERE / 'data' / 'tiles'
BG = (0x30, 0x2b, 0x25)
def font(n):
    try: return ImageFont.truetype('segoeui.ttf', n)
    except Exception: return ImageFont.load_default()
def render(entry, zoom, pitch, yaw, cell, row='bind'):
    TILES.mkdir(parents=True, exist_ok=True)
    name = hashlib.md5(f'{entry}|{zoom}|{pitch}|{yaw}|{cell}|{row}'.encode()).hexdigest()[:16]
    png = TILES / (name + '.png')
    if not png.exists() or entry.startswith('def:'):
        bmp = TILES / (name + '.bmp')
        r = subprocess.run([cmesh.EXE, cmesh.CACHE, 'draw', str(bmp), f'rows={row}', f'zoom={zoom}', f'pitch={pitch}', f'yaw={yaw}', f'cell={cell[0]}x{cell[1]}', 'aim=own',
                            cmesh.entry_to_exe(entry, TILES / 'defs')], capture_output=True, text=True)
        if r.returncode: raise SystemExit(r.stdout + r.stderr)
        Image.open(bmp).convert('RGB').save(png); bmp.unlink()
    return Image.open(png).convert('RGB')
def crop_to(im, w, h):
    """Centre the figure (non-background pixels) in a w x h box, cropping if it is larger."""
    import numpy as np
    a = np.asarray(im); mask = (a != np.array(BG, dtype=a.dtype)).any(2)
    out = Image.new('RGB', (w, h), BG)
    if not mask.any(): return out
    ys, xs = np.nonzero(mask); cx, cy = (xs.min() + xs.max()) // 2, (ys.min() + ys.max()) // 2
    big = Image.new('RGB', (im.width + 2 * w, im.height + 2 * h), BG); big.paste(im, (w, h))
    box = (cx + w - w // 2, cy + h - h // 2, cx + w - w // 2 + w, cy + h - h // 2 + h)
    out.paste(big.crop(box), (0, 0)); return out
def tile(key, rows, tile_px=300, game_h=200, yaw=192, labels=True):
    r = rows.get(key)
    entry = r['entry'] if r else key
    if r: D = max(r['size_x'], r['size_y'], r['size_z'])
    else:
        import numpy as np
        m = cmesh.load(entry); D = float((m['V'].max(0) - m['V'].min(0)).max())
    zoom = max(120, int(512 * D / (tile_px * 0.74)))
    close = crop_to(render(entry, zoom, 128, yaw, (tile_px + 80, tile_px + 80)), tile_px, tile_px)
    gw = max(tile_px, int(D * 0.445 * 1.5) + 40)
    game = crop_to(render(entry, 1150, 220, yaw, (gw, gw)), tile_px, game_h)
    im = Image.new('RGB', (tile_px, tile_px + game_h + (34 if labels else 0)), BG)
    im.paste(close, (0, 0)); im.paste(game, (0, tile_px))
    d = ImageDraw.Draw(im); d.line([(0, tile_px), (tile_px, tile_px)], fill=(70, 64, 56))
    if labels:
        if r:
            mid = r['model_ids'][0] if r['model_ids'] else 'ours'
            d.text((4, tile_px + game_h + 1), f"{r['name'][:30]}", fill=(235, 235, 235), font=font(14))
            d.text((4, tile_px + game_h + 17), f"{r['group']} npc {key} model {mid}  {r['faces']} faces  {r['colours']} colours", fill=(170, 170, 150), font=font(11))
        else: d.text((4, tile_px + game_h + 1), key[-40:], fill=(235, 235, 235), font=font(13))
    return im
def sheet(out, keys, cols=6, tile_px=300, game_h=200, title=None, yaw=192, labels=True, numbers=False):
    rows = json.loads((HERE / 'data' / 'era.json').read_text()) if (HERE / 'data' / 'era.json').exists() else {}
    tiles = [tile(k, rows, tile_px, game_h, yaw, labels) for k in keys]
    cols = min(cols, len(tiles)); n = -(-len(tiles) // cols); tw, th = tiles[0].size; top = 34 if title else 0
    im = Image.new('RGB', (cols * tw, n * th + top), (20, 18, 16)); d = ImageDraw.Draw(im)
    if title: d.text((8, 6), title, fill=(240, 240, 240), font=font(18))
    for i, t in enumerate(tiles):
        im.paste(t, ((i % cols) * tw, top + (i // cols) * th))
        if numbers: d.text(((i % cols) * tw + 6, top + (i // cols) * th + 4), str(i + 1), fill=(255, 255, 255), font=font(20))
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True); im.save(out); print(out, im.size)
def era(outdir):
    import era as E
    outdir = pathlib.Path(outdir)
    name = dict(E1='EARLY E1: model ids under 12,500 (2004-2005)', E2='LATE CLASSIC E2: model ids 13,000-27,700 (2006-Aug 2007)', R='RECENT R: model ids 36,000 and up (2019 onward)')
    for g, lst in E.GROUPS.items():
        small = [str(i) for i, p in lst if p != 'b']; big = [str(i) for i, p in lst if p == 'b']
        if g == 'R':
            for tag, plans in (('quadrupeds', 'q'), ('spiders', 's'), ('humanoids', 'h')):
                sheet(outdir / f'era-R-{tag}.png', [str(i) for i, p in lst if p in plans], 8, 240, 170, f'{name[g]}: {tag}.  Top: close.  Bottom: game size.')
        else:
            sheet(outdir / f'era-{g}.png', small, 8, 240, 170, f'{name[g]}.  Top: close.  Bottom: game size.')
        sheet(outdir / f'era-{g}-bosses.png', big, 6, 360, 330, f'{name[g]}: bosses and large creatures.  Top: close.  Bottom: game size.')
    q = lambda g: [str(i) for i, p in E.GROUPS[g] if p == 'q']
    sheet(outdir / 'like-quadrupeds.png', q('E1')[:7] + ['hound'] + q('E2')[:7] + ['hound-1'] + q('R')[:8], 8, 240, 170,
          'Quadrupeds. Row 1: early E1 + OUR HOUND (v2). Row 2: late classic E2 + our first hound. Row 3: recent R.')
    sheet(outdir / 'like-wolf-skeleton.png', ['104', '106', '3133', 'hound', '6613', '6473', '12976', '11068'], 8, 300, 200,
          'Wolf-skeleton dogs and recent cats: Hellhound, Wolf, horned Hellhound (2007), OUR HOUND, Skeleton Hellhound (2023), Terror dog, Jaguar, Lynx')
    sheet(outdir / 'like-spiders.png', ['2477', '134', 'spider', '8713', '11175', '13680', '626', '11697'], 8, 300, 200,
          'Spiders: Giant spider, Huge spider (2006-07), OUR SPIDER, Sarachnis, Araxyte, Dreadborn Araxyte, Fever spider, Scarab (recent)')
    sheet(outdir / 'like-bosses.png', ['2215', 'boss-1', 'boss-2', '13599', '12812', '12223', '14147', '11278'], 4, 420, 330,
          'Humanoid bosses: General Graardor (2007), OUR BOSS v1, OUR BOSS v2, Tormented Demon, Minotaur, Vardorvis, Eldric the Ice King, Nex')
if __name__ == '__main__':
    a = sys.argv
    def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
    if '--era' in a: era(opt('--era'))
    else: sheet(a[1], a[2].split(','), int(opt('--cols', 6)), int(opt('--tile', 300)), int(opt('--game-box', 200)), opt('--title'), int(opt('--yaw', 192)), '--no-labels' not in a)
