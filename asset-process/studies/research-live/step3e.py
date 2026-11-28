"""step3e.py: the eight Meridian inventory icons from the client's own icon function BY ITEM ID (item_icon.rs Icons::get
on the PACKED cache: definition from archive 2 group 10, inventory model from archive 7), against the loose route
(research_items.exe icons def:FILE.def+model:FILE on the SOURCE cache, the replica every icon picture has used).
usage: step3e.py
"""
import sys, pathlib, shutil
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from livelib import *
OUT = ROOT / 'render' / 'research-live-3e'
if OUT.exists(): shutil.rmtree(OUT)
OUT.mkdir(parents=True)
C = HERE.parent / 'research-items' / 'completed'
NAMES = ['longsword', 'glaive', 'staff', 'cuirass', 'gauntlets', 'legs', 'boots', 'barbute']
total = 0; cells = []
for bg in ('3e3529', '202020'):
    a = OUT / f'icons-byid-{bg}.bmp'; b = OUT / f'icons-loose-{bg}.bmp'
    o1 = run('research_items', PACKED, 'icons', a, 'outline=1', 'shadow=302020', f'bg={bg}', *[f'item:{60000 + i}' for i in range(8)])
    o2 = run('research_items', SRC, 'icons', b, 'outline=1', 'shadow=302020', f'bg={bg}', *[f'def:{C / n / "item.def"}+model:{C / n / "inventory.bin"}' for n in NAMES])
    r = diff(b, a); total += r[0]
    line = f'icons on background {bg}: by id vs loose {r[0]} differing of {r[1]} pixels'
    print(line); LOG.append(line)
    if bg == '3e3529':
        print(o1.strip())
        for im, cap in ((b, 'loose definition + model files (replica)'), (a, f'BY ITEM ID 60000..60007, Icons::get, packed cache: {r[0]} differing of {r[1]}')):
            big = Image.open(im).convert('RGB'); big = big.resize((big.width * 5, big.height * 5), Image.NEAREST)
            cells.append((big, cap))
# real items beside ours, all through Icons::get on the packed cache
mix = OUT / 'mixed.bmp'
run('research_items', PACKED, 'icons', mix, 'outline=1', 'shadow=302020', 'bg=3e3529', 'item:1303', 'item:60000', 'item:3204', 'item:60001', 'item:1381', 'item:60002',
    'item:1127', 'item:60003', 'item:7462', 'item:60004', 'item:1079', 'item:60005', 'item:4131', 'item:60006', 'item:1147', 'item:60007')
big = Image.open(mix).convert('RGB'); big = big.resize((big.width * 3, big.height * 3), Image.NEAREST)
cells.append((big, 'packed cache, Icons::get: real item then ours, pair by pair (rune longsword|60000, dragon halberd|60001, staff of air|60002, ...)'))
print(run('research_items', PACKED, 'verify', *[str(60000 + i) for i in range(8)]).strip()[-600:])
board(ROOT / 'render' / 'research-live-board-3e.png', cells, 1, title='3e inventory icons')
LOG.append(f'TOTAL 3e: {total} differing pixels'); print(LOG[-1]); save_log(HERE / 'step3e.log')
