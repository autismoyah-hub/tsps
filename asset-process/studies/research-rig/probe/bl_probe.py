import bpy, sys, time
import numpy as np
from mathutils import Matrix, Vector, Quaternion
print('BLENDER', bpy.app.version_string, 'numpy', np.__version__, sys.version)
bpy.ops.wm.read_factory_settings(use_empty=True)
arm = bpy.data.armatures.new('rig'); ob = bpy.data.objects.new('rig', arm); bpy.context.scene.collection.objects.link(ob)
bpy.context.view_layer.objects.active = ob
bpy.ops.object.mode_set(mode='EDIT')
prev = None
for i in range(4):
    b = arm.edit_bones.new(f'b{i:03d}'); b.head = (0, 0, i * 1.0); b.tail = (0, 0, i + 1.0); b.roll = 0
    if prev: b.parent = prev
    prev = b
bpy.ops.object.mode_set(mode='OBJECT')
for b in arm.bones:
    b.inherit_scale = 'FULL'; b['osrs_index'] = 3
    print(b.name, [list(r) for r in b.matrix_local][:1], b.parent.name if b.parent else None)
# mesh: a tube
import bmesh
me = bpy.data.meshes.new('tube'); bm = bmesh.new()
bmesh.ops.create_cone(bm, segments=8, radius1=.3, radius2=.3, depth=4, cap_ends=True)
bmesh.ops.translate(bm, verts=bm.verts, vec=(0, 0, 2))
bmesh.ops.subdivide_edges(bm, edges=[e for e in bm.edges if abs(e.verts[0].co.z - e.verts[1].co.z) > 1], cuts=7)
bm.to_mesh(me); bm.free()
mo = bpy.data.objects.new('tube', me); bpy.context.scene.collection.objects.link(mo)
# auto weights in background
bpy.ops.object.select_all(action='DESELECT'); mo.select_set(True); ob.select_set(True); bpy.context.view_layer.objects.active = ob
try:
    r = bpy.ops.object.parent_set(type='ARMATURE_AUTO'); print('parent_set AUTO', r, [g.name for g in mo.vertex_groups], len(mo.modifiers))
    print('weights v0', [(g.group, round(g.weight, 3)) for g in me.vertices[0].groups])
except Exception as e: print('AUTO failed', e)
# action API
act = bpy.data.actions.new('test')
print('action attrs', [a for a in dir(act) if not a.startswith('_')])
ob.animation_data_create(); ob.animation_data.action = act
pb = ob.pose.bones['b001']; pb.rotation_mode = 'QUATERNION'
t = time.time()
pb.rotation_quaternion = Quaternion((0, 1, 0), 0.5); pb.keyframe_insert('rotation_quaternion', frame=0)
pb.rotation_quaternion = Quaternion((0, 1, 0), 1.0); pb.keyframe_insert('rotation_quaternion', frame=10)
print('slots', [s.name_display for s in act.slots] if hasattr(act, 'slots') else None, 'layers', len(act.layers) if hasattr(act, 'layers') else None)
print('has fcurves attr', hasattr(act, 'fcurves'))
try:
    from bpy_extras import anim_utils
    print('anim_utils', [a for a in dir(anim_utils) if 'channelbag' in a or 'slot' in a])
    cb = anim_utils.action_get_channelbag_for_slot(act, ob.animation_data.action_slot)
    print('channelbag fcurves', [(f.data_path, f.array_index, len(f.keyframe_points)) for f in cb.fcurves])
    fc = cb.fcurves.new('pose.bones["b002"].location', index=0)
    fc.keyframe_points.add(3); fc.keyframe_points.foreach_set('co', [0, 0, 5, 1, 10, 2]); fc.keyframe_points.foreach_set('interpolation', [1, 1, 1]); fc.update()
    print('new fcurve ok', [tuple(k.co) for k in fc.keyframe_points], fc.keyframe_points[0].interpolation, fc.group)
    print('fcurve attrs', [a for a in dir(cb.fcurves) if not a.startswith('_')])
except Exception as e:
    import traceback; traceback.print_exc()
bpy.context.scene.frame_set(5); bpy.context.view_layer.update()
print('pose matrix b002 at 5', [[round(x, 4) for x in r] for r in ob.pose.bones['b002'].matrix])
dg = bpy.context.evaluated_depsgraph_get(); ev = mo.evaluated_get(dg); m2 = ev.to_mesh(); print('deformed v', len(m2.vertices), tuple(round(c, 3) for c in m2.vertices[len(m2.vertices) - 1].co))
print('interp enum', bpy.types.Keyframe.bl_rna.properties['interpolation'].enum_items.keys()[:4])
bpy.ops.wm.save_as_mainfile(filepath=sys.argv[sys.argv.index('--') + 1])
