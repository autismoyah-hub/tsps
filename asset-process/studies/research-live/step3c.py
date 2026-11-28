"""step3c.py: the Meridian warden (NPC 16350) and the star hound (NPC 16351) BY NPC ID from the packed cache, in the
rest pose, stand, walk, attack and death, against the loose-file route (a definition text plus model files, source cache).
  by id  asset_npc.exe PACKED draw npc:ID  -> npc::Definition::load_all (archive 2 group 9) -> npc_model::mesh (models
         from archive 7, Model::combine, recolours) -> light -> Animations::apply_actor -> resize -> Mode::Auto painting.
  loose  asset_npc.exe SOURCE draw def:FILE.npcdef (the same steps on files).
Stand and walk come from the packed definition itself; attack and death are not part of an NPC definition (the server
sends them), so they are given on the command line to both routes.
usage: step3c.py
"""
import sys, pathlib, shutil, json
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from livelib import *
OUT = ROOT / 'render' / 'research-live-3c'
if OUT.exists(): shutil.rmtree(OUT)
OUT.mkdir(parents=True)
B = HERE.parent
total = 0; cells = []; n = 0


def side(p):
    im = Image.open(p).convert("RGB"); n = im.height // 260
    out = Image.new("RGB", (260 * n, 260))
    for k in range(n): out.paste(im.crop((0, 260 * k, 260, 260 * k + 260)), (260 * k, 0))
    return out


def npcdef(path, out):
    d = json.loads(path.read_text())
    models = [m if m.startswith('cache:') else str(pathlib.Path(m) if pathlib.Path(m).is_absolute() else path.parent / m) for m in d['models']]
    lines = [f'name={d["name"]}', 'models=' + '|'.join(models)]
    if d.get('recolor'): lines.append('recolor=' + ','.join(f'{a}:{b}' for a, b in d['recolor']))
    lines += [f'scale={d["scale"][0]},{d["scale"][1]}', f'ambient={d["ambient"]}', f'contrast={d["contrast"]}', f'size={d["size"]}',
              f'zbuffer={1 if d["zbuffer"] else 0}', f'stand={d["stand"]}', f'walk={d["walk"]}']
    out.write_text('\n'.join(lines) + '\n')
    return d


for nid, src_json, real in ((16350, B / 'research-npc' / 'warden' / 'warden.json', 'npc:3010'), (16351, B / 'creature-star-hound-2' / 'star-hound.json', 'npc:106')):
    f = OUT / f'npc-{nid}.npcdef'
    d = npcdef(src_json, f)
    seqs = f';attack={d["attack"]};death={d["death"]}'
    for rows, name in (('bind,stand:0,stand:6', 'stand'), ('walk:0,walk:2,walk:4,walk:6', 'walk'), ('attack:1,attack:3,attack:5,attack:7', 'attack'), ('death:1,death:3,death:5,death:8', 'death')):
        for yaw in (0, 600):
            tag = f'c-{nid}-{name}-yaw{yaw}'
            common = ['draw', None, f'rows={rows}', 'zoom=620', 'cell=260x260'] + ([f'yaw={yaw}'] if yaw else [])
            a = OUT / f'{tag}-byid.bmp'; b = OUT / f'{tag}-loose.bmp'
            common[1] = a; run('asset_npc', PACKED, *common, f'npc:{nid}{seqs}')
            common[1] = b; run('asset_npc', SRC, *common, f'def:{f}{seqs}')
            r = diff(b, a); total += r[0]; n += 1
            line = f'{tag}: by id vs loose {r[0]} differing of {r[1]} pixels ({r[2]} drawn)'
            print(line); LOG.append(line)
            pa = OUT / f'{tag}-byid.png'; pb = OUT / f'{tag}-loose.png'
            side(a).save(pa); side(b).save(pb); a.unlink(); b.unlink()
            if r[0]: diff_image(pb, pa, OUT / f'{tag}-diff.png')
            if yaw == 0:
                cells += [(pb, f'{d["name"]} {name}: loose files'), (pa, f'NPC {nid} {name}: BY ID, packed cache\n{r[0]} differing of {r[1]}')]
board(ROOT / 'render' / 'research-live-board-3c.png', cells, 4, cell=(620, 160),
      title='3c NPCs (frames left to right): loose definition + model files | BY NPC ID from the packed cache')
LOG.append(f'TOTAL 3c: {total} differing pixels over {n} sheet pairs'); print(LOG[-1])
save_log(HERE / 'step3c.log')
