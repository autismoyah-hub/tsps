"""Construction census: for every creature of the sample, how its small contrasting features are built.
-> data/construct.json and a printed table."""
import sys, pathlib, json, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import cleanlib as cl, features as ft
from explore import all_entries
HERE = pathlib.Path(__file__).resolve().parent
KINDS = ['paint', 'inset', 'relief', 'piece-open', 'piece-closed', 'floating']

def summary(c):
    feats, hgt = ft.find(c)
    skin = c['drawn'] & (c['A'] == 0); nf = int(skin.sum())
    small = [f for f in feats if f['size_h'] <= 0.07 and f['contrast'] >= 20]
    out = dict(features=len(feats), small=len(small))
    cnt = collections.Counter(f['construction'] for f in small)
    for k in KINDS: out['small_' + k] = cnt.get(k, 0)
    shells = [f for f in feats if f['kind'] == 'shell']
    closed = [f for f in shells if f['closed'] and f['touching']]
    out['pieces'] = len(shells); out['pieces_closed'] = len(closed); out['pieces_open'] = sum(1 for f in shells if not f['closed'] and f['touching']); out['pieces_floating'] = sum(1 for f in shells if not f['touching'])
    out['closed_piece_faces'] = sum(f['faces'] for f in closed) / nf
    stuck = [f for f in closed if f['sunk'] >= 0.1]
    out['stuck_pieces'] = len(stuck); out['stuck_faces'] = sum(f['faces'] for f in stuck) / nf
    # small closed lumps: a closed piece no longer than 7% of the height (eye, nostril, stud scale)
    lumps = [f for f in closed if f['size_h'] <= 0.07]
    out['small_closed_lumps'] = len(lumps); out['small_closed_lump_faces'] = sum(f['faces'] for f in lumps) / nf
    out['faces_per_small_closed_lump'] = float(np.median([f['faces'] for f in lumps])) if lumps else 0.0
    out['small_closed_relief_h_max'] = max([f['relief_h'] for f in lumps], default=0.0)
    return out, feats

if __name__ == '__main__':
    rows = {}
    for key, entry, name, st in all_entries():
        c = cl.load(entry); r, _ = summary(c); r['name'] = name; r['style'] = st; rows[key] = r
        print('.', end='', file=sys.stderr, flush=True)
    (HERE / 'data' / 'construct.json').write_text(json.dumps(rows, indent=0))
    ks = ['small', 'small_paint', 'small_inset', 'small_relief', 'small_piece-open', 'small_piece-closed', 'small_floating', 'pieces', 'pieces_closed', 'stuck_pieces', 'stuck_faces', 'small_closed_lumps', 'small_closed_lump_faces', 'faces_per_small_closed_lump']
    print('\n' + f"{'creature':20s} st " + ' '.join(f'{k[-9:]:>9s}' for k in ks))
    for k, r in rows.items():
        print(f"{r['name'][:20]:20s} {r['style']}  " + ' '.join((f'{r[q]:9.3f}' if isinstance(r[q], float) else f'{r[q]:9d}') for q in ks))
