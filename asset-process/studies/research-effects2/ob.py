"""ob.py: animated OBJECTS through the harness (fx.py job `loc`; client example asset_arena.rs).

An object EFFECT is `loc:ID` (a real object by id: Scene::replace_location + WorldMesh, the client's own path) or
`locf:PATH.loc` (ours, or a real one dumped by `dump`, as loose files).

  ob.py dump ID OUTDIR                 a real animated object as loose files: OUTDIR/loc-ID.loc, model-*.bin, anim-SEQ/
  ob.py faithful ID [ID..] [--out DIR] the proof: each real object by id against itself through the file path, two rotations,
                                       close and game size, ten moments of one pass: differing pixels
  ob.py sheet OUT.png EFFECT.. [--n 8] [--close] [--rot R] [--life CYCLES]
  ob.py gif OUT.gif EFFECT.. [--close] [--rot R] [--life CYCLES] [--step 2]
  ob.py random OUT.gif ID|PATH.loc [--n 9]      a 3 x 3 field of one object WITHOUT sync: what a server gets by default
                                                (every instance starts at a random frame unless the definition has opcode 89)
.loc file (key=value): name, model=FILE[,FILE] | cache:ID, anim=DIR | seq=ID, ambient, contrast (as STORED; x5 in the client),
scale=X,Y,Z, offset=X,Y,Z, recolor=A:B,.., size=X,Y, mirror=1, randomstart=0 (opcode 89). A comment line `#shape=22` tells the
harness the placement shape (default 10).
"""
import sys, pathlib, json
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fx, objlib
from fxlib import *
from PIL import Image, ImageDraw
import numpy as np


def dump(oid, outdir):
    outdir = pathlib.Path(outdir); outdir.mkdir(parents=True, exist_ok=True)
    d = objlib.obj(oid); shape = objlib.shape_for(d)
    ms = objlib.models_for(d, shape)
    if not ms: raise SystemExit(f'object {oid}: no model for shape {shape}')
    files = []
    for m in ms:
        (outdir / f'model-{m}.bin').write_bytes(oc.unpack(oc.group(7, m))); files.append(f'model-{m}.bin')
    L = [f'# real object {oid} {names(6).get(oid, "")}: its own bytes as loose files (ob.py dump)', f'#shape={shape}', f'name=loc{oid}', 'model=' + ','.join(files),
         f'ambient={d["ambient"]}', f'contrast={d["contrast"]}', 'scale=' + ','.join(map(str, d['scale'])), 'offset=' + ','.join(map(str, d['offset'])), f'size={d["size"][0]},{d["size"][1]}']
    if d['recolor']: L.append('recolor=' + ','.join(f'{a}:{b}' for a, b in d['recolor']))
    if d['retexture']: raise SystemExit(f'object {oid}: retextured; the file path does not retexture')
    if d['mirror']: L.append('mirror=1')
    if not d['randomstart']: L.append('randomstart=0')
    if d['contour'] >= 0: L.append(f'contour={d["contour"]}')
    if d['anim'] is not None:
        c = al.cache(); s = al.sequence(d['anim'])
        if s['cached_animation'] is not None: raise SystemExit(f'object {oid}: skeletal sequence {d["anim"]}; the file path poses classic sequences only')
        ad = outdir / f'anim-{d["anim"]}'; ad.mkdir(exist_ok=True)
        (ad / 'seq.dat').write_bytes(c.file(2, 12, d['anim']))
        bases = set()
        for fid in set(s['frames']):
            data = c.files(0, fid >> 16)[fid & 0xffff]
            (ad / f'frame_{fid >> 16}_{fid & 0xffff}.dat').write_bytes(data); bases.add(int.from_bytes(data[:2], 'big'))
        for b in bases: (ad / f'base_{b}.dat').write_bytes(c.file(1, b, 0))
        L.append(f'anim=anim-{d["anim"]}')
    p = outdir / f'loc-{oid}.loc'
    p.write_text('\n'.join(L) + '\n', encoding='utf-8')
    return p, d


def faithful(ids, outdir):
    outdir = pathlib.Path(outdir); outdir.mkdir(parents=True, exist_ok=True)
    log = []; tot = [0, 0, 0]; cells = []
    for oid in ids:
        p, d = dump(oid, HERE / 'real' / f'loc-{oid}')
        e_id, e_f = f'loc:{oid}', f'locf:{p.as_posix()}'
        for rot in (0, 1):
            for close in (True, False):
                fx.ROT = rot
                total = fx.life(e_id, 'loc'); cyc = fx.moments(10, total)
                A, la = fx.render(e_id, 'loc', cyc, close=close, size=(480, 400))
                B, lb = fx.render(e_f, 'loc', cyc, close=close, size=(480, 400))
                nd = sum(fx.diffcount(a, b)[0] for a, b in zip(A, B)); npx = len(cyc) * 480 * 400
                line = f'object {oid} {names(6).get(oid, "")}: rotation {rot}{" close" if close else " game size"}: {len(cyc)} frames over {total} cycles: {nd} differing of {npx} pixels'
                print(line); log.append(line); tot[0] += nd; tot[1] += npx; tot[2] += len(cyc)
                if close and rot == 0: cells.append((A[len(cyc) // 2], B[len(cyc) // 2], f'{oid} c{cyc[len(cyc) // 2]}'))
    fx.ROT = 0
    line = f'TOTAL: {tot[0]} differing of {tot[1]} pixels over {tot[2]} frame pairs, {len(ids)} objects x 2 rotations x 2 lenses'
    print(line); log.append(line)
    (outdir / 'faithful-objects.log').write_text('\n'.join(log) + '\n', encoding='utf-8')
    cw, ch = 240, 200; cols = 8
    rows = (len(cells) + cols - 1) // cols
    im = Image.new('RGB', (cols * cw, rows * (2 * ch + 20)), (22, 24, 28)); dr = ImageDraw.Draw(im)
    for k, (a, b, cap) in enumerate(cells):
        x = (k % cols) * cw; y = (k // cols) * (2 * ch + 20)
        im.paste(a.resize((cw, ch)), (x, y)); im.paste(b.resize((cw, ch)), (x, y + ch))
        dr.text((x + 3, y + 2 * ch + 2), cap, fill=(235, 235, 235), font=fx.font(11)); dr.text((x + 3, y + 2), 'by id', fill=(255, 255, 160), font=fx.font(11)); dr.text((x + 3, y + ch + 2), 'files', fill=(160, 255, 255), font=fx.font(11))
    im.save(outdir / 'faithful-objects-board.png')


def random_field(out, what, n=9, life=180, step=3, size=(420, 340)):
    """n instances of one object spawned together, WITHOUT sync: the client's default start for each."""
    x, y = fx.PLACE
    by_id = what.isdigit()
    L = [f'size {size[0]} {size[1]}', f'focal {fx.GAME_FOCAL * 1.6}', f'camera {x + .5} {y + .5} 0 1250 750 40', 'pin']
    import random
    rng = random.Random(7)
    k = 0
    for dx in (-2, 0, 2):
        for dy in (-2, 0, 2):
            if k >= n: break
            if by_id: L.append(f'loc {what} {x + dx} {y + dy} {objlib.shape_for(objlib.obj(int(what)))} 0 4 -')
            else: L.append(f'floc {what} {x + dx} {y + dy} 10 0 4 - draws={rng.random():.4f},{rng.random():.4f}')
            k += 1
    cyc = list(range(4, 4 + life, step))
    tmp = RENDER / 'research-effects2-tmp'; tmp.mkdir(exist_ok=True)
    L.append(f'render {",".join(map(str, cyc))} {tmp.as_posix()}/rf')
    fx.run(L)
    ims = fx.frames(f'{tmp.as_posix()}/rf', cyc)
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True)
    ims[0].save(out, save_all=True, append_images=ims[1:], duration=20 * step, loop=0)
    return ims


if __name__ == '__main__':
    argv = sys.argv[1:]

    def opt(name, default=None, flag=False):
        if name in argv:
            k = argv.index(name)
            if flag: del argv[k]; return True
            v = argv[k + 1]; del argv[k:k + 2]; return v
        return False if flag else default
    n = int(opt('--n', '8')); close = opt('--close', flag=True); outd = opt('--out'); step = int(opt('--step', '2'))
    fx.ROT = int(opt('--rot', '0')); life = opt('--life'); fx.LOC_LIFE = int(life) if life else None
    cmd = argv[0]
    if cmd == 'dump': print(*dump(int(argv[1]), argv[2]))
    elif cmd == 'faithful': faithful([int(a) for a in argv[1:]], outd or RENDER / 'research-effects2-faithful')
    elif cmd == 'sheet': print(fx.sheet(argv[1], argv[2:], 'loc', n, close)[0][:2])
    elif cmd == 'gif': print(fx.gif(argv[1], argv[2:], 'loc', close, step=step))
    elif cmd == 'random': random_field(argv[1], argv[2], n if n != 8 else 9)
    else: raise SystemExit(__doc__)
