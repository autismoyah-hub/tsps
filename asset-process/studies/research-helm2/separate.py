"""separate.py: which measures put the ACCEPTED helms on one side and the REJECTED on the other?
usage: separate.py [--md OUT.md] [--show KEY,KEY..]
For every numeric measure in data/measures.json: the band of the accepted CORE (the owner's five + Meridian v5), the band
of the wider accepted group (plus five more real helms), and where each rejected helm falls. Verdicts:
  CLEAN   every rejected helm (recent AND earlier) outside the core band on the same side
  RECENT  both Umbral helms outside the core band on the same side (the earlier rejected ones not all outside)
  A-ONLY  Umbral A outside the core band (B inside)
  none    otherwise
`wide` says whether the verdict survives widening the accepted band to all eleven.
gap = distance from the nearest rejected helm (of the set the verdict names) to the band edge, as a share of the band width.
"""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import h2

D = json.load(open(h2.HERE / 'data' / 'measures.json'))
CORE = h2.CORE
WIDE = [k for k in h2.ORDER if h2.H[k]['group'] in ('ACC', 'ACC+') and k in D]
RR = ['umbralA', 'umbralB']; RE = ['v4a', 'v4b', 'v4c', 'v3a', 'v2a']


def side(v, lo, hi):
    return -1 if v < lo - 1e-9 else (1 if v > hi + 1e-9 else 0)


def judge(key, acc):
    vals = [D[k].get(key) for k in acc]
    if any(v is None or isinstance(v, str) for v in vals): return None
    lo, hi = min(vals), max(vals); w = max(hi - lo, 1e-9)
    def sides(ks):
        out = []
        for k in ks:
            v = D[k].get(key)
            out.append(None if v is None else side(v, lo, hi))
        return out
    sr, se = sides(RR), sides(RE)
    def gap(ks):
        g = []
        for k in ks:
            v = D[k].get(key)
            if v is None: continue
            g.append((lo - v) / w if v < lo else (v - hi) / w)
        return min(g) if g else 0
    if None in sr: return None
    verdict = 'none'; g = 0
    if sr[0] != 0 and sr[0] == sr[1] and None not in se and all(s == sr[0] for s in se):
        verdict = 'CLEAN'; g = gap(RR + RE)
    elif sr[0] != 0 and sr[0] == sr[1]:
        verdict = 'RECENT'; g = gap(RR)
    elif sr[0] != 0:
        verdict = 'A-ONLY'; g = gap(['umbralA'])
    return verdict, lo, hi, g, sr, se


def table():
    keys = sorted({k for d in D.values() for k, v in d.items() if isinstance(v, (int, float)) and not isinstance(v, bool)})
    rows = []
    for key in keys:
        j = judge(key, CORE)
        if not j: continue
        jw = judge(key, WIDE)
        rows.append((key, j, jw))
    return rows


if __name__ == '__main__':
    rows = table()
    order = {'CLEAN': 0, 'RECENT': 1, 'A-ONLY': 2, 'none': 3}
    rows.sort(key=lambda r: (order[r[1][0]], -r[1][3]))
    show = None
    if '--show' in sys.argv: show = sys.argv[sys.argv.index('--show') + 1].split(',')
    for key, j, jw in rows:
        if show and key not in show: continue
        if not show and j[0] == 'none' and '--all' not in sys.argv: continue
        v, lo, hi, g, sr, se = j
        wide = jw[0] if jw else '-'
        print(f'{v:7} wide:{wide:7} gap {g:5.2f}  {key:28} core {lo:g}..{hi:g}  wide {jw[1]:g}..{jw[2]:g}  A {D["umbralA"].get(key)}  B {D["umbralB"].get(key)}  v5 {D["v5"].get(key)} torva {D["torva"].get(key)} | RE ' + ' '.join(str(D[k].get(key)) for k in RE))
    n = collections = None
