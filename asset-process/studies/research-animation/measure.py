"""Measure real classic animations: timing, which groups move, how far a joint turns per frame and per client cycle.

usage: measure.py survey [BASE]          every classic sequence on frame base BASE (default 0, the player):
                                         frame counts, frame lengths, durations, group usage, pivot offsets
       measure.py seq ID [ID...]         one line of timing and a per-joint table for each sequence
       measure.py track ID ENTRY [L,..]  path of posed label centroids per frame (through asset_anim.exe track=)
Angles: a rotate value is 1/256 of a turn (1.40625 degrees). A client cycle is 20 ms; a frame length is in cycles.
The turn between two frames of one joint is measured as the angle of the relative rotation (not the difference of
the three stored numbers).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, math, collections, statistics, subprocess, pathlib, json
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import animlib as A, rigs
NAMES = json.loads(pathlib.Path(_AP.fix(r'<TSPS_CHECKOUT>\server\data\definitions\sequence-names.json')).read_text(encoding='utf-8'))
EXE = _AP.fix(r'<CLIENT_BIN>\asset_anim.exe')


def rot_matrix(v):
    """The rotation a rotate group applies (deform.rs order: z, then x, then y), as a 3x3 matrix."""
    ax, ay, az = ((x & 255) * 2 * math.pi / 256 for x in v)
    sz, cz = math.sin(az), math.cos(az); sx, cx = math.sin(ax), math.cos(ax); sy, cy = math.sin(ay), math.cos(ay)
    Z = [[cz, sz, 0], [-sz, cz, 0], [0, 0, 1]]
    X = [[1, 0, 0], [0, cx, -sx], [0, sx, cx]]
    Y = [[cy, 0, sy], [0, 1, 0], [-sy, 0, cy]]
    mul = lambda a, b: [[sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]
    return mul(Y, mul(X, Z))


def angle_between(v1, v2):
    a, b = rot_matrix(v1), rot_matrix(v2)
    tr = sum(a[i][j] * b[i][j] for i in range(3) for j in range(3))
    return math.degrees(math.acos(max(-1.0, min(1.0, (tr - 1) / 2))))


def base_of(s):
    if not s['frames']: return None
    try:
        return int.from_bytes(A.frame_group(s['frames'][0] >> 16)[s['frames'][0] & 65535][:2], 'big')
    except Exception:
        return None


def joint_table(sid, quiet=False):
    s, bid, base, frames = A.sequence_frames(sid)
    L = s['lengths']
    rows = []
    used = sorted({g for f in frames for g in f['groups']})
    for g in used:
        kind = base['types'][g]
        default = [128] * 3 if kind == 3 else [0] * 3
        vals = [f['groups'].get(g, default) for f in frames]
        moving = any(v != vals[0] for v in vals)
        row = dict(group=g, kind=A.KIND.get(kind, kind), name=rigs.name(bid, g), moving=moving, values=vals)
        if kind == 2 and moving:
            steps = [angle_between(vals[i], vals[i + 1]) for i in range(len(vals) - 1)]
            # the turn from frame i to i+1 is shown over the time frame i is held
            row['max_step_deg'] = max(steps)
            row['max_rate_deg_per_cycle'] = max(st / max(L[i], 1) for i, st in enumerate(steps))
            row['range_deg'] = max(angle_between(vals[0], v) for v in vals)
        elif kind == 1 and moving:
            steps = [math.dist(vals[i], vals[i + 1]) for i in range(len(vals) - 1)]
            row['max_step_units'] = max(steps)
        rows.append(row)
    if not quiet:
        nm = NAMES.get(str(sid), '?')
        print(f'seq {sid} {nm}: base {bid}; {len(frames)} frames; lengths {L}; total {sum(L)} cycles = {sum(L) * 0.02:.2f} s = {sum(L) / 30:.2f} server ticks; '
              f'priority {s["priority"]} interleave {"yes (" + str(len(s["interleave"])) + " groups)" if s["interleave"] else "no"} loop_frames {s["loop_frames"]} max_loops {s["max_loops"]} '
              f'precedence {s["precedence"]} priority_moving {s["priority_moving"]} weapon {s["weapon"]} shield {s["shield"]}')
        for r in rows:
            if not r['moving']:
                continue
            extra = ''
            if 'max_step_deg' in r:
                extra = f'range {r["range_deg"]:5.1f} deg, largest step {r["max_step_deg"]:5.1f} deg, fastest {r["max_rate_deg_per_cycle"]:5.1f} deg/cycle'
            elif 'max_step_units' in r:
                extra = f'largest step {r["max_step_units"]:.1f} units'
            print(f'   g{r["group"]:3d} {r["kind"]:9s} {r["name"]:14s} {extra}')
        still = [f'{r["group"]}{"(" + r["name"] + ")" if r["name"] else ""}={r["values"][0]}' for r in rows if not r['moving']]
        print('   held constant in every frame:', ', '.join(still))
    return s, rows


def survey(bid):
    seqs = A.sequences()
    n = 0
    nframes, lengths, totals = [], [], []
    use_any = collections.Counter(); use_move = collections.Counter()
    pivots = collections.defaultdict(collections.Counter)
    interleave = collections.Counter(); prio = collections.Counter(); loops = collections.Counter()
    step_max = collections.defaultdict(list)
    base = A.frame_base(bid)
    for sid, data in seqs.items():
        s = A.decode_sequence(data)
        if s['cached_animation'] is not None or base_of(s) != bid:
            continue
        try:
            _, rows = joint_table(sid, quiet=True)
        except Exception:
            continue
        n += 1
        nframes.append(len(s['frames'])); lengths += s['lengths']; totals.append(sum(s['lengths']))
        interleave[tuple(s['interleave']) if s['interleave'] else None] += 1
        prio[s['priority']] += 1; loops[s['loop_frames'] != -1] += 1
        for r in rows:
            use_any[r['group']] += 1
            if r['moving']: use_move[r['group']] += 1
            if r['kind'] == 'origin':
                for v in r['values']: pivots[r['group']][tuple(v)] += 1
            if 'max_step_deg' in r: step_max[r['group']].append((r['max_step_deg'], r['max_rate_deg_per_cycle'], r['range_deg']))
    q = lambda xs, p: sorted(xs)[min(len(xs) - 1, int(p * len(xs)))]
    print(f'frame base {bid}: {len(base["types"])} groups; {n} classic sequences use it')
    print(f'frames per sequence: 10th {q(nframes, .1)}, median {q(nframes, .5)}, 90th {q(nframes, .9)}, max {max(nframes)}')
    print(f'frame length (cycles of 20 ms): 10th {q(lengths, .1)}, median {q(lengths, .5)}, 90th {q(lengths, .9)}; share of frames by length: '
          + ', '.join(f'{k}: {v * 100 // len(lengths)}%' for k, v in sorted(collections.Counter(min(x, 10) for x in lengths).items())) + ' (10 = 10 or more)')
    print(f'total length (cycles): 10th {q(totals, .1)}, median {q(totals, .5)}, 90th {q(totals, .9)}')
    print(f'priority: {dict(sorted(prio.items()))}; looping (loop_frames set): {loops[True]} of {n}')
    print(f'interleave lists: {sum(v for k, v in interleave.items() if k)} sequences have one; distinct lists {len([k for k in interleave if k])}; most common:')
    for k, v in interleave.most_common(4):
        print(f'   {v:5d} x {list(k) if k else None}')
    print('groups by how many sequences MOVE them (value changes between frames) / mention them:')
    for g, c in sorted(use_any.items(), key=lambda kv: -use_move[kv[0]])[:60]:
        st = step_max.get(g)
        extra = ''
        if st:
            extra = (f'  largest step per sequence: median {q([x[0] for x in st], .5):.0f} deg, 90th {q([x[0] for x in st], .9):.0f}; '
                     f'fastest rate: median {q([x[1] for x in st], .5):.1f} deg/cycle, 90th {q([x[1] for x in st], .9):.1f}; range median {q([x[2] for x in st], .5):.0f}, 90th {q([x[2] for x in st], .9):.0f}')
        print(f'   g{g:3d} {A.KIND.get(base["types"][g], "?"):9s} {rigs.name(bid, g):16s} moves in {use_move[g]:5d} ({use_move[g] * 100 // n}%), present in {c:5d}{extra}')
    print('pivot offsets (origin groups): most common value, share of the frames that mention the group')
    for g in sorted(pivots):
        tot = sum(pivots[g].values())
        (v, c), = pivots[g].most_common(1)
        if tot > 200:
            print(f'   g{g:3d} {rigs.name(bid, g):18s} {list(v)} in {c * 100 // tot}% of {tot} frames; next {pivots[g].most_common(3)[1:]}')


def track(sid, entry, labels):
    s = A.sequence(sid)
    fl = ','.join(str(i) for i in range(len(s['frames'])))
    out = pathlib.Path(_AP.fix(r'<ROOT>\render\research-animation-roundtrip\_track.bmp'))
    r = subprocess.run([EXE, str(A.CACHE), str(out), f'seq={sid}', f'frames=bind,{fl}', f'track={labels}', entry], capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    out.unlink()
    pos = collections.defaultdict(dict)
    for line in r.stdout.splitlines():
        if line.startswith('track '):
            p = line.split()
            pos[int(p[6])][p[4]] = (float(p[10]), float(p[12]), float(p[14]))
    L = s['lengths']
    for lab, d in pos.items():
        print(f'seq {sid} label {lab} centroid (x, height, z) per frame; step = distance from the previous frame, rate = step / length of the previous frame')
        prev = None
        for i in range(len(s['frames'])):
            p = d[str(i)]
            if prev is None:
                print(f'   bind {d["bind"]}\n   frame {i} len {L[i]}: {p}')
            else:
                st = math.dist(p, prev)
                print(f'   frame {i} len {L[i]}: {p}  step {st:5.1f} units, {st / L[i - 1]:5.1f} units/cycle')
            prev = p


if __name__ == '__main__':
    cmd = sys.argv[1]
    if cmd == 'survey': survey(int(sys.argv[2]) if len(sys.argv) > 2 else 0)
    elif cmd == 'seq':
        for x in sys.argv[2:]: joint_table(int(x))
    elif cmd == 'track': track(int(sys.argv[2]), sys.argv[3], sys.argv[4] if len(sys.argv) > 4 else '27,50')
