"""Give one of our worn armour models its animation labels BY RULE (the label atlas), optionally refined from a real item.

usage: label_armour.py OURS.bin SLOT OUT.bin [--sleeveless] [--template ITEM_ID] [--near N] [--no-balance]
                       [--no-face-labels] [--quiet]
       label_armour.py --check ITEM_ID [--sleeveless] [--template ITEM_ID]   (strip a real item's labels, relabel, compare)

SLOT  head | body | legs | feet | hands | cape   (or the client's slot number 0, 4, 7, 10, 9, 1)

The model must be in worn space (the standard male figure: feet at height 0, crown near 196, x = 0 the middle,
the character's RIGHT side at negative x, its FRONT at negative z; cache y is down, so height = -y).
Only vertex labels and face labels are written. Vertices, faces, colours, per-face priorities, flat/smooth render
types, alpha, face textures and depth offsets are carried through unchanged (the script stops if the model has a
complex texture mapping, which our encoder cannot rewrite).

How the labels are chosen (author-pack/labels/LABELS.md has the pictures and the reasons):
  1. SEAM POINTS. Real pieces share exact coordinates where two slots meet (neck ring, waist ring); a vertex within
     1 unit of one takes that point's label.
  2. REGION RULES. Each slot is cut by planes at the joints: a vertex takes the label of the region it lies in. Only
     the standard rigid set of the slot is used (no skirt, pauldron or hem extras).
  3. --template ITEM_ID (a real item of the same family and slot). The script first measures how much of our mesh
     the template covers (share of our vertices with a template vertex within 2 units).
     Covered 75% or more (the same shape, e.g. boots on boots): a vertex takes the label of the nearest template
     vertex when it is within --near units (default 3), or when the rule was unsure and one is within 8 units, or
     when the template's label moves exactly like the rule's (same transform class).
     Covered less (a different shape): only the last two cases apply, and the labels that define joint pivots
     are left where the rule put them. (Copying pivot labels from a differently shaped item moved the joints by
     6 to 9 units in the tests and made the motion worse than the rule alone.)
  4. PIVOT BALANCE (off with --no-balance). A joint turns about the centroid of the vertices carrying its pivot
     label (17/23 elbows, 21/25 shoulders, 29 hips, 40/42 hip joints, 34+31 / 35+37 knees, 47/48 toe hinges).
     Where that centroid is more than 1 unit from where real armour puts it (the template's own pivot when a
     template is given), vertices are moved one at a time to the sibling label that receives the same transforms
     (17->20, 21->20, 35->43, 37->38, 40->43, 47->38, 29->30 and mirrors) until it is, or no change helps.
     Measured against real items this neither helps nor hurts on average (they vary by 3-5 units around the
     median); it is there so that an authored piece turns about the standard joint centres.
  5. FACE LABELS follow from the vertex labels (the body region the face belongs to; they are only used by
     sequences that fade parts of the figure).
--sleeveless (body slot only): the piece is a body piece that KEEPS THE FIGURE'S OWN ARMS, i.e. it is worn as
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
It prints the histograms, the joint pivots the labelling produces against where real armour puts them, and every
vertex it was unsure of (within 2 units of a boundary, on the centre line of a limb split, or in a zone where real
pieces disagree with each other). A wrong label only shows in motion: look at the result with pose.py.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np

SLOTS = {'head': 0, 'helm': 0, 'cape': 1, 'body': 4, 'torso': 4, 'legs': 7, 'hands': 9, 'gloves': 9, 'feet': 10, 'boots': 10}
SLOT_NAME = {0: 'head', 1: 'cape', 4: 'body', 7: 'legs', 9: 'hands', 10: 'feet'}

# ---- the atlas, as data ------------------------------------------------------------------------------------------
# Short names (the character's right is negative x).
NAME = {0: 'root anchor', 1: 'head', 2: 'neck base ring', 3: 'neck', 4: 'abdomen', 5: 'waist ring back', 8: 'chest',
        9: 'cape hinge 2', 10: 'cape top hinge', 11: 'cape upper panel', 12: 'cape hem', 13: 'cape hinge 4', 14: 'cape hinge 3',
        15: 'cape lower panel', 16: 'cape middle panel', 17: 'R elbow ring', 19: 'R forearm', 20: 'R upper arm', 21: 'R shoulder',
        22: 'L forearm', 23: 'L elbow ring', 25: 'L shoulder', 26: 'L upper arm', 27: 'R hand', 28: 'L hand', 29: 'hip ring',
        30: 'waist ring front', 31: 'R knee ring lower', 32: 'R shin', 34: 'R knee ring upper', 35: 'L knee ring upper',
        37: 'L knee ring lower', 38: 'L shin', 39: 'crotch centre', 40: 'L hip', 41: 'pelvis', 42: 'R hip', 43: 'L thigh',
        44: 'R thigh', 45: 'R toes', 46: 'L toes', 47: 'L foot hinge', 48: 'R foot hinge', 57: 'skirt centre (left thigh)',
        78: 'skirt centre (pelvis)'}
# Labels that receive exactly the same move/turn/scale transforms in every sequence examined (93 player sequences,
# skeleton 0). Two labels with one class number differ only in which joint pivot their vertices help to define.
CLASS = {0: 0, 1: 1, 2: 2, 3: 1, 4: 3, 5: 3, 6: 3, 7: 4, 8: 2, 9: 5, 10: 2, 11: 5, 12: 6, 13: 7, 14: 8, 15: 7, 16: 8, 17: 9, 18: 9,
         19: 10, 20: 9, 21: 9, 22: 11, 23: 12, 24: 12, 25: 12, 26: 12, 27: 13, 28: 14, 29: 3, 30: 3, 31: 15, 32: 15, 33: 16,
         34: 17, 35: 18, 36: 19, 37: 20, 38: 20, 39: 21, 40: 18, 41: 22, 42: 17, 43: 18, 44: 17, 45: 23, 46: 24, 47: 20, 48: 15,
         49: 2, 57: 21, 58: 25, 59: 2, 60: 2, 67: 26, 68: 27, 69: 2, 71: 28, 74: 22, 75: 22, 76: 20, 77: 15, 78: 22, 79: 22,
         80: 17, 81: 18, 82: 20, 83: 15, 84: 22, 85: 22, 86: 2, 89: 28, 90: 28, 91: 2, 92: 2, 93: 25, 94: 2, 126: 29, 127: 29,
         128: 28, 157: 1, 158: 30, 166: 17, 167: 18, 168: 22, 181: 31, 197: 28, 255: 32}
# Coarser classes: the same transforms in stand 808, walk 819, run 824, slash 390, stab 386.
BASIC = {0: 0, 1: 1, 2: 2, 3: 1, 4: 3, 5: 3, 6: 3, 7: 4, 8: 2, 9: 5, 10: 2, 11: 5, 12: 6, 13: 7, 14: 8, 15: 7, 16: 8, 17: 9, 18: 9,
         19: 10, 20: 9, 21: 9, 22: 11, 23: 12, 24: 12, 25: 12, 26: 12, 27: 13, 28: 14, 29: 3, 30: 3, 31: 15, 32: 15, 33: 15,
         34: 16, 35: 17, 36: 18, 37: 18, 38: 18, 39: 17, 40: 17, 41: 19, 42: 16, 43: 17, 44: 16, 45: 20, 46: 21, 47: 18, 48: 15,
         49: 2, 57: 17, 58: 2, 59: 2, 60: 2, 67: 22, 68: 23, 69: 2, 71: 19, 74: 19, 75: 19, 76: 18, 77: 15, 78: 19, 79: 19,
         80: 16, 81: 17, 82: 18, 83: 15, 84: 19, 85: 19, 86: 2, 89: 19, 90: 19, 91: 2, 92: 2, 93: 2, 94: 2, 126: 16, 127: 16,
         128: 19, 157: 1, 158: 17, 166: 16, 167: 17, 168: 19, 181: 0, 197: 19, 255: 0}
# Exact coordinates (x, height, z) that real pieces of a slot share, with their label.
SEAMS = {
    0: {(-7, 164, -8): 2, (7, 164, -8): 2, (-7, 165, 4): 2, (7, 165, 4): 2, (-3, 169, -9): 3, (3, 169, -9): 3, (-5, 173, 2): 3, (5, 173, 2): 3},
    4: {(-7, 164, -8): 2, (7, 164, -8): 2, (-7, 165, 4): 2, (7, 165, 4): 2, (-13, 114, 8): 5, (13, 114, 8): 5,
        (-11, 114, -8): 4, (11, 114, -8): 4, (-11, 123, 5): 4, (11, 123, 5): 4, (-8, 123, -8): 4, (8, 123, -8): 4},
    7: {(-13, 114, 8): 5, (13, 114, 8): 5, (-13, 114, -8): 30, (13, 114, -8): 30,
        (-13, 110, 8): 29, (13, 110, 8): 29, (-13, 110, -8): 29, (13, 110, -8): 29},
    10: {(-6, -6, -2): 0, (6, -6, -2): 0, (0, -6, 6): 0},
    1: {(-7, 164, -8): 2},
    9: {},
}
# Joint pivots: the labels whose centroid the client uses, and where real armour of the slot puts that centroid
# (median over the real items, x, height, z). `need` = the slot must supply them (the body kit it replaces did).
PIVOTS = {
    0: [('neck (head turns about it)', (3, 2), (0, 167.8, -2.8), False)],
    4: [('right shoulder', (21,), (-23.5, 156.2, 0), True), ('left shoulder', (25,), (23.5, 156.2, 0), True),
        ('right elbow', (17,), (-27, 131, 0.7), True), ('left elbow', (23,), (27, 131, 0.7), True),
        ('neck base', (2,), (0, 164.5, -2), False)],
    7: [('hips (upper body turns about it)', (29,), (0, 110, 0), True), ('left hip', (40,), (14, 95.3, -0.3), True),
        ('right hip', (42,), (-14, 95, 0.2), True), ('left knee', (35, 37), (9.6, 53.8, -0.6), True),
        ('right knee', (34, 31), (-9.6, 53.7, -0.6), True)],
    9: [('right wrist', (27,), (-30.8, 96.3, 0.7), True), ('left wrist', (28,), (30.8, 96.3, 0.7), True)],
    10: [('left toe hinge', (47,), (9.5, 0.5, 1), True), ('right toe hinge', (48,), (-9.5, 0.5, 1), True),
         ('root (whole-body turn)', (0,), (0, -6, 0.7), False)],
    1: [('cape top hinge', (10,), (0, 157.5, 12.6), True), ('cape hinge 2', (9,), (0, 114.3, 16.3), True),
        ('cape hinge 3', (14,), (0, 73.4, 20.1), True), ('cape hinge 4', (13,), (0, 46.4, 21.4), True)],
}
# vertex label -> face label (the region a face fades with), from the real pieces' faces
FACE_OF = {1: 0, 157: 0, 2: 1, 3: 1, 49: 1, 69: 1, 94: 1, 8: 4, 59: 4, 60: 4, 91: 4, 92: 4, 58: 4, 93: 4, 10: 4, 4: 2, 5: 17, 30: 17,
           29: 17, 86: 17, 20: 9, 17: 9, 18: 9, 19: 10, 67: 10, 21: 11, 22: 12, 68: 12, 23: 14, 24: 14, 26: 14, 25: 13, 27: 15, 28: 16,
           41: 22, 39: 22, 79: 22, 74: 22, 75: 22, 40: 23, 43: 23, 35: 23, 167: 23, 57: 23, 81: 23, 42: 24, 44: 24, 34: 24, 166: 24, 80: 24,
           36: 20, 37: 20, 33: 19, 31: 19, 38: 21, 76: 21, 82: 21, 47: 21, 32: 18, 77: 18, 83: 18, 48: 18, 46: 26, 45: 25, 0: 27,
           11: 5, 9: 5, 12: 6, 13: 6, 15: 7, 14: 7, 16: 8, 6: 3, 7: 3, 78: 22, 168: 22}
# In the body slot a face made only of 4/5 corners is the abdomen (2); 5 beside 30/29 is the waist (17).
SLOT_FACE = {4: {5: 2}, 0: {8: 4}, 10: {48: 25, 47: 26}}
# In the feet slot the hinge ring belongs to the foot faces unless the shin outvotes it; handled by majority.

# ---- region rules ------------------------------------------------------------------------------------------------
# Thresholds are heights (and x, z) in model units, read from where the real pieces change label.
ARM_X, ARM_X_LOW, ARM_LOW_H = 19.0, 22.0, 118   # the arm begins at |x| >= 19 (>= 22 below height 118, where robes flare)
SHOULDER_H, UPPER_H, ELBOW_H = 147, 135, 130   # >= 147 shoulder 21/25; 135..147 upper arm 20/26; 130..135 elbow ring 17/23; below forearm 19/22
WRIST_H = 103                      # hands slot: above forearm 19/22, below hand 27/28
WAIST_H, HIP_H, PELVIS_H, CROTCH_H = 113.5, 108.5, 101, 92   # legs: >=113.5 waist 30/5; 108.5.. hip ring 29; 101.. pelvis 41; 92.. hips 40/42/39
KNEE_UP, KNEE_MID, KNEE_LOW = 62, 56, 46       # 56..62 upper knee ring 35/34; 46..56 lower knee ring 37/31; below shin 38/32
KNEECAP_Z, KNEECAP_H = -12, (50, 68)          # a vertex this far forward in the knee band is a knee cap: it goes with the shin (37/31)
TOE_S, HEEL_S, ANKLE_H, TOE_Z = -3, 1.5, 8, -10   # feet: s = height + z; z <= -10 or s <= -3 toes 46/45; -3 < s < 1.5 hinge 47/48; s >= 1.5 or height >= 8 shin 38/32
CAPE = [(160, 8), (148, 10), (122, 11), (105, 9), (85, 16), (66, 14), (54, 15), (42, 13), (-999, 12)]


def sides(V, F):
    """-1 right, +1 left, 0 centre; a vertex on the centre line takes the side of its face neighbours."""
    x = V[:, 0].astype(float)
    s = np.sign(np.where(np.abs(x) < 0.5, 0, x))
    acc = np.zeros(len(V)); cnt = np.zeros(len(V))
    for f in F:
        m = x[list(f)].mean()
        for i in f: acc[i] += m; cnt[i] += 1
    nb = np.where(cnt > 0, acc / np.maximum(cnt, 1), 0)
    weak = s == 0
    s[weak] = np.sign(np.where(np.abs(nb[weak]) < 0.75, 0, nb[weak]))
    return s.astype(int), weak


ARM_LABELS = frozenset(range(17, 27)) | {67, 68}      # vertex labels that follow an arm
ARM_FACE_LABELS = frozenset(range(9, 15))             # face labels of the arm regions


def pivots(slot, sleeveless=False):
    """The slot's joint pivots; a sleeveless body piece defines none of the arm's (the arms kit does)."""
    rows = PIVOTS.get(slot, [])
    if sleeveless and slot == 4:
        rows = [r for r in rows if not (set(r[1]) & ARM_LABELS)]
    return rows


def rule_labels(slot, V, F, sleeveless=False):
    """V (n,3) cache coordinates. Returns labels (n,) and {vertex index: reason it is unsure}.
    sleeveless (body slot): no arm region at all, and seam labels only at the exact seam coordinates."""
    V = np.asarray(V, float); n = len(V)
    x, h, z = V[:, 0], -V[:, 1], V[:, 2]
    side, centre = sides(V, F)
    L = np.zeros(n, int); unsure = {}

    def near(i, value, boundary, what, tol=2.0):
        if abs(value - boundary) < tol: unsure.setdefault(i, f'{abs(value - boundary):.1f} from the {what} boundary')

    def lr(i, right, left, what):
        if side[i] == 0:
            unsure[i] = f'on the centre line: cannot tell left from right {what}; given the right-side label'
            return right
        if centre[i]: unsure.setdefault(i, f'on the centre line: side taken from its faces ({what})')
        return right if side[i] < 0 else left
    for i in range(n):
        if slot == 0:
            L[i] = 1
            if h[i] < 166: unsure[i] = 'below the neck: label 1 turns with the head; a part that should rest on the chest needs 49 (see a template)'
        elif slot == 4 and sleeveless:
            L[i] = 8
            if h[i] < 106: unsure[i] = 'below the waist: given chest 8 (hangs rigidly); a long skirt needs a template'
        elif slot == 4:
            edge = ARM_X if h[i] >= ARM_LOW_H else ARM_X_LOW
            if abs(x[i]) >= edge and h[i] >= 100:
                if h[i] >= SHOULDER_H: L[i] = lr(i, 21, 25, 'shoulder')
                elif h[i] >= UPPER_H: L[i] = lr(i, 20, 26, 'upper arm')
                elif h[i] >= ELBOW_H: L[i] = lr(i, 17, 23, 'elbow')
                else: L[i] = lr(i, 19, 22, 'forearm')
                if h[i] >= UPPER_H - 2:
                    if abs(x[i]) - edge < 4: unsure.setdefault(i, f'shoulder seam zone ({abs(x[i]) - edge:.1f} outside the chest/arm line): real pieces disagree here')
                near(i, h[i], ELBOW_H, 'elbow/forearm'); near(i, h[i], UPPER_H, 'upper arm/elbow')
            else:
                L[i] = 8
                if h[i] >= UPPER_H - 2 and edge - abs(x[i]) < 3: unsure[i] = f'shoulder seam zone ({edge - abs(x[i]):.1f} inside the chest/arm line): real pieces disagree here'
                elif h[i] >= 100 and abs(x[i]) > 19 and edge - abs(x[i]) < 2: unsure[i] = 'close to the chest/arm line'
                if h[i] < 106: unsure.setdefault(i, 'below the waist: given chest 8 (hangs rigidly); a long skirt needs a template')
        elif slot == 7:
            if h[i] >= WAIST_H: L[i] = 30 if z[i] < 0 else 5
            elif h[i] >= HIP_H: L[i] = 29
            elif h[i] >= PELVIS_H: L[i] = 41
            elif h[i] >= CROTCH_H:
                if abs(x[i]) <= 2: L[i] = 39
                else: L[i] = lr(i, 42, 40, 'hip')
                near(i, h[i], CROTCH_H, 'hip/thigh')
            else:
                if side[i] == 0:
                    L[i] = 57 if h[i] >= KNEE_MID else 78
                    unsure[i] = 'on the centre line below the crotch (a skirt?): given a skirt-centre label; check with a skirt template'
                elif KNEECAP_H[0] <= h[i] <= KNEECAP_H[1] and z[i] <= KNEECAP_Z:
                    L[i] = lr(i, 31, 37, 'knee cap')
                    unsure.setdefault(i, 'knee cap (far forward at the knee): given the shin-side label')
                elif h[i] >= KNEE_UP: L[i] = lr(i, 44, 43, 'thigh')
                elif h[i] >= KNEE_MID: L[i] = lr(i, 34, 35, 'knee')
                elif h[i] >= KNEE_LOW: L[i] = lr(i, 31, 37, 'knee')
                else: L[i] = lr(i, 32, 38, 'shin')
                near(i, h[i], KNEE_MID, 'thigh/shin (knee)', 2.5)
            if h[i] >= PELVIS_H - 2: near(i, h[i], PELVIS_H, 'pelvis/hip'); near(i, h[i], HIP_H, 'hip ring/pelvis')
        elif slot == 9:
            L[i] = lr(i, 19, 22, 'forearm') if h[i] >= WRIST_H else lr(i, 27, 28, 'hand')
            near(i, h[i], WRIST_H, 'forearm/hand', 1.0)
        elif slot == 10:
            s = h[i] + z[i]
            if z[i] <= TOE_Z and h[i] < 14: L[i] = lr(i, 45, 46, 'toes')
            elif h[i] >= ANKLE_H or s >= HEEL_S: L[i] = lr(i, 32, 38, 'ankle')
            elif s <= TOE_S: L[i] = lr(i, 45, 46, 'toes')
            else: L[i] = lr(i, 48, 47, 'foot hinge')
            if h[i] < ANKLE_H and z[i] > TOE_Z: near(i, s, TOE_S, 'toe/hinge', 1.5); near(i, s, HEEL_S, 'hinge/ankle', 1.5)
        elif slot == 1:
            if z[i] < 8: L[i] = 8
            else: L[i] = next(l for t, l in CAPE if h[i] >= t)
            unsure[i] = 'cape rows are guessed from height alone: check against a cape template'
        else:
            raise SystemExit(f'no rules for slot {slot}')
    # seam points override the regions
    for (sx, sh, sz), l in SEAMS.get(slot, {}).items():
        d = np.abs(x - sx) + np.abs(h - sh) + np.abs(z - sz)
        for i in np.nonzero(d <= (0 if sleeveless and slot == 4 else 1))[0]:
            L[i] = l; unsure.pop(i, None)
    return L, unsure


def template_mesh(item_id):
    import reencode_item
    d, ids, parts, g = reencode_item.worn(item_id)
    return d, np.array(g['vertices'], float), np.array(g['vertex_skins'], int), np.array(g['faces'], int), np.array(g['face_skins'], int)


COVER = 0.75     # share of our vertices with a template vertex within 2 units above which the template is "the same shape"


def coverage(V, TV):
    V = np.asarray(V, float)
    return float(np.mean([np.sqrt(((TV - p) ** 2).sum(1).min()) <= 2 for p in V]))


def refine(L, unsure, V, TV, TL, near=3.0, far=8.0, slot=None, close=True, sleeveless=False):
    """Template refinement (rule 3 of the header). Returns new labels, new unsure dict, counts.
    close=False (the template is a different shape): the labels that define joint pivots are left as the rule put
    them, and a template label is taken only where the rule was unsure or where it moves exactly like the rule's."""
    V = np.asarray(V, float); L = L.copy(); unsure = dict(unsure)
    stats = collections.Counter()
    pivotal = {l for _, labels, _, _ in pivots(slot, sleeveless) for l in labels}
    for i in range(len(V)):
        d = ((TV - V[i]) ** 2).sum(1)
        # a template vertex on the other side of the body is never the right neighbour for a limb
        if abs(V[i, 0]) >= 1: d = d + np.where(np.sign(TV[:, 0]) * np.sign(V[i, 0]) < 0, 1e6, 0)
        k = int(d.argmin()); dist = float(d[k]) ** 0.5; t = int(TL[k])
        if t == L[i]:
            if dist <= far: unsure.pop(i, None)
            stats['agree'] += 1; continue
        if sleeveless and t in ARM_LABELS:
            stats['template arm label refused (sleeveless)'] += 1; continue
        if not close and (int(L[i]) in pivotal or t in pivotal):
            stats['pivot label kept as the rule put it'] += 1; continue
        if dist <= near and close:
            L[i] = t; unsure.pop(i, None); stats['template (near)'] += 1
        elif i in unsure and dist <= far:
            L[i] = t; unsure[i] = f'rule was unsure; took {t} from a template vertex {dist:.1f} away'; stats['template (rule unsure)'] += 1
        elif dist <= far and CLASS.get(t) == CLASS.get(int(L[i])):
            L[i] = t; stats['template (same transforms)'] += 1
        else:
            unsure.setdefault(i, f'rule {L[i]} against template {t} at {dist:.1f}: kept the rule')
            stats['rule kept against template'] += 1
    return L, unsure, stats


def face_labels(slot, L, F):
    over = SLOT_FACE.get(slot, {})
    out = []
    for f in F:
        c = collections.Counter(over.get(int(L[i]), FACE_OF.get(int(L[i]), 0)) for i in f)
        best = max(c.values())
        out.append(max(k for k, v in c.items() if v == best))
    return out


def pivot_report(slot, L, V, sleeveless=False):
    V = np.asarray(V, float); rows = []
    for name, labels, want, need in pivots(slot, sleeveless):
        m = np.isin(L, labels)
        if not m.any():
            rows.append((name, labels, None, want, need)); continue
        c = V[m].mean(0); c = (c[0], -c[1], c[2])
        rows.append((name, labels, c, want, need))
    return rows


# A pivot label and the label that receives the same transforms without defining the pivot.
DEMOTE = {17: 20, 23: 26, 21: 20, 25: 26, 35: 43, 34: 44, 37: 38, 31: 32, 40: 43, 42: 44, 47: 38, 48: 32, 29: 30}


def balance_pivots(slot, L, V, targets=None, tol=1.0, keep=3, sleeveless=False):
    """A joint turns about the CENTROID of the vertices carrying its pivot label, so which vertices carry it matters
    more than how many. Where the centroid is off its target (the real armour's median, or the template's own
    pivot), vertices are moved one at a time to the sibling label with the same transforms until the centroid is
    within `tol` units or no single change helps. Returns the labels and a list of notes."""
    V = np.asarray(V, float); L = L.copy(); notes = []
    P = V * np.array([1, -1, 1])
    for name, labels, want, need in pivots(slot, sleeveless):
        if not all(l in DEMOTE for l in labels): continue
        t = np.array((targets or {}).get(labels, want), float)
        idx = [int(i) for i in np.nonzero(np.isin(L, labels))[0]]
        if not idx: continue
        d0 = float(np.linalg.norm(P[idx].mean(0) - t)); moved = 0
        while len(idx) > keep:
            s = P[idx].sum(0); n = len(idx)
            d = float(np.linalg.norm(s / n - t))
            if d <= tol: break
            cand = np.linalg.norm((s - P[idx]) / (n - 1) - t, axis=1)
            k = int(cand.argmin())
            if cand[k] >= d - 1e-9: break
            L[idx[k]] = DEMOTE[int(L[idx[k]])]; idx.pop(k); moved += 1
        if moved:
            d1 = float(np.linalg.norm(P[idx].mean(0) - t))
            notes.append(f'{name}: pivot centroid was {d0:.1f} from its target; {moved} vertices given the non-pivot sibling label, now {d1:.1f}')
    return L, notes


def template_pivots(slot, TV, TL):
    out = {}
    for name, labels, want, need in PIVOTS.get(slot, []):
        m = np.isin(TL, labels)
        if m.any():
            c = TV[m].mean(0); out[labels] = (c[0], -c[1], c[2])
    return out


def label_model(slot, V, F, template=None, near=3.0, balance=True, sleeveless=False):
    sleeveless = bool(sleeveless) and slot == 4
    L, unsure = rule_labels(slot, V, F, sleeveless)
    stats = None; targets = None
    if template is not None:
        d, TV, TL, TF, TFL = template_mesh(template)
        cov = coverage(V, TV)
        close = cov >= COVER
        L, unsure, stats = refine(L, unsure, V, TV, TL, near, 8.0, slot, close, sleeveless)
        stats[f'template covers {100 * cov:.0f}% of our vertices within 2 units: ' + ('same shape, its labels and pivots are trusted' if close else 'a different shape, used only for family sub-labels and unsure vertices')] = 1
        targets = template_pivots(slot, TV, TL) if close else None
    if balance:
        L, notes = balance_pivots(slot, L, V, targets, sleeveless=sleeveless)
        if stats is None: stats = collections.Counter()
        stats['pivot notes'] = notes
    if sleeveless and (set(int(l) for l in L) & ARM_LABELS):
        raise SystemExit('internal error: a sleeveless piece received an arm label')
    return L, unsure, stats


def report(slot, L, FL, unsure, V, stats, quiet=False, out=print, sleeveless=False):
    hist = dict(sorted(collections.Counter(int(l) for l in L).items()))
    out(f'  vertex labels {hist}')
    out('     ' + '; '.join(f'{l} {NAME.get(l, "family extra")}' for l in hist))
    out(f'  face labels   {dict(sorted(collections.Counter(FL).items()))}')
    if stats is not None:
        notes = stats.get('pivot notes') or []
        rest = {k: v for k, v in stats.items() if k != 'pivot notes'}
        if rest: out(f'  template refinement: {rest}')
        for n in notes: out(f'  pivot balanced: {n}')
    if sleeveless and slot == 4:
        out('  sleeveless: no arm labels (17-26) and no arm face labels (9-14) given; shoulder and elbow pivots are not '
            'defined, balanced or checked here. THE ARMS KIT MUST STAY: wear this piece as @body without +arms.')
    elif slot == 4 and not (set(hist) & {17, 19, 22, 23}):
        out("  note: no elbow or forearm vertex was found. If this piece keeps the figure's own arms (worn @body "
            'without +arms), rerun with --sleeveless.')
    for name, labels, c, want, need in pivot_report(slot, np.asarray(L), V, sleeveless):
        lab = '+'.join(map(str, labels))
        if c is None:
            out(f'  {"WARNING" if need else "note"}: no vertex carries label {lab}, so this piece does not define the {name} pivot'
                + (': if the slot it replaces is hidden, that joint will turn about the wrong point' if need else ' (another worn piece or kit must)'))
        else:
            off = max(abs(a - b) for a, b in zip(c, want))
            out(f'  pivot {name}: labels {lab} centroid ({c[0]:.1f}, {c[1]:.1f}, {c[2]:.1f}); real armour ({want[0]}, {want[1]}, {want[2]})'
                + (f'  WARNING: {off:.1f} units off' if off > 6 else ''))
    out(f'  unsure of {len(unsure)} of {len(L)} vertices')
    if not quiet:
        for i in sorted(unsure):
            out(f'    v{i} ({V[i][0]:.0f}, {-V[i][1]:.0f}, {V[i][2]:.0f}) label {int(L[i])}: {unsure[i]}')


def check(item_id, template=None, near=3.0, quiet=True, out=print, sleeveless=False):
    """Strip a real item's labels, relabel it, and compare with the truth. Returns a dict of the numbers."""
    import reencode_item
    d, ids, parts, g = reencode_item.worn(item_id)
    slot = d['wear_positions'][0]
    V = np.array(g['vertices'], float); F = np.array(g['faces'], int); T = np.array(g['vertex_skins'], int)
    if slot == 4 and 6 not in d['wear_positions'][1:] and not sleeveless:
        out(f'note: item {item_id} is worn in the body slot and does not blank the arms (wear positions {d["wear_positions"]}): it is a sleeveless piece, check it with --sleeveless')
    if sleeveless and slot == 4 and 6 in d['wear_positions'][1:]:
        out(f'note: item {item_id} blanks the arms slot (wear positions {d["wear_positions"]}): it is NOT sleeveless; --sleeveless leaves its arms on the chest')
    L, unsure, stats = label_model(slot, V, F, template, near, sleeveless=sleeveless)
    FL = face_labels(slot, L, F)
    same = L == T
    cls = np.array([CLASS.get(int(a), -1) == CLASS.get(int(b), -2) for a, b in zip(L, T)])
    basic = np.array([BASIC.get(int(a), -1) == BASIC.get(int(b), -2) for a, b in zip(L, T)])
    conf = collections.Counter((int(t), int(l)) for t, l in zip(T, L) if t != l)
    fsame = float(np.mean(np.array(FL) == np.array(g['face_skins'])))
    wrong_unsure = sum(1 for i in np.nonzero(~basic)[0] if i in unsure)
    out(f'item {item_id} {d["name"]!r} slot {SLOT_NAME.get(slot, slot)}' + (f' template {template}' if template else ' rule only')
        + f': {len(V)} vertices; exact label {100 * same.mean():.1f}%; same transforms in all sequences {100 * cls.mean():.1f}%; '
          f'same transforms in stand/walk/run/slash/stab {100 * basic.mean():.1f}%; face labels {100 * fsame:.1f}%')
    out(f'   true labels {dict(sorted(collections.Counter(T.tolist()).items()))}')
    out(f'   confusions (true->given: count; * = moves differently in walk/run/attack) '
        + ', '.join(f'{t}->{l}: {n}{"*" if BASIC.get(t) != BASIC.get(l) else ""}' for (t, l), n in conf.most_common(14)))
    out(f'   unsure flagged {len(unsure)}; of the {int((~basic).sum())} vertices that move differently, {wrong_unsure} were flagged')
    if not quiet: report(slot, L, FL, unsure, V, stats, True, out, sleeveless)
    return dict(item=item_id, name=d['name'], slot=slot, n=len(V), exact=float(same.mean()), cls=float(cls.mean()), basic=float(basic.mean()),
                face=fsame, L=L, T=T, FL=FL, unsure=unsure, g=g, d=d)


def encode(m, L, FL):
    import osrscache
    sys.path.insert(1, _AP.fix(r'<NATIVE_ASSETS>'))
    import native_codec
    g = dict(vertices=[list(p) for p in m['vertices']], faces=[tuple(f) for f in m['faces']], colors=list(m['colors']),
             vertex_skins=[int(l) for l in L], priorities=list(m['priorities']))
    if FL is not None: g['face_skins'] = [int(l) for l in FL]
    if any(m['render_types']): g['render_types'] = list(m['render_types'])
    if any(m['alpha']): g['alpha'] = list(m['alpha'])
    if any(m['depth_bias']): g['face_flags'] = [(b // 2) & 255 for b in m['depth_bias']]
    if any(t != -1 for t in m['face_textures']): g['face_textures'] = list(m['face_textures'])
    data = native_codec.model(g)
    back = osrscache.decode_model(data)
    for k in ('vertices', 'faces', 'colors', 'priorities', 'render_types', 'alpha', 'depth_bias', 'face_textures'):
        a = [tuple(v) for v in back[k]] if k in ('vertices', 'faces') else list(back[k])
        b = [tuple(v) for v in m[k]] if k in ('vertices', 'faces') else list(m[k])
        if a != b: raise SystemExit(f'round trip changed {k}: not written')
    return data


def main():
    a = sys.argv[1:]
    opt = lambda k, d=None: a[a.index(k) + 1] if k in a else d
    template = int(opt('--template')) if opt('--template') else None
    near = float(opt('--near', 3))
    if a and a[0] == '--check':
        check(int(a[1]), template, near, quiet='--quiet' in a, sleeveless='--sleeveless' in a)
        return
    pos = [v for i, v in enumerate(a) if not v.startswith('--') and (i == 0 or a[i - 1] not in ('--template', '--near'))]
    if len(pos) != 3: raise SystemExit(__doc__)
    import osrscache
    src, slot, dst = pos
    slot = SLOTS[slot] if slot in SLOTS else int(slot)
    if slot not in SLOT_NAME: raise SystemExit(f'no rules for slot {slot}; slots: {sorted(SLOTS)}')
    m = osrscache.decode_model(pathlib.Path(src).read_bytes())
    V = np.array(m['vertices'], float); F = np.array(m['faces'], int)
    sleeveless = '--sleeveless' in a
    if sleeveless and slot != 4: raise SystemExit('--sleeveless applies to the body slot only')
    L, unsure, stats = label_model(slot, V, F, template, near, '--no-balance' not in a, sleeveless)
    FL = None if '--no-face-labels' in a else face_labels(slot, L, F)
    if sleeveless and FL is not None and (set(FL) & ARM_FACE_LABELS):
        raise SystemExit('internal error: a sleeveless piece received an arm face label')
    data = encode(m, L, FL)
    pathlib.Path(dst).parent.mkdir(parents=True, exist_ok=True)
    pathlib.Path(dst).write_bytes(data)
    print(f'{src} -> {dst} ({len(data)} bytes) slot {SLOT_NAME[slot]}' + (f', template item {template}' if template else ', rule only')
          + (', sleeveless' if sleeveless else '') + f': {len(V)} vertices, {len(F)} faces; colours, priorities, render types, alpha, depth offsets unchanged (round trip checked)')
    if m['vertex_skins'] is not None and len(set(m['vertex_skins'])) > 1:
        old = np.array(m['vertex_skins'])
        print(f'  the input already carried labels {dict(sorted(collections.Counter(old.tolist()).items()))}: {int((old != L).sum())} vertices changed')
    report(slot, L, FL if FL is not None else m['face_skins'], unsure, V, stats, '--quiet' in a, sleeveless=sleeveless)


if __name__ == '__main__':
    main()
