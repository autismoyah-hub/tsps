"""style2.py EFFECT KIND [--height H]: the style check for an ORIGINAL effect: numbers only, against what was measured on real
effects of the same job (EFFECTS.md section 3, EFFECTS2.md sections 2 and 3). It reads only YOUR files and draws only yours.
  EFFECT  def:PATH.def (spot animation / projectile) | locf:PATH.loc (animated object)
  KIND    projectile | impact | telegraph | hit | hazard | object
Lines: ok / OUT / note. OUT is a number outside what real effects of that job do; it is a question to answer, not a verdict.
"""
import sys, pathlib
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fx, soft, gallery
import numpy as np

JOB = dict(projectile='proj', impact='actor', telegraph='spot', hit='spot', hazard='spot', object='loc')
# kind: (pixels at largest at game size lo..hi, mean distance from the scene lo..hi, life in cycles lo..hi)   [ranges of the study set]
BAND = dict(projectile=((73, 741), (24, 119), None), impact=((81, 2067), (21, 78), (20, 68)), telegraph=((429, 1097), (18, 72), (56, 300)),
            hit=((429, 5249), (13, 62), (30, 120)), hazard=((215, 1142), (40, 122), None), object=((215, 1142), (27, 106), None))


def main(effect, kind, height=0):
    job = JOB[kind]; out = []
    def say(ok, text): out.append(('ok   ' if ok is True else 'OUT  ' if ok is False else 'note ') + text)
    if kind == 'object': fx.LOC_LIFE = max(fx.life(effect, 'loc'), 180)
    r = soft.one(effect, job, height); f = r['files']; sc = r['screen']
    g = gallery.screen_stats(effect, job, height=height); px = [m['px'] for m in g['moments']]; d = [m['dist'] for m in g['moments'] if m['px']]
    (plo, phi), (dlo, dhi), life = BAND[kind]
    say(plo <= max(px) <= phi, f'pixels at its largest, game size: {max(px)} (real {kind}s {plo}..{phi}; a player is about 1,100)')
    say(dlo <= float(np.mean(d)) <= dhi if d else False, f'mean distance from the scene behind: {np.mean(d) if d else 0:.1f} (real {dlo}..{dhi})')
    if life: say(life[0] <= g['total'] <= life[1], f'life {g["total"]} cycles (real {life[0]}..{life[1]})')
    if kind == 'telegraph':
        say(g['total'] % 30 == 0, f'life is a whole number of game ticks ({g["total"] / 30:.2f}): the warning is counted in ticks')
        say(px[0] <= 0.25 * max(px), f'starts small and grows: first {px[0]} of {max(px)} pixels')
        say(f['alpha_share'] >= 0.6, f'translucent share {f["alpha_share"]} (real telegraphs: 77% have translucent faces, 26% wholly)')
    if kind == 'impact': say(px[0] <= 60, f'starts from (almost) nothing: first frame {px[0]} pixels')
    if kind == 'hit': say(px[0] >= 0.3 * max(px) or px[1] >= 0.5 * max(px), f'starts large: first {px[0]}, second moment {px[1]} of {max(px)}')
    hard = sum(1 for _ in [0])
    say(f['smooth_share'] >= 0.6, f'smooth-shaded faces {f["smooth_share"]} (real: median 1.0; 42 of 55 wholly smooth; flat is for metal, crystal, shards)')
    say(254 not in [a for a, _ in f['alpha_values']], 'no face at alpha 254')
    if sc.get('edge_ratio') is not None:
        lim = 1.2 if kind in ('hazard', 'object') else 0.95
        say(sc['edge_ratio'] <= lim, f'edge ratio {sc["edge_ratio"]} (change 1 px inside the outline over the core; real {kind}s median ' + dict(projectile='0.64', impact='0.54', telegraph='0.56', hit='0.86', hazard='0.93', object='0.59')[kind] + '; 1.0 = a cut-out)')
        say(sc['levels'] >= 3, f'tonal levels across the mass {sc["levels"]} (real median 4)')
    big = f['faces'] / max(1, f['parts'])
    say(None, f'{f["faces"]} faces in {f["parts"]} pieces ({big:.0f} a piece; real lump clouds 12 a piece); nesting depth {f["nest_depth"]}; alpha values {f["alpha_values"]}')
    mo = f.get('motion')
    if mo: say(None, f'motion: size {mo["size_first"]} -> {mo["size_max"]} at {mo["peak_at"]} of the life; mean alpha {mo["alpha_first"]} -> {mo["alpha_at_peak"]} -> {mo["alpha_last"]}')
    fx.LOC_LIFE = None
    print('\n'.join(out)); return out


if __name__ == '__main__':
    a = sys.argv[1:]
    if len(a) < 2: raise SystemExit(__doc__)
    main(a[0], a[1], int(a[a.index('--height') + 1]) if '--height' in a else 0)
