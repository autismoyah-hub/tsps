"""measure.py: every measure of the study on every real arena of arenas.py -> data/arenas.json (+ data/room-rules.json).

  measure.py [KEY ..]        (pictures must exist: run draw.py first; `--nopic` skips the picture measures)
For each arena: the model (arenamodel), its measures, the picture measures (palette), and ROOMS.md's own measures and
rules run on it as if it were a furnished room (roomcheck.check with kind 'any').
"""
import sys, re, json, math, collections, pathlib
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import arenalib as A, arenamodel as AM, draw, palette
from arenas import ARENAS, BY
R = A.R
sys.path.insert(0, str(A.TF / 'studies' / 'research-rooms'))
import roomcheck, roommeasure

LIGHT = re.compile(r'torch|candle|lamp|lantern|brazier|chandelier|fire|flame|sconce|candelab|beacon|glow|lightorb|lava_?fall|burn', re.I)
ENTRY = re.compile(r'^(pass|enter|exit|leave|quick-?\w*|climb\w*|escape|open|squeeze\w*|jump\w*|cross|use|go-through|walk-through|peek|teleport)', re.I)
UNSEEN = re.compile(r'invis|blankwall|blockrange|greybox', re.I)       # invisible collision objects (their game symbols)
_SYM = None
def sym(oid):
    global _SYM
    if _SYM is None: _SYM = {i: b.decode('latin-1') for i, b in R.CACHE.files(24, 6).items()}
    return _SYM.get(oid, '')


def build(a):
    """The arena model of a real arena (arenamodel's dict) + the World it came from."""
    W, S = draw.floor_of(a)
    H = AM.hull_of(S)
    kind = {t: W.kind(t) for t in S}
    cause = {}; look = {}
    U, Ov = A.floors()
    for t, v in W.t.items():
        void = not (v[1] or v[2]) or (v[2] and Ov.get(v[2] - 1, {}).get('rgb') == 0xff00ff and Ov.get(v[2] - 1, {}).get('texture') is None)
        look[t] = dict(kind=W.kind(t), h=v[0], void=void)
        if t in S: continue
        b = W.blocked.get(t)
        if isinstance(b, tuple) and UNSEEN.search(sym(b[1])): b = 'unseen'       # the game's invisible wall objects
        cause[t] = b if b else ('void' if void else 'clip')
    objs = []
    for oid, x, y, sh, rot in W.locs:
        d = R.resolved(oid) or {}
        if not d.get('models') or UNSEEN.search(sym(oid)): continue
        sx, sz = d.get('size', [1, 1])
        if sh not in (10, 11): sx = sz = 1
        if rot & 1: sx, sz = sz, sx
        nm = d.get('name', 'null'); s = sym(oid)
        it = d.get('interact', 2)
        block = (it != 0) if sh != 22 else (it == 1)
        objs.append(dict(id=oid, name=nm if nm not in ('null', '') else s, sym=s, x=x, y=y, w=sx, d=sz, shape=sh, rot=rot, block=block,
                         h=R.height(oid), anim=d.get('anim') is not None, light=bool(LIGHT.search(nm + ' ' + s)), ops=d.get('ops', [])))
    hfun = lambda x, y: draw.ground(W, x, y)
    ent = []
    for o in objs:
        if any(ENTRY.search(op or '') for op in o['ops']):
            f = AM.fp(o)
            near = [t for t in S if any(max(abs(t[0] - q[0]), abs(t[1] - q[1])) <= 2 for q in f)]
            if near:
                q = f[len(f) // 2]; ent.append(min(near, key=lambda t: abs(t[0] - q[0]) + abs(t[1] - q[1])) + (o['name'], '/'.join(o['ops'])))
    # one entry per object name and side
    seen = {};
    for e in ent: seen.setdefault((e[2], round(e[0] / 6), round(e[1] / 6)), e)
    M = dict(floor=S, kind=kind, hfun=hfun, wall=W.edges, wall_unseen={e for e, oid in W.edges.items() if UNSEEN.search(sym(oid))}, cause=cause, look=look, objects=objs, boss=draw.boss_box(a),
             entries=[(e[0], e[1]) for e in seen.values()], entry_names=[(e[2], e[3], e[0], e[1]) for e in seen.values()])
    return M, W


def as_room(M, W):
    """The arena in roommeasure's room form: tiles = the outline filled plus object-blocked tiles touching it (a room's
    tiles include the tiles under its furniture); doors = the outline edges nearest the entries."""
    H = AM.hull_of(M['floor'])
    T = set(H)
    for t in list(H):
        for dx, dy in AM.DIRS:
            n = (t[0] + dx, t[1] + dy)
            if n not in T and isinstance(M['cause'].get(n), tuple): T.add(n)
    bnd = roommeasure.boundary(sorted(T)); doors = set()
    for e in M['entries']:
        if bnd: doors.add(min(bnd, key=lambda b: abs(b[0] - e[0]) + abs(b[1] - e[1])))
    objs = []
    for o in M['objects']:
        if not set(AM.fp(o)) & T: continue
        k = R.kind(o['id'], o['shape'])
        if k in ('wall', 'roof', 'door'): continue
        nm = (R.resolved(o['id']) or {}).get('name', 'null')
        cls = k
        if nm == 'null':
            if k in ('decor', 'light-wall'): cls = 'trim' if R.span(o['id']) >= 100 else 'decor'
            elif k == 'furniture': cls = 'scenery-unnamed'
        objs.append(dict(id=o['id'], name=nm, cls=cls, func=R.func_class(nm), x=o['x'], y=o['y'], w=o['w'], d=o['d'], rot=o['rot'],
                         shape=o['shape'], block=o['block'], h=o['h'], inside=True, ops=o['ops']))
    return dict(tiles=sorted(T), doors=sorted(doors), objects=objs)


def as_room_all(M, W):
    """The same, but every standing object counted as furniture whether the game names it or not (arena scenery is
    mostly unnamed): the kindest reading for the room rules."""
    r = as_room(M, W)
    for o in r['objects']:
        if o['cls'] == 'scenery-unnamed': o['cls'] = 'furniture'; o['name'] = sym(o['id']) or str(o['id'])
        elif o['cls'] == 'clutter' and o['shape'] in (10, 11) and o['h'] >= 40: o['cls'] = 'furniture'; o['name'] = sym(o['id']) or str(o['id'])
    return r


def colours(M, W):
    out = []
    cnt = collections.Counter(M['kind'][t] for t in M['floor'])
    for k, n in cnt.most_common(5):
        rgb = W.kind_rgb(k); tex = W.kind_texture(k)
        out.append(dict(kind=('overlay ' if k[0] == 'o' else 'underlay ') + str(k[1] - 1), share=round(n / len(M['floor']), 3), rgb='#%02x%02x%02x' % rgb, texture=tex))
    return out


def one(a, pictures=True):
    M, W = build(a)
    m = AM.measure(M)
    m['key'] = a['key']; m['name'] = a['name']; m['group'] = a['group']; m['plane'] = a['plane']
    b = draw.bbox(M['floor']); m['bbox'] = list(b); m['where'] = a['where']; m['src'] = a['src']
    m['entries'] = [list(e) for e in M['entry_names']]
    m['floor_colours'] = colours(M, W)
    m['n_underlays'] = len({W.t[t][1] for t in M['floor'] if W.t[t][1]})
    m['n_overlays'] = len({W.t[t][2] for t in M['floor'] if W.t[t][2]})
    m['overlay_share'] = sum(1 for t in M['floor'] if W.t[t][2]) / len(M['floor'])
    if pictures:
        H, D, c, Z = AM.zones(M)
        C = draw.cams_of(a, W, M['floor']); cam = C['top']
        P = draw.project(cam, draw.ground(W, cam[0], cam[1]))
        try:
            m['pic'] = palette.measure(draw.OUT / f'{a["key"]}-top.png', draw.OUT / f'{a["key"]}-top-empty.png', P, M, Z, M['hfun'])
        except FileNotFoundError as e:
            m['pic'] = None; print('no pictures for', a['key'])
    room = {}
    for tag, fn in (('named', as_room), ('all', as_room_all)):
        r = fn(M, W)
        try:
            rm, rules, grp = roomcheck.check(r, 'any')
            room[tag] = dict(measures={k: v for k, v in rm.items() if k not in ('pair_list', 'pairs', 'opposite')}, rules=[(ok, rid) for ok, rid, msg in rules])
        except Exception as e:
            room[tag] = dict(error=repr(e))
    m['room'] = room
    return m


def main():
    keys = [k for k in sys.argv[1:] if not k.startswith('--')] or [a['key'] for a in ARENAS]
    out = HERE / 'data' / 'arenas.json'; out.parent.mkdir(exist_ok=True)
    data = json.loads(out.read_text()) if out.exists() else {}
    for k in keys:
        m = one(BY[k], '--nopic' not in sys.argv); data[k] = m
        p = m.get('pic') or {}
        print(f"{k:10} area {m['area']:4} {m['w']:2}x{m['h']:2} {m['outline']:14} boss {m.get('boss_place')}, tiles/boss tile {m.get('tiles_per_boss_tile', 0):.1f} plain {m['plain_share']:.2f} edge {m['edge_main']} dE {p.get('deltaE_mean', 0):.0f}", flush=True)
    out.write_text(json.dumps({k: data[k] for k in [a['key'] for a in ARENAS] if k in data}, indent=1, default=str))
    print('->', out)


if __name__ == '__main__':
    main()
