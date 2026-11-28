"""motion_fit.py: step 1, the helm against the collar and shoulders as the game draws it, standing and through walk
819 and run 824 (all 8 frames each), from the front (yaw 0) and the side (yaw 512).
Per outfit and frame, from the drawn silhouette (all columns share the bare figure's camera):
  head_w   median width of the figure over heights 180..194 (units)
  neck_w   narrowest width over heights 162..175, and its ratio to head_w ("pinch": 1 = no neck seen, a column)
  skin     skin pixels (hue 4, saturation 3) in the band 160..177 within 16 units of the head's middle: the neck shown
  sh_ratio head_w / widest row over heights 140..166 (front view only): helm to shoulders in the whole figure
usage: motion_fit.py -> motion-fit.json, pictures in render/research-fit-motion/"""
import sys, pathlib, json
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import lib as L

OUT = L.ROOT / 'render' / 'research-fit-motion'
ZOOM = 260; CELL = (300, 520)
ent, keys = [], []
for key, hid, name, bid, lid, note in L.REFS:
    ent.append(f'item:{hid}+item:{bid}+item:{lid}'); keys.append(key)
for key, path, spec, name, bpath, rnd in L.OURS:
    rest = L.SET_V2 if bpath == L.CUIRASS_V2 else L.SET_V3
    ent.append(f'model:{path}@{spec}+{rest}'); keys.append(key)
ent.append('none'); keys.append('none')
res = {k: {} for k in keys}
for seq, sname in ((819, 'walk'), (824, 'run')):
    for yaw, vname in ((0, 'front'), (512, 'side')):
        frames = ['bind'] + [str(i) for i in range(8)]
        a = L.pose(OUT / f'{sname}-{vname}.png', ent, '--seq', seq, '--frames', ','.join(frames), '--zoom', ZOOM, '--yaw', yaw, '--cell', f'{CELL[0]}x{CELL[1]}')
        ch = a.shape[0] // len(frames); cw = a.shape[1] // len(ent)
        bg = a[2, 2]
        for fi, fr in enumerate(frames):
            row = a[fi * ch:(fi + 1) * ch]
            none = row[:, (len(ent) - 1) * cw:]
            nm = np.abs(none - bg).sum(2) > 0
            ys = np.nonzero(nm.any(1))[0]; ytop, ybot = ys.min(), ys.max()
            u = (ybot - ytop) / 196.0                                  # pixels a unit (crown 196 to the soles)
            yr = lambda h: int(round(ytop + (196 - h) * u))
            headcols = np.nonzero(nm[yr(192):yr(178)].any(0))[0]; cx = (headcols.min() + headcols.max()) / 2
            for k, key in enumerate(keys[:-1]):
                c = row[:, k * cw:(k + 1) * cw]; m = np.abs(c - bg).sum(2) > 0
                w = m.sum(1) / u
                def span(y0, y1):
                    out = []
                    for y in range(y0, y1):
                        xs = np.nonzero(m[y])[0]; out.append((xs.max() - xs.min() + 1) / u if len(xs) else 0.0)
                    return np.array(out)
                hw = float(np.median(span(yr(194), yr(180))))
                nk = float(span(yr(175), yr(162)).min())
                x0, x1 = int(cx - 16 * u), int(cx + 16 * u)
                reg = c[yr(177):yr(160), x0:x1].reshape(-1, 3)
                h_, s_, l_, ok = L.hsl_of(reg)
                skin = int((ok & (h_ == 4) & (s_ == 3) & (l_ > 20)).sum())
                d = dict(head_w=round(hw, 1), neck_w=round(nk, 1), pinch=round(nk / max(hw, 1), 2), skin=skin)
                if vname == 'front':
                    d['sh_ratio'] = round(hw / max(float(span(yr(166), yr(140)).max()), 1), 2)
                res[key][f'{sname}-{vname}-{fr}'] = d
json.dump(res, open(pathlib.Path(__file__).parent / 'motion-fit.json', 'w'), indent=1)
print(f"{'key':11}{'stand: head_w f/s':>20}{'pinch f/s':>12}{'sh_ratio':>9}{'skin f/s':>10} | walk pinch f (min..max) s (min..max) skin max | run pinch f, s, skin max")
for key in keys[:-1]:
    r = res[key]
    def rng(seq, v, f):
        vals = [r[f'{seq}-{v}-{i}'][f] for i in range(8)]
        return f'{min(vals):g}..{max(vals):g}'
    b = lambda v, f: r[f'walk-{v}-bind'][f]
    print(f"{key:11}{str(b('front', 'head_w')) + '/' + str(b('side', 'head_w')):>20}{str(b('front', 'pinch')) + '/' + str(b('side', 'pinch')):>12}{b('front', 'sh_ratio'):>9}"
          f"{str(b('front', 'skin')) + '/' + str(b('side', 'skin')):>10} | {rng('walk', 'front', 'pinch'):>10} {rng('walk', 'side', 'pinch'):>10} {max(r[f'walk-{v}-{i}']['skin'] for v in ('front', 'side') for i in range(8)):>4}"
          f" | {rng('run', 'front', 'pinch'):>10} {rng('run', 'side', 'pinch'):>10} {max(r[f'run-{v}-{i}']['skin'] for v in ('front', 'side') for i in range(8)):>4}")
