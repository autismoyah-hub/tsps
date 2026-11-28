"""skelanim.py: author classic animations for a skeleton compiled by skelc.py, by JOINT NAME.

usage: skelanim.py CLIP.anim.json [OUT_DIR]     writes a loose sequence directory: seq.dat, frame_G_F.dat, base_B.dat,
                                                anim.txt (asset_skel.exe anim=DIR shows it; skelpack.py packs it)
CLIP.anim.json
{ "name": "sentinel_stand",
  "rig": "skel/rig.json",              the compiled skeleton (rig.json beside base_B.dat), path relative to this file
  "id": 20100, "frame_group": 20100,   new sequence id and new frame group id (archive 0)
  "sequence": {"priority": 5},         definition fields (animlib.SEQ_DEFAULTS names); "like": SEQ copies a real one's
  "lengths": [4, 4, 4, ...],           hold of each frame in client cycles (20 ms); or "frames": N, "hold": C
  "loop": true,                        the clip is a cycle: every channel must return, at the total time, to its value
                                       at 0 (turn channels may differ by whole turns of 256); checked, not assumed
  "keys": [ {"t": 0, "pose": {"ring_a.turn": [0, 0, 0], "body.move": [0, -6, 0]}},
            {"t": 32, "ease": "inout", "pose": {...}}, ... ],
  "symmetry": {"ring_a.turn": 128},   a part that looks the same after part of a turn may loop on that part (units)
  "waves": [ {"channel": "body.move", "axis": 1, "amp": 5, "period": 64, "phase": 0} ]   optional sine added on top
                                       of the keyed value (bobbing, sway): amp * sin(2 pi (t / period + phase)) }
Channels are JOINT.CHANNEL as rig.json names them (turn, move, scale, or a named extra channel) and JOINT.pivot for
the joint's origin offset. Values are what a frame stores: turn = [x, y, z] in 1/256 of a turn (64 = a quarter; the
client turns about z, then x, then y); move in units, +y is DOWN (model space); scale 128 = 1.0; alpha = [amount, 0, 0]
(added transparency in steps of 8: 32 = gone).
"ground": {"mesh": "../sentinel.bin", "channel": "body.move", "exclude": ["shadow"], "from": 44, "settle_from": 60, "height": 2}
   floor contact for falls: from time "from" any frame whose lowest vertex is under the floor is lifted onto it through
   that move channel; from "settle_from" the lowest vertex is put exactly on the floor (lifted or lowered).
On a child-first skeleton a joint's turn is LOCAL to its parent. Every channel rests at identity; there is no rest
sequence to copy on a skeleton of one's own: the bind pose IS the rest pose.
Interpolation: per channel, component by component, between its own keys; "ease" on a key shapes the segment
arriving at it (linear, in, out, inout, hold). Values are not wrapped: 0 -> 256 is one whole turn.
"""
import sys, json, math, pathlib
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'research-animation'))
import animlib as A

EASE = {
    'linear': lambda u: u, 'in': lambda u: u * u, 'out': lambda u: 1 - (1 - u) * (1 - u),
    'inout': lambda u: u * u * (3 - 2 * u), 'hold': lambda u: 0.0 if u < 1 else 1.0,
    'in3': lambda u: u ** 3, 'out3': lambda u: 1 - (1 - u) ** 3,
}


def rot_matrix(v):
    ax, ay, az = ((x & 255) * 2 * math.pi / 256 for x in v)
    sz, cz = math.sin(az), math.cos(az); sx, cx = math.sin(ax), math.cos(ax); sy, cy = math.sin(ay), math.cos(ay)
    Z = [[cz, sz, 0], [-sz, cz, 0], [0, 0, 1]]; X = [[1, 0, 0], [0, cx, -sx], [0, sx, cx]]; Y = [[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]]
    mul = lambda a, b: [[sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]
    return mul(Y, mul(X, Z))


def angle_between(v1, v2):
    a, b = rot_matrix(v1), rot_matrix(v2)
    tr = sum(a[i][j] * b[i][j] for i in range(3) for j in range(3))
    return math.degrees(math.acos(max(-1.0, min(1.0, (tr - 1) / 2))))


def load_rig(path):
    path = pathlib.Path(path)
    rig = json.loads(path.read_text())
    base = A.decode_base((path.parent / f'base_{rig["base_id"]}.dat').read_bytes())
    chan = {}
    for j, e in rig['joints'].items():
        chan[f'{j}.pivot'] = e['origin']
        for c, v in e['channels'].items(): chan[f'{j}.{c}'] = v['group']
    return rig, base, chan


def build(spec, here):
    rig, base, chan = load_rig(here / spec['rig'])
    bid = rig['base_id']
    lengths = spec.get('lengths') or [spec['hold']] * spec['frames']
    total = sum(lengths)
    starts = [sum(lengths[:k]) for k in range(len(lengths))]
    default = lambda g: [128] * 3 if base['types'][g] == 3 else [0] * 3
    def group(name):
        if name not in chan: raise SystemExit(f'unknown channel `{name}`; the rig has: ' + ', '.join(sorted(chan)))
        return chan[name]
    tracks = {}
    for k in sorted(spec['keys'], key=lambda k: k['t'] if 't' in k else starts[k['frame']]):
        t = k['t'] if 't' in k else starts[k['frame']]
        for name, v in k['pose'].items():
            assert len(v) == 3, f'{name}: three values'
            tracks.setdefault(group(name), []).append((t, list(v), k.get('ease', 'linear')))
    waves = {}
    for w in spec.get('waves', []): waves.setdefault(group(w['channel']), []).append(w)

    def sample(g, t):
        ks = tracks.get(g)
        if not ks: v = [float(x) for x in default(g)]
        elif t <= ks[0][0]: v = [float(x) for x in ks[0][1]]
        elif t >= ks[-1][0]: v = [float(x) for x in ks[-1][1]]
        else:
            for (t0, v0, _), (t1, v1, e) in zip(ks, ks[1:]):
                if t0 <= t <= t1:
                    u = EASE[e]((t - t0) / (t1 - t0)) if t1 > t0 else 1.0
                    v = [a + (b - a) * u for a, b in zip(v0, v1)]; break
        for w in waves.get(g, []):
            v[w.get('axis', 1)] += w['amp'] * math.sin(2 * math.pi * (t / w['period'] + w.get('phase', 0.0)))
        return [int(round(x)) for x in v]

    used = sorted(set(tracks) | set(waves))
    notes = []
    sym = {group(n): v for n, v in spec.get('symmetry', {}).items()}
    if spec.get('loop'):
        for g in used:
            a, b = sample(g, 0), sample(g, total)
            per = sym.get(g, 256)
            ok = all((x - y) % per == 0 for x, y in zip(a, b)) if base['types'][g] == 2 else a == b
            if not ok: notes.append(f'LOOP BROKEN on g{g}: value {a} at 0, {b} at the total time {total}')
    frames = []
    for t in starts:
        groups = {}
        for g in used:
            v = sample(g, t)
            if base['types'][g] == 2: v = [((x + 128) % 256) - 128 for x in v]       # stored small; the client wraps anyway
            if v != default(g): groups[g] = v
        frames.append(dict(base=bid, groups=groups))
    if spec.get('ground'):
        # floor contact: the author keys the tumble, the tool keeps the figure ON the floor
        sys.path.insert(0, str(HERE.parent / 'research-npc'))
        import meshio
        gr = spec['ground']; m = meshio.load(here / gr['mesh'])
        V = [list(v) for v in m['vertices']]; L = list(m['vertex_skins'])
        skip = {rig['labels'][n] for n in gr.get('exclude', [])}
        g = group(gr.get('channel', 'body.move'))
        for k, t in enumerate(starts):
            if t < gr.get('from', 0): continue
            P = A.apply_frame([list(v) for v in V], L, frames[k], base)
            low, lowlab = max((p[1], l) for p, l in zip(P, L) if l not in skip)        # y is down: positive = below the floor
            lowname = next((n for n, v in rig['labels'].items() if v == lowlab), lowlab)
            lift = low + gr.get('height', 0)
            if lift > 0 or t >= gr.get('settle_from', 1e9):
                v = frames[k]['groups'].get(g, [0, 0, 0]); v = [v[0], v[1] - lift, v[2]]
                frames[k]['groups'][g] = v
                notes.append(f'ground: frame {k} (t {t}) {gr.get("channel", "body.move")} y {v[1] + lift} -> {v[1]} (lowest point was {-low} above the floor, on {lowname})')
    return rig, base, chan, lengths, starts, frames, used, notes


def write(path, out=None):
    path = pathlib.Path(path); spec = json.loads(path.read_text())
    out = pathlib.Path(out) if out else path.parent / path.name.replace('.anim.json', '')
    out.mkdir(parents=True, exist_ok=True)
    for old in out.glob('frame_*.dat'): old.unlink()
    rig, base, chan, lengths, starts, frames, used, notes = build(spec, path.parent)
    bid = rig['base_id']; fg = spec['frame_group']
    ids, seen = [], {}
    for f in frames:
        e = A.encode_frame(f, base)
        assert A.decode_frame(e, base)['groups'] == f['groups'], 'encoder self-check'
        if e not in seen:
            seen[e] = len(seen) + 1; (out / f'frame_{fg}_{seen[e]}.dat').write_bytes(e)
        ids.append((fg << 16) | seen[e])
    s = dict(A.SEQ_DEFAULTS)
    if 'like' in spec:
        s = {k: v for k, v in A.sequence(spec['like']).items() if k not in ('ops', 'sounds', 'chat_frames')}
    s.update(spec.get('sequence', {}))
    s['frames'], s['lengths'] = ids, list(lengths)
    data = A.encode_sequence(s); back = A.decode_sequence(data)
    assert back['frames'] == ids and back['lengths'] == list(lengths)
    (out / 'seq.dat').write_bytes(data)
    (out / f'base_{bid}.dat').write_bytes(A.encode_base(base))
    name_of = {g: n for n, g in chan.items()}
    lines = [f'{spec.get("name")}: base {bid}, sequence id {spec.get("id")}, frame group {fg}, {len(frames)} frames ({len(seen)} distinct files), lengths {lengths}, '
             f'total {sum(lengths)} cycles = {sum(lengths) * 0.02:.2f} s = {sum(lengths) / 30:.2f} server ticks',
             f'definition: priority {s.get("priority")} loop_frames {s.get("loop_frames")} max_loops {s.get("max_loops")} precedence {s.get("precedence")} '
             f'priority_moving {s.get("priority_moving")} restart {s.get("restart")}' + (f' (fields copied from {spec["like"]})' if 'like' in spec else '')]
    for g in used:
        kind = base['types'][g]; d = [128] * 3 if kind == 3 else [0] * 3
        vals = [f['groups'].get(g, d) for f in frames]
        extra = ''
        if kind == 2 and len(vals) > 1:
            steps = [angle_between(vals[i], vals[(i + 1) % len(vals)]) for i in range(len(vals) - (0 if spec.get('loop') else 1))]
            extra = f'   largest step {max(steps):5.1f} deg, fastest {max(st / lengths[i] for i, st in enumerate(steps)):4.1f} deg/cycle'
        lines.append(f'  g{g:2d} {A.KIND.get(kind, kind):9s} {name_of[g]:18s} ' + ' | '.join(','.join(map(str, v)) for v in vals) + extra)
    lines += notes
    (out / 'anim.txt').write_text('\n'.join(lines) + '\n')
    print('\n'.join(lines[:2] + notes)); print(f'wrote {out}')
    return out


if __name__ == '__main__':
    write(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None)
