"""Blender: bend the test limbs.   blender -b --python-exit-code 1 -P bend/animate_bend.py -- IN.blend OUT.blend
bend (80 cycles, a key every 2): the elbow closes forward to 140 degrees by cycle 56 and the knee closes backward to
130; both linear in time, so cycle = angle / 2.5 for the elbow. Then both open again."""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))
import rig_pose as RP
a = sys.argv[sys.argv.index('--') + 1:]
rig = RP.load(a[0])
an = RP.Anim(rig, 'bend', 80, None, step=2)
an.pose(0, [])
an.pose(56, [('arm_fore', 'X', 140), ('leg_shin', 'X', -130), ('arm_hand', 'X', 20), ('leg_foot', 'X', 30)])
an.pose(80, [])
an.build(interpolation='LINEAR'); RP.save(a[1])
