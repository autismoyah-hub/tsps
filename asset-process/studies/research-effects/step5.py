"""step5.py: the originals BY ID from the packed cache, through the client's ordinary path (ClientState::load reads the
spot definitions of archive 2 group 13; world_actors::Actors draws them), against the same effects as LOOSE FILES on the
source cache. Also a regression: real effects by id, source cache against packed cache.
usage: step5.py   -> render/research-effects-packed/, step5.log
"""
import sys, pathlib, json
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fx
from fxlib import *
from PIL import Image, ImageDraw

PACKED = str(HERE / 'cache-packed')
OUT = RENDER / 'research-effects-packed'; OUT.mkdir(parents=True, exist_ok=True)
SET = [('ring-bolt', ['proj', 'spot']), ('star-burst', ['actor', 'nactor', 'spot']), ('meridian-mark', ['spot']), ('meridian-slam', ['spot']), ('star-pool', ['spot', 'actor'])]
log = []; total = [0, 0, 0]; cells = []


def note(s): print(s); log.append(s)


for name, jobs in SET:
    ids = json.loads((HERE / 'originals' / name / 'ids.json').read_text())
    d = decode_spot(al.Cache(PACKED).file(2, 13, ids['spot']))
    note(f'{name}: spot {ids["spot"]} read back from the packed cache: model {d["model"]} sequence {d["sequence"]} scale {d["scale"]} ambient {d["ambient"]} contrast {d["contrast"]}')
    for job in jobs:
        for close in (False, True):
            e_id = f'id:{ids["spot"]}'; e_f = f'def:{(HERE / "originals" / name / (name + ".def")).as_posix()}'
            life = fx.life(e_f, job)
            cyc = fx.moments(10, life)
            kw = dict(close=close, size=(480, 400), height=100 if job in ('actor', 'nactor') else 0)
            A, _ = fx.render(e_id, job, cyc, cache=PACKED, **kw)
            B, _ = fx.render(e_f, job, cyc, cache=fx.SRC, **kw)
            nd = sum(fx.diffcount(a, b)[0] for a, b in zip(A, B)); npx = len(cyc) * 480 * 400
            total[0] += nd; total[1] += npx; total[2] += len(cyc)
            note(f'  {job}{" close" if close else ""}: {len(cyc)} frames over {life} cycles: {nd} differing of {npx} pixels')
            if close or job == 'proj': cells.append((A[len(cyc) // 2], B[len(cyc) // 2], f'{name} {job} c{cyc[len(cyc) // 2]}: by id {ids["spot"]} (packed) above, files below'))
note(f'TOTAL originals by id (packed cache) against loose files (source cache): {total[0]} differing of {total[1]} pixels over {total[2]} frame pairs')
reg = [0, 0]
for sid, job in ((2206, 'proj'), (2008, 'actor'), (1446, 'spot')):
    cyc = fx.moments(6, fx.life(f'id:{sid}', job))
    A, _ = fx.render(f'id:{sid}', job, cyc, cache=PACKED); B, _ = fx.render(f'id:{sid}', job, cyc, cache=fx.SRC)
    nd = sum(fx.diffcount(a, b)[0] for a, b in zip(A, B)); reg[0] += nd; reg[1] += len(cyc) * 480 * 400
    note(f'regression: real spot {sid} as {job}, packed cache against source cache: {nd} differing of {len(cyc) * 480 * 400}')
W, H = 320, 266
im = Image.new('RGB', (W * 4, ((len(cells) + 3) // 4) * (2 * H + 20)), (22, 24, 28)); dr = ImageDraw.Draw(im)
for k, (a, b, cap) in enumerate(cells):
    x = (k % 4) * W; y = (k // 4) * (2 * H + 20)
    im.paste(a.resize((W, H)), (x, y)); im.paste(b.resize((W, H)), (x, y + H)); dr.text((x + 3, y + 2 * H + 3), cap[:52], fill=(235, 235, 235), font=fx.font(11))
im.save(OUT / 'packed-vs-files.png')
(HERE / 'step5.log').write_text('\n'.join(log) + '\n', encoding='utf-8')
