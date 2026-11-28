"""fpose.py: tools/pose.py with two more options, on the research copy of the pose example (research_pose.exe).

usage: fpose.py OUT.png "ENTRY,ENTRY,..." [--gender 0|1] [--head 1] [every option of tools/pose.py]
  --gender 1   the standard FEMALE figure (Appearance::standard(kits, 1): the lowest-numbered enabled kit of body
               parts 7..13) with the same colours as the male harness figure; item:ID entries load the item's
               FEMALE worn models and female offset (item_model.rs wearable, gender 1). model:PATH@SLOT entries
               are used as given, so hand it the female file.
  --head 1     the chat-head mesh instead of the body: kits' head models, items' head models (male or female by
               --gender), lit 64 / 768 from (-50, -10, -50) as interface_render.rs lights a chat head. Use with
               --draw widget --yaw 1882 --pitch 40 --zoom 796 (see chathead.py, which does this).
  --jaw 0      empty the jaw (beard) slot: Appearance::standard gives BOTH body types a goatee (female kit 292).
With --gender 0 and no --head the picture is byte-identical to tools/pose.py (checked: cmp of the two BMPs).
Everything else (entries, --seq, --frames, --diff, --overlap, --tone, --aim, ...) is tools/pose.py; read its header.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib
from PIL import Image, ImageDraw, ImageFont
EXE = _AP.fix(r'<CLIENT_BIN>\research_pose.exe')
CACHE = _AP.fix(r'<OSRS_CACHE_239>')
argv = sys.argv
def opt(name):
    return argv[argv.index(name) + 1] if name in argv else None
out = pathlib.Path(argv[1]); entries = argv[2].split(',')
frames = (opt('--frames') or ('0' if opt('--seq') else 'bind')).split(',')
extra = [f'{k}={opt("--" + k)}' for k in ('seq', 'frames', 'zoom', 'yaw', 'pitch', 'cell', 'under', 'center', 'aim', 'draw', 'gender', 'head', 'jaw') if opt('--' + k)]
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
head = f'seq {opt("--seq")} frames {",".join(frames)} yaw {opt("--yaw") or 192} zoom {opt("--zoom") or 520} draw {opt("--draw") or "actor"} gender {opt("--gender") or 0}' + (f' under {opt("--under")}' if opt('--under') else '') + (f' aim {opt("--aim")}' if opt('--aim') else '')
out.with_suffix('.key.txt').write_text(head + '\n' + ''.join(f'{k + 1}: {e}\n' for k, e in enumerate(entries)) + ''.join(x + '\n' for x in report))
print(out, im.size); print(r.stdout.strip())
for x in report: print(x)
