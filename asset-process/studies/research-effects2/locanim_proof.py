"""locanim_proof.py: LOC_ADD_CHANGE, LOC_ANIM, a swap and LOC_DEL on a real object by id against the same on loose files.
The Nightmare's spore as the owner's server scripts it (plugins/bosses/nightmare/Spores.Nightmare.js; data spores.locs):
  cycle 4    object 37738 nightmare_spores_initial appears           (by id: loc; files: floc)
  cycle 10   LOC_ANIM 8630 (spawnAnim) on it
  cycle 70   swapped for 37739 nightmare_spores (armed; its idle loops)
  cycle 150  LOC_ANIM 8632 (burstAnim)
  cycle 200  removed
-> render/research-effects2-faithful/locanim-proof.log: differing pixels per frame, 50 frames, close and game size."""
import sys, pathlib, tempfile, shutil
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fx, ob
from fxlib import *
x, y = fx.PLACE
a, _ = ob.dump(37738, HERE / 'real' / 'loc-37738'); b, _ = ob.dump(37739, HERE / 'real' / 'loc-37739')
cyc = list(range(0, 216, 4)); log = []; tot = 0
for close in (True, False):
    out = []
    for lines in ([f'loc 37738 {x} {y} 10 0 4 70 sync', f'locanim {x} {y} 10 0 8630 10', f'loc 37739 {x} {y} 10 0 70 200 sync', f'locanim {x} {y} 10 0 8632 150'],
                  [f'floc {a.as_posix()} {x} {y} 10 0 4 70 sync', f'flocanim {x} {y} seq:8630 10', f'floc {b.as_posix()} {x} {y} 10 0 70 200 sync', f'flocanim {x} {y} seq:8632 150']):
        tmp = pathlib.Path(tempfile.mkdtemp(prefix='la-'))
        fx.run(fx.scene('loc', close, (480, 400)) + lines + [f'render {",".join(map(str, cyc))} {tmp.as_posix()}/f'])
        out.append(fx.frames(f'{tmp.as_posix()}/f', cyc)); shutil.rmtree(tmp, ignore_errors=True)
    per = [fx.diffcount(p, q)[0] for p, q in zip(*out)]
    vis = sum(1 for p, q in zip(out[0], out[0][:1] * len(cyc)) if fx.diffcount(p, q)[0] > 50)
    line = f'{"close" if close else "game size"}: {len(cyc)} frames: {sum(per)} differing of {len(cyc) * 480 * 400} pixels; frames in which the object changes the picture against cycle 0: {vis}; per frame {per}'
    print(line); log.append(line); tot += sum(per)
    if close:
        from PIL import Image
        sh = Image.new('RGB', (240 * 9, 400));
        for k, i in enumerate((2, 6, 12, 17, 22, 30, 38, 44, 51)): sh.paste(out[0][i].resize((240, 200)), (k * 240, 0)); sh.paste(out[1][i].resize((240, 200)), (k * 240, 200))
        sh.save(RENDER / 'research-effects2-faithful' / 'locanim-proof.png')
(RENDER / 'research-effects2-faithful' / 'locanim-proof.log').write_text('\n'.join(log) + '\n')
