"""Measure every found room, choose the study set, and summarise the ranges by kind.

usage: study.py
writes  measures-all.csv     every distinct candidate (copies and house-building templates dropped)
        study.json           the chosen study rooms (normalised, with their measures)
        measures-study.csv   the study set
        ranges.json          per kind (and 'all', 'tower', 'recent', 'classic', 'well', 'bare'): p10 p25 median p75 p90 per measure
        objects-study.csv    every object in every study room: room, id, name, class, function, x, y, size, rotation, shape
Selection: per kind, rooms in a named place first, then by the strength of the kind's vote, at most 3 from one place
per kind, so the set spreads across the map; quotas below. Towers are chosen by plan (see classify.py), any kind.
"""
import sys, json, csv, collections, statistics, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rlib as R
import roommeasure as M
from real import normalize

QUOTA = dict(armoury=16, library=10, workshop=7, smithy=6, kitchen=10, bedroom=10, throne=6, shop=6, bank=4,
             temple=8, laboratory=6, tavern=6)
TOWERS = 10
NUM = [k for k in M.NAMES]


def pct(v, q):
    v = sorted(v)
    if not v: return None
    i = (len(v) - 1) * q
    lo = int(i); hi = min(lo + 1, len(v) - 1)
    return v[lo] + (v[hi] - v[lo]) * (i - lo)


def summarise(ms):
    out = {'n': len(ms)}
    for k in NUM:
        v = [m[k] for m in ms if m.get(k) is not None]
        if v:
            out[k] = dict(p10=pct(v, .1), p25=pct(v, .25), med=pct(v, .5), p75=pct(v, .75), p90=pct(v, .9), n=len(v))
    return out


def richness(m):
    """A single furnishing score used only to sort rooms for 'well furnished' and 'bare' examples."""
    return m['per10'] + 3 * m['decor_per10w'] / 10 + m['variety_all'] / 4 + 2 * m['floor_cover']


def main():
    rooms = json.loads((R.HERE / 'rooms.json').read_text())
    cand = [r for r in rooms if r['dup_of'] is None and r['kind'] != 'poh-template']
    allrows = []
    for r in cand:
        n = normalize(r)
        m = M.measures(n)
        r['norm'] = n; r['m'] = m
        allrows.append(r)
    cols = ['id', 'kind', 'tower', 'place', 'era', 'x', 'y', 'plane'] + NUM
    def write(path, rs):
        with open(path, 'w', newline='') as f:
            w = csv.writer(f); w.writerow(cols)
            for r in rs:
                w.writerow([r['id'], r['kind'], int(r['tower']), r['place'], r['era'], r['centre'][0], r['centre'][1], r['plane']] +
                           [('' if r['m'].get(k) is None else round(r['m'][k], 3)) for k in NUM])
    write(R.HERE / 'measures-all.csv', allrows)
    # selection
    chosen = []
    for kind, q in QUOTA.items():
        pool = [r for r in allrows if r['kind'] == kind and r['m']['n_furn'] + r['m']['n_decor'] > 0]
        pool.sort(key=lambda r: (r['place'] in ('elsewhere', 'underground/instance'), -r['votes'].get(kind, 0), r['id']))
        per = collections.Counter(); got = []
        for r in pool:
            if per[r['place']] >= 3: continue
            per[r['place']] += 1; got.append(r)
            if len(got) == q: break
        chosen += got
    ids = {r['id'] for r in chosen}
    tw = [r for r in allrows if r['tower'] and r['id'] not in ids and r['m']['n_furn'] >= 2]
    tw.sort(key=lambda r: (r['place'] in ('elsewhere', 'underground/instance'), -r['m']['n_furn'], r['id']))
    per = collections.Counter()
    for r in tw:
        if per[r['place']] >= 2: continue
        per[r['place']] += 1; chosen.append(r)
        if sum(1 for c in chosen if c['tower']) >= TOWERS: break
    write(R.HERE / 'measures-study.csv', chosen)
    with open(R.HERE / 'objects-study.csv', 'w', newline='') as f:
        w = csv.writer(f); w.writerow(['room', 'kind', 'place', 'world_x', 'world_y', 'plane', 'id', 'name', 'class', 'function', 'size_w', 'size_d', 'rotation', 'shape', 'height', 'faces_in'])
        for r in chosen:
            bx, by = r['base']
            for o in r['norm']['objects']:
                if o['cls'] in ('wall',): continue
                w.writerow([r['id'], r['kind'], r['place'], bx + o['x'], by + o['y'], r['plane'], o['id'], o['name'], o['cls'], o['func'], o['w'], o['d'], o['rot'], o['shape'], o['h'], int(o['inside'])])
    # ranges
    ranges = {}
    groups = collections.defaultdict(list)
    for r in chosen:
        groups[r['kind']].append(r['m']); groups['all'].append(r['m'])
        if r['tower']: groups['tower'].append(r['m'])
        if r['era'] in ('recent', 'classic'): groups[r['era']].append(r['m'])
    srt = sorted(chosen, key=lambda r: richness(r['m']))
    q = max(1, len(srt) // 4)
    groups['bare'] = [r['m'] for r in srt[:q]]; groups['well'] = [r['m'] for r in srt[-q:]]
    for k, ms in groups.items(): ranges[k] = summarise(ms)
    pop = collections.defaultdict(list)
    for r in allrows:
        pop['pop-' + r['kind']].append(r['m']); pop['pop-all'].append(r['m'])
    for k, ms in pop.items(): ranges[k] = summarise(ms)
    # pairs: lift of touching function pairs over chance, study set
    pc = collections.Counter(); fc = collections.Counter()
    for r in chosen:
        for p in r['m']['pair_list']: pc[p] += 1
        for o in r['norm']['objects']:
            if o['cls'] in ('furniture', 'light'): fc[o['func']] += 1
    tot = sum(pc.values()); n = sum(fc.values())
    pairs = []
    for (a, b), c in pc.most_common():
        exp = tot * (fc[a] * fc[b] * (1 if a != b else 0.5) * 2) / (n * n)
        pairs.append(dict(a=a, b=b, count=c, lift=round(c / exp, 2) if exp else None))
    ranges['_pairs'] = pairs
    ranges['_well_ids'] = [r['id'] for r in srt[-q:]]; ranges['_bare_ids'] = [r['id'] for r in srt[:q]]
    (R.HERE / 'ranges.json').write_text(json.dumps(ranges, indent=1))
    out = []
    for r in chosen:
        out.append({k: r[k] for k in ('id', 'kind', 'tower', 'place', 'era', 'centre', 'plane', 'base', 'w', 'h', 'area', 'square')} | dict(norm=r['norm'], m=r['m']))
    (R.HERE / 'study.json').write_text(json.dumps(out))
    c = collections.Counter(r['kind'] for r in chosen)
    print('candidates measured', len(allrows), '; study set', len(chosen), dict(c), 'towers', sum(r['tower'] for r in chosen),
          'eras', dict(collections.Counter(r['era'] for r in chosen)))


if __name__ == '__main__':
    main()
