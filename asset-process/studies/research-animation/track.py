"""Numbers for an author: where the hands and the weapon are in every frame, posed by the client (asset_anim track=).

usage: track.py SOURCE ENTRY [--under SEQ:FRAME]
   SOURCE = seq=ID | anim=DIR        ENTRY as for apose.py (use the weapon the animation is for)
Prints per frame, in worn space (x: negative = the character's right; h: height above the feet; z: negative = front):
   R hand (label 27) and L hand (label 28) centroids, the weapon's centroid (label 50),
   the direction from the R hand to the weapon centroid as a unit vector and as words,
   how far the L hand is from the weapon's long axis (for two-handed grips; the axis is fitted to the weapon's
   posed vertices), and the R hand's travel since the previous frame in units and units per client cycle.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib, math, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import animlib as A
EXE = _AP.fix(r'<CLIENT_BIN>\asset_anim.exe')


def run(source, entry, under=None, labels='27,28,50,51'):
    if source.startswith('seq='): s = A.sequence(int(source[4:]))
    else: s = A.decode_sequence((pathlib.Path(source[5:]) / 'seq.dat').read_bytes())
    n = len(s['frames'])
    out = pathlib.Path(__file__).parent / '_track.bmp'
    args = [EXE, str(A.CACHE), str(out), source, 'frames=' + ','.join(map(str, range(n))), f'track={labels}', entry]
    if under: args.append(f'under={under}')
    r = subprocess.run(args, capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    out.unlink()
    pos = collections.defaultdict(dict)
    for line in r.stdout.splitlines():
        if line.startswith('track '):
            p = line.split()
            pos[int(p[4])][int(p[6])] = (float(p[10]), float(p[12]), float(p[14]))
    return s, pos


def words(d):
    out = []
    if abs(d[1]) > 0.35: out.append('up' if d[1] > 0 else 'down')
    if abs(d[2]) > 0.35: out.append('front' if d[2] < 0 else 'back')
    if abs(d[0]) > 0.35: out.append('right' if d[0] < 0 else 'left')
    return '+'.join(out)


if __name__ == '__main__':
    under = sys.argv[sys.argv.index('--under') + 1] if '--under' in sys.argv else None
    s, pos = run(sys.argv[1], sys.argv[2], under)
    L = s['lengths']; prev = None
    for f in range(len(L)):
        p = pos[f]
        rh, lh, w = p.get(27), p.get(28), p.get(50)
        line = f'f{f:2d} {L[f]:2d}c  Rhand {rh[0]:6.1f} {rh[1]:6.1f} {rh[2]:6.1f}  Lhand {lh[0]:6.1f} {lh[1]:6.1f} {lh[2]:6.1f}'
        if w:
            d = [w[i] - rh[i] for i in range(3)]; n = math.sqrt(sum(x * x for x in d)) or 1
            d = [x / n for x in d]
            # distance of the left hand from the line through the right hand along d
            v = [lh[i] - rh[i] for i in range(3)]; along = sum(v[i] * d[i] for i in range(3))
            off = math.sqrt(max(0.0, sum(x * x for x in v) - along * along))
            line += f'  weapon dir {d[0]:5.2f} {d[1]:5.2f} {d[2]:5.2f} ({words(d)})  Lhand off shaft {off:5.1f} at {along:6.1f} along'
        if prev:
            st = math.dist(rh, prev)
            line += f'  Rhand moved {st:5.1f} = {st / L[f - 1]:4.1f}/cycle'
        prev = rh
        print(line)
