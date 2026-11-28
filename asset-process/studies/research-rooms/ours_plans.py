"""Plans of our rooms in the study's plan style: ours_plans.py OUT.png CSV:TITLE ..."""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from render import plan
import roomcheck as C
from PIL import Image

ims = []
for arg in sys.argv[2:]:
    path, title = arg.split(':', 1)
    ims.append(plan(C.load(path), pathlib.Path(path).with_suffix('.plan.png'), title, cell=26))
W = sum(i.size[0] for i in ims) + 10 * len(ims); H = max(i.size[1] for i in ims)
sheet = Image.new('RGB', (W, H), (20, 20, 22)); x = 0
for i in ims: sheet.paste(i, (x, 0)); x += i.size[0] + 10
sheet.save(sys.argv[1]); print(sys.argv[1])
