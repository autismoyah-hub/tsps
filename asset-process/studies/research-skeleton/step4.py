"""step4.py: pack the creature (model, NPC definition, NEW frame base, frames, sequences) into a private cache copy and
draw it BY ID through the client's ordinary NPC path, against the loose-file pictures.

usage: step4.py ROUND_DIR OUT_CACHE RENDER_DIR
Ids: model 62500, NPC 16360, frame base 2700, sequences and frame groups 20100..20104.
Loose side:  asset_skel.exe SOURCE_CACHE  def:ROUND/sentinel.npcdef anim=ROUND/clips/NAME   (files, loose base)
Packed side: asset_npc.exe OUT_CACHE draw npc:16360 rows=seq:ID:FRAME                          (npc_model::mesh by id,
             Animations::apply_observed by sequence id: the client's own loaders and no file of ours)
Same zoom, yaw, pitch, cell and aim on both; every frame of every sequence, front and back.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, subprocess, pathlib
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE)); sys.path.insert(0, str(HERE.parent / 'research-animation'))
import animlib as A, skelview as S
from PIL import Image
NPC_EXE = _AP.fix(r'<CLIENT_BIN>\asset_npc.exe')
rd, cache, out = pathlib.Path(sys.argv[1]).resolve(), pathlib.Path(sys.argv[2]).resolve(), pathlib.Path(sys.argv[3])
out.mkdir(parents=True, exist_ok=True)
LOG = []
def say(*a):
    s = ' '.join(str(x) for x in a); print(s); LOG.append(s)
CLIPS = {'stand': 20100, 'float': 20101, 'attack': 20102, 'flinch': 20103, 'death': 20104}
d = dict(l.split('=', 1) for l in (rd / 'sentinel.npcdef').read_text().splitlines() if '=' in l and not l.startswith('#'))
sc = [int(x) for x in d['scale'].split(',')]
man = dict(models={'62500': str(rd / 'sentinel.bin')},
           bases={'2700': str(rd / 'clips' / 'stand' / 'base_2700.dat')},
           npcs={'16360': dict(name=d['name'], models=[62500], scale=sc, ambient=int(d.get('ambient', 0)), contrast=int(d.get('contrast', 0)), size=int(d.get('size', 1)),
                               zbuffer=True, stand=CLIPS['stand'], walk=CLIPS['float'], combat=120, ops=[None, 'Attack', None, None, None])},
           sequences={str(v): str(rd / 'clips' / k) for k, v in CLIPS.items()})
(out / 'manifest.json').write_text(json.dumps(man, indent=1))
if not cache.exists():
    r = subprocess.run([sys.executable, '-I', '-B', str(HERE / 'skelpack.py'), str(A.CACHE), str(cache), str(out / 'manifest.json')], capture_output=True, text=True)
    say('$ skelpack.py SOURCE', cache, out / 'manifest.json'); say((r.stdout + r.stderr).strip())
    if r.returncode: raise SystemExit(1)
else:
    say(f'{cache} exists: not packed again')
# the packed files against the loose ones, byte for byte, through animlib's reader
chk = A.Cache(cache)
assert chk.file(1, 2700, 0) == (rd / 'clips' / 'stand' / 'base_2700.dat').read_bytes()
assert chk.file(7, 62500, 0) == (rd / 'sentinel.bin').read_bytes()
n = 2
for k, sid in CLIPS.items():
    assert chk.files(2, 12)[sid] == (rd / 'clips' / k / 'seq.dat').read_bytes(); n += 1
    for p in (rd / 'clips' / k).glob('frame_*.dat'):
        _, g, f = p.stem.split('_'); assert chk.file(0, int(g), int(f)) == p.read_bytes(); n += 1
say(f'read back {n} packed files equal to the loose files (model, base, 5 definitions, frames)')
VIEW = dict(zoom=760, cell=(250, 290), aim=140)
tot = [0, 0, 0, 0]
board = []
for yaw in (192, 1216):
    for k, sid in CLIPS.items():
        loose, text = S.strip(f'def:{rd / "sentinel.npcdef"}', f'anim={rd / "clips" / k}', None, yaw=yaw, **VIEW)
        assert 'base 2700: from' in text
        nf = len(loose)
        bmp = out / '_p.bmp'
        cmd = [NPC_EXE, str(cache), 'draw', str(bmp), 'rows=' + ','.join(f'seq:{sid}:{i}' for i in range(nf)), 'zoom=760', f'yaw={yaw}', 'pitch=0', 'cell=250x290', 'aim=140', 'npc:16360']
        r = subprocess.run(cmd, capture_output=True, text=True)
        if r.returncode: raise SystemExit(' '.join(cmd) + '\n' + r.stdout + r.stderr)
        if yaw == 192: LOG.append('$ ' + ' '.join(cmd)); LOG.append(r.stdout.strip())
        assert 'skeleton 2700' in r.stdout, 'the packed sequence did not use frame base 2700'
        im = Image.open(bmp).convert('RGB'); im.load(); bmp.unlink()
        packed = [im.crop((0, i * 290, 250, (i + 1) * 290)) for i in range(nf)]
        dsum = 0
        for a, b in zip(loose, packed):
            dd = S.diff(a, b); dsum += dd[0]; tot[0] += dd[0]; tot[1] += dd[1]; tot[2] += dd[2]; tot[3] += 1
        say(f'{k} (sequence {sid}) yaw {yaw}: {nf} frames, {dsum} differing pixels between loose files and npc:16360 by id')
        if yaw == 192:
            for i in sorted({0, nf // 3, (2 * nf) // 3, nf - 2}): board.append((k, i, loose[i], packed[i]))
# the stand and walk the DEFINITION names, through the NPC's own animation slots
bmp = out / '_p.bmp'
cmd = [NPC_EXE, str(cache), 'draw', str(bmp), 'rows=bind,stand:5,walk:5', 'zoom=760', 'yaw=192', 'cell=250x290', 'aim=140', 'npc:16360']
r = subprocess.run(cmd, capture_output=True, text=True); say('$ ' + ' '.join(cmd)); say(r.stdout.strip())
im = Image.open(bmp).convert('RGB'); im.load(); bmp.unlink()
for row, (k, fr) in enumerate((('bind', None), ('stand', 5), ('float', 5))):
    loose, _ = S.strip(f'def:{rd / "sentinel.npcdef"}', 'bind' if k == 'bind' else f'anim={rd / "clips" / k}', None if k == 'bind' else [fr], **VIEW)
    dd = S.diff(loose[0], im.crop((0, row * 290, 250, (row + 1) * 290)))
    say(f'definition row {k}: {dd[0]} of {dd[1]} pixels differ ({dd[2]} figure pixels)'); tot[0] += dd[0]; tot[1] += dd[1]; tot[2] += dd[2]; tot[3] += 1
say(f'TOTAL: {tot[3]} cells, {tot[0]} of {tot[1]} pixels differ ({tot[2]} figure pixels)')
sheet = Image.new('RGB', (250 * 8, 290 * 5), S.BG)
from PIL import ImageDraw
dr = ImageDraw.Draw(sheet); font, small = S.fonts()
for n_, (k, i, a, b) in enumerate(board):
    row = list(CLIPS).index(k); colm = [x for x in board if x[0] == k].index((k, i, a, b))
    sheet.paste(a, (colm * 500, row * 290)); sheet.paste(b, (colm * 500 + 250, row * 290))
    dr.text((colm * 500 + 4, row * 290 + 3), f'{k} f{i}: loose files', fill=(230, 225, 190), font=small)
    dr.text((colm * 500 + 254, row * 290 + 3), f'npc:16360 seq {CLIPS[k]} by id', fill=(230, 225, 190), font=small)
sheet.save(out / 'packed-vs-loose.png')
(out / 'step4.log').write_text('\n'.join(LOG) + '\n')
print(out / 'packed-vs-loose.png')
