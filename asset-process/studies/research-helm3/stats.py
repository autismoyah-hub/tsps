"""stats.py: the pre-registered test of PREREG.md on data/h3.json -> data/stats.json, data/table.md (printed too)."""
import sys, pathlib, json, itertools
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
import numpy as np

ACC = ['torva', 'oathplate', 'justiciar', 'faceguard', 'serpentine', 'v5']
REJ = ['umbralA', 'umbralB', 'umbralC', 'v4a', 'v4b', 'v4c', 'v3a', 'v2a']
ACCP = ['inquisitor', 'masori', 'dharok', 'statius', 'armadyl']
ALL = ACC + REJ
CI = ALL.index('umbralC')
MEASURES = ['cr_hard_step_close', 'cr_ramp_game', 'cr_soft_fold', 'mo_relief', 'mo_steel_shading',
            'df_line_contrast_game', 'df_survive', 'df_dark_cut', 'un_edge_cv_plate', 'un_face_logsd', 'un_asym',
            'ms_sil_front', 'ms_sil_side', 'ms_dents', 'in_long_lines', 'in_dangling', 'in_mirror']
DIR = dict(cr_hard_step_close=+1, cr_ramp_game=-1, cr_soft_fold=-1, mo_relief=+1, mo_steel_shading=+1,
           df_line_contrast_game=+1, df_survive=+1, df_dark_cut=+1, un_edge_cv_plate=-1, un_face_logsd=-1, un_asym=-1,
           ms_sil_front=-1, ms_sil_side=-1, ms_dents=-1, in_long_lines=+1, in_dangling=-1, in_mirror=+1)


def best_cut(z, y):
    """z: values (ACC predicted higher), y: 1 = ACC. -> cut; predict ACC if z > cut"""
    u = np.unique(z)
    cuts = np.r_[u[0] - 1, (u[1:] + u[:-1]) / 2, u[-1] + 1]
    err = np.array([((z > c) != (y == 1)).sum() for c in cuts])
    tied = cuts[err == err.min()]
    return float(tied[len(tied) // 2]), int(err.min())


def loo(z, y):
    pred = np.zeros(len(z), int)
    for i in range(len(z)):
        m = np.ones(len(z), bool); m[i] = False
        c, _ = best_cut(z[m], y[m]); pred[i] = int(z[i] > c)
    return pred


def survives(z, y):
    p = loo(z, y); ok = (p == y)
    return int(ok.sum()), bool(y[CI] == 0 and p[CI] == 0), p


if __name__ == '__main__':
    D = json.load(open(HERE / 'data' / 'h3.json'))
    y = np.array([1] * len(ACC) + [0] * len(REJ))
    combos = list(itertools.combinations(range(len(ALL)), len(ACC)))
    res = {}; exp_surv = exp_loo = 0.0
    for m in MEASURES:
        x = np.array([D[k][m] for k in ALL], float); z = DIR[m] * x
        n_ok, c_ok, pred = survives(z, y)
        cut, err = best_cut(z, y)
        acc = x[:len(ACC)]
        # predicted-side band of the accepted helms
        band = ('>=', float(acc.min())) if DIR[m] > 0 else ('<=', float(acc.max()))
        rejout = [k for k in REJ if (D[k][m] < band[1] if DIR[m] > 0 else D[k][m] > band[1])]
        # exact chance
        hits = hits_loo = 0
        for cb in combos:
            yy = np.zeros(len(ALL), int); yy[list(cb)] = 1
            a, b, _ = survives(z, yy)
            if a >= 12: hits_loo += 1
            if a >= 12 and b: hits += 1
        p = hits / len(combos); pl = hits_loo / len(combos); exp_surv += p; exp_loo += pl
        # separation in the opposite direction (reported, never counted)
        n_opp, _, _ = survives(-z, y)
        res[m] = dict(dir=DIR[m], loo_correct=n_ok, C_rejected=c_ok, survives=bool(n_ok >= 12 and c_ok),
                      full_cut=round(DIR[m] * cut, 4), full_errors=err, band=band, rej_outside_band=rejout,
                      p_chance=round(p, 4), p_chance_loo_only=round(pl, 4), loo_opposite_dir=n_opp,
                      misclassified=[k for k, a, b in zip(ALL, pred, y) if a != b])
        print(f"{m:24} dir {'+' if DIR[m] > 0 else '-'}  LOO {n_ok:2}/14  C->REJ {str(c_ok):5}  survives {str(res[m]['survives']):5}  "
              f"band {band[0]}{band[1]:g}  rej outside {len(rejout)}/8  p {p:.4f}  (opp LOO {n_opp})  wrong {res[m]['misclassified']}")
    print(f'expected survivors by chance (sum of p over {len(MEASURES)}): {exp_surv:.3f};  LOO>=12 alone: {exp_loo:.3f}')
    json.dump(dict(measures=res, expected_chance=exp_surv, expected_chance_loo_only=exp_loo), open(HERE / 'data' / 'stats.json', 'w'), indent=1)
    # table
    rows = ['| helm | group | ' + ' | '.join(MEASURES) + ' |', '|---|---|' + '---:|' * len(MEASURES)]
    rows.append('| predicted ACC | | ' + ' | '.join('higher' if DIR[m] > 0 else 'lower' for m in MEASURES) + ' |')
    for k in ALL + ACCP:
        rows.append(f"| {k} | {D[k]['group']} | " + ' | '.join(str(D[k][m]) for m in MEASURES) + ' |')
    rows.append('| LOO correct /14 | | ' + ' | '.join(str(res[m]['loo_correct']) for m in MEASURES) + ' |')
    rows.append('| C called REJ | | ' + ' | '.join('yes' if res[m]['C_rejected'] else 'no' for m in MEASURES) + ' |')
    rows.append('| p chance | | ' + ' | '.join(f"{res[m]['p_chance']:.3f}" for m in MEASURES) + ' |')
    (HERE / 'data' / 'table.md').write_text('\n'.join(rows) + '\n')
