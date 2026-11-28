"""Blender: a NEW bone skeleton from nothing, with a weighted mesh that BENDS: a standing worm.
    blender -b --python-exit-code 1 -P worm/build_worm.py -- OUT.blend
Armature "rig": 14 bones in one chain, root on the ground, each 24 game units long (root, seg01 .. seg13), described
the way an author would in Blender (edit bones head to tail, +Z up, 1 unit = 128 game units). No game data is read.
Mesh "worm": an 8-sided tapering tube, a ring every 12 units (two rings per bone, so skin crosses every joint), 336
units tall with a pointed tip; painted in bands (one per bone, two tones) with a dark stripe down the back and a pale
one down the belly, so a bend, a twist and a stretched face all show. Not an art piece.
Weights are NOT assigned here: rig_weights.py does that (method chain).
"""
import sys, math, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))
import bpy
import rig_blender as RB
out = sys.argv[sys.argv.index('--') + 1]
RB.reset(); U = 128.0; SEG = 24; NB = 14; SIDES = 8
arm = bpy.data.armatures.new('rig'); rig = bpy.data.objects.new('rig', arm); bpy.context.scene.collection.objects.link(rig)
bpy.context.view_layer.objects.active = rig; bpy.ops.object.mode_set(mode='EDIT'); prev = None
for k in range(NB):
    b = arm.edit_bones.new('root' if k == 0 else f'seg{k:02d}'); b.head = (0, 0, k * SEG / U); b.tail = (0, 0, (k + 1) * SEG / U); b.roll = 0
    if prev is not None: b.parent = prev; b.use_connect = True
    prev = b
bpy.ops.object.mode_set(mode='OBJECT')
for b in arm.bones: b.inherit_scale = 'FULL'
for pb in rig.pose.bones: pb.rotation_mode = 'QUATERNION'
# ---- mesh: rings every 12 units
H = NB * SEG; rings = list(range(0, H + 1, 12)); verts = []; faces = []; cols = []
def radius(h): return 34 - 22 * h / H
for h in rings:
    for j in range(SIDES):
        a = 2 * math.pi * (j + 0.5) / SIDES; r = radius(h); verts.append((r * math.sin(a) / U, -r * math.cos(a) / U, h / U))     # j = 0 and 7 face the front (-Y)
tip = len(verts); verts.append((0, 0, (H + 14) / U)); base = len(verts); verts.append((0, 0, 0))
def hsl(h, s, l): return (h << 10) | (s << 7) | l
for i in range(len(rings) - 1):
    band = (rings[i] // SEG) % 2
    for j in range(SIDES):
        a, b, c, d = i * SIDES + j, i * SIDES + (j + 1) % SIDES, (i + 1) * SIDES + (j + 1) % SIDES, (i + 1) * SIDES + j
        back = j in (3, 4); belly = j in (7,)
        col = hsl(8, 5, 26) if back else hsl(10, 3, 64 + 6 * band) if belly else hsl(9, 5, 44 + 10 * band)
        tris = [(a, b, c), (a, c, d)] if (i + j) % 2 == 0 else [(a, b, d), (b, c, d)]
        faces += tris; cols += [col, col]
top = (len(rings) - 1) * SIDES
for j in range(SIDES):
    faces.append((top + j, top + (j + 1) % SIDES, tip)); cols.append(hsl(8, 6, 34))
    faces.append((j, base, (j + 1) % SIDES)); cols.append(hsl(8, 5, 20))
me = bpy.data.meshes.new('worm'); me.from_pydata(verts, [], faces); slots = {}; idx = []
for c in cols:
    m = RB.material(c)
    if m.name not in slots: slots[m.name] = len(me.materials); me.materials.append(m)
    idx.append(slots[m.name])
me.polygons.foreach_set('material_index', idx); me.polygons.foreach_set('use_smooth', [True] * len(idx)); me.update()
ob = bpy.data.objects.new('worm', me); bpy.context.scene.collection.objects.link(ob); ob['osrs_part'] = 0; ob['osrs_priority'] = 0
import numpy as np
V = np.array(verts); F = np.array(faces); vol = float(np.einsum('ij,ij->i', V[F[:, 0]], np.cross(V[F[:, 1]], V[F[:, 2]])).sum() / 6)
print(f'worm: {NB} bones, {len(verts)} vertices, {len(faces)} faces, {len(slots)} colours, height {H + 14}; signed volume {vol:.4f} (positive = faces outward)')
bpy.ops.wm.save_as_mainfile(filepath=str(pathlib.Path(out).resolve()))
