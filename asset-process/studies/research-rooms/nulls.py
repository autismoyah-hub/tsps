"""What the unnamed ('null') objects in a few rooms are: nulls.py ROOMINDEX..."""
import sys, json, collections, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rlib as R
rooms = json.loads((R.HERE / 'rooms-all.json').read_text())
c = collections.Counter()
for i in map(int, sys.argv[1:]):
    for it in rooms[i]['items']:
        if R.name(it[0]) == 'null': c[(it[0], it[3], it[5])] += 1
for (o, sh, k), n in c.most_common(60):
    d = R.resolved(o)
    print(o, 'shape', sh, k, 'x', n, 'sym', R.symbol(o), 'h', R.height(o), d['size'], 'interact', d['interact'], d['models'][:3])
