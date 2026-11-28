"""lib.py: shared loaders and helpers for the research-fit study (fit and definition of worn gear).
Everything here is read-only against the cache and the project's tools. Private: cache-derived.

Worn space: x (the character's right is negative), height up (feet 0, collar 164, crown 196), z (front negative).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, subprocess, math
import numpy as np

sys.dont_write_bytecode = True
ROOT = pathlib.Path(_AP.fix('<ROOT>'))
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'tools'))
sys.path.insert(0, str(ROOT / 'author-pack' / 'tools'))
import osrscache as o          # noqa: E402
import gamecol                 # noqa: E402

PY = _AP.fix('<PYTHON>')
POSE = str(ROOT / 'tools' / 'pose.py')
BB = str(ROOT / 'builds')
KITS = json.load(open(ROOT / 'author-pack/labels/body-kits.json'))['slots']
_IDX = None


def idx():
    global _IDX
    if _IDX is None:
        _IDX = {e['id']: e for e in json.load(open(ROOT / 'builds/meridian-helm-v3/study/index.json'))}
    return _IDX


def kit(name):
    """body kit -> (V [n,3] float, F list of triples, colours hsl per face)"""
    k = KITS[name]
    V = np.array(k['vertices'], float)
    F = [tuple(f) for f in k['faces'] if len(set(f)) == 3]
    return V, F


def tris(V, F):
    V = np.asarray(V, float)
    return np.array([[V[a], V[b], V[c]] for a, b, c in F if len({a, b, c}) == 3], float).reshape(-1, 3, 3)


def mesh_real(item_id):
    """a real item's male worn mesh -> dict(V, F, col (h,s,l) per face, prio, flat)"""
    e = idx()[item_id]
    m = o.merge([o.model(k) for k in e['wm'][0] if k not in (-1, None)])
    return _pack(m, e.get('rec'))


def mesh_ours(path):
    m = o.decode_model(pathlib.Path(path).read_bytes())
    return _pack(m, None)


def _pack(m, rec):
    cols = list(m['colors'])
    if rec:
        try:
            mp = {int(a): int(b) for a, b in rec}
            cols = [mp.get(c, c) for c in cols]
        except Exception:
            pass
    return dict(V=np.array([(p[0], -p[1], p[2]) for p in m['vertices']], float), F=[tuple(f) for f in m['faces']],
                col=[o.hsl_split(c) for c in cols], raw=cols, prio=list(m['priorities']),
                flat=[int(r == 1) for r in m['render_types']], alpha=list(m.get('alpha', [])))


def wear_positions(item_id):
    return idx()[item_id]['wp']


def item_name(item_id):
    return idx()[item_id]['name']


# ---------------------------------------------------------------------------------------------- the study's items
REFS = [      # key, helm id, name, body id, legs id, note
    ('torva',      26382, 'Torva full helm',      26384, 26386, 'own set'),
    ('justiciar',  22326, 'Justiciar faceguard',  22327, 22328, 'own set'),
    ('serpentine', 12931, 'Serpentine helm',      11832, 11834, 'with Bandos'),
    ('faceguard',  24271, 'Neitiznot faceguard',  11832, 11834, 'with Bandos'),
    ('oathplate',  30750, 'Oathplate helm',       30753, 30756, 'own set'),
]
COVETED = [
    ('armadyl',    11826, 'Armadyl helmet',          11828, 11830, 'own set'),
    ('neitiznot',  10828, 'Helm of neitiznot',       10551, 4087,  'Fighter torso, Dragon platelegs'),
    ('inquisitor', 24419, "Inquisitor's great helm", 24420, 24421, 'own set'),
    ('masori',     27235, 'Masori mask (f)',         27238, 27241, 'own set'),
    ('virtus',     26241, 'Virtus mask',             26243, 26245, 'own set'),
    ('dragon',     11335, 'Dragon full helm',        21892, 4087,  'own set'),
    ('thirdage',   10350, '3rd age full helmet',     10348, 10346, 'own set'),
    ('dharok',     4716,  "Dharok's helm",           4720,  4722,  'own set'),
    ('verac',      4753,  "Verac's helm",            4757,  4759,  'own set'),
    ('guthan',     4724,  "Guthan's helm",           4728,  4730,  'own set'),
]
PLAIN = [
    ('steelfull',  1157, 'Steel full helm',   1119, 1069, 'own metal'),
    ('runefull',   1163, 'Rune full helm',    1127, 1079, 'own metal'),
    ('ironmed',    1137, 'Iron med helm',     1115, 1067, 'own metal'),
    ('runemed',    1147, 'Rune med helm',     1127, 1079, 'own metal'),
    ('initiate',   5574, 'Initiate sallet',   1127, 1079, 'on rune'),
    ('proselyte',  9672, 'Proselyte sallet',  9674, 9676, 'own set'),
    ('white',      6623, 'White full helm',   1119, 1069, 'on steel'),
    ('granite',    10589, 'Granite helm',     1119, 1069, 'on steel'),
    ('warrior',    3753, 'Warrior helm',      1127, 1079, 'on rune'),
    ('torag',      4745, "Torag's helm",      4749, 4751, 'own set'),
]
CUIRASS_V2 = f'{BB}/meridian-cuirass-v2/meridian-cuirass-v2-labelled.bin'
CUIRASS_V3 = f'{BB}/meridian-cuirass-v3/meridian-cuirass-v3-labelled.bin'
OURS = [      # key, path, slotspec, name, body path, round, the owner's words
    ('v1',  f'{BB}/meridian-helm/meridian-helm-labelled.bin',         'head+hair',     'open sallet with hoop', CUIRASS_V2, 1),
    ('v2a', f'{BB}/meridian-helm-v2/meridian-helm-v2-a-labelled.bin', 'head+hair+jaw', 'closed helm',           CUIRASS_V2, 2),
    ('v2b', f'{BB}/meridian-helm-v2/meridian-helm-v2-b-labelled.bin', 'head+hair',     'barbute',               CUIRASS_V2, 2),
    ('v2c', f'{BB}/meridian-helm-v2/meridian-helm-v2-c-labelled.bin', 'head+hair',     'crested sallet',        CUIRASS_V2, 2),
    ('v3a', f'{BB}/meridian-helm-v3/meridian-helm-v3-a-labelled.bin', 'head+hair+jaw', 'crescent crest',        CUIRASS_V3, 3),
    ('v3b', f'{BB}/meridian-helm-v3/meridian-helm-v3-b-labelled.bin', 'head+hair+jaw', 'notched wings',         CUIRASS_V3, 3),
    ('v3c', f'{BB}/meridian-helm-v3/meridian-helm-v3-c-labelled.bin', 'head+hair+jaw', 'star visor',            CUIRASS_V3, 3),
]
SET_V2 = (f'model:{CUIRASS_V2}@body+arms+model:{BB}/meridian-gauntlets/meridian-gauntlets-labelled.bin@hands'
          f'+model:{BB}/meridian-legs/meridian-legs-labelled.bin@legs+model:{BB}/meridian-boots/meridian-boots-labelled.bin@feet')
SET_V3 = (f'model:{CUIRASS_V3}@body+arms+model:{BB}/meridian-gauntlets-v3/meridian-gauntlets-v3-labelled.bin@hands'
          f'+model:{BB}/meridian-legs-v3/meridian-legs-v3-labelled.bin@legs+model:{BB}/meridian-boots-v3/meridian-boots-v3-labelled.bin@feet')
HELM_V2A = f'model:{BB}/meridian-helm-v2/meridian-helm-v2-a-labelled.bin@head+hair+jaw'
HELM_V3A = f'model:{BB}/meridian-helm-v3/meridian-helm-v3-a-labelled.bin@head+hair+jaw'


def ref_outfit(r, helm=True):
    return '+'.join(f'item:{i}' for i in ([r[1]] if helm else []) + [r[3], r[4]])


# ---------------------------------------------------------------------------------------------- pictures
def pose(png, entries, *extra):
    """run tools/pose.py; returns the picture as an int array [h, w, 3] with the label strip rows still in place"""
    from PIL import Image
    pathlib.Path(png).parent.mkdir(parents=True, exist_ok=True)
    r = subprocess.run([PY, '-I', POSE, str(png), ','.join(entries), *[str(x) for x in extra], '--keep-bmp'], capture_output=True, text=True)
    if r.returncode:
        raise SystemExit(r.stdout + r.stderr)
    bmp = pathlib.Path(png).with_suffix('.bmp')
    a = np.asarray(Image.open(bmp).convert('RGB')).astype(np.int32)      # the picture BEFORE labels are drawn
    bmp.unlink()
    return a


_rgb2c = None


def hsl_of(px):
    """pixels [n,3] -> hue, sat, light, known (exact lookup in the client's colour table)"""
    global _rgb2c
    if _rgb2c is None:
        t = gamecol.table()
        key = (t[:, 0].astype(np.int64) << 16) | (t[:, 1].astype(np.int64) << 8) | t[:, 2].astype(np.int64)
        _rgb2c = {}
        for c in range(65535, -1, -1):
            _rgb2c[int(key[c])] = c
    k = (px[:, 0].astype(np.int64) << 16) | (px[:, 1].astype(np.int64) << 8) | px[:, 2].astype(np.int64)
    u, inv = np.unique(k, return_inverse=True)
    cu = np.array([_rgb2c.get(int(v), -1) for v in u], dtype=np.int64)
    c = cu[inv]
    return (c >> 10) & 63, (c >> 7) & 7, c & 127, c >= 0
