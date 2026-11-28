"""Blender: the hard-bend test object: a THICK arm with an elbow and a thick leg with a knee, on a small new skeleton.
    blender -b --python-exit-code 1 -P bend/build_bend.py -- OUT.blend
Armature "rig" (7 bones): root; arm_upper (shoulder at height 170) -> arm_fore (elbow at 100) -> arm_hand (wrist at 30);
leg_thigh (hip at 170, 130 units to the side) -> leg_shin (knee at 100) -> leg_foot (ankle at 30).
Each limb: one welded 10-sided skin, radius 24 at the top to 17 at the wrist (limb length / radius about 3 a bone:
thick), rings every 10 units and three close rings at the joint (5 units apart), a swell on the upper part, capped.
Painted in bands with a pale stripe down the inner side of the bend and a dark one down the outer, so collapse,
pinch and stretch show. Weights are NOT assigned here."""
import sys, math, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))
import bpy
import rig_blender as RB
out = sys.argv[sys.argv.index('--') + 1]
RB.reset(); U = 128.0
arm = bpy.data.armatures.new('rig'); rig = bpy.data.objects.new('rig', arm); bpy.context.scene.collection.objects.link(rig)
bpy.context.view_layer.objects.active = rig; bpy.ops.object.mode_set(mode='EDIT')
def bone(name, head, tail, parent=None):
    b = arm.edit_bones.new(name); b.head = [x / U for x in head]; b.tail = [x / U for x in tail]; b.roll = 0
    if parent: b.parent = arm.edit_bones[parent]; b.use_connect = False
    return b
bone('root', (0, 0, 0), (0, 0, 20))
for nm, x, names in (('arm', 0, ('arm_upper', 'arm_fore', 'arm_hand')), ('leg', 130, ('leg_thigh', 'leg_shin', 'leg_foot'))):
    bone(names[0], (x, 0, 170), (x, 0, 100), 'root'); bone(names[1], (x, 0, 100), (x, 0, 30), names[0]); bone(names[2], (x, 0, 30), (x, 0, 8), names[1])
bpy.ops.object.mode_set(mode='OBJECT')
for b in arm.bones: b.inherit_scale = 'FULL'
for pb in rig.pose.bones: pb.rotation_mode = 'QUATERNION'
def hsl(h, s, l): return (h << 10) | (s << 7) | l
verts = []; faces = []; cols = []; SIDES = 10
heights = [178, 170, 160, 150, 140, 130, 120, 112, 105, 100, 95, 88, 80, 70, 60, 50, 40, 30, 20, 12]
def radius(h):
    t = (178 - h) / 166; r = 24 - 7 * t
    return r + 3.0 * math.exp(-((h - 140) / 22) ** 2) + 1.5 * math.exp(-((h - 72) / 18) ** 2)
for li, x0 in enumerate((0, 130)):
    base = len(verts); hue = 6 if li == 0 else 22
    for h in heights:
        for j in range(SIDES):
            a = 2 * math.pi * (j + 0.5 * (heights.index(h) % 2)) / SIDES; r = radius(h)            # alternate rings turned half a step: an irregular split
            verts.append(((x0 + r * math.sin(a)) / U, (-r * math.cos(a)) / U, h / U))
    top = len(verts); verts.append((x0 / U, 0, 184 / U)); bot = len(verts); verts.append((x0 / U, 0, 6 / U))
    for i in range(len(heights) - 1):
        for j in range(SIDES):
            a, b = base + i * SIDES + j, base + i * SIDES + (j + 1) % SIDES; c, d = a + SIDES, b + SIDES
            front = j in (0, 9); back = j in (4, 5); band = (i // 2) % 2
            col = hsl(hue, 3, 84 + 4 * band) if front else hsl(hue, 5, 26 + 3 * band) if back else hsl(hue, 5, 46 + 9 * band)
            if i % 2 == 0: tris = [(a, b, c), (b, d, c)]
            else: tris = [(a, b, d), (a, d, c)]
            faces += [(t[0], t[2], t[1]) for t in tris]; cols += [col, col]
    for j in range(SIDES):
        faces.append((base + j, base + (j + 1) % SIDES, top)); cols.append(hsl(hue, 4, 38))
        l = base + (len(heights) - 1) * SIDES; faces.append((l + (j + 1) % SIDES, l + j, bot)); cols.append(hsl(hue, 4, 30))
me = bpy.data.meshes.new('limbs'); me.from_pydata(verts, [], faces); slots = {}; idx = []
for c in cols:
    m = RB.material(c)
    if m.name not in slots: slots[m.name] = len(me.materials); me.materials.append(m)
    idx.append(slots[m.name])
me.polygons.foreach_set('material_index', idx); me.polygons.foreach_set('use_smooth', [True] * len(idx)); me.update()
ob = bpy.data.objects.new('limbs', me); bpy.context.scene.collection.objects.link(ob); ob['osrs_part'] = 0; ob['osrs_priority'] = 0
import numpy as np
V = np.array(verts); F = np.array(faces); vol = float(np.einsum('ij,ij->i', V[F[:, 0]], np.cross(V[F[:, 1]], V[F[:, 2]])).sum() / 6)
print(f'bend: {len(arm.bones)} bones, {len(verts)} vertices, {len(faces)} faces, {len(slots)} colours; signed volume {vol:.4f} (positive = faces outward)')
bpy.ops.wm.save_as_mainfile(filepath=str(pathlib.Path(out).resolve()))
