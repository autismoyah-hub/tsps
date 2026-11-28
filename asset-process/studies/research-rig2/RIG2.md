# The route for bone-weighted creatures, second study: joint rings, posing that scales, the hard bend, a first original

Private to this machine (cache-derived). Research of 2026-11-26, folder `{ROOT}/studies/research-rig2/`.
It continues `research-rig/RIG.md` (read that first: the route, the three facts, the paint rule); tools were COPIED
from `research-rig/` and changed here; nothing in that folder, in `tools/`, `author-pack/` or the cache was modified.
`P` = `{PYTHON} -I`; `BL` = `"{BLENDER}" -b --python-exit-code 1`.
All commands are run in this folder. Everything stated as measured was run; section 6 lists what was not.

New or changed tools: `ring_census.py`, `ring_summary.py`, `ring_profile.py`, `ring_fit.py`, `ring_fit2.py`,
`ring_table.py`, `ring_webs.py` (the measurements), `ring_eval.py` (the rule against 49 real paintings), `ring_lab.py`
and `ring.sh` (one trial on a real creature, scored and gated), `wlib.py` (`paint_rings`, `joint_kinds`),
`rig_weights.py` (`--rings`, `--webs`; parts file `joints` and `styles`), `rig_pose2.py` (pose helpers), `gif_frames.py`
(a moving picture as a sheet of its frames), `bend_view.py` (a joint close up, measured), `style_rig.py` (the style
check for a weighted creature), `rig_blender.py` (one change: section 4).

## 1. Joint rings

### 1.1 What real creatures paint (49 creatures, one per frame base, 3,072 bone-to-child boundaries)

`P ring_census.py --json data/ring-census.json` (36 s; every weighted creature with under 97% one-bone vertices and
280 or more vertices, the largest model list per base), then `ring_summary.py`, `ring_profile.py`, `ring_fit.py`,
`ring_fit2.py`, `ring_table.py`, `ring_webs.py` on the JSON; logs `logs/ring-*.log`.
Method: vertices welded by position; a vertex belongs to its heaviest bone; a boundary = mesh edges joining the
regions of a bone and its child; ring +k / -k = hop distance from that boundary on the child's / parent's side;
x = distance from the joint plane along the limb in RADII of the limb at the joint. Joint kind from the creature's
own sequences: rigid (the child never turns 8 degrees against its parent, or its region does not touch the
parent's), chain (a run of 4 or more single-child bones), hinge (90% of the turning about one axis), ball (the rest).

| kind | boundaries | child's range of turning, median (10th-90th) | blended at all | 
|---|---|---|---|
| hinge | 695 | 55 (20-116) degrees | 0.43 |
| ball | 659 | 77 (30-160) | 0.51 |
| chain | 1,082 | 77 (27-163) | 0.50 |
| rigid, touching but never turning | 335 | 0 | 0.22 |
| rigid, separate shell (no boundary) | 301 | 45 | (cannot be) |

1. **Whether a joint is blended is the creature's style, not the joint's kind.** By the creature's share of one-bone
   vertices: under 0.2: 0.67 of its joints blended; 0.2 to 0.4: 0.61; 0.4 to 0.6: 0.57; 0.6 to 0.8: 0.32; over 0.8:
   0.25 (closed rings of 6+ vertices only: 0.80 / 0.70 / 0.63 / 0.37 / 0.15). Everything about the joint itself
   moves it less: kind 0.43 to 0.51; range of turning 0.28 (under 20 degrees) to 0.64 (over 90); boundary size 0.31
   (3 to 5 vertices) to 0.68 (30+); how far the boundary goes round the limb 0.61 (closed) to 0.28 (one side only).
   Ba-Ba, Vardorvis' head and the Large Tendril blend nothing; the rays, the gull and the dolphin blend everything.
2. **Where and how much, on joints that are blended** (cross share = the weight of the bone across the joint):

   | ring | hinge: mean / median | ball | chain | commonest bytes |
   |---|---|---|---|---|
   | +-1 (touching the other region) | 0.26 / 0.30 | 0.28 / 0.33 | 0.31 / 0.38 | 127 and 128 (half and half), then 63 (a quarter), 76 |
   | +-2 | 0.10 / 0.00 | 0.11 / 0.00 | 0.11 / 0.02 | 127, 63, then anything |
   | +-3 | 0.07 / 0.00 | 0.04 / 0.00 | 0.04 / 0.00 | 1 to 16 (painting residue) |

   Along the limb the child's share runs 0 to 1 through 0.5 AT the joint (hinge 0.45 at x = 0, ball and chain 0.55):
   the boundary between the two one-bone regions sits on the joint (median x +0.04, +0.01, -0.16). Per joint the
   farthest vertex still carrying a tenth of the other bone sits 0.44 to 0.62 radii from the joint (quartiles 0.2
   to 1.1) = 0.20 to 0.34 of that side's bone; a hinge reaches farther after the joint (0.58 radii, 0.34 of the
   child bone) than before it (0.44, 0.20). Sideways it stops: of vertices within 0.35 radii of the joint plane,
   0.74 to 0.81 are blended up to 1.4 radii from the axis, 0.33 to 0.62 at 1.4 to 2.5, almost none beyond.
3. **By style, over ALL joints** (`ring_table.py`; soft = under 0.4 one-bone vertices, 17 creatures; medium 0.4 to
   0.65, 18; hard, 14): soft creatures: the touching ring within 0.45 radii of the joint plane carries a half
   (median 0.37 to 0.48), out to 1 radius a quarter, the second ring a quarter near the joint; medium: the touching
   ring within 0.45 radii carries a quarter (hinge 0.20 to 0.36, chain 0.32 to 0.40), the rest nothing; hard: nothing.
4. **Hinge, ball and chain are painted alike within those spreads.** The differences the task asked for are small:
   a hinge's blend is a little narrower before the joint and longer after it; a chain has the most vertices lying
   exactly ON the joint plane (1,679 within 0.08 radii, cross median 0.50); a part that never turns is left rigid
   (78% carry no blended vertex); a plate, horn or spike is usually its own shell (301 boundaries) and cannot blend.
5. **WEBS (not in the previous study).** Where the regions of two bones that are NOT parent and child touch (a
   membrane between two wing fingers, skin between a limb and the next trunk bone), `ring_webs.py`, 48,753 pairs:
   a vertex carries the neighbouring unrelated bone at a median 0.15 to 0.25 when that bone owns 45% or more of its
   mesh neighbourhood (soft and medium creatures; 52 to 63% of such vertices over 0.1), 0.00 when it owns under a
   quarter, and 0.00 on hard creatures.

### 1.2 The rule (`wlib.paint_rings`; `rig_weights.py --rings soft|medium|hard [--webs distance|on|off] [--web-min 0.45]`)

Given the armature and ONE bone per vertex (whatever the method or the parts file assigned), for each bone c with
parent p whose regions touch on the welded mesh with 3 or more vertices: axis = the bisector of the limb before and
after the joint; radius = the median distance of the boundary's vertices from it; a vertex within `hops` mesh rings
of the boundary and within 2 radii of the axis is given `amp x (1 - |x| / reach)` of the bone across the joint.

| style | amp | reach, radii | hops | after rounding to quarters |
|---|---|---|---|---|
| soft | 0.5 | 1.1 | 2 | half and half within 0.28 radii of the joint plane, a quarter out to 0.83 |
| medium | 0.36 | 0.7 | 1 | a quarter on the touching ring within 0.45 radii |
| hard | 0 | | | one bone a vertex |

Reach is scaled by kind (hinge: 0.85 before the joint, 1.3 after; rigid: none) and never passes 0.4 of the bone on
that side. Kinds: declared in the parts file (`"joints": {"hinge": [..], "rigid": [..]}`), else chain from the tree,
else ball; a part marked `rigid` gets no ring. `"styles": {"hard": [BONE..]}` sets one joint's style. Webs: a vertex
whose neighbourhood is 45% or more an unrelated bone's region shares with it by the two bones' distances, never
over a half. A vertex keeps its own bone as the heaviest; then the tool's ordinary cleanup (`--max`, quarters, bytes
summing to 255).

### 1.3 How well a rule can match a painting (`P ring_eval.py --json data/ring-eval.json`; `logs/ring-eval.log`)

Each of the 49 creatures: real weights reduced to the heaviest bone, rings painted by rule, posed through up to four
of its sequences; mean vertex displacement from the real weights' pose as a percentage of the creature's size.

| creatures | one bone a vertex | rule, soft | rule, medium | the REAL two heaviest | rule beats rigid on |
|---|---|---|---|---|---|
| soft (17) | 0.467 | **0.387** | 0.412 | 0.158 | 17 of 17 |
| medium (18) | 0.312 | 0.313 | **0.299** | 0.100 | 12 (soft), 14 (medium) |
| hard (14) | **0.074** | 0.130 | 0.093 | 0.015 | 5 |

So: the rule moves a soft creature a sixth of the way from rigid toward its real painting, does nothing for a medium
one on average, and harms a hard one. A rule cannot know which joints an artist blended; it paints the family's
habit. Joint kinds from the creature's motion or from the tree alone give the same figures on soft creatures (0.387 against 0.386).

### 1.4 The gate (`sh ring.sh NAME TRIAL real|rule options`; `logs/ring-gate-real.log`, `logs/ring-gate-rule.log`)

Bosses: the Maggot King 15742 (as before), and two the tool had not seen: the **Doom of Mokhaiotl 14707** (the owner
named it; five model files, so it also tests several models; 46% one-bone vertices; no walk: sequence 12408 fills the
slot) and **Yama 14176** (a humanoid with elbows and knees: the hinge case; three model files). Both import and
export through the route; a model file with no weights at all (a ground decal) is passed through unweighted.
"Lost" = the gate's "overlap lost against standing", stand / walk / attack / death; PASS is 0.05.

| weights | Maggot King: mean error, lost, verdict | Doom | Yama |
|---|---|---|---|
| real, two heaviest | 0.21; 0.005 0.004 0.004 0.004; PASS | 0.29; 0.002 0.002 0.002 0.003; PASS | 0.90; 0.007 0.007 0.010 0.005; PASS |
| real, one heaviest (rigid) | 2.64; 0.056 0.060 0.057 0.057; BORDERLINE | 1.26; 0.005 0.006 0.008 0.008; **PASS** | 2.96; 0.016 0.018 0.025 0.018; **PASS** |
| real heaviest bone + rings and webs by rule, medium | 2.46; **0.048 0.052 0.050 0.049**; BORDERLINE (three of the four lines PASS, walk 0.052) | 1.34; 0.005 0.006 0.008 0.008; PASS | 2.38; 0.012 0.014 0.018 0.010; PASS |
| the same, soft | 2.86; 0.047 0.052 0.049 0.047; BORDERLINE | 1.36; 0.005 0.006 0.009 0.008; PASS | 2.30; 0.012 0.013 0.018 0.011; PASS |
| REAL WEIGHTS THROWN AWAY: nearest bone, one a vertex | 7.29; 0.068 0.075 0.074 0.071; BORDERLINE | 13.84; 0.217 0.176 0.239 0.355; FAIL | 27.35; 0.095 0.110 0.126 0.166; FAIL |
| thrown away: nearest (shift 0) + rings soft | 6.83; 0.057 0.067 0.064 0.063; BORDERLINE (bone-centroid line now PASS) | 12.12; 0.204 0.157 0.217 0.345; FAIL | 26.83; 0.082 0.098 0.128 0.176; FAIL |

(The Maggot King rule rows use the one cut rule of `parts/maggot-king-cut.parts.json`; Doom and Yama have NO parts
file: nothing about them was read.)

What this says, exactly:
1. **The previous study's diagnosis was wrong.** On the Maggot King the joint rings are not what separates
   BORDERLINE from PASS. `P scratch/diag2.py work/maggot-king`: with every real parent-child blend restored (564
   vertices) the lost overlap is 0.049 0.051 0.047 0.045: still at the line. With ONLY the 202 vertices whose two
   bones are NOT parent and child restored it is 0.014 0.015 0.016 0.017: PASS. Those vertices are the wing
   membranes between the fingers (`scratch/diag3.py`: bones 47+49, 79+81, 49+53: weights 192+63 and 128+127) and
   the shoulders; 154 of the 202 touch the second bone's region. Picture:
   `render/research-rig2-rings/maggot-king-r-rigid-errors.png` (red = off by 3% of the size: the membranes).
   That is why section 1.1 point 5 and the web part of the rule exist.
2. **Rings and webs by rule on a correct one-bone assignment**: Maggot King from 0.056-0.060 to 0.048-0.052: three
   of four motion lines PASS, the walk misses by 0.002. Doom and Yama PASS, but they pass rigid too: on them the
   rule only improves the numbers (Yama's attack 0.025 to 0.018).
3. **With the real weights thrown away the verdict is decided by WHICH bone owns a vertex, not by rings**: Maggot
   King BORDERLINE (best so far: 0.057 to 0.067, was 0.065 to 0.071), Doom and Yama FAIL without a parts file, as
   Zebak did before its parts file in the previous study. Rings recover 0.01 of overlap. PASS by rule alone is NOT
   reached on any of the three.
4. **The gate's line.** "Overlap lost against standing at most 0.05" IS met by real weights simplified sensibly:
   the two heaviest influences pass on all three (0.002 to 0.010), and even one bone a vertex passes on Doom and
   Yama. No recalibration is proposed. What the Maggot King shows is that on a creature with membranes the line
   reacts to about 200 web vertices: a rebuild of such a creature must paint its webs.

## 2. Posing that scales (`rig_pose2.py`; worked examples `anim/idle_r1..r7.py`, `anim/attack_r1..r5.py`)

```
import rig_pose2 as P2
rig = P2.load('creature.blend'); a = P2.Anim2(rig, 'attack', 120, stance=('seq_13926', 0), step=3); G = a.G
G.describe()                                  # every chain: bones, length, side, mirror partner (blender ... -P anim/describe.py)
MAG = G.chain('b124'); WING = G.chain('b076'); RIBS = [G.chain(b) for b in ('b071', 'b067', 'b063')]; LEG = G.chain('b003')
a.define('coil', a.shift(0, 9, -10) + P2.curl(MAG, 'X', -62, bias=1.3)               # named pose of the WHOLE body
         + a.both([(WING[0], 'X', 56)] + P2.spread(RIBS, 'X', 40) + [x for r in RIBS for x in P2.curl(r, 'X', 30, bias=1.6)]))
a.pose(33, a.mix(('coil', 1.1))); a.pose(45, a.mix(('lash', 0.95))); a.pose(50, a.mix(('lash', 1.08), ('tip', 1.0)))
a.follow(MAG[4:], each=1, start=1)            # follow-through: each bone a cycle later down the chain
a.hold(['b000', 'b009'], chain=3)             # both feet stay where the stance has them while the root shifts
a.layer(P2.windowed(P2.breath(SPINE, 'X', 2.5, 20, each=0.1), 120, 9))     # under every pose; faded so it joins the stance
a.build()
```
| helper | what it does |
|---|---|
| `Groups`: `line`, `chain`, `fan`, `limb`, `mirror`, `describe` | groups read from the armature (no names needed): a bone's main line to the tip, its single-child run, the lines under a hub ordered by angle, a subtree, the partner across the middle |
| `curl(line, axis, total, bias)` | a turn shared along a chain; bias over 1 puts it at the tip |
| `spread(lines, axis, total)` | fan the roots of several lines apart |
| `wave`, `sway`, `breath`, `bob` | a travelling S; and LAYERS: functions of the cycle applied on top of whatever the poses give |
| `a.mirror`, `a.both` | the same turns on the other side (X turns keep their sign, Y and Z turns and x moves flip) |
| `a.define`, `a.mix((name, w), ..)`, `scale` | named poses, weighted and added, keyed like any pose |
| `a.follow(line, each, start)` | overlap by per-bone delay |
| `a.hold(bones, chain)` | after every pose and every baked frame the `chain` bones above each listed joint are turned (cyclic coordinate descent) until the joint is back where the stance has it: planted feet |
| `windowed(fn, length, ramp)` | a layer faded in and out, so an action starts and ends exactly on its stance pose |

With a layer or a hold the action is baked in Blender (a key every `step` cycles), which is what `rig_export` writes.

**Proof on the Maggot King** (`sh anim/round2.sh NAME ROUND "real:SEQ:label"`; outputs `anim/out/`, pictures
`render/research-rig2-anim/`; `rig_timing.py` on every round; notes in `anim/NOTES2.md`):

| | bones turning over 5 degrees | mean speed, units/cycle | peak / mean | farthest vertex | near-still cycles |
|---|---|---|---|---|---|
| previous idle (research-rig r4) | 9 | | | | |
| **idle r6** (loops on itself) | **59** | 0.60 | 1.3 | 84 | 0 |
| **idle r7** (starts and ends exactly on the stand's pose) | 55 | 0.62 | 1.9 | 74 | 0 |
| real stand 13926 | 61 | 0.44 | 1.4 | 73 | 0 |
| previous attack (research-rig r4) | 22 | | 5.1 | 407 | 6 |
| **attack r5** | **140** | 7.41 | 3.1 | 424 | 3 |
| real 13925 | 138 | 9.79 | 2.7 | 750 | 12 |
| the creature's other real sequences (13928, 13929, 13935) | 128 to 134 | 3.7 to 10.5 | 1.3 to 1.5 | 267 to 329 | 0 |

Speed by tenth of the length, attack r5 against the real 13925: 7.3 6.8 5.0 14.5 5.0 9.3 6.3 6.8 7.0 6.2 against
8.1 9.0 5.5 21.4 14.2 5.2 4.5 7.0 13.0 10.1. Bone counts are inside the creature's real range (55 to 59 against 61
for the stand is just under; 140 against 128 to 138). The idle is 36 to 40% faster than the real stand; the
attack's peak / mean (3.1) is still above the real set's (1.3 to 2.7) and its median vertex travels 89 units where
the real one's travels 279.

## 3. The hard bend (`sh bend/step3.sh`; `P bend_view.py ...`; `render/research-rig2-bend/bend-final.png`, `bend-client.png/.gif`)

A new 7-bone skeleton (`bend/build_bend.py`): a thick arm and a thick leg, radius 21 at the joint on bones 70 long
(length / radius 3.3), 10-sided, rings every 10 units and three 5 apart at the joint; weights by `--method nearest
--shift 0 --rings STYLE` with `bend/bend.parts.json` (two parts, the elbow and knee declared hinges); the elbow closed
to 140 degrees, the knee to 130. `bend_view.py` holds the parent bone still and draws the exact posed vertices from
the side and from inside the bend, and measures. Beside it the real Sol Heredit's arm (bone 44: radius 12, bones 50
long, painted 0.42 / 0.31 on the rings at the joint, turning 123 degrees in its own sequences).

| at 120 degrees (110 in brackets) | the ring ON the joint keeps of its radius | faces turned over | faces under a fifth of their area |
|---|---|---|---|
| ours, hard (one bone a vertex) | 1.00 | 9 of 350 | 0 |
| ours, medium | 0.69 (0.74) | 11 of 370 | 0 |
| ours, soft | 0.56 (0.63) | 5 of 370 | 0 |
| ours, knee, soft | 0.55 (0.62) | 13 of 370 | 0 |
| real Sol Heredit arm | 0.94 (0.94) | 10 of 297 (12) | 10 (4) |

Read from the picture (I looked at `bend-elbow.png`, `bend-final.png` and `bend-client.png`): **nothing opens** (one
welded skin cannot) and nothing tears. HARD: from 90 degrees the forearm slices through the upper arm and the outer
elbow is a sharp corner. MEDIUM and SOFT: the outer elbow is a round arc at every angle; the inner side folds into a
crease in which a few faces turn over, as the real arm's does (10 to 12 faces at 110 to 120 degrees). **It does
collapse**: a half-and-half ring loses radius as cos(angle / 2), and the soft ring on the joint is down to 0.56 of
its radius at 120 degrees (medium 0.69), where the real painted arm keeps 0.94: the real elbow's own joint ring is
not half and half in the plane of the bend. By the client's own drawing at a creature's size the bend reads as a
clean bent limb. Rule for an author: on a thick hinge that closes past 90 degrees use `medium` for that joint
(`"styles": {"medium": [BONE]}`), look at it with `bend_view.py`, and expect to repaint the ring on the joint.

## 4. The first original weighted creature: the Zodiac ray (`sh ray/round.sh N`; `ray/notes.md`)

`ray/build_ray.py` (armature and mesh; no game data read), `ray/ray.parts.json`, `ray/anim_ray.py` (five sequences
by the helpers), `ray/pack.json`. Final files: `ray/work/r8/export/` (`part-0.bin` 16,171 bytes, `base_12720.dat`
3,046 bytes, `anim_stand|move|attack|flinch|death.dat`, `zodiac-ray.json`), scene `ray/work/r8/ray.blend`, every
round's sources `ray/rounds/rN/`. Packed: `ray/work/cache-packed` (NPC 16390, model 62610, frame base 12720, keyframe
groups 3110 to 3114, sequences 21210 to 21214); by id against loose files: 840 cells, 75,600,000 pixels, 2,800,110
figure pixels, **0 differing** (`logs/ray-byid.log`). Pictures `render/research-rig2-ray/rN/`, `byid/`.
39 bones (root, hub, head, snout, two back bones, ten tail bones, fin, three ribs of three bones a wing, two lobes of
two), 739 vertices, 1,474 faces, 41 colours, one welded skin, 60 inlaid details by the style tool (51 insets were made: 18 stars, 2 eyes, 6 gill slits, a mouth, 12 marks on the back, 12 under the wings); weights: 317 vertices one bone, 371
two, 51 three; 36 joints ringed, 71 web vertices. `style_rig.py`: 0 GAME limits out, 0 RECENT bands out.
Found on the way and fixed in `rig_blender.export_base`: a NEW bone lying along the armature's front-back axis got a
game rest rotation of exactly 90 degrees about x, the pole of the client's angle formula (residual 0.005 to 0.007
on the exported channels); new bones are now written as pure translations, as 4,602 real bones are (residual under
0.000001).

## 5. Rules added for an author (to RIG.md section 7)

- Assign ONE bone per vertex first (parts file), then `--rings`: soft for a creature that should bend like a ray, a
  tail or a wing; medium for a jointed animal; hard (or a `rigid` part) for plates, jaws, anything that must not
  shear. Declare elbows and knees as hinges. Never leave a membrane between two chains on one of them: check the
  tool's [quoted words withheld] count is not 0 on a wing.
- Pose WHOLE-BODY named poses and mix them; put a layer under everything; window the layers of any action that must
  join the stand; measure every round with `rig_timing.py` beside a real sequence of the same kind (bones turning
  over 5 degrees, speed by tenth, peak / mean, near-still cycles).
- Read the moving picture as a sheet of its frames (`gif_frames.py`), not only the contact sheet.
- A new bone's game rest matrix is a translation (the exporter does it); do not turn bones under scale helpers.

## 6. Not proven, not run

- No weights assigned wholly by rule reach PASS on a real boss (Maggot King BORDERLINE; Doom and Yama FAIL with no
  parts file). No parts files were written for Doom and Yama: [quoted words withheld] on an unseen boss is untested.
- The rows [quoted words withheld] read the real weights for the assignment: they isolate the ring rule, they
  are not a method.
- Style thresholds (0.4, 0.65) and the rule's numbers were set from the same 49 creatures the rule is scored on in
  1.3 (no held-out creatures there); the Maggot King, Doom and Yama are among the 49.
- The web rule's distance form was chosen on the Maggot King's score (2.46 against 2.54 for a flat quarter).
- The bend's collapse was measured, not cured; the hypothesis about how the real elbow keeps its radius was not tested.
- `bend_view.py` is a diagram (painter's sort of exact vertices), not the client's rasteriser.
- The Zodiac ray's flinch has no measured real flinch behind its timing. Its weights were never scored against
  anything (there is no answer to score against); nobody but me has looked at it; it was not played in a session.
- `rig_light.py` is still the float port; constraints and IK in Blender still untried (the hold is my own CCD).
- Idle and attack on the Maggot King: I read every frame of the moving pictures of idle r1 to r3 and r6 and attack
  r5, and every second frame of idle r4, r5 and attack r1 to r4; idle r7's pictures were made and not looked at.
