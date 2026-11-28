"""Blender: pose a WHOLE creature, not single bones. Built on rig_pose.py (turns about the armature's axes, Bezier
between poses, baked export); everything here returns or consumes the same turn lists.

    import rig_pose2 as P2
    rig = P2.load('creature.blend'); G = P2.Groups(rig); G.describe()          # print the chains: name them once
    a = P2.Anim2(rig, 'attack', length=90, stance=('seq_13926', 0), step=3)
    WING_L = G.fan('b076')            # every chain under a bone, as a list of chains (root to tip), nearest first
    a.define('coil', P2.curl(G.line('b124'), 'X', -70, bias=1.5) + a.both(P2.spread(WING_L, 'Y', 30)))
    a.define('lash', ...)
    a.pose(0, []); a.pose(24, 'coil', hold=3); a.pose(33, a.mix(('lash', 1.0), ('coil', -0.15))); a.pose(90, [])
    a.follow(G.line('b124'), each=1)                                           # overlap: each bone a cycle later
    a.hold(['b030', 'b060'], chain=2)                                          # these joints stay where the stance has them
    a.layer(P2.breath(['b148', 'b139'], 'X', 2.5, period=45) )                  # under every pose
    a.build(); P2.save('out.blend')

GROUPS, read from the armature (no names needed in the file):
  G.line(bone)          the bone and the bones below it along the main line (the child with the most bones below), to the tip
  G.chain(bone)         the bone and the single-child run below it (stops at a branch)
  G.fan(bone)           the lines that start at each child of `bone` (a hand's fingers, a wing's ribs), ordered by angle
  G.limb(bone)          every bone below and including `bone`
  G.mirror(bone)        its partner across the creature's middle (x -> -x), or itself on the middle line
  G.describe()          prints every chain: bones, length, side, partner
TURN LISTS (plain lists of (bone, axis, degrees) | (bone, 'MOVE', (dx, dy, dz)) | (bone, 'SCALE', s)):
  curl(line, axis, total, bias=1)       share `total` degrees along a line; bias > 1 puts more at the tip, < 1 at the root
  spread(lines, axis, total)            fan the ROOTS of several lines apart by `total` degrees in all (first -total/2 .. last +total/2)
  wave(line, axis, amp, phase, waves=1) a travelling S: bone k gets amp * sin(2 pi (waves * k / n - phase))
  scale(turns, k)                       every angle and move times k;   a.mix((name, w), ..)  named poses weighted and added
  a.mirror(turns) / a.both(turns)       the same on the other side (X turns keep their sign, Y and Z turns and x moves flip)
ANIM2 adds to rig_pose.Anim:
  define(name, turns), pose(frame, name | turns | mix, hold=N)
  follow(line, each=1, start=0)         follow-through: bone k of the line reaches every pose start + k * each cycles late
  hold(bones, chain=2)                  after every pose (and every baked frame) the `chain` bones above each listed bone are
                                        turned (cyclic coordinate descent) until the bone's joint is back where the STANCE has it:
                                        planted feet under a shifting body. shift(root, dx, dy, dz) is the move that needs it.
  layer(fn)                             fn(cycle) -> turn list applied ON TOP of whatever the poses give at that cycle (breath, sway);
                                        breath(bones, axis, degrees, period, phase=0, each=0) and sway(...) make such functions.
With a layer or a hold the action is BAKED here: a key every `step` cycles holding the final pose (what rig_export
writes anyway). One Blender frame = one client cycle.
"""
import sys, pathlib, math
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import bpy
import numpy as np
from mathutils import Matrix, Vector
import rig_blender as RB
import rig_pose as RP
from rig_pose import load, save
RM = RB.RM


class Groups:
    def __init__(self, rig):
        self.rig = rig; bones = RB.game_bones(rig); self.names = [b.name for b in bones]
        self.parent = {b.name: (RB.game_parent(b).name if RB.game_parent(b) else None) for b in bones}
        self.kids = {n: [] for n in self.names}
        for n, p in self.parent.items():
            if p: self.kids[p].append(n)
        self.head = {b.name: np.array(b.head_local) * RM.UNIT for b in bones}
        self.size = {}
        def sz(n):
            if n not in self.size: self.size[n] = 1 + sum(sz(c) for c in self.kids[n])
            return self.size[n]
        for n in self.names: sz(n)
        self.root = next(n for n in self.names if self.parent[n] is None)
        self.depth = {}
        for n in self.names:
            d = 0; q = n
            while self.parent[q]: q = self.parent[q]; d += 1
            self.depth[n] = d
        # mirror partners: the bone of the same depth nearest the mirrored joint (x -> -x)
        self.partner = {}; tol = 0.03 * max(float(np.ptp(np.array(list(self.head.values())), axis=0).max()), 1)
        for n in self.names:
            m = self.head[n] * np.array([-1, 1, 1]); best = min((o for o in self.names if self.depth[o] == self.depth[n]), key=lambda o: np.linalg.norm(self.head[o] - m))
            self.partner[n] = best if np.linalg.norm(self.head[best] - m) <= tol else None
    def name(self, b):
        if isinstance(b, int): return RB.bone_name(b)
        return b
    def line(self, bone):
        out = [self.name(bone)]
        while self.kids[out[-1]]: out.append(max(self.kids[out[-1]], key=lambda c: (self.size[c], c)))
        return out
    def chain(self, bone):
        out = [self.name(bone)]
        while len(self.kids[out[-1]]) == 1: out.append(self.kids[out[-1]][0])
        return out
    def limb(self, bone):
        b = self.name(bone); return [b] + [y for c in self.kids[b] for y in self.limb(c)]
    def fan(self, bone, min_len=1):
        b = self.name(bone); lines = [self.line(c) for c in self.kids[b]]; lines = [l for l in lines if len(l) >= min_len]
        if len(lines) < 2: return lines
        tips = np.array([self.head[l[-1]] - self.head[b] for l in lines]); c = tips.mean(0); u, s, vt = np.linalg.svd(tips - c)
        ang = np.arctan2((tips - c) @ vt[1], (tips - c) @ vt[0]) if len(lines) > 2 else (tips - c) @ vt[0]
        return [lines[i] for i in np.argsort(ang)]
    def mirror(self, bone):
        m = self.partner.get(self.name(bone)); return m if m else self.name(bone)
    def side(self, bone):
        x = self.head[self.name(bone)][0]; return 'C' if abs(x) < 2 else ('R' if x > 0 else 'L')          # +x is the model's right as drawn
    def describe(self, min_len=1):
        seen = set()
        def walk(n):
            if n in seen: return
            ch = self.chain(n); seen.update(ch)
            if len(ch) >= min_len:
                ln = sum(np.linalg.norm(self.head[a] - self.head[b]) for a, b in zip(ch, ch[1:])); h = self.head[ch[0]]
                print(f"  {ch[0]}..{ch[-1]} ({len(ch)} bones, {ln:.0f} units, side {self.side(ch[len(ch) // 2])}, starts at x {h[0]:.0f} front {-h[1]:.0f} up {h[2]:.0f}, {self.size[ch[0]] - len(ch)} more below, partner {self.partner.get(ch[0])})")
            for c in self.kids[ch[-1]]: walk(c)
        walk(self.root)


# ------------------------------------------------------------------------------------------------- turn lists
def curl(line, axis, total, bias=1.0):
    n = len(line); w = np.array([((k + 1) / n) ** (bias - 1) for k in range(n)]); w = w / w.sum()
    return [(b, axis, float(total * x)) for b, x in zip(line, w)]
def spread(lines, axis, total):
    n = len(lines)
    if n < 2: return [(l[0], axis, float(total)) for l in lines]
    return [(l[0], axis, float(total * (i / (n - 1) - 0.5))) for i, l in enumerate(lines)]
def wave(line, axis, amp, phase, waves=1.0, grow=0.0):
    n = len(line); return [(b, axis, float(amp * (1 + grow * k / n) * math.sin(2 * math.pi * (waves * k / n - phase)))) for k, b in enumerate(line)]
def scale(turns, k):
    out = []
    for b, ax, v in turns:
        if ax == 'MOVE': out.append((b, ax, tuple(float(x) * k for x in v)))
        elif ax == 'SCALE': out.append((b, ax, float(v) ** k))
        else: out.append((b, ax, float(v) * k))
    return out
def breath(bones, axis, degrees, period, phase=0.0, each=0.0):
    """A layer: every listed bone turns by degrees * sin(2 pi (t / period - phase - k * each)). Loops when the length is a multiple of the period."""
    return lambda t: [(b, axis, float(degrees * math.sin(2 * math.pi * (t / period - phase - k * each)))) for k, b in enumerate(bones)]
def sway(line, axis, amp, period, waves=1.0, phase=0.0, grow=0.0):
    """A layer: a travelling wave down a line (tail, neck, tentacle)."""
    return lambda t: wave(line, axis, amp, t / period + phase, waves, grow)
def windowed(fn, length, ramp=9):
    """A layer faded in over the first `ramp` cycles and out over the last: the action starts and ends ON its stance pose
    whatever the layer's phase (an action that must join another sequence without a jump)."""
    def f(t):
        k = min(1.0, t / ramp, (length - t) / ramp); k = max(0.0, k); k = k * k * (3 - 2 * k)
        return scale(fn(t), k)
    return f
def bob(bone, dz, period, phase=0.0, axis=2):
    """A layer: the bone (usually the root) rises and falls by dz units."""
    def f(t):
        v = [0.0, 0.0, 0.0]; v[axis] = float(dz * math.sin(2 * math.pi * (t / period - phase))); return [(bone, 'MOVE', tuple(v))]
    return f


class Anim2(RP.Anim):
    def __init__(self, rig, name, length, stance=None, step=3):
        super().__init__(rig, name, length, stance, step); self.G = Groups(rig); self.named = {}; self.layers = []; self.holds = []
        self.stance_world = {pb.name: pb.matrix.translation.copy() for pb in rig.pose.bones}
        for pb, m in zip(self.pbs, self.stance): pb.matrix_basis = m.copy()
        bpy.context.view_layer.update(); self.stance_world = {pb.name: pb.matrix.translation.copy() for pb in rig.pose.bones}
    # ---- named poses
    def define(self, name, turns): self.named[name] = self._flat(turns); return self.named[name]
    def _flat(self, turns):
        if isinstance(turns, str): return list(self.named[turns])
        out = []
        for b, ax, v in turns:
            if isinstance(b, (list, tuple)) and ax not in ('MOVE', 'SCALE'): out += [(x, ax, v / len(b)) for x in b]
            else: out.append((b, ax, v))
        return out
    def mix(self, *parts):
        out = []
        for name, w in parts: out += scale(self._flat(name), w)
        return out
    def pose(self, frame, turns, hold=0): super().pose(frame, self._flat(turns), hold)
    # ---- mirror
    def mirror(self, turns):
        out = []
        for b, ax, v in self._flat(turns):
            m = self.G.mirror(self.bone(b).name)
            if ax == 'MOVE': out.append((m, ax, (-v[0], v[1], v[2])))
            elif ax == 'SCALE' or ax == 'X': out.append((m, ax, v))
            elif ax in ('Y', 'Z'): out.append((m, ax, -v))
            else: raise ValueError('mirror: turns about a bone\'s own axes (LX, LY, LZ) have no defined mirror; use X, Y, Z')
        return out
    def both(self, turns): t = self._flat(turns); return t + self.mirror(t)
    # ---- overlap, holds, layers
    def follow(self, line, each=1, start=0): self.delay(list(line), start, each)
    def hold(self, bones, chain=2, iters=12): self.holds += [(self.bone(b).name, int(chain), int(iters)) for b in bones]
    def layer(self, fn): self.layers.append(fn)
    def shift(self, dx=0, dy=0, dz=0, bone=None): return [(bone or self.G.root, 'MOVE', (dx, dy, dz))]
    def _ccd(self):
        worst = 0.0
        for name, n, iters in self.holds:
            f = self.rig.pose.bones[name]; target = self.stance_world[name]; anc = []; q = f.parent
            while q is not None and len(anc) < n:
                if q.bone.get('osrs_helper') is None: anc.append(q)
                q = q.parent
            for _ in range(iters):
                if (f.matrix.translation - target).length < 0.3 / RM.UNIT: break
                for pb in anc:
                    piv = pb.matrix.translation.copy(); v1 = f.matrix.translation - piv; v2 = target - piv
                    if v1.length < 1e-9 or v2.length < 1e-9: continue
                    ax = v1.cross(v2)
                    if ax.length < 1e-12: continue
                    ang = v1.angle(v2); R = Matrix.Translation(piv) @ Matrix.Rotation(ang, 4, ax.normalized()) @ Matrix.Translation(-piv)
                    pb.matrix = R @ pb.matrix; bpy.context.view_layer.update()
            worst = max(worst, (f.matrix.translation - target).length * RM.UNIT)
        return worst
    def _apply(self, turns, base=None):
        saved = self.stance
        if base is not None: self.stance = base
        try: touched = super()._apply(turns)
        finally: self.stance = saved
        if self.holds: self.hold_gap = max(getattr(self, 'hold_gap', 0.0), self._ccd())
        return touched
    def build(self, interpolation='BEZIER'):
        act = super().build(interpolation)
        if not self.layers and not self.holds: return act
        # bake: the poses' curves (with their delays) sampled every `step` cycles, the layers on top, the holds after
        times = sorted(set(range(0, self.length + 1, self.step)) | {self.length}); n = len(self.pbs)
        loc = np.zeros((len(times), n, 3)); quat = np.zeros((len(times), n, 4)); sc = np.ones((len(times), n, 3)); scn = bpy.context.scene
        RB.fcurve_bag(self.rig, act)
        bases = []
        for t in times:
            scn.frame_set(int(t)); bpy.context.view_layer.update(); bases.append([pb.matrix_basis.copy() for pb in self.pbs])
        self.rig.animation_data.action = None
        for k, t in enumerate(times):
            turns = []
            for fn in self.layers: turns += self._flat(fn(t if t < self.length else 0))
            self._apply(turns, base=bases[k])
            for i, pb in enumerate(self.pbs):
                l, q, s = pb.matrix_basis.decompose(); loc[k, i] = l; quat[k, i] = (q.w, q.x, q.y, q.z); sc[k, i] = s
        props = {k: act[k] for k in act.keys()}
        bpy.data.actions.remove(act)
        act, bag = RB.write_action(self.rig, self.name, times, loc, quat, sc, self.form.names, 'LINEAR')
        for k, v in props.items(): act[k] = v
        act.use_frame_range = True; act.frame_start = 0; act.frame_end = self.length
        print(f'{self.name}: baked {len(times)} keys with {len(self.layers)} layers, {len(self.holds)} held joints (largest gap left {getattr(self, "hold_gap", 0.0):.1f} units)')
        self.action = act; return act
