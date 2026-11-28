"""heldout_draw.py SEED: choose the six held-out panels for the second blind test by a seeded random draw.

Pool: modal panels exact against TSPS (ifref.exact_pool), minus every panel seen while tuning (148, 648, 383, 303,
68, 210, 912, 476). Each pool panel is put into kind buckets AUTOMATICALLY from counts of its classify() regions
(no picture is drawn or looked at; only these counts are printed). Then, bucket by bucket in a fixed order, one
panel is drawn with random.Random(SEED) among the bucket's panels not yet chosen. Buckets:
  titled-inner : a titled frame plus at least one inner framed box (steel/thin/bevel/two-tone/sprite-built)
  table-list   : rule lines, or >= 6 texts with at least 2 rows holding >= 2 texts each
  tabs-buttons : at least one button region
  hud-overlay  : no titled frame, see-through pixels, drawn area >= 2000 px (an overlay drawn in the modal)
  item-grid    : an item grid or >= 4 item slots
  dialogue     : body text (> 40 characters) or a stone frame
If a bucket is empty after earlier draws, the draw falls back to any remaining pool panel and says so.
Writes recipes/heldout.json. The chosen six are then EXCLUDED from every recipe census (recipes/census.py)."""
import sys, json, random, collections
from pathlib import Path
HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import ifref as R

SEEN = {148, 648, 383, 303, 68, 210, 912, 476}
ORDER = ['dialogue', 'hud-overlay', 'item-grid', 'tabs-buttons', 'table-list', 'titled-inner']


def features(g):
    S = R.real_scene(g)
    regions, texts = R.classify(S)
    fam = collections.Counter(r['family'] for r in regions)
    titled = sum(1 for r in regions if r['family'] in ('steel', 'stone', 'stone2') and r.get('divider') is not None)
    inner = sum(v for k, v in fam.items() if k in ('steel', 'thin', 'bevel-dark', 'bevel-light', 'two-tone',
                                                    'sprite-built', 'stone', 'stone2')) - titled
    buttons = sum(1 for r in regions if r.get('button'))
    body = sum(1 for t in texts if t['role'] == 'body')
    rows = collections.Counter(t['ink'][1] // 4 for t in texts)
    cols = collections.Counter(t['ink'][0] // 4 for t in texts)
    tabular = len(texts) >= 6 and sum(1 for v in rows.values() if v >= 2) >= 2
    a, b, _ = S.images()
    pr = S.panel_rect()
    x, y, w, h = pr
    import numpy as np
    through = int(((a[y:y + h, x:x + w] == np.array([0x29, 0x25, 0x1f])).all(-1) &
                   (b[y:y + h, x:x + w] == np.array([0, 255, 0])).all(-1)).sum())
    items = fam.get('item', 0) + sum(len(r['members']) for r in regions if r['family'] == 'grid')
    return dict(group=g, name=R.if_name(g), size=pr[2:], families=dict(fam), titled=titled, inner=inner,
                buttons=buttons, texts=len(texts), body=body, tabular=tabular, lines=fam.get('line', 0),
                see_through=through, items=items, stone=fam.get('stone', 0) + fam.get('stone2', 0))


def buckets(f):
    b = set()
    if f['titled'] and f['inner'] >= 1:
        b.add('titled-inner')
    if f['lines'] or f['tabular']:
        b.add('table-list')
    if f['buttons']:
        b.add('tabs-buttons')
    if not f['titled'] and f['see_through'] > 0 and f['size'][0] * f['size'][1] >= 2000:
        b.add('hud-overlay')
    if f['items'] >= 4 or 'grid' in f['families']:
        b.add('item-grid')
    if f['body'] or f['stone']:
        b.add('dialogue')
    return b


def main():
    seed = int(sys.argv[1])
    pool = [g for g in R.exact_pool() if g not in SEEN]
    feats = {g: features(g) for g in pool}
    for g in pool:
        f = feats[g]
        print(g, f['name'], f['size'], sorted(buckets(f)), {k: f[k] for k in ('titled', 'inner', 'buttons', 'texts',
                                                                                 'body', 'lines', 'items', 'see_through')})
    rng = random.Random(seed)
    chosen = []
    for b in ORDER:
        cands = sorted(g for g in pool if b in buckets(feats[g]) and g not in [c['group'] for c in chosen])
        note = ''
        if not cands:
            cands = sorted(g for g in pool if g not in [c['group'] for c in chosen])
            note = 'bucket empty: drawn from the whole remaining pool'
        g = rng.choice(cands)
        chosen.append(dict(group=g, name=feats[g]['name'], bucket=b, candidates=cands, note=note,
                           size=feats[g]['size']))
        print(f'{b}: {g} {feats[g]["name"]} from {cands} {note}')
    (HERE / 'recipes' / 'heldout.json').write_text(json.dumps(dict(seed=seed, pool=pool, seen=sorted(SEEN),
                                                                   chosen=chosen), indent=1))


if __name__ == '__main__':
    main()
