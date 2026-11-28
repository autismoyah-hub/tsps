"""Feasibility probe (no study measure): names of the chosen ids from the census, and whether each entry loads,
its face count, number of distinct vertex labels and whether it is bone-weighted. Run before PREREG is final."""
import sys, pathlib, json
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gaplib as g

if __name__ == '__main__':
    cen = g.cl.census()
    for k, e in g.ALL.items():
        try:
            c = g.load(e)
            nl = len(set(c['L'].tolist()))
            print(f"{k:28s} {c['name'][:30]:30s} faces {int(c['drawn'].sum()):5d} labels {nl:4d} transl {int(((c['A']>0)&(c['A']<255)).sum()):4d}")
        except SystemExit as ex:
            print(k, 'FAILED', str(ex)[:200])
