"""Print the eight automatic zones of a creature: share of skin, marks, motifs. usage: zones.py ENTRY"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import paintlib as pl
for e in sys.argv[1:]:
    a = pl.analyse(pl.skin(e)); n = max(a['m']['marks'], 1)
    print(e[-50:], 'marks', a['m']['marks'])
    for r in a['parts']:
        if r.get('share', 0) > 0: print(f"   {r['name']:22s} skin {100 * r['share']:4.1f}%  marks {r['marks']:3d} = {100 * r['marks'] / n:4.1f}%  boundary density {r['boundary_density']:.2f}  " + ' '.join(f'{k}:{v}' for k, v in r['motifs'].items() if v))
