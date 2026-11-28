"""ifgate_proof.py [GROUP ...]: prove ifgate.py (Teacher only; reads sheets-teacher-only/).

For each target: the CONTROL (the real panel in the rebuild form, sheets-teacher-only/G/control.json) must PASS at 0
pixels; then damaged copies of it, each one mistake an author could make:
  title_font      the title in p12_full instead of b12_full (or the first text: next font)
  box_moved_3px   the first inner region (box, well, bar, button) moved 3px right
  colour_1_level  the title's (or first text's) colour off by one level in blue (ff981f -> ff981e)
  colour_ff9933   the title's colour the neighbouring measured orange ff9933
  missing_divider the title divider strip removed (HUD: the bar's outline)
  close_moved_3px the close button 3px left
  wider_20px      the panel 20px wider: everything right of the middle moves 20px right, spans stretch
  backing_phase   (harmless-looking) the panel backing tiled from (2,1) instead of (1,1): a 1px phase shift
  text_1px_down   (within tolerance) the title (or first text) 1px lower: structure must accept it
  text_3px_down   the same 3px lower: structure must reject it
  icons_swapped   two different icons exchanged (where the panel has icons)
Also the proof rebuild of 148. Writes gate-proof/<G>-<damage>.json, gate-proof/results.json, results.txt, boards.
"""
import sys, json, io, contextlib
from pathlib import Path
import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import ifref as R
import ifgate as G

OUT = HERE / 'gate-proof'
T = HERE / 'sheets-teacher-only'


def load(g):
    comps = json.loads((T / str(g) / 'control.json').read_text())['components']
    cl = json.loads((T / str(g) / 'classify.json').read_text())
    return comps, cl


def copy(c):
    return json.loads(json.dumps(c))


def first_text(cl, role=None):
    for t in cl['texts']:
        if role is None or t['role'] == role:
            return t
    return None


def damage(g, name, comps, cl):
    c = copy(comps)
    regs = cl['regions']
    frame = next((r for r in regs if r['family'] in ('steel', 'stone', 'stone2') and 'titled' in r['words']), None)
    t = first_text(cl, 'title') or first_text(cl)
    if name == 'title_font':
        if not t:
            return None
        w = c[t['member']]
        w['font'] = {496: 495, 495: 496, 494: 495, 497: 495}.get(w['font'], 495)
        return c
    if name in ('colour_1_level', 'colour_ff9933'):
        if not t:
            return None
        w = c[t['member']]
        w['colors'][0] = (w['colors'][0] ^ 1) if name == 'colour_1_level' else (
            0xff9933 if w['colors'][0] != 0xff9933 else 0xff981f)
        return c
    if name == 'box_moved_3px':
        pri = ['steel', 'thin', 'two-tone', 'bevel-light', 'bevel-dark', 'bevel-mid', 'bevel-beige', 'tile-grey',
               'tile-red', 'sprite-built', 'well', 'bar', 'outline']
        cands = [r for r in regs if r is not frame and r['family'] in pri]
        if not cands:
            return None
        r = sorted(cands, key=lambda r: pri.index(r['family']))[0]
        for m in r['members']:
            c[m]['raw_position'][0] += 3
        return c
    if name == 'missing_divider':
        if frame is None:
            outl = [r for r in regs if r['family'] == 'outline']
            if len(outl) < 2:
                return None
            drop = set(outl[1]['members'])
        else:
            drop = {m for m in frame['members'] if c[m]['kind'] == 5 and c[m]['sprites'][0] in R.DIVIDERS}
        keep = [w for n, w in enumerate(c) if n not in drop]
        return keep
    if name == 'close_moved_3px':
        if frame is None:
            return None
        for m in frame['members']:
            if c[m]['kind'] == 5 and c[m]['sprites'][0] in R.CLOSES:
                c[m]['raw_position'][0] -= 3
        return c
    if name == 'backing_phase':
        if frame is None:
            return None
        for m in frame['members']:
            if c[m]['kind'] == 5 and c[m]['sprites'][0] in R.BACKINGS:
                c[m]['raw_position'][0] += 1; c[m]['raw_size'][0] -= 1
        return c
    if name == 'wider_20px':
        n = R.normalize(c)
        S = R.rebuild_scene(n, 'hud' if g == 303 else 'modal')
        pr = json.loads((T / str(g) / 'meta.json').read_text())['panel']
        mid = pr[0] + pr[2] // 2
        absr = {}
        for w in n:
            p = S.lay[w['id']]
            x, y = p['abs']; ww, hh = p['size']
            if x + ww <= mid:
                pass
            elif x >= mid:
                x += 20
            else:
                ww += 20
            absr[w['id']] = (x, y, ww, hh)
        for w in n:
            x, y, ww, hh = absr[w['id']]
            px, py = (absr[w['parent']][:2] if w['parent'] is not None else (0, 0))
            w['raw_position'] = [x - px, y - py]; w['raw_size'] = [ww, hh]
            w['size_mode'] = [0, 0]; w['position_mode'] = [0, 0]
        return n
    if name in ('text_1px_down', 'text_3px_down'):
        if not t:
            return None
        c[t['member']]['raw_position'][1] += 1 if name == 'text_1px_down' else 3
        return c
    if name == 'icons_swapped':
        ic = [r for r in regs if r['family'] == 'icon']
        a = next((r for r in ic if r['words'] != ic[0]['words']), None) if ic else None
        if a is None:
            return None
        m0, m1 = ic[0]['members'][0], a['members'][0]
        c[m0]['sprites'][0], c[m1]['sprites'][0] = c[m1]['sprites'][0], c[m0]['sprites'][0]
        return c
    raise ValueError(name)


DAMAGES = ['title_font', 'box_moved_3px', 'colour_1_level', 'colour_ff9933', 'missing_divider', 'close_moved_3px',
           'wider_20px', 'backing_phase', 'text_1px_down', 'text_3px_down', 'icons_swapped']


def run(g, label, comps, results, text):
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        v, rows, px = G.gate_rebuild(g, comps, detail=True, board_path=OUT / f'{g}-{label}-board.png')
    text.append(f'### {g} {label}\n' + buf.getvalue())
    fails = [r['measure'] for r in rows if r['verdict'] in ('FAIL', 'BORDERLINE')]
    results.append(dict(group=g, case=label, verdict=v, pixels=round(px['share'], 3), differ=px['differ'],
                        failing=fails))
    print(f'{g:>4} {label:<16} {v:<10} pixels {px["share"]:.3f}% ({px["differ"]})  failing: {"; ".join(fails) or "-"}',
          flush=True)


def main():
    OUT.mkdir(exist_ok=True)
    groups = [int(a) for a in sys.argv[1:]] or [68, 648, 210, 476, 383, 303, 912, 148]
    results, text = [], []
    for g in groups:
        comps, cl = load(g)
        run(g, 'control', comps, results, text)
        for d in DAMAGES:
            c = damage(g, d, comps, cl)
            if c is None:
                print(f'{g:>4} {d:<16} n/a')
                continue
            (OUT / f'{g}-{d}.json').write_text(json.dumps(dict(components=c)))
            run(g, d, c, results, text)
        if g == 148:
            p = json.loads((HERE / 'proof' / 'rebuild-148.json').read_text())['components']
            run(148, 'proof_rebuild', p, results, text)
    tag = '-'.join(map(str, groups)) if sys.argv[1:] else 'all'
    (OUT / f'results-{tag}.json').write_text(json.dumps(results, indent=1))
    (OUT / f'results-{tag}.txt').write_text('\n'.join(text))


if __name__ == '__main__':
    main()
