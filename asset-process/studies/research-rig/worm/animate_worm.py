"""Blender: two posed animations on the worm's own skeleton.   blender -b --python-exit-code 1 -P worm/animate_worm.py -- IN.blend OUT.blend
sway   (stand, loops, 80 cycles, baked every 4): an S-bend travelling up the body, the tip lagging.
strike (60 cycles, baked every 3): coil back, hold, whip forward and down past the horizontal, settle.
Posed with rig_pose (turns about the armature's axes; chains share a turn; delays give the overlap)."""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))
import rig_pose as RP
a = sys.argv[sys.argv.index('--') + 1:]
rig = RP.load(a[0])
S = [f'seg{k:02d}' for k in range(1, 14)]; LOW, MID, TOP = S[:4], S[4:9], S[9:]
sw = RP.Anim(rig, 'sway', 80, None, step=4)
sw.delay(S, 0, each=1)                                         # each segment one cycle later than the one below: a wave
sw.pose(0, [(LOW, 'Y', 10), (MID, 'Y', -16), (TOP, 'Y', 10)])
sw.pose(20, [(LOW, 'Y', 22), (MID, 'Y', 6), (TOP, 'Y', -24), (S, 'X', 10)])
sw.pose(40, [(LOW, 'Y', -10), (MID, 'Y', 16), (TOP, 'Y', -10)])
sw.pose(60, [(LOW, 'Y', -22), (MID, 'Y', -6), (TOP, 'Y', 24), (S, 'X', -8)])
sw.pose(80, [(LOW, 'Y', 10), (MID, 'Y', -16), (TOP, 'Y', 10)])
sw.build()
st = RP.Anim(rig, 'strike', 60, None, step=3)
st.delay(TOP, 1, each=1)
st.pose(0, [(LOW, 'Y', 10), (MID, 'Y', -16), (TOP, 'Y', 10)])                                    # joins the sway's first pose
st.pose(15, [(LOW, 'X', -40), (MID, 'X', -50), (TOP, 'X', 60), (S, 'LY', 30)], hold=6)            # coiled back, head tucked forward, a twist
st.pose(27, [(LOW, 'X', 50), (MID, 'X', 70), (TOP, 'X', 20)])                                    # the whip: 6 cycles
st.pose(31, [(LOW, 'X', 46), (MID, 'X', 64), (TOP, 'X', 70)], hold=5)                            # the tip snaps over; held
st.pose(48, [(LOW, 'X', -8), (MID, 'X', -6), (TOP, 'X', -14)])
st.pose(60, [(LOW, 'Y', 10), (MID, 'Y', -16), (TOP, 'Y', 10)])
st.build()
RP.save(a[1])
