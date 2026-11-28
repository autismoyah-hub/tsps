"""fit_study.py: step 1. Measure the fit of 32 helms (5 references, 10 further coveted, 10 plain, our 7).
Writes fit-measures.json and prints the tables. usage: fit_study.py"""
import sys, pathlib, json
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import lib as L, fitlib as F

rows = []
for g, lst in (('R', L.REFS), ('C', L.COVETED), ('P', L.PLAIN)):
    for key, hid, name, bid, lid, note in lst:
        wp = L.wear_positions(hid)
        m = L.mesh_real(hid); body = L.mesh_real(bid)
        d = F.measure(m, body, hides=(8 in wp[1:], 11 in wp[1:]))
        d.update(group=g, key=key, name=name, id=hid, body=bid, hides_hair=8 in wp[1:], hides_jaw=11 in wp[1:])
        rows.append(d)
for key, path, spec, name, bpath, rnd in L.OURS:
    m = L.mesh_ours(path); body = L.mesh_ours(bpath)
    d = F.measure(m, body, hides=('hair' in spec, 'jaw' in spec))
    d.update(group='O', key=key, name=name, id=None, body=pathlib.Path(bpath).parent.name, hides_hair='hair' in spec, hides_jaw='jaw' in spec)
    rows.append(d)
    # each of ours also against the other cuirass
    other = L.CUIRASS_V3 if bpath == L.CUIRASS_V2 else L.CUIRASS_V2
    d2 = F.measure(m, L.mesh_ours(other), hides=('hair' in spec, 'jaw' in spec))
    d['other_body'] = {k: d2[k] for k in ('helm_to_shoulder', 'pinch_f', 'pinch_s', 'pinch_f_ratio', 'pinch_s_ratio', 'chin_overhang', 'nape_overhang', 'shoulder_w', 'collar_top')}
json.dump(dict(head=F.head_measure(), helms=rows), open(pathlib.Path(__file__).parent / 'fit-measures.json', 'w'), indent=1)


def table(cols, title):
    print('\n' + title)
    print(f"{'g':1} {'key':11}" + ''.join(f'{c[:10]:>11}' for c in cols))
    for d in rows:
        print(f"{d['group']:1} {d['key']:11}" + ''.join(f"{('-' if d.get(c) is None else d.get(c)):>11}" for c in cols))
    for g in 'RCPO':
        sel = [d for d in rows if d['group'] == g]
        def rng(c):
            v = [d[c] for d in sel if d.get(c) is not None]
            return f'{min(v):g}..{max(v):g}' if v else '-'
        print(f"{g} {'range':11}" + ''.join(f'{rng(c):>11}' for c in cols))


table(['cover', 'so_p10', 'so_med', 'so_p90', 'so_iqr', 'so_crown', 'so_brow', 'so_temple', 'so_nape', 'so_cheek', 'so_jaw'], 'STAND-OFF from the bare head (units)')
table(['wid', 'hgt', 'dep', 'above', 'below', 'fore', 'aft', 'w_crown', 'w_brow', 'w_cheek', 'w_jaw', 'w_neck', 'w_skull', 'w_max', 'h_wmax'], 'SIZE against the bare head (ratios; units)')
table(['d_brow', 'd_cheek', 'd_jaw', 'd_neck', 'd_skull', 'taper_jaw', 'taper_neck', 'box_fill', 'low_front', 'low_side', 'low_back'], 'DEPTH, TAPER, LOWER EDGE')
table(['shoulder_w', 'helm_to_shoulder', 'helmmax_to_shoulder', 'collar_top', 'body_top', 'pinch_f', 'pinch_f_h', 'pinch_s', 'pinch_s_h', 'pinch_f_ratio', 'pinch_s_ratio', 'chin_overhang', 'nape_overhang', 'lap_front'], 'HELM AGAINST THE BODY PIECE')
