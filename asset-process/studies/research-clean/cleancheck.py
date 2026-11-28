"""cleancheck: is a creature CLEAN, DEFINED and BUILT as real ones are? (research-clean, CLEAN.md)

usage: cleancheck.py ENTRY [ENTRY ..] [--pic OUT.png] [--json OUT.json]
       cleancheck.py --calibrate          re-measure the 24 real creatures of the sample -> data/bands.json
ENTRY = a real NPC id (12812 or npc:12812) | def:PATH.json | PATH.json | one of our build folders

What this answers, paraphrased from a review of the stone watcher beside the Minotaur and Tekton: the real
references read clean, defined and smooth; ours had features that looked stuck on and paint that looked blotchy
rather than smooth or deliberately detailed.

Everything is measured on the REST mesh lit exactly as the client lights it (once, at rest, by vertex vote), on the
drawn opaque skin. Only measures that put all 24 real creatures of the sample on one side and the criticised
watcher (both paints) on the other are verdict lines. Terms:

  line       a shared edge the eye can see: the two faces differ in paint, or the lit lightness has a kink there of 4
             or more (change of the light gradient across the edge, taken over 2% of the creature's height)
  plane      faces joined across edges that are not lines: one flat-looking area of the lit picture
  small      a plane under 0.2% of the skin
  mark       a small plane that stands out: 6 or more lit lightness from what surrounds it, or bordered 70% by another material
  facet      a small weak plane whose border is at least a third COMMITTED: a hard fold (50 degrees or more), a strong
             tone step (6 or more lit lightness) or another material
  SPLOTCH    a small weak plane that is neither: it differs a little from its neighbours for no drawn reason
  hard fold  50 degrees or more between two faces; firm fold 30 to 50 (uncommitted: neither a plane change nor a crease)
  piece      a separate shell (shares no vertex with the main skin) of at most 6% of the faces
  lump       a piece that is CLOSED (no open edge) and touches the skin; small lump: no longer than 7% of the height

Verdict lines (real band = min to max of the 24; the picture marks the offenders):
  1 splotch            share of the skin in splotches
  2 committed folds    hard-fold length / firm-fold length inside one material
  3 firm folds         share of in-material edge length folded 30 to 50 degrees
  4 one skin           share of the faces in the largest welded piece
  5 stuck-on lumps     share of the faces in closed pieces sunk into the skin (a tenth or more of their vertices inside)
  6 small lumps        closed pieces no longer than 7% of the height (the scale of an eye, a nostril, a stud)
  7 features as lumps  share of the small contrasting features (up to 7% of the height, 20 grey levels) that are closed pieces
Style line: deliberate small detail (marks + facets, share of skin): under 0.20 = SMOOTH (as the Minotaur), 0.20 or
  more = DETAILED (as Tekton). Splotch has a band per style.
Exit code 0 if every verdict line is INSIDE, 2 otherwise.
"""
import sys, pathlib, json, math, collections
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import numpy as np
import cleanlib as cl
import features as ft
from explore import same_mat, REAL

SMALL, WEAK, HARD, FIRM, KINK, ANCHOR, STRONG = 0.002, 6.0, 50.0, 30.0, 4.0, 0.34, 6.0
STYLE_SPLIT = 0.20

def prep(c):
    cl.topology(c)
    F, C, V, LC = c['F'], c['C'], c['V'], c['LC'].astype(float); area = c['area']
    skin = c['drawn'] & (c['A'] == 0); c['skinmask'] = skin
    used = np.unique(F[skin]); c['hgt'] = float(np.ptp(V[used][:, 1])); c['tot'] = float(area[skin].sum())
    ea, eb = c['e_a'], c['e_b']; both = skin[ea] & skin[eb]
    c['sea'], c['seb'], c['sel'], c['sev'], c['sdih'] = ea[both], eb[both], c['e_len'][both], c['e_v'][both], c['e_dih'][both]
    ea, eb, ev = c['sea'], c['seb'], c['sev']
    a, b, d = V[F[:, 0]], V[F[:, 1]], V[F[:, 2]]
    e1, e2 = b - a, d - a
    n = np.cross(e1, e2); nn = (n * n).sum(1); nn[nn == 0] = 1
    l0, l1, l2 = LC[:, 0], LC[:, 1], LC[:, 2]
    grad = ((l1 - l0)[:, None] * np.cross(e2, n) + (l2 - l0)[:, None] * np.cross(n, e1)) / nn[:, None]      # lit lightness per unit, in the face
    ed = V[ev[:, 1]] - V[ev[:, 0]]; ed /= np.maximum(np.linalg.norm(ed, axis=1), 1e-9)[:, None]
    ta = np.cross(c['unit'][ea], ed); tb = np.cross(c['unit'][eb], ed)
    dc = c['cen'][eb] - c['cen'][ea]
    sa = np.sign((dc * ta).sum(1)); sb = np.sign((dc * tb).sum(1))
    c['kink'] = np.abs((grad[ea] * ta).sum(1) * sa - (grad[eb] * tb).sum(1) * sb) * 0.02 * c['hgt']
    c['samecol'] = C[ea] == C[eb]
    c['sm'] = np.array([same_mat(int(x), int(y)) for x, y in zip(C[ea], C[eb])])
    c['litf'] = LC.mean(1)
    pl = (C & 127).astype(float)
    c['step'] = np.abs(pl[ea] - pl[eb]) * (c['LN'][ev].mean(1) / 128)          # the paint step as drawn (lit lightness)
    return c

def planes(c):
    """Small planes of the lit picture, each classed mark / facet / splotch. Returns shares and the faces of each class."""
    area, tot, val = c['area'], c['tot'], c['litf']
    link = c['samecol'] & (c['kink'] < KINK)
    parent = {int(i): int(i) for i in np.nonzero(c['skinmask'])[0]}
    def find(x):
        while parent[x] != x: parent[x] = parent[parent[x]]; x = parent[x]
        return x
    for x, y, l in zip(c['sea'], c['seb'], link):
        if l: parent[find(int(x))] = find(int(y))
    root = {i: find(i) for i in parent}
    ra = collections.Counter(); rv = collections.Counter(); members = collections.defaultdict(list)
    for i, r in root.items(): ra[r] += area[i]; rv[r] += area[i] * val[i]; members[r].append(i)
    nb = collections.defaultdict(lambda: [0.0, 0.0, 0.0, 0.0, 0.0])   # border, sum(len * neighbour lit), other material, committed, paint-step border
    anch = (np.abs(c['sdih']) >= HARD) | (c['step'] >= STRONG) | (~c['sm'])
    for x, y, l, s, an, sc in zip(c['sea'], c['seb'], c['sel'], c['sm'], anch, c['samecol']):
        rx, ry = root[int(x)], root[int(y)]
        if rx == ry: continue
        for r, o in ((rx, y), (ry, x)):
            n = nb[r]; n[0] += l; n[1] += l * val[o]
            if not s: n[2] += l
            if an: n[3] += l
            if not sc: n[4] += l
    out = dict(mark=0.0, facet=0.0, splotch=0.0); faces = dict(mark=[], facet=[], splotch=[]); bp = bl = 0.0
    for r, a in ra.items():
        n = nb[r]
        if a > SMALL * tot or n[0] == 0: continue
        con = abs(rv[r] / a - n[1] / n[0])
        k = 'mark' if (n[2] / n[0] >= 0.7 or con >= WEAK) else 'facet' if n[3] / n[0] >= ANCHOR else 'splotch'
        out[k] += a / tot; faces[k] += members[r]
        if k == 'splotch': bp += n[4]; bl += n[0]
    out['splotch_border_paint'] = bp / max(bl, 1e-9)          # share of the splotches' border that is a paint step (the rest is light alone)
    out['plane_count'] = len(ra)
    return out, faces

def folds(c):
    el, ad, sm = c['sel'], np.abs(c['sdih']), c['sm']
    hard = sm & (ad >= HARD); firm = sm & (ad >= FIRM) & (ad < HARD)
    out = dict(hard_over_firm=float(el[hard].sum() / max(el[firm].sum(), 1e-9)), firm_share=float(el[firm].sum() / max(el[sm].sum(), 1e-9)),
               hard_share=float(el[hard].sum() / max(el[sm].sum(), 1e-9)),
               mushy_unpainted_over_hard=float(el[firm & c['samecol']].sum() / max(el[(ad >= HARD)].sum(), 1e-9)))
    return out, np.nonzero(firm & c['samecol'])[0]

def pieces(c):
    feats, hgt = ft.find(c)
    nf = int(c['skinmask'].sum())
    shells = [f for f in feats if f['kind'] == 'shell']
    closed = [f for f in shells if f['closed'] and f['touching']]
    stuck = [f for f in closed if f['sunk'] >= 0.1]
    lumps = [f for f in closed if f['size_h'] <= 0.07]
    small = [f for f in feats if f['size_h'] <= 0.07 and f['contrast'] >= 20]
    cnt = collections.Counter(f['construction'] for f in small)
    sizes = collections.Counter(c['shell'][int(i)] for i in np.nonzero(c['skinmask'])[0])
    out = dict(largest_piece=max(sizes.values()) / nf, shells=len(sizes), pieces=len(shells), closed_pieces=len(closed),
               stuck_faces=sum(f['faces'] for f in stuck) / nf, stuck_pieces=len(stuck),
               small_lumps=len(lumps), small_lump_faces=sum(f['faces'] for f in lumps) / nf,
               lump_height_max=max([f['relief_h'] for f in lumps], default=0.0),
               small_features=len(small), features_as_lumps=cnt.get('piece-closed', 0) / max(len(small), 1),
               features_in_skin=(cnt.get('paint', 0) + cnt.get('inset', 0) + cnt.get('relief', 0)) / max(len(small), 1),
               small_paint=cnt.get('paint', 0), small_inset=cnt.get('inset', 0), small_relief=cnt.get('relief', 0), small_open=cnt.get('piece-open', 0),
               small_closed=cnt.get('piece-closed', 0), small_floating=cnt.get('floating', 0))
    return out, dict(stuck=[int(i) for f in stuck for i in f['fs']], lumps=[int(i) for f in lumps for i in f['fs']], lump_list=lumps, stuck_list=stuck)

def measure(c):
    prep(c)
    p, pf = planes(c); f, mush = folds(c); q, qf = pieces(c)
    m = dict(splotch=p['splotch'], marks=p['mark'], facets=p['facet'], deliberate=p['mark'] + p['facet'], splotch_border_paint=p['splotch_border_paint'], **f, **q,
             faces=int(c['skinmask'].sum()))
    m['style'] = 'detailed' if m['deliberate'] >= STYLE_SPLIT else 'smooth'
    return m, dict(splotch=pf['splotch'], mark=pf['mark'], facet=pf['facet'], mush_edges=mush, **qf)

# key, text, side ('max' = must not exceed the real maximum, 'min' = must reach the real minimum), format
LINES = [
    ('splotch', '1 splotch: skin in small weak planes with no committed border', 'max', '{:.3f}'),
    ('hard_over_firm', '2 committed folds: hard-fold length / firm-fold length', 'min', '{:.2f}'),
    ('firm_share', '3 firm folds (30-50 deg): share of in-material edge length', 'max', '{:.3f}'),
    ('largest_piece', '4 one skin: faces in the largest welded piece', 'min', '{:.3f}'),
    ('stuck_faces', '5 stuck-on: faces in closed pieces sunk into the skin', 'max', '{:.3f}'),
    ('small_lumps', '6 small closed lumps (eye / nostril / stud scale)', 'max', '{:.0f}'),
    ('features_as_lumps', '7 small contrasting features built as closed pieces', 'max', '{:.3f}'),
]
INFO = [('deliberate', 'deliberate small detail (marks + facets), share of skin'), ('marks', '   marks'), ('facets', '   facets'),
        ('splotch_border_paint', 'splotch borders that are paint steps (rest: light alone)'), ('mushy_unpainted_over_hard', 'firm folds with no paint change / hard folds'),
        ('features_in_skin', 'small contrasting features that are faces of the skin'), ('lump_height_max', 'tallest small lump above the skin, share of height'),
        ('closed_pieces', 'closed pieces touching the skin'), ('shells', 'shells in all'), ('faces', 'skin faces')]

def bands():
    p = HERE / 'data' / 'bands.json'
    if not p.exists(): raise SystemExit('run cleancheck.py --calibrate first')
    return json.loads(p.read_text())

def calibrate():
    rows = {}
    for i, (name, st) in REAL.items():
        m, _ = measure(cl.load(f'npc:{i}')); m['name'] = name; rows[str(i)] = m; print('measured', name, file=sys.stderr)
    (HERE / 'data').mkdir(exist_ok=True)
    (HERE / 'data' / 'bands.json').write_text(json.dumps(rows, indent=0))
    return rows

def band_of(rows, key, style=None):
    v = [r[key] for r in rows.values() if style is None or r['style'] == style]
    return min(v), float(np.median(v)), max(v), len(v)

def report(name, m, rows, quiet=False):
    outs = []
    lines = [f'== {name}', f"   style: {m['style'].upper()}  (deliberate small detail {m['deliberate']:.3f} of the skin; under {STYLE_SPLIT:.2f} = smooth like the Minotaur "
             f"{rows['12812']['deliberate']:.3f}, else detailed like Tekton {rows['7541']['deliberate']:.3f})"]
    for key, text, side, fmt in LINES:
        lo, med, hi, n = band_of(rows, key)
        x = m[key]; ok = x <= hi + 1e-12 if side == 'max' else x >= lo - 1e-12
        if not ok: outs.append(key)
        extra = ''
        if key == 'splotch':
            slo, smed, shi, sn = band_of(rows, key, m['style'])
            extra = f"; {m['style']} style ({sn} real): {slo:.3f}-{shi:.3f}, median {smed:.3f}: {'inside' if x <= shi else 'OUTSIDE'}"
        nearer = 'Minotaur' if abs(x - rows['12812'][key]) <= abs(x - rows['7541'][key]) else 'Tekton'
        lines.append(f"   {text:66s} {fmt.format(x):>7s} | real {fmt.format(lo)}-{fmt.format(hi)} (median {fmt.format(med)}; Minotaur {fmt.format(rows['12812'][key])}, Tekton {fmt.format(rows['7541'][key])}) "
                     f"| {'INSIDE' if ok else 'OUTSIDE'}{extra}")
    lines.append('   -- not verdicts:')
    for key, text in INFO:
        lo, med, hi, n = band_of(rows, key); x = m[key]
        lines.append(f"      {text:63s} {x:7.3f} | real {lo:.3f}-{hi:.3f} (median {med:.3f}; Minotaur {rows['12812'][key]:.3f}, Tekton {rows['7541'][key]:.3f})")
    lines.append(f"   small contrasting features by construction: paint {m['small_paint']}, inset {m['small_inset']}, relief {m['small_relief']}, open piece {m['small_open']}, "
                 f"CLOSED piece {m['small_closed']}, floating {m['small_floating']}")
    lines.append(f"   VERDICT: {'INSIDE the real band on all 7 lines' if not outs else 'OUTSIDE on ' + str(len(outs)) + ' of 7: ' + ', '.join(outs)}")
    if not quiet: print('\n'.join(lines))
    return outs, lines

# ---------------------------------------------------------------------------------------------------- the picture
MAG, RED, YEL, CYAN, GREEN = (255, 40, 220), (255, 50, 40), (255, 235, 60), (60, 230, 255), (90, 230, 90)
def picture(c, m, off, out, title=None, size=430):
    """Three views (front quarter, back quarter, head and chest close), each plain above and marked below.
    magenta = splotch planes; red = closed pieces stuck on (bright red = small lumps); yellow lines = firm folds
    (30-50 degrees) carrying no paint change."""
    from PIL import Image, ImageDraw, ImageFont
    def font(n):
        try: return ImageFont.truetype('segoeui.ttf', n)
        except Exception: return ImageFont.load_default()
    V = c['V']; skin = c['skinmask']; used = np.unique(c['F'][skin]); Vu = V[used]; lo, hi = Vu.min(0), Vu.max(0); h = hi[1] - lo[1]
    keep = np.nonzero(skin)[0]
    top = Vu[Vu[:, 1] <= lo[1] + 0.34 * h]; headbox = (top.min(0), top.max(0))
    tint = {int(i): MAG for i in off['splotch']}
    tint.update({int(i): (170, 30, 30) for i in off['stuck']}); tint.update({int(i): RED for i in off['lumps']})
    views = [(25, None), (205, None), (25, headbox)]
    rows = []
    for marked in (False, True):
        cells = []
        for yaw, box in views:
            img, ids = cl.view(c, yaw, 8, size=size, box=box, faces=keep, tint=tint if marked else None)
            canvas = np.zeros((size, size, 3), np.uint8); canvas[:] = cl.BG
            hh, ww = min(img.shape[0], size), min(img.shape[1], size); canvas[:hh, :ww] = img[:hh, :ww]
            im = Image.fromarray(canvas)
            if marked:
                X, Y, Z, W_, H_ = cl.frame(c, yaw, 8, None, size, box, keep, 8)
                d = ImageDraw.Draw(im); r_, down, fwd = cl.camera(yaw, 8)
                for k in off['mush_edges']:
                    fa, fb = c['sea'][k], c['seb'][k]
                    if c['unit'][fa] @ fwd > 0 and c['unit'][fb] @ fwd > 0: continue
                    a, b = c['sev'][k]
                    mx, my = int((X[a] + X[b]) / 2), int((Y[a] + Y[b]) / 2)
                    if not (0 <= mx < ww and 0 <= my < hh) or ids[my, mx] not in (fa, fb): continue
                    d.line([(X[a], Y[a]), (X[b], Y[b])], fill=YEL, width=1)
            cells.append(np.array(im))
        rows.append(np.concatenate(cells, 1))
    body = np.concatenate(rows, 0)
    head = Image.new('RGB', (body.shape[1], 64), (20, 18, 16)); d = ImageDraw.Draw(head)
    d.text((6, 2), title or c['name'], fill=(240, 220, 150), font=font(17))
    d.text((6, 24), f"style {m['style']}   splotch {m['splotch']:.3f}   hard/firm folds {m['hard_over_firm']:.2f}   firm share {m['firm_share']:.3f}   largest piece {m['largest_piece']:.2f}   "
                    f"stuck-on faces {m['stuck_faces']:.3f}   small lumps {m['small_lumps']}", fill=(235, 235, 235), font=font(13))
    d.text((6, 44), 'top: as lit.  bottom:', fill=(200, 200, 200), font=font(12)); x = 130
    for col, txt in ((MAG, 'splotch planes'), (RED, 'small closed lumps'), ((170, 30, 30), 'other stuck-on closed pieces'), (YEL, 'firm folds (30-50 deg) with no paint change')):
        d.rectangle([x, 47, x + 12, 59], fill=col); d.text((x + 16, 44), txt, fill=(220, 220, 220), font=font(12)); x += 34 + int(d.textlength(txt, font=font(12)))
    board = Image.new('RGB', (body.shape[1], body.shape[0] + 64)); board.paste(head, (0, 0)); board.paste(Image.fromarray(body), (0, 64))
    board.save(out)
    return board

def jsonable(m): return {k: (float(v) if isinstance(v, (np.floating, float)) else int(v) if isinstance(v, (np.integer,)) else v) for k, v in m.items()}

if __name__ == '__main__':
    a = sys.argv[1:]
    if not a: raise SystemExit(__doc__)
    if '--calibrate' in a:
        rows = calibrate()
        for key, text, side, fmt in LINES:
            lo, med, hi, n = band_of(rows, key); print(f'{text:66s} real {fmt.format(lo)} - {fmt.format(hi)}, median {fmt.format(med)}  ({n} creatures)')
        raise SystemExit(0)
    rows = bands(); pic = a[a.index('--pic') + 1] if '--pic' in a else None; js = a[a.index('--json') + 1] if '--json' in a else None
    entries = [x for i, x in enumerate(a) if not x.startswith('--') and (i == 0 or a[i - 1] not in ('--pic', '--json'))]
    worst = 0; allm = {}
    for n, e in enumerate(entries):
        c = cl.load(e); m, off = measure(c); outs, _ = report(c['name'], m, rows); allm[c['name']] = jsonable(m)
        if outs: worst = 2
        if pic:
            p = pathlib.Path(pic); p = p if len(entries) == 1 else p.with_name(f'{p.stem}-{n + 1}{p.suffix}')
            picture(c, m, off, p); print('   picture:', p)
    if js: pathlib.Path(js).write_text(json.dumps(allm, indent=1))
    raise SystemExit(worst)
