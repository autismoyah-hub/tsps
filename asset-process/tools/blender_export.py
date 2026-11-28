"""Blender -> the game's model format, to the corrected rules. Run inside Blender:

    blender -b scene.blend -P blender_export.py -- OUT.bin [--collection NAME] [--weapon | --labels V,F]

Rules enforced, not requested:
  * every mesh object in the scene (or collection) is exported with modifiers and transforms applied;
  * 1 Blender unit = 1 tile = 128 game units; Blender +Z is up, +Y is north;
  * positions are rounded to whole game units and welded: one vertex per position;
  * faces are smooth-shaded unless their material name ends in _flat (hsl_H_S_L_flat): a deliberate facet;
  * a face's colour is its material's, and the material must be named  hsl_H_S_L  (H 0-63, S 0-7, L 0-127),
    the game's own 16-bit colour, so tone steps are chosen exactly and nothing is converted from RGB;
  * a material name may end in _pN (N 0 to 11, after _flat if both): that face's draw priority. Within the
    model a higher priority is drawn later, so a thin strip lying on a surface is given a higher one. Default 4;
  * a material name may also end in _dN: a per-face depth offset. On scenery it makes a face lying exactly in
    another's plane show (use 3); on a player it does nothing, priority decides there;
  * faces keep Blender's outward normals.
  * append _tID and/or _aBYTE after the other suffixes for a revision-239 texture and raw transparency.
    An indexed cache import keeps its basis vertices. A new authored triangle uses its own ordered corners.
Prints a report: counts, size in game units, colours with their share of the surface, and warnings.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, re, math, collections
import bpy, bmesh

sys.path.insert(0, _AP.fix(r'<ROOT>\tools'))
from author import Mesh, hsl
import cache_material

argv = sys.argv[sys.argv.index('--') + 1:]
out = argv[0]
coll = argv[argv.index('--collection') + 1] if '--collection' in argv else None
# --weapon: label every vertex 50 and every face 29, as real right-hand weapons are, so it follows the hand when animated
vlabel, flabel = (50, 29) if '--weapon' in argv else (None, None)
if '--labels' in argv:
    vlabel, flabel = map(int, argv[argv.index('--labels') + 1].split(','))
objs = [o for o in (bpy.data.collections[coll].all_objects if coll else bpy.context.scene.objects) if o.type == 'MESH' and not o.hide_render]
if not objs:
    raise SystemExit('no mesh objects to export')
deps = bpy.context.evaluated_depsgraph_get()
base_priority = int(argv[argv.index('--priority')+1]) if '--priority' in argv else (10 if '--weapon' in argv else 4)
def export_authored():
    textures, alphas = [], []
    m = Mesh()
    share = collections.Counter()
    warn = []
    for ob in objs:
        ev = ob.evaluated_get(deps)
        me = ev.to_mesh()
        bm = bmesh.new(); bm.from_mesh(me); bmesh.ops.triangulate(bm, faces=bm.faces[:]); bm.normal_update()
        mw = ob.matrix_world
        nmat = mw.to_3x3().inverted_safe().transposed()
        for f in bm.faces:
            mat = ob.material_slots[f.material_index].material if ob.material_slots else None
            mt, texture, alpha = cache_material.parse(mat)
            h, s, l = int(mt[1]), int(mt[2]), int(mt[3])
            n = nmat @ f.normal
            pts = [mw @ v.co for v in f.verts]
            idx = [m.vert(p.x * 128, p.z * 128, p.y * 128) for p in pts]
            m.out = lambda c, n=n: (n.x, n.z, n.y)
            before = len(m.f)
            m.tri(idx[0], idx[1], idx[2], hsl(h, s, l), flat=bool(mt[4]), prio=int(mt[5]) if mt[5] else None, depth=int(mt[6]) if mt[6] else 0)
            if len(m.f) > before:
                textures.append(texture)
                alphas.append(alpha)
                share[(h, s, l)] += f.calc_area()
        bm.free(); ev.to_mesh_clear()
    if not m.f:
        raise SystemExit('nothing survived welding; is the object far too small? 1 Blender unit is 1 tile')
    import pathlib
    # A player is painted in priority order with no depth test inside the figure, so priority is a worn piece's whole
    # layering. Real weapons and shields are priority 10; scenery ignores priority.
    pathlib.Path(out).write_bytes(m.encode(priority=base_priority, vertex_label=vlabel, face_label=flabel, face_textures=textures, alpha=alphas))
    xs, ys, zs = zip(*m.v)
    tot = sum(share.values()) or 1
    print(f'ANIMATION LABELS: vertex {vlabel}, face {flabel}' if vlabel is not None else 'ANIMATION LABELS: none (scenery). A worn weapon needs --weapon.')
    print(f'EXPORT {out}: {len(m.v)} vertices, {len(m.f)} faces, welded; {sum(m.flat)} flat-shaded faces, the rest smooth; priorities used: {sorted(set(base_priority if p is None else p for p in m.prio))}')
    print(f'  size in game units: {max(xs)-min(xs)} wide (x), {max(ys)-min(ys)} tall, {max(zs)-min(zs)} deep (z); base at height {min(ys)}')
    for (h, s, l), a in share.most_common():
        print(f'  colour hsl_{h}_{s}_{l}: {100*a/tot:5.1f}% of surface')
    fam = collections.defaultdict(set)
    for (h, s, l) in share: fam[(h, s)].add(l)
    if all(len(v) == 1 for v in fam.values()):
        warn.append('every hue has a single tone: real assets paint shade with lightness steps inside a hue')
    if min(ys) != 0:
        warn.append(f'the base is at height {min(ys)}, not 0: the object will float or sink')
    for w in warn: print('  WARNING:', w)

if any('cache_basis' in ob.data for ob in objs):
    if len(objs) != 1:
        raise ValueError('export the indexed cache mesh in its own collection')
    cache_material.export_indexed(objs[0], deps, out, base_priority, vlabel, flabel, '--preserve-packing' in argv)
else:
    export_authored()
