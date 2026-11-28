"""The ceiling case, and deliberately degraded copies, for the creature rebuild test.

usage: reencode_npc.py NPC_ID OUTDIR [degradations]
Writes the real NPC's own model files re-encoded through OUR encoder (studies/research-npc/meshio.py -> native_codec)
as OUTDIR/part-N.bin and OUTDIR/NAME.json (the definition: recolours, scale, ambient, contrast, stand, walk, plus the
attack and death the test will use). With no degradation this is the CEILING: it must score as identical.
Refuses a bone-weighted NPC (our encoder writes no weights).

Degradations (any combination; applied to the re-encoded parts; seeded, so repeatable):
  --jitter N          move every vertex position by up to +/-N units on each axis (equal positions move equally)
  --tones Q           round every painted lightness to a multiple of Q (merges neighbouring tones)
  --lightshift K      add K to every painted lightness
  --hueshift K        add K to every hue
  --freeze A,B,..:T   give the vertices labelled A, B, .. the label T (they stop following their own joints)
  --unlabel A,B,..    the same with T = the creature's most common label
  --stretch PCT       scale the mesh about the origin by PCT percent in x and z
  --name NAME         file stem (default npc-ID-ceiling or npc-ID-degraded)
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, json, subprocess, random, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent)); sys.path.insert(1, _AP.fix(r'<ROOT>\studies\research-npc'))
import meshio, cmesh, rebuild_lib as L
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
npc_id = int(a[1]); out = pathlib.Path(a[2]); out.mkdir(parents=True, exist_ok=True)
d = L.npc_def(npc_id); models = L.census('models')
if any(models[m]['weights'] for m in d['models'] if m in models): raise SystemExit('bone-weighted NPC: our encoder writes no weights; no ceiling possible')
raw = out / 'raw'; raw.mkdir(exist_ok=True)
r = subprocess.run([cmesh.EXE, cmesh.CACHE, 'dump', str(npc_id), str(raw)], capture_output=True, text=True)
if r.returncode: raise SystemExit(r.stdout + r.stderr)
kv = dict(l.split('=', 1) for l in (raw / f'npc-{npc_id}.npcdef').read_text().splitlines() if '=' in l and not l.startswith('#'))
parts = [meshio.load(raw / f) for f in kv['models'].split('|')]
degraded = any(x in a for x in ('--jitter', '--tones', '--lightshift', '--hueshift', '--freeze', '--unlabel', '--stretch'))
rng = random.Random(239); notes = []
common = collections.Counter(l for p in parts for l in p['vertex_skins']).most_common(1)[0][0]
if opt('--jitter'):
    n = int(opt('--jitter')); move = {}
    for p in parts:
        nv = []
        for v in p['vertices']:
            if v not in move: move[v] = tuple(c + rng.randint(-n, n) for c in v)
            nv.append(move[v])
        p['vertices'] = nv
    notes.append(f'jitter +/-{n}')
if opt('--stretch'):
    s = int(opt('--stretch')) / 100
    for p in parts: p['vertices'] = [(int(round(x * s)), y, int(round(z * s))) for x, y, z in p['vertices']]
    notes.append(f'stretch {s}')
def recol(fn):
    for p in parts: p['colors'] = [fn(c) if c != 65535 else c for c in p['colors']]
if opt('--tones'):
    q = int(opt('--tones')); recol(lambda c: (c & ~127) | max(2, min(126, int(round((c & 127) / q)) * q))); notes.append(f'tones rounded to {q}')
if opt('--lightshift'):
    k = int(opt('--lightshift')); recol(lambda c: (c & ~127) | max(2, min(126, (c & 127) + k))); notes.append(f'lightness {k:+d}')
if opt('--hueshift'):
    k = int(opt('--hueshift')); recol(lambda c: ((((c >> 10) + k) & 63) << 10) | (c & 1023)); notes.append(f'hue {k:+d}')
for key, target in (('--freeze', None), ('--unlabel', common)):
    if opt(key):
        spec = opt(key); labs, _, t = spec.partition(':'); t = int(t) if t else target; labs = {int(x) for x in labs.split(',')}; n = 0
        for p in parts:
            for i, l in enumerate(p['vertex_skins']):
                if l in labs: p['vertex_skins'][i] = t; n += 1
        notes.append(f'labels {sorted(labs)} -> {t} ({n} vertices)')
name = opt('--name', f'npc-{npc_id}-' + ('degraded' if degraded else 'ceiling'))
files = []
for i, p in enumerate(parts):
    f = f'{name}-part-{i}.bin'; meshio.save(p, out / f); files.append(f)
seqs = L.sequences(npc_id)
j = dict(name=d['name'] + (' (degraded: ' + '; '.join(notes) + ')' if degraded else ' (re-encoded)'), models=files,
         recolor=[[int(x) for x in pair.split(':')] for pair in kv.get('recolor', '').split(',') if pair],
         scale=[int(x) for x in kv['scale'].split(',')], ambient=int(kv['ambient']), contrast=int(kv['contrast']), size=int(kv['size']), zbuffer=kv['zbuffer'] == '1', **seqs)
(out / f'{name}.json').write_text(json.dumps(j, indent=1))
print(out / f'{name}.json', '|', '; '.join(notes) or 'ceiling', '|', len(parts), 'parts')
