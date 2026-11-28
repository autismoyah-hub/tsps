"""Print rooms found near a world position: peek.py X Y [RADIUS]"""
import sys, json, collections, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rlib as R
rooms = json.loads((R.HERE / 'rooms-all.json').read_text())
X, Y = int(sys.argv[1]), int(sys.argv[2]); rad = int(sys.argv[3]) if len(sys.argv) > 3 else 40
for i, r in enumerate(rooms):
    bx, by = r['base']
    cx = bx + sum(t[0] for t in r['tiles']) / r['area']; cy = by + sum(t[1] for t in r['tiles']) / r['area']
    if abs(cx - X) > rad or abs(cy - Y) > rad: continue
    c = collections.Counter(f"{R.name(it[0])}[{it[5]}]" for it in r['items'] if it[5] not in ('wall',))
    print(f"#{i} ({cx:.0f},{cy:.0f}) p{r['plane']} {r['w']}x{r['h']} area {r['area']} doors {r['doors']} open {len(r['openings'])} roof {r['roofflag']}/{r['roofed']}")
    print('    ', ', '.join(f'{k} x{v}' if v > 1 else k for k, v in c.most_common()))
