"""NPC harness: real NPCs, NPCs of ours given as model files plus a small definition, and player figures, side by side,
in the rest pose and at frames of their stand, walk, attack and death sequences, lit and painted as the live client does.

usage: npc.py OUT.png "ENTRY,ENTRY,..." [--rows bind,stand:0,walk:2,attack:3,death:5,seq:ID:FRAME]
              [--zoom N] [--yaw N] [--pitch N] [--cell WxH] [--aim HEIGHT|own] [--game] [--draw def|auto|actor|depth]
              [--diff A:B[,A:C..]] [--keep-bmp]
       npc.py --dump NPC_ID OUTDIR      write a real NPC's own model files and its definition as OUTDIR/npc-ID.npcdef
       npc.py --anims NPC_ID            print the attack / death sequences the server tables give for a real NPC

ENTRY = npc:ID[;attack=SEQ][;death=SEQ][;NAME=SEQ]   a real NPC from the cache (client's own npc_model::mesh). Stand and
                                                      walk come from its definition; attack and death are NOT in the
                                                      cache: they are looked up in the owner's server tables (TSPS
                                                      data/definitions/npc-combat-defs.json, then npc-captured-
                                                      animations.json by hit timing, then npc-animations.json is only
                                                      listed) unless given here.
      | def:PATH.json[;NAME=SEQ]                      one of OURS: a JSON definition
      | def:PATH.npcdef                               the same in the harness's own key=value text form
      | player:OUTFIT[;stand=SEQ;walk=SEQ;attack=SEQ] a player figure; OUTFIT is a pose.py entry (none, item:ID,
                                                      model:PATH@SLOT+HIDE, joined with +)
JSON definition (paths relative to the JSON file):
  {"name": "Meridian warden", "models": ["a.bin", "b.bin", "cache:249"], "recolor": [[from, to], ...],
   "scale": [128, 128], "ambient": 0, "contrast": 0, "size": 1, "zbuffer": true,
   "stand": 813, "walk": 1146, "attack": 440, "death": 836}
  zbuffer false = NPC opcode 147: the NPC is painted by face priority like a player (14 real NPCs do this).
  ambient and contrast are the values as STORED in a definition (the client multiplies contrast by 5).
ROW   = bind | NAME:FRAME | seq:ID:FRAME   (a frame past the end is clamped to the last)

How the game draws an NPC (client source, quoted in examples/asset_npc.rs): every model of the definition is combined
into ONE model (npc_model.rs mesh -> Model::combine), recoloured, lit once at rest with light(64 + ambient,
850 + contrast, (-30, -50, -30)), animated, scaled, and painted with Mode::Auto: faces in model order, each pixel
depth-tested, face priorities IGNORED, per-face depth offsets honoured (world_actors.rs:500 with model_order.rs:68,
render3d.rs:864). Only a definition with opcode 147 is painted by priority as a player is. --draw overrides this to
compare (auto = as an ordinary NPC, actor = as a player).
--game is the game-size preset: --zoom 1150 --pitch 220 --cell 160x200 (a person about 90 pixels tall, seen from above).
--aim is the height (units above the feet, after scale) the camera looks at; by default half the tallest entry's height,
so every figure stands on one ground line; `own` centres each on itself.
--diff A:B counts differing pixels between two columns, per row (1-based).  Writes OUT.png and OUT.key.txt.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib, json
from PIL import Image, ImageDraw, ImageFont
EXE = _AP.fix(r'<CLIENT_BIN>\asset_npc.exe')
CACHE = _AP.fix(r'<OSRS_CACHE_239>')
TSPS = pathlib.Path(_AP.fix(r'<TSPS_CHECKOUT>\server\data\definitions'))
_tables = None
def server_anims(npc_id):
    """(attack, death, where) for a real NPC from the server's tables; None where unknown."""
    global _tables
    if _tables is None:
        rd = lambda n: json.loads((TSPS / n).read_text(encoding='utf-8'))
        _tables = (rd('npc-combat-defs.json')['npcs'], rd('npc-captured-animations.json')['npcs'], rd('npc-animations.json'))
    combat, captured, listed = _tables
    k = str(npc_id); attack = death = None; where = []
    if k in combat and 'anims' in combat[k]:
        a = combat[k]['anims']; attack, death = a.get('attack'), a.get('death'); where.append('npc-combat-defs')
    if k in captured and (attack is None or death is None):
        seen = captured[k]['seen']
        if attack is None:
            best = max(seen.items(), key=lambda kv: kv[1][2])
            if best[1][2] > 0: attack = int(best[0])
        if death is None:
            best = max(seen.items(), key=lambda kv: kv[1][0])
            if best[1][0] > 0: death = int(best[0])
        where.append('npc-captured-animations')
    if attack is not None and attack < 0: attack = None
    if death is not None and death < 0: death = None
    return attack, death, '+'.join(where) or 'none', listed.get(k)
def to_npcdef(path, outdir):
    """A JSON definition as the harness's text form; model paths made absolute."""
    j = json.loads(path.read_text(encoding='utf-8'))
    lines = [f'# from {path}', f"name={j.get('name', path.stem)}"]
    ms = [m if m.startswith('cache:') or pathlib.Path(m).is_absolute() else str((path.parent / m).resolve()) for m in j['models']]
    lines.append('models=' + '|'.join(ms))
    if j.get('recolor'): lines.append('recolor=' + ','.join(f'{a}:{b}' for a, b in j['recolor']))
    if j.get('retexture'): lines.append('retexture=' + ','.join(f'{a}:{b}' for a, b in j['retexture']))
    sc = j.get('scale', [128, 128]); lines.append(f'scale={sc[0]},{sc[1]}')
    for k in ('ambient', 'contrast', 'size'):
        if k in j: lines.append(f'{k}={j[k]}')
    lines.append(f"zbuffer={1 if j.get('zbuffer', True) else 0}")
    for k, v in j.items():
        if k not in ('name', 'models', 'recolor', 'retexture', 'scale', 'ambient', 'contrast', 'size', 'zbuffer', 'note', 'heads', 'combat') and isinstance(v, int):
            lines.append(f'{k}={v}')
    out = outdir / (path.stem + '.npcdef'); out.write_text('\n'.join(lines) + '\n', encoding='utf-8')
    return out
def main():
    argv = sys.argv
    def opt(name): return argv[argv.index(name) + 1] if name in argv else None
    if '--dump' in argv:
        i = argv.index('--dump'); r = subprocess.run([EXE, CACHE, 'dump', argv[i + 1], argv[i + 2]], capture_output=True, text=True)
        print(r.stdout + r.stderr); raise SystemExit(r.returncode)
    if '--anims' in argv:
        print(server_anims(int(opt('--anims')))); raise SystemExit
    out = pathlib.Path(argv[1]); entries = argv[2].split(',')
    out.parent.mkdir(parents=True, exist_ok=True)
    rows = (opt('--rows') or 'bind').split(',')
    o = {k: opt('--' + k) for k in ('zoom', 'yaw', 'pitch', 'cell', 'aim', 'draw')}
    if '--game' in argv:
        o['zoom'] = o['zoom'] or '1150'; o['pitch'] = o['pitch'] or '220'; o['cell'] = o['cell'] or '160x200'
    resolved = []; notes = []
    for e in entries:
        head, *rest = e.split(';')
        if head.startswith('npc:'):
            given = {p.split('=')[0] for p in rest}
            attack, death, where, listed = server_anims(int(head[4:]))
            if 'attack' not in given and attack is not None: rest.append(f'attack={attack}')
            if 'death' not in given and death is not None: rest.append(f'death={death}')
            notes.append(f'{head}: server tables ({where}) attack {attack} death {death}; npc-animations.json lists {listed}')
        elif head.startswith('def:') and head.lower().endswith('.json'):
            head = 'def:' + str(to_npcdef(pathlib.Path(head[4:]), out.parent))
        resolved.append(';'.join([head] + rest))
    bmp = out.with_suffix('.bmp')
    cmd = [EXE, CACHE, 'draw', str(bmp), 'rows=' + ','.join(rows)] + [f'{k}={v}' for k, v in o.items() if v] + resolved
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    im = Image.open(bmp).convert('RGB')
    if '--keep-bmp' not in argv: bmp.unlink()
    cw, ch = im.width // len(entries), im.height // len(rows)
    report = []
    if opt('--diff'):
        px = im.load(); background = px[0, 0]
        for pair in opt('--diff').split(','):
            a, b = (int(v) - 1 for v in pair.split(':'))
            for row, f in enumerate(rows):
                n = drawn = 0
                for y in range(ch):
                    for x in range(cw):
                        p, q = px[a * cw + x, row * ch + y], px[b * cw + x, row * ch + y]
                        drawn += p != background
                        n += p != q
                report.append(f'diff col {a + 1} vs col {b + 1} row {f}: {n} of {cw * ch} pixels differ ({drawn} figure pixels in col {a + 1})')
    d = ImageDraw.Draw(im)
    try: font = ImageFont.truetype('segoeuib.ttf', 22 if cw >= 200 else 14); small = ImageFont.truetype('segoeuib.ttf', 14 if cw >= 200 else 10)
    except Exception: font = small = ImageFont.load_default()
    for k in range(len(entries)): d.text((k * cw + 6, 4), str(k + 1), fill=(240, 240, 240), font=font)
    for row, f in enumerate(rows):
        d.text((4, row * ch + ch - 18), f, fill=(200, 200, 160), font=small)
        if row: d.line([(0, row * ch), (im.width, row * ch)], fill=(70, 64, 56))
    im.save(out)
    head = 'rows ' + ','.join(rows) + ' ' + ' '.join(f'{k}={v}' for k, v in o.items() if v)
    out.with_suffix('.key.txt').write_text(head + '\n' + ''.join(f'{k + 1}: {e}\n' for k, e in enumerate(resolved)) + '\n'.join(notes) + '\n' + r.stdout + '\n'.join(report) + '\n', encoding='utf-8')
    print(out, im.size)
    if '--quiet' not in argv: print(r.stdout.strip())
    for x in notes + report: print(x)
if __name__ == '__main__':
    main()
