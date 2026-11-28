"""A creature's frame base, read for an animation author.

usage: creature.py NPC_ID [SEQ ...]
Prints the NPC's stand and walk sequences (from its cache definition, through asset_anim.exe), the frame base they
use, how many classic sequences use that base, and for every group that the listed sequences (default: stand and
walk, plus the server's attack/block/death for the NPC when its table has them) actually mention: its type, how
many of the NPC's vertices it moves, where those vertices are (centre and box, x / height / z in model space, front
is negative z), and whether the sequences move it. That is enough to name the joints.
Writes mesh/npc-ID.txt (the rest mesh with labels) for reach.py-style tools.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib, json, re, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import animlib as A, author, measure
EXE = _AP.fix(r'<CLIENT_BIN>\asset_anim.exe')
HERE = pathlib.Path(__file__).parent
SERVER = pathlib.Path(_AP.fix(r'<TSPS_CHECKOUT>\server\data\definitions'))
npc = int(sys.argv[1])
mesh = HERE / 'mesh' / f'npc-{npc}.txt'
tmp = HERE / '_c.bmp'
r = subprocess.run([EXE, str(A.CACHE), str(tmp), f'dump={mesh}', f'npc:{npc}'], capture_output=True, text=True)
if r.returncode: raise SystemExit(r.stdout + r.stderr)
tmp.unlink()
line = next(l for l in r.stdout.splitlines() if l.startswith(f'npc:{npc}:'))
print(line[:330])
anims = [int(x) for x in re.search(r'animations \[([^\]]*)\]', line).group(1).split(',')]
seqs = [int(x) for x in sys.argv[2:]] or [a for a in (anims[0], anims[3]) if a >= 0]
table = json.loads((SERVER / 'npc-animations.json').read_text(encoding='utf-8')).get(str(npc))
if table and not sys.argv[2:]:
    print(f'server npc-animations.json for {npc}: {table} (attack, block, death)')
    seqs += [a for a in table if a >= 0 and a not in seqs]
verts, labels = author.load_mesh(mesh)
by = collections.defaultdict(list)
for i, l in enumerate(labels): by[l].append(i)
info = {}
bid = None
for sid in seqs:
    s, b, base, frames = A.sequence_frames(sid)
    bid = b if bid is None else bid
    print(f'  seq {sid} {measure.NAMES.get(str(sid), "?")}: base {b}, {len(frames)} frames, lengths {s["lengths"]}, total {sum(s["lengths"])} cycles, priority {s["priority"]}, '
          f'interleave {s["interleave"]}, loop_frames {s["loop_frames"]}')
    if b != bid: continue
    for g in sorted({g for f in frames for g in f["groups"]}):
        d = [128] * 3 if base['types'][g] == 3 else [0] * 3
        vals = [f['groups'].get(g, d) for f in frames]
        e = info.setdefault(g, dict(moved=set(), const={}))
        if any(v != vals[0] for v in vals): e['moved'].add(sid)
        else: e['const'][sid] = vals[0]
base = A.frame_base(bid)
users = sum(1 for sid, d in A.sequences().items() if measure.base_of(A.decode_sequence(d)) == bid)
kinds = collections.Counter(A.KIND.get(t, t) for t in base['types'])
print(f'frame base {bid}: {len(base["types"])} groups {dict(kinds)}; {users} classic sequences use it; the mesh has {len(verts)} vertices, {len(by)} labels')
for g in sorted(info):
    idx = [i for l in base['labels'][g] for i in by.get(l, ())]
    kind = A.KIND.get(base['types'][g], base['types'][g])
    where = 'moves no vertex of this mesh'
    if idx:
        c = [sum(verts[i][a] for i in idx) / len(idx) for a in range(3)]
        lo = [min(verts[i][a] for i in idx) for a in range(3)]; hi = [max(verts[i][a] for i in idx) for a in range(3)]
        where = f'{len(idx):3d} vertices, centre x {c[0]:5.0f} h {-c[1]:4.0f} z {c[2]:5.0f}; box x {lo[0]}..{hi[0]} h {-hi[1]}..{-lo[1]} z {lo[2]}..{hi[2]}'
    e = info[g]
    print(f'  g{g:3d} {kind:9s} labels {str(base["labels"][g])[:44]:44s} {where}; moved by {sorted(e["moved"]) or "-"}' + (f'; constant {list(e["const"].items())[:2]}' if e['const'] and kind == 'origin' else ''))
