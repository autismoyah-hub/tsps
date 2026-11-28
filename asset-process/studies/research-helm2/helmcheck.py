"""helmcheck.py: the measures that separated the helms the owner ACCEPTED from the ones the owner REJECTED (HELM2.md).
A GUARD, NOT A JUDGE: a helm outside a band has a measured difference from every accepted helm; a helm inside
all bands has not thereby passed (Umbral A passed fit.py, the worn gate and the brief's tables and failed the owner).

usage: helmcheck.py KEY                                   a helm of the study (torva, v5, umbralA, ... see h2.py)
       helmcheck.py item:ID --rest "item:BODY+item:LEGS"  a real helm in an outfit
       helmcheck.py HELM-labelled.bin SLOTSPEC --rest "model:..@body+arms+model:..@legs+.." --body CUIRASS.bin [--name N]
       helmcheck.py --all                                 every helm of the study, one line each (from data/measures.json)
   --rest  the rest of the outfit as pose.py pieces (the helm is judged IN the outfit); --body the body piece's mesh
   --json OUT.json   write every measure

Band = minimum to maximum of the six accepted helms (Torva 26382, Oathplate 30750, Justiciar 22326, Neitiznot
faceguard 24271, Serpentine 12931, and our Meridian v5, the only helm of ours the owner passed). `wide` = the same
with five more real end-game helms (Inquisitor's great helm, Masori mask (f), Dharok's helm, Statius's full helm,
Armadyl helmet), which the owner has NOT judged. Class says how cleanly the line separated:
  CLEAN   all seven rejected helms outside the band (both Umbral helms and the five earlier ones)
  RECENT  both Umbral helms outside; the earlier rejected helms not all outside
  A       Umbral A (the one the owner judged in the fashionscape test) outside; Umbral B inside
Exit code: 0 if every CLEAN line is inside, 1 otherwise. The other lines are printed as warnings.
"""
import sys, pathlib, json
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.dont_write_bytecode = True

# key, class, side ('min' = must reach the band's minimum, 'max' = must not exceed its maximum), what
LINES = [
    ('fold45_share', 'CLEAN', 'min', 'SHARP: share of the shell\'s edge length folded more than 45 degrees'),
    ('fold45_dome', 'CLEAN', 'min', 'SHARP: the same on the skull alone (not the face, not the signature)'),
    ('g_front_range_vs_body', 'RECENT', 'min', 'OUTFIT: tone range of the helm / tone range of the body under it (front, game size)'),
    ('g_front_p95_minus_body_p95', 'RECENT', 'min', 'OUTFIT: the helm\'s lights minus the body\'s lights (95th percentile of grey)'),
    ('g_front_body_range', 'RECENT', 'max', 'OUTFIT: tone range of the body under the helm (a stand-in louder than any accepted body)'),
    ('g_side_void_big', 'RECENT', 'max', 'DARK SIDE: largest near-black patch, share of the helm, side view at game size'),
    ('g_quarter_void_big', 'A', 'max', 'DARK SIDE: the same in the front quarter view'),
    ('bk_sideback_under20', 'A', 'max', 'DARK SIDE: share of side- and back-facing steel painted under lightness 20'),
    ('g_side_band_ratio', 'A', 'max', 'STRIPES: level tone bands met going down the helm / upright bands going across (side)'),
    ('g_front_level_ratio', 'A', 'max', 'STRIPES: level tone steps / upright tone steps on the helm (front)'),
    ('g_min_band_ratio', 'A', 'max', 'STRIPES: the least striped of the five views'),
    ('pr_h_over_wskull', 'A', 'max', 'PROPORTION: whole height of the helm / width of the shell at the skull'),
    ('pr_above_over_wskull', 'A', 'max', 'PROPORTION: rise above the crown / width of the shell at the skull'),
    ('br_min_L', 'A', 'max', 'DEPTH: darkest paint on the helm (accepted helms reach lightness 0 to 2)'),
]
CORE = ['torva', 'oathplate', 'justiciar', 'faceguard', 'serpentine', 'v5']
WIDE = CORE + ['inquisitor', 'masori', 'dharok', 'statius', 'armadyl']


def bands():
    D = json.load(open(HERE / 'data' / 'measures.json'))
    out = {}
    for key, cls, side, what in LINES:
        c = [D[k][key] for k in CORE if D[k].get(key) is not None]; w = [D[k][key] for k in WIDE if D[k].get(key) is not None]
        out[key] = (min(c), max(c), min(w), max(w))
    return out, D


def verdict(v, lo, hi, side):
    if v is None: return 'n/a'
    if side == 'min': return 'inside' if v >= lo - 1e-9 else 'OUTSIDE'
    return 'inside' if v <= hi + 1e-9 else 'OUTSIDE'


def report(name, r, B, out=print):
    out(f'helmcheck of {name}')
    out(f"  {'class':7}{'measure':28}{'value':>8}  {'accepted six':>16}  {'with five more':>16}  verdict   what")
    bad = {'CLEAN': 0, 'RECENT': 0, 'A': 0}
    for key, cls, side, what in LINES:
        lo, hi, wlo, whi = B[key]; v = r.get(key)
        vd = verdict(v, lo, hi, side)
        if vd == 'OUTSIDE':
            bad[cls] += 1
            if verdict(v, wlo, whi, side) == 'inside': vd = 'OUTSIDE*'
        band = f'>= {lo:g}' if side == 'min' else f'<= {hi:g}'
        wband = f'>= {wlo:g}' if side == 'min' else f'<= {whi:g}'
        out(f"  {cls:7}{key:28}{('-' if v is None else v):>8}  {band:>16}  {wband:>16}  {vd:9} {what}")
    n = sum(bad.values())
    out(f"  RESULT: {'inside every band' if n == 0 else 'OUTSIDE on ' + str(n) + ' of ' + str(len(LINES))} "
        f"(CLEAN {bad['CLEAN']}, RECENT {bad['RECENT']}, A {bad['A']}).  * = inside the wider band of eleven.  A guard, not a judge.")
    return bad


def measure_one(key):
    import h2, measures as M, extra as X
    P = M.pictures([key]); mh = h2.mesh(key); mb = h2.body_mesh(key)
    r = dict(group=h2.H[key]['group'], name=h2.H[key]['name'])
    r.update(M.pix(key, P)); r.update(M.outline(key, mh)); r.update(M.mesh_measures(key, mh, mb)); r.update(M.fit_measures(key))
    r.update(X.sig_pixels([key])[key]); r.update(X.baked(key, mh)); r.update(X.brief_row(key, mh))
    try: r.update(X.clean(key))
    except Exception: pass
    r['pr_h_over_wskull'] = round(r['sil_front_h'] / r['fit_w_skull'], 2)
    r['pr_above_over_wskull'] = round(r['fit_above'] / r['fit_w_skull'], 2)
    return r


if __name__ == '__main__':
    a = sys.argv[1:]
    def opt(n):
        if n in a:
            i = a.index(n); v = a[i + 1]; del a[i:i + 2]; return v
    rest = opt('--rest'); body = opt('--body'); name = opt('--name'); js = opt('--json')
    B, D = bands()
    if '--all' in a:
        import h2
        print(f"{'helm':12}{'group':6}" + ''.join(f'{k[-14:]:>15}' for k, *_ in LINES) + '   outside (CLEAN/RECENT/A)')
        print(f"{'band':12}{'':6}" + ''.join(f"{('>=' + format(B[k][0], 'g')) if s == 'min' else ('<=' + format(B[k][1], 'g')):>15}" for k, c, s, w in LINES))
        for key in [k for k in h2.ORDER if k in D] + [k for k in D if k not in h2.ORDER]:
            r = D[key]; bad = {'CLEAN': 0, 'RECENT': 0, 'A': 0}; cells = ''
            for k, cls, side, what in LINES:
                v = r.get(k); vd = verdict(v, B[k][0], B[k][1], side)
                if vd == 'OUTSIDE': bad[cls] += 1
                cells += f"{(str(v) + ('!' if vd == 'OUTSIDE' else '')):>15}"
            print(f"{key:12}{r['group']:6}{cells}   {bad['CLEAN']}/{bad['RECENT']}/{bad['A']}")
        raise SystemExit(0)
    if not a: raise SystemExit(__doc__)
    import h2
    if a[0] in h2.H:
        key = a[0]
    elif a[0].startswith('item:'):
        import lib as L
        hid = int(a[0][5:]); key = f'item{hid}'
        if not rest: raise SystemExit('--rest "item:BODY+item:LEGS" is needed for a real helm outside the study')
        bid = int(rest.split('+')[0].split(':')[1].split('@')[0])
        h2.H[key] = dict(group='?', name=L.item_name(hid), helm=a[0], rest=rest, mesh=('real', hid), body=('real', bid),
                         spec='head+hair+jaw' if 11 in L.wear_positions(hid)[1:] else 'head+hair')
    else:
        if not (rest and body): raise SystemExit('--rest and --body are needed for a model')
        key = name or pathlib.Path(a[0]).stem
        h2.ours(key, '?', key, pathlib.Path(a[0]).as_posix(), a[1] if len(a) > 1 else 'head+hair+jaw', rest, body)
    r = measure_one(key)
    bad = report(h2.H[key]['name'], r, B)
    if js: json.dump(r, open(js, 'w'), indent=1)
    sys.exit(1 if bad['CLEAN'] else 0)
