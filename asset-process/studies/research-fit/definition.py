"""definition.py: is a worn set divided into readable parts as the owner's reference sets are?

usage: definition.py OUTFIT_ENTRY [--json OUT.json] [--keep DIR]
   OUTFIT_ENTRY as for pose.py, one figure: "model:path/to/helm.bin@head+hair+jaw+model:path/to/body.bin@body+arms+item:1079"

Draws the entry at game size (pose.py --zoom 300, the game's own painting of a player) standing and through all 8
frames of walk 819 and run 824, from the front (yaw 192) and the back (yaw 1216): 34 pictures. Prints, per measure,
the median over the 34, the front and back medians, the range of the five reference sets (Torva; Justiciar; Serpentine
helm on Bandos; Neitiznot faceguard on Bandos; Oathplate) and the range of the references of the entry's own VALUE
CLASS, and PASS or FAIL against that class:
   dark class  (the figure's median grey under 35): Torva, Oathplate
   light class (35 or more):                        Justiciar, Serpentine on Bandos, faceguard on Bandos
A line passes when the entry is at least as divided as the LEAST divided reference of its class (step, level steps,
lines, regions: at or above the class minimum; largest undivided region: at or below the class maximum) and at least
as dark in its darks (p05 at or below the class maximum; dark share at or above the class minimum). Being MORE
divided than the references is reported as 'over' and is not a failure: the owner chose our earlier set, which is.
Exit code 0 if no line fails. Measures are defined in deflib.py; ranges come from def-measures.json beside this file."""
import sys, pathlib, json, tempfile
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
import lib as L, deflib as D

HERE = pathlib.Path(__file__).resolve().parent
REFSETS = ['torva', 'justiciar', 'serpentine', 'faceguard', 'oathplate']
CLASS = dict(dark=['torva', 'oathplate'], light=['justiciar', 'serpentine', 'faceguard'])
# (key, direction: +1 = must be at least the class minimum, -1 = must be at most the class maximum, what)
LINES = [
    ('step',       +1, 'tone steps of 24 grey levels or more, per 100 figure pixels'),
    ('step_level', +1, 'of those, level ones (bands across trunk and limbs)'),
    ('step40',     +1, 'strong steps (40 or more), per 100 figure pixels'),
    ('lines',      +1, 'pixels on a line (dark, light or second material), per 100 figure pixels'),
    ('line_dark',  +1, 'pixels on a dark line, per 100'),
    ('regions',    +1, 'readable regions (even patches of 12 pixels or more)'),
    ('reg_big',    -1, 'share of the figure in its largest undivided patch'),
    ('p05',        -1, 'the darkest darks: 5th percentile of grey (0..255)'),
    ('dark',       +1, 'share of the figure darker than grey 30'),
    ('range',      +1, 'value range: 95th minus 5th percentile of grey'),
]


def refs(suffix=''):
    A = json.load(open(HERE / 'def-measures.json'))['300']
    return {n: A[n + suffix]['summary'] for n in REFSETS}


def check(entry, quiet=False, keep=None, out=print):
    tmp = pathlib.Path(keep) if keep else pathlib.Path(tempfile.mkdtemp(prefix='definition-'))
    res, cells = D.run_entries(tmp / 'd.png', [entry])
    s = D.summarise(res[0])
    R = refs('' if any(p.split('@')[-1].split('+')[0] in ('head', '0') or _is_helm(p) for p in _pieces(entry)) else '-nohelm')
    cls = 'dark' if s['p50'] < 35 else 'light'
    fails = 0; rows = []
    if not quiet:
        out(f'definition of {entry}')
        out(f'value class: {cls} (median grey {s["p50"]}); compared with {", ".join(CLASS[cls])}; 34 pictures at zoom 300')
        out(f"{'measure':12}{'all':>8}{'front':>8}{'back':>8}  {'the five':>16}  {'its class':>16}  verdict  what")
    for key, sign, what in LINES:
        five = [R[n][key] for n in REFSETS]; own = [R[n][key] for n in CLASS[cls]]
        v = s[key]
        if sign > 0:
            ok = v >= min(own) - 1e-9; over = v > max(five)
        else:
            ok = v <= max(own) + 1e-9; over = v < min(five)
        verdict = ('PASS' if ok else 'FAIL') + (' over' if ok and over else '')
        fails += (not ok)
        rows.append((key, v, min(five), max(five), min(own), max(own), verdict))
        if not quiet:
            out(f"{key:12}{v:>8}{s[key + '_front']:>8}{s[key + '_back']:>8}  {f'{min(five):g} .. {max(five):g}':>16}  {f'{min(own):g} .. {max(own):g}':>16}  {verdict:8} {what}")
    if not quiet:
        out(f"means of division: steps between two materials {s['step_mat']:.2f}, onto a dark {s['step_dark']:.2f}, within one lit material {s['step_tone']:.2f}; "
            f"line pixels dark {s['line_dark']}, light {s['line_light']}, second material {s['line_mat']}")
        out(f'RESULT: {"PASS" if not fails else "FAIL"}: {fails} of {len(LINES)} lines below the least of its class')
    return s, rows, fails


def _pieces(entry):
    out = []; cur = ''
    for tok in entry.split('+'):
        if tok.startswith(('item:', 'model:', 'none')):
            if cur: out.append(cur)
            cur = tok
        else:
            cur += '+' + tok
    if cur: out.append(cur)
    return out


def _is_helm(piece):
    if piece.startswith('item:') and '@' not in piece:
        try: return L.wear_positions(int(piece[5:]))[0] == 0
        except Exception: return False
    return False


if __name__ == '__main__':
    a = sys.argv[1:]
    def opt(n):
        if n in a:
            i = a.index(n); v = a[i + 1]; del a[i:i + 2]; return v
    js = opt('--json'); keep = opt('--keep')
    if not a:
        raise SystemExit(__doc__)
    s, rows, fails = check(a[0], keep=keep)
    if js:
        json.dump(s, open(js, 'w'), indent=1)
    sys.exit(1 if fails else 0)
