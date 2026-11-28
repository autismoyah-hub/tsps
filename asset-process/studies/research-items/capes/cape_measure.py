"""cape_measure.py: how three real capes are built and how their rows move.

usage: cape_measure.py     (prints; no files written)
For Red cape 1007, Fire cape 6570, Attack cape 9747, male and female worn model:
  rows by vertex label (count, height range, z range, centroid), priorities per face-label band, colours;
  the cape groups of skeleton 0 that walk 819, run 824, slash 390 and stand 808 use, with their largest angle;
  displacement of every label's vertices per frame (studies/research-animation/animlib.apply_frame, the port of
  the client's deform.rs), cape alone: the pivots are then the cape's own label centroids, as on the figure
  (no other slot carries labels 9..16; label 10's pivot is the cape's own).
Coordinates printed (x, height, z); height = -y.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, collections
sys.path.insert(0, _AP.fix('<ROOT>/studies/research-items'))
sys.path.insert(0, _AP.fix('<ROOT>/studies/research-animation'))
import numpy as np
import itemlib as L
import animlib as A

IT = L.items()
CAPES = [1007, 6570, 9747]
ROWS = [2, 8, 10, 11, 9, 16, 14, 15, 13, 12, 6, 7]
CAPE_LABELS = {6, 7, 9, 10, 11, 12, 13, 14, 15, 16}
for cid in CAPES:
    d = IT[cid]
    print(f'\n=== {cid} {d["name"]}: wear_positions {d["wear_positions"]} wear_models {d["wear_models"]} offsets {d["wear_offset"]} '
          f'recolor {d["recolor"]} retexture {d["retexture"]} inventory model {d["model"]} zoom {d["zoom"]} angles {d["angles"]} offset {d["offset"]}')
    inv = L.model(d['model'])
    print(f'  inventory model: {len(inv["V"])} vertices {len(inv["F"])} faces, box {inv["V"].min(0).tolist()}..{inv["V"].max(0).tolist()} (x, y down, z), '
          f'priorities {dict(collections.Counter(inv["priorities"]))}, labels {sorted(set(inv["vertex_skins"] or [0]))}, textured {any(t != -1 for t in inv["face_textures"])}')
    for g, gname in enumerate(('male', 'female')):
        m = L.concat(L.worn_ids(d, g))
        V = m['V'] * [1, -1, 1]; lab = np.array(m['vertex_skins'])
        print(f'  {gname} worn {L.worn_ids(d, g)}: {len(V)} vertices {len(m["F"])} faces, box {V.min(0).tolist()}..{V.max(0).tolist()}, textured faces {sum(t != -1 for t in m["face_textures"])}')
        for l in ROWS:
            P = V[lab == l]
            if len(P):
                print(f'     label {l:2d}: {len(P):3d} vertices, height {P[:, 1].min()}..{P[:, 1].max()}, z {P[:, 2].min()}..{P[:, 2].max()}, |x| up to {np.abs(P[:, 0]).max()}, centroid {P.mean(0).round(1).tolist()}')
        pr = collections.defaultdict(collections.Counter)
        for f, p, fl in zip(m['F'], m['priorities'], m['face_skins']): pr[p][fl] += 1
        print('     priority -> {face label: faces}:', {p: dict(c) for p, c in sorted(pr.items())})
        # which side does each priority face: mean face normal z (cache winding), to tell inside from outside
        for p in sorted(pr):
            idx = [i for i, q in enumerate(m['priorities']) if q == p]
            F = m['F'][idx]; a, b, c = m['V'][F[:, 0]], m['V'][F[:, 1]], m['V'][F[:, 2]]
            n = np.cross(b - a, c - a).astype(float)
            cz = ((a + b + c) / 3)
            print(f'     priority {p}: {len(idx)} faces, height {-cz[:, 1].max():.0f}..{-cz[:, 1].min():.0f}, mean z of faces {cz[:, 2].mean():.1f}, '
                  f'normals toward +z (back) {np.mean(n[:, 2] > 0) * 100:.0f}% / -z (front) {np.mean(n[:, 2] < 0) * 100:.0f}%; colours {dict(collections.Counter(np.array(m["colors"])[idx].tolist()).most_common(4))}')

print('\n=== cape groups in the sequences (skeleton groups whose labels are only cape labels)')
for sid, name in ((808, 'stand'), (819, 'walk'), (824, 'run'), (390, 'slash'), (386, 'stab')):
    s, bid, base, frames = A.sequence_frames(sid)
    cape_groups = [i for i, labs in enumerate(base['labels']) if labs and set(labs) <= CAPE_LABELS | {109, 181}]
    used = collections.defaultdict(list)
    for fr in frames:
        for g, v in fr['groups'].items():
            if g in cape_groups: used[g].append(v)
    print(f'seq {sid} {name}: base {bid}, {len(frames)} frames; cape groups used: ' +
          ('; '.join(f'{g} ({A.KIND[base["types"][g]]} {base["labels"][g]}) in {len(v)} frames, largest value {np.abs(np.array(v)).max(0).tolist()}' for g, v in sorted(used.items())) or 'none'))
    for cid in CAPES:
        d = IT[cid]
        for g, gname in enumerate(('male', 'female')):
            m = L.concat(L.worn_ids(d, g))
            lab = list(m['vertex_skins'])
            rest = m['V'].astype(int)
            worst = collections.defaultdict(float)
            for fr in frames:
                verts = [list(map(int, v)) for v in rest]
                A.apply_frame(verts, lab, fr, base)
                dv = np.linalg.norm(np.array(verts) - rest, axis=1)
                for l in set(lab): worst[l] = max(worst[l], float(dv[np.array(lab) == l].max()))
            print(f'    {cid} {gname}: largest travel by label over the frames (units): ' + ', '.join(f'{l}: {worst[l]:.0f}' for l in ROWS if l in worst))
