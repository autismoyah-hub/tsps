"""Author a NEW classic animation from keyframes and encode it against an existing frame base.

usage: author.py ANIM.json [OUT_DIR]        writes a loose sequence directory (seq.dat, frame_G_F.dat, anim.txt)
                                            that asset_anim.exe / apose.py show and packcache.py packs.

ANIM.json
{
  "name": "meridian_cleave",
  "base": 0,                         frame base (0 = the player)
  "id": 20001,                       the new sequence id (only used when packing)
  "frame_group": 20001,              the new frame group id in archive 0 (frames are files 1..N of it)
  "like": 390,                       a real sequence whose definition fields are copied (priority, interleave,
                                     precedence, ... everything except frames, lengths and sounds); "sequence": {...}
                                     overrides single fields, e.g. {"priority": 6, "loop_frames": 4}
  "rest": {"seq": 808, "frame": 0},  the pose every channel holds until it is keyed: all group values of that
                                     real frame, INCLUDING its pivot offsets and the small hand shifts real frames carry
  "lengths": [4, 4, 4, 4, 8, 2, 1, 1, 8, 3],     frame lengths in client cycles (20 ms). Frame k is shown from
                                     the sum of the lengths before it, and holds the pose sampled AT THAT TIME.
  "keys": [
     {"t": 0,  "pose": "rest"},
     {"t": 16, "ease": "out", "pose": {"r_shoulder": [-61, 7, -41], "torso": [-4, 18, 0]}},
     {"frame": 6, "pose": {...}},    a key may name the frame whose start time it sits on instead of "t"
     {"t": 36, "pose": "rest"}
  ]
}
Channels: one per group. A pose names groups by rig name (rigs.py: head, torso, r_shoulder, r_elbow, r_hand, weapon,
l_shoulder, ..., root_move, torso_move, ...) or as "g48"; pivot offsets as "pivot:NAME" or "gN". Values are what the
frame stores: rotate = [x, y, z] in 1/256 turn (1.40625 degrees; the client turns about z, then x, then y, in the
figure's own axes, about the group's pivot); translate = units; scale = 128 is 1.0.
Between two keys of a channel the three numbers are interpolated one by one and rounded, which is what the real
frames show (slash 390 frames 0 to 4 are an exact linear ramp). "ease" on a key shapes the segment ARRIVING at it:
linear (default), in (slow start), out (slow finish), inout, hold (stay on the previous key, then jump).
"pose": "rest" keys every channel that is keyed anywhere back to its rest value.
A channel never keyed keeps its rest value in every frame. Identical frames are stored once and reused.
"""
import sys, json, math, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import animlib as A, rigs

EASE = {
    'linear': lambda u: u,
    'in': lambda u: u * u,
    'out': lambda u: 1 - (1 - u) * (1 - u),
    'inout': lambda u: u * u * (3 - 2 * u),
    'hold': lambda u: 0.0 if u < 1 else 1.0,
}


def group_of(base_id, key):
    if key.startswith('g') and key[1:].isdigit():
        return int(key[1:])
    table = rigs.PLAYER if base_id == 0 else getattr(rigs, 'RIGS', {}).get(base_id, {})
    if key.startswith('pivot:'):
        return table[key[6:]][2]
    if key not in table:
        raise SystemExit(f'unknown channel `{key}` for base {base_id}; use a rig name or gN')
    return table[key][0]


def build(spec):
    bid = spec.get('base', 0)
    base = A.frame_base(bid)
    lengths = spec['lengths']
    starts = [sum(lengths[:k]) for k in range(len(lengths))]
    rest = {}
    if spec.get('rest'):
        rs, rb, _, rframes = A.sequence_frames(spec['rest']['seq'])
        assert rb == bid, 'rest pose is on another frame base'
        rest = {g: list(v) for g, v in rframes[spec['rest'].get('frame', 0)]['groups'].items()}
    default = lambda g: [128] * 3 if base['types'][g] == 3 else [0] * 3
    # channels: group -> [(t, value, ease)]
    chan = {}
    keys = []
    for k in spec['keys']:
        t = starts[k['frame']] if 'frame' in k else k['t']
        keys.append((t, k))
    keys.sort(key=lambda x: x[0])
    keyed = set()
    for t, k in keys:
        if k['pose'] != 'rest':
            for name in k['pose']:
                keyed.add(group_of(bid, name))
    for t, k in keys:
        ease = k.get('ease', 'linear')
        if k['pose'] == 'rest':
            for g in keyed:
                chan.setdefault(g, []).append((t, list(rest.get(g, default(g))), ease))
        else:
            for name, v in k['pose'].items():
                g = group_of(bid, name)
                assert len(v) == 3, f'{name}: three values'
                chan.setdefault(g, []).append((t, list(v), ease))

    def sample(g, t):
        ks = chan[g]
        if t <= ks[0][0]:
            # before its first key a channel moves from rest (at time 0) to that key
            t0, v0 = 0, rest.get(g, default(g))
            t1, v1, e = ks[0]
            if t1 <= 0 or t >= t1: return list(v1)
            u = EASE[e](t / t1)
            return [round(a + (b - a) * u) for a, b in zip(v0, v1)]
        for (t0, v0, _), (t1, v1, e) in zip(ks, ks[1:]):
            if t0 <= t <= t1:
                if t1 == t0: return list(v1)
                u = EASE[e]((t - t0) / (t1 - t0))
                return [round(a + (b - a) * u) for a, b in zip(v0, v1)]
        return list(ks[-1][1])

    frames = []
    for t in starts:
        groups = {g: list(v) for g, v in rest.items()}
        for g in chan:
            groups[g] = sample(g, t)
        # drop identity entries (the client would skip them anyway), but keep explicit pivot offsets
        groups = {g: v for g, v in groups.items() if v != default(g)}
        frames.append(dict(base=bid, groups=groups))
    if spec.get('grip'):
        solve_grip(spec, bid, base, frames)
    return bid, base, lengths, starts, frames, keys


def load_mesh(path):
    verts, labels = [], []
    for line in pathlib.Path(path).read_text().splitlines():
        p = line.split()
        if len(p) == 4:
            verts.append([int(p[0]), int(p[1]), int(p[2])]); labels.append(int(p[3]))
    return verts, labels


def grip_state(verts, labels, frame, base, hand_label=28, other_label=27, weapon_label=50):
    """Pose the mesh; return (free hand centroid, holding hand centroid, weapon axis point, unit axis, extent).
    The axis is the weapon's longest direction (power iteration on its posed vertices), pointing to the end that is
    farther from the holding hand (the head of a polearm)."""
    v = A.apply_frame([list(x) for x in verts], labels, frame, base)
    cen = lambda lab: [sum(v[i][a] for i, l in enumerate(labels) if l == lab) / max(1, labels.count(lab)) for a in range(3)]
    w = [v[i] for i, l in enumerate(labels) if l == weapon_label]
    c = [sum(p[a] for p in w) / len(w) for a in range(3)]
    d = [1.0, 1.0, 1.0]
    for _ in range(12):
        n = [0.0, 0.0, 0.0]
        for p in w:
            q = [p[a] - c[a] for a in range(3)]; dot = sum(q[a] * d[a] for a in range(3))
            for a in range(3): n[a] += q[a] * dot
        L = math.sqrt(sum(x * x for x in n)) or 1.0
        d = [x / L for x in n]
    hold = cen(other_label)
    proj = [sum((p[a] - hold[a]) * d[a] for a in range(3)) for p in w]
    if -min(proj) > max(proj):
        d = [-x for x in d]; proj = [-x for x in proj]
    return cen(hand_label), hold, d, (min(proj), max(proj))


def solve_grip(spec, bid, base, frames):
    """Two-handed grip: for the frames named in spec['grip'] the LEFT arm (l_shoulder, l_elbow) is solved so the left
    hand's centroid lies on the weapon's long axis, "along" units from the right hand (negative = toward the butt).
    {"mesh": dump of the rest figure holding the weapon (asset_anim dump=), "frames": [first, last],
     "along": [lo, hi], "blend_in": n, "blend_out": n}  The first n / last n frames of the range are mixed with
    the keyed left arm so the hand travels to the shaft instead of jumping."""
    g = spec['grip']
    verts, labels = load_mesh(g['mesh'])
    a, b = g['frames']; lo, hi = g['along']
    GS, GE = rigs.PLAYER['l_shoulder'][0], rigs.PLAYER['l_elbow'][0]
    prev = None
    report = []
    for k in range(a, b + 1):
        f = frames[k]
        keyed = (list(f['groups'].get(GS, [0, 0, 0])), list(f['groups'].get(GE, [0, 0, 0])))
        cur = [list(prev[0]), list(prev[1])] if prev else [list(keyed[0]), list(keyed[1])]

        def cost(c):
            f['groups'][GS], f['groups'][GE] = list(c[0]), list(c[1])
            lh, rh, d, ext = grip_state(verts, labels, f, base)
            rel = [lh[i] - rh[i] for i in range(3)]
            al = sum(rel[i] * d[i] for i in range(3))
            alc = min(max(al, lo), hi)
            off = math.sqrt(sum((rel[i] - alc * d[i]) ** 2 for i in range(3)))
            ref = prev or keyed
            reg = 0.03 * sum(abs(c[j][i] - ref[j][i]) for j in range(2) for i in range(3))
            return off + reg, off, al, ext
        best = cost(cur)
        for step in (16, 8, 4, 2, 1):
            improved = True
            while improved:
                improved = False
                for j in range(2):
                    for i in range(3):
                        for sgn in (1, -1):
                            trial = [list(cur[0]), list(cur[1])]; trial[j][i] += sgn * step
                            if abs(trial[j][i]) > 110: continue
                            c = cost(trial)
                            if c[0] < best[0] - 1e-6:
                                cur, best, improved = trial, c, True
        prev = (list(cur[0]), list(cur[1]))
        w = min(1.0, (k - a + 1) / (g.get('blend_in', 0) + 1), (b - k + 1) / (g.get('blend_out', 0) + 1))
        mix = [[round(keyed[j][i] + (cur[j][i] - keyed[j][i]) * w) for i in range(3)] for j in range(2)]
        f['groups'][GS], f['groups'][GE] = mix
        _, off, al, ext = cost(mix)
        f['groups'][GS], f['groups'][GE] = mix
        report.append(f'grip f{k}: weight {w:.2f} l_shoulder {mix[0]} l_elbow {mix[1]} -> left hand {off:.1f} units off the shaft, {al:.0f} along (weapon spans {ext[0]:.0f}..{ext[1]:.0f} from the right hand)')
    spec['_grip_report'] = report


def write(spec, out):
    out = pathlib.Path(out); out.mkdir(parents=True, exist_ok=True)
    for old in out.glob('frame_*.dat'): old.unlink()
    bid, base, lengths, starts, frames, keys = build(spec)
    fg = spec.get('frame_group', 20000)
    encoded, ids, seen = [], [], {}
    for f in frames:
        e = A.encode_frame(f, base)
        assert A.decode_frame(e, base)['groups'] == f['groups'], 'encoder self-check'
        if e not in seen:
            seen[e] = len(seen) + 1
            (out / f'frame_{fg}_{seen[e]}.dat').write_bytes(e)
        ids.append((fg << 16) | seen[e])
    if 'like' in spec:
        s = A.sequence(spec['like'])
        s = {k: v for k, v in s.items() if k not in ('ops', 'sounds', 'chat_frames')}
    else:
        s = dict(A.SEQ_DEFAULTS)
    s.update(spec.get('sequence', {}))
    s['frames'], s['lengths'] = ids, list(lengths)
    data = A.encode_sequence(s)
    back = A.decode_sequence(data)
    assert back['frames'] == ids and back['lengths'] == list(lengths), 'sequence self-check'
    (out / 'seq.dat').write_bytes(data)
    lines = [f'{spec.get("name", "?")}: base {bid}, sequence id {spec.get("id")}, frame group {fg}, {len(frames)} frames ({len(seen)} distinct files), '
             f'lengths {lengths}, total {sum(lengths)} cycles = {sum(lengths) * 0.02:.2f} s',
             f'definition: priority {s.get("priority")} interleave {s.get("interleave")} loop_frames {s.get("loop_frames")} max_loops {s.get("max_loops")} '
             f'precedence {s.get("precedence")} priority_moving {s.get("priority_moving")} restart {s.get("restart")} stretch {s.get("stretch")} '
             f'weapon {s.get("weapon")} shield {s.get("shield")}' + (f' (fields copied from {spec["like"]})' if 'like' in spec else ''),
             'frame start times: ' + ', '.join(f'f{k}@{t}' for k, t in enumerate(starts))]
    off = [t for t, _ in keys if t not in starts and t < sum(lengths)]
    if off:
        lines.append(f'NOTE keys at times {off} do not sit on a frame start: those poses are never shown exactly')
    used = sorted({g for f in frames for g in f['groups']})
    import measure
    for g in used:
        kind = base['types'][g]
        d = [128] * 3 if kind == 3 else [0] * 3
        vals = [f['groups'].get(g, d) for f in frames]
        if all(v == vals[0] for v in vals):
            continue
        extra = ''
        if kind == 2:
            steps = [measure.angle_between(vals[i], vals[i + 1]) for i in range(len(vals) - 1)]
            extra = f'  largest step {max(steps):5.1f} deg, fastest {max(st / lengths[i] for i, st in enumerate(steps)):5.1f} deg/cycle'
        lines.append(f'  g{g:3d} {A.KIND.get(kind, kind):9s} {rigs.name(bid, g):14s} ' + ' | '.join(','.join(map(str, v)) for v in vals) + extra)
    lines += spec.get('_grip_report', [])
    (out / 'anim.txt').write_text('\n'.join(lines) + '\n')
    print('\n'.join(lines))
    print(f'wrote {out}')


if __name__ == '__main__':
    p = pathlib.Path(sys.argv[1])
    spec = json.loads(p.read_text())
    write(spec, sys.argv[2] if len(sys.argv) > 2 else p.with_suffix('').with_suffix('') if p.name.endswith('.anim.json') else p.with_suffix(''))
