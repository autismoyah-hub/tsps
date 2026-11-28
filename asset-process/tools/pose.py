"""Real items and our models on a real figure at chosen frames of a real animation, posed by the client's own code.

usage: pose.py OUT.png "ENTRY,ENTRY,..." [--seq ID] [--frames F1,F2,...] [--zoom N] [--yaw N] [--pitch N]
               [--cell WxH] [--under SEQ:FRAME] [--center body|model] [--aim head|body|legs|feet|hands|HEIGHT]
               [--overlap A:B[,A:C...]] [--diff A:B[,A:C...]] [--tone A:B[,A:C...]] [--keep-bmp]
               [--draw actor|depth|auto|widget]
One column per entry, one row per frame; a frame of `bind` (or no --seq) is the untouched rest pose.

ENTRY = PIECE[+PIECE...]      every piece is worn by the SAME figure (an outfit), applied left to right
PIECE = none                        the bare figure
      | item:ID                     a real item in the slot its definition declares, hiding the slots it declares
                                    (an item that declares no slot goes in the weapon slot, the old meaning)
      | item:ID@SLOT                the item in SLOT, still hiding the slots its definition declares
      | item:ID@SLOT+HIDE[+HIDE]    the item in SLOT, hiding exactly the listed slots
      | item:ID@SLOT+nohide         the item in SLOT, hiding nothing else
      | model:PATH                  one of ours (authored in worn space) merged after the body; hides nothing
      | model:PATH@SLOT[+HIDE...]   one of ours standing in SLOT; the body kit of SLOT and of each HIDE is removed
After an `@`, a `+word` that is a slot belongs to that piece; `+item:`, `+model:`, `+none` start the next piece.
(So a model path may not contain `+` or `,`, and `@` only before the slot.)
  "item:1127"                                    Rune platebody: body slot, arms hidden (from its definition)
  "model:path/to/plate.bin@body+arms"               ours in its place: torso and arms kits removed
  "item:1127@body+item:1079@legs+model:path/to/helm.bin@head+hair+jaw"     one figure wearing all three

Equipment slots, by index or name (the client's numbering: player_model.rs `equipment: [u32; 12]`;
KIT_SLOTS = [8, 11, 4, 6, 9, 7, 10] places the body kits hair, jaw, torso, arms, hands, legs, feet;
Appearance::sequence_equipment writes a sequence's shield to 5 and weapon to 3):
   0 head    1 cape    2 amulet   3 weapon   4 body (torso)   5 shield
   6 arms    7 legs    8 hair     9 hands   10 feet           11 jaw (beard)
Slots 4, 6, 7, 8, 9, 10, 11 hold a body kit on the bare figure; 0, 1, 2, 3, 5 are empty.

How the client hides the body under armour: there is no geometric test. Each slot holds ONE value (nothing,
a body kit, or an item), so an item in slot 4 takes the torso kit's place. The item definition carries three
wear positions (item.rs opcodes 13, 14, 27): the first is the slot it is worn in, the second and third are
further slots blanked while it is worn (widget_appearance.rs Composition::equip; for other players the server
sends the twelve values already blanked). In this cache: Rune platebody [4, 6, -1] (blanks arms), Rune full
helm [0, 8, 11] (blanks hair and jaw), Rune platelegs [7, -1, -1]. Whatever our model replaces it must redraw:
a body piece written @body+arms must include the arms, a helm written @head+hair+jaw must cover the whole head.
A model in a slot is combined in slot order and receives the player's skin and clothes recolours like a real
item (player_colors.rs BASE: face colours 6798 hair, 8741 torso, 25238 legs, 4626 feet, 4550 skin are replaced
by the figure's own; use them on purpose for bare skin, avoid them otherwise).

--draw chooses the routine that paints the figure. The default changed on 2026-11-25 from depth to actor.
  actor  (default) as the live client paints a PLAYER in the world (world_actors.rs place_actor with
         model_order::Mode::Priority; render3d.rs draw_model_phase). All faces of the whole figure (body and every
         worn piece are one model) are painted in order of face priority 0..9, far to near inside a priority,
         with priorities 10 and 11 slotted in by depth. There is NO depth test between the figure's own faces:
         a later face paints over an earlier one wherever both are drawn. So a strip lying exactly in a surface
         shows whole if its priority is higher and is completely hidden if lower, and a higher-priority face
         also shows THROUGH a lower-priority face in front of it (back faces are still culled). Per-face depth
         offsets (face_flags) do not change which of the figure's own faces shows.
  depth  the old picture (item_icon.rs draw_models): per-pixel depth test between all faces, priorities
         ignored, depth offsets honoured. Coplanar faces flicker with pitch and zoom in this mode; the live
         client never paints a player like this.
  auto   as the live client paints scenery and ordinary NPCs (Mode::Auto): per-pixel depth, faces in model
         order, priorities ignored, depth offsets honoured. Use it to preview a model meant for an NPC.
  widget as the client's interface paints a player (equipment screen; item_icon.rs draw_widget_models):
         priority order, no depth buffer at all.
--yaw is the view angle in the client's 2048-unit circle (default 192, the three-quarter front view).
--under SEQ:FRAME plays a locomotion sequence beneath the action, as the client blends walking under an attack.
--center model frames each cell on its own mesh as figure.py does (default: all cells share the bare figure's
  camera, so cells register pixel for pixel).
--aim points the camera at a part of the figure instead of its middle (height 98): a height in worn-space units
  (feet 0, crown 196) or a name: head = 180, body = 140, hands = 100, legs = 65, feet = 10. All columns and rows
  still share the one camera, so --diff, --overlap and --tone work as before. Use it with a small --zoom for a
  close view of a helm or boots, which otherwise leave the cell (--zoom alone keeps aiming at the body):
     pose.py helm.png "item:1163,model:path/to/helm.bin@head+hair+jaw,none" --aim head --zoom 130 --cell 400x400
     pose.py boots.png "item:11840,model:path/to/boots.bin@feet,none" --aim feet --zoom 130 --cell 400x400
  The zoom is the camera's distance in units: a unit is about 512 / zoom pixels (1 at the default 520, 4 at
  130). A zoom under 400 gives 600 x 800 cells unless --cell says otherwise; the aimed-at height is the middle
  of the cell, so a small --cell crops to the part. Measured: zoom 110 still draws cleanly; at 90 faces near
  the camera are cut away (holes in the helm and boots). The closer the camera, the stronger the perspective.
--overlap A:B measures, per row, how well the worn pixels of column B coincide with those of column A
  (1-based column numbers). Worn pixels are those differing from a `none` column, which must be among the
  entries. Prints pixel counts, intersection over union, and the distance between the two centroids.
--diff A:B measures, per row, the raw pixel difference between two columns (taken before labels are drawn):
  the number of differing pixels, their share of the cell, and the largest channel difference. 0 = identical.
  It counts ANY difference, so it does not move between a wrong tone and a nearly right one; for that use --tone.
--tone A:B measures, per row, how far apart two columns are in TONE. It needs a `none` column, like --overlap.
  The pixels measured are those where column A or column B differs from the `none` column (the worn piece in
  either column; the bare figure and the background are left out). Over them it prints
     the count of such pixels,
     the mean absolute grey difference (grey = (r + g + b) / 3, 0 to 255): 0 = the same picture,
     the signed mean difference, B minus A: positive = column B is lighter than column A, negative = darker,
     and how many of the pixels differ at all.
  A right shape in a slightly wrong tone gives a small mean with a signed mean of nearly the same size; a wrong
  shape gives a large mean with a signed mean near 0. Follow the signed mean to 0 while tuning paint.
  Measured scale: a real item against its re-encoded copy gives 0.00; against a copy with every face painted
  ONE lightness step darker it gives 1.0 (Rune full helm), 1.6 (Iron chainbody), 0.45 (Dragon boots, dark
  paint), signed mean the same size and negative, while --diff reports thousands of differing pixels.
Writes OUT.png and OUT.key.txt. The example also prints the worn pieces' vertex labels and bounding box per cell.
`asset_pose.exe CACHE info item:ID` prints an item's wear positions, its worn and head model ids for both body
types, and for every worn model the vertex count, face count and histograms of vertex labels, face labels,
priorities and colours. tools/reencode_item.py exports a real item's worn mesh through our encoder.

Sequences (all are classic frame animations on skeleton 0, the player's; checked with
`asset_pose.exe CACHE info ID...`, which also accepts item:ID):
  808 stand (12 frames)   819 walk (8)   824 run (8)       the player defaults, as in the client's own examples
  386 one-handed stab (6)   390 one-handed slash (10)       sword and scimitar attacks
  9471 Osmumten's fang stab (14)
  813 stand, 1205 walk, 428 stab (6), 440 swipe (13)        two-handed spear and polearm set
  809 stand, 1146 walk, 419 pound (11)                      staff set (halberds share 809/1146)
Item definitions in the cache hold no stance or attack ids (the server sends them with the player's
appearance), and the client source only names the defaults 808/823/819/820/821/822/824; the weapon-class ids
above are the usual OSRS ones, confirmed present in this cache and judged by reading the renders.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib
from PIL import Image, ImageDraw, ImageFont
EXE = _AP.fix(r'<CLIENT_BIN>\asset_pose.exe')
CACHE = _AP.fix(r'<OSRS_CACHE_239>')
argv = sys.argv
def opt(name):
    return argv[argv.index(name) + 1] if name in argv else None
out = pathlib.Path(argv[1]); entries = argv[2].split(',')
frames = (opt('--frames') or ('0' if opt('--seq') else 'bind')).split(',')
extra = [f'{k}={opt("--" + k)}' for k in ('seq', 'frames', 'zoom', 'yaw', 'pitch', 'cell', 'under', 'center', 'aim', 'draw') if opt('--' + k)]
bmp = out.with_suffix('.bmp')
out.parent.mkdir(parents=True, exist_ok=True)
r = subprocess.run([EXE, CACHE, str(bmp), *extra, *entries], capture_output=True, text=True)
if r.returncode:
    raise SystemExit(r.stdout + r.stderr)
im = Image.open(bmp).convert('RGB')
if '--keep-bmp' not in argv:
    bmp.unlink()
cw, ch = im.width // len(entries), im.height // len(frames)
report = []
if opt('--overlap'):
    if 'none' not in entries:
        raise SystemExit('--overlap needs a `none` entry to mask against')
    px = im.load(); base = entries.index('none')
    def mask(col, row):
        return {(x, y) for y in range(ch) for x in range(cw)
                if px[col * cw + x, row * ch + y] != px[base * cw + x, row * ch + y]}
    def centre(m):
        return (sum(p[0] for p in m) / len(m), sum(p[1] for p in m) / len(m)) if m else (0.0, 0.0)
    for pair in opt('--overlap').split(','):
        a, b = (int(v) - 1 for v in pair.split(':'))
        for row, f in enumerate(frames):
            ma, mb = mask(a, row), mask(b, row)
            inter, union = len(ma & mb), len(ma | mb)
            ca, cb = centre(ma), centre(mb)
            dist = ((ca[0] - cb[0]) ** 2 + (ca[1] - cb[1]) ** 2) ** 0.5
            report.append(f'overlap col {a + 1} vs col {b + 1} frame {f}: pixels {len(ma)} / {len(mb)}, shared {inter}, '
                          f'IoU {inter / union if union else 0:.3f}, of col {a + 1} covered {inter / len(ma) if ma else 0:.3f}, '
                          f'centroid distance {dist:.1f} px')
if opt('--diff'):
    px = im.load(); background = px[0, 0]
    for pair in opt('--diff').split(','):
        a, b = (int(v) - 1 for v in pair.split(':'))
        for row, f in enumerate(frames):
            n = worst = drawn = 0
            for y in range(ch):
                for x in range(cw):
                    p, q = px[a * cw + x, row * ch + y], px[b * cw + x, row * ch + y]
                    drawn += p != background
                    if p != q:
                        n += 1; worst = max(worst, *(abs(u - v) for u, v in zip(p, q)))
            report.append(f'diff col {a + 1} vs col {b + 1} frame {f}: {n} of {cw * ch} pixels differ '
                          f'({drawn} figure pixels in col {a + 1}), largest channel difference {worst}')
if opt('--tone'):
    if 'none' not in entries:
        raise SystemExit('--tone needs a `none` entry: the pixels measured are those differing from it')
    px = im.load(); base = entries.index('none')
    grey = lambda p: (p[0] + p[1] + p[2]) / 3
    for pair in opt('--tone').split(','):
        a, b = (int(v) - 1 for v in pair.split(':'))
        for row, f in enumerate(frames):
            n = changed = 0; total = signed = 0.0
            for y in range(ch):
                for x in range(cw):
                    p, q, o = px[a * cw + x, row * ch + y], px[b * cw + x, row * ch + y], px[base * cw + x, row * ch + y]
                    if p != o or q != o:
                        n += 1; changed += p != q
                        dd = grey(q) - grey(p); signed += dd; total += abs(dd)
            report.append(f'tone col {a + 1} vs col {b + 1} frame {f}: {n} worn pixels, mean absolute grey difference '
                          f'{total / n if n else 0:.2f}, signed mean (col {b + 1} minus col {a + 1}) {signed / n if n else 0:+.2f} '
                          f'of 255; {changed} of them differ')
d = ImageDraw.Draw(im)
try: font = ImageFont.truetype('segoeuib.ttf', 30); small = ImageFont.truetype('segoeuib.ttf', 16)
except Exception: font = small = ImageFont.load_default()
for k in range(len(entries)): d.text((k * cw + 12, 6), str(k + 1), fill=(240, 240, 240), font=font)
for row, f in enumerate(frames):
    d.text((6, row * ch + ch - 24), f'frame {f}', fill=(200, 200, 160), font=small)
    if row: d.line([(0, row * ch), (im.width, row * ch)], fill=(70, 64, 56))
im.save(out)
head = f'seq {opt("--seq")} frames {",".join(frames)} yaw {opt("--yaw") or 192} zoom {opt("--zoom") or 520} draw {opt("--draw") or "actor"}' + (f' under {opt("--under")}' if opt('--under') else '') + (f' aim {opt("--aim")}' if opt('--aim') else '')
out.with_suffix('.key.txt').write_text(head + '\n' + ''.join(f'{k + 1}: {e}\n' for k, e in enumerate(entries)) + ''.join(x + '\n' for x in report))
print(out, im.size); print(r.stdout.strip())
for x in report: print(x)
