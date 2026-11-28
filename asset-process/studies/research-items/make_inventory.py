"""make_inventory.py: the inventory (and ground) model of one of ours, and its icon view values, from the male worn model.

usage: make_inventory.py WORN.bin POSE OUT.bin --like REAL_ID [--row ID,ID,...] [--flip long,mid] [--no-fit]
                         [--ambient A --contrast C] [--name TEXT]
POSE   how the worn model is turned into the model that lies in the inventory and on the ground. Each is what real
       items do (measure-items.txt, section 2 and the rotations printed by q2.py):
  upright   moved only: boots and helms stand on the ground as worn (real "moved copies": 13 of 13 boots, 98 helms)
  lay       laid on its back, top toward +z: (x, y, z) -> (x, z, -y). Real legs and bodies that are rigid copies of
            their worn model use exactly this turn (44 of 46 legs, 7 of 7 bodies)
  gloves    the two hands brought together (inner edges 4 apart), then laid as `lay`
  weapon    laid flat along the neighbour's own inventory model: principal axes of ours (long, middle, thin) turned
            onto the neighbour's, the heavy end to the same end. --flip long / mid turns it end for end / over if
            the automatic choice is wrong (look at the icon).
After the turn the box is put where the neighbour's box is (centre in x and z, the same base height), so the
neighbour's icon angles and offsets apply to ours. Labels are dropped (real inventory models carry none: 0 to 1%);
priorities, colours, flat faces and alpha are kept. Faces above the model's lowest priority get a per-face depth
offset of 3 x their priority rank, because on the ground the model is drawn with the depth buffer, where a part lying
in another's plane needs one (RULES.md "How the game draws"); in the icon priorities decide and the offsets do nothing.

View values: angles and offsets start as the neighbour's. Unless --no-fit, zoom and offsets are then fitted with the
client's icon renderer (research_items.exe icons) until our drawing's box is the size and place of the
neighbour's (or of the median box of --row): the largest of width and height equal, centres equal.
Writes OUT.bin, OUT.view.json (zoom, angles, offset, ambient, contrast, and the fit's record).
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, subprocess, re, statistics as st
import numpy as np
import itemlib as L, itemcodec


def axes(V):
    """Principal axes (rows), longest first, each signed so that the heavier end (third moment) is positive."""
    c = (V.max(0) + V.min(0)) / 2.0
    P = V - V.mean(0)
    _, _, Vt = np.linalg.svd(P, full_matrices=False)
    ext = [(P @ a).max() - (P @ a).min() for a in Vt]
    Vt = Vt[np.argsort(ext)[::-1]]
    for i in range(3):
        if ((P @ Vt[i]) ** 3).sum() < 0: Vt[i] = -Vt[i]
    if np.linalg.det(Vt) < 0: Vt[2] = -Vt[2]
    return Vt


def pose(m, kind, like, flips=()):
    V = m['V'].astype(float)
    if kind == 'upright':
        out = V
    elif kind == 'lay':
        out = np.stack([V[:, 0], V[:, 2], -V[:, 1]], 1)
    elif kind == 'gloves':
        V = V.copy()
        r = V[:, 0] < 0
        V[r, 0] += -V[r, 0].max() - 2; V[~r, 0] -= V[~r, 0].min() - 2
        out = np.stack([V[:, 0], V[:, 2], -V[:, 1]], 1)
    elif kind == 'weapon':
        Ao, An = axes(V), axes(like['V'].astype(float))
        if 'long' in flips: Ao[0] = -Ao[0]; Ao[2] = -Ao[2]
        if 'mid' in flips: Ao[1] = -Ao[1]; Ao[2] = -Ao[2]
        out = (V - V.mean(0)) @ Ao.T @ An
    else:
        raise SystemExit(f'unknown pose {kind}')
    lo, hi = out.min(0), out.max(0); nlo, nhi = like['V'].min(0), like['V'].max(0)
    shift = np.array([(nlo[0] + nhi[0]) / 2 - (lo[0] + hi[0]) / 2, nhi[1] - hi[1], (nlo[2] + nhi[2]) / 2 - (lo[2] + hi[2]) / 2])
    return np.rint(out + shift).astype(int)


def icon_boxes(specs):
    r = subprocess.run([L.RESEARCH_ITEMS, L.CACHE, 'icons', str(L.HERE / 'data' / '_fit.bmp')] + specs, capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    out = []
    for line in r.stdout.splitlines():
        g = re.search(r': (\d+) pixels drawn, box x (\d+)\.\.(\d+) y (\d+)\.\.(\d+)', line)
        if g: out.append(tuple(int(x) for x in g.groups()))
        elif 'pixels drawn' in line: out.append((0, 0, 0, 0, 0))
    (L.HERE / 'data' / '_fit.bmp').unlink(missing_ok=True)
    return out


def fit_view(model_path, view, target, log):
    """target: (x0, x1, y0, y1) in the 36 x 32 cell. Adjusts zoom, then offsets."""
    tmp = L.HERE / 'data' / '_fit.def'
    tw, th = target[1] - target[0] + 1, target[3] - target[2] + 1
    tcx, tcy = (target[0] + target[1]) / 2, (target[2] + target[3]) / 2

    def draw(v):
        tmp.write_bytes(itemcodec.encode(dict(model=1000000, name='fit', zoom=v['zoom'], angles=v['angles'], offset=v['offset'], ambient=v['ambient'], contrast=v['contrast'])))
        return icon_boxes([f'def:{tmp}+model:{model_path}'])[0]
    best = None
    for step in range(14):
        n, x0, x1, y0, y1 = draw(view)
        if n == 0:
            view['zoom'] = int(view['zoom'] * 1.5); continue
        w, h = x1 - x0 + 1, y1 - y0 + 1
        ratio = max(w / tw, h / th)
        clipped = x0 == 0 or x1 == 35 or y0 == 0 or y1 == 31
        dx, dy = tcx - (x0 + x1) / 2, tcy - (y0 + y1) / 2
        log.append(f'zoom {view["zoom"]} offset {view["offset"]}: {n} px, box x {x0}..{x1} y {y0}..{y1} (target x {target[0]}..{target[1]} y {target[2]}..{target[3]})')
        err = abs(w - tw) + abs(h - th) if ratio <= 1.0 else 99
        score = (abs(ratio - 1) > 0.04, abs(dx) + abs(dy), abs(ratio - 1))
        if best is None or score < best[0]: best = (score, dict(view, offset=list(view['offset'])))
        if abs(ratio - 1) <= 0.04 and abs(dx) <= 0.5 and abs(dy) <= 0.5: break
        if abs(ratio - 1) > 0.04 or clipped:
            view['zoom'] = max(100, int(round(view['zoom'] * (ratio if not clipped or ratio > 1 else 1.15))))
        else:
            # one pixel is about zoom / 512 units at the model; x offset moves the drawing right, y offset down
            u = view['zoom'] / 512.0
            view['offset'] = [int(round(view['offset'][0] + dx * u)), int(round(view['offset'][1] + dy * u))]
    tmp.unlink(missing_ok=True)
    return best[1]


def main():
    a = [x for x in sys.argv[1:]]
    opt = lambda name, d=None: a[a.index(name) + 1] if name in a else d
    if len(a) < 3: raise SystemExit(__doc__)
    worn, kind, out = a[0], a[1], pathlib.Path(a[2])
    like_id = int(opt('--like'))
    IT = L.items(); d = IT[like_id]
    like = L.model(d['model'])
    like = dict(like, V=np.rint(like['V'] * (np.array(d['scale']) / 128.0)).astype(int))
    m = L.load_bin(worn)
    V = pose(m, kind, like, flips=(opt('--flip') or '').split(','))
    pr = list(m['priorities']); ranks = {p: i for i, p in enumerate(sorted(set(pr)))}
    m2 = dict(m, depth_bias=[6 * ranks[p] for p in pr])            # depth_bias is 2 x the stored face flag: 3 per rank
    L.save_bin(out, m2, V=V, vertex_skins=None, face_skins=None)
    lo, hi = V.min(0), V.max(0); nlo, nhi = like['V'].min(0), like['V'].max(0)
    print(f'{out}: {len(V)} vertices {len(m["F"])} faces, pose {kind}; box x {lo[0]}..{hi[0]} height {-hi[1]}..{-lo[1]} z {lo[2]}..{hi[2]}')
    print(f'  neighbour {like_id} {d["name"]!r} inventory model {d["model"]}: box x {nlo[0]}..{nhi[0]} height {-nhi[1]}..{-nlo[1]} z {nlo[2]}..{nhi[2]}; '
          f'zoom {d["zoom"]} angles {d["angles"]} offset {d["offset"]} ambient {d["ambient"]} contrast {d["contrast"]}')
    print(f'  priorities {sorted(ranks)} -> depth offsets {sorted({3 * r for r in ranks.values()})}; labels dropped')
    view = dict(zoom=d['zoom'], angles=list(d['angles']), offset=list(d['offset']), ambient=int(opt('--ambient', d['ambient'])), contrast=int(opt('--contrast', d['contrast'])))
    log = []
    row = [int(x) for x in opt('--row', str(like_id)).split(',')]
    boxes = icon_boxes([f'item:{i}' for i in row])
    target = tuple(int(round(st.median(b[k] for b in boxes))) for k in (1, 2, 3, 4))
    if '--no-fit' not in a:
        # first guess: scale the zoom by the longest extent
        view['zoom'] = int(round(view['zoom'] * max(hi - lo) / max(nhi - nlo)))
        view = fit_view(out, view, target, log)
    view.update(like=like_id, row=row, target_box=target, fit_log=log, pose=kind)
    out.with_suffix('.view.json').write_text(json.dumps(view, indent=1))
    print(f'  view: zoom {view["zoom"]} angles {view["angles"]} offset {view["offset"]} ambient {view["ambient"]} contrast {view["contrast"]} '
          f'(target box x {target[0]}..{target[1]} y {target[2]}..{target[3]} from {len(row)} real icons; {len(log)} fit renders)')
    if log: print('  last:', log[-1])


if __name__ == '__main__':
    main()
