"""obtools.py: the rebuild test for ANIMATED OBJECTS (fxtools.py's, for objects with a sequence).

  obtools.py reencode ID OUTDIR [--name N] [--noalpha] [--noscale] [--slow PCT] [--lightshift K] [--stretch PCT] [--noloop]
        a real animated object through OUR encoders as loose files (.loc + model + sequence): the ceiling, or a degraded copy
        (--noloop: drop the sequence's loop field: the object then plays once and RESTS, which is what an author gets who forgets it)
  obtools.py sheet ID OUTDIR                 the reference sheet an author may see: pictures + numbers, no mesh
  obtools.py compare|gate ID OURS.loc [--json F] [--out DIR]
  obtools.py lineup OUT.gif EFFECT.. [--seed N] [--close] [--life CYCLES]     blind moving line-up + OUT.key.txt
The real object is always drawn BY ID through the client's own path (LOC_ADD_CHANGE + LOC_ANIM of its own sequence, so it
starts at frame 0); ours as files, started the same way. One pass of the sequence is compared (a spawn-then-idle sequence:
the spawn and one idle).
Gate = fxtools' gate (life equal; faces within 25% or 24; colours; colour distance 2.5; translucent share 0.12; rest box 6%;
largest posed dimension 10%; overlap mean 0.72, worst 0.45; coverage 0.80..1.25; tone mean 12, worst 24) PLUS the object's own:
loop field equal (frames looped from the end), start rule equal (opcode 89), size in tiles equal, ambient within 10 and
stored contrast within 10 (they are 5 light units each in the client), walk-through or solid equal.
"""
import sys, pathlib, json, random, collections
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fx, fxtools, ob, objlib, objcensus, census
from fxlib import *
sys.path.insert(1, str(ROOT / 'builds' / 'research-npc'))
import meshio
import numpy as np
from PIL import Image, ImageDraw


def read_loc(path):
    """A loose object as fxtools.read_def returns a spot (so def_stats and the gate work on it)."""
    path = pathlib.Path(path)
    kv = dict(l.split('=', 1) for l in path.read_text(encoding='utf-8').splitlines() if '=' in l and not l.strip().startswith('#'))
    ms = [oc.decode_model(oc.unpack(oc.group(7, int(f[6:]))) if f.startswith('cache:') else (path.parent / f).read_bytes()) for f in kv['model'].split(',')]
    if len(ms) > 1: raise SystemExit('obtools reads one-model objects')
    m = ms[0]
    if kv.get('recolor'):
        rc = dict(tuple(int(x) for x in p.split(':')) for p in kv['recolor'].split(','))
        m['colors'] = [rc.get(c, c) for c in m['colors']]
    d = dict(kv=kv, path=path, model=m, scale=[int(x) for x in kv.get('scale', '128,128,128').split(',')], seq=None)
    if 'anim' in kv:
        ad = path.parent / kv['anim']; s = al.decode_sequence((ad / 'seq.dat').read_bytes()); frames = []; base = bid = None
        for fid in s['frames']:
            data = (ad / f'frame_{fid >> 16}_{fid & 0xffff}.dat').read_bytes()
            if base is None: bid = int.from_bytes(data[:2], 'big'); base = al.decode_base((ad / f'base_{bid}.dat').read_bytes())
            frames.append(al.decode_frame(data, base))
        d['seq'] = dict(s=s, base=base, bid=bid, frames=frames, dir=ad)
    return d


def loc_stats(d):
    st = fxtools.def_stats(d); kv = d['kv']
    st['object'] = dict(size=[int(x) for x in kv.get('size', '1,1').split(',')], randomstart=kv.get('randomstart', '1') != '0', contour=kv.get('contour'), walkable=kv.get('solid', '0') == '0',
                        loop=d['seq']['s']['loop_frames'] if d['seq'] else None)
    return st


def reencode(oid, outdir, name=None, noalpha=False, noscale=False, slow=100, lightshift=0, stretch=100, noloop=False):
    outdir = pathlib.Path(outdir); outdir.mkdir(parents=True, exist_ok=True)
    d = objlib.obj(oid); shape = objlib.shape_for(d); ms = objlib.models_for(d, shape)
    if len(ms) != 1: raise SystemExit(f'object {oid}: {len(ms)} models for shape {shape}; reencode takes one-model objects')
    raw = oc.unpack(oc.group(7, ms[0])); mm = oc.decode_model(raw)
    if mm['has_bone_weights'] or any(t != -1 for t in mm['face_textures']): raise SystemExit('bone weights or textures: our encoder writes neither')
    m = meshio.load_bytes(raw); notes = []
    if noalpha: m['alpha'] = [0] * len(m['faces']); notes.append('opaque')
    if lightshift: m['colors'] = [(c & ~127) | max(2, min(126, (c & 127) + lightshift)) for c in m['colors']]; notes.append(f'lightness {lightshift:+d}')
    if stretch != 100: m['vertices'] = [tuple(int(round(c * stretch / 100)) for c in v) for v in m['vertices']]; notes.append(f'stretch {stretch}%')
    name = name or f'loc-{oid}-' + ('degraded' if notes or noscale or noloop or slow != 100 else 'ceiling')
    fxtools.save_model(m, outdir / f'{name}.bin', labelled=mm['vertex_skins'] is not None)
    L = [f'# real object {oid} {names(6).get(oid, "")} through our encoders (obtools.py reencode)', f'#shape={shape}', f'name={name}', f'model={name}.bin', f'ambient={d["ambient"]}', f'contrast={d["contrast"]}',
         'scale=' + ','.join(map(str, d['scale'])), 'offset=' + ','.join(map(str, d['offset'])), f'size={d["size"][0]},{d["size"][1]}', f'solid={0 if d["interact"] == 0 else 1}']
    rc = d['recolor']
    if rc and lightshift:
        f = lambda c: (c & ~127) | max(2, min(126, (c & 127) + lightshift)); rc = [(f(a), f(b)) for a, b in rc]
    if rc: L.append('recolor=' + ','.join(f'{a}:{b}' for a, b in rc))
    if d['mirror']: L.append('mirror=1')
    if not d['randomstart']: L.append('randomstart=0')
    if d['contour'] >= 0: L.append(f'contour={d["contour"]}')
    s, bid, base, frames = al.sequence_frames(d['anim'])
    ad = outdir / f'{name}-anim'; ad.mkdir(exist_ok=True)
    for old in ad.glob('*.dat'): old.unlink()
    if slow != 100: s['lengths'] = [max(1, int(round(x * slow / 100))) for x in s['lengths']]; notes.append(f'holds x{slow}%')
    if noloop: s['loop_frames'] = -1; s['ops'] = [o for o in s['ops'] if o != 2]; notes.append('loop field dropped')
    (ad / 'seq.dat').write_bytes(al.encode_sequence(s)); (ad / f'base_{bid}.dat').write_bytes(al.encode_base(base))
    for fid, fr in zip(s['frames'], frames):
        fr = dict(fr, groups=dict(fr['groups']), masks=dict(fr['masks']))
        for g in list(fr['groups']):
            if noscale and base['types'][g] == 3: del fr['groups'][g]; fr['masks'].pop(g, None)
        (ad / f'frame_{fid >> 16}_{fid & 0xffff}.dat').write_bytes(al.encode_frame(fr, base))
    if noscale: notes.append('no scale transforms')
    L.append(f'anim={name}-anim'); L.insert(1, f'note={"; ".join(notes) or "ceiling: nothing changed"}')
    p = outdir / f'{name}.loc'; p.write_text('\n'.join(L) + '\n', encoding='utf-8')
    print(p, '|', '; '.join(notes) or 'ceiling')
    return p


def strips(effect, cyc, yaw, close, size, life):
    fx.LOC_LIFE = life
    A, _ = fx.render(effect, 'loc', cyc, close=close, size=size, yaw=yaw)
    import tempfile, shutil
    tmp = pathlib.Path(tempfile.mkdtemp(prefix='ob-'))
    fx.run(fx.scene('loc', close, size, yaw) + [f'render {",".join(str(4 + c) for c in cyc)} {tmp.as_posix()}/e'])
    E = fx.frames(f'{tmp.as_posix()}/e', [4 + c for c in cyc]); shutil.rmtree(tmp, ignore_errors=True)
    return A, E


def compare(oid, ours, out=None, n=10):
    real = f'loc:{oid}'; mine = f'locf:{pathlib.Path(ours).as_posix()}'
    rp, d = ob.dump(oid, HERE / 'real' / f'loc-{oid}')
    R = loc_stats(read_loc(rp)); O = loc_stats(read_loc(ours))
    R['object'].update(walkable=d['interact'] == 0); R['ambient'] = d['ambient']; R['contrast'] = d['contrast']
    total_r = objlib.seq_cycles(al.sequence(d['anim'])); total_o = O['sequence']['cycles'] if O['sequence'] else 1
    cyc = fx.moments(n, total_r); cells = []; board = []
    for yaw in (0, 4096):
        Ar, Er = strips(real, cyc, yaw, True, (360, 300), total_r); fx.LOC_LIFE = total_r
        Ao, _ = fx.render(mine, 'loc', cyc, close=True, size=(360, 300), yaw=yaw)
        Gr, Ge = strips(real, cyc, yaw, False, (512, 334), total_r); fx.LOC_LIFE = total_r
        Go, _ = fx.render(mine, 'loc', cyc, close=False, size=(512, 334), yaw=yaw)
        Mr, Mo = fxtools.masks(Ar, Er), fxtools.masks(Ao, Er); Nr, No = fxtools.masks(Gr, Ge), fxtools.masks(Go, Ge)
        for k, c in enumerate(cyc):
            u = (Mr[k] | Mo[k]).sum(); i = (Mr[k] & Mo[k]).sum(); gu = Nr[k] | No[k]
            tone = float(np.abs(np.asarray(Gr[k]).astype(int) - np.asarray(Go[k]).astype(int))[gu].mean()) if gu.any() else 0.0
            cells.append(dict(yaw=yaw, cycle=c, px_real=int(Mr[k].sum()), px_ours=int(Mo[k].sum()), overlap=round(float(i / u), 3) if u else 1.0, game_px_real=int(Nr[k].sum()), game_px_ours=int(No[k].sum()),
                              tone=round(tone, 2), differing_close=int((np.asarray(Ar[k]) != np.asarray(Ao[k])).any(axis=2).sum())))
        board.append((yaw, cyc, Ar, Ao))
    fx.LOC_LIFE = None
    big = [c for c in cells if max(c['px_real'], c['px_ours']) >= fxtools.GATE['min_px']]
    sr = sum(c['game_px_real'] for c in cells); so = sum(c['game_px_ours'] for c in cells)
    res = dict(real=oid, name=names(6).get(oid, ''), ours=str(ours), job='loc', height=0, static=dict(real=R, ours=O), life=dict(real=total_r, ours=total_o),
               motion=dict(cells=cells, overlap_mean=round(float(np.mean([c['overlap'] for c in big])), 3) if big else 1.0, overlap_worst=min([c['overlap'] for c in big]) if big else 1.0,
                           coverage=round(so / sr, 3) if sr else (1.0 if not so else 99.0), tone_mean=round(float(np.mean([c['tone'] for c in cells])), 2), tone_worst=max(c['tone'] for c in cells),
                           differing_close=sum(c['differing_close'] for c in cells), pixels_close=len(cells) * 360 * 300))
    if out:
        out = pathlib.Path(out); out.mkdir(parents=True, exist_ok=True); W = 240; H = 200
        im = Image.new('RGB', (W * len(cyc), (2 * H + 16) * 2), (22, 24, 28)); dr = ImageDraw.Draw(im)
        for r, (yaw, cyc_, Ar, Ao) in enumerate(board):
            y = r * (2 * H + 16); dr.text((4, y + 2), f'real object {oid} above, ours below; yaw {yaw}; close lens', fill=(240, 240, 240), font=fx.font(12))
            for k in range(len(cyc_)):
                im.paste(Ar[k].resize((W, H)), (k * W, y + 16)); im.paste(Ao[k].resize((W, H)), (k * W, y + 16 + H)); dr.text((k * W + 3, y + 18), f'c{cyc_[k]}', fill=(255, 255, 160), font=fx.font(11))
        im.save(out / 'compare.png')
    return res


def gate(res):
    verdict, lines = fxtools.gate(res)
    R, O = res['static']['real'], res['static']['ours']; ro, oo = R['object'], O['object']; fails = []

    def check(ok, text):
        lines.insert(-1, ('ok   ' if ok else 'FAIL ') + text)
        if not ok: fails.append(text)
    check(ro['loop'] == oo['loop'], f'loop field (frames looped from the end; -1 = plays once and rests) {oo["loop"]} against {ro["loop"]}')
    check(ro['randomstart'] == oo['randomstart'], f'random start {oo["randomstart"]} against {ro["randomstart"]} (opcode 89 clears it)')
    check(ro['size'] == oo['size'], f'size in tiles {oo["size"]} against {ro["size"]}')
    check(abs(O['ambient'] - R['ambient']) <= 10 and abs(O['contrast'] - R['contrast']) <= 10, f'ambient {O["ambient"]} / contrast {O["contrast"]} against {R["ambient"]} / {R["contrast"]} (within 10)')
    check(ro['walkable'] == oo['walkable'], f'walk-through {oo["walkable"]} against {ro["walkable"]}')
    return ('PASS' if verdict == 'PASS' and not fails else 'FAIL'), lines


def ref_sheet(oid, outdir):
    """What an author may see of an animated-object rebuild target: pictures by id and numbers; no vertex, no face."""
    outdir = pathlib.Path(outdir); outdir.mkdir(parents=True, exist_ok=True)
    rp, d = ob.dump(oid, HERE / 'real' / f'loc-{oid}')
    st = loc_stats(read_loc(rp)); e = f'loc:{oid}'; s = al.sequence(d['anim']); total = objlib.seq_cycles(s)
    loop = s['loop_frames']; idle = sum(s['lengths'][-loop:]) if loop and loop > 0 else 0
    shown = total + (idle if 0 < loop < len(s['frames']) else 0)       # a spawn-then-idle sequence: show the idle twice
    fx.LOC_LIFE = shown; rows = []
    for label, close, yaw, cam, rot in ((f'at game size (a player two tiles away)', False, 0, None, 0), ('close', True, 0, None, 0), ('close, camera a quarter turn round', True, 4096, None, 0),
                                        ('close, the OBJECT turned a quarter (rotation 1)', True, 0, None, 1), ('from above', True, 0, (120, 1500, 0, 1300 if max(d['size']) == 1 else 700), 0),
                                        ('level with the ground', True, 0, (1500, 60, 60, 1300 if max(d['size']) == 1 else 700), 0)):
        fx.CAM = cam; fx.ROT = rot; cy = fx.moments(8, shown)
        ims, _ = fx.render(e, 'loc', cy, close=close, size=(300, 250), yaw=yaw); rows.append((label, cy, ims))
    fx.CAM = None; fx.ROT = 0; W, H = 300, 250
    im = Image.new('RGB', (W * 8, (H + 18) * len(rows)), (22, 24, 28)); dr = ImageDraw.Draw(im)
    for r, (label, cy, ims) in enumerate(rows):
        y = r * (H + 18); dr.text((4, y + 2), f'object {oid} {names(6).get(oid, "")}: {label}', fill=(240, 240, 240), font=fx.font(13))
        for k, f in enumerate(ims): im.paste(f, (k * W, y + 18)); dr.text((k * W + 3, y + 20), f'c{cy[k]}', fill=(255, 255, 160), font=fx.font(11))
    im.save(outdir / f'sheet-{oid}.png')
    fx.gif(outdir / f'motion-{oid}.gif', [e], 'loc', close=True, size=(360, 300), labels=['close'])
    fx.CAM = (1250, 750, 40, 1150); cyc = list(range(0, shown, 2)); a, _ = fx.render(e, 'loc', cyc, size=(360, 300), yaw=0); b, _ = fx.render(e, 'loc', cyc, size=(360, 300), yaw=4096); fx.CAM = None
    fr = []
    for x, y in zip(a, b):
        f = Image.new('RGB', (720, 300)); f.paste(x, (0, 0)); f.paste(y, (360, 0)); fr.append(f)
    fr[0].save(outdir / f'motion-{oid}-quarter.gif', save_all=True, append_images=fr[1:], duration=40, loop=0)
    fx.gif(outdir / f'motion-{oid}-game.gif', [e], 'loc', close=False, size=(512, 334), labels=['game size'])
    ob.random_field(outdir / f'field-{oid}.gif', str(oid), life=max(120, shown))
    fx.LOC_LIFE = None
    m = st['model']; q = st['sequence']
    J = dict(id=oid, name=names(6).get(oid, ''), kind='animated object',
             definition=dict(size_tiles=d['size'], placement_shape=objlib.shape_for(d), walk_through=d['interact'] == 0, random_start=d['randomstart'], follows_ground=d['contour'] >= 0,
                             ambient=d['ambient'], contrast_stored=d['contrast'], scale=d['scale'], offset=d['offset'], recolours=len(d['recolor']), menu=[t for t in d['ops'].values() if t]),
             counts=dict(vertices=m['verts'], faces=m['faces'], colours=m['colours'], flat_faces=m['flat'], translucent_faces=m['alpha_faces'], translucent_share_of_surface=m['alpha_share'],
                         alpha_values=m['alpha_values'], vertex_labels=m['vlabels'], face_labels=m['flabels']),
             palette=[dict(hsl=t[:3], share=t[3]) for t in m['top']],
             rest_box=dict(x=[m['lo'][0], m['hi'][0]], height=[-m['hi'][1], -m['lo'][1]], z=[m['lo'][2], m['hi'][2]]),
             sequence=dict(cycles=q['cycles'], frames=q['frames'], holds=q['holds'], loop_frames=q['loop'], idle_cycles=idle, transform_kinds_used=q['uses'], groups_in_base=q['groups'],
                           per_frame=[dict(frame=c['frame'], hold=c['hold'], box=c['box'], top=c['top'], bottom=c['bottom'], mean_alpha=c['mean_alpha']) for c in q['curve']]),
             note='an OBJECT is lit once at rest with ambient 64 + a, contrast 768 + 5 x stored, from (-50, -10, -50): from the SIDE, not from above as a spot animation. '
                  'loop_frames = how many frames from the END repeat for ever (the frames before them play once: the spawn); -1 = plays once, then the object stands in its REST pose. '
                  'box = posed size x, height, z in model units before the definition scale; mean_alpha 0 opaque .. 255 invisible. field-ID.gif = nine instances with the definition\'s own start rule.')
    jdump(outdir / f'sheet-{oid}.json', J); print(outdir / f'sheet-{oid}.png')
    return J


def lineup(out, effects, seed=None, close=False, life=None):
    fx.LOC_LIFE = life; rng = random.Random(seed); order = list(range(len(effects))); rng.shuffle(order); eff = [effects[i] for i in order]
    fx.gif(out, eff, 'loc', close=close, size=(300, 260), labels=[str(i + 1) for i in range(len(eff))])
    key = pathlib.Path(out).with_suffix('.key.txt'); key.write_text('\n'.join(f'{k + 1}: {e}' for k, e in enumerate(eff)) + '\n', encoding='utf-8'); fx.LOC_LIFE = None
    print(out, key)


if __name__ == '__main__':
    argv = sys.argv[1:]

    def opt(name, default=None, flag=False):
        if name in argv:
            k = argv.index(name)
            if flag: del argv[k]; return True
            v = argv[k + 1]; del argv[k:k + 2]; return v
        return False if flag else default
    jpath = opt('--json'); outd = opt('--out'); seed = opt('--seed'); close = opt('--close', flag=True); name = opt('--name'); life = opt('--life')
    deg = dict(noalpha=opt('--noalpha', flag=True), noscale=opt('--noscale', flag=True), slow=int(opt('--slow', '100')), lightshift=int(opt('--lightshift', '0')), stretch=int(opt('--stretch', '100')), noloop=opt('--noloop', flag=True))
    cmd = argv[0]
    if cmd == 'reencode': reencode(int(argv[1]), argv[2], name, **deg)
    elif cmd == 'sheet': ref_sheet(int(argv[1]), argv[2])
    elif cmd in ('compare', 'gate'):
        res = compare(int(argv[1]), argv[2], outd); verdict, lines = gate(res); res['gate'] = dict(verdict=verdict, lines=lines)
        if jpath: jdump(jpath, res)
        print(f'compare object {res["real"]} {res["name"]} against {res["ours"]}'); print('\n'.join(lines))
        if cmd == 'gate': print('GATE:', verdict)
    elif cmd == 'lineup': lineup(argv[1], argv[2:], int(seed) if seed else None, close, int(life) if life else None)
    else: raise SystemExit(__doc__)
