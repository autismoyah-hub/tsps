"""fight.py: a fight as data. One small JSON names every beat of an attack by OUR asset names; this tool
  (a) PLAYS it through the harness (asset_arena.exe: the client's own clocks and painter), so an encounter designer sees an
      attack before any server code exists, and
  (b) EMITS the same beats in the form the owner's server data uses (plugins/bosses/data/*.json: the Nex / Nightmare / Yama
      dialect) plus the plugin calls each beat needs, as a file an engineer can paste. The server is never touched.

usage: fight.py play FIGHT.json ATTACK [OUT.gif]          -> OUT.gif, OUT.png (a strip of the beats), OUT.log
       fight.py film OUT.gif FIGHT.json:ATTACK[+ATTACK..] [FIGHT.json:ATTACK ..]     several fights side by side, one clock
       fight.py emit FIGHT.json [OUTDIR]                  -> OUTDIR/NAME.server.json, NAME.server.js
       fight.py check FIGHT.json                          the file against the rules below

THE FORMAT (fight.json)
{ "name": "armillary-sentinel",
  "cache": "path",                     optional: a packed cache holding the NPC (default: the source cache)
  "assets": { NAME: {"kind": "npc" | "seq" | "spot" | "loc", "id": N,            the id it has (or will have) in the cache
                     "file": "path.def | path.loc",                            ours, as loose files (drawn from the files)
                     "sequences": {"despawn": {"id": N, "dir": "anim-despawn"}} } },   a loc's extra sequences (LOC_ANIM)
          an asset with an id and no file is drawn BY ID (a real one, or one already packed)
  "stage": {"npc": NAME, "npc_size": 2, "distance": 6},    the creature, its size in tiles, tiles between it and the player
  "attacks": { ATTACK: {"speedTicks": 5, "beats": [ BEAT, .. ] } } }
TIME. "tick": server game ticks from the start of the attack (600 ms; whole numbers; what the server schedules).
      "cycle": extra client cycles (20 ms; 30 to a tick) for things the CLIENT delays inside a packet (a graphic's delay, a
      projectile's launch and landing). A beat happens at tick x 30 + cycle.
HEIGHTS. "height" of a graphic is the packet's value (units; 128 a tile; a chest is about 100). Projectile "startHeight" /
      "endHeight" are the SERVER's units = packet / 4 (its PacketSender multiplies by 4), as in nex.json.
BEATS  {"do": "npc_anim", "tick": 0, "seq": NAME}
       {"do": "player_anim", "tick": 3, "seq": NAME | id}                          (playback only: the target's reaction)
       {"do": "projectile", "tick": 0, "asset": NAME, "from": "npc", "to": "player", "start": 24, "end": 74,
        "startHeight": 47, "endHeight": 30, "angle": 10, "progress": 110}          start / end = cycles after the beat: launch, landing
       {"do": "impact", "tick": 0, "cycle": 74, "asset": NAME, "on": "player", "height": 100}       a graphic on an actor
       {"do": "telegraph", "tick": 0, "asset": NAME, "tile": "player" | "npc" | [dx, dy], "warnTicks": 4}   a graphic on a tile; its
        sequence should last warnTicks x 30 cycles (checked)
       {"do": "tile_graphic", "tick": 4, "asset": NAME, "tile": .., "height": 0}
       {"do": "hit", "tick": 4, "target": "player" | "tile", "tile": .., "radius": 0, "maxHit": 30, "style": "ranged" | "magic" | "melee" | "typeless"}
       {"do": "object", "tick": 4, "asset": NAME, "tile": .., "lifetimeTicks": 10, "despawn": "despawn", "despawnCycles": 30,
        "tickDamage": [3, 7]}                                                       a hazard OBJECT: spawned, LOC_ANIM despawn before removal
       {"do": "camera_shake", "tick": 4, "slot": 1, "amplitude": 5, "cycles": 20}      slot 0 left-right, 1 up-down, 2 forward-back
"tile": [dx, dy] is relative to the PLAYER's tile at the start of the attack (playback keeps the player still).
"""
import sys, json, pathlib, math
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent; sys.path.insert(0, str(HERE.parent))
import fx
from fxlib import *
from PIL import Image, ImageDraw

DOS = ('npc_anim', 'player_anim', 'projectile', 'impact', 'telegraph', 'tile_graphic', 'hit', 'object', 'camera_shake')


def load(path):
    path = pathlib.Path(path); F = json.loads(path.read_text(encoding='utf-8')); F['_dir'] = path.parent
    return F


def at(b): return int(b.get('tick', 0)) * 30 + int(b.get('cycle', 0))


def asset(F, name):
    if name not in F['assets']: raise SystemExit(f'{F["name"]}: no asset named "{name}"')
    return F['assets'][name]


def apath(F, a): return (F['_dir'] / a['file']).resolve().as_posix()


def seq_cycles_of(F, a):
    """cycles of one pass of an asset's sequence (files or cache)."""
    if 'file' in a: return fx.life(('locf:' if a['kind'] == 'loc' else 'def:') + apath(F, a), 'loc' if a['kind'] == 'loc' else 'spot')
    cache = str((F['_dir'] / F['cache']).resolve()) if F.get('cache') else fx.SRC
    return fx.life(('loc:' if a['kind'] == 'loc' else 'id:') + str(a['id']), 'loc' if a['kind'] == 'loc' else 'spot', cache)


def check(F):
    out = []
    for n, a in F['assets'].items():
        if a['kind'] not in ('npc', 'seq', 'spot', 'loc'): out.append(f'asset {n}: kind {a["kind"]}')
        if 'id' not in a: out.append(f'asset {n}: no id (the server names everything by id)')
        elif a['kind'] in ('spot', 'loc', 'seq') and not 0 <= a['id'] <= 65535: out.append(f'asset {n}: id {a["id"]} does not fit the two bytes every packet gives it')
        if 'file' in a and not (F['_dir'] / a['file']).exists(): out.append(f'asset {n}: file {a["file"]} missing')
    for an, A in F['attacks'].items():
        for i, b in enumerate(A['beats']):
            w = f'{an} beat {i} ({b.get("do")})'
            if b.get('do') not in DOS: out.append(f'{w}: unknown "do"'); continue
            for k in ('asset', 'seq'):
                if k in b and isinstance(b[k], str) and b[k] not in F['assets']: out.append(f'{w}: asset "{b[k]}" is not defined')
            if b['do'] == 'projectile':
                if not b['end'] > b['start'] >= 0: out.append(f'{w}: end must be after start')
                hit = [h for h in A['beats'] if h['do'] == 'hit' and h.get('target') == 'player']
                land = at(b) + b['end']
                if hit and not any(abs(at(h) - land) <= 30 for h in hit): out.append(f'{w}: lands at cycle {land} (tick {land / 30:.1f}) but no hit on the player is within a tick of it')
            if b['do'] == 'telegraph':
                a = asset(F, b['asset']); life = seq_cycles_of(F, a)
                if life != b['warnTicks'] * 30: out.append(f'{w}: the telegraph\'s sequence lasts {life} cycles but warnTicks {b["warnTicks"]} is {b["warnTicks"] * 30}')
                if not any(h['do'] == 'hit' and at(h) == at(b) + b['warnTicks'] * 30 for h in A['beats']): out.append(f'{w}: no hit at tick {b.get("tick", 0) + b["warnTicks"]}, when the warning ends')
            if b['do'] == 'object':
                a = asset(F, b['asset'])
                if b.get('despawn') and b['despawn'] not in a.get('sequences', {}): out.append(f'{w}: asset has no sequence "{b["despawn"]}"')
            if b['do'] == 'impact' and not 0 <= b.get('height', 0) <= 65535: out.append(f'{w}: height')
            if b['do'] in ('telegraph', 'tile_graphic') and not 0 <= b.get('height', 0) <= 255: out.append(f'{w}: a tile graphic\'s height is one byte')
    return out


# ------------------------------------------------------------------------------------------------ play
def script(F, attacks, origin=None, start=20):
    """asset_arena script lines for the attacks played one after another; returns (lines, events [(cycle, text)], total cycles)."""
    x, y = origin or fx.PLACE; st = F['stage']; dist = st.get('distance', 6); size = st.get('npc_size', 1)
    px, py = x - 1, y - dist // 2; nx, ny = x - 1 - (size - 1) // 2, py + dist
    L = []; ev = [(0, 'the creature stands')]; t0 = start; npc_seq = []; pl_seq = []; body = []

    def tile(spec):
        if spec in (None, 'player'): return px, py
        if spec == 'npc': return nx + (size - 1) // 2, ny
        return px + spec[0], py + spec[1]
    for an in attacks:
        A = F['attacks'][an]; end = t0
        for b in A['beats']:
            c = t0 + at(b); do = b['do']
            if do == 'npc_anim':
                npc_seq.append((asset(F, b['seq'])['id'], c)); ev.append((c, f'{an}: the creature\'s sequence "{b["seq"]}"'))
            elif do == 'player_anim':
                pl_seq.append((asset(F, b['seq'])['id'] if isinstance(b['seq'], str) else b['seq'], c))
            elif do == 'projectile':
                a = asset(F, b['asset']); ends = {'npc': 'n1', 'player': 'p1'}
                src, dst = ends[b.get('from', 'npc')], ends[b.get('to', 'player')]
                h0, h1 = int(round(b['startHeight'] * 4)), int(round(b['endHeight'] * 4))
                head = f'fproj {apath(F, a)}' if 'file' in a else f'proj {a["id"]}'
                body.append(f'{head} {src} {dst} {h0} {h1} {c + b["start"]} {c + b["end"]} {b.get("angle", 16)} {b.get("progress", 64)}')
                ev.append((c + b['start'], f'{an}: projectile "{b["asset"]}" flies ({b["end"] - b["start"]} cycles)')); end = max(end, c + b['end'])
            elif do == 'impact':
                a = asset(F, b['asset']); who = {'npc': 'n1', 'player': 'p1'}[b.get('on', 'player')]
                body.append((f'faspot {who} {apath(F, a)}' if 'file' in a else f'aspot {who} {a["id"]}') + f' {b.get("height", 0)} {c}')
                ev.append((c, f'{an}: impact "{b["asset"]}" on the {b.get("on", "player")}')); end = max(end, c + seq_cycles_of(F, a))
            elif do in ('telegraph', 'tile_graphic'):
                a = asset(F, b['asset']); tx, ty = tile(b.get('tile'))
                body.append((f'fspot {apath(F, a)}' if 'file' in a else f'spot {a["id"]}') + f' {tx} {ty} {b.get("height", 0)} {c}')
                ev.append((c, f'{an}: {"telegraph" if do == "telegraph" else "graphic"} "{b["asset"]}" on the floor' + (f': {b["warnTicks"]} ticks of warning' if do == 'telegraph' else '')))
                end = max(end, c + seq_cycles_of(F, a))
            elif do == 'hit':
                ev.append((c, f'{an}: HIT (up to {b.get("maxHit", "?")}, {b.get("style", "typeless")})'))
                if b.get('target') == 'player' and b.get('flinch', True): pl_seq.append((424, c))
            elif do == 'object':
                a = asset(F, b['asset']); tx, ty = tile(b.get('tile')); life = b['lifetimeTicks'] * 30; gone = c + life
                if 'file' in a:
                    body.append(f'floc {apath(F, a)} {tx} {ty} 10 0 {c} {gone} sync')
                    if b.get('despawn'): body.append(f'flocanim {tx} {ty} {(pathlib.Path(apath(F, a)).parent / a["sequences"][b["despawn"]]["dir"]).as_posix()} {gone - b.get("despawnCycles", 30)}')
                else:
                    import objlib
                    body.append(f'loc {a["id"]} {tx} {ty} {objlib.shape_for(objlib.obj(a["id"]))} 0 {c} {gone}' + (' sync' if b.get('sync', True) else ''))
                    if b.get('despawn'): body.append(f'locanim {tx} {ty} 10 0 {a["sequences"][b["despawn"]]["id"]} {gone - b.get("despawnCycles", 30)}')
                ev.append((c, f'{an}: hazard object "{b["asset"]}" for {b["lifetimeTicks"]} ticks')); ev.append((gone - b.get('despawnCycles', 30) if b.get('despawn') else gone, f'{an}: the hazard goes'))
                end = max(end, gone + 10)
            elif do == 'camera_shake':
                body.append(f'shake {b.get("slot", 1)} {b.get("amplitude", 5) * 2.2} {c} {c + b.get("cycles", 20)}'); ev.append((c, f'{an}: camera shake'))
        t0 = max(end + 30, t0 + A.get('speedTicks', 4) * 30)
    npc = asset(F, st['npc'])
    L.append(f'npc {npc["id"]} {nx} {ny} 1024 ' + ' '.join(f'seq={s}@{c}' for s, c in npc_seq))
    L.append(f'player {px} {py} 0 0 {fx.PLAYER_ITEMS} ' + ' '.join(f'seq={s}@{c}' for s, c in pl_seq))
    return L + body, sorted(ev), t0


def render(F, attacks, size=(560, 420), step=2):
    x, y = fx.PLACE
    L, ev, total = script(F, attacks)
    cyc = list(range(0, total, step))
    import tempfile, shutil
    tmp = pathlib.Path(tempfile.mkdtemp(prefix='fight-'))
    head = [f'size {size[0]} {size[1]}', f'focal {560 * size[0] / 560:.0f}', f'camera {x + .5} {y + .5} 4096 1250 780 80', 'pin']
    lines = head + L + [f'render {",".join(map(str, cyc))} {tmp.as_posix()}/f']
    cache = str((F['_dir'] / F['cache']).resolve()) if F.get('cache') else fx.SRC
    log = fx.run(lines, cache)
    ims = fx.frames(f'{tmp.as_posix()}/f', cyc); shutil.rmtree(tmp, ignore_errors=True)
    return ims, cyc, ev, lines, log


def caption(im, c, ev, title=None):
    d = ImageDraw.Draw(im); text = [t for a, t in ev if a <= c][-1]
    d.rectangle((0, 0, im.width, 18), fill=(16, 18, 22)); d.text((5, 2), (f'{title}   ' if title else '') + f'tick {c / 30:4.1f}  {text}'[:int(im.width / 6.2)], fill=(240, 240, 240), font=fx.font(12))
    return im


def play(path, attack, out=None):
    F = load(path); bad = check(F)
    for b in bad: print('CHECK:', b)
    attacks = attack.split('+'); out = pathlib.Path(out or RENDER / 'research-effects2-fight' / f'{F["name"]}-{attack}.gif'); out.parent.mkdir(parents=True, exist_ok=True)
    ims, cyc, ev, lines, log = render(F, attacks)
    ims = [caption(im, c, ev) for im, c in zip(ims, cyc)]
    ims[0].save(out, save_all=True, append_images=ims[1:] + [ims[-1]] * 10, duration=40, loop=0)
    # a strip: the frame 4 cycles after each event
    pick = sorted({min(len(ims) - 1, (a + 6) // 2) for a, _ in ev})
    W, H = ims[0].size; cols = 5; rows = (len(pick) + cols - 1) // cols
    sh = Image.new('RGB', (W * cols // 2, (H // 2) * rows))
    for k, i in enumerate(pick): sh.paste(ims[i].resize((W // 2, H // 2)), ((k % cols) * (W // 2), (k // cols) * (H // 2)))
    sh.save(out.with_suffix('.png'))
    out.with_suffix('.log').write_text('# script given to asset_arena\n' + '\n'.join(lines) + '\n# events\n' + '\n'.join(f'{a:5d}  {t}' for a, t in ev) + '\n', encoding='utf-8')
    print(out, len(ims), 'frames;', len(ev), 'events;', 'check:', 'clean' if not bad else f'{len(bad)} problems')


def film(out, specs, size=(430, 360)):
    panels = []
    for sp in specs:
        p, attack = sp.rsplit(':', 1); F = load(p)
        ims, cyc, ev, _, _ = render(F, attack.split('+'), size=size)
        panels.append((F.get('title', F['name']), ims, cyc, ev))
    n = max(len(p[1]) for p in panels); frames = []
    for t in range(n):
        f = Image.new('RGB', (size[0] * len(panels), size[1]), (22, 24, 28))
        for k, (title, ims, cyc, ev) in enumerate(panels):
            i = min(t, len(ims) - 1); f.paste(caption(ims[i].copy(), cyc[i], ev, title), (k * size[0], 0))
        frames.append(f)
    out = pathlib.Path(out); out.parent.mkdir(parents=True, exist_ok=True)
    frames[0].save(out, save_all=True, append_images=frames[1:] + [frames[-1]] * 10, duration=40, loop=0)
    # a sheet: every 24 cycles
    pick = list(range(0, n, 12)); cols = 4; W = size[0] * len(panels) // 2; H = size[1] // 2; rows = (len(pick) + cols - 1) // cols
    sh = Image.new('RGB', (W * cols, H * rows))
    for k, i in enumerate(pick): sh.paste(frames[i].resize((W, H)), ((k % cols) * W, (k // cols) * H))
    sh.save(out.with_name(out.stem + '-sheet.png'))
    print(out, n, 'frames of 2 cycles;', len(panels), 'panels')


# ------------------------------------------------------------------------------------------------ emit
def emit(path, outdir=None):
    F = load(path); outdir = pathlib.Path(outdir or F['_dir']); outdir.mkdir(parents=True, exist_ok=True)
    idof = lambda n: asset(F, n)['id']
    S = {'$comment': f'{F["name"]}: emitted by fight.py from {pathlib.Path(path).name}; do not edit by hand. Dialect of plugins/bosses/data (nex.json, nightmare.json, yama.json): ticks are game ticks; '
                     'projectile start / end and graphic delays are client cycles (30 to a tick) counted from the send; projectile heights are the server\'s units (the packet\'s value over 4: '
                     'PacketSender.sendProjectile multiplies by 4); graphic heights are the packet\'s. Every id is the id the asset has in the cache the server and the client load. '
                     'A JSON alone does nothing on this server: each boss has its own plugin that reads its own keys (see the .server.js beside this file).',
         'npcs': {F['stage']['npc']: idof(F['stage']['npc'])}, 'spotanims': {n: a['id'] for n, a in F['assets'].items() if a['kind'] == 'spot'},
         'locs': {n: a['id'] for n, a in F['assets'].items() if a['kind'] == 'loc'}, 'anims': {n: a['id'] for n, a in F['assets'].items() if a['kind'] == 'seq'},
         'locAnims': {f'{n}.{k}': v['id'] for n, a in F['assets'].items() if a['kind'] == 'loc' for k, v in a.get('sequences', {}).items()}, 'attacks': {}}
    js = [f'// {F["name"]}: the plugin calls for each attack, emitted by fight.py beside {F["name"]}.server.json. A sketch to paste into the boss\'s own plugin',
          '// (plugins/bosses/<boss>/Attacks.<Boss>.js): it uses only calls the Nex and Nightmare plugins already make. NOT run against the server.',
          '// `core` = api.core; `data` = the parsed .server.json; `later(ticks, fn)` = the plugin\'s own TaskManager wrapper (plugins/bosses/nex/Common.Nex.js:36).',
          'function graphic(core, id, height, delay) { const g = new core.Graphic(id); g.height = height || 0; g.delay = delay || 0; return g; }   // never new Graphic(id, n): n is read as a priority or a height', '']
    for an, A in F['attacks'].items():
        o = {'speedTicks': A.get('speedTicks', 4)}; fn = [f'function {an}(core, data, npc, player, area, later) {{', f'  const a = data.attacks.{an};']
        tiles = {}
        for b in A['beats']:
            do = b['do']; T = b.get('tick', 0); C = b.get('cycle', 0)
            wrap = (lambda s, T=T: f'  later({T}, () => {{ {s} }});' if T else f'  {s}')
            tl = b.get('tile')
            texpr = 'npc.getLocation()' if tl == 'npc' else ('target' if tl in (None, 'player') else f'target.transform({tl[0]}, {tl[1]})')
            if do == 'npc_anim':
                if 'anim' in o: o.setdefault('againAnims', []).append(dict(anim=idof(b['seq']), tick=T, delay=C))
                else: o['anim'] = idof(b['seq']); o['animTick'] = T
                fn.append(wrap('npc.performAnimation(new core.Animation(a.anim));'))
            elif do == 'projectile':
                o['projectile'] = dict(id=idof(b['asset']), start=C + b['start'], end=C + b['end'], angle=b.get('angle', 16), progress=b.get('progress', 64), startHeight=b['startHeight'], endHeight=b['endHeight'])
                o['projectileTicks'] = max(1, math.ceil((T * 30 + C + b['end']) / 30))
                fn.append(wrap('const p = a.projectile; new core.Projectile(core.Projectile.centreOf(npc), player.getLocation(), player, p.id, p.start, p.end, p.startHeight, p.endHeight, area)'
                               '.withAngle(p.angle).withProgress(p.progress).withSource(npc).sendProjectile();'))
            elif do == 'impact':
                o['impact'] = dict(id=idof(b['asset']), height=b.get('height', 0), delay=T * 30 + C)
                fn.append('  player.performGraphic(graphic(core, a.impact.id, a.impact.height, a.impact.delay));     // delay in client cycles: sent with the projectile')
            elif do == 'telegraph':
                o['warnSpot'] = idof(b['asset']); o['warnTicks'] = b['warnTicks']; o['warnTick'] = T
                if 'target' not in tiles: fn.append('  const target = player.getLocation().clone();     // the tile is fixed when the warning starts'); tiles['target'] = 1
                fn.append(wrap(f'player.getPacketSender().sendGlobalGraphic(graphic(core, a.warnSpot, {b.get("height", 0)}, {C}), {texpr});'))
            elif do == 'tile_graphic':
                o.setdefault('tileGraphics', []).append(dict(id=idof(b['asset']), tick=T, delay=C, height=b.get('height', 0)))
                if 'target' not in tiles: fn.append('  const target = player.getLocation().clone();'); tiles['target'] = 1
                fn.append(wrap(f'player.getPacketSender().sendGlobalGraphic(graphic(core, {idof(b["asset"])}, {b.get("height", 0)}, {C}), {texpr});'))
            elif do == 'hit':
                o['hitTicks'] = T; o['maxHit'] = b.get('maxHit', 0); o['style'] = b.get('style', 'typeless')
                if b.get('radius') is not None: o['radius'] = b['radius']
                if b.get('target') == 'tile':
                    fn.append(wrap(f'if (player.getLocation().getDistance({texpr}) <= {b.get("radius", 0)}) player.getCombat().getHitQueue().addPendingDamage([new core.HitDamage(core.Misc.getRandom(a.maxHit), core.HitMask.RED)]);'))
                else:
                    fn.append(f'  core.CombatFactory.addPendingHit(new core.PendingHit(npc, player, this, {{ delay: a.hitTicks, damage: core.Misc.getRandom(a.maxHit) }}));     // style: {o["style"]}')
            elif do == 'object':
                a_ = asset(F, b['asset'])
                o['hazard'] = dict(loc=a_['id'], spawnTick=T, lifetimeTicks=b['lifetimeTicks'], tickDamage=b.get('tickDamage'), type=10, face=0)
                if b.get('despawn'): o['hazard'].update(despawnAnim=a_['sequences'][b['despawn']]['id'], despawnTicks=math.ceil(b.get('despawnCycles', 30) / 30))
                if 'target' not in tiles: fn.append('  const target = player.getLocation().clone();'); tiles['target'] = 1
                fn += [f'  later({T}, () => {{', f'    const pool = new core.GameObject(a.hazard.loc, {texpr}, a.hazard.type, a.hazard.face, area);',
                       '    core.ObjectManager.register(pool, true);                         // LOC_ADD_CHANGE: the object appears and its sequence starts',
                       '    // while it stands: each tick, players on its tile take a.hazard.tickDamage (the plugin\'s own tick task)']
                if b.get('despawn'):
                    fn += ['    later(a.hazard.lifetimeTicks - a.hazard.despawnTicks, () => pool.performAnimation(new core.Animation(a.hazard.despawnAnim)));   // LOC_ANIM: the despawn sequence']
                fn += ['    later(a.hazard.lifetimeTicks, () => core.ObjectManager.deregister(pool, true));   // LOC_DEL', '  });']
            elif do == 'camera_shake':
                o.setdefault('cameraShake', []).append([b.get('slot', 1), b.get('amplitude', 5), T, math.ceil(b.get('cycles', 20) / 30)])
                fn.append(wrap(f'player.getPacketSender().sendCameraShake({b.get("slot", 1)}, {b.get("amplitude", 5)}); later({max(1, math.ceil(b.get("cycles", 20) / 30))}, () => player.getPacketSender().sendCameraReset());'))
        S['attacks'][an] = o; js += fn + ['}', '']
    (outdir / f'{F["name"]}.server.json').write_text(json.dumps(S, indent=2), encoding='utf-8')
    (outdir / f'{F["name"]}.server.js').write_text('\n'.join(js) + '\n', encoding='utf-8')
    print(outdir / f'{F["name"]}.server.json', outdir / f'{F["name"]}.server.js')


if __name__ == '__main__':
    a = sys.argv[1:]
    if not a: raise SystemExit(__doc__)
    if a[0] == 'play': play(a[1], a[2], a[3] if len(a) > 3 else None)
    elif a[0] == 'film': film(a[1], a[2:])
    elif a[0] == 'emit': emit(a[1], a[2] if len(a) > 2 else None)
    elif a[0] == 'check':
        bad = check(load(a[1])); print('\n'.join(bad) or 'clean')
    else: raise SystemExit(__doc__)
