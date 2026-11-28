"""debug_family.py FAMILY GROUP: the instances of a family in one group: pieces relative to the box, with keys."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import common as K
import census_frames as F

name, g = sys.argv[1], int(sys.argv[2])
rows, by = K.load(g)
for inst in F.find_instances(rows, by, F.FAMILIES[name]):
    print('box', inst['box'], 'parent', inst['parent'] & 0xffff)
    for r in inst['members']:
        print('   ', r['sprites'][0], F.rel(inst, r), 'tiled', int(r['sprite_tiling']), 'tr', r['transparency'],
              'flip', int(r['flip_h']), int(r['flip_v']), 'order', r['order'])
