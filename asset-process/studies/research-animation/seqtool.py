"""Classic sequences as loose files and as readable tables.

usage: seqtool.py dump SEQ [SEQ...]          the definition and, per frame, every group with its values
       seqtool.py export SEQ DIR [--raw]     decode the sequence and its frames from the cache, RE-ENCODE them with
                                             animlib's encoders and write a loose sequence directory
                                             (DIR/seq.dat, DIR/frame_G_F.dat) for asset_anim.exe anim=DIR.
                                             Prints whether each re-encoded file equals the cache's bytes.
                                             --raw writes the cache's own bytes instead (a control).
       seqtool.py json SEQ OUT.json          the decoded sequence as JSON (definition fields and per-frame groups)
"""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import animlib as A

# names for the player frame base (base 0); see FORMAT.md / RIG below in rig_player.json
def group_name(bid, g):
    try:
        import rigs
        return rigs.name(bid, g)
    except Exception:
        return ''


def dump(sid):
    s, bid, base, frames = A.sequence_frames(sid)
    print(f'seq {sid}: base {bid}, {len(frames)} frames, lengths {s["lengths"]} (total {sum(s["lengths"])} client cycles of 20 ms = '
          f'{sum(s["lengths"]) * 0.02:.2f} s), loop_frames {s["loop_frames"]}, priority {s["priority"]}, interleave {s["interleave"]}, '
          f'weapon {s["weapon"]}, shield {s["shield"]}, max_loops {s["max_loops"]}, precedence {s["precedence"]}, '
          f'priority_moving {s["priority_moving"]}, restart {s["restart"]}, stretch {s["stretch"]}, extra {s["extra"]}, sounds {s["sounds"]}')
    used = sorted({g for f in frames for g in f['groups']})
    print('  frame ids:', [f'{f >> 16}:{f & 65535}' for f in s['frames']])
    for g in used:
        kind = A.KIND.get(base['types'][g], base['types'][g])
        row = []
        for f in frames:
            v = f['groups'].get(g)
            row.append('.' if v is None else ','.join(str(x) for x in v))
        print(f'  g{g:3d} {kind:9s} {group_name(bid, g):22s} ' + ' | '.join(f'{x:>12s}' for x in row))


def export(sid, out, raw=False):
    out = pathlib.Path(out); out.mkdir(parents=True, exist_ok=True)
    for old in out.glob('frame_*.dat'): old.unlink()
    data = A.sequences()[sid]
    s, bid, base, frames = A.sequence_frames(sid)
    enc = data if raw else A.encode_sequence(s)
    (out / 'seq.dat').write_bytes(enc)
    same = int(enc == data); total = 1
    for fid, f in zip(s['frames'], frames):
        orig = A.frame_group(fid >> 16)[fid & 0xffff]
        e = orig if raw else A.encode_frame(dict(base=f['base'], groups=f['groups']), base)   # masks re-derived from the values
        same += e == orig; total += 1
        (out / f'frame_{fid >> 16}_{fid & 0xffff}.dat').write_bytes(e)
    print(f'seq {sid} -> {out}: {total} files written ({"cache bytes copied" if raw else "re-encoded"}); {same} of {total} equal the cache bytes')


def to_json(sid, path):
    s, bid, base, frames = A.sequence_frames(sid)
    d = {k: v for k, v in s.items() if k != 'ops'}
    d['base'] = bid
    d['frame_groups'] = [{str(g): v for g, v in sorted(f['groups'].items())} for f in frames]
    pathlib.Path(path).write_text(json.dumps(d, indent=1))
    print(path)


if __name__ == '__main__':
    cmd = sys.argv[1]
    if cmd == 'dump':
        for x in sys.argv[2:]: dump(int(x))
    elif cmd == 'export':
        export(int(sys.argv[2]), sys.argv[3], '--raw' in sys.argv)
    elif cmd == 'json':
        to_json(int(sys.argv[2]), sys.argv[3])
