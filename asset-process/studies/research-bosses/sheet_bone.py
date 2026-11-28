"""Reference sheet of a real creature for a HELD-OUT rebuild: what the author may see without seeing the mesh.
This copy (research-bosses) of research-creatures/sheet_npc.py also makes the sheet of a BONE-WEIGHTED creature.

usage: sheet_bone.py NPC_ID OUTDIR [--px N]
Writes OUTDIR/sheet-ID.png, OUTDIR/sheet-ID.json and, when the archive has one, a copy of the archive's lit
six-view sheet as OUTDIR/archive-ID.png.

What the sheet shows (as sheet_npc.py: CREATURES.md "The reference sheet"), and what changes for a weighted creature:
  PAINT MAPS   six orthographic views of the REST mesh, every visible face in its PAINTED colour, unlit, no edges, on a
               grid in MODEL units. For a weighted creature the rest mesh is the BIND pose (limbs spread, wings
               open): the game never shows it. So the sheet adds
  POSED VIEWS  the creature as the game draws it (lit, depth-tested) at frame 0 of its stand and at the middle of its
               attack, from four sides. These are for reading the stance; the paint is read from the paint maps.
  BONE MAPS    (instead of label maps) side, top and front views with every face in the colour of the bone that pulls
               it hardest, softened; over them the SKELETON at rest: a dot at each bone's joint, a line to its
               parent, the bone's number. The JSON has every bone: parent, depth, rest position, how many vertices it
               dominates, the weighted centroid and extent of everything it pulls, and its total weight.
               The skeleton is the contract: the rebuild keeps the real frame base and keyframe files and must weight
               its own vertices to these bones.
  PALETTE, NUMBERS as before, plus the share of vertices with one, two, three, four or more influences.
What it hides: vertex positions, the triangulation, where faces are spent, the per-vertex weights themselves.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, json, collections, shutil, colorsys
sys.path.insert(0, str(pathlib.Path(__file__).parent)); sys.path.insert(1, _AP.fix(r'<ROOT>\author-pack\tools'))
import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageFilter
import bone_rebuild_lib as BL
import gamecol
L = BL.L; cmesh = BL.cmesh
ARCH = pathlib.Path(_AP.fix(r'<REFERENCE_ARCHIVE>\npcs'))
VIEWS = [  # name, horizontal (axis, sign), vertical (axis, sign), nearer = larger (axis, sign)
    ('LEFT SIDE (looking at the -x flank)', (2, 1), (1, 1), (0, -1)), ('RIGHT SIDE (+x flank)', (2, -1), (1, 1), (0, 1)),
    ('FRONT (looking at the -z end)', (0, 1), (1, 1), (2, -1)), ('BACK (+z end)', (0, -1), (1, 1), (2, 1)),
    ('TOP (from above)', (0, 1), (2, 1), (1, -1)), ('BOTTOM (from below)', (0, 1), (2, -1), (1, 1))]
def font(n):
    try: return ImageFont.truetype('segoeui.ttf', n)
    except Exception: return ImageFont.load_default()
def label_colour(l):
    r, g, b = colorsys.hsv_to_rgb((l * 0.381966) % 1.0, 0.7, 1.0); return int(r * 255), int(g * 255), int(b * 255)
def raster(V, F, cols, view, lo, hi, px, pad=26):
    """Orthographic picture with a depth buffer; returns an RGB array and the function unit -> pixel."""
    _, (ha, hs), (va, vs), (da, ds) = view
    w, h = int((hi[ha] - lo[ha]) * px) + 2 * pad, int((hi[va] - lo[va]) * px) + 2 * pad
    hx = lambda x: ((x - lo[ha]) if hs > 0 else (hi[ha] - x)) * px + pad
    vy = lambda y: ((y - lo[va]) if vs > 0 else (hi[va] - y)) * px + pad
    img = np.zeros((h, w, 3), np.uint8); img[:] = (30, 36, 46); zb = np.full((h, w), -1e18)
    X, Y, Z = hx(V[:, ha]), vy(V[:, va]), V[:, da] * ds
    for f, c in zip(F, cols):
        x, y, z = X[f], Y[f], Z[f]
        x0, x1 = max(int(np.floor(x.min())), 0), min(int(np.ceil(x.max())), w - 1); y0, y1 = max(int(np.floor(y.min())), 0), min(int(np.ceil(y.max())), h - 1)
        if x1 < x0 or y1 < y0: continue
        gx, gy = np.meshgrid(np.arange(x0, x1 + 1) + .5, np.arange(y0, y1 + 1) + .5)
        den = (y[1] - y[2]) * (x[0] - x[2]) + (x[2] - x[1]) * (y[0] - y[2])
        if abs(den) < 1e-9: continue
        a = ((y[1] - y[2]) * (gx - x[2]) + (x[2] - x[1]) * (gy - y[2])) / den; b = ((y[2] - y[0]) * (gx - x[2]) + (x[0] - x[2]) * (gy - y[2])) / den; g = 1 - a - b
        inside = (a >= -1e-6) & (b >= -1e-6) & (g >= -1e-6); zz = a * z[0] + b * z[1] + g * z[2]
        sub = zb[y0:y1 + 1, x0:x1 + 1]; win = inside & (zz > sub)
        sub[win] = zz[win]; img[y0:y1 + 1, x0:x1 + 1][win] = c
    return img, hx, vy
def decorate(img, view, lo, hi, hx, vy, step=20):
    name, (ha, hs), (va, vs), _ = view; im = Image.fromarray(img); d = ImageDraw.Draw(im, 'RGBA'); f = font(10); ax = 'xyz'
    for g in range(int(np.floor(lo[ha] / step)) * step, int(hi[ha]) + step, step):
        if lo[ha] <= g <= hi[ha]: x = hx(g); d.line([(x, 0), (x, im.height)], fill=(255, 255, 255, 46 if g else 110)); d.text((x + 2, im.height - 12), str(g), fill=(200, 210, 220), font=f)
    for g in range(int(np.floor(lo[va] / step)) * step, int(hi[va]) + step, step):
        if lo[va] <= g <= hi[va]:
            y = vy(g); d.line([(0, y), (im.width, y)], fill=(255, 255, 255, 46 if g else 110))
            d.text((2, y - 11), str(-g if va == 1 else g), fill=(200, 210, 220), font=f)
    vert = 'height (up)' if va == 1 else f"{'+' if vs > 0 else '-'}{ax[va]} down the page"
    d.text((4, 2), f"{name}   across: {'+' if hs > 0 else '-'}{ax[ha]} to the right; {vert}", fill=(240, 240, 240), font=font(11))
    return im
def build(npc_id, out, px=None):
    npc_id = int(npc_id); out = pathlib.Path(out); out.mkdir(parents=True, exist_ok=True)
    d = L.npc_def(npc_id); weighted = BL.is_weighted(npc_id)
    m = BL.with_weights(f'npc:{npc_id}') if weighted else cmesh.load(f'npc:{npc_id}')
    V = m['V']; Lb = m['DOM'] if weighted else m['L']; word = 'bones' if weighted else 'labels'
    keep = (m['A'] != 255) & (m['T'] < 2); F, C = m['F'][keep], m['C'][keep]
    used = np.unique(F); lo, hi = V[used].min(0), V[used].max(0); ext = hi - lo
    big = ext.max() > 400; step = 50 if big else 20
    px = px or max(1.25 if big else 2.0, min(5.0, 520 / ext.max()))
    table = gamecol.table()
    paint = [tuple(int(x) for x in table[int(c)]) for c in C]
    fl = Lb[F]; maj = np.where(fl[:, 1] == fl[:, 2], fl[:, 1], fl[:, 0]); labc = [label_colour(int(l)) for l in maj]
    by = collections.defaultdict(list)
    allused = np.unique(m['F'])                      # labels count on undrawn faces too: they can define a pivot
    for v, l in zip(V[allused], Lb[allused]): by[int(l)].append(v)
    drawn_labels = set(Lb[used].tolist()); hidden_only = sorted(set(by) - drawn_labels)
    cents = {l: np.mean(vs, 0) for l, vs in by.items()}
    seqs = L.sequences(npc_id); sk = L.seq_frames(seqs.get('stand', seqs.get('walk', -1)))
    bones = None
    if weighted:
        base_id = BL.bonelib.base_of_seq(seqs['stand']); bones, base = BL.skeleton(base_id); tab = BL.bone_table(m)
    pics = []
    for view in VIEWS:
        img, hx, vy = raster(V, F, paint, view, lo, hi, px); pics.append(decorate(img, view, lo, hi, hx, vy, step))
    labs = []
    for view in (VIEWS[0], VIEWS[4], VIEWS[2]):
        img, hx, vy = raster(V, F, labc, view, lo, hi, px)
        # softened on purpose: the regions must be readable, the face edges between them must not be
        img = np.asarray(Image.fromarray(img).filter(ImageFilter.GaussianBlur(float(px) * 2.2)))
        im = decorate(img, view, lo, hi, hx, vy, step); dd = ImageDraw.Draw(im)
        if weighted:
            P = lambda r: (hx(r[view[1][0]]), vy(r[view[2][0]]))
            for b in bones:
                if b['parent'] >= 0: dd.line([P(b['rest']), P(bones[b['parent']]['rest'])], fill=(255, 255, 255), width=1)
            for b in bones:
                p = P(b['rest']); dd.ellipse([p[0] - 2, p[1] - 2, p[0] + 2, p[1] + 2], fill=(255, 255, 0) if b['bone'] in tab else (150, 150, 150))
                dd.text((p[0] + 3, p[1] - 6), str(b['bone']), fill=(255, 255, 255), font=font(10), stroke_width=2, stroke_fill=(0, 0, 0))
        else:
            for l, c in cents.items():
                p = (hx(c[view[1][0]]), vy(c[view[2][0]])); dd.text((p[0] - 5, p[1] - 6), str(l), fill=(255, 255, 255), font=font(11), stroke_width=2, stroke_fill=(0, 0, 0))
        labs.append(im)
    # palette
    area, unit, cen = cmesh.face_geometry(m); area = area[keep]; tot = area.sum(); pal = collections.Counter()
    for c, a in zip(C, area): pal[int(c)] += a
    palette = [dict(colour=c, hsl=list(cmesh.hsl(c)), share=round(a / tot, 4)) for c, a in pal.most_common()]
    meas = cmesh.measure(m, tuple(d['scale']))
    info = dict(npc_id=npc_id, name=d['name'], model_ids=d['models'], recolor=d['recolor'], scale=d['scale'], ambient=d['ambient'], contrast=d['contrast'] // 5, size_tiles=d['size'],
                skeleton=sk[2], weighted=weighted, sequences=seqs, sequence_frames={k: L.seq_frames(v)[0] for k, v in seqs.items()},
                bounds_model_units=dict(min=[int(x) for x in lo], max=[int(x) for x in hi], note='x left-right, y DOWN (height = -y), z along the body'),
                size_after_scale=[meas['size_x'], meas['size_y'], meas['size_z']],
                counts=dict(vertices=int(len(allused)), faces=int(keep.sum()), colours=len(pal), flat_faces=int((m['T'][keep] & 1).sum()), depth_offset_faces=int((m['D'][keep] != 0).sum()),
                            translucent_faces=int((m['A'][keep] > 0).sum()), undrawn_vote_faces=int((~keep).sum()), shells=meas['shells'], models=len(d['models'])),
                palette=palette)
    posed = []
    if weighted:
        infl = collections.Counter(min(len(m['W'][i]), 4) for i in allused); n = len(allused)
        info['counts']['bones_in_base'] = len(bones); info['counts']['bones_weighted'] = len(tab)
        info['influences'] = dict(one=round(infl[1] / n, 3), two=round(infl[2] / n, 3), three=round(infl[3] / n, 3), four_or_more=round(infl[4] / n, 3), unweighted=infl[0])
        info['all_keyframe_sequences_on_base'] = [dict(seq=s, frames=f) for s, f in BL.bonelib.seqs_on_base(base_id)]
        ext_of = {}
        for b in tab:
            idx = [i for i in allused if any(x == b for x, _ in m['W'][i])]; ext_of[b] = (V[idx].min(0), V[idx].max(0), len(idx))
        info['bones'] = [dict(bone=b['bone'], parent=b['parent'], depth=b['depth'], rest_position=[round(v, 1) for v in b['rest']],
                              vertices_dominated=int(tab[b['bone']][0]) if b['bone'] in tab else 0, vertices_touched=ext_of[b['bone']][2] if b['bone'] in tab else 0,
                              total_weight=round(tab[b['bone']][2], 1) if b['bone'] in tab else 0,
                              weighted_centroid=[round(float(x), 1) for x in tab[b['bone']][1]] if b['bone'] in tab else None,
                              min=[int(x) for x in ext_of[b['bone']][0]] if b['bone'] in tab else None, max=[int(x) for x in ext_of[b['bone']][1]] if b['bone'] in tab else None) for b in bones]
        info['note_rest_pose'] = 'the paint and bone maps show the BIND pose stored in the model; the posed views show the stand and the attack as the game draws them'
        # posed views by the client's own drawing
        rows = ['stand:0'] + ([f"attack:{L.seq_frames(seqs['attack'])[0] // 2}"] if 'attack' in seqs else [])
        D = float(max(meas['size_x'], meas['size_y'], meas['size_z'])); cs = 330; zoom = int(512 * D * 1.5 / cs)
        for yaw, nm in ((0, 'yaw 0'), (512, 'yaw 512'), (1024, 'yaw 1024'), (1536, 'yaw 1536')):
            a, _ = L.draw([L.entry_real(npc_id, seqs)], rows, zoom, yaw, 128, (cs, cs), int(meas['size_y'] / 3))
            im = Image.fromarray(a); ImageDraw.Draw(im).text((4, 2), f'POSED, lit: {nm}; rows: ' + ', '.join(rows), fill=(240, 240, 240), font=font(11)); posed.append(im)
    else:
        info['labels_only_on_undrawn_faces'] = hidden_only; info['counts']['labels'] = len(by)
        info['labels'] = {str(l): dict(vertices=len(vs), centroid=[round(float(x), 1) for x in cents[l]], min=[int(x) for x in np.min(vs, 0)], max=[int(x) for x in np.max(vs, 0)]) for l, vs in sorted(by.items())}
    (out / f'sheet-{npc_id}.json').write_text(json.dumps(info, indent=1))
    # compose
    gap = 8; rows_ = [pics[:4], pics[4:] + labs] + ([posed] if posed else [])
    def row_w(r): return sum(p.width for p in r) + gap * (len(r) - 1)
    def row_h(r): return max(p.height for p in r)
    W = max([row_w(r) for r in rows_] + [1500]); head = 150; palh = 24 * (-(-len(palette) // 8)) + 30
    H = head + sum(row_h(r) + gap for r in rows_) + palh + 2 * gap + 40
    S = Image.new('RGB', (W, H), (18, 22, 28)); dd = ImageDraw.Draw(S); f = font(13)
    if weighted:
        rig = (f"BONE-WEIGHTED: frame base {base_id} with {len(bones)} bones ({len(tab)} carry weight), deepest chain {max(b['depth'] for b in bones) + 1}; influences per vertex: one {info['influences']['one']:.0%}, two {info['influences']['two']:.0%}, "
               f"three {info['influences']['three']:.0%}, four or more {info['influences']['four_or_more']:.0%}; no vertex labels. THE MAPS SHOW THE BIND POSE; the bottom row shows it posed.")
        last = f"grid every {step} model units, {px:.2f} pixels per unit; edge numbers are model units (heights positive upward). Bone maps: face colour = the bone that pulls it hardest; dots and lines = the skeleton at rest; numbers = bones."
    else:
        rig = f"{len(by)} labels; undrawn faces (alpha 255: never seen, but they vote and carry labels {hidden_only})"
        last = f"grid every {step} model units, {px:.2f} pixels per unit; the numbers on the edges are model units (heights shown positive upward). Label maps: face colour = vertex label, number at the label's centroid."
    lines = [f"CREATURE REFERENCE SHEET (held-out rebuild)  |  NPC {npc_id}  {d['name']}  |  cache revision 239  |  PAINT, not lit: every face in its painted colour",
             f"models {d['models']}  recolours {len(d['recolor'])} pairs (already applied here)  |  definition: scale {d['scale']} (128 = natural), ambient {d['ambient']}, contrast {d['contrast'] // 5}, size {d['size']} tiles",
             f"model units: x {int(lo[0])}..{int(hi[0])}, height {int(-hi[1])}..{int(-lo[1])}, z {int(lo[2])}..{int(hi[2])}  (extent {int(ext[0])} x {int(ext[1])} x {int(ext[2])});  after scale {meas['size_x']:.0f} x {meas['size_y']:.0f} x {meas['size_z']:.0f}",
             f"counts: {len(allused)} vertices, {int(keep.sum())} drawn faces, {len(pal)} colours, {info['counts']['flat_faces']} flat faces, {info['counts']['depth_offset_faces']} with depth offsets, {info['counts']['translucent_faces']} translucent, {info['counts']['undrawn_vote_faces']} undrawn faces, {meas['shells']} shells.  " + rig,
             f"skeleton (frame base) {sk[2]};  sequences the gate plays: " + ', '.join(f'{k} {v} ({L.seq_frames(v)[0]} frames)' for k, v in seqs.items()),
             last]
    for i, t in enumerate(lines): dd.text((10, 8 + i * 22), t, fill=(230, 235, 240), font=f)
    y = head
    for r in rows_:
        x = 0
        for p in r: S.paste(p, (x, y)); x += p.width + gap
        y += row_h(r) + gap
    dd.text((10, y), 'PALETTE (hue, saturation, lightness : % of the drawn surface)', fill=(230, 235, 240), font=f); y += 24
    cw = W // 8
    for i, p in enumerate(palette):
        x, yy = (i % 8) * cw + 10, y + (i // 8) * 24
        dd.rectangle([x, yy, x + 30, yy + 18], fill=tuple(int(v) for v in table[p['colour']]), outline=(90, 90, 90))
        dd.text((x + 36, yy + 1), f"{p['hsl'][0]},{p['hsl'][1]},{p['hsl'][2]} : {100 * p['share']:.1f}%", fill=(220, 225, 230), font=font(12))
    S.save(out / f'sheet-{npc_id}.png')
    hits = list(ARCH.glob(f'*/{npc_id}.png'))
    if hits: shutil.copy(hits[0], out / f'archive-{npc_id}.png')
    print(out / f'sheet-{npc_id}.png', S.size, '| archive sheet copied' if hits else '| no archive sheet')
if __name__ == '__main__':
    a = sys.argv; build(a[1], a[2], float(a[a.index('--px') + 1]) if '--px' in a else None)
