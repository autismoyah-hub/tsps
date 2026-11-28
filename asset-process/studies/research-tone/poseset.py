"""The POSE SET of the boss sheet: the real boss drawn by the client's harness from eight directions in the rest pose and
in frames of its four sequences, as three registered pictures per cell:
    ROW-yY-pP-lit.png     the harness picture itself (what the game shows; an author may already draw this with npc.py)
    ROW-yY-pP-paint.png   the PAINT of the front-most opaque face at each pixel (exact colour-table RGB; alpha 0 = none)
    ROW-yY-pP-tpaint.png  where a translucent face is in front: its paint, PNG alpha = its opacity
    ROW-yY-pP-light.png   the client's light number of the opaque layer (grey level = light number / 2 x 2)
Why poses: a boss's rest pose is not what the player sees. Tekton's rest mesh holds the intact legs AND the fracture
plates of its death in the same place; a wing folds over surfaces the rest pose hides. Paint can only be placed on those
from pictures of the poses that show them.

Teacher-side:  poseset.py build NPC_ID SHEET_DIR        (reads the real mesh to make the paint pictures)
Author-side:   cells(sheet_dir, def_json) yields, for every cell, the same three layers of the AUTHOR's mesh (drawn by
               the same harness under the same camera) beside the sheet's pictures. No real mesh is read.
Frames are chosen between the gate's own sample frames, and the eight yaws (64 + 256k) avoid the gate's two (192, 1216).
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json
import numpy as np
from PIL import Image
import tl
from tl import L
CELL = (800, 800)
YAWS = [64 + 256 * k for k in range(8)]
def layer_variants(name, mesh, base_def, sub):
    """Definitions for: idA (every face opaque) digits 0/1, idO (translucent faces undrawn) digits 0/1, light of idO."""
    nf = len(mesh['faces']); out = []
    for tag, alpha in (('A', [255 if a == 255 else 0 for a in mesh['alpha']]), ('O', [255 if a else 0 for a in mesh['alpha']])):
        for digit in (0, 1):
            m = tl.clone(mesh); m['colors'] = [tl.code_colour((i // (tl.NCODE ** digit)) % tl.NCODE) for i in range(nf)]; m['alpha'] = alpha; m['face_skins'] = [0] * nf
            out.append(tl.write_variant(f'{name}-id{tag}{digit}', m, base_def, sub, ambient=tl.ID_AMB, contrast=tl.ID_CON, recolor=[]))
    m = tl.clone(mesh); m['colors'] = [tl.pack(0, 0, 64)] * nf; m['alpha'] = [255 if a else 0 for a in mesh['alpha']]; m['face_skins'] = [0] * nf
    out.append(tl.write_variant(f'{name}-lightO', m, base_def, sub, recolor=[]))
    return out
def decode(cs, nf):
    """cells [idA0, idA1, idO0, idO1, lightO] -> fidA, fidO, ln"""
    def fid(i0, i1):
        c0, c1 = tl.decode_codes(i0), tl.decode_codes(i1); f = np.where((c0 >= 0) & (c1 >= 0), c0 + tl.NCODE * c1, -1); f[f >= nf] = -1; return f
    return fid(cs[0], cs[1]), fid(cs[2], cs[3]), tl.decode_light(cs[4])
def pick_rows(seqs, per_gate=6):
    gate = set(L.sample_rows(seqs, per_gate)); rows = ['bind']
    for name, m in (('stand', 2), ('walk', 2), ('attack', 2), ('death', 4)):
        if name not in seqs: continue
        n = L.seq_frames(seqs[name])[0]
        for i in range(m):
            f = int(round((i + 0.5) * (n - 1) / m))
            while f'{name}:{f}' in gate and f < n - 1: f += 1
            if f'{name}:{f}' not in rows: rows.append(f'{name}:{f}')
    return rows
def cameras(rows):
    """(yaw, pitch, rows drawn there)"""
    cams = [(y, 128, rows) for y in YAWS]
    cams += [(y, 400, ['bind']) for y in YAWS[::2]] + [(y, 1920, ['bind']) for y in YAWS[1::2]]
    return cams
def draw_layers(entries, rows, cam, yaw, pitch):
    out = []
    for s0 in range(0, len(rows), 4):
        cells, text = tl.draw(entries, rows[s0:s0 + 4], dict(cam, pitch=pitch), yaw); out.extend(cells)
    return out

def build(npc, sheet_dir, gate_cells=False):
    """gate_cells=True writes the GATE's own cells instead (its two yaws, its sample frames, its 380 px cell): used only
    for the Teacher-side bound 'the best any repaint of this mesh could score', never given to an author."""
    sheet_dir = pathlib.Path(sheet_dir); out = sheet_dir / 'pose'; out.mkdir(parents=True, exist_ok=True)
    m, kv = tl.real_mesh(npc); d = tl.real_def(npc); a = tl.arrays(m); nf = len(a['F']); seqs = L.sequences(npc); cam = tl.camera(npc, (380, 380) if gate_cells else CELL); rows = L.sample_rows(seqs, 6) if gate_cells else pick_rows(seqs)
    cams = [(192, 128, rows), (1216, 128, rows)] if gate_cells else cameras(rows)
    var = layer_variants(f'real-{npc}', m, d, 'ids'); ents = [L.entry_real(npc, seqs)] + [tl.entry(p, seqs) for p in var]; T = tl.TABLE; n = 0
    for yaw, pitch, rws in cams:
        for row, cs in zip(rws, draw_layers(ents, rws, cam, yaw, pitch)):
            lit = cs[0]; fA, fO, ln = decode(cs[1:], nf); stem = f"{row.replace(':', '')}-y{yaw}-p{pitch}"
            Image.fromarray(lit).save(out / f'{stem}-lit.png')
            im = np.zeros(fO.shape + (4,), np.uint8); ok = fO >= 0; im[..., :3] = np.where(ok[..., None], T[a['C'][np.clip(fO, 0, None)]], 0); im[..., 3] = np.where(ok, 255, 0); Image.fromarray(im, 'RGBA').save(out / f'{stem}-paint.png')
            tr = (fA >= 0) & (a['A'][np.clip(fA, 0, None)] > 0); im = np.zeros(fO.shape + (4,), np.uint8); im[..., :3] = np.where(tr[..., None], T[a['C'][np.clip(fA, 0, None)]], 0); im[..., 3] = np.where(tr, 255 - a['A'][np.clip(fA, 0, None)], 0); Image.fromarray(im, 'RGBA').save(out / f'{stem}-tpaint.png')
            l8 = np.zeros(fO.shape + (2,), np.uint8); l8[..., 0] = np.clip(np.nan_to_num(ln), 0, 255); l8[..., 1] = np.where(np.isfinite(ln), 255, 0); Image.fromarray(l8, 'LA').save(out / f'{stem}-light.png'); n += 1
    info = dict(cell=list(cam['cell']), zoom=cam['zoom'], aim=cam['aim'], rows=rows, sequences=seqs, cameras=[[y, p, r] for y, p, r in cams],
                note='harness camera: asset_npc draw zoom=Z yaw=Y pitch=P cell=WxH aim=A; rows are NAME:FRAME of the sequences listed')
    (sheet_dir / 'pose.json').write_text(json.dumps(info, indent=1)); print(sheet_dir / 'pose.json', n, 'cells', rows)

def cells(sheet_dir, def_json, sub='posefit-ids', rows_filter=None):
    """Author-side iterator. Yields dict(row, yaw, pitch, fA, fO, ln (author's), paint, tpaint, topac, lit (rgb), light (sheet))."""
    from sheet_boss import Sheet
    sheet_dir = pathlib.Path(sheet_dir); info = json.loads((sheet_dir / 'pose.json').read_text()); sh = Sheet(sheet_dir)
    m, j = tl.ours_mesh(def_json); nf = len(m['faces']); seqs = info['sequences']
    cam = dict(zoom=info['zoom'], aim=info['aim'], cell=tuple(info['cell']))
    var = layer_variants(pathlib.Path(def_json).stem, m, j, sub); ents = [tl.entry(p, seqs) for p in var]
    lut = {(r << 16) | (g << 8) | b: c for (r, g, b), c in sh.rgb2col.items()}
    def cols(im):
        flat = (im[..., 0].astype(np.int64) << 16) | (im[..., 1].astype(np.int64) << 8) | im[..., 2]; u, inv = np.unique(flat, return_inverse=True)
        c = np.array([lut.get(int(x), -1) for x in u])[inv].reshape(flat.shape); c[im[..., 3] == 0] = -1; return c
    for yaw, pitch, rws in info['cameras']:
        rws = [r for r in rws if rows_filter is None or rows_filter(r)]
        if not rws: continue
        for row, cs in zip(rws, draw_layers(ents, rws, cam, yaw, pitch)):
            fA, fO, ln = decode(cs, nf); stem = sheet_dir / 'pose' / f"{row.replace(':', '')}-y{yaw}-p{pitch}"
            pa = np.asarray(Image.open(f'{stem}-paint.png')); tp = np.asarray(Image.open(f'{stem}-tpaint.png')); li = np.asarray(Image.open(f'{stem}-light.png'))
            yield dict(row=row, yaw=yaw, pitch=pitch, fA=fA, fO=fO, ln=ln, paint=cols(pa), tpaint=cols(tp), topac=tp[..., 3], lit=np.asarray(Image.open(f'{stem}-lit.png').convert('RGB')),
                       light=np.where(li[..., 1] > 0, li[..., 0].astype(float), np.nan))
if __name__ == '__main__':
    if sys.argv[1] == 'build': build(int(sys.argv[2]), sys.argv[3], '--gate-cells' in sys.argv)
