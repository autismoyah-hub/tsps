"""Shared parts of the creature rebuild test: definitions, sequences, the motion grid, picture metrics.

Everything is drawn by examples/asset_npc.rs (the client's own combine, recolour, light-once-at-rest, animate, scale and
depth-tested painting; see studies/research-npc/NPCS.md and LIGHTING.md here).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, subprocess, json, hashlib, tempfile
import numpy as np
from PIL import Image
HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE)); sys.path.insert(1, _AP.fix(r'<ROOT>\studies\research-npc'))
import cmesh
CENSUS = pathlib.Path(_AP.fix(r'<ROOT>\studies\research-npc\census'))
BG = np.array([0x30, 0x2b, 0x25], np.uint8)
_c = {}
def census(name):
    if name not in _c:
        _c[name] = {j['id']: j for j in (json.loads(l) for l in (CENSUS / f'{name}.jsonl').read_text().splitlines() if l.strip())}
    return _c[name]
def npc_def(i): return census('npcs')[int(i)]
def seq_frames(s):
    q = census('sequences').get(int(s)); return (q['frames'], q['length'], q['skeleton'], q['kind']) if q else (0, 0, -1, 'none')
# attack / death the Teacher has fixed for creatures the server tables do not cover (stated as INFERRED in the brief)
KNOWN_SEQS = {11068: dict(attack=9092, death=9093), 12976: dict(attack=12494, death=12499), 13796: dict(attack=4672, death=4673), 11175: dict(attack=9140, death=9141)}
def sequences(npc_id, overrides=None):
    """stand, walk from the definition; attack, death from the server tables, else KNOWN_SEQS; overrides win."""
    import npc
    d = npc_def(npc_id); out = {}
    if d['anims'][0] >= 0: out['stand'] = d['anims'][0]
    if d['anims'][3] >= 0: out['walk'] = d['anims'][3]
    try:
        a, de, where, _ = npc.server_anims(int(npc_id))
        if a: out['attack'] = a
        if de: out['death'] = de
    except Exception: pass
    out.update(KNOWN_SEQS.get(int(npc_id), {})); out.update(overrides or {})
    return out
def sample_rows(seqs, per=6):
    rows = ['bind']
    for name in ('stand', 'walk', 'attack', 'death'):
        if name not in seqs: continue
        n = seq_frames(seqs[name])[0]
        if n <= 0: continue
        for f in sorted({int(round(k * (n - 1) / max(per - 1, 1))) for k in range(min(per, n))}): rows.append(f'{name}:{f}')
    return rows
def entry_real(npc_id, seqs): return f'npc:{npc_id}' + ''.join(f';{k}={v}' for k, v in seqs.items())
def entry_ours(json_path, seqs, work):
    return cmesh.entry_to_exe('def:' + str(json_path), pathlib.Path(work)) + ''.join(f';{k}={v}' for k, v in seqs.items())
def size_after_scale(entry_json_or_id):
    if str(entry_json_or_id).isdigit():
        d = npc_def(entry_json_or_id); m = cmesh.load(f'npc:{entry_json_or_id}'); sc = d['scale']
    else:
        j = json.loads(pathlib.Path(entry_json_or_id).read_text()); m = cmesh.load('def:' + str(entry_json_or_id)); sc = j.get('scale', [128, 128])
    used = np.unique(m['F']); V = m['V'][used] * np.array([sc[0], sc[1], sc[0]]) / 128
    return V.min(0), V.max(0), m, sc
def draw(entries, rows, zoom, yaw, pitch, cell, aim, out_png=None):
    """One harness call: columns = entries, rows = rows. Returns the picture as an array and the harness text."""
    tmp = pathlib.Path(tempfile.mkdtemp(prefix='cr-', dir=str(HERE / 'data'))) if not out_png else None
    bmp = (tmp / 'g.bmp') if tmp else pathlib.Path(out_png).with_suffix('.bmp')
    bmp.parent.mkdir(parents=True, exist_ok=True)
    cmd = [cmesh.EXE, cmesh.CACHE, 'draw', str(bmp), 'rows=' + ','.join(rows), f'zoom={zoom}', f'yaw={yaw}', f'pitch={pitch}', f'cell={cell[0]}x{cell[1]}', f'aim={aim}'] + list(entries)
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    im = Image.open(bmp).convert('RGB'); a = np.asarray(im).copy()
    if out_png: im.save(out_png)
    bmp.unlink()
    if tmp: tmp.rmdir()
    return a, r.stdout
def cells(a, ncols, nrows):
    ch, cw = a.shape[0] // nrows, a.shape[1] // ncols
    return [[a[r * ch:(r + 1) * ch, c * cw:(c + 1) * cw] for c in range(ncols)] for r in range(nrows)]
GREY = np.array([0.299, 0.587, 0.114])
def cell_metrics(A, B):
    """Real cell A against ours B. overlap = outline overlap (IoU of drawn pixels); differ = share of the union whose
    pixels differ at all; tone = mean absolute grey difference (0-255) where BOTH are drawn; all = the same over the
    union (a pixel drawn by one only is compared with the background)."""
    ma, mb = (A != BG).any(2), (B != BG).any(2)
    U, I = ma | mb, ma & mb
    if not U.any(): return dict(overlap=1.0, differ=0.0, tone=0.0, all=0.0, pixels=0)
    d = np.abs(A.astype(float) @ GREY - B.astype(float) @ GREY)
    return dict(overlap=float(I.sum() / U.sum()), differ=float(((A != B).any(2) & U).sum() / U.sum()),
                tone=float(d[I].mean()) if I.any() else 0.0, all=float(d[U].mean()), pixels=int(ma.sum()))
def colour_shares(m):
    area, unit, cen = cmesh.face_geometry(m); keep = (m['A'] != 255) & (m['T'] < 2)
    by = {}
    for c, a in zip(m['C'][keep], area[keep]): by[cmesh.hsl(int(c))] = by.get(cmesh.hsl(int(c)), 0) + a
    t = sum(by.values()) or 1
    return {k: v / t for k, v in by.items()}
def colour_distance(a, b):
    """The furniture gate's measure (tools/compare.py): each unit of our surface is matched to the nearest real colour
    still unfilled; distance = hue steps (weighted down when nearly grey) + saturation steps + lightness steps / 4."""
    def d(x, y):
        dh = min(abs(x[0] - y[0]), 64 - abs(x[0] - y[0])); w = min(x[1], y[1]) / 7 + 0.15
        return dh * w + abs(x[1] - y[1]) + abs(x[2] - y[2]) / 4
    need = dict(b); cost = 0.0
    for k, share in sorted(a.items(), key=lambda kv: -kv[1]):
        left = share
        for t in sorted(need, key=lambda t: d(k, t)):
            if left <= 1e-9: break
            take = min(left, need[t])
            if take <= 0: continue
            cost += take * d(k, t); need[t] -= take; left -= take
    return cost
def rest_outlines(mr, scr, mo, sco, px=2.0):
    """Outline overlap of the two rest meshes in BODY space (not re-centred: a creature is authored on its skeleton)."""
    def scaled(m, sc): return m['V'] * np.array([sc[0], sc[1], sc[0]]) / 128
    Vr, Vo = scaled(mr, scr), scaled(mo, sco)
    Fr = mr['F'][(mr['A'] != 255) & (mr['T'] < 2)]; Fo = mo['F'][(mo['A'] != 255) & (mo['T'] < 2)]
    allv = np.concatenate([Vr[np.unique(Fr)], Vo[np.unique(Fo)]]); lo, hi = allv.min(0), allv.max(0)
    import cv2
    out = {}
    for name, (ax, ay) in dict(side=(2, 1), front=(0, 1), top=(0, 2)).items():
        w, h = int((hi[ax] - lo[ax]) * px) + 9, int((hi[ay] - lo[ay]) * px) + 9; ims = []
        for V, F in ((Vr, Fr), (Vo, Fo)):
            im = np.zeros((h, w), np.uint8); pts = np.stack([(V[:, ax] - lo[ax]) * px + 4, (V[:, ay] - lo[ay]) * px + 4], 1)
            for t in np.round(pts[F] * 16).astype(np.int32): cv2.fillConvexPoly(im, t, 255, lineType=cv2.LINE_8, shift=4)
            ims.append(im > 0)
        out[name] = float((ims[0] & ims[1]).sum() / max((ims[0] | ims[1]).sum(), 1))
    return out
