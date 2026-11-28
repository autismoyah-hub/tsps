"""Small helpers for authoring inside Blender in GAME UNITS (128 to a tile, a person is about 200 tall).

Coordinates given to these helpers are (x, y, z) with y UP, as on the reference sheets: x is the sheet's
left-right in the FRONT view, y is height above the floor, z is depth (the FRONT view looks from -z).
Import inside a Blender script:
    import sys; sys.path.insert(0, r'<ROOT>\\tools'); from blender_kit import *
"""
import bpy

_mats = {}


def reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    _mats.clear()


def mat(h, s, l, flat=False, prio=None, depth=None, texture=None, alpha=0):
    """Cache texture ID (None for HSL), raw transparency byte (0 opaque).
    New textured triangles map their three cache corners to the texture basis."""
    name = f'hsl_{h}_{s}_{l}' + ('_flat' if flat else '') + (f'_p{prio}' if prio is not None else '') + (f'_d{depth}' if depth else '')
    # depth: per-face depth offset; on scenery, 3 makes a face lying in another's plane show
    if texture is not None:
        if type(texture) is not int or not (0 <= texture <= 210 and texture != 54):
            raise ValueError('unknown revision-239 texture ID')
        name += f'_t{texture}'
    if type(alpha) is not int or not 0 <= alpha <= 255:
        raise ValueError('transparency must be a byte')
    if alpha:
        name += f'_a{alpha}'
    # flat: a flat-shaded facet.  prio 0-11: draw priority, lower drawn first (default 4); real furniture layers
    # whole parts by height (legs 0, seat 1, back 2). It does not make a coplanar strip show.
    if name not in _mats:
        _mats[name] = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    return _mats[name]


def add_mesh(name, verts, faces, colours):
    """verts: (x, y_up, z) in game units. faces: tuples of 3 or 4 indices. Winding: for corners a, b, c in the order
    given, the plain cross product (b - a) x (c - a), computed in these (x, y up, z) coordinates, must point OUT of
    the object. (An earlier wording of this rule was wrong and turned two authors' first builds inside out.) Check
    in round 1: an inside-out model shows its far side and loses single-sided parts. For closed shells you can call
    recalc_outward(ob) instead. colours: one (h, s, l), (h, s, l, flat), (h, s, l, flat, prio) or (h, s, l, flat, prio, depth) per face. Returns the object."""
    me = bpy.data.meshes.new(name)
    # (x, y_up, z) -> Blender (x, z, y_up)/128 mirrors the shape, so each face is reversed to stay outward
    me.from_pydata([(x / 128, z / 128, y / 128) for x, y, z in verts], [], [tuple(reversed(f)) for f in faces])
    slots = {}
    for i, c in enumerate(colours):
        m = mat(*c)
        if m.name not in slots:
            slots[m.name] = len(me.materials); me.materials.append(m)
        me.polygons[i].material_index = slots[m.name]
    me.update()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def recalc_outward(ob):
    """Let Blender make a closed shell's normals consistent and outward. Use only on closed shells."""
    import bmesh
    bm = bmesh.new(); bm.from_mesh(ob.data); bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:]); bm.to_mesh(ob.data); bm.free()


def save(path):
    bpy.ops.wm.save_as_mainfile(filepath=path)
