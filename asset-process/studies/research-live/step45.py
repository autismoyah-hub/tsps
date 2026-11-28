"""step45.py: steps 4 and 5 on the packed cache.
 5a  the map route against the loose route, pixel by pixel: square 4378 (the room on a floor of the scene tool's colour)
     drawn by Scene::load_normal + WorldMesh::build (live_world.exe) against authored_scene_snapshot.exe on loose files.
 5b  square 4374 (grass, real trees, drawers and chest beside our room) drawn by the client binary's own
     `scene-render` command and by live_world.exe.
 4a  the world frame without a server (live_world.exe): on square 4374, and at a REAL location (Lumbridge), one of
     our objects spawned by id, the warden and the hound by NPC id, players wearing the items by item id through the
     client's appearance-block decoder, standing and in the new sequences.
 4b  the real client window in its offline cache viewport (`gielinor-client scene-view`), captured with PrintWindow.
 3a+ the same scenery added AFTER load (as a server object spawn) against the same scenery in the map as loaded.
usage: step45.py
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, shutil, subprocess
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from livelib import *
G = ROOT / 'client-build' / 'rust-client' / 'target' / 'release' / 'gielinor-client.exe'
O5 = ROOT / 'render' / 'research-live-5'; O4 = ROOT / 'render' / 'research-live-4'
for o in (O5, O4):
    if o.exists(): shutil.rmtree(o)
    o.mkdir(parents=True)
png = lambda p: (Image.open(p).save(p.with_suffix('.png')), p.unlink(), p.with_suffix('.png'))[2]
def note(s): print(s); LOG.append(s)

# ---- 5a
cells5 = []
sc = HERE / 'inputs' / 'scene-full' / 'loose' / 'content.txt'
for name, cam in (('full', (6144, 1700, 1450, 65)), ('near', (0, 1440, 1280, 65)), ('threeq', (14336, 1510, 1190, 65)), ('low', (3000, 1100, 500, 80))):
    d = O5 / f'{name}-dir'
    run('authored_scene_snapshot', sc, SRC, 44.5, 44.5, *cam, 'classic', 'auto', d)
    loose = O5 / f'proof-{name}-loose.png'; Image.open(d / 'view.bmp').save(loose); shutil.rmtree(d)
    m = O5 / f'proof-{name}-map.bmp'
    run('live_world', PACKED, m, 1124.5, 1700.5, *cam); m = png(m)
    r = diff(loose, m)
    note(f'5a {name}: map square 4378 through the map loader vs loose scene: {r[0]} differing of {r[1]} pixels')
    if r[0]: diff_image(loose, m, O5 / f'proof-{name}-diff.png')
    if name in ('full', 'threeq'):
        cells5 += [(loose, f'{name}: loose files, scene tool'), (m, f'{name}: map square 4378, client map loader\n{r[0]} differing of {r[1]}')]

# ---- 5b
sr = O5 / 'scene-render-4374.bmp'
out = subprocess.run([str(G), 'scene-render', PACKED, '140', '180', '0', str(sr)], capture_output=True, text=True)
LOG.append(f'$ {G} scene-render {PACKED} 140 180 0 {sr}'); note('5b ' + (out.stdout + out.stderr).strip()); sr = png(sr)
ACT = ['npc=16350:1122:1436:0', 'npc=16351:1126:1435:512', 'player=1124:1437:0:0:60003,60005,60004,60006,60007,60001:-1:0:809:1146',
       'player=1127:1437:1:0:60003,60005,60004,60006,60007,60000', 'npc=16350:1120:1437:0:440:5', 'player=1129:1437:0:0:60003,60005,60004,60006,60007,60002:20003:5']
w1 = O5 / 'square-4374-wide.bmp'; note('5b ' + run('live_world', PACKED, w1, 1124.5, 1442, 0, 2300, 1700, 60, *ACT).strip().splitlines()[-1]); w1 = png(w1)
w2 = O5 / 'square-4374-close.bmp'; run('live_world', PACKED, w2, 1124.5, 1436.5, 0, 900, 420, 90, *ACT); w2 = png(w2)
cells5 += [(sr, 'square 4374 by the client binary: gielinor-client scene-render 140 180 0'), (w1, 'square 4374, live_world: real trees, drawers, chest; warden, hound, players by id')]
board(ROOT / 'render' / 'research-live-board-5.png', cells5, 2, cell=(900, 640), title='5 map placement: the observatory as a real map square in the packed cache, drawn by the client\'s own map loader')

# ---- 4a: a real location. Lumbridge, in front of the castle; nothing here is in the map: all of it is "server state"
L = ['object=63046:3222:3222:10:0', 'npc=16350:3220:3219:0', 'npc=16351:3225:3220:512', 'player=3222:3219:0:0:60003,60005,60004,60006,60007,60001:-1:0:809:1146',
     'player=3224:3218:1:0:60003,60005,60004,60006,60007,60000', 'player=3221:3217:0:0:1127,1079,1163,1305', 'player=3226:3218:0:0:60003,60005,60004,60006,60007,60000:20001:5',
     'player=3218:3218:0:0:60003,60005,60004,60006,60007,60001:20002:6:809:1146']
import json
desk = [k for k, v in json.loads((HERE / 'inputs' / 'ids.json').read_text())['objects'].items() if 'desk.bin' in v][0]
L[0] = f'object={desk}:3222:3222:10:0'
a = O4 / 'lumbridge-wide.bmp'; o = run('live_world', PACKED, a, 3222.5, 3219.5, 0, 1500, 900, 60, *L); note('4a ' + o.strip().replace('\n', '\n4a ')); a = png(a)
b = O4 / 'lumbridge-close.bmp'; run('live_world', PACKED, b, 3222.5, 3219.5, 0, 800, 380, 90, *L); b = png(b)
c = O4 / 'lumbridge-bare.bmp'; run('live_world', PACKED, c, 3222.5, 3219.5, 0, 800, 380, 90); c = png(c)
r = diff(c, b); note(f'4a Lumbridge close view: {r[0]} of {r[1]} pixels changed by our object, NPCs and players (same camera, with and without them)')
# the same real location from the SOURCE cache: the packed copy must not change what is real
s = O4 / 'lumbridge-bare-source.bmp'; run('live_world', SRC, s, 3222.5, 3219.5, 0, 800, 380, 90); s = png(s)
r = diff(s, c); note(f'4a regression: Lumbridge with nothing added, source cache vs packed cache: {r[0]} differing of {r[1]} pixels')

# ---- 4b: the real window
ps = _AP.fix(r'<POWERSHELL>')
win = O4 / 'client-window-scene-view.png'
o = subprocess.run([ps, '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', str(HERE / 'capture_window.ps1'), str(G), PACKED, '140', '180', str(win)], capture_output=True, text=True)
LOG.append(f'$ powershell -File capture_window.ps1 {G} {PACKED} 140 180 {win}'); note('4b ' + (o.stdout + o.stderr).strip().encode('ascii', 'replace').decode())
o = subprocess.run([str(G), 'scene-view', PACKED, '140', '180', '0', '--smoke'], capture_output=True, text=True)
LOG.append(f'$ {G} scene-view {PACKED} 140 180 0 --smoke'); note('4b ' + (o.stdout + o.stderr).strip())
cells4 = [(a, 'Lumbridge (real map), live_world: our desk spawned by object id, warden 16350, hound 16351,\nplayers wearing items 60000..60007 by id; a player in real rune (items 1127, 1079, 1163, 1305) among them'),
          (b, 'the same, close'), (w2, 'test square 4374, close: warden stand and attack 440, players stand / salute 20003'),
          (win, 'THE CLIENT WINDOW (gielinor-client scene-view, offline cache viewport) on the packed cache,\ncaptured with PrintWindow')]
board(ROOT / 'render' / 'research-live-board-4.png', cells4, 2, cell=(900, 640), title='4 the world without a server: client map loader + world-actor path fed by hand; and the real client window (offline viewport)')

# ---- 3a+: runtime spawn against map-loaded
O3 = ROOT / 'render' / 'research-live-3a'
scs = HERE / 'inputs' / 'scene-sw' / 'byid'
cam = (44.5, 44.5, 14336, 1510, 1190, 65)
x = O3 / 'threeq-asloaded.bmp'; y = O3 / 'threeq-runtime.bmp'
run('live_scene', scs / 'content.txt', PACKED, scs / 'placements.txt', *cam, x); run('live_scene', scs / 'content.txt', PACKED, scs / 'placements.txt', *cam, y, 'runtime')
x = png(x); y = png(y); r = diff(x, y); diff_image(x, y, O3 / 'threeq-runtime-diff.png')
note(f'3a+ the same objects added after the scene is loaded (server spawn) vs in the scene as loaded: {r[0]} differing of {r[1]} pixels (ground shading only; see threeq-runtime-diff.png)')
save_log(HERE / 'step45.log')
