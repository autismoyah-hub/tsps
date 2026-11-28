import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from patchlib import Patch
P = Patch(_AP.fix(r'<ROOT>\tools\label_armour.py'))
rep = P.rep
rep("""usage: label_armour.py OURS.bin SLOT OUT.bin [--template ITEM_ID] [--near N] [--no-balance] [--no-face-labels] [--quiet]
       label_armour.py --check ITEM_ID [--template ITEM_ID]     (strip a real item's labels, relabel, compare)
""", """usage: label_armour.py OURS.bin SLOT OUT.bin [--sleeveless] [--template ITEM_ID] [--near N] [--no-balance]
                       [--no-face-labels] [--quiet]
       label_armour.py --check ITEM_ID [--sleeveless] [--template ITEM_ID]   (strip a real item's labels, relabel, compare)
""")
rep("""It prints the histograms, the joint pivots""", """--sleeveless (body slot only): the piece is a body piece that KEEPS THE FIGURE'S OWN ARMS, i.e. it is worn as
`model:PIECE.bin@body` WITHOUT `+arms` (a chainbody, a leather body, a shirt; a real item of this kind declares
wear positions [4, -1, -1], which `asset_pose.exe CACHE info item:ID` prints). Use it for every such piece. Then:
  - only the body labels are given: chest 8 everywhere, and 2 / 4 / 5 at the seam points (86 only when a template
    supplies it). No vertex gets an arm label (17 to 26, 67, 68) whatever its x, not even from a template, and so
    no face gets an arm face label (9 to 14). Without the flag anything at |x| >= 19 is called arm, which on a
    sleeveless piece tears the side walls off with the swinging arms.
  - the shoulder and elbow pivots are not balanced, reported or warned about: the arms kit defines them, so THE
    ARMS KIT MUST STAY (do not hide slot 6 with a sleeveless piece); the tool prints that instead.
  - a seam label is given only to a vertex at the EXACT seam coordinates of LABELS.md (the four collar points for
    2, the waist points for 4 and 5), not to one within 1 unit of them: a pattern vertex beside the collar is chest.
It prints the histograms, the joint pivots""")
rep('''def rule_labels(slot, V, F):
    """V (n,3) cache coordinates. Returns labels (n,) and {vertex index: reason it is unsure}."""''',
    '''ARM_LABELS = frozenset(range(17, 27)) | {67, 68}      # vertex labels that follow an arm
ARM_FACE_LABELS = frozenset(range(9, 15))             # face labels of the arm regions


def pivots(slot, sleeveless=False):
    """The slot's joint pivots; a sleeveless body piece defines none of the arm's (the arms kit does)."""
    rows = PIVOTS.get(slot, [])
    if sleeveless and slot == 4:
        rows = [r for r in rows if not (set(r[1]) & ARM_LABELS)]
    return rows


def rule_labels(slot, V, F, sleeveless=False):
    """V (n,3) cache coordinates. Returns labels (n,) and {vertex index: reason it is unsure}.
    sleeveless (body slot): no arm region at all, and seam labels only at the exact seam coordinates."""''')
rep("""        elif slot == 4:
            edge = ARM_X""", """        elif slot == 4 and sleeveless:
            L[i] = 8
            if h[i] < 106: unsure[i] = 'below the waist: given chest 8 (hangs rigidly); a long skirt needs a template'
        elif slot == 4:
            edge = ARM_X""")
rep("""        for i in np.nonzero(d <= 1)[0]:""", """        for i in np.nonzero(d <= (0 if sleeveless and slot == 4 else 1))[0]:""")
rep("""def refine(L, unsure, V, TV, TL, near=3.0, far=8.0, slot=None, close=True):""",
    """def refine(L, unsure, V, TV, TL, near=3.0, far=8.0, slot=None, close=True, sleeveless=False):""")
rep("""    pivotal = {l for _, labels, _, _ in PIVOTS.get(slot, []) for l in labels}""",
    """    pivotal = {l for _, labels, _, _ in pivots(slot, sleeveless) for l in labels}""")
rep("""            stats['agree'] += 1; continue
""", """            stats['agree'] += 1; continue
        if sleeveless and t in ARM_LABELS:
            stats['template arm label refused (sleeveless)'] += 1; continue
""")
rep("""def pivot_report(slot, L, V):
    V = np.asarray(V, float); rows = []
    for name, labels, want, need in PIVOTS.get(slot, []):""", """def pivot_report(slot, L, V, sleeveless=False):
    V = np.asarray(V, float); rows = []
    for name, labels, want, need in pivots(slot, sleeveless):""")
rep("""def balance_pivots(slot, L, V, targets=None, tol=1.0, keep=3):""",
    """def balance_pivots(slot, L, V, targets=None, tol=1.0, keep=3, sleeveless=False):""")
rep("""    P = V * np.array([1, -1, 1])
    for name, labels, want, need in PIVOTS.get(slot, []):""", """    P = V * np.array([1, -1, 1])
    for name, labels, want, need in pivots(slot, sleeveless):""")
rep("""def label_model(slot, V, F, template=None, near=3.0, balance=True):
    L, unsure = rule_labels(slot, V, F)""", """def label_model(slot, V, F, template=None, near=3.0, balance=True, sleeveless=False):
    sleeveless = bool(sleeveless) and slot == 4
    L, unsure = rule_labels(slot, V, F, sleeveless)""")
rep("""        L, unsure, stats = refine(L, unsure, V, TV, TL, near, 8.0, slot, close)""",
    """        L, unsure, stats = refine(L, unsure, V, TV, TL, near, 8.0, slot, close, sleeveless)""")
rep("""        L, notes = balance_pivots(slot, L, V, targets)""",
    """        L, notes = balance_pivots(slot, L, V, targets, sleeveless=sleeveless)""")
rep("""        stats['pivot notes'] = notes
    return L, unsure, stats""", """        stats['pivot notes'] = notes
    if sleeveless and (set(int(l) for l in L) & ARM_LABELS):
        raise SystemExit('internal error: a sleeveless piece received an arm label')
    return L, unsure, stats""")
rep("""def report(slot, L, FL, unsure, V, stats, quiet=False, out=print):""",
    """def report(slot, L, FL, unsure, V, stats, quiet=False, out=print, sleeveless=False):""")
rep("""    for name, labels, c, want, need in pivot_report(slot, np.asarray(L), V):""",
    """    if sleeveless and slot == 4:
        out('  sleeveless: no arm labels (17-26) and no arm face labels (9-14) given; shoulder and elbow pivots are not '
            'defined, balanced or checked here. THE ARMS KIT MUST STAY: wear this piece as @body without +arms.')
    elif slot == 4 and not (set(hist) & {17, 19, 22, 23}):
        out("  note: no elbow or forearm vertex was found. If this piece keeps the figure's own arms (worn @body "
            'without +arms), rerun with --sleeveless.')
    for name, labels, c, want, need in pivot_report(slot, np.asarray(L), V, sleeveless):""")
rep("""def check(item_id, template=None, near=3.0, quiet=True, out=print):""",
    """def check(item_id, template=None, near=3.0, quiet=True, out=print, sleeveless=False):""")
rep("""    L, unsure, stats = label_model(slot, V, F, template, near)
    FL = face_labels(slot, L, F)
    same = L == T""", """    if slot == 4 and 6 not in d['wear_positions'][1:] and not sleeveless:
        out(f'note: item {item_id} is worn in the body slot and does not blank the arms (wear positions {d["wear_positions"]}): it is a sleeveless piece, check it with --sleeveless')
    if sleeveless and slot == 4 and 6 in d['wear_positions'][1:]:
        out(f'note: item {item_id} blanks the arms slot (wear positions {d["wear_positions"]}): it is NOT sleeveless; --sleeveless leaves its arms on the chest')
    L, unsure, stats = label_model(slot, V, F, template, near, sleeveless=sleeveless)
    FL = face_labels(slot, L, F)
    same = L == T""")
rep("""    if not quiet: report(slot, L, FL, unsure, V, stats, True, out)""",
    """    if not quiet: report(slot, L, FL, unsure, V, stats, True, out, sleeveless)""")
rep("""        check(int(a[1]), template, near, quiet='--quiet' in a)""",
    """        check(int(a[1]), template, near, quiet='--quiet' in a, sleeveless='--sleeveless' in a)""")
rep("""    L, unsure, stats = label_model(slot, V, F, template, near, '--no-balance' not in a)
    FL = None if '--no-face-labels' in a else face_labels(slot, L, F)""", """    sleeveless = '--sleeveless' in a
    if sleeveless and slot != 4: raise SystemExit('--sleeveless applies to the body slot only')
    L, unsure, stats = label_model(slot, V, F, template, near, '--no-balance' not in a, sleeveless)
    FL = None if '--no-face-labels' in a else face_labels(slot, L, F)
    if sleeveless and FL is not None and (set(FL) & ARM_FACE_LABELS):
        raise SystemExit('internal error: a sleeveless piece received an arm face label')""")
rep("""(f', template item {template}' if template else ', rule only')
          + f': {len(V)} vertices""", """(f', template item {template}' if template else ', rule only')
          + (', sleeveless' if sleeveless else '') + f': {len(V)} vertices""")
rep("""    report(slot, L, FL if FL is not None else m['face_skins'], unsure, V, stats, '--quiet' in a)""",
    """    report(slot, L, FL if FL is not None else m['face_skins'], unsure, V, stats, '--quiet' in a, sleeveless=sleeveless)""")
P.save()
