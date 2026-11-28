"""Author a keyframe (skeletal) animation on a real skeleton, from a small JSON, through bonecodec.encode_anim.

usage: bone_author.py SPEC.json OUTDIR
Writes OUTDIR/anim.dat (the archive-22 keyframe file), OUTDIR/seq.dat (a sequence definition naming it: priority,
opcode 13 = the asset id given in the spec, opcode 15 = range 0..length) and OUTDIR/anim.txt (what was written).

SPEC:
{ "name": "flinch",
  "base": 2653,                              the frame base (skeleton) the animation is for
  "stance": {"seq": 13926, "frame": 0},      the pose every channel STARTS from: the real sequence's value at that frame
                                             (a weighted creature's rest mesh is a spread bind pose, never a stance)
  "length": 60,                              client cycles (20 ms each); the sequence's range is 0..length
  "mode": "baked" | "spline",                baked  = what the real files do: one key every `step` cycles whose value is
                                                      our own interpolation, each key stepping to the next (the pose
                                                      changes every `step` cycles and is held between);
                                             spline = sparse keys with tangents: the CLIENT interpolates every cycle
  "step": 3,
  "asset": 65536000, "priority": 6,          for seq.dat only
  "tracks": [ {"bone": 134, "channel": "rot.x", "keys": [[0, 0], [6, 0.25], [24, -0.06], [48, 0]]}, ... ],
  "chains": [ {"bones": [124, 123, 122], "channel": "rot.x", "keys": [...], "delay": 2, "gain": 1.15} ] }
A track's values are OFFSETS added to the stance value of that channel (radians for rot.*, model units for tran.*, a
factor added to scale.*). Between two keys the offset eases with zero slope at both keys (the cubic 3s^2 - 2s^3),
which is exactly what the client computes for an unweighted curve whose tangents are (1, 0). A chain gives the same
keys to each bone in turn, each `delay` cycles later and `gain` times larger (follow-through along a tail or neck).
Rotation order in the client (skeletal.rs compose): about z, then x, then y, as quaternions; then scale, then
translation; a bone's matrix is local to its parent.
"""
import sys, json, pathlib, struct
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import bonelib as L, bonecodec as B
FMAX = struct.unpack('>f', struct.pack('>f', 3.4028234663852886e38))[0]
CH = {v: k for k, v in B.CHANNEL_NAMES.items()}

def value_at(curve, t):
    """The client's Curve::evaluate for an integer time inside the keys, for hold, step-to-next and unweighted spline
    segments (curve.rs); a weighted segment is refused."""
    keys = curve['keys']
    if t <= keys[0][0]: return keys[0][1]
    if t >= keys[-1][0]: return keys[-1][1]
    i = max(k for k in range(len(keys)) if keys[k][0] <= t); a = keys[i]
    if (a[4], a[5]) == (0.0, 0.0): return a[1]
    if a[4] == FMAX and a[5] == FMAX: return keys[i + 1][1] if t != a[0] else a[1]
    if curve['weighted']: raise NotImplementedError('weighted curve')
    b = keys[i + 1]; span = b[0] - a[0]; s0 = a[5] / a[4] if a[4] else 0.0; s1 = b[3] / b[2] if b[2] else 0.0
    dy = b[1] - a[1]; m0, m1 = span * s0, span * s1; c0 = (m0 + m1 - 2 * dy) / span ** 3; c1 = (3 * dy - 2 * m0 - m1) / span ** 2; dt = t - a[0]
    return a[1] + dt * (s0 + (c1 + dt * c0) * dt)
def ease(keys, t):
    if t <= keys[0][0]: return keys[0][1]
    if t >= keys[-1][0]: return keys[-1][1]
    for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
        if t0 <= t <= t1:
            s = (t - t0) / (t1 - t0); return v0 + (v1 - v0) * (3 * s * s - 2 * s * s * s)
def f32(v): return struct.unpack('>f', struct.pack('>f', v))[0]

def build(spec):
    base = B.decode_base(L.base_bytes(spec['base'])); nb = len(base['bones']); n = int(spec['length']); step = int(spec.get('step', 3)); mode = spec.get('mode', 'baked')
    aid, _ = L.anim_id(spec['stance']['seq']); like = B.decode_anim(L.anim_bytes(aid)); t0 = spec['stance'].get('frame', 0)
    assert like['base'] == spec['base'], 'the stance sequence is on another base'
    stance = {}; alpha = []
    for c in like['channels']:
        if c['kind'] == 1: stance[(c['target'], c['channel'])] = (value_at(c['curve'], t0), c['curve']['byte0'])
        elif c['kind'] == 4: alpha.append((c, value_at(c['curve'], t0)))
    offs = {}
    def add(bone, channel, keys):
        assert 0 <= bone < nb and channel in CH and CH[channel] <= 9, f'bad track {bone} {channel}'
        keys = sorted([float(a), float(b)] for a, b in keys); assert keys[0][0] >= 0 and keys[-1][0] <= n, 'keys must lie in 0..length'
        offs.setdefault((bone, CH[channel]), []).append(keys)
    for t in spec.get('tracks', []): add(t['bone'], t['channel'], t['keys'])
    for c in spec.get('chains', []):
        for i, bone in enumerate(c['bones']):
            d = c.get('delay', 0) * i; g = c.get('gain', 1.0) ** i; ks = c['keys']; last = ks[-1][0]
            # the first and last keys stay where they are; the keys between arrive `d` cycles later (never past the last)
            add(bone, c['channel'], [[k[0] if j in (0, len(ks) - 1) else min(last - (len(ks) - 1 - j), k[0] + d), k[1] * g] for j, k in enumerate(ks)])
    channels = []; moving = 0
    for bone in range(nb):
        for ch in range(1, 10):
            v0, byte0 = stance.get((bone, ch), (None, {1: 0, 2: 0, 3: 0, 4: 1, 5: 1, 6: 1}.get(ch, 3)))
            if v0 is None: raise SystemExit(f'the stance file has no channel {ch} for bone {bone}')
            tracks = offs.get((bone, ch))
            if not tracks:
                keys = [[0, f32(v0), 1.0, 0.0, 1.0, 0.0]]
            elif mode == 'baked':
                moving += 1; times = sorted(set(range(0, n + 1, step)) | {n})
                keys = [[t, f32(v0 + sum(ease(k, t) for k in tracks)), float(step), 0.0, FMAX, FMAX] for t in times]
                keys[-1][4:6] = [1.0, 0.0]
            else:
                moving += 1; times = sorted({int(k[0]) for tr in tracks for k in tr} | {0, n})
                if len(tracks) > 1: times = sorted(set(range(0, n + 1, step)) | {n})       # summed tracks: sample, still splined
                keys = [[t, f32(v0 + sum(ease(k, t) for k in tracks)), 1.0, 0.0, 1.0, 0.0] for t in times]
            channels.append(dict(kind=1, target=bone, channel=ch, curve=dict(byte0=byte0, before=0, after=0, weighted=0, keys=keys)))
    for c, v in alpha:
        channels.append(dict(kind=4, target=c['target'], channel=16, curve=dict(byte0=c['curve']['byte0'], before=0, after=0, weighted=0, keys=[[0, f32(v), 1.0, 0.0, 1.0, 0.0]])))
    return dict(version=1, base=spec['base'], range=[0, n], pose=0, channels=channels), moving

if __name__ == '__main__':
    spec = json.loads(pathlib.Path(sys.argv[1]).read_text()); out = pathlib.Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
    anim, moving = build(spec); data = B.encode_anim(anim)
    assert B.encode_anim(B.decode_anim(data)) == data
    (out / 'anim.dat').write_bytes(data)
    asset = int(spec.get('asset', 0)); n = spec['length']
    (out / 'seq.dat').write_bytes(bytes([5, spec.get('priority', 6), 13]) + asset.to_bytes(4, 'big') + bytes([15]) + (0).to_bytes(2, 'big') + n.to_bytes(2, 'big') + b'\0')
    keys = sum(len(c['curve']['keys']) for c in anim['channels'])
    text = f"{spec.get('name', 'animation')}: base {spec['base']}, {n} cycles ({n * 20} ms), mode {spec.get('mode', 'baked')}, {len(anim['channels'])} channels of which {moving} move, {keys} keys, {len(data)} bytes; stance = sequence {spec['stance']['seq']} frame {spec['stance'].get('frame', 0)}"
    (out / 'anim.txt').write_text(text + '\n'); print(text)
