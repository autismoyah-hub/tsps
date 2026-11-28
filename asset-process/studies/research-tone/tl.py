"""research-tone shared library. Everything is drawn by the client's own NPC harness (asset_npc.exe) through
research-creatures/rebuild_lib.py, with that library's temporary storage redirected into THIS folder (work/).

Three kinds of harness picture of the same entry under the same camera, from which every tone experiment is made:
  LIT     the entry as the game draws it (what the gate scores);
  ID      every drawn face painted with a code colour (hue and saturation carry the code; lighting changes only
          lightness, so the code survives), all faces opaque: the front-most face at every pixel;
  LIGHT   every face painted grey lightness 64, opaque: the client's own interpolated light number at every pixel.
With the per-pixel face and light number of BOTH meshes, any counterfactual picture can be synthesised
(real paint under our light, our paint under the real light, ..) with the game's colour table.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, json, subprocess, shutil, os
B = pathlib.Path(__file__).resolve().parent
C = B.parent / 'research-creatures'
N = B.parent / 'research-npc'
sys.path.insert(0, str(C)); sys.path.insert(1, str(N)); sys.path.insert(2, _AP.fix(r'<ROOT>\author-pack\tools'))
import numpy as np
from PIL import Image
import cmesh, rebuild_lib as L, meshio, gamecol
WORK = B / 'work'; (WORK / 'data').mkdir(parents=True, exist_ok=True)
cmesh.DATA = WORK / 'data'; L.HERE = WORK
RENDER = B.parents[1] / 'render'
TABLE = gamecol.table()
GREY = np.array([0.299, 0.587, 0.114])
BG = L.BG
P = [sys.executable, '-I']

BOSSES = {
    'tekton1': dict(npc=7541, ours=B.parent / 'boss-rebuild-7541' / 'boss-rebuild-7541.json'),
    'tekton2': dict(npc=7541, ours=B.parent / 'boss-rebuild-7541-2' / 'boss-rebuild-7541-2.json'),
    'nex': dict(npc=11278, ours=B.parent / 'boss-rebuild-11278' / 'boss-rebuild-11278.json'),
}
MIDS = {
    'lynx': dict(npc=11068, ours=B.parent / 'creature-rebuild-11068' / 'creature-rebuild-11068.json'),
    'araxyte': dict(npc=11175, ours=B.parent / 'creature-rebuild-11175' / 'creature-rebuild-11175.json'),
    'minotaur': dict(npc=12812, ours=B.parent / 'creature-rebuild-12812' / 'creature-rebuild-12812.json'),
}

# ---------------------------------------------------------------- meshes as files
def real_files(npc_id):
    """The real NPC's own model file(s) and definition, dumped by the harness into work/raw-ID (read-only use)."""
    raw = WORK / f'raw-{npc_id}'
    if not (raw / f'npc-{npc_id}.npcdef').exists():
        raw.mkdir(parents=True, exist_ok=True)
        r = subprocess.run([cmesh.EXE, cmesh.CACHE, 'dump', str(npc_id), str(raw)], capture_output=True, text=True)
        if r.returncode: raise SystemExit(r.stdout + r.stderr)
    kv = dict(l.split('=', 1) for l in (raw / f'npc-{npc_id}.npcdef').read_text().splitlines() if '=' in l and not l.startswith('#'))
    files = [raw / f for f in kv['models'].split('|')]
    return files, kv
def real_mesh(npc_id):
    files, kv = real_files(npc_id)
    if len(files) != 1: raise SystemExit('one-model NPCs only')
    return meshio.load(files[0]), kv
def real_def(npc_id):
    files, kv = real_files(npc_id); seqs = L.sequences(npc_id)
    return dict(name=kv.get('name', str(npc_id)), recolor=[[int(x) for x in p.split(':')] for p in kv.get('recolor', '').split(',') if p],
                scale=[int(x) for x in kv['scale'].split(',')], ambient=int(kv['ambient']), contrast=int(kv['contrast']), size=int(kv['size']), **seqs)
def ours_mesh(json_path):
    j = json.loads(pathlib.Path(json_path).read_text())
    if len(j['models']) != 1: raise SystemExit('one-model definitions only')
    return meshio.load(pathlib.Path(json_path).parent / j['models'][0]), j
def write_variant(name, mesh, base_def, sub='variants', **over):
    """Write mesh + definition as work/SUB/NAME.bin / .json; returns the json path."""
    d = WORK / sub; d.mkdir(parents=True, exist_ok=True)
    meshio.save(mesh, d / f'{name}.bin')
    j = {k: v for k, v in base_def.items() if k != 'models'}; j['models'] = [f'{name}.bin']; j.update(over)
    (d / f'{name}.json').write_text(json.dumps(j, indent=1))
    return d / f'{name}.json'
def clone(m): return {k: list(v) for k, v in m.items()}
def arrays(m):
    """meshio dict -> numpy: V float [n,3], F int [m,3], C, A, T, D, L (vertex labels), FS (face groups)."""
    return dict(V=np.array(m['vertices'], float), F=np.array(m['faces'], int), C=np.array(m['colors'], int), A=np.array(m['alpha'], int),
                T=np.array(m['render_types'], int), D=np.array(m['depth'], int), L=np.array(m['vertex_skins'], int), FS=np.array(m['face_skins'], int))
def drawn(a): return (a['A'] != 255) & (a['T'] < 2)
def hsl(c): return (c >> 10) & 63, (c >> 7) & 7, c & 127
def pack(h, s, l): return (int(h) << 10) | (int(s) << 7) | int(l)

# ---------------------------------------------------------------- camera and drawing
def camera(npc_id, cell=(380, 380)):
    lo, hi, mr, sc = L.size_after_scale(npc_id)
    D = float((hi - lo).max()); zoom = int(512 * D * 1.7 / cell[0]); aim = int((hi[1] - lo[1]) / 2)
    return dict(zoom=zoom, aim=aim, cell=cell, pitch=128)
def entry(json_path, seqs):
    return L.entry_ours(pathlib.Path(json_path).resolve(), seqs, WORK / 'data' / 'defs')
def draw(entries, rows, cam, yaw, out_png=None):
    a, text = L.draw(entries, rows, cam['zoom'], yaw, cam.get('pitch', 128), cam['cell'], cam['aim'], out_png)
    return L.cells(a, len(entries), len(rows)), text
def grey(img): return img.astype(float) @ GREY
def mask(img): return (img != BG).any(2)
def tone(A, Bc):
    """The gate's tone: mean absolute grey difference where both are drawn."""
    I = mask(A) & mask(Bc)
    return float(np.abs(grey(A) - grey(Bc))[I].mean()) if I.any() else 0.0

# ---------------------------------------------------------------- ID and LIGHT variants
CODE_S = (3, 5, 7); NCODE = 64 * len(CODE_S)            # 192 codes per picture; two pictures -> 36,864 faces
ID_AMB, ID_CON, ID_L = 50, 127, 64                      # light number about 114 +/- 44: lit lightness 35..79
def code_colour(k): return pack(k % 64, CODE_S[k // 64], ID_L)
_inv = None
def decode_codes(img):
    """RGB picture of an ID render -> code per pixel (-1 = background or unknown)."""
    global _inv
    if _inv is None:
        _inv = {}
        for k in range(NCODE):
            c = code_colour(k) & ~127
            for l in range(2, 127):
                r, g, b = (int(x) for x in TABLE[c | l]); key = (r << 16) | (g << 8) | b
                if key in _inv and _inv[key] != k: _inv[key] = -2          # ambiguous
                else: _inv[key] = k
    flat = (img[..., 0].astype(np.int64) << 16) | (img[..., 1].astype(np.int64) << 8) | img[..., 2]
    u, inv = np.unique(flat, return_inverse=True)
    vals = np.array([_inv.get(int(x), -1) for x in u])
    out = vals[inv].reshape(flat.shape); out[~mask(img)] = -1
    return out
def id_variants(name, mesh, base_def, sub='ids'):
    """Three definitions: ID digit 0, ID digit 1, LIGHT. Faces keep their order; undrawn faces stay undrawn."""
    out = []
    nf = len(mesh['faces'])
    for digit in (0, 1):
        m = clone(mesh)
        m['colors'] = [code_colour((i // (NCODE ** digit)) % NCODE) for i in range(nf)]
        m['alpha'] = [255 if a == 255 else 0 for a in mesh['alpha']]
        m['face_skins'] = [0] * nf                                       # no alpha animation of the code picture
        out.append(write_variant(f'{name}-id{digit}', m, base_def, sub, ambient=ID_AMB, contrast=ID_CON, recolor=[]))
    m = clone(mesh); m['colors'] = [pack(0, 0, 64)] * nf; m['alpha'] = [255 if a == 255 else 0 for a in mesh['alpha']]; m['face_skins'] = [0] * nf
    out.append(write_variant(f'{name}-light', m, base_def, sub, recolor=[]))
    return out
_ginv = None
def decode_light(img):
    """LIGHT render (paint lightness 64, grey) -> the client's light number per pixel (about 2 x drawn lightness)."""
    global _ginv
    if _ginv is None:
        _ginv = {}
        for l in range(127, 1, -1):
            r, g, b = (int(x) for x in TABLE[l]); _ginv[(r << 16) | (g << 8) | b] = l
    flat = (img[..., 0].astype(np.int64) << 16) | (img[..., 1].astype(np.int64) << 8) | img[..., 2]
    u, inv = np.unique(flat, return_inverse=True)
    vals = np.array([_ginv.get(int(x), -1) for x in u])
    out = vals[inv].reshape(flat.shape).astype(float) * 2.0; out[~mask(img)] = np.nan
    return out
def synth(paint, ln):
    """Picture grey from per-pixel paint (16-bit colour, -1 = none) and light number: the client's rule
    lit lightness = clamp((paint lightness x light number) >> 7, 2, 126), then the colour table."""
    ok = (paint >= 0) & np.isfinite(ln)
    p = np.where(ok, paint, 0); l = np.clip((((p & 127) * np.where(ok, ln, 0)).astype(np.int64)) >> 7, 2, 126)
    rgb = TABLE[(p & ~127) | l].astype(float)
    g = rgb @ GREY; g[~ok] = np.nan
    return g

# ---------------------------------------------------------------- the gate, storage redirected
def gate(real_id, ours_json, tag, out_dir=None, per=6):
    """Run research-creatures/gate_npc.py unchanged (this module already redirected its temporary storage).
    Returns (verdict, {line name: value}) and writes work/gates/TAG.txt / .json."""
    import io, contextlib, gate_npc
    g = WORK / 'gates'; g.mkdir(exist_ok=True); buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        code, rows, R = gate_npc.gate(real_id, str(ours_json), per, None, str(out_dir) if out_dir else None)
    (g / f'{tag}.txt').write_text(buf.getvalue())
    (g / f'{tag}.json').write_text(json.dumps(dict(checks=rows, verdict=gate_npc.LEVELS[code]), indent=1, default=str))
    return gate_npc.LEVELS[code], {r['name']: r['value'] for r in rows}, rows
def tone_lines(vals):
    out = {}
    for k, v in vals.items():
        if 'tone difference' in k: out['bind' if k.startswith('standing') else k.split(':')[0]] = v
    return out
def show_tones(tag, verdict, vals):
    t = tone_lines(vals)
    nont = [f"{k}={v}" for k, v in vals.items() if k.startswith(('bounding', 'rest outline', 'colour distance'))]
    print(f"{tag:28s} {verdict:10s} tone " + ' '.join(f'{k} {t[k]:.3f}' for k in ('bind', 'stand', 'walk', 'attack', 'death') if k in t), '|', '; '.join(nont), flush=True)
