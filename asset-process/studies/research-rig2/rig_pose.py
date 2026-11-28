"""Blender: author a keyframe animation by POSING the armature (not by typing channel values).

    import sys; sys.path.insert(0, r'<ROOT>\\studies\\research-rig'); import rig_pose as RP
    rig = RP.load('creature.blend')
    a = RP.Anim(rig, 'idle_sway', length=90, stance=('seq_13926', 0), step=5)
    a.pose(0, [])                                              # the stance itself
    a.pose(30, [('b132', 'X', 12), (['b124', 'b123', 'b122'], 'Z', 30)], hold=0)
    a.pose(90, [])                                             # back where it began: it loops
    a.build(); RP.save('creature-anim.blend')
then rig_export.py --actions idle_sway writes the keyframe file (baked every `step` cycles, or --mode spline).

A pose = the stance with TURNS applied, parents before children:
  (bone, axis, degrees)            turn that bone (and all it carries) about an axis through its joint.
  ([bone, bone, ..], axis, deg)    a chain: the total is shared equally along the listed bones (a curl).
  axis 'X' | 'Y' | 'Z'             the ARMATURE's axes as the viewport shows them: X = the creature's right-left axis
                                   (a positive turn about X leans the top of a part toward the FRONT), Z = up (a
                                   positive turn about Z swings the front toward the creature's... check on a sheet),
                                   Y = front-back axis (a roll). 'LX' | 'LY' | 'LZ' = the bone's own axes (LY = along it).
  (bone, 'MOVE', (dx, dy, dz))     shift the bone in armature axes, Blender units x 128 = game units given here.
  (bone, 'SCALE', s)               uniform scale.
Between poses Blender interpolates each bone's channels with Bezier curves and auto-clamped handles (ease in and out
at every pose; no overshoot); `hold=N` repeats the pose N cycles later (a held drawing). What is exported is what the
viewport plays, sampled every `step` cycles.
Blender frame = client cycle (20 ms).
"""
import sys, json, pathlib, math
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import bpy
import numpy as np
from mathutils import Matrix, Vector
import rig_blender as RB
RM = RB.RM


def load(path):
    bpy.ops.wm.open_mainfile(filepath=str(pathlib.Path(path).resolve())); return RB.find_rig()
def save(path): bpy.ops.wm.save_as_mainfile(filepath=str(pathlib.Path(path).resolve()))


class Anim:
    def __init__(self, rig, name, length, stance=None, step=3):
        self.rig, self.name, self.length, self.step = rig, name, int(length), int(step); self.poses = []; self.delays = {}
        base, self.sk, _, _, _ = RB.export_base(rig); self.form = RB.Form(rig, self.sk); self.pbs = [rig.pose.bones[n] for n in self.form.names]
        self.stance_act = None
        if stance:
            act = bpy.data.actions[stance[0]]; RB.fcurve_bag(rig, act); bpy.context.scene.frame_set(int(stance[1])); bpy.context.view_layer.update()
            self.stance_act, self.stance_frame = act, int(stance[1])
        else:
            if rig.animation_data: rig.animation_data.action = None
            for pb in self.pbs: pb.matrix_basis = Matrix.Identity(4)
        self.stance = [pb.matrix_basis.copy() for pb in self.pbs]
        if rig.animation_data: rig.animation_data.action = None
    def bone(self, x):
        if isinstance(x, str) and x not in self.rig.pose.bones and x[:1] == 'b' and x[1:].isdigit(): x = int(x[1:])
        return self.rig.pose.bones[x if isinstance(x, str) else RB.bone_name(x)]
    def delay(self, bones, cycles, each=0):
        """Overlap: these bones arrive at every pose `cycles` late; with each=N the k-th listed bone a further k x N."""
        for k, b in enumerate(bones if isinstance(bones, (list, tuple)) else [bones]): self.delays[self.bone(b).name] = cycles + k * each
    def pose(self, frame, turns, hold=0):
        self.poses.append((int(frame), list(turns)))
        if hold: self.poses.append((int(frame) + int(hold), list(turns)))
    def _apply(self, turns):
        for pb, m in zip(self.pbs, self.stance): pb.matrix_basis = m.copy()
        bpy.context.view_layer.update(); touched = set()
        flat = []
        for b, ax, v in turns:
            if isinstance(b, (list, tuple)):
                for x in b: flat.append((x, ax, v / len(b)))
            else: flat.append((b, ax, v))
        for b, ax, v in flat:
            pb = self.bone(b); P = pb.matrix.copy(); touched.add(pb.name)
            if ax == 'MOVE':
                d = Vector(v) / RM.UNIT; pb.matrix = Matrix.Translation(d) @ P
            elif ax == 'SCALE':
                pb.scale = pb.scale * float(v)
            else:
                axis = {'X': Vector((1, 0, 0)), 'Y': Vector((0, 1, 0)), 'Z': Vector((0, 0, 1))}.get(ax)
                if axis is None: axis = (P.to_3x3() @ {'LX': Vector((1, 0, 0)), 'LY': Vector((0, 1, 0)), 'LZ': Vector((0, 0, 1))}[ax]).normalized()
                piv = P.translation.copy(); R = Matrix.Translation(piv) @ Matrix.Rotation(math.radians(v), 4, axis) @ Matrix.Translation(-piv)
                pb.matrix = R @ P
            bpy.context.view_layer.update()
        return touched
    def build(self, interpolation='BEZIER'):
        poses = sorted(self.poses, key=lambda p: p[0]); times = [p[0] for p in poses]; assert times[0] == 0 and times[-1] <= self.length and len(set(times)) == len(times), 'poses need distinct frames from 0'
        n = len(self.pbs); loc = np.zeros((len(poses), n, 3)); quat = np.zeros((len(poses), n, 4)); sc = np.ones((len(poses), n, 3)); touched = set()
        for k, (f, turns) in enumerate(poses):
            touched |= self._apply(turns)
            for i, pb in enumerate(self.pbs):
                l, q, s = pb.matrix_basis.decompose(); loc[k, i] = l; quat[k, i] = (q.w, q.x, q.y, q.z); sc[k, i] = s
        if self.name in bpy.data.actions: bpy.data.actions.remove(bpy.data.actions[self.name])
        act, bag = RB.write_action(self.rig, self.name, times, loc, quat, sc, self.form.names, interpolation)
        import re
        for fc in bag.fcurves:
            mt = re.match(r'pose\.bones\["(.+?)"\]', fc.data_path); d = self.delays.get(mt[1], 0) if mt else 0; kps = fc.keyframe_points
            if d and len(kps) > 2:                      # overlap: this bone reaches every pose d cycles late (first and last stay)
                for k in range(len(kps) - 2, 0, -1): kps[k].co.x = min(kps[k].co.x + d, kps[k + 1].co.x - 1)
            for kp in kps: kp.handle_left_type = kp.handle_right_type = 'AUTO_CLAMPED'
            fc.update()
        act['osrs_length'] = self.length; act['osrs_step'] = self.step; act.use_frame_range = True; act.frame_start = 0; act.frame_end = self.length
        if self.stance_act is not None and self.stance_act.get('osrs_file'):
            act['osrs_stance_file'] = self.stance_act['osrs_file']; act['osrs_stance_frame'] = self.stance_frame
        print(f'{self.name}: {self.length} cycles, {len(poses)} poses at {times}, {len(touched)} bones posed')
        self.action = act; return act
