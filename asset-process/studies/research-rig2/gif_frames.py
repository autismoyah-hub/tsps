"""Every frame of a moving picture laid out as one sheet, so it can be READ frame by frame.
usage: gif_frames.py IN.gif OUT.png [--cols 6] [--first N] [--every K] [--scale S] [--from F]
Frames are numbered in the corner (frame index; rig_show.py's moving pictures step 3 cycles a frame)."""
import sys
from PIL import Image, ImageDraw, ImageSequence
a = sys.argv
def opt(n, d): return type(d)(a[a.index(n) + 1]) if n in a else d
im = Image.open(a[1]); frames = [f.convert('RGB') for f in ImageSequence.Iterator(im)]
frames = frames[opt('--from', 0):][::opt('--every', 1)][:opt('--first', 9999)]; s = opt('--scale', 1.0)
if s != 1.0: frames = [f.resize((int(f.width * s), int(f.height * s)), Image.LANCZOS) for f in frames]
cols = opt('--cols', 6); rows = -(-len(frames) // cols); w, h = frames[0].size
out = Image.new('RGB', (cols * w, rows * h), (0, 0, 0)); d = ImageDraw.Draw(out)
for k, f in enumerate(frames):
    x, y = (k % cols) * w, (k // cols) * h; out.paste(f, (x, y)); d.text((x + 3, y + h - 12), str(opt('--from', 0) + k * opt('--every', 1)), fill=(255, 255, 0))
out.save(a[2]); print(a[2], out.size, len(frames), 'frames')
