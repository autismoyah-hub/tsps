"""gapcheck: our creature against the real 2019-on band on the measures of PREREG.md.

    P gapcheck.py MODEL_OR_NPC_JSON [--json OUT.json]
      MODEL_OR_NPC_JSON = an NPC definition JSON (as in builds/*), a build folder, a model .bin (scale 128, no light
      values), or an NPC id / npc:ID.
Exit 0 if the one surviving measure (C9 `concave`) is inside the real band's cut, 2 if not.

Only `concave` survived the pre-registered test (data/stats.json). The other 16 are printed as INFORMATION with the
REAL p10..p90: none separated our creatures from real ones, so being outside them says nothing proven. This is a
guard, not a judge: no creature built to it has been shown to the owner. Read with GAP.md's review questions.
"""
import sys, pathlib, json
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gaplib as g, measures as M
cl = g.cl

def entry_of(arg):
    p = pathlib.Path(arg)
    if p.suffix.lower() == '.bin' and p.exists():
        d = HERE / 'data' / 'gapcheck-defs'; d.mkdir(parents=True, exist_ok=True)
        j = d / (p.stem + '.json'); j.write_text(json.dumps({'name': p.stem, 'models': [str(p.resolve())], 'scale': [128, 128]}))
        return 'def:' + str(j)
    return cl.norm_entry(arg)

def main():
    if len(sys.argv) < 2: print(__doc__); raise SystemExit(1)
    e = entry_of(sys.argv[1])
    st = json.loads((HERE / 'data' / 'stats.json').read_text())
    c = cl.load(e); m = M.measure(c, M.scale_of(e))
    print(f"{c['name']}  ({e})  skin faces {m['skin_faces']}, height {m['H']:.0f}")
    print(f"{'measure':14s} {'value':>8s}  {'REAL p10..p90':>17s}  {'cut':>7s}  status")
    bad = 0
    for k in M.KEYS:
        r = st[k]; v = m.get(k)
        if v is None: print(f"{k:14s} {'n/a':>8s}"); continue
        hi = r['direction'] == 'higher'
        side_ok = (v > r['cut']) if hi else (v < r['cut'])
        if r['survives']:
            status = 'PASS' if side_ok else 'OUTSIDE (the one measure that survived: ours fall on this side)'
            bad += not side_ok
        else:
            status = 'info only (did not survive)'
        print(f"{k:14s} {v:8.3f}  {r['real_p10']:7.3f}..{r['real_p90']:<7.3f}  {r['cut']:7.3f}  {status}")
    print("\nconcave = of edge length inside one paint, the share that folds INWARD 30 degrees or more: carved recesses"
          " between forms (muscle, plate, slab), not trim. Raise it by carving the form, not by grooves along trim.")
    if '--json' in sys.argv: pathlib.Path(sys.argv[sys.argv.index('--json') + 1]).write_text(json.dumps(m, indent=1, default=float))
    raise SystemExit(2 if bad else 0)

if __name__ == '__main__':
    main()
