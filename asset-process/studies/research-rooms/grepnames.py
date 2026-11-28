"""Which rooms contain an object whose name matches a pattern: grepnames.py REGEX [N]"""
import sys, re, json, collections, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rlib as R
rooms = json.loads((R.HERE / 'rooms.json').read_text())
pat = re.compile(sys.argv[1], re.I); N = int(sys.argv[2]) if len(sys.argv) > 2 else 40
hits = collections.Counter(); where = collections.defaultdict(list)
for r in rooms:
    ns = {R.name(it[0]) for it in r['items']}
    for n in ns:
        if pat.search(n): hits[n] += 1; where[n].append(r['id'])
for n, c in hits.most_common(N):
    print(f'{n!r} {c} rooms e.g. ' + ', '.join(f"#{i} {rooms[i]['place']} {rooms[i]['kind']} d{sum(1 for it in rooms[i]['items'] if it[3]==9)}" for i in where[n][:4]))
