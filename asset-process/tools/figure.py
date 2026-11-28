"""Real items and our models on a real figure, drawn by the dedicated client build as the game lights worn gear.

usage: figure.py OUT.png "ENTRY,ENTRY,..." [--shuffle SEED] [--zoom N] [--draw actor|depth|auto|widget]
   (--zoom N below 400 is a close view; default 520)
One column per entry, three angles. Writes OUT.png and OUT.key.txt (which column is which; keep it from a judge).

ENTRY = PIECE[+PIECE...]      every piece is worn by the SAME figure (an outfit), applied left to right
PIECE = none                        the bare figure
      | item:ID                     a real item in the slot its definition declares, hiding the slots it declares
                                    (an item that declares no slot goes in the weapon slot, the old meaning)
      | item:ID@SLOT                the item in SLOT, still hiding the slots its definition declares
      | item:ID@SLOT+HIDE[+HIDE]    the item in SLOT, hiding exactly the listed slots (`+nohide`: none)
      | model:PATH                  one of ours (authored in worn space) merged after the body; hides nothing
      | model:PATH@SLOT[+HIDE...]   one of ours standing in SLOT; the body kit of SLOT and of each HIDE is removed
  "item:1127,model:path/to/plate.bin@body+arms,none"
  "item:1127@body+item:1079@legs+model:path/to/helm.bin@head+hair+jaw"     one figure wearing all three
SLOT (the client's numbering, player_model.rs `equipment: [u32; 12]`), by index or name:
   0 head  1 cape  2 amulet  3 weapon  4 body (torso)  5 shield  6 arms  7 legs  8 hair  9 hands  10 feet  11 jaw
The client hides the body by slot, not by geometry: an item takes the place of the body kit in its slot, and
its definition's second and third wear positions blank further slots (platebody: arms; full helm: hair, jaw).
--draw chooses how the figure is painted (default actor; pose.py explains all four and why it matters):
   actor  as the live client paints a player: by face priority, no depth test between the figure's own faces
   depth  the old picture (per-pixel depth between all faces, priorities ignored); pictures made before
          2026-11-25 were all drawn this way
See pose.py for the full account, the measuring options, and `asset_pose.exe CACHE info item:ID`.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib, random
from PIL import Image, ImageDraw, ImageFont
EXE = _AP.fix(r'<CLIENT_BIN>\asset_figure.exe')
CACHE = _AP.fix(r'<OSRS_CACHE_239>')
out = pathlib.Path(sys.argv[1]); entries = sys.argv[2].split(',')
if '--shuffle' in sys.argv:
    random.Random(int(sys.argv[sys.argv.index('--shuffle') + 1])).shuffle(entries)
bmp = out.with_suffix('.bmp')
extra = [f'{k}={sys.argv[sys.argv.index("--" + k) + 1]}' for k in ('zoom', 'draw') if '--' + k in sys.argv]
r = subprocess.run([EXE, CACHE, str(bmp), *entries, *extra], capture_output=True, text=True)
if r.returncode:
    raise SystemExit(r.stdout + r.stderr)
im = Image.open(bmp).convert('RGB'); bmp.unlink()
d = ImageDraw.Draw(im)
try: font = ImageFont.truetype('segoeuib.ttf', 30)
except Exception: font = ImageFont.load_default()
cw = im.width // len(entries)
for k in range(len(entries)): d.text((k * cw + 12, 6), str(k + 1), fill=(240, 240, 240), font=font)
im.save(out)
out.with_suffix('.key.txt').write_text(''.join(f'{k + 1}: {e}\n' for k, e in enumerate(entries)))
print(out, im.size); print(r.stdout.strip())
