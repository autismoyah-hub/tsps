"""Print a creature's palette: each colour's (hue, saturation, lightness), share of the drawn surface, face count.
usage: palette.py ENTRY [ENTRY...] [--min PERCENT]     ENTRY = npc:ID | def:PATH.json
"""
import sys, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import cmesh
def palette(entry):
    m = cmesh.load(entry); area, unit, cen = cmesh.face_geometry(m)
    keep = (m['A'] != 255) & (m['T'] < 2) & (area > 0)
    by = collections.Counter(); n = collections.Counter()
    for c, a in zip(m['C'][keep], area[keep]): by[int(c)] += a; n[int(c)] += 1
    tot = sum(by.values())
    return [(cmesh.hsl(c), a / tot, n[c], c) for c, a in by.most_common()]
if __name__ == '__main__':
    a = sys.argv[1:]; mn = float(a[a.index('--min') + 1]) if '--min' in a else 0.5
    for e in [x for x in a if ':' in x]:
        p = palette(e); print(e, f'{len(p)} colours')
        print('   ' + '  '.join(f'{k[0]},{k[1]},{k[2]}:{100 * s:.0f}%' for k, s, n, c in p if 100 * s >= mn))
