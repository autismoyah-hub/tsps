"""helm3check.py: prints the measures of the pre-registered helm3 test (PREREG.md, HELM3.md) for a helm, with bands.

usage: helm3check.py HELM-labelled.bin [SLOTSPEC] [--rest "model:..@body+arms+..."]

RESULT OF THE TEST: NO measure survived (0 of 17). There is therefore no gate here and the exit code is always 0.
For information only, every measure is printed with the accepted six's band on the side the owner's words predicted,
how many of the eight rejected helms fall outside that band, and whether the helm is inside. 'inside' means nothing
about whether the owner will accept it: Umbral C was inside nearly every band below and was judged worse than A.
Mesh measures are always printed; picture measures only with --rest (the helm is drawn in that outfit).
"""
import sys, pathlib, json
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

SURVIVORS = []          # filled from data/stats.json; empty after the 2026-11-27 run
MESH = ['cr_soft_fold', 'mo_relief', 'un_edge_cv_plate', 'un_face_logsd', 'un_asym', 'ms_sil_front', 'ms_sil_side',
        'ms_dents', 'in_long_lines', 'in_dangling', 'in_mirror']


def main(a):
    if not a or a[0].startswith('-'): raise SystemExit(__doc__)
    rest = None
    if '--rest' in a:
        i = a.index('--rest'); rest = a[i + 1]; del a[i:i + 2]
    path = pathlib.Path(a[0]).resolve().as_posix(); spec = a[1] if len(a) > 1 else 'head+hair+jaw'
    import h3
    S = json.load(open(HERE / 'data' / 'stats.json'))['measures']
    surv = [m for m, v in S.items() if v['survives']]
    r = h3.mesh_measures(h3.L.mesh_ours(path))
    if rest:
        h3.h2.ours('_chk', '?', 'check', path, spec, rest, path)
        r.update(h3.pix('_chk', h3.pictures(['_chk'], tag='chk')))
    print(f'helm3check of {path}')
    print(f'surviving measures: {", ".join(surv) if surv else "NONE (0 of 17 survived leave-one-out with C called rejected)"}')
    print(f"  {'measure':24}{'value':>9}  {'accepted band':>15}  {'rej outside':>11}  {'LOO':>5}  verdict")
    for m in h3.MEASURES:
        if m not in r: continue
        s = S[m]; op, b = s['band']; v = r[m]
        ins = v >= b if op == '>=' else v <= b
        tag = 'SURVIVOR' if m in surv else 'info'
        print(f"  {m:24}{v:>9}  {op + format(b, 'g'):>15}  {len(s['rej_outside_band']):>8}/8  {s['loo_correct']:>3}/14  "
              f"{'inside' if ins else 'OUTSIDE'} ({tag})")
    if not rest: print('  (picture measures skipped: give --rest to draw the helm in an outfit)')
    print('No gate: none of these lines separated the accepted helms from the rejected ones well enough to count.')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
