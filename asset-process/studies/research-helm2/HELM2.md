# HELM2: what the passed helm and the real helms share that Umbral A lacks

Private to this machine (cache-derived). Folder `{ROOT}/studies/research-helm2/`.
`P` = `{PYTHON} -I`, run in this folder. Every number here was run; what I am
unsure of is in section 8.

The owner's verdict (the fashionscape test on Umbral A): preferred the earlier passed helm; the new one was judged below
the level of the real helms. Umbral A passed `fit.py`, the worn gate and the brief's
tables.

```
P measures.py [KEY..]   P extra.py [KEY..]     every measure -> data/measures.json (about 710 numbers per helm)
P separate.py [--all]                          which measures separate, ranked    P tab.py KEY,KEY [--md]   numbers per helm
P helmcheck.py KEY | HELM.bin SLOT --rest .. --body ..  | --all      the guard (section 6)
P boards.py     P board_a2.py                  the boards in boards/
umbral-helm-a2/cycle.py TAG                    the test helm: build, export, label, fit, helmcheck, look
```

## 1. Groups and method

| group | helms |
|---|---|
| ACCEPTED, core six (the band) | Torva full helm 26382, Oathplate helm 30750, Justiciar faceguard 22326, Neitiznot faceguard 24271, Serpentine helm 12931 (named by the owner); `meridian-helm-v5-astra` (passed by the owner) |
| ACCEPTED+, five more (NOT judged by the owner; names checked in the cache index) | Inquisitor's great helm 24419, Masori mask (f) 27235, Dharok's helm 4716, Statius's full helm 22625, Armadyl helmet 11826, each in its own set |
| REJECTED-RECENT | `umbral-helm-a` (judged: No), `umbral-helm-b` (the direction not chosen) |
| REJECTED-EARLIER | `meridian-helm-v4-a/-b/-c` (fitted, [quoted words withheld]), `v3-a` and `v2-a` (by eye) |

Each helm is measured IN ITS OUTFIT as the owner saw it (v5 on the v5 set, Umbral A on its `*-watch` stand-in, B on
its own recolour, v4 and v2 on the earlier set, v3 on the v3 set, real helms on their sets or Bandos). Pixel measures
use the client's own picture (`tools/pose.py`, bind pose) at game size (zoom 300) and close (zoom 130), from the
front, front quarter, unlit side, back and lit side; the helm's pixels are those that differ with and without the
helm. Mesh measures use the worn mesh. About 713 numeric measures per helm were tested.

A measure is called
- **CLEAN** if all seven rejected helms are outside the core six's band on the same side,
- **RECENT** if both Umbral helms are outside (the earlier five not all),
- **A-ONLY** if Umbral A is outside and B inside (B was not the helm the owner judged in the test, but it was the
  direction the owner liked less, so an A-ONLY line cannot be the whole account).

Count over the 713: 4 CLEAN (all four are one family: fold share), 16 RECENT, 81 A-ONLY, the rest none.

## 2. What separates, ranked by how cleanly

Numbers per helm (`!` = outside the core six's band). This is `P helmcheck.py --all`.

```
helm        group  fold45  fold45  range   lights  body   side   quarter side/back side   front  least  height rise   darkest
                   share   dome    vs body -body   range  void   void    under 20  bands  level  band   /width /width paint
band               >=0.254 >=0.241 >=0.90  >=-16.3 <=122  <=0.189 <=0.114 <=0.341  <=1.44 <=0.95 <=1.11 <=1.83 <=0.53 <=2
torva       ACC    0.254   0.315   1.08    -7.7    107.3  0.05   0.056   0.191     1.44   0.71   1.11   1.4    0.31   0
oathplate   ACC    0.279   0.278   1.15    9.7     62.7   0.189  0.114   0.341     1.25   0.81   0.64   1.59   0.53   0
justiciar   ACC    0.302   0.329   0.9     -16.3   122.0  0.079  0.033   0.0       0.87   0.95   0.87   1.55   0.19   0
faceguard   ACC    0.277   0.279   2.04    54.0    76.7   0.017  0.026   0.0       1.07   0.55   0.4    1.56   0.43   0
serpentine  ACC    0.447   0.488   1.24    -7.0    76.7   0.073  0.035   0.26      1.38   0.84   0.92   1.83   0.37   0
v5 (passed) ACC    0.271   0.241   1.25    -10.7   98.7   0.093  0.039   0.178     0.88   0.89   0.88   1.6    0.25   2
inquisitor  ACC+   0.247!  0.285   1.35    11.3    59.0   0.264! 0.144!  0.071     2.0!   1.23!  1.4!   1.58   0.0    0
masori      ACC+   0.224!  0.17!   1.24    10.0    92.7   0.179  0.048   0.39!     1.86!  0.74   0.5    1.6    0.35   0
dharok      ACC+   0.383   0.206!  0.76!   -34.4!  116.3  0.0    0.0     0.197     1.0    2.0!   0.0    1.91!  0.52   0
statius     ACC+   0.217!  0.146!  0.89!   -7.6    113.7  0.0    0.0     0.028     2.5!   0.71   1.0    1.88!  0.3    0
armadyl     ACC+   0.335   0.299   1.28    -10.3   107.6  0.001  0.006   0.0       1.08   0.79   0.0    1.76   0.18   0
umbral A    RR     0.188!  0.131!  0.77!   -41.3!  137.7! 0.397! 0.189!  0.465!    4.33!  1.4!   2.19!  2.21!  0.72!  4!
umbral B    RR     0.2!    0.103!  0.77!   -24.4!  149.7! 0.2!   0.057   0.289     0.9    0.95   0.9    1.6    0.38   0
v4-a        RE     0.169!  0.192!  1.6     14.7    79.3   0.099  0.039   0.195     1.17   1.18!  0.9    1.53   0.21   8!
v4-b        RE     0.082!  0.096!  1.48    1.7     79.3   0.07   0.042   0.035     3.4!   1.66!  1.58!  1.6    0.25   0
v4-c        RE     0.189!  0.193!  1.53    8.7     79.3   0.028  0.032   0.031     1.38   1.66!  1.17!  1.69   0.33   8!
v3-a        RE     0.213!  0.219!  1.71    13.0    70.3   0.105  0.041   0.0       2.25!  1.7!   1.15!  1.65   0.5    8!
v2-a        RE     0.028!  0.036!  1.59    14.0    79.3   0.032  0.044   0.039     0.56   0.82   0.56   1.38   0.25   10!
A2 on A's body     0.257   0.245   0.81!   -39.3!  137.7! 0.072  0.04    0.276     1.0    0.91   0.86   1.78   0.41   0
A2 matched body    0.257   0.245   1.09    -4.0    102.3  0.072  0.04    0.276     1.0    0.91   0.86   1.78   0.41   0
A on matched body  0.188!  0.131!  1.04    -6.0    102.3  0.397! 0.189!  0.465!    4.33!  1.4!   2.19!  2.21!  0.72!  4!
```

### 2.1 CLEAN: hard folds (the only family that separates all seven rejected helms)

**Share of the shell's edge length folded more than 45 degrees** (`fold45_share`; welded two-face edges of the helm's
own shell, weighted by length). Core six 0.254 to 0.447. All seven rejected 0.028 to 0.213. It survives widening to
all eleven real helms and v5, but only just: the lowest real helm is Statius's at 0.217, the highest rejected is
v3-a at 0.213. The same on the **skull alone** (not the face, not the signature; `fold45_dome`): core 0.241 to 0.488;
rejected 0.036 to 0.219; not clean against the wider group (Statius 0.146, Masori 0.170, Dharok 0.206).
Threshold variants: over 35 degrees separates both Umbral helms and four of the earlier five; over 55 degrees on the
skull is clean against the core.

What it is in the mesh (board 1): Umbral A is a sixteen-sided lathe of eight rings. 2,183 of its 3,004 units of edge
are folded under 30 degrees; its hard edges are the sight, the crescent and the hem. v5 has the same total edge length
(3,001) and 817 units over 45 degrees against A's 566: the 250 extra are the cut sight, the vents, the flute, the nape
lap and the blade roots. Torva has 1,669 units in all (214 faces) and 423 hard: few faces, most of them meeting at a
crease. v5 was made by closing exactly this gap from v4-a (0.169 to 0.271) and passed.

This number is ALREADY asked for by the brief (step 4: [quoted words withheld]) and Umbral A's
own REPORT prints it: 18.8% against 27.5 to 43.2 for the five. It was computed after the last round ("calculated
later on the archived exports") and never closed. Umbral B's report does not print it at all. The brief asked for
the measure and gave it no number and no gate.

Chance: with 13 helms in two groups, about 0.8 of 713 unrelated measures would separate cleanly by accident. One
family did. What argues it is not accident: it was named in advance by the brief, it is what "sharpness" in the
owner's own v4 verdict was turned into, and the one helm of ours that passed is the one that was moved across it.

### 2.2 RECENT: the stand-in body is louder than the helm (both Umbral helms; none of the earlier five)

At game size from the front, the body just under the helm (40 rows of the figure, skin left out):
- tone range of the body (95th minus 5th percentile of grey): core 62.7 to 122.0; Umbral A's stand-in 137.7, B's
  149.7. This survives the wider band (eleven real sets: 59 to 122).
- the helm's lights minus the body's lights: core -16.3 to +54; A -41.3, B -24.4 (Dharok -34.4 in the wider band).
- the helm's tone range / the body's: core 0.90 to 2.04; A 0.77, B 0.77 (Dharok 0.76, Statius 0.89).

Cause: the recoloured Meridian pieces carry silver at lightness 76 and 91 on steel at 8 to 22. No accepted body has a
second paint that far from its steel (painted lightness at the 95th percentile of the body: accepted 39 to 74 core,
20 to 88 wide; A's stand-in 91). On every accepted outfit the helm is at least as contrasty as the body under it; on
board-67 the Umbral body takes the eye first and the helm is the dull part. It is NOT busyness: the v5 set has more
tone steps (20.2 per 100 px) than the Umbral stand-in (15.5) and passed.

The earlier five rejected helms are on the far side of this line (their helms are louder than their bodies), so it
is not why a helm is rejected in general; it is something new on board-67 that the passed outfit did not have.
Putting A on a body recoloured to match (silver 27 lightness steps lower) brings all three numbers inside
(1.04, -6.0, 102.3) and leaves A outside on every other line.

### 2.3 A-ONLY (Umbral A outside the core band, B inside), in order of margin

- **Level stripes.** A is painted ring by ring (`build_blender.py`: `steel([28,20,39,28,39,28,39][j])`). In the side
  view at game size it shows 4.33 level tone bands for every upright one (core 0.87 to 1.44; wider up to 2.5); least
  striped view 2.19 (core at most 1.11); level steps over upright steps in front 1.40 (core 0.55 to 0.95; wider up
  to 2.0). Three of the earlier five are also outside on these (v4-b, v4-c, v3-a), which makes it more than an
  accident of A, but Dharok, Statius and Inquisitor break the lines in the wider group.
- **A dead dark side.** Largest connected near-black patch (grey under 16) in the unlit side view: 0.397 of the helm
  (core 0.017 to 0.189; wider to 0.264; B 0.200 is a hair outside the core). Front quarter 0.189 (core at most 0.114,
  wider 0.144). Cause in the paint: 0.465 of A's side- and back-facing steel is painted under lightness 20 (core at
  most 0.341, wider 0.39). A's author painted [quoted words withheld] 16 and [quoted words withheld] 22: the shade was
  baked into the paint and the game's light then darkened it again. Torva's steel is the same family (0/0, 12 to 57,
  median 26 against A's 12 to 76, median 22) but its sides are painted like its front.
- **Proportion.** Whole height over the shell's width at the skull 2.21 (core 1.40 to 1.83, wider to 1.91); rise
  above the crown over that width 0.72 (core and wider at most 0.53). A's shell is the narrowest allowed (22.2) and
  its crest the tallest allowed (16): each inside `fit.py`'s range, the pair outside every accepted helm.
- **Darkest paint.** A's darkest face is lightness 4; all eleven real helms reach 0 and v5 reaches 2 (v4-a, v4-c,
  v3-a 8, v2-a 10: four of the five earlier rejected helms are outside too).

## 3. What does NOT separate (tested, numbers in `data/tab-nonsep.md`, `data/tab-nonsep2.md`, `P tab.py`)

| candidate | result |
|---|---|
| the dark steel as paint (hue, saturation, lightness) | A 0/0, 12 / 22 / 76 (5th / median / 95th); Torva 0/0, 12 / 26 / 57; Oathplate 12 / 24 / 39. Inside. A is a real dark armour's steel |
| the helm against the ground at game size | share of the outline within 12 grey of the ground: A 0.26, B 0.30; Torva 0.30, Oathplate 0.28, Serpentine 0.29. Inside |
| drawn lightness of the helm in front | median grey A 49, Oathplate 43, Serpentine 48. Inside (the unlit side, 20.7, is just under the core's 24.7; Dharok 19.3) |
| definition of the helm (steps, lines, regions per 100 px) | A 13.1 steps / 6.2 lines over five views; core 6.8 to 20.0 / 2.9 to 12.5. Inside |
| the signature's area, front and side | A 93 / 171 square units; core 62 to 316 / 63 to 310. Inside |
| the signature's thickness at game size | mean width A 4.7 / 4.4 px; core 1.8 to 5.5 / 3.0 to 7.7 (Torva 3.3 / 3.0). The crescent is NOT thinner than Torva's spike and fins. Inside |
| signature pixels standing 24 grey off the ground, least of front and the two sides | A 163, B 129; core 18 (Torva) to 273. Inside |
| the face: black void's share, contrast round the largest void, eye relief | A 0.033 of the front, 17.7 grey round it, relief 3.6; core 0.011 to 0.087, 13.3 to 106, 0.8 to 10.2. Inside (Serpentine's void contrast is lower than A's) |
| plates: number of planes, share in small planes, faces per plane | A 131 planes, 0.37 small; core 92 to 181, 0.27 to 0.60. Inside |
| trim: second material's share and its lightness against the steel | A 0.004 (silver counted as steel family) / B 0.05; Torva 0.003. Inside |
| helm against shoulders (`helm_to_shoulder`, helm share of the figure) | A 0.29 / 0.103; core 0.29 to 0.41 / 0.085 to 0.112. Inside (at the edge) |
| face count and where spent | A 376, face 0.32 / back 0.23; core 214 to 377, 0.12 to 0.50 / 0.09 to 0.37. Inside |
| the brief's own table (upper fold, aft tip, edge thickness, painted hard folds, shallow joins, eye relief, step, front lightness range, paints) | A inside on every column except darkest paint (2.3). This is why the brief passed it |
| the creature study's `cleancheck.py` (splotch, committed folds, firm folds, one skin, lumps) | does not separate. Splotch 0.000 to 0.006 on every helm, ours and real (creature band up to 0.158); hard/firm folds A 1.78, B 1.60, core 0.94 to 3.41 (Torva 1.04, faceguard 0.94); no closed lumps on any helm; largest welded piece A 0.87, core 0.87 to 1.0 (B 0.62 and v3-a 0.62 are outside: their horns and face are separate pieces). Helms are not splotchy or lumpy; the creature lines are not the helm's fault |

## 4. Looking (my view, labelled as such)

Before measuring I wrote down what I saw on board-67 and in `work/look-head.png`: A reads as a striped beehive with
a thin horn and no decided face; its side is a black shape. Of these, the stripes and the black side were confirmed
by a measure, [quoted words withheld] was NOT (it is as thick as Torva's fins; what I saw was its dark spine vanishing, and that
did not separate either), and [quoted words withheld] was not confirmed by any face measure I could build. So half of what
I saw held. The fold share is not something I saw at all; it came from the numbers.

## 5. The test: `umbral-helm-a2/`

A's idea kept: a closed dark watch helm, steel 0/0 with silver 37/1 and one petrol accent, one oblique crescent.
A's eight fitted profiles are reused unchanged in plan, so the shell sits as A's does. Changed (`a2_geom.py`):

| gap | A | what A2 does | A2 |
|---|---|---|---|
| hard folds, whole / skull | 0.188 / 0.131 | four plates that lap with a real step (gorget under mask and band; bowl over the band; mask ending in an upright step at the temple), a keel ridge brow to nape, a brow shelf over a sight 3 deep, one V cut per cheek plane, the top ring removed | 0.257 / 0.245 |
| level stripes, side / front / least | 4.33 / 1.40 / 2.19 | paint by plate, not by ring; upright lines from the keel strip, the cheek cuts and the mask's edge | 1.0 / 0.91 / 0.86 |
| dark side: black patch / steel under 20 | 0.397 / 0.465 | side and back steel 22 to 28, front 33 to 57 | 0.072 / 0.276 |
| height / skull width; rise / width | 2.21 / 0.72 | crescent lowered from 16 to 10 above the crown and made deeper; bowl 0.7 further out | 1.78 / 0.41 |
| darkest paint | 4 | sight floor 0 | 0 |
| body louder than helm (range ratio / lights / body range) | 0.77 / -41.3 / 137.7 | not a helm change. On A's body: 0.81 / -39.3 / 137.7. On `*-match.bin` (silver 27 steps lower): | 1.09 / -4.0 / 102.3 |

396 faces (A 376). `fit.py`: PASS, 0 of the starred lines fail, 0 size lines outside (`umbral-helm-a2/fit.txt`).
Worn gate: PASS (`gate.txt`). Made by script in four rounds (render/umbral-helm-a2-r1 to -r4), built and exported
through Blender as A was.

Board: `boards/board-A2-test.png` (the left half of board-67 again: A2 on the matched body, A2 on A's body, A,
Meridian wardens, Torva, Oathplate, Justiciar).

**My view (an opinion, and my opinions of this owner's taste are right about half the time):** A2 looks nearer the
real helms than A in the front and front-quarter views: it has a face that reads at game size (a lit keel strip
between two dark eyes, barred cheeks) and the plates can be counted. From the side it is no longer a black shape.
It is still plainer than Torva and v5 from behind, the two petrol wedges on the brow are larger than I would like,
and the face is more of a visor grille than A's single idea. On the matched body the figure is calmer and the helm
is the brightest thing on it; on A's body the silver bands still pull the eye down. I would not predict a pass:
A2 was built in four scripted rounds to move numbers, with no round of looking beside the references by an author.

## 6. `helmcheck.py`

Prints the fourteen lines of section 2 for a helm in an outfit with the core band and the wider band. Exit code 1
only if a CLEAN line is outside. A guard, not a judge: four of the five extra real helms fail a CLEAN line (Inquisitor,
Masori and Statius the whole-shell line by 0.007 to 0.037; Masori, Statius and Dharok the skull line), and a helm inside every band has not thereby passed.

## 7. Boards (`boards/`)

- `board-1-sharp-folds.png`: A, v5, Torva, Oathplate, A2: every edge drawn, hard folds orange.
- `board-2-stripes-dark-side.png`: the same five, client pictures beside what the stripe and black-patch measures see.
- `board-3-proportion-outfit.png`: A, A on the matched body, A2 on both bodies, v5, Torva, Oathplate at game size
  with the height / skull-width box and the helm and body tone ranges.
- `board-A2-test.png`: the test in board-67's layout.

## 8. Unsure, and what failed

- **The accepted group holds ONE helm of ours.** Every band rests on five real helms and v5. The fold line is the
  only one that also has the v4-a to v5 history behind it. A second passed helm could move any band.
- The wider real group breaks most lines: Inquisitor, Masori and Statius are under 0.254 on folds; Dharok's helm is
  duller than its body; Statius and Inquisitor are striped by my measure. Either those helms would not pass this
  owner beside Torva, or the bands are too narrow. I cannot tell which; the owner has not judged them.
- The Umbral B helm was never put to the fashionscape test; "rejected" for B means [quoted words withheld].
- Body and helm were judged together. Whether the owner's [quoted words withheld] was moved by the stand-in body
  cannot be read from one verdict; A on the matched body is on board 3 for that question.
- The pose background (grey 43) stands in for the game's ground; [quoted words withheld] was measured against
  that one colour only.
- Bind pose only; no motion measures were tried here.
- Dharok's recolour pairs apply incompletely in the mesh loader (its helm reads one lightness); its pixel numbers
  are right, its paint columns are not.
- Did not work: every measure of the FACE (void share, contrast round the void, void count, eye relief) and of the
  signature's thickness and readability. I believed both differed and could not make a number say so. The creature
  study's seven lines do not carry over to helms.
- A2's fold shares are inside by 0.003 and 0.004: closed to the edge of the band, not to its middle.
