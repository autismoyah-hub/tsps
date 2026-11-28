"""Two keyframe files (or a file and a real sequence) on the same creature, drawn frame against frame.
usage: rig_pair.py WORKDIR A B [--frames all|every:K] [--model DEF.npcdef] [--base FILE] [--yaw 192]
A, B = a keyframe file path, or seq:ID. Prints frames, figure pixels, differing pixels, cells that differ, distinct poses of each."""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import riglib as R
L, B = R.L, R.B
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); entry = f"def:{pathlib.Path(opt('--model')).resolve()}" if opt('--model') else f"npc:{man['npc']}"
cs = 300; M = L.merged(entry, work / 'pair.tmp.txt'); (work / 'pair.tmp.txt').unlink(); zoom = int(1.6 * max(M['V'].max(0) - M['V'].min(0)))
def length(x): return next(s['frames'] for s in man['sequences'] if s['seq'] == int(x[4:])) if x.startswith('seq:') else B.decode_anim(pathlib.Path(x).read_bytes())['range'][1]
n = min(length(a[2]), length(a[3]))
def draw(x):
    o = dict(frames=opt('--frames', 'all'), cols=20, zoom=zoom, yaw=int(opt('--yaw', 192)), pitch=128, cell=f'{cs}x{cs}', count=n)
    if x.startswith('seq:'): return L.grid(f"npc:{man['npc']}", seq=int(x[4:]), **o)
    return L.grid(entry, anim=pathlib.Path(x).resolve(), base=opt('--base'), **o)
A, ha, _ = draw(a[2]); Bm, hb, _ = draw(a[3]); d = L.diff_cells(A, Bm, cs, cs)[:len(ha)]
print(f"{a[2]} against {a[3]}: {len(ha)} frames ({opt('--frames', 'all')}), {sum(x[1] for x in d)} figure pixels, {sum(x[0] for x in d)} differing in {sum(1 for x in d if x[0])} cells (worst cell {max(x[0] for x in d)}); distinct poses {len(set(ha.values()))} against {len(set(hb.values()))}")
