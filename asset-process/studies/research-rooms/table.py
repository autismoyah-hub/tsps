"""Print the ranges (p25 / median / p75, and p10..p90) of the main measures for each group in ranges.json.

usage: table.py [GROUP ...]    (default: every kind plus all, well, bare, tower, recent, classic, pop-all)
"""
import sys, json, pathlib
HERE = pathlib.Path(__file__).resolve().parent
R = json.loads((HERE / 'ranges.json').read_text())
KEYS = ['area', 'n_furn', 'per10', 'furn_per10', 'occ_share', 'wall_share', 'grouped_share', 'mean_group', 'n_decor', 'decor_per10w',
        'decor_edge_share', 'wall_lined', 'bare_run', 'n_trim', 'floor_cover', 'n_floor_named', 'n_lights', 'n_clutter', 'variety', 'variety_all',
        'max_repeat', 'repeat_share', 'rows', 'sym_axis', 'sym_id', 'faces_door', 'reach1', 'reach2', 'free_share', 'centre_occ',
        'centre_floor', 'corner_use']
groups = sys.argv[1:] or ['all', 'well', 'bare', 'armoury', 'library', 'workshop', 'smithy', 'kitchen', 'bedroom', 'throne', 'shop',
                          'bank', 'temple', 'laboratory', 'tavern', 'tower', 'recent', 'classic', 'pop-all']
f = lambda v: '   -' if v is None else (f'{v:4.0f}' if abs(v) >= 10 else f'{v:4.2f}')
print('measure'.ljust(16) + ''.join(g[:9].rjust(16) for g in groups))
print(''.ljust(16) + ''.join(('n=' + str(R[g]['n'])).rjust(16) for g in groups if g in R))
for k in KEYS:
    row = k.ljust(16)
    for g in groups:
        s = R.get(g, {}).get(k)
        row += ('   -' if not s else f"{f(s['p25'])}/{f(s['med'])}/{f(s['p75'])}").rjust(16)
    print(row)
