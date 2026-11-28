"""fit.py: does a helm sit on the head as the owner's five reference helms do?

usage: fit.py HELM-labelled.bin SLOTSPEC [--body BODY-labelled.bin | --body item:ID] [--json OUT.json] [--quiet]
       fit.py item:ID                      (a real helm; its slots and usual body are read from the cache / the study)
   SLOTSPEC as for pose.py: head+hair+jaw (the helm hides hair and jaw) or head+hair (the jaw kit stays).
   --body: the body piece it is worn with (default: the Meridian cuirass v2). Used for the collar and shoulder lines.

Prints one line per measure: the helm's value, the range of the five references (Torva full helm 26382, Justiciar
faceguard 22326, Serpentine helm 12931, Neitiznot faceguard 24271, Oathplate helm 30750), the range of all 25 real
helms measured (the five, ten more coveted, ten plain), and PASS or FAIL against the FIVE. Lines marked * are the
fit lines proper (stand-off and the side profile): the ones every one of our seven rejected helms failed. The others
describe size and are reported so a reviewer sees them; a horned or winged helm may leave them on purpose.
Exit code 0 if every * line passes, 1 otherwise. Numbers are defined in fitlib.py. Mesh only: nothing is drawn.
Ranges come from fit-measures.json beside this file (made by fit_study.py)."""
import sys, pathlib, json
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
import lib as L, fitlib as F

HERE = pathlib.Path(__file__).resolve().parent
# (key, core?, what it is, tolerance added to both ends of the five's range)
LINES = [
    ('so_med',        True,  'stand-off from the bare head, median over the skull and face (units)'),
    ('so_p10',        True,  'stand-off where the helm is closest (10th percentile)'),
    ('so_brow',       True,  'stand-off at the brow'),
    ('so_temple',     True,  'stand-off at the temples'),
    ('so_cheek',      True,  'stand-off at the cheeks'),
    ('so_crown',      True,  'stand-off at the crown'),
    ('so_jaw',        True,  'stand-off at the jaw (closed helms)'),
    ('so_nape',       False, 'stand-off at the nape'),
    ('chin_overhang', True,  'side view: the helm\'s face stands this far in front of the neck and collar under it'),
    ('pinch_s_ratio', True,  'side view: narrowest depth of helm + collar between 158 and 175, as a share of the skull\'s depth'),
    ('cover',         False, 'share of the head\'s directions the shell covers'),
    ('wid',           False, 'whole width / bare head width (22)'),
    ('hgt',           False, 'height above the collar / bare head height (32)'),
    ('dep',           False, 'whole depth / bare head depth (24)'),
    ('w_skull',       False, 'width of the shell at the skull, median of heights 180..194 (units)'),
    ('d_skull',       False, 'depth of the shell at the skull, median of heights 180..194 (units)'),
    ('fore',          False, 'reach in front of the bare face (units)'),
    ('above',         False, 'rise above the bare crown, crest included (units)'),
    ('low_front',     False, 'height of the lowest point in front (collar 164)'),
    ('low_back',      False, 'height of the lowest point behind'),
    ('helm_to_shoulder', False, 'shell width at the skull / the body piece\'s shoulder width'),
]


def ranges():
    rows = json.load(open(HERE / 'fit-measures.json'))['helms']
    out = {}
    for key, core, what in LINES:
        r5 = [d[key] for d in rows if d['group'] == 'R' and d.get(key) is not None]
        r25 = [d[key] for d in rows if d['group'] in 'RCP' and d.get(key) is not None]
        out[key] = (min(r5), max(r5), min(r25), max(r25))
    return out


def load(arg, spec):
    if arg.startswith('item:'):
        hid = int(arg[5:]); wp = L.wear_positions(hid)
        return L.mesh_real(hid), (8 in wp[1:], 11 in wp[1:]), f'{L.item_name(hid)} {hid}'
    return L.mesh_ours(arg), ('hair' in spec, 'jaw' in spec), pathlib.Path(arg).name


def body_for(arg, opt):
    if opt:
        return L.mesh_real(int(opt[5:])) if opt.startswith('item:') else L.mesh_ours(opt)
    if arg.startswith('item:'):
        hid = int(arg[5:])
        for r in L.REFS + L.COVETED + L.PLAIN:
            if r[1] == hid: return L.mesh_real(r[3])
    return L.mesh_ours(L.CUIRASS_V2)


def check(arg, spec='head+hair+jaw', body=None, quiet=False, out=print):
    m, hides, name = load(arg, spec)
    d = F.measure(m, body_for(arg, body), hides)
    R = ranges()
    fails = core_fails = 0; rows = []
    if not quiet:
        out(f'fit of {name}  (hides hair {hides[0]}, jaw {hides[1]}; {d["faces"]} faces)')
        out(f"{'':2}{'measure':17}{'value':>8}  {'the five':>14}  {'25 real helms':>14}  verdict   what")
    for key, core, what in LINES:
        lo, hi, lo25, hi25 = R[key]; v = d.get(key)
        if v is None:
            verdict = 'n/a'
        else:
            ok = lo - 1e-9 <= v <= hi + 1e-9
            verdict = 'PASS' if ok else 'FAIL'
            if not ok:
                fails += 1; core_fails += core
        rows.append((key, core, v, lo, hi, verdict))
        if not quiet:
            out(f"{'*' if core else ' ':2}{key:17}{('-' if v is None else v):>8}  {f'{lo:g} .. {hi:g}':>14}  {f'{lo25:g} .. {hi25:g}':>14}  {verdict:8}  {what}")
    if not quiet:
        out(f'RESULT: {"PASS" if core_fails == 0 else "FAIL"}: {core_fails} of the * fit lines fail; {fails - core_fails} of the size lines are outside the five')
    return d, rows, core_fails, fails


if __name__ == '__main__':
    a = [x for x in sys.argv[1:]]
    def opt(n):
        if n in a:
            i = a.index(n); v = a[i + 1]; del a[i:i + 2]; return v
    body = opt('--body'); js = opt('--json'); quiet = '--quiet' in a
    a = [x for x in a if x != '--quiet']
    if not a:
        raise SystemExit(__doc__)
    d, rows, cf, f = check(a[0], a[1] if len(a) > 1 else 'head+hair+jaw', body, quiet)
    if js:
        json.dump({k: v for k, v in d.items() if not k.startswith('_')}, open(js, 'w'), indent=1)
    sys.exit(1 if cf else 0)
