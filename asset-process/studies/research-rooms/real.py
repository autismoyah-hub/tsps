"""Turn an extracted map room (rooms.json entry) into the measure.py room form."""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rlib as R
from extract import edge_key, blocks_edges


def normalize(r):
    tiles = [tuple(t) for t in r['tiles']]
    S = set(tiles)
    bnd = [(x, y, d) for x, y in tiles for d, (dx, dy) in enumerate([(-1, 0), (0, 1), (1, 0), (0, -1)]) if (x + dx, y + dy) not in S]
    bkey = {edge_key(*b): b for b in bnd}
    doors = set()
    for o in r['openings']:
        k = tuple(o)
        if k in bkey: doors.add(bkey[k])
    objs = []
    for oid, x, y, sh, rot, k in r['items']:
        inside = not k.endswith(':outer'); k = k.split(':')[0]
        d = R.resolved(oid) or {}
        name = d.get('name', 'null')
        if sh in (0, 2):
            ks = [edge_key(x, y, e) for e in blocks_edges(sh, rot)]
            if k == 'door':
                for kk in ks:
                    if kk in bkey: doors.add(bkey[kk])
        sx, sz = d.get('size', [1, 1])
        if sh in (10, 11) and rot % 2 == 1: sx, sz = sz, sx
        if sh not in (10, 11): sx = sz = 1
        cls = k
        h = R.height(oid)
        if name == 'null':
            # unnamed wall dressing: a piece running the tile's whole width is architecture (rails, beams, frames);
            # a narrower one is a fitting a player sees as decoration (sconces, plaques, brackets, hangings)
            if k in ('decor', 'light-wall'): cls = 'trim' if R.span(oid) >= 100 else 'decor'
            elif k == 'furniture': cls = 'scenery-unnamed'
        if k == 'window' and sh in (0, 2): cls = 'window'
        obj = dict(id=oid, name=name, cls=cls, func=R.func_class(name), x=x, y=y, w=sx, d=sz, rot=rot, shape=sh,
                   block=d.get('interact', 2) != 0, h=h, inside=inside, ops=d.get('ops', []))
        if cls == 'window' and sh in (0, 2):
            # a window wall: place it on the boundary edge it stands on
            for e in blocks_edges(sh, rot):
                b = bkey.get(edge_key(x, y, e))
                if b: obj.update(x=b[0], y=b[1], rot=b[2])
        objs.append(obj)
    return dict(tiles=tiles, doors=sorted(doors), objects=objs)
