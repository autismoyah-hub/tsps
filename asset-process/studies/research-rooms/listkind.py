"""List rooms of one kind: listkind.py KIND [N]"""
import sys, json, collections, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rlib as R
rooms = json.loads((R.HERE / 'rooms.json').read_text())
K = sys.argv[1]; N = int(sys.argv[2]) if len(sys.argv) > 2 else 60
sel = [r for r in rooms if r['kind'] == K]
sel.sort(key=lambda r: -r['votes'].get(K, 0))
for r in sel[:N]:
    c = collections.Counter(R.name(it[0]) for it in r['items'] if it[5] in ('furniture', 'decor', 'light', 'light-wall', 'clutter', 'floor', 'window') and R.name(it[0]) != 'null')
    print(f"#{r['id']} {r['place']} ({r['era']}) {r['centre']} p{r['plane']} {r['w']}x{r['h']} a{r['area']} v{r['votes'].get(K)} d{r['doors']} o{len(r['openings'])} | " +
          ', '.join(f'{k} x{v}' if v > 1 else k for k, v in c.most_common(14)))
