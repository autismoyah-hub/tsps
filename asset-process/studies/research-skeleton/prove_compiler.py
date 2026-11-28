"""prove_compiler.py: the proof that skelc.py compiles real skeletons.

usage: prove_compiler.py BASE NPC OUTDIR [--seqs A,B,..]
For the real frame base BASE and a real NPC on it, with every classic sequence on the base (or --seqs):
  0. control: the NPC's own model files re-fed as files (def:) against npc:ID, bind pose.
  A. EXACT. describe(BASE) -> description -> compile. The compiled base must have the same group types and label
     sets in the same order. The real frames (re-encoded by animlib) are written loose beside the COMPILED base and
     drawn; compared with the cache drawing pixel for pixel.
  B. RE-AUTHORED. describe(BASE, parent-first) with a NEW base id and every vertex label renumbered (a random
     permutation, seed 1), compiled; the real frames re-targeted through the joint / channel map (group numbers change);
     the real model files relabelled with the permutation; drawn and compared.
Writes OUTDIR/prove.log and, when B differs, OUTDIR/diff-*.png.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, random, subprocess, collections
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE)); sys.path.insert(0, str(HERE.parent / 'research-animation')); sys.path.insert(0, str(HERE.parent / 'research-npc'))
import animlib as A, skelc, skelread, skelview, meshio
from PIL import Image
NPC_EXE = _AP.fix(r'<CLIENT_BIN>\asset_npc.exe')
argv = sys.argv
bid, npc, out = int(argv[1]), int(argv[2]), pathlib.Path(argv[3])
out.mkdir(parents=True, exist_ok=True)
LOG = []
def say(*a):
    s = ' '.join(str(x) for x in a); print(s); LOG.append(s)

seqs = [int(x) for x in argv[argv.index('--seqs') + 1].split(',')] if '--seqs' in argv else skelread.seqs_on(bid)
VIEW = dict(zoom=int(argv[argv.index('--zoom') + 1]) if '--zoom' in argv else 700, cell=(240, 240))
say(f'base {bid}, npc {npc}, {len(seqs)} sequences; view {VIEW}')

# ---- 0. the NPC as files
real_dir = out / 'real'
r = subprocess.run([NPC_EXE, str(A.CACHE), 'dump', str(npc), str(real_dir)], capture_output=True, text=True)
if r.returncode: raise SystemExit(r.stdout + r.stderr)
defpath = real_dir / f'npc-{npc}.npcdef'
a, _ = skelview.strip(f'npc:{npc}', 'bind', **VIEW); b, _ = skelview.strip(f'def:{defpath}', 'bind', **VIEW)
d = skelview.diff(a[0], b[0]); say(f'0. control, own files against npc:{npc}, bind: {d[0]} of {d[1]} pixels differ ({d[2]} figure pixels)')

# ---- A. exact
desc, base, blocks, order, info = skelc.describe(bid)
(out / f'desc-{bid}-exact.json').write_text(json.dumps(desc, indent=1))
cbase, meta, warnings = skelc.compile_desc(desc)
same_types = cbase['types'] == base['types']
same_sets = all(set(x) == set(y) and len(x) == len(y) for x, y in zip(cbase['labels'], base['labels']))
same_bytes = A.encode_base(dict(cbase, tail=base['tail'])) == A.encode_base(base)
say(f'A. describe -> {len(desc["joints"])} joints; {info}; compile -> {len(cbase["types"])} groups; types equal: {same_types}; label sets equal in every group: {same_sets}; '
    f'bytes equal (label order inside lists included): {same_bytes}; {len(warnings)} order warnings')
for w in warnings[:6]: say('   ', w)
tot = [0, 0, 0, 0]; worst = (0, None)
dirA = out / 'exact'
frames_total = 0
loaded = {}
for sid in seqs:
    try: s, b_, _, frames = A.sequence_frames(sid)
    except Exception as e:
        say(f'   seq {sid}: not decodable ({e}); skipped'); continue
    if b_ != bid: continue
    loaded[sid] = (s, frames)
    dA = dirA / str(sid); dA.mkdir(parents=True, exist_ok=True)
    (dA / 'seq.dat').write_bytes(A.encode_sequence(s))
    for fid, f in zip(s['frames'], frames):
        (dA / f'frame_{fid >> 16}_{fid & 65535}.dat').write_bytes(A.encode_frame(dict(base=bid, groups=f['groups']), base))
    (dA / f'base_{bid}.dat').write_bytes(A.encode_base(dict(cbase, tail=base['tail'])))
    ref, _ = skelview.strip(f'npc:{npc}', f'seq={sid}', **VIEW)
    got, text = skelview.strip(f'npc:{npc}', f'anim={dA}', **VIEW)
    assert f'base {bid}: from' in text, 'the loose base was not used'
    for k, (x, y) in enumerate(zip(ref, got)):
        d = skelview.diff(x, y); tot[0] += d[0]; tot[1] += d[1]; tot[2] += d[2]; tot[3] += 1
        if d[0] > worst[0]: worst = (d[0], (sid, k))
    loaded[sid] += (ref,)
say(f'A. {len(loaded)} sequences, {tot[3]} frames drawn from the cache and from loose frames on the COMPILED base: {tot[0]} of {tot[1]} pixels differ ({tot[2]} figure pixels); worst cell {worst}')

# ---- B. re-authored: new id, labels permuted, joints re-ordered
real_issues = skelc.order_check(base)
say(f'   order check on the REAL base as stored: {len(real_issues)} findings ' + str(dict(collections.Counter(k for _, k, _ in real_issues))))
rng = random.Random(1)
perm = list(range(256)); rng.shuffle(perm)
P = lambda l: perm[l]
# the rest mesh, for the vertex-level comparison with the integer port
mesh = out / 'rest.txt'; tmp = out / '_d.bmp'
subprocess.run([skelview.EXE, str(A.CACHE), str(tmp), f'dump={mesh}', 'cell=40x40', f'npc:{npc}'], capture_output=True, text=True); tmp.unlink()
V, L = [], []
for line in mesh.read_text().splitlines():
    p_ = line.split(); V.append([int(p_[0]), int(p_[1]), int(p_[2])]); L.append(int(p_[3]))
L2 = [P(l) for l in L]
# relabelled model files, and a plain re-encode as the control
lab_dir = out / 'relabelled'; lab_dir.mkdir(exist_ok=True); enc_dir = out / 'reencoded'; enc_dir.mkdir(exist_ok=True)
for line in defpath.read_text().splitlines():
    if line.startswith('models='):
        for n in line[7:].split('|'):
            m = meshio.load(real_dir / n); meshio.save(m, enc_dir / n)
            m['vertex_skins'] = [P(l) for l in m['vertex_skins']]; meshio.save(m, lab_dir / n)
for d_ in (lab_dir, enc_dir): (d_ / defpath.name).write_text(defpath.read_text())
def2 = lab_dir / defpath.name
a, _ = skelview.strip(f'npc:{npc}', 'bind', **VIEW); b, _ = skelview.strip(f'def:{enc_dir / defpath.name}', 'bind', **VIEW)
d = skelview.diff(a[0], b[0]); say(f'B. control, model files re-encoded by meshio (labels untouched), bind: {d[0]} of {d[1]} pixels differ')
for variant, NEW, FG in (('same-order', 2700, 30000), ('child-first', 2701, 30001), ('parent-first', 2702, 30002)):
    desc2, _, _, order2, info2 = skelc.describe(bid, parent_first=variant == 'parent-first', child_first=variant == 'child-first')
    for j in desc2['joints']:
        j['pivot'] = [P(l) for l in j['pivot']]; j['own'] = [P(l) for l in j['own']]
        for c in j['channels']:
            if isinstance(c, dict):
                for k in ('add', 'drop'):
                    if k in c: c[k] = [P(l) for l in c[k]]
    desc2['base_id'] = NEW; desc2['name'] = f'base {bid} re-authored {variant}'
    (out / f'desc-{bid}-{variant}.json').write_text(json.dumps(desc2, indent=1))
    base2, meta2, warn2 = skelc.compile_desc(desc2)
    gmap = {}
    for b_i in order2:
        B = blocks[b_i]; e = meta2['joints'][f'j{B["origin"]}']
        gmap[B['origin']] = e['origin']
        for g, (cname, c) in zip(B['groups'], e['channels'].items()): gmap[g] = c['group']
    assert len(gmap) == len(base['types']) and sorted(gmap.values()) == list(range(len(base['types'])))
    for g, n in gmap.items():
        assert base['types'][g] == base2['types'][n]
        if base['types'][g] != 5: assert {P(l) for l in base['labels'][g]} == set(base2['labels'][n])
    moved = sum(1 for g, n in gmap.items() if g != n)
    issues = skelc.order_check(base2)
    say(f'B/{variant}: {len(desc2["joints"])} joints, written order {meta2["order"]}, {moved} of {len(gmap)} groups change number, base id {NEW}, every vertex label renumbered; '
        f'order check: {len(issues)} findings {dict(collections.Counter(k for _, k, _ in issues))}')
    tot = [0, 0, 0, 0]; cells_diff = 0; worst = (0, None); per = {}
    dirB = out / variant
    for sid, (s, frames, ref) in loaded.items():
        dB = dirB / str(sid); dB.mkdir(parents=True, exist_ok=True)
        s2 = dict(s); s2.pop('ops', None)
        ids = []
        for k, f in enumerate(frames):
            g2 = {gmap[g]: v for g, v in f['groups'].items()}
            (dB / f'frame_{FG}_{k + 1}.dat').write_bytes(A.encode_frame(dict(base=NEW, groups=g2), base2))
            ids.append((FG << 16) | (k + 1))
        s2['frames'] = ids
        if s2.get('interleave'): s2['interleave'] = sorted(gmap[g] for g in s2['interleave'])
        (dB / 'seq.dat').write_bytes(A.encode_sequence(s2))
        (dB / f'base_{NEW}.dat').write_bytes(A.encode_base(base2))
        got, text = skelview.strip(f'def:{def2}', f'anim={dB}', **VIEW)
        assert f'base {NEW}: from' in text
        n = 0
        for k, (x, y) in enumerate(zip(ref, got)):
            d = skelview.diff(x, y); tot[0] += d[0]; tot[1] += d[1]; tot[2] += d[2]; tot[3] += 1; n += d[0]
            cells_diff += d[0] > 0
            if d[0] > worst[0]:
                worst = (d[0], (sid, k)); w = Image.new('RGB', (x.width * 2, x.height)); w.paste(x, (0, 0)); w.paste(y, (x.width, 0)); w.save(out / f'diff-worst-{variant}.png')
        per[sid] = n
    say(f'B/{variant}: {len(loaded)} sequences, {tot[3]} frames, relabelled model files + re-targeted frames against the cache: {tot[0]} of {tot[1]} pixels differ '
        f'({tot[2]} figure pixels) in {cells_diff} cells; worst cell {worst}')
    nv = dv = 0; mx = 0; hist = collections.Counter()
    for sid, (s, frames, ref) in loaded.items():
        for f in frames:
            a_ = A.apply_frame([list(v) for v in V], L, f, base)
            b_ = A.apply_frame([list(v) for v in V], L2, dict(base=NEW, groups={gmap[g]: v for g, v in f['groups'].items()}), base2)
            for x, y in zip(a_, b_):
                e = max(abs(x[i] - y[i]) for i in range(3)); nv += 1; dv += e > 0; mx = max(mx, e); hist[min(e, 5)] += 1
    say(f'B/{variant}: vertices (integer port of deform.rs): {dv} of {nv} posed vertices differ from the real base\'s result; largest difference {mx} units; '
        f'by size in units {dict(sorted(hist.items()))} (5 = 5 or more)')
(out / 'prove.log').write_text('\n'.join(LOG) + '\n')
