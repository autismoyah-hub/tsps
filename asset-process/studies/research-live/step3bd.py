"""step3bd.py: (b) a player wearing the complete Meridian items BY ITEM ID, male and female, with each weapon, in the
rest pose, the stand and the walk; (d) the new sequences BY SEQUENCE ID on that player.
  by id  research_pose.exe / asset_pose.exe on the PACKED cache with item:ID entries and seq=ID: the client's
         item::Definitions::load -> item_model::Models::wearable (gender) -> Appearance::mesh -> Animations::apply_actor.
  loose  the same programs on the SOURCE cache with model:FILE@SLOT+HIDE entries (and asset_anim.exe anim=DIR for the
         new sequences): the route every figure picture has used so far.
Every pair of sheets is compared pixel by pixel.
usage: step3bd.py
"""
import sys, pathlib, shutil, json
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from livelib import *
OUT = ROOT / 'render' / 'research-live-3b'
if OUT.exists(): shutil.rmtree(OUT)
OUT.mkdir(parents=True)
man = json.loads((HERE / 'inputs' / 'manifest.json').read_text())
ITEMS = {'longsword': 60000, 'glaive': 60001, 'staff': 60002, 'cuirass': 60003, 'gauntlets': 60004, 'legs': 60005, 'boots': 60006, 'barbute': 60007}
SLOT = {0: 'head', 3: 'weapon', 4: 'body', 5: 'shield', 6: 'arms', 7: 'legs', 8: 'hair', 9: 'hands', 10: 'feet'}


def loose_piece(name, g):
    spec = man['items'][str(ITEMS[name])]
    path = man['models'][str(spec['wear_models'][g][0])]
    assert spec.get('wear_offset', [0, 0])[g] == 0
    pos = [p for p in spec['wear_positions'] if p != -1]
    return f'model:{path}@' + '+'.join(SLOT[p] for p in pos)


def outfits(weapon, g):
    order = ['cuirass', 'legs', 'gauntlets', 'boots', 'barbute', weapon]
    return '+'.join(f'item:{ITEMS[n]}' for n in order), '+'.join(loose_piece(n, g) for n in order)


total = 0; cells_b = []; cells_d = []; npairs = 0
def pair(tag, by_id_cmd, loose_cmd, cells, caption):
    global total, npairs
    a = OUT / f'{tag}-byid.bmp'; b = OUT / f'{tag}-loose.bmp'
    run(by_id_cmd[0], PACKED, a, *by_id_cmd[1:]); run(loose_cmd[0], SRC, b, *loose_cmd[1:])
    d = diff(b, a); total += d[0]; npairs += 1
    line = f'{tag}: by id vs loose {d[0]} differing of {d[1]} pixels ({d[2]} drawn)'
    print(line); LOG.append(line)
    pa = OUT / f'{tag}-byid.png'; pb = OUT / f'{tag}-loose.png'
    Image.open(a).save(pa); Image.open(b).save(pb); a.unlink(); b.unlink()
    if d[0]: diff_image(pb, pa, OUT / f'{tag}-diff.png')
    cells += [(pb, f'{caption}: loose files'), (pa, f'{caption}: BY ID, packed cache\n{d[0]} differing of {d[1]}')]
    return d

# ---- (b) rest pose, stand, walk; both genders; three weapons; two camera yaws
for g in (0, 1):
    for weapon, stand, walk in (('longsword', 808, 819), ('glaive', 809, 1146), ('staff', 808, 819)):
        by_id, loose = outfits(weapon, g)
        for row, seq in (('bind', []), ('stand', [f'seq={stand}', 'frames=0,4']), ('walk', [f'seq={walk}', 'frames=0,2,4,6'])):
            for yaw in (0, 600):
                common = [f'gender={g}', *seq, 'draw=actor', 'cell=220x270', 'zoom=560'] + ([f'yaw={yaw}'] if yaw else [])
                tag = f'b-{"mf"[g]}-{weapon}-{row}-yaw{yaw}'
                keep = cells_b if yaw == 0 and row != 'bind' else []
                pair(tag, ['research_pose', *common, by_id], ['research_pose', *common, loose], keep, f'{"male female".split()[g]} {weapon} {row}')
board(ROOT / 'render' / 'research-live-board-3b.png', cells_b, 4, cell=(520, 300),
      title='3b player wearing Meridian items: loose model files | BY ITEM ID from the packed cache (cuirass, legs, gauntlets, boots, barbute + weapon)')
tb = total
LOG.append(f'TOTAL 3b: {tb} differing pixels over {npairs} sheet pairs'); print(LOG[-1])

# ---- (d) the new sequences by id on a player (male: asset_anim has no gender switch), every playback frame
import importlib
sys.path.insert(0, str(HERE.parent / 'research-animation'))
import animlib as A
nb = npairs
for sid, weapon in ((20001, 'longsword'), (20002, 'glaive'), (20003, 'staff')):
    d = pathlib.Path(man['sequences'][str(sid)])
    n = len(A.decode_sequence((d / 'seq.dat').read_bytes())['frames'])
    by_id, loose = outfits(weapon, 0)
    for part, frames in enumerate([list(range(n))[i:i + 8] for i in range(0, n, 8)]):
        fr = 'frames=' + ','.join(map(str, frames))
        common = [fr, 'draw=actor', 'cell=220x270', 'zoom=560']
        pair(f'd-seq{sid}-{weapon}-part{part}', ['asset_pose', f'seq={sid}', *common, by_id], ['asset_anim', f'anim={d}', *common, loose],
             cells_d, f'sequence {sid} with {weapon}, frames {frames[0]}..{frames[-1]}')
        # the same sequence id on the bare figure and on real items, packed cache against loose animation files
        pair(f'd-seq{sid}-real-part{part}', ['asset_pose', f'seq={sid}', *common, 'none', 'item:1127+item:1079+item:1323'],
             ['asset_anim', f'anim={d}', *common, 'none', 'item:1127+item:1079+item:1323'], [], '')
board(ROOT / 'render' / 'research-live-board-3d.png', cells_d, 2, cell=(900, 300),
      title='3d new sequences 20001 crosscut, 20002 overhead, 20003 salute: loose frames + loose models | BY SEQUENCE ID and ITEM ID, packed cache')
LOG.append(f'TOTAL 3d: {total - tb} differing pixels over {npairs - nb} sheet pairs'); print(LOG[-1])
save_log(HERE / 'step3bd.log')
