"""step3a.py: scenery. The furnished observatory with the scribe's desk, drawn twice from the same camera:
  loose  authored_scene_snapshot.exe, SOURCE cache, models and object definitions mounted as a package (today's route)
  by id  live_scene.exe, PACKED cache, the same floor, every object placed by its cache object id (archive 2 group 6,
         models from archive 7); the mounted package holds one unplaced dummy and none of the scene's assets.
Control: live_scene.exe given the loose content and no by-id rows must equal authored_scene_snapshot.exe (same program
steps). Counts differing pixels of 1280 x 900.
usage: step3a.py
"""
import sys, pathlib, shutil
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from livelib import *
OUT = ROOT / 'render' / 'research-live-3a'
if OUT.exists(): shutil.rmtree(OUT)
OUT.mkdir(parents=True)
VIEWS = {'threeq': ('sw', (44.5, 44.5, 14336, 1510, 1190, 65)), 'south': ('s', (44.5, 44.5, 0, 1440, 1280, 65)),
         'game': ('s', (44.5, 44.0, 0, 1024, 896, 60)), 'full': ('', (44.5, 44.5, 6144, 1700, 1450, 65)),
         'desk': ('sw', (45.0, 42.5, 14336, 600, 420, 40))}
empty = OUT / 'no-rows.txt'; empty.write_text('')
cells = []; total = 0
for name, (cut, cam) in VIEWS.items():
    sc = HERE / 'inputs' / f'scene-{cut or "full"}'
    d = OUT / f'{name}-loose-dir'
    run('authored_scene_snapshot', sc / 'loose' / 'content.txt', SRC, *cam, 'classic', 'auto', d)
    loose = OUT / f'{name}-loose.png'; Image.open(d / 'view.bmp').save(loose); shutil.rmtree(d)
    ctl = OUT / f'{name}-control.bmp'
    run('live_scene', sc / 'loose' / 'content.txt', SRC, empty, *cam, ctl)
    byid = OUT / f'{name}-byid.bmp'
    print(run('live_scene', sc / 'byid' / 'content.txt', PACKED, sc / 'byid' / 'placements.txt', *cam, byid).strip())
    c = diff(loose, ctl); r = diff(loose, byid)
    Image.open(byid).save(OUT / f'{name}-byid.png')
    line = f'{name}: control (same loose content, live_scene vs snapshot) {c[0]} differing; BY ID vs LOOSE {r[0]} differing of {r[1]} pixels'
    print(line); LOG.append(line); total += r[0]
    if r[0]: diff_image(loose, byid, OUT / f'{name}-diff.png')
    cells += [(loose, f'{name}: loose files, source cache'), (OUT / f'{name}-byid.png', f'{name}: by object id, packed cache\n{r[0]} differing pixels of {r[1]}')]
    ctl.unlink(); byid.unlink()
board(ROOT / 'render' / 'research-live-board-3a.png', cells, 4, cell=(640, 450),
      title='3a scenery: furnished observatory + scribe\'s desk. Loose-file route | objects by id from the packed cache')
LOG.append(f'TOTAL by-id vs loose: {total} differing pixels over {len(VIEWS)} views')
print(LOG[-1]); save_log(HERE / 'step3a.log')
