"""look.py: client pictures of C, A, Meridian v5, Torva (helm in outfit, head aimed) -> work/look-*.png, for looking."""
import sys, pathlib
sys.dont_write_bytecode = True
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import h3
from h3 import h2
KEYS = ['umbralC', 'umbralA', 'v5', 'torva']
if __name__ == '__main__':
    es = [h2.entry(k) for k in KEYS]
    for zn, zoom, cell in (('g', 300, (170, 170)), ('c', 130, (320, 340)), ('x', 80, (480, 500))):
        for yaw in (0, 256, 512, 1024):
            h2.draw(f'look-{zn}-{yaw}', es, yaw=yaw, zoom=zoom, cell=cell, aim='head')
