"""fx.py: the effects harness. Draws spot animations (graphics) and projectiles in the world through
client/rust-client/examples/asset_effect.rs (the client's own map loader, actor path, clocks and rasteriser).

An EFFECT is `id:N` (a real spot animation, drawn by the client's own path from the cache given) or `def:PATH`
(one of ours, or a real one dumped by `dump`, as loose files: model + definition + sequence, see asset_effect.rs).

  fx.py dump ID OUTDIR                      a real spot animation as loose files: OUTDIR/spot-ID.def, model, anim/
  fx.py sheet OUT.png EFFECT [EFFECT..] [--job spot|actor|proj] [--n 8] [--close] [--cache DIR] [--height H]
                                            contact sheet: one row per effect, N evenly spaced moments of its life
  fx.py gif OUT.gif EFFECT [EFFECT..] [--job ..] [--close] [--step 2] [--labels a,b] [--cache DIR]
                                            the same as a moving picture at the game's timing (STEP cycles of 20 ms a frame)
  fx.py faithful ID [ID..] [--out DIR]      the proof: each real effect by id against itself through the file path,
                                            alone, on a tile, on an actor, as a projectile; differing pixels per frame
Jobs:  spot  = on a tile (zone packet), a player standing two tiles away for scale;
       actor = attached to a player (--height as the packet's height, default 0); nactor = attached to an NPC (a Man);
       proj  = a projectile from a Man to a player four tiles away (start height 172, end 124, slope 16, offset 64, 60 cycles
               unless --proj H0,H1,SLOPE,OFFSET,CYCLES).
--close: a camera 3x nearer and no bystanders for the spot job.
Importable: scene(), run(), life(), dump_real().
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, subprocess, json, shutil, tempfile
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from fxlib import *
from PIL import Image, ImageDraw, ImageFont
import numpy as np

EXE_FX = _AP.exe('asset_effect.exe', EXE)
SRC = str(CACHE)
PLACE = (3222, 3219)        # Lumbridge castle yard: flat real floor, real walls behind
PLAYER_ITEMS = '1127,1079,1163'


def font(size=14):
    try: return ImageFont.truetype('arial.ttf', size)
    except Exception: return ImageFont.load_default()


def run(lines, cache=SRC, keep_log=None):
    """lines: script lines WITHOUT render paths resolved (we pass them through). Returns stdout."""
    tmp = pathlib.Path(tempfile.mkdtemp(prefix='fx-'))
    sp = tmp / 'script.txt'
    sp.write_text('\n'.join(lines) + '\n', encoding='utf-8')
    r = subprocess.run([str(EXE_FX), str(cache), str(sp)], capture_output=True, text=True)
    if keep_log: pathlib.Path(keep_log).write_text('$ asset_effect ' + str(cache) + ' SCRIPT\n# SCRIPT:\n' + '\n'.join(lines) + '\n# OUTPUT:\n' + r.stdout + r.stderr, encoding='utf-8')
    if r.returncode: raise SystemExit('asset_effect failed:\n' + '\n'.join(lines) + '\n' + r.stdout + r.stderr)
    shutil.rmtree(tmp, ignore_errors=True)
    return r.stdout


def frames(prefix, cycles):
    """Load and delete the BMPs a render line wrote."""
    out = []
    for c in cycles:
        p = pathlib.Path(f'{prefix}-{c:04}.bmp')
        out.append(Image.open(p).convert('RGB').copy()); p.unlink()
    return out


def effect_lines(effect, job, start=4, height=0, proj=(172, 124, 16, 64, 60), src='n1', dst='p1', tile=None):
    kind, v = effect.split(':', 1)
    f = '' if kind == 'id' else 'f'
    x, y = tile or PLACE
    if job == 'spot': return [f'{f}spot {v} {x} {y} {height} {start}']
    if job == 'actor': return [f'{f}aspot p1 {v} {height} {start}']
    if job == 'nactor': return [f'{f}aspot n1 {v} {height} {start}']
    if job == 'proj':
        h0, h1, slope, off, dur = proj
        return [f'{f}proj {v} {src} {dst} {h0} {h1} {start} {start + dur} {slope} {off}']
    raise SystemExit('job: spot | actor | proj')


GAME_FOCAL = 450     # pixels: a player (about 190 units) is about 62 px tall, as in the game's 512 x 334 view at default zoom
CLOSE_FOCAL = 1150   # 2.5 times nearer


CAM = None   # optional override for every scene: (distance, elevation, target_up, focal)


def scene(job, close=False, size=(480, 400), yaw=0, up=None):
    """Camera and bystanders for a job. Returns script lines (without effects or render). The camera stands 1250 back
    and 750 up (the game's ordinary downward look); the lens is fixed in pixels, so a cell is a crop at game size."""
    x, y = PLACE
    if CAM:
        L = [f'size {size[0]} {size[1]}', f'focal {CAM[3]}', f'camera {x + .5} {y + .5} {yaw} {CAM[0]} {CAM[1]} {CAM[2]}']
    else:
        L = [f'size {size[0]} {size[1]}', f'focal {CLOSE_FOCAL if close else GAME_FOCAL}']
        L += [f'camera {x + .5} {y + .5} {yaw} 1250 750 {up if up is not None else ((210 if close else 110) if job == "proj" else 70 if job != "spot" or not close else 40)}']
    if job == 'spot':
        if not close: L += [f'player {x + 2} {y} 0 512 {PLAYER_ITEMS}']
    elif job == 'actor':
        L += [f'player {x} {y} 0 0 {PLAYER_ITEMS}']
    elif job == 'nactor':
        L += [f'npc 3106 {x} {y} 256']
    elif job == 'proj':
        L += [f'player {x + 2} {y} 0 1536 {PLAYER_ITEMS}', f'npc 3106 {x - 2} {y} 512']
    return L


def life(effect, job, cache=SRC, proj=(172, 124, 16, 64, 60)):
    """How many cycles the effect shows for in this job."""
    if job == 'proj': return proj[4]
    kind, v = effect.split(':', 1)
    if kind == 'id':
        c = al.Cache(cache) if str(cache) != SRC else None
        d = decode_spot(spots(c)[int(v)])
        if d['sequence'] < 0: return 1
        s = al.sequence(d['sequence'], c)
    else:
        kv = dict(l.split('=', 1) for l in pathlib.Path(v).read_text().splitlines() if '=' in l and not l.startswith('#'))
        if 'anim' in kv:
            p = pathlib.Path(kv['anim']);
            if not p.is_absolute(): p = pathlib.Path(v).parent / p
            s = al.decode_sequence((p / 'seq.dat').read_bytes())
        elif 'seq' in kv: s = al.sequence(int(kv['seq']))
        else: return 1
    if s['cached_animation'] is not None: return (s['cached_range'][1] - s['cached_range'][0])
    return sum(s['lengths'])


def render(effect, job, cycles, cache=SRC, close=False, size=(480, 400), height=0, proj=(172, 124, 16, 64, 60), log=None, yaw=0, start=4):
    tmp = pathlib.Path(tempfile.mkdtemp(prefix='fxr-'))
    L = scene(job, close, size, yaw) + effect_lines(effect, job, start=start, height=height, proj=proj)
    L += [f'render {",".join(str(start + c) for c in cycles)} {tmp.as_posix()}/f']
    out = run(L, cache, keep_log=log)
    ims = frames(f'{tmp.as_posix()}/f', [start + c for c in cycles])
    shutil.rmtree(tmp, ignore_errors=True)
    return ims, out


def moments(n, total):
    """n cycles spread over a life of `total` cycles (first visible cycle is 0; the last is total - 1)."""
    if total <= 1: return [0]
    n = min(n, total)
    return sorted(set(int(round(k * (total - 1) / (n - 1))) for k in range(n))) if n > 1 else [0]


def label_of(effect):
    kind, v = effect.split(':', 1)
    if kind == 'id':
        return f'{v} {names(8).get(int(v), "")}'
    return pathlib.Path(v).stem


def sheet(out, effects, job='spot', n=8, close=False, cache=SRC, height=0, proj=(172, 124, 16, 64, 60), size=(300, 250), labels=None):
    rows = []
    for e in effects:
        total = life(e, job, cache, proj)
        cyc = moments(n, total)
        ims, _ = render(e, job, cyc, cache, close, size, height, proj)
        rows.append((e, total, cyc, ims))
    W = size[0] * n + 8; H = (size[1] + 20) * len(rows) + 4
    im = Image.new('RGB', (W, H), (22, 24, 28)); d = ImageDraw.Draw(im)
    for r, (e, total, cyc, ims) in enumerate(rows):
        y = 2 + r * (size[1] + 20)
        d.text((6, y + 2), f'{labels[r] if labels else label_of(e)}   [{job}; {total} cycles = {total * 0.02:.2f} s]', fill=(240, 240, 240), font=font(13))
        for k, (c, f) in enumerate(zip(cyc, ims)):
            im.paste(f, (4 + k * size[0], y + 18))
            d.text((8 + k * size[0], y + 20), f'c{c}', fill=(255, 255, 160), font=font(11))
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True)
    im.save(out)
    return rows


def gif(out, effects, job='spot', close=False, cache=SRC, height=0, proj=(172, 124, 16, 64, 60), size=(300, 260), step=2, labels=None, cols=None):
    cells = []
    for e in effects:
        total = life(e, job, cache, proj)
        cyc = list(range(0, total, step))
        ims, _ = render(e, job, cyc, cache, close, size, height, proj)
        cells.append(ims)
    steps = max(len(c) for c in cells)
    cols = cols or len(cells); rws = (len(cells) + cols - 1) // cols
    out_frames = []
    names_ = labels or [label_of(e) for e in effects]
    for t in range(steps + max(1, 500 // (20 * step))):
        f = Image.new('RGB', (cols * size[0], rws * (size[1] + 18)), (22, 24, 28)); d = ImageDraw.Draw(f)
        for k, c in enumerate(cells):
            x = (k % cols) * size[0]; y = (k // cols) * (size[1] + 18)
            # an effect that has ended shows the empty scene: its own last-plus-one state is "gone"; hold the final drawn frame dimmed out
            if t < len(c): f.paste(c[t], (x, y + 18))
            else: f.paste(Image.blend(c[-1], Image.new('RGB', c[-1].size, (22, 24, 28)), 0.6), (x, y + 18))
            d.text((x + 4, y + 3), names_[k][:46], fill=(235, 235, 235), font=font(12))
        out_frames.append(f)
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True)
    out_frames[0].save(out, save_all=True, append_images=out_frames[1:], duration=20 * step, loop=0, optimize=False)
    return steps


def dump_real(sid, outdir, cache=None):
    """A real spot animation as the loose files the file path takes. Bytes are the cache's own."""
    outdir = pathlib.Path(outdir); outdir.mkdir(parents=True, exist_ok=True)
    c = al.cache() if cache is None else al.Cache(cache)
    d = decode_spot(c.file(2, 13, sid))
    (outdir / f'model-{d["model"]}.bin').write_bytes(oc.unpack(oc.group(7, d['model'])) if cache is None else c.file(7, d['model'], 0))
    L = [f'# real spot animation {sid} {names(8).get(sid, "")}: its own bytes as loose files (fx.py dump)', f'name=spot{sid}', f'model=model-{d["model"]}.bin',
         f'scale={d["scale"][0]},{d["scale"][1]}', f'rotation={d["rotation"]}', f'ambient={d["ambient"]}', f'contrast={d["contrast"]}']
    if d['recolor']: L.append('recolor=' + ','.join(f'{a}:{b}' for a, b in d['recolor']))
    if not d['flag']: L.append('noflag=1')
    if d['sequence'] >= 0:
        s = al.decode_sequence(c.file(2, 12, d['sequence']))
        if s['cached_animation'] is not None: raise SystemExit(f'spot {sid}: skeletal sequence {d["sequence"]}; the file path poses classic sequences only')
        ad = outdir / f'anim-{d["sequence"]}'; ad.mkdir(exist_ok=True)
        (ad / 'seq.dat').write_bytes(c.file(2, 12, d['sequence']))
        bases = set()
        for fid in set(s['frames']):
            data = c.files(0, fid >> 16)[fid & 0xffff]
            (ad / f'frame_{fid >> 16}_{fid & 0xffff}.dat').write_bytes(data)
            bases.add(int.from_bytes(data[:2], 'big'))
        for b in bases: (ad / f'base_{b}.dat').write_bytes(c.file(1, b, 0))
        L.append(f'anim=anim-{d["sequence"]}')
    p = outdir / f'spot-{sid}.def'
    p.write_text('\n'.join(L) + '\n', encoding='utf-8')
    return p, d


def diffcount(a, b):
    A = np.asarray(a).astype(int); B = np.asarray(b).astype(int)
    return int((A != B).any(axis=2).sum()), A.shape[0] * A.shape[1]


def faithful(ids, outdir):
    outdir = pathlib.Path(outdir); outdir.mkdir(parents=True, exist_ok=True)
    log = []; total_px = 0; total_diff = 0; cells_n = 0
    board_cells = []
    for sid in ids:
        p, d = dump_real(sid, HERE / 'real' / f'spot-{sid}')
        for job, close in (('spot', True), ('spot', False), ('actor', False), ('nactor', False), ('proj', False)):
            e_id, e_f = f'id:{sid}', f'def:{p.as_posix()}'
            total = life(e_id, job)
            cyc = moments(10, total)
            kw = dict(close=close, size=(480, 400), height=(60 if job in ('actor', 'nactor') else 0))
            A, la = render(e_id, job, cyc, **kw)
            B, lb = render(e_f, job, cyc, **kw)
            # the empty scene, to count what the effect itself paints
            E, _ = render(e_id, job, [0], start=100000 if job != 'proj' else 4, proj=(172, 124, 16, 64, 0) if job == 'proj' else (172, 124, 16, 64, 60), **kw) if False else (None, None)
            nd = 0; npx = 0; changed = []
            for c, a, b in zip(cyc, A, B):
                n, t = diffcount(a, b); nd += n; npx += t; changed.append(n)
            # frames of the two paths must also agree on what state they drew
            states_a = [l.split(': ', 1)[1].split(' ', 2)[2] if False else l for l in la.splitlines() if l.startswith('cycle') and (' spot ' in l or ' aspot ' in l or ' proj ' in l)]
            states_b = [l for l in lb.splitlines() if l.startswith('cycle') and ('fspot' in l or 'faspot' in l or 'fproj' in l)]
            line = f'spot {sid} {names(8).get(sid, "")}: {job}{" close" if close else ""}: {len(cyc)} frames over {total} cycles: {nd} differing of {npx} pixels; drawn states by-id {len(states_a)} file {len(states_b)}'
            print(line); log.append(line)
            total_px += npx; total_diff += nd; cells_n += len(cyc)
            mid = len(cyc) // 2
            board_cells.append((A[mid], B[mid], f'{sid} {job}{" close" if close else ""} c{cyc[mid]}: {changed[mid]} differing'))
    line = f'TOTAL: {total_diff} differing of {total_px} pixels over {cells_n} frame pairs, {len(ids)} effects x 5 placements'
    print(line); log.append(line)
    (outdir / 'faithful.log').write_text('\n'.join(log) + '\n', encoding='utf-8')
    # board: by-id above, file below
    cw, ch = 240, 200; cols = 8
    rows = (len(board_cells) + cols - 1) // cols
    im = Image.new('RGB', (cols * cw, rows * (2 * ch + 30)), (22, 24, 28)); dr = ImageDraw.Draw(im)
    for k, (a, b, cap) in enumerate(board_cells):
        x = (k % cols) * cw; y = (k // cols) * (2 * ch + 30)
        im.paste(a.resize((cw, ch)), (x, y)); im.paste(b.resize((cw, ch)), (x, y + ch))
        dr.text((x + 3, y + 2 * ch + 2), cap[:40], fill=(235, 235, 235), font=font(11))
        dr.text((x + 3, y + 2), 'by id', fill=(255, 255, 160), font=font(11)); dr.text((x + 3, y + ch + 2), 'files', fill=(160, 255, 255), font=font(11))
    im.save(outdir / 'faithful-board.png')
    return total_diff, total_px


if __name__ == '__main__':
    argv = sys.argv[1:]

    def opt(name, default=None, flag=False):
        if name in argv:
            k = argv.index(name)
            if flag: del argv[k]; return True
            v = argv[k + 1]; del argv[k:k + 2]; return v
        return False if flag else default

    job = opt('--job', 'spot'); n = int(opt('--n', '8')); close = opt('--close', flag=True); cache = opt('--cache', SRC)
    height = int(opt('--height', '0')); step = int(opt('--step', '2')); labels = opt('--labels'); outd = opt('--out')
    pj = opt('--proj'); proj = tuple(int(x) for x in pj.split(',')) if pj else (172, 124, 16, 64, 60)
    size = opt('--size'); size = tuple(int(x) for x in size.split('x')) if size else None
    cmd = argv[0]
    if cmd == 'dump':
        p, d = dump_real(int(argv[1]), argv[2]); print(p, d)
    elif cmd == 'sheet':
        rows = sheet(argv[1], argv[2:], job, n, close, cache, height, proj, size or (300, 250), labels.split(',') if labels else None)
        print(argv[1], [(e, t) for e, t, _, _ in rows])
    elif cmd == 'gif':
        s = gif(argv[1], argv[2:], job, close, cache, height, proj, size or (300, 260), step, labels.split(',') if labels else None)
        print(argv[1], s, 'steps of', step, 'cycles')
    elif cmd == 'faithful':
        faithful([int(x) for x in argv[1:]], outd or RENDER / 'research-effects-faithful')
    else:
        raise SystemExit(__doc__)
