"""Shared reading for the rooms study: cache, object definitions, map squares, object kinds.

Read only. Ports of the client's own decoders (rust-client/src/object.rs Definition::decode_mode,
terrain.rs Region::decode and locations, disk_cache.rs reference tables / split_files via animlib).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, re, json, pickle, pathlib
HERE = pathlib.Path(__file__).resolve().parent
TF = HERE.parents[1]
sys.path.insert(0, str(TF / 'tools'))
sys.path.insert(0, str(TF / 'builds' / 'research-animation'))
import animlib as A
import osrscache as O

CACHE = A.Cache()


# ------------------------------------------------------------------------------------------ object definitions
class Rd:
    def __init__(s, d): s.d, s.p = d, 0
    def u8(s): v = s.d[s.p]; s.p += 1; return v
    def u16(s): v = int.from_bytes(s.d[s.p:s.p + 2], 'big'); s.p += 2; return v
    def s16(s): v = int.from_bytes(s.d[s.p:s.p + 2], 'big', signed=True); s.p += 2; return v
    def u24(s): v = int.from_bytes(s.d[s.p:s.p + 3], 'big'); s.p += 3; return v
    def u32(s): v = int.from_bytes(s.d[s.p:s.p + 4], 'big'); s.p += 4; return v
    def string(s):
        e = s.d.index(0, s.p); v = s.d[s.p:e].decode('cp1252', 'replace'); s.p = e + 1; return v
    def unsigned_smart(s):
        return s.u8() if s.d[s.p] < 128 else s.u16() - 32768
    def incremental_smart(s):
        v = 0
        while True:
            c = s.unsigned_smart(); v += c
            if c != 32767: return v
    def left(s): return len(s.d) - s.p


def decode_object(data):
    d = dict(name='null', models=[], shapes=None, size=[1, 1], interact=2, solid=False, interactive=255, ops=[],
             scale=[128, 128, 128], offset=[0, 0, 0], anim=None, transforms=None, decor=16, map_scene=None)
    r = Rd(data)
    while True:
        op = r.u8()
        if op == 0: break
        elif op == 2: d['name'] = r.string()
        elif op in (1, 5):
            n = r.u8(); ms = []; sh = [] if op == 1 else None
            for _ in range(n):
                ms.append(r.u16())
                if sh is not None: sh.append(r.u8())
            if n and not d['models']: d['models'], d['shapes'] = ms, sh
        elif op in (6, 7):
            n = r.u8(); ms = []; sh = [] if op == 6 else None
            for _ in range(n):
                ms.append(r.u32())
                if sh is not None: sh.append(r.u8())
            if n and not d['models']: d['models'], d['shapes'] = ms, sh
        elif op == 14: d['size'][0] = r.u8()
        elif op == 15: d['size'][1] = r.u8()
        elif op == 17: d['interact'] = 0
        elif op == 18: pass
        elif op == 19: d['interactive'] = r.u8()
        elif op in (21, 22, 23, 27, 62, 64, 73, 74, 89, 90):
            if op == 27: d['interact'] = 1
            if op == 74: d['solid'] = True
        elif op == 24:
            v = r.u16(); d['anim'] = None if v == 65535 else v
        elif op == 28: d['decor'] = r.u8()
        elif op in (29, 39): r.u8()
        elif 30 <= op <= 34:
            t = r.string()
            if t.lower() != 'hidden': d['ops'].append(t)
        elif op == 100:
            r.u8(); r.u8(); d['ops'].append(r.string())
        elif op in (101, 102):
            r.u8(); r.u16(); r.u16()
            if op == 102: r.u16()
            r.u32(); r.u32(); d['ops'].append(r.string())
        elif op in (40, 41):
            n = r.u8(); r.p += 4 * n
        elif op == 61: r.u16()
        elif 65 <= op <= 67: d['scale'][op - 65] = r.u16()
        elif op == 68: d['map_scene'] = r.u16()
        elif op == 69: r.u8()
        elif 70 <= op <= 72: d['offset'][op - 70] = r.s16()
        elif op == 75: r.u8()
        elif op in (77, 92):
            r.u16(); r.u16()
            if op == 92: r.u16()
            n = r.u8(); d['transforms'] = [r.u16() for _ in range(n + 1)]
        elif op == 78: r.u16(); r.u8(); r.u8()
        elif op == 79:
            r.u16(); r.u16(); r.u8(); r.u8(); n = r.u8(); r.p += 2 * n
        elif op == 81: r.u8()
        elif op == 82: r.u16()
        elif op == 91: r.u8()
        elif op == 93: r.u8(); r.u16(); r.u8(); r.u16()
        elif op in (95, 96): r.u8()
        elif op == 249:
            for _ in range(r.u8()):
                k = r.u8(); r.u24()
                if k == 1: r.string()
                elif k == 2: r.p += 8
                else: r.u32()
        else:
            pass
    return d


_DEFS = None
def defs():
    """{object id: decoded definition}, cached on disk."""
    global _DEFS
    if _DEFS is None:
        p = HERE / 'cache' / 'defs.pkl'
        if p.exists():
            _DEFS = pickle.loads(p.read_bytes())
        else:
            p.parent.mkdir(exist_ok=True)
            files = CACHE.files(2, 6)
            out = {}
            for i, b in files.items():
                try: out[i] = decode_object(b)
                except Exception as e: out[i] = dict(name='?', models=[], shapes=None, size=[1, 1], interact=2, ops=[], err=str(e))
                out[i]['raw'] = b
            p.write_bytes(pickle.dumps(out)); _DEFS = out
    return _DEFS


def resolved(oid):
    """A definition, following a transform (varbit-switched) object to its first variant that has models."""
    d = defs().get(oid)
    if d and not d['models'] and d.get('transforms'):
        for t in d['transforms']:
            if t != 65535 and t in defs() and defs()[t]['models']:
                e = dict(defs()[t]);
                if e['name'] == 'null' and d['name'] != 'null': e['name'] = d['name']
                return e
    return d


_HEIGHT = {}
def height(oid):
    """Height in units (128 = one tile) of the object's shape-10 models, scaled; cached."""
    if oid in _HEIGHT: return _HEIGHT[oid]
    d = resolved(oid); h = 0
    try:
        ms = d['models'] if not d['shapes'] else [m for m, s in zip(d['models'], d['shapes']) if s in (10, 4, 22)] or d['models'][:1]
        for m in ms[:3]:
            ys = [-v[1] for v in O.model(m)['vertices']]
            if ys: h = max(h, max(ys) * d['scale'][1] / 128)
    except Exception:
        h = -1
    _HEIGHT[oid] = round(h)
    return _HEIGHT[oid]


_SPAN = {}
def span(oid):
    """Longest horizontal extent (units) of the object's first models, scaled: a trim runs the tile's width (128)."""
    if oid in _SPAN: return _SPAN[oid]
    d = resolved(oid); s = 0
    try:
        for m in d['models'][:2]:
            v = O.model(m)['vertices']
            if v:
                s = max(s, (max(p[0] for p in v) - min(p[0] for p in v)) * d['scale'][0] / 128,
                        (max(p[2] for p in v) - min(p[2] for p in v)) * d['scale'][2] / 128)
    except Exception:
        s = -1
    _SPAN[oid] = round(s)
    return _SPAN[oid]


# ------------------------------------------------------------------------------------------ map squares
def squares():
    return sorted(CACHE.reference(5)['groups'])


def square(gid):
    """(terrain region tiles or None, locations list) of one map square; locations as (id, plane, x, y, shape, rot)."""
    files = CACHE.files(5, gid)
    locs = []
    if 1 in files and files[1]:
        r = Rd(files[1]); obj = -1
        while True:
            dl = r.incremental_smart()
            if dl == 0: break
            obj += dl; packed = 0
            while True:
                dp = r.unsigned_smart()
                if dp == 0: break
                packed += dp - 1
                a = r.u8()
                locs.append((obj, packed >> 12, (packed >> 6) & 63, packed & 63, a >> 2, a & 3))
    flags = None; over = None
    if 0 in files and files[0]:
        flags = bytearray(4 * 4096); over = [0] * (4 * 4096)
        r = Rd(files[0])
        for plane in range(4):
            for x in range(64):
                for y in range(64):
                    i = plane * 4096 + x * 64 + y
                    while True:
                        op = r.u16()
                        if op == 0: break
                        if op == 1: r.u8(); break
                        if op <= 49: over[i] = r.u16()
                        elif op <= 81: flags[i] = op - 49
    return flags, over, locs


# ------------------------------------------------------------------------------------------ kinds of objects
WALL_SHAPES = {0, 1, 2, 3, 9}
DECOR_SHAPES = {4, 5, 6, 7, 8}
ROOF_SHAPES = set(range(12, 22))

LIGHT = re.compile(r'torch|candle|lamp|lantern|brazier|chandelier|fire\b|fireplace|sconce|candelabr|light|lit |beacon|firepit|fire pit|lightorb|light orb|glowing', re.I)
FLOORDEC = re.compile(r'rug|carpet|mat\b|floor|rubble|debris|straw|hay|puddle|stain|blood|pentagram|symbol|tile|grate|trapdoor|hatch', re.I)
CLUTTER = re.compile(r'\b(pot|pots|jug|plate|plates|bowl|bottle|bottles|cup|mug|vial|vials|bucket|book|books|scroll|scrolls|candles?|skull|bones|bone|basket|sack|sacks|box|boxes|tools?|hammer|bag|bags|pile|jar|jars|flask|flasks|food|bread|cheese|cake|pie|fish|cabbage|potato|flour|apple|tea|beer|wine|ale|glass|papers?|letter|ink|quill|kettle|pan|pans|cauldron|logs|planks?|ore|bars?|wool|clothes|cloth|shoes|boots|hat|helmet|shield|sword|weapons?|arrows?|bow|spear|axe|pickaxe|cushion|pillow|flowers?|vase|plant|pot plant|ornament|candlestick|tray|teapot|keg)\b', re.I)

FUNC = [  # functional furniture class, by name; first match wins
    ('door', r'door|gate|doorway|curtain'),
    ('stairs', r'stair|ladder|trapdoor'),
    ('bed', r'\bbed\b|bunk|hammock|cot\b|bedroll'),
    ('throne', r'throne'),
    ('altar', r'altar|shrine'),
    ('rack', r'rack|armour stand|weapon stand|suit of armour|armour|dummy|target|display'),
    ('bookcase', r'bookcase|book ?shelf|bookshel|books'),
    ('shelf', r'shel(f|ves)|cupboard|cabinet|wardrobe|dresser|drawers|chest of drawers|sideboard|dresser|closet|crate shelf'),
    ('chest', r'chest|coffer|strongbox|trunk|safe'),
    ('storage', r'crate|barrel|sack|box|keg|cask|basket|bin\b|tub'),
    ('table', r'table|desk|counter|bench top|workbench|lectern|bar\b|stall'),
    ('seat', r'chair|stool|bench|seat|sofa|couch|pew|settee'),
    ('workstation', r'anvil|furnace|forge|range|stove|oven|loom|spinning|wheel|potter|sink|well|cooking|grindstone|lathe|bellows|kiln|press|vice|mill|churn|cauldron|still\b|tannery|smelt'),
    ('statue', r'statue|bust|figure|idol|totem|pillar|column|plinth|pedestal'),
    ('plant', r'plant|tree|bush|flower|fern|pot plant|planter|vine'),
    ('light', LIGHT.pattern),
    ('bank', r'bank|deposit'),
    ('machine', r'telescope|orrery|globe|machine|device|apparatus|contraption|experiment|lab\b|laboratory|alchem|crystal|portal|orb'),
]
FUNC = [(k, re.compile(p, re.I)) for k, p in FUNC]


def func_class(name):
    for k, p in FUNC:
        if p.search(name): return k
    return 'other'


def kind(oid, shape):
    """'wall', 'door', 'window', 'roof', 'decor' (wall decoration), 'light-wall', 'floor' (floor decoration),
    'light', 'clutter', 'furniture', 'stairs'."""
    d = resolved(oid) or {}
    name = d.get('name', 'null')
    ops = ' '.join(d.get('ops', [])).lower()
    if shape in WALL_SHAPES:
        if re.search(r'door|gate|curtain', name, re.I) or 'open' in ops or 'close' in ops: return 'door'
        if re.search(r'window|slit', name, re.I): return 'window'
        return 'wall'
    if shape in ROOF_SHAPES: return 'roof'
    if shape in DECOR_SHAPES:
        if re.search(r'window', name, re.I): return 'window'
        return 'light-wall' if LIGHT.search(name) else 'decor'
    if shape == 22:
        return 'floor'
    # 10, 11
    fc = func_class(name)
    if fc == 'stairs': return 'stairs'
    if fc == 'door': return 'door'
    if LIGHT.search(name): return 'light'
    sx, sz = d.get('size', [1, 1])
    h = height(oid)
    if d.get('interact', 2) == 0 and sx * sz == 1 and fc not in ('bed', 'throne', 'altar', 'table', 'seat', 'rack', 'workstation', 'bookcase', 'shelf', 'chest'):
        return 'clutter'
    if fc == 'other' and CLUTTER.search(name) and sx * sz == 1: return 'clutter'
    if 0 <= h < 40 and sx * sz == 1 and fc in ('other', 'storage', 'plant'): return 'clutter'
    if name == 'null' and d.get('interact', 2) == 0: return 'clutter'
    return 'furniture'


def name(oid):
    d = resolved(oid) or {}
    return d.get('name', 'null')


ARCH = pathlib.Path(_AP.fix(r'<REFERENCE_ARCHIVE>\objects'))
_SYM = {}
def symbol(oid):
    """The rev-239 debug symbol of an object if the reference archive has it (helps name 'null' objects)."""
    if oid in _SYM: return _SYM[oid]
    s = None
    for sh in (10, 22, 4, 0, 9, 3):
        f = ARCH / f'shape-{sh}' / f'{oid}.json'
        if f.exists():
            try: s = json.loads(f.read_text(encoding='utf-8')).get('rev239_symbol')
            except Exception: s = None
            break
    _SYM[oid] = s
    return s
