"""Shared parts of the skeletal-creature tools (research-bosses): cache reads, the harness, transcoding.

The harness is examples/asset_bone.rs (grid: one cell per frame of a keyframe animation from the cache or from loose
files; merged: the rest mesh with bone weights as text). Read-only on the cache.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, subprocess, json, tempfile, re
import numpy as np
from PIL import Image
HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE)); sys.path.insert(1, _AP.fix(r'<ROOT>\studies\research-animation')); sys.path.insert(2, _AP.fix(r'<ROOT>\studies\research-npc'))
import animlib as A, bonecodec as B
EXE = _AP.fix(r'<CLIENT_BIN>\asset_bone.exe')
NPC_EXE = _AP.fix(r'<CLIENT_BIN>\asset_npc.exe')
CACHE = _AP.fix(r'<OSRS_CACHE_239>')
CENSUS = pathlib.Path(_AP.fix(r'<ROOT>\studies\research-npc\census'))
BG = np.array([0x30, 0x2b, 0x25], np.uint8)
_cache = None
def cache():
    global _cache
    if _cache is None: _cache = A.Cache()
    return _cache
_cs = {}
def census(name):
    if name not in _cs:
        _cs[name] = {j['id']: j for j in (json.loads(l) for l in (CENSUS / f'{name}.jsonl').read_text().splitlines() if l.strip())}
    return _cs[name]
def npc_def(i): return census('npcs')[int(i)]
def seq_def(sid): return A.decode_sequence(A.sequences()[int(sid)])
def anim_id(sid):
    s = seq_def(sid); return s['cached_animation'], (s['cached_range'] or (0, 0))
def anim_bytes(aid): return cache().file(22, aid >> 16, aid & 65535)
def base_bytes(bid): return cache().file(1, bid, 0)
def model_bytes(mid): return cache().file(7, mid, 0)
def base_of_seq(sid):
    aid, _ = anim_id(sid)
    if aid is None: return None
    return int.from_bytes(anim_bytes(aid)[1:3], 'big')
def seqs_on_base(bid):
    """[(sequence id, frames)] of the keyframe sequences on a base, from the census."""
    return sorted((s['id'], s['frames']) for s in census('sequences').values() if s['skeleton'] == bid and s['kind'] == 'cached')

def run(cmd):
    r = subprocess.run([str(c) for c in cmd], capture_output=True, text=True)
    if r.returncode: raise SystemExit('FAILED: ' + ' '.join(str(c) for c in cmd) + '\n' + r.stdout[-2000:] + r.stderr[-2000:])
    return r.stdout
def grid(entry, out_png=None, keep=False, **o):
    """One asset_bone grid call. Returns (picture array, {cell: vertex hash}, stdout). Options as the exe's (seq, base,
    anim, count, frames, cols, zoom, yaw, pitch, cell, aim, draw)."""
    tmpd = None
    if out_png: bmp = pathlib.Path(out_png).with_suffix('.bmp'); bmp.parent.mkdir(parents=True, exist_ok=True)
    else: tmpd = pathlib.Path(tempfile.mkdtemp(prefix='bone-')); bmp = tmpd / 'g.bmp'
    cmd = [EXE, CACHE, 'grid', bmp] + [f'{k}={v}' for k, v in o.items() if v is not None] + [entry]
    text = run(cmd)
    im = Image.open(bmp).convert('RGB'); a = np.asarray(im).copy()
    if out_png: im.save(out_png)
    bmp.unlink()
    if tmpd: tmpd.rmdir()
    hashes = {int(m.group(1)): m.group(2) for m in re.finditer(r'^cell (\d+): .*?vertices ([0-9a-f]{16});', text, re.M)}
    return a, hashes, text
def merged(entry, out_txt):
    run([EXE, CACHE, 'merged', entry, out_txt]); return load_merged(out_txt)
def load_merged(path):
    """The rest mesh with weights: V (n,3), L labels, W [[(bone, weight)..]], F, C colours, P, T flat, D depth, A alpha, FL."""
    V, L, W, F, C, P, T, D, Al, FL = [], [], [], [], [], [], [], [], [], []
    for line in pathlib.Path(path).read_text().splitlines():
        p = line.split()
        if not p or p[0] == '#': continue
        if p[0] == 'v':
            V.append((int(p[1]), int(p[2]), int(p[3]))); L.append(int(p[4])); n = int(p[5])
            W.append([(int(p[6 + 2 * k]), int(p[7 + 2 * k])) for k in range(n)])
        elif p[0] == 'f':
            F.append((int(p[1]), int(p[2]), int(p[3]))); C.append(int(p[4])); P.append(int(p[5])); T.append(int(p[6])); D.append(int(p[7])); Al.append(int(p[8])); FL.append(int(p[9]))
    return dict(V=np.array(V, float).reshape(-1, 3), L=np.array(L, int), W=W, F=np.array(F, int).reshape(-1, 3), C=np.array(C, int), P=np.array(P, int),
                T=np.array(T, int), D=np.array(D, int), A=np.array(Al, int), FL=np.array(FL, int))
def dump_npc(npc_id, outdir):
    """The real NPC's own model files and its definition text (asset_npc dump). Returns ({key: value}, [model paths])."""
    outdir = pathlib.Path(outdir); outdir.mkdir(parents=True, exist_ok=True)
    run([NPC_EXE, CACHE, 'dump', npc_id, outdir])
    kv = dict(l.split('=', 1) for l in (outdir / f'npc-{npc_id}.npcdef').read_text().splitlines() if '=' in l and not l.startswith('#'))
    return kv, [outdir / f for f in kv['models'].split('|')]
def write_npcdef(path, name, model_files, kv, extra=None):
    lines = [f'# {name}', f'name={name}', 'models=' + '|'.join(str(pathlib.Path(m).resolve()) for m in model_files)]
    for k in ('recolor', 'retexture', 'scale', 'ambient', 'contrast', 'size', 'zbuffer'):
        if kv.get(k) not in (None, ''): lines.append(f'{k}={kv[k]}')
    for k, v in (extra or {}).items(): lines.append(f'{k}={v}')
    pathlib.Path(path).write_text('\n'.join(lines) + '\n'); return path
def diff_cells(a, b, cw, ch):
    """Per-cell differing pixel counts between two grids of the same layout, and the drawn pixel counts of a."""
    assert a.shape == b.shape, (a.shape, b.shape)
    d = (a != b).any(2); drawn = (a != BG).any(2); out = []
    for r in range(a.shape[0] // ch):
        for c in range(a.shape[1] // cw):
            out.append((int(d[r * ch:(r + 1) * ch, c * cw:(c + 1) * cw].sum()), int(drawn[r * ch:(r + 1) * ch, c * cw:(c + 1) * cw].sum())))
    return out

# ---------------------------------------------------------------------------------------------- transcoding
def renumber(models, base, anims, new_base_id, seed=239):
    """The same creature with everything that is only a NUMBER changed: the base gets a new id, the bones a new order
    (a seeded shuffle), every model its vertices in reverse order, every keyframe file its channels in reverse order.
    Weights and channel targets follow the bones. Nothing that the client computes with is changed, so the picture
    must not change. Returns (models, base, anims) as new dicts."""
    import random, copy
    rng = random.Random(seed); n = len(base['bones']); perm = list(range(n)); rng.shuffle(perm)      # old index -> new index
    nb = copy.deepcopy(base); nb['bones'] = [None] * n
    for old, bone in enumerate(base['bones']):
        nb['bones'][perm[old]] = dict(parent=perm[bone['parent']] if bone['parent'] >= 0 else bone['parent'], matrices=bone['matrices'], extra=bone['extra'])
    out_models = []
    for m in models:
        m2 = copy.deepcopy(m); nv = len(m['vertices']); rv = lambda i: nv - 1 - i
        m2['vertices'] = m['vertices'][::-1]
        if m['vertex_skins'] is not None: m2['vertex_skins'] = m['vertex_skins'][::-1]
        if m['bone_weights'] is not None: m2['bone_weights'] = [[(perm[b], w) for b, w in ws] for ws in m['bone_weights'][::-1]]
        m2['faces'] = [(rv(a), rv(b), rv(c)) for a, b, c in m['faces']]
        m2['texture_triangles'] = [tuple(rv(v) for v in t) for t in m['texture_triangles']]
        for k in ('_vflags', '_ops'): m2.pop(k, None)
        out_models.append(m2)
    out_anims = []
    for a in anims:
        a2 = copy.deepcopy(a); a2['base'] = new_base_id
        for c in a2['channels']:
            if c['kind'] == 1: c['target'] = perm[c['target']]
        a2['channels'] = a2['channels'][::-1]
        out_anims.append(a2)
    return out_models, nb, out_anims
