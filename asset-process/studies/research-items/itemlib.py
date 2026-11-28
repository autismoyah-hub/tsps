"""itemlib.py: shared helpers for the item research tools (studies/research-items).

Item definitions come from data/items.jsonl, which research_items.exe writes with the CLIENT's decoder
(item.rs Definition::decode_local): one JSON object per cache record, fields named as in the Rust struct.
Models come from tools/osrscache.py (a port of the client's model decoder) and are cached in data/models.pkl.
Coordinates are the cache's: y is DOWN (height = -y), the character's right is -x, its front is -z.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, pickle, subprocess
HERE = pathlib.Path(__file__).resolve().parent
TOOLS = pathlib.Path(_AP.fix(r'<ROOT>\tools'))
sys.path.insert(0, str(TOOLS))
sys.path.insert(1, _AP.fix(r'<NATIVE_ASSETS>'))
import osrscache, native_codec
import numpy as np

EXE_DIR = pathlib.Path(_AP.fix(r'<CLIENT_BIN>'))
RESEARCH_ITEMS = str(_AP.exe('research_items.exe', EXE_DIR))
RESEARCH_POSE = str(_AP.exe('research_pose.exe', EXE_DIR))
CACHE = str(osrscache.CACHE)
PY = [_AP.fix(r'<PYTHON>'), '-I']
SLOT_NAMES = {0: 'head', 1: 'cape', 2: 'amulet', 3: 'weapon', 4: 'body', 5: 'shield', 6: 'arms', 7: 'legs', 8: 'hair',
              9: 'hands', 10: 'feet', 11: 'jaw', 12: 'ring', 13: 'ammo'}

_items = None


def items():
    """id -> definition dict (raw records, templates NOT resolved)."""
    global _items
    if _items is None:
        p = HERE / 'data' / 'items.jsonl'
        if not p.exists():
            p.parent.mkdir(exist_ok=True)
            subprocess.run([RESEARCH_ITEMS, CACHE, 'defs', str(p)], check=True)
        _items = {}
        with p.open(encoding='utf-8') as f:
            for line in f:
                d = json.loads(line)
                _items[d['id']] = d
    return _items


def plain(d):
    """Not a note, bought or placeholder template, and has a name."""
    return d['note'][1] == -1 and d['bought'][1] == -1 and d['placeholder'][1] == -1 and d['name'] != 'null'


_models = None
_dirty = 0


def model(mid):
    """Decoded cache model (dict of osrscache.decode_model) with 'V' (n,3 int array) added; None when it cannot be read."""
    global _models, _dirty
    p = HERE / 'data' / 'models.pkl'
    if _models is None:
        _models = pickle.loads(p.read_bytes()) if p.exists() else {}
    if mid not in _models:
        try:
            m = osrscache.model(mid)
            m['V'] = np.array(m['vertices'], dtype=np.int32).reshape(-1, 3)
            m['F'] = np.array(m['faces'], dtype=np.int32).reshape(-1, 3)
        except Exception as e:      # missing or undecodable
            m = None
        _models[mid] = m
        _dirty += 1
    return _models[mid]


def save_models():
    global _dirty
    if _dirty:
        (HERE / 'data' / 'models.pkl').write_bytes(pickle.dumps(_models))
        _dirty = 0


def worn_ids(d, gender):
    """Model ids the client loads for the worn mesh (item_model.rs wearable: the third only when a second exists)."""
    ids = d['wear_models'][gender]
    if ids[0] == -1:
        return []
    return ids[:1] if ids[1] == -1 else [i for i in ids if i != -1]


def head_ids(d, gender):
    ids = d['head_models'][gender]
    if ids[0] == -1:
        return []
    return ids[:1] if ids[1] == -1 else [i for i in ids if i != -1]


def concat(ids):
    """Plain concatenation of several models (no coordinate merge): V, F and per-face channels."""
    ms = [model(i) for i in ids]
    if not ms or any(m is None for m in ms):
        return None
    out = dict(V=[], F=[], colors=[], priorities=[], vertex_skins=[], face_skins=[], render_types=[], alpha=[], depth_bias=[], face_textures=[])
    o = 0
    for m in ms:
        out['V'].append(m['V']); out['F'].append(m['F'] + o); o += len(m['V'])
        for k in ('colors', 'priorities', 'vertex_skins', 'face_skins', 'render_types', 'alpha', 'depth_bias', 'face_textures'):
            v = m.get(k)
            n = len(m['V']) if k == 'vertex_skins' else len(m['F'])
            out[k] += list(v) if v is not None else [0] * n
    out['V'] = np.concatenate(out['V']); out['F'] = np.concatenate(out['F'])
    return out


def load_bin(path):
    m = osrscache.decode_model(pathlib.Path(path).read_bytes())
    m['V'] = np.array(m['vertices'], dtype=np.int32).reshape(-1, 3)
    m['F'] = np.array(m['faces'], dtype=np.int32).reshape(-1, 3)
    return m


def save_bin(path, m, V=None, vertex_skins='keep', face_skins='keep', priorities='keep', colors=None):
    """Write a model through the project's encoder (native_codec.model), carrying every channel our files use.
    V replaces the vertices (cache coordinates, whole units). vertex_skins / face_skins / priorities: 'keep', None (drop)
    or a list. Checks the round trip through the decoder."""
    V = m['V'] if V is None else np.asarray(V)
    nf = len(m['F'])
    enc = dict(vertices=[[int(a) for a in v] for v in V], faces=[tuple(int(a) for a in f) for f in m['F']],
               colors=list(m['colors'] if colors is None else colors))

    def pick(arg, key):
        return m.get(key) if isinstance(arg, str) else arg
    vs, fs, pr = pick(vertex_skins, 'vertex_skins'), pick(face_skins, 'face_skins'), pick(priorities, 'priorities')
    if vs is not None: enc['vertex_skins'] = list(vs)
    if fs is not None: enc['face_skins'] = list(fs)
    if pr is not None: enc['priorities'] = list(pr)
    if m.get('render_types') is not None and any(m['render_types']): enc['render_types'] = list(m['render_types'])
    if m.get('alpha') is not None and any(m['alpha']): enc['alpha'] = list(m['alpha'])
    if m.get('depth_bias') is not None and any(m['depth_bias']): enc['face_flags'] = [(b // 2) & 255 for b in m['depth_bias']]
    if m.get('face_textures') is not None and any(t != -1 for t in m['face_textures']):
        raise SystemExit('textured model: the encoder does not carry textures')
    data = native_codec.model(enc)
    back = osrscache.decode_model(data)
    assert [tuple(v) for v in back['vertices']] == [tuple(v) for v in enc['vertices']], 'vertices changed in the round trip'
    assert [tuple(f) for f in back['faces']] == enc['faces'] and list(back['colors']) == enc['colors']
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    pathlib.Path(path).write_bytes(data)
    return data


def hsl(c):
    return (c >> 10) & 63, (c >> 7) & 7, c & 127


def bmp_to_png(bmp, png=None, keep=False):
    from PIL import Image
    im = Image.open(bmp).convert('RGB')
    if png:
        im.save(png)
    if not keep:
        pathlib.Path(bmp).unlink()
    return im
