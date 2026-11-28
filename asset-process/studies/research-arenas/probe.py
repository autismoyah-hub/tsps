"""probe.py: look at a piece of the real map before calling it an arena.

  probe.py plan X Y PLANE [RADIUS] [OUT.png]   a plan of the window: floor kinds in their own colours, blocked tiles hatched,
                                               wall edges white, the walkable region from (X, Y) outlined; prints what is placed
  probe.py find REGEX [x0 y0 x1 y1]            every placed object whose name or gameval symbol matches (index cached)
  probe.py ids ID[,ID..]                       where those object ids are placed
"""
import sys, re, pickle, collections, pathlib
sys.dont_write_bytecode = True
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import arenalib as A
R = A.R
from PIL import Image, ImageDraw


def index():
    p = A.HERE / 'cache' / 'locindex.pkl'
    if p.exists(): return pickle.loads(p.read_bytes())
    p.parent.mkdir(exist_ok=True)
    ix = collections.defaultdict(list)
    for g in R.squares():
        try: _, _, locs = R.square(g)
        except Exception: continue
        rx, ry = g >> 8, g & 255
        for oid, pl, x, y, sh, rot in locs: ix[oid].append((rx * 64 + x, ry * 64 + y, pl, sh, rot))
    ix = dict(ix); p.write_bytes(pickle.dumps(ix)); return ix


def plan(W, S, path, cell=10, marks=(), title=''):
    x0, y0, x1, y1 = W.x0, W.y0, W.x1, W.y1
    im = Image.new('RGB', ((x1 - x0 + 1) * cell, (y1 - y0 + 1) * cell + 14), (0, 0, 0)); dr = ImageDraw.Draw(im)
    P = lambda x, y: ((x - x0) * cell, (y1 - y) * cell)
    for (x, y), t in W.t.items():
        a = P(x, y); k = W.kind((x, y))
        c = W.kind_rgb(k) if (t[1] or t[2]) else (0, 0, 0)
        if W.kind_texture(k) is not None and c == (0, 0, 0): c = (70, 60, 90)
        dr.rectangle([a[0], a[1], a[0] + cell - 1, a[1] + cell - 1], fill=c)
        b = W.blocked.get((x, y))
        if b == 'terrain': dr.line([a[0], a[1], a[0] + cell - 1, a[1] + cell - 1], fill=(200, 40, 40))
        elif b: dr.rectangle([a[0] + 2, a[1] + 2, a[0] + cell - 3, a[1] + cell - 3], fill=(120, 90, 50), outline=(0, 0, 0))
    for (x, y, d) in W.edges:
        a = P(x, y)
        seg = {0: [a[0], a[1], a[0], a[1] + cell], 2: [a[0] + cell - 1, a[1], a[0] + cell - 1, a[1] + cell],
               1: [a[0], a[1], a[0] + cell, a[1]], 3: [a[0], a[1] + cell - 1, a[0] + cell, a[1] + cell - 1]}[d]
        dr.line(seg, fill=(255, 255, 255))
    for x, y in S:
        for d, (dx, dy) in enumerate(A.DIRS):
            if (x + dx, y + dy) in S: continue
            a = P(x, y)
            seg = {0: [a[0], a[1], a[0], a[1] + cell], 2: [a[0] + cell - 1, a[1], a[0] + cell - 1, a[1] + cell],
                   1: [a[0], a[1], a[0] + cell, a[1]], 3: [a[0], a[1] + cell - 1, a[0] + cell, a[1] + cell - 1]}[d]
            dr.line(seg, fill=(255, 255, 0), width=2)
    for (x, y, s, c) in marks:
        a = P(x, y + s - 1); dr.rectangle([a[0], a[1], a[0] + s * cell - 1, a[1] + s * cell - 1], outline=c, width=2)
    dr.text((2, im.size[1] - 13), title, fill='white')
    im.save(path); return im


def main():
    cmd = sys.argv[1]
    if cmd == 'plan':
        x, y, pl = map(int, sys.argv[2:5]); r = int(sys.argv[5]) if len(sys.argv) > 5 else 30
        W = A.World(x - r, y - r, x + r, y + r, pl)
        try: S = W.floor((x, y))
        except SystemExit as e: print(e); S = set()
        out = sys.argv[6] if len(sys.argv) > 6 else str(A.HERE / 'probe' / f'plan-{x}-{y}-{pl}.png')
        pathlib.Path(out).parent.mkdir(exist_ok=True, parents=True)
        plan(W, S, out, title=f'({x},{y}) plane {pl} r {r}; walkable {len(S)}')
        print('walkable', len(S), '->', out)
        c = collections.Counter((oid, sh) for oid, *_r, sh, rot in W.locs)
        for (oid, sh), n in sorted(c.items(), key=lambda kv: -kv[1]):
            d = R.resolved(oid) or {}
            print(f'{n:4} x {oid:6} shape {sh:2} {d.get("name")!r:28} size {d.get("size")} interact {d.get("interact")} anim {d.get("anim")} ops {d.get("ops")} h {R.height(oid)}')
    elif cmd == 'find':
        rx = re.compile(sys.argv[2], re.I); box = list(map(int, sys.argv[3:7])) if len(sys.argv) > 6 else None
        D = R.defs(); ix = index()
        for oid, pl in sorted(ix.items()):
            d = R.resolved(oid) or {}
            if not rx.search(d.get('name', '')): continue
            q = [p for p in pl if not box or (box[0] <= p[0] <= box[2] and box[1] <= p[1] <= box[3])]
            if q: print(oid, repr(d.get('name')), 'size', d.get('size'), len(q), 'placed; first', q[:6])
    elif cmd == 'ids':
        ix = index()
        for oid in map(int, sys.argv[2].split(',')):
            print(oid, repr((R.resolved(oid) or {}).get('name')), ix.get(oid, [])[:12], len(ix.get(oid, [])))


if __name__ == '__main__':
    main()
