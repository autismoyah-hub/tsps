"""Step 4 board: Minotaur, Tekton, the watcher as the owner saw it, the repainted watcher; standing, walk, attack;
game size and close; front and back. Every cell by the client's harness (research-npc/npc.py).
usage: board_watcher.py OUTDIR [REPAINT.json]"""
import sys, pathlib, subprocess
from PIL import Image, ImageDraw, ImageFont
HERE = pathlib.Path(__file__).resolve().parent
B = HERE.parent
out = pathlib.Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
rep = pathlib.Path(sys.argv[2]) if len(sys.argv) > 2 else HERE / 'repaint' / 'watcher-2-repaint.json'
E = ['npc:12812', 'npc:7541', 'def:' + str(B / 'creature-stone-watcher-2' / 'creature-stone-watcher-2.json'), 'def:' + str(rep.resolve())]
NAMES = ['Minotaur 12812 (real)', 'Tekton 7541 (real)', 'watcher as the owner saw it', 'watcher repainted to the plan']
ROWS = 'stand:0,walk:3,attack:4'
def font(n):
    try: return ImageFont.truetype('segoeui.ttf', n)
    except Exception: return ImageFont.load_default()
def draw(name, extra):
    p = out / f'{name}.png'
    r = subprocess.run([sys.executable, '-I', str(B / 'research-npc' / 'npc.py'), str(p), ','.join(E), '--rows', ROWS] + extra, capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    return Image.open(p).convert('RGB')
views = [('game size, front', ['--game', '--cell', '250x300']), ('game size, back', ['--game', '--cell', '250x300', '--yaw', '1024']),
         ('close, front', ['--zoom', '900', '--cell', '400x470']), ('close, back', ['--zoom', '900', '--cell', '400x470', '--yaw', '1024'])]
ims = [(t, draw(t.replace(', ', '-').replace(' ', '-'), x)) for t, x in views]
for pair, tag in ((ims[:2], 'game'), (ims[2:], 'close')):
    W = sum(im.width for _, im in pair) + 20; H = max(im.height for _, im in pair) + 70
    board = Image.new('RGB', (W, H), (20, 18, 16)); d = ImageDraw.Draw(board); x = 0
    for t, im in pair:
        board.paste(im, (x, 70)); d.text((x + 8, 4), t, fill=(240, 220, 150), font=font(20))
        cw = im.width / len(E)
        for i, n in enumerate(NAMES): d.text((x + i * cw + 6, 38), f'{i + 1}  {n}', fill=(235, 235, 235), font=font(13 if tag == 'game' else 15))
        x += im.width + 20
    board.save(out / f'board-{tag}.png'); print(out / f'board-{tag}.png', board.size)
