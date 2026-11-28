"""Make an NPC copy of a worn piece that was authored for the player's priority painting. The player file is not touched.

usage: npc_convert.py IN.bin OUT.bin [--lift P:N[,P:N]] [--recolour H.S.L:H.S.L@MINH..MAXH[/MINZ..MAXZ][,..]] [--drop-priority P]

Why: an ordinary NPC is ONE combined model painted with a per-pixel depth test; face priorities are ignored
(world_actors.rs:500, model_order.rs:68, render3d.rs:864). A player piece leans on priority in two ways that a depth
test does not honour:
  1. an overlay lying exactly in its plate's plane (a chest device, eyes on a face) shows on a player because its
     priority is higher. Depth-tested, it fights with the plate. Real worn models that carry such overlays give those
     faces a per-face depth offset (the White Knight's helm 11744: its 8 visor faces at priority 7 carry offset 2).
     --lift P:N gives every face of priority P the depth offset N (which priorities are overlays is the piece's own
     design: read its notes).
  2. a lower-priority part that pokes out past a higher one is painted over on a player and SHOWS on an NPC (the legs'
     hose above the tassets at the back waist). --recolour repaints faces of one colour whose vertices all lie in a
     height band (and optionally a z band; back is +z) to another colour; --drop-priority removes a whole priority layer.
Labels, geometry and everything else are carried unchanged.
"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import meshio
a = sys.argv
def opt(n): return a[a.index(n) + 1] if n in a else None
m = meshio.load(a[1]); nf = len(m['faces']); notes = []
if opt('--lift'):
    for pair in opt('--lift').split(','):
        p, n = (int(v) for v in pair.split(':')); k = 0
        for i in range(nf):
            if m['priorities'][i] == p: m['depth'][i] = n; k += 1
        notes.append(f'priority {p}: {k} faces given depth offset {n}')
if opt('--recolour'):
    for rule in opt('--recolour').split(','):
        cols, band = rule.split('@'); band, _, zband = band.partition('/'); lo, hi = (int(v) for v in band.split('..'))
        zlo, zhi = (int(v) for v in zband.split('..')) if zband else (-9999, 9999)
        src, dst = (meshio.col(*(int(v) for v in c.split('.'))) for c in cols.split(':')); k = 0
        for i, f in enumerate(m['faces']):
            if m['colors'][i] == src and all(lo <= -m['vertices'][v][1] <= hi and zlo <= m['vertices'][v][2] <= zhi for v in f): m['colors'][i] = dst; k += 1
        notes.append(f'{cols} in heights {lo}..{hi}: {k} faces repainted')
if opt('--drop-priority'):
    p = int(opt('--drop-priority')); keep = [i for i in range(nf) if m['priorities'][i] != p]
    for key in ('faces', 'colors', 'priorities', 'render_types', 'alpha', 'depth', 'face_skins'): m[key] = [m[key][i] for i in keep]
    notes.append(f'priority {p}: {nf - len(keep)} faces removed')
meshio.save(m, a[2])
print(a[1], '->', a[2]); [print('  ', n) for n in notes]
