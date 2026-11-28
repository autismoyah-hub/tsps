"""fxtools.py: the rebuild test for effects (spot animations and projectiles).

  fxtools.py reencode ID OUTDIR [--name N] [degradations]   a real effect through OUR encoders, as loose files (the ceiling), or degraded
        --noalpha            every face opaque                     --noscale     drop every scale transform from the frames
        --norotate           drop every rotate transform           --lightshift K   add K to every painted lightness
        --hueshift K         add K to every hue                    --stretch PCT  scale the mesh about the origin
        --slow PCT           multiply every frame hold by PCT/100  --jitter N    move every vertex by up to +/-N
  fxtools.py sheet ID JOB OUTDIR [--height H]               the reference sheet an author may see: pictures + numbers, no mesh
  fxtools.py compare ID OURS.def JOB [--height H] [--json F] [--out DIR]
  fxtools.py gate ID OURS.def JOB [--height H] [--json F] [--out DIR]      PASS / FAIL (thresholds below, all provisional)
  fxtools.py lineup OUT.gif JOB EFFECT [EFFECT..] [--height H] [--seed N] [--close]    blind moving line-up + OUT.key.txt
  fxtools.py style OURS.def JOB                             an original against the real effects of its job (numbers only)
JOB is spot | actor | proj (fx.py). The real effect is always drawn BY ID through the client's own path; ours as files.

What compare measures. Static, from the files: faces, vertices, colours, translucent share, flat faces, box at rest,
the largest posed dimension over the sequence, the life in cycles, frame count. In MOTION, from pictures: at 10 moments
of the real effect's life, from two sides (yaw 0 and a quarter turn), close and at game size, each effect is drawn in the
same scene and the same scene is drawn without it; the pixels an effect changes by 8 or more in a channel are its MASK (translucent pixels count; a face faded to nothing does not).
  overlap   = intersection / union of the two masks (close lens)        coverage = ours' mask size / real's (game lens)
  tone      = mean absolute difference per channel (0..255) over the union of the masks, ours against real (game lens)
Gate (rebuild): life equal to the real one's (a projectile: the loop period within 10%); faces within 25% or 24 faces;
colours at most real * 1.5 + 1; colour distance of the painted palettes at most 2.5; translucent share within 0.12; rest box within 6% of the real largest dimension (at least
3 units) on every axis; largest posed dimension within 10%; overlap at least 0.72 on average and 0.45 in the worst cell
that has at least 60 pixels; coverage 0.80 to 1.25 on average; tone at most 12 on average and 24 in the worst cell.
"""
import sys, pathlib, json, random, shutil, collections
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fx
from fxlib import *
import census
sys.path.insert(1, str(ROOT / 'studies' / 'research-npc'))
import meshio
import numpy as np
from PIL import Image, ImageDraw

GATE = dict(colour_distance=2.5, face_frac=0.25, face_abs=24, colour_mult=1.5, alpha_share=0.12, box_frac=0.06, posed_frac=0.10,
            overlap_mean=0.72, overlap_worst=0.45, min_px=60, coverage=(0.80, 1.25), tone_mean=12, tone_worst=24)


# ------------------------------------------------------------------------------------------------ loose definitions
def read_def(path):
    path = pathlib.Path(path)
    kv = dict(l.split('=', 1) for l in path.read_text(encoding='utf-8').splitlines() if '=' in l and not l.strip().startswith('#'))
    rel = lambda v: (pathlib.Path(v) if pathlib.Path(v).is_absolute() else path.parent / v)
    d = dict(kv=kv, path=path)
    if kv['model'].startswith('cache:'): raw = oc.unpack(oc.group(7, int(kv['model'][6:])))
    else: raw = rel(kv['model']).read_bytes()
    m = oc.decode_model(raw)
    if kv.get('recolor'):
        rc = dict(tuple(int(x) for x in p.split(':')) for p in kv['recolor'].split(','))
        m['colors'] = [rc.get(c, c) for c in m['colors']]
    d['model'] = m
    d['scale'] = [int(x) for x in kv.get('scale', '128,128').split(',')]
    d['seq'] = None
    if 'anim' in kv:
        ad = rel(kv['anim'])
        s = al.decode_sequence((ad / 'seq.dat').read_bytes())
        frames = []; base = None; bid = None
        for fid in s['frames']:
            data = (ad / f'frame_{fid >> 16}_{fid & 0xffff}.dat')
            data = data.read_bytes() if data.exists() else al.frame_group(fid >> 16)[fid & 0xffff]
            if base is None:
                bid = int.from_bytes(data[:2], 'big')
                bp = ad / f'base_{bid}.dat'
                base = al.decode_base(bp.read_bytes()) if bp.exists() else al.frame_base(bid)
            frames.append(al.decode_frame(data, base))
        d['seq'] = dict(s=s, base=base, bid=bid, frames=frames, dir=ad)
    elif 'seq' in kv:
        s, bid, base, frames = al.sequence_frames(int(kv['seq']))
        d['seq'] = dict(s=s, base=base, bid=bid, frames=frames, dir=None)
    return d


def def_stats(d):
    """Numbers from the files: the census's model numbers, plus the sequence and the posed size / alpha over it."""
    m = d['model']; st = census.model_stats(m)
    out = dict(model={k: st[k] for k in ('verts', 'faces', 'alpha_faces', 'alpha_share', 'alpha_values', 'mean_alpha', 'flat', 'colours', 'colours90', 'top', 'prios', 'vlabels', 'flabels', 'box', 'lo', 'hi')},
               scale=d['scale'], ambient=int(d['kv'].get('ambient', 0)), contrast=int(d['kv'].get('contrast', 0)), rotation=int(d['kv'].get('rotation', 0)))
    q = d['seq']
    if q:
        s = q['s']; base = q['base']
        kinds = collections.Counter()
        labels = m['vertex_skins'] or [0] * len(m['vertices'])
        curve = []
        for i, fr in enumerate(q['frames']):
            for g, v in fr['groups'].items():
                k = base['types'][g]
                if any(x != (128 if k == 3 else 0) for x in v): kinds[al.KIND.get(k, str(k))] += 1
            v = [list(p) for p in m['vertices']]
            al.apply_frame(v, labels, fr, base)
            lo = [min(p[i2] for p in v) for i2 in range(3)]; hi = [max(p[i2] for p in v) for i2 in range(3)]
            a = list(m['alpha'])
            for g, val in al.transforms(fr, base):
                if base['types'][g] == 5:
                    ls = set(base['labels'][g])
                    for j in range(len(a)):
                        if m['face_skins'][j] in ls: a[j] = max(0, min(255, a[j] + 8 * val[0]))
            curve.append(dict(frame=i, hold=s['lengths'][i], box=[hi[k2] - lo[k2] for k2 in range(3)], top=-lo[1], bottom=-hi[1], mean_alpha=round(sum(a) / max(1, len(a)), 1)))
        out['sequence'] = dict(frames=len(s['frames']), distinct=len(set(s['frames'])), cycles=sum(s['lengths']), holds=s['lengths'], loop=s['loop_frames'], max_loops=s['max_loops'],
                               groups=len(base['types']), group_types=[al.KIND.get(t, str(t)) for t in base['types']], uses={k: n for k, n in kinds.items() if k != 'origin'}, curve=curve,
                               posed_max=max(max(c['box']) for c in curve), posed_min=min(max(c['box']) for c in curve))
    else:
        out['sequence'] = None
    return out


def save_model(m, path, labelled=True):
    """meshio.save, except that a model with NO vertex labels stays without them. That matters: the client skips every
    vertex transform on an unlabelled model (only face alpha still animates); written with labels all 0 the same frames
    would move it (found on the first projectile ceiling, spot 2206)."""
    if labelled: return meshio.save(m, path)
    import native_codec
    nf = len(m['faces'])
    g = dict(vertices=[list(p) for p in m['vertices']], faces=[tuple(f) for f in m['faces']], colors=list(m['colors']), priorities=list(m.get('priorities') or [0] * nf))
    if any(m.get('face_skins') or []): g['face_skins'] = [int(l) for l in m['face_skins']]
    if any(m.get('render_types') or []): g['render_types'] = list(m['render_types'])
    if any(m.get('alpha') or []): g['alpha'] = list(m['alpha'])
    if any(m.get('depth') or []): g['face_flags'] = [d & 255 for d in m['depth']]
    data = native_codec.model(g)
    back = oc.decode_model(data)
    assert back['vertex_skins'] is None and [tuple(v) for v in back['vertices']] == [tuple(v) for v in m['vertices']] and back['colors'] == list(m['colors']) and back['alpha'] == list(m['alpha'])
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True); pathlib.Path(path).write_bytes(data)
    return data


# ------------------------------------------------------------------------------------------------ re-encode / degrade
def reencode(sid, outdir, name=None, noalpha=False, noscale=False, norotate=False, lightshift=0, hueshift=0, stretch=100, slow=100, jitter=0):
    outdir = pathlib.Path(outdir); outdir.mkdir(parents=True, exist_ok=True)
    d = decode_spot(spots()[sid])
    raw = oc.unpack(oc.group(7, d['model']))
    mm = oc.decode_model(raw)
    if mm['has_bone_weights'] or any(t != -1 for t in mm['face_textures']): raise SystemExit(f'spot {sid}: bone weights or textures; our encoder writes neither')
    m = meshio.load_bytes(raw)
    notes = []
    rng = random.Random(239)
    if noalpha: m['alpha'] = [0] * len(m['faces']); notes.append('opaque')
    if lightshift: m['colors'] = [(c & ~127) | max(2, min(126, (c & 127) + lightshift)) for c in m['colors']]; notes.append(f'lightness {lightshift:+d}')
    if hueshift: m['colors'] = [((((c >> 10) + hueshift) & 63) << 10) | (c & 1023) for c in m['colors']]; notes.append(f'hue {hueshift:+d}')
    if stretch != 100: m['vertices'] = [tuple(int(round(c * stretch / 100)) for c in v) for v in m['vertices']]; notes.append(f'stretch {stretch}%')
    if jitter:
        mv = {}
        nv = []
        for v in m['vertices']:
            if v not in mv: mv[v] = tuple(c + rng.randint(-jitter, jitter) for c in v)
            nv.append(mv[v])
        m['vertices'] = nv; notes.append(f'jitter {jitter}')
    name = name or f'spot-{sid}-' + ('degraded' if notes or noscale or norotate or slow != 100 else 'ceiling')
    # recolours of the definition are applied by the definition, as the real one does; keep them in the def
    save_model(m, outdir / f'{name}.bin', labelled=mm['vertex_skins'] is not None)
    L = [f'# real spot animation {sid} {names(8).get(sid, "")} through our encoders (fxtools.py reencode)', f'name={name}', f'model={name}.bin',
         f'scale={d["scale"][0]},{d["scale"][1]}', f'rotation={d["rotation"]}', f'ambient={d["ambient"]}', f'contrast={d["contrast"]}']
    rc = d['recolor']
    if rc and (lightshift or hueshift):
        f = lambda c: ((((((c >> 10) + hueshift) & 63) << 10) | (c & 1023)) & ~127) | max(2, min(126, (c & 127) + lightshift))
        rc = [(f(a), f(b)) for a, b in rc]
    if rc: L.append('recolor=' + ','.join(f'{a}:{b}' for a, b in rc))
    if d['sequence'] >= 0:
        s, bid, base, frames = al.sequence_frames(d['sequence'])
        ad = outdir / f'{name}-anim'; ad.mkdir(exist_ok=True)
        for old in ad.glob('*.dat'): old.unlink()
        if slow != 100: s['lengths'] = [max(1, int(round(x * slow / 100))) for x in s['lengths']]; notes.append(f'holds x{slow}%')
        (ad / 'seq.dat').write_bytes(al.encode_sequence(s))
        (ad / f'base_{bid}.dat').write_bytes(al.encode_base(base))
        for fid, fr in zip(s['frames'], frames):
            fr = dict(fr, groups=dict(fr['groups']), masks=dict(fr['masks']))
            for g in list(fr['groups']):
                k = base['types'][g]
                if (noscale and k == 3) or (norotate and k == 2): del fr['groups'][g]; fr['masks'].pop(g, None)
            (ad / f'frame_{fid >> 16}_{fid & 0xffff}.dat').write_bytes(al.encode_frame(fr, base))
        if noscale: notes.append('no scale transforms')
        if norotate: notes.append('no rotate transforms')
        L.append(f'anim={name}-anim')
    L.insert(1, f'note={"; ".join(notes) or "ceiling: nothing changed"}')
    p = outdir / f'{name}.def'
    p.write_text('\n'.join(L) + '\n', encoding='utf-8')
    print(p, '|', '; '.join(notes) or 'ceiling')
    return p


# ------------------------------------------------------------------------------------------------ pictures and masks
def strips(effect, job, cyc, yaw, close, height, size):
    """(frames with the effect, frames of the same scene without it) at the given life cycles."""
    A, _ = fx.render(effect, job, cyc, close=close, size=size, height=height, yaw=yaw)
    L = fx.scene(job, close, size, yaw)
    tmp = RENDER / 'research-effects-tmp'; tmp.mkdir(exist_ok=True)
    tag = f'{tmp.as_posix()}/e{random.randrange(1 << 30)}'
    fx.run(L + [f'render {",".join(str(4 + c) for c in cyc)} {tag}'])
    return A, fx.frames(tag, [4 + c for c in cyc])


def masks(A, E):
    return [(np.abs(np.asarray(a).astype(int) - np.asarray(e).astype(int)) >= 8).any(axis=2) for a, e in zip(A, E)]


def compare(sid, ours, job, height=0, out=None, n=10):
    real = f'id:{sid}'; mine = f'def:{pathlib.Path(ours).as_posix()}'
    rp, _ = fx.dump_real(sid, HERE / 'real' / f'spot-{sid}')
    R = def_stats(read_def(rp)); O = def_stats(read_def(ours))
    total_r = fx.life(real, job); total_o = fx.life(mine, job)
    cyc = fx.moments(n, total_r)
    cells = []
    board = []
    for yaw in (0, 4096):
        Ar, Er = strips(real, job, cyc, yaw, True, height, (360, 300))
        Ao, _ = fx.render(mine, job, cyc, close=True, size=(360, 300), height=height, yaw=yaw)
        Gr, Ge = strips(real, job, cyc, yaw, False, height, (512, 334))
        Go, _ = fx.render(mine, job, cyc, close=False, size=(512, 334), height=height, yaw=yaw)
        Mr, Mo = masks(Ar, Er), masks(Ao, Er)
        Nr, No = masks(Gr, Ge), masks(Go, Ge)
        for k, c in enumerate(cyc):
            u = (Mr[k] | Mo[k]).sum(); i = (Mr[k] & Mo[k]).sum()
            gu = Nr[k] | No[k]
            tone = float(np.abs(np.asarray(Gr[k]).astype(int) - np.asarray(Go[k]).astype(int))[gu].mean()) if gu.any() else 0.0
            cells.append(dict(yaw=yaw, cycle=c, px_real=int(Mr[k].sum()), px_ours=int(Mo[k].sum()), overlap=round(float(i / u), 3) if u else 1.0,
                              game_px_real=int(Nr[k].sum()), game_px_ours=int(No[k].sum()), tone=round(tone, 2), differing_close=int((np.asarray(Ar[k]) != np.asarray(Ao[k])).any(axis=2).sum())))
        board.append((yaw, cyc, Ar, Ao))
    big = [c for c in cells if max(c['px_real'], c['px_ours']) >= GATE['min_px']]
    sr = sum(c['game_px_real'] for c in cells); so = sum(c['game_px_ours'] for c in cells)
    res = dict(real=sid, name=names(8).get(sid, ''), ours=str(ours), job=job, height=height,
               static=dict(real=R, ours=O), life=dict(real=total_r, ours=total_o),
               motion=dict(cells=cells, overlap_mean=round(float(np.mean([c['overlap'] for c in big])), 3) if big else 1.0,
                           overlap_worst=min([c['overlap'] for c in big]) if big else 1.0, coverage=round(so / sr, 3) if sr else (1.0 if not so else 99.0),
                           tone_mean=round(float(np.mean([c['tone'] for c in cells])), 2), tone_worst=max(c['tone'] for c in cells),
                           differing_close=sum(c['differing_close'] for c in cells), pixels_close=len(cells) * 360 * 300))
    if out:
        out = pathlib.Path(out); out.mkdir(parents=True, exist_ok=True)
        W = 240; H = 200
        im = Image.new('RGB', (W * len(cyc), (2 * H + 16) * 2), (22, 24, 28)); dr = ImageDraw.Draw(im)
        for r, (yaw, cyc_, Ar, Ao) in enumerate(board):
            y = r * (2 * H + 16)
            dr.text((4, y + 2), f'real {sid} above, ours below; yaw {yaw}; close lens', fill=(240, 240, 240), font=fx.font(12))
            for k in range(len(cyc_)):
                im.paste(Ar[k].resize((W, H)), (k * W, y + 16)); im.paste(Ao[k].resize((W, H)), (k * W, y + 16 + H))
                dr.text((k * W + 3, y + 18), f'c{cyc_[k]}', fill=(255, 255, 160), font=fx.font(11))
        im.save(out / 'compare.png')
    return res


def gate(res):
    R, O = res['static']['real'], res['static']['ours']; g = GATE; fails = []; lines = []

    def check(ok, text):
        lines.append(('ok   ' if ok else 'FAIL ') + text)
        if not ok: fails.append(text)
    if res['job'] == 'proj' and R['sequence'] and O['sequence']:
        check(abs(O['sequence']['cycles'] - R['sequence']['cycles']) <= 0.1 * R['sequence']['cycles'], f'loop period {O["sequence"]["cycles"]} against {R["sequence"]["cycles"]} cycles (within 10%)')
    else:
        check(res['life']['ours'] == res['life']['real'], f'life {res["life"]["ours"]} against {res["life"]["real"]} cycles (equal)')
    fr, fo = R['model']['faces'], O['model']['faces']
    check(abs(fo - fr) <= max(g['face_abs'], g['face_frac'] * fr), f'faces {fo} against {fr}')
    check(O['model']['colours'] <= R['model']['colours'] * g['colour_mult'] + 1, f'colours {O["model"]["colours"]} against {R["model"]["colours"]}')
    def cdist(A, B):
        t = 0; w = 0
        for h, sa, l, share in A:
            t += share * min((min(abs(h - h2), 64 - abs(h - h2)) if sa and s2 else 0) + abs(sa - s2) + abs(l - l2) / 4 for h2, s2, l2, _ in B); w += share
        return t / max(w, 1e-9)
    cd = round((cdist(O['model']['top'], R['model']['top']) + cdist(R['model']['top'], O['model']['top'])) / 2, 2)
    check(cd <= g['colour_distance'], f'colour distance of the painted palettes {cd} (at most {g["colour_distance"]}; hue steps + saturation steps + lightness / 4, by surface share)')
    check(abs(O['model']['alpha_share'] - R['model']['alpha_share']) <= g['alpha_share'], f'translucent share {O["model"]["alpha_share"]} against {R["model"]["alpha_share"]}')
    big = max(R['model']['box']); tol = max(3, g['box_frac'] * big)
    for a, ax in enumerate('xyz'):
        check(abs(O['model']['lo'][a] - R['model']['lo'][a]) <= tol and abs(O['model']['hi'][a] - R['model']['hi'][a]) <= tol,
              f'rest box {ax}: {O["model"]["lo"][a]}..{O["model"]["hi"][a]} against {R["model"]["lo"][a]}..{R["model"]["hi"][a]} (tolerance {tol:.0f})')
    if R['sequence'] and O['sequence']:
        check(abs(O['sequence']['posed_max'] - R['sequence']['posed_max']) <= g['posed_frac'] * R['sequence']['posed_max'], f'largest posed dimension {O["sequence"]["posed_max"]} against {R["sequence"]["posed_max"]}')
    elif bool(R['sequence']) != bool(O['sequence']):
        check(False, 'one has a sequence and the other none')
    m = res['motion']
    check(m['overlap_mean'] >= g['overlap_mean'], f'overlap in motion, mean {m["overlap_mean"]} (at least {g["overlap_mean"]})')
    check(m['overlap_worst'] >= g['overlap_worst'], f'overlap in motion, worst cell {m["overlap_worst"]} (at least {g["overlap_worst"]})')
    check(g['coverage'][0] <= m['coverage'] <= g['coverage'][1], f'coverage at game size {m["coverage"]} of the real one (0.80 to 1.25)')
    check(m['tone_mean'] <= g['tone_mean'], f'tone difference, mean {m["tone_mean"]} (at most {g["tone_mean"]})')
    check(m['tone_worst'] <= g['tone_worst'], f'tone difference, worst cell {m["tone_worst"]} (at most {g["tone_worst"]})')
    lines.append(f'     identical pixels check (close lens): {m["differing_close"]} differing of {m["pixels_close"]}')
    return ('PASS' if not fails else 'FAIL'), lines


# ------------------------------------------------------------------------------------------------ reference sheet
def ref_sheet(sid, job, outdir, height=0):
    """What an author may see of a rebuild target: pictures by id and numbers; no vertex, no face."""
    outdir = pathlib.Path(outdir); outdir.mkdir(parents=True, exist_ok=True)
    rp, d = fx.dump_real(sid, HERE / 'real' / f'spot-{sid}')
    st = def_stats(read_def(rp))
    e = f'id:{sid}'
    total = fx.life(e, job)
    rows = []
    cyc = fx.moments(8, total)
    for label, close, yaw, cam, jb in ((f'in its job ({job}), game size', False, 0, None, job), ('in its job, close', True, 0, None, job), ('in its job, close, quarter turn', True, 4096, None, job),
                                       ('in its job, close, from behind', True, 8192, None, job), ('alone on a tile, from above', True, 0, (120, 1500, 0, 1300), 'spot'),
                                       ('alone on a tile, level with the ground', True, 0, (1500, 60, 60, 1300), 'spot')):
        fx.CAM = cam
        life = fx.life(e, jb); cy = fx.moments(8, life)
        ims, _ = fx.render(e, jb, cy, close=close, size=(300, 250), height=height if jb == job else 0, yaw=yaw)
        rows.append((label, cy, ims))
    fx.CAM = None
    W, H = 300, 250
    im = Image.new('RGB', (W * 8, (H + 18) * len(rows)), (22, 24, 28)); dr = ImageDraw.Draw(im)
    for r, (label, cy, ims) in enumerate(rows):
        y = r * (H + 18)
        dr.text((4, y + 2), f'{sid} {names(8).get(sid, "")}: {label}', fill=(240, 240, 240), font=fx.font(13))
        for k, f in enumerate(ims):
            im.paste(f, (k * W, y + 18)); dr.text((k * W + 3, y + 20), f'c{cy[k]}', fill=(255, 255, 160), font=fx.font(11))
    im.save(outdir / f'sheet-{sid}.png')
    fx.gif(outdir / f'motion-{sid}.gif', [e, e], job, close=True, height=height, size=(360, 300), labels=['close', 'close (same)'])
    fx.gif(outdir / f'motion-{sid}-game.gif', [e], job, close=False, height=height, size=(512, 334), labels=['game size'])
    m = st['model']
    J = dict(id=sid, name=names(8).get(sid, ''), job=job, height=height,
             definition=dict(scale=d['scale'], rotation=d['rotation'], ambient=d['ambient'], contrast=d['contrast'], recolours=len(d['recolor'])),
             counts=dict(vertices=m['verts'], faces=m['faces'], colours=m['colours'], flat_faces=m['flat'], translucent_faces=m['alpha_faces'], translucent_share_of_surface=m['alpha_share'],
                         alpha_values=m['alpha_values'], vertex_labels=m['vlabels'], face_labels=m['flabels'], priorities=m['prios']),
             palette=[dict(hsl=t[:3], share=t[3]) for t in m['top']],
             rest_box=dict(x=[m['lo'][0], m['hi'][0]], height=[-m['hi'][1], -m['lo'][1]], z=[m['lo'][2], m['hi'][2]]),
             sequence=st['sequence'] and dict(cycles=st['sequence']['cycles'], frames=st['sequence']['frames'], holds=st['sequence']['holds'], loop=st['sequence']['loop'],
                                              transform_kinds_used=st['sequence']['uses'], groups_in_base=st['sequence']['groups'], group_types=collections.Counter(st['sequence']['group_types']),
                                              per_frame=[dict(frame=c['frame'], hold=c['hold'], box=c['box'], top=c['top'], bottom=c['bottom'], mean_alpha=c['mean_alpha']) for c in st['sequence']['curve']]),
             note='box = posed size x, height, z in model units before the definition scale; mean_alpha 0 opaque .. 255 invisible; the client lights the model once at rest with ambient 64 + a, contrast 850 + c, from (-30, -50, -30)')
    jdump(outdir / f'sheet-{sid}.json', J)
    print(outdir / f'sheet-{sid}.png')
    return J


# ------------------------------------------------------------------------------------------------ blind line-up
def lineup(out, job, effects, height=0, seed=None, close=False, size=(300, 260)):
    rng = random.Random(seed)
    order = list(range(len(effects))); rng.shuffle(order)
    eff = [effects[i] for i in order]
    fx.gif(out, eff, job, close=close, height=height, size=size, labels=[str(i + 1) for i in range(len(eff))])
    key = pathlib.Path(out).with_suffix('.key.txt')
    key.write_text('\n'.join(f'{k + 1}: {e}' for k, e in enumerate(eff)) + '\n', encoding='utf-8')
    print(out, key)


def style(ours, job):
    """An original against the census of real effects of its job (by name), numbers only."""
    O = def_stats(read_def(ours))
    rows = [json.loads(l) for l in (HERE / 'data' / 'spots.jsonl').open(encoding='utf-8')]
    import summary
    want = {'proj': 'projectile', 'spot': None, 'actor': None}[job]
    rs = [r for r in rows if r['s'].get('kind') == 'classic' and (want is None or summary.job(r['name']) == want)]

    def band(v, lo=10, hi=90):
        v = sorted(v); return v[int(len(v) * lo / 100)], v[min(len(v) - 1, int(len(v) * hi / 100))]
    out = []
    for label, mine, pop in (('faces', O['model']['faces'], [r['m']['faces'] for r in rs]), ('colours', O['model']['colours'], [r['m']['colours'] for r in rs]),
                             ('largest dimension', max(O['model']['box']), [max(r['m']['box']) for r in rs]),
                             ('life in cycles', O['sequence']['cycles'] if O['sequence'] else 0, [r['s']['cycles'] for r in rs]),
                             ('frames', O['sequence']['frames'] if O['sequence'] else 0, [r['s']['frames'] for r in rs])):
        lo, hi = band(pop)
        out.append(f'{"ok  " if lo <= mine <= hi else "OUT "} {label}: {mine} (real 10th..90th centile {lo}..{hi}, n={len(pop)})')
    out.append(f'     translucent share {O["model"]["alpha_share"]}, alpha values {O["model"]["alpha_values"]}, flat faces {O["model"]["flat"]}, transform kinds {O["sequence"]["uses"] if O["sequence"] else None}')
    print('\n'.join(out))
    return out


if __name__ == '__main__':
    argv = sys.argv[1:]

    def opt(name, default=None, flag=False):
        if name in argv:
            k = argv.index(name)
            if flag: del argv[k]; return True
            v = argv[k + 1]; del argv[k:k + 2]; return v
        return False if flag else default
    height = int(opt('--height', '0')); jpath = opt('--json'); outd = opt('--out'); seed = opt('--seed'); close = opt('--close', flag=True); name = opt('--name')
    deg = dict(noalpha=opt('--noalpha', flag=True), noscale=opt('--noscale', flag=True), norotate=opt('--norotate', flag=True), lightshift=int(opt('--lightshift', '0')),
               hueshift=int(opt('--hueshift', '0')), stretch=int(opt('--stretch', '100')), slow=int(opt('--slow', '100')), jitter=int(opt('--jitter', '0')))
    cmd = argv[0]
    if cmd == 'reencode': reencode(int(argv[1]), argv[2], name, **deg)
    elif cmd == 'sheet': ref_sheet(int(argv[1]), argv[2], argv[3], height)
    elif cmd in ('compare', 'gate'):
        res = compare(int(argv[1]), argv[2], argv[3], height, outd)
        verdict, lines = gate(res)
        res['gate'] = dict(verdict=verdict, lines=lines)
        if jpath: jdump(jpath, res)
        m = res['motion']
        print(f'compare {res["real"]} {res["name"]} against {res["ours"]} as {res["job"]}')
        print('\n'.join(lines))
        if cmd == 'gate': print('GATE:', verdict)
    elif cmd == 'lineup': lineup(argv[1], argv[2], argv[3:], height, int(seed) if seed else None, close)
    elif cmd == 'style': style(argv[1], argv[2])
    else: raise SystemExit(__doc__)
