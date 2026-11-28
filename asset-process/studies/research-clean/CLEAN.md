# Clean, defined, built-in: what separates real creatures from the stone watcher

Private to this machine (cache-derived). Folder `{ROOT}/studies/research-clean/`.
`P` = `{PYTHON} -I`, run in this folder. Everything stated as a number was run;
what was not, and what I am unsure of, is in section 9.

The owner's words (the repainted watcher beside the Minotaur and Tekton, `board-65-watcher-repaint.png`):
"Both references are very clean, well defined and smooth. Our minitour facial features, horns, and other body
features are less defined. The eyes and nostrils make me think of playdough and playdough features stuck on a clay
bound. The texturing I would describe as splotchy rather than smooth (minitour) or thoughtfully detailed (tekton)."

```
P cleancheck.py ENTRY [..] [--pic OUT.png] [--json OUT.json]   the checker (ENTRY = NPC id | def:PATH.json | build folder); exit 0 / 2
P cleancheck.py --calibrate                                    re-measure the 24 real creatures -> data/bands.json
P features.py ENTRY --box upLo,upHi [--zfrac a,b] --pic OUT    every small feature in a region with how it is built
P shells.py ENTRY ..     P horns.py     P boards.py [1 2 3]    P clean_watcher.py     P board_clean.py
P explore.py / explore2.py / explore3.py / explore4.py / explore5.py / construct.py     the measures tried (data/*.json); P tab.py KEYS
```

## 1. Sample and method

24 real creatures, chosen by looking at lit sheets (`look/cand.png`) for well-regarded mid and high-level creatures
of both kinds: Minotaur 12812, Tekton 7541, Tormented Demon 13599, Basilisk Knight 9293, General Graardor 2215,
K'ril Tsutsaroth 3129, Cerberus 5862, Nex 11278, Sol Heredit 12821, Vardorvis 12223, Lynx 11068, Jaguar 12976,
Hellhound 104, Greater demon 2025, Demonic gorilla 7144, Ba-Ba 11778, Yama 14176, Bull 15625, Skotizo 7286,
Callisto 6609, Dusk 7849, Vorkath 8061, Akkha 11789, Duke Sucellus 12191. Ours: the stone watcher as first judged
(`creature-stone-watcher-2`), its repaint (the picture these words are about), star hound 3 (graded Strong),
Warden-General III (not yet judged).

Every creature is the rest mesh as the client assembles it, lit by a NumPy port of the client's light (once, at
rest, by vertex vote; `cleanlib.py`; equal to the checked port `light_npc.py` on every lit corner of the six
entries tested, 49,719 corners, 0 differ). Measures are on the drawn opaque skin. About 150 candidate measures
were computed (`explore*.py`); a measure was KEPT only if all 24 real creatures lie on one side and both paints of
the watcher on the other. Seven survive (section 2); those dropped are listed in section 8.

## 2. The seven measures that separate (real band = minimum to maximum of the 24)

| line | real band (median) | Minotaur | Tekton | watcher as first judged | watcher repainted | star hound 3 | Warden-General III |
|---|---|---|---|---|---|---|---|
| 1 splotch: share of skin in small weak planes with no committed border | 0.000-0.158 (0.079) | 0.088 | 0.098 | **0.226** | **0.229** | 0.116 | 0.113 |
| 2 committed folds: hard-fold length / firm-fold length | 1.10-3.83 (1.61) | 1.10 | 1.41 | **0.68** | **0.70** | **0.59** | 1.22 |
| 3 firm folds (30-50 degrees), share of in-material edge length | 0.058-0.199 (0.155) | 0.190 | 0.172 | **0.231** | **0.227** | **0.205** | 0.143 |
| 4 one skin: faces in the largest welded piece | 0.655-1.000 (0.993) | 1.000 | 0.723 | **0.488** | **0.495** | 0.882 | 0.666 |
| 5 stuck-on: faces in closed pieces sunk into the skin | 0.000-0.124 (0.000) | 0.000 | 0.124 | **0.389** | **0.384** | 0.118 | 0.108 |
| 6 small closed lumps (pieces no longer than 7% of the height) | 0-2 (0) | 0 | 0 | **17** | **17** | 0 | 1 |
| 7 small contrasting features built as closed pieces | 0.000-0.034 (0.000) | 0.000 | 0.000 | **0.132** | **0.231** | 0.000 | 0.000 |

Bold = outside. The watcher is outside on 7 of 7 in both paints; the repaint moved none of them (it changed
colours only, and line 1 went from 0.226 to 0.229). Star hound 3 is outside on the two fold lines; Warden-General
III is inside on all seven (close to the edge on lines 4 and 5).

Definitions (exact, in `cleancheck.py`):
- **line**: a shared edge the eye can see: the two faces differ in paint, OR the lit lightness has a kink of 4 or
  more there (the light gradient across the edge changes by that much over 2% of the creature's height).
- **plane**: faces joined across edges that are not lines. **small**: under 0.2% of the skin.
- **mark**: a small plane 6 or more lit lightness from its surround, or bordered 70% by another material.
- **facet**: a small weak plane whose border is at least a third committed: a hard fold (50 degrees or more), a
  strong tone step (6 or more lit lightness) or another material.
- **splotch**: a small weak plane that is neither: it differs a little from its neighbours for no drawn reason.
- **hard fold** 50 degrees or more; **firm fold** 30 to 50 (uncommitted).
- **piece**: a shell sharing no vertex with the skin, at most 6% of the faces. **lump**: a piece with no open edge
  that touches the skin. **sunk**: a tenth or more of its vertices lie inside the main skin.

## 3. Complaint 1: [quoted words withheld] against "splotchy"

**The hypothesis as given was mostly wrong, and the way it was wrong is the finding.** I expected the watcher's lit
surface to be rougher face to face. It is not:
- roughness of lit lightness between a face and its same-material neighbours (rms, lightness steps): real 0.9 to
  10.7, Minotaur 2.3, Tekton 2.9; watcher 2.2 and 2.0. Paint alone: real 0.4 to 11.7; watcher 1.6, 1.5. Light
  alone: real 0.7 to 2.9; watcher 1.2, 1.1.
- vertex light number against the mean of its neighbours: real 6.5 to 13.8; watcher 9.0 (smoother than the median).
- single-face lit tone islands, slivers, neighbour size ratios, near-unwelded vertices, picture high-pass energy,
  share of weak, middling and strong picture edges: all inside the real band (section 8).
- (c) broken vertex votes: the watcher has 0 split vertices and no more near-coincident unwelded vertices than
  real creatures (0.07 against a real range of 0 to 0.50; Tekton's rock plates give 0.50). Not the cause.

What does separate is **where the small tone differences sit**. The watcher has as many small planes as a detailed
boss (skin in small planes 0.51; Tekton 0.45; Minotaur 0.24) but its small planes are weak and unanchored:
**23% of its skin lies in splotches** against 0 to 16% in 24 real creatures (Minotaur 9%, Tekton 10%). The
separation holds in all ten variants of the thresholds tried (`explore4.py`: small = 0.1 / 0.2 / 0.5% of skin, hard
= 40 / 50 degrees, anchored border a quarter / third / half, kink 3 / 4 / 6, weak 4 / 6): the watcher is 1.2 to 1.6
times the real maximum every time.

Cause, measured: 65% of the watcher's splotch border length is LIGHT ALONE (a kink with no paint step) and 35% a
paint step. The light part comes from uncommitted folds (complaint 2: firm folds carrying no paint change are 1.00
and 0.75 of hard-fold length on the watcher; real 0.18 to 0.70). The paint part was ADDED by the repaint: skin in
small weak one-colour patches rose from 0.063 to 0.117 (real maximum 0.110, Tekton 0.100, Minotaur 0.027), because
`PAINT.md` rule 5 ([quoted words withheld]) was applied to a mesh whose
planes are small: one soft step on every small plane is a mottle. That rule needs a size limit (section 6, S2).

**Two styles?** Deliberate small detail (marks + facets, share of skin) runs from 0.02 to 0.45 with no gap:
Callisto 0.02, Bull 0.03, Hellhound 0.05, Lynx and Basilisk Knight 0.09, Greater demon 0.10, Dusk 0.13, **Minotaur
0.15**, Graardor 0.18, Demonic gorilla 0.19, Yama and Duke 0.20 | Jaguar 0.22, Skotizo 0.23, Cerberus 0.26, Nex
0.28, K'ril 0.30, Vardorvis and Vorkath 0.31, Akkha 0.34, **Tekton 0.35**, Ba-Ba 0.38, Tormented Demon 0.40, Sol
Heredit 0.45. It is a scale, not two clusters; the Minotaur and Tekton sit a third and three quarters of the way
along it. Cutting it at 0.20 agrees with my labels made by looking on 19 of 24. The numbers per side:

| | smooth (under 0.20; 12 real) | detailed (0.20 or more; 12 real) | watcher repainted |
|---|---|---|---|
| splotch | 0.000-0.127, median 0.044 | 0.019-0.158, median 0.098 | 0.229 |
| marks | 0.005-0.064, median 0.032 | 0.058-0.185, median 0.151 | 0.069 |
| facets | 0.008-0.169, median 0.093 | 0.047-0.272, median 0.193 | 0.210 |
| skin faces | 519-2,982, median 1,710 | 1,020-5,238, median 2,425 | 3,842 |

So the watcher has a detailed creature's amount of small planes and facets, a smooth creature's amount of marks
(0.069: at the bottom of the detailed band), and more splotch than either. That is the owner's sentence in numbers:
[quoted words withheld]. A detailed real creature spends its small planes on marks
(another material or 6+ lightness: Tekton's orange seams, 0.185) and on facets bounded by hard folds (0.165).

Board: `boards/board-1-splotch.png` (splotch planes in magenta; real and ours).

## 4. Complaint 2: [quoted words withheld] features

**Whole creature: a real fold is committed.** Inside one material, real creatures have MORE edge length folded 50
degrees or more than folded 30 to 50: ratio 1.10 to 3.83 in all 24 (median 1.61). The watcher has 0.68 / 0.70 and
star hound 3 has 0.59: more in-between folds than creases. Firm folds are at most 0.199 of edge length in all 24;
watcher 0.23, hound 0.205. A firm fold is too sharp to shade smoothly and too soft to read as an edge; it shows as
a soft kink in the light. The amount of hard folding itself does NOT separate (hard-fold length per height: real
20.5 to 104; watcher 30), nor does the median fold (real 3.0 to 28.0 degrees; watcher 21): the watcher is not
"crumpled" by any overall figure. It is uncommitted.

**Horns, measured as objects** (`horns.py`; the horn's own faces):

| | joins the skull | inner edge length hard / firm | fold where it meets the skull | lit tone step there (grey levels) | length / own width |
|---|---|---|---|---|---|
| Minotaur horn (45 faces) | part of the welded skin | 0.44 / 0.03 | 80 degrees | 55 | 3.7 |
| Greater demon horn (42) | part of the welded skin | 0.38 / 0.09 | 42 | 27 | 5.5 |
| Graardor tusk (18) | part of the welded skin | 0.44 / 0.10 | 57 | 73 | 4.3 |
| watcher horn (48) + collar | welded, with a separate closed 56-face collar pressed round its base (20-23% sunk) | **0.26 / 0.25** | **20** | **23** | **2.7** |
| star hound 3 horn (88) | **separate closed shell** | **0.22 / 0.27** | none (not joined) | - | 3.7 |

Real horns: 3 of 3 measured are ridged (two fifths of their inner edge length is hard, a tenth or less firm), meet
the skull at a fold of 42 to 80 degrees with a tone step of 27 to 73 grey levels (the Minotaur's dark band), and
are 3.7 to 5.5 times as long as wide. The Bull's and the Tormented Demon's horns are also faces of the main shell
(no horn piece exists in their shell lists). The watcher's horn is rounded (as much firm as hard), stubby, and
meets the skull at 20 degrees with 23 grey levels: it grows out of the head like a soft cone, then has a ring
stuck on.

**Eyes and nostrils as defined features**: section 5 (a real eye's contrast with its surround is 12 to 128 grey
levels, median about 50, on 1 to 8 faces; the watcher's 57 to 73: contrast is NOT what it lacks; construction is).

**Head measures that did not separate** (11 real humanoid heads, `explore5.py`): share of faces on the head
(real 0.04 to 0.32; watcher 0.28), hard-fold density on the head, strong-line density, lit tone span, colours.

Board: `boards/board-2-defined.png` (hard folds cyan, firm folds with no paint change yellow).

## 5. Complaint 3: [quoted words withheld]: how real features are built

**Census of every small contrasting feature** (up to 7% of the creature's height, 20 or more grey levels from its
surround; `construct.py`): 1,536 on the 24 real creatures.

| construction | real, of 1,536 | watcher repainted, of 52 |
|---|---|---|
| paint: faces of the welded skin, flat on it (height within -0.12 to +0.20 of the feature's own width) | 1,050 (68%) | 37 |
| inset: faces of the welded skin, sunk | 216 (14%) | 0 |
| relief: faces of the welded skin, raised | 242 (16%) | 3 |
| separate piece, open where it meets the skin (Greater demon's 12 teeth and 2 eyes, 3 others) | 18 (1.2%) | 0 |
| separate CLOSED piece touching the skin (Graardor's two earrings) | 2 (0.1%) | **12 (23%)** |
| separate, not touching (Tormented Demon's flames, Basilisk Knight's and Jaguar's bits) | 8 (0.5%) | 0 |

98% of real small features are faces of the skin itself. Small closed lumps of any colour: 20 of 24 real creatures
have none, the most is 2 (Graardor's earrings, Sol Heredit's two 12-face studs; Tormented Demon and Skotizo 1
each); the watcher has **17**, 32 faces each at the median, 13% of all its faces.

**Eyes** (named by reading `feat/*.png` and the tables; 12 real creatures):

| creature | built as | faces | height above (+) / below (-) the skin, % of creature height | contrast, grey |
|---|---|---|---|---|
| Minotaur | skin faces, dark, under a brow fold, red lid patch beside | 2 | -0.15 | 43-55 |
| Bull | skin faces, red, + 1 black pupil face | 8 + 1 | +0.41 | 12-24 |
| Hellhound | skin faces, red | 2 | -0.42 | 31-52 |
| Lynx | skin faces, yellow, dark surround of 4 faces | 2 | +0.30 | 54-77 |
| Jaguar | skin faces, pale + 2 black | 2 + 2 | -0.13 | 126 |
| General Graardor | skin faces, yellow | 2 to 3 | +0.28 to +0.36 | 35-59 |
| Yama | skin face, bright on 4 darker | 1 | 0.00 | 89 |
| Nex | skin faces, white | 2 | -0.02 | 73 |
| Cerberus | skin faces, yellow | 1 to 2 | 0.00 to -0.47 | 122-128 |
| Skotizo | skin faces, violet | 2 | +0.01 | 17 |
| Tekton (visor slits) | skin faces, orange | 2 | +0.09 | 76 |
| Greater demon (2007) | a two-face OPEN card over 2 black inset skin faces | 2 | +0.12 | 37-40 |
| ours, star hound 3 (Strong) | skin faces, inset | 2 | -0.27 | 155 |
| **ours, watcher** | **a closed 26-face lump on a closed 32/38-face dark lump a third sunk in the head** | 26 + 35 | **+1.9 to +2.05** | 57-73 |

11 of 12 real eyes are 1 to 8 faces (median 2) of the welded skin, within 0.47% of the creature's height of the
surface; the twelfth is two open faces 0.12% above it. None is a closed piece. The watcher's eye stands 4 to 5
times prouder than the proudest real one and uses 30 times the faces.

**Nostrils** (4 real): Minotaur 2 dark skin faces, +0.13%; Bull 2 dark skin faces, 0.00; Callisto 1 dark skin face,
0.00; Lynx a nose of 9 to 10 dark skin faces, inset 0.7 to 1.3%. The watcher: a closed 26-face lump per nostril,
+1.5% of its height (visible in profile as two balls on the snout), plus four more closed dark lumps for the mouth
corners and lip and two skin-coloured brow lumps 75 to 92% buried.

**Larger pieces.** Faces in the largest welded piece: real 0.655 to 1.000, median 0.993 (17 of 24 at 0.94 or more);
watcher 0.49. Faces in closed pieces sunk into the skin: 21 of 24 real have none; Tekton 0.124 (ten rock plates of
20 to 32 faces on its legs: rigid slabs of the same stone, each one label), Graardor 0.025, Skotizo 0.005; watcher
0.384 (shoulders 196 + 190 faces, forearm guards 148 x 2, shin plates 108 x 2, loin plates, scar and studs, horn
collars, and the 17 small lumps). Real separate pieces are either open where they meet the body or rigid plates.

What ours does differently, in one line: the watcher's face is **12 closed lumps of 20 to 38 faces pressed onto a
head** (eyes, sockets, nostrils, mouth corners, lip, brows) where a real face is **2-face patches of the head's
own skin**; and half its faces are not in its skin at all.

Board: `boards/board-3-construction.png` (closed pieces red, open pieces green, floating blue).

## 6. Rules for an author (each with the numbers behind it; `cleancheck.py` checks S1, D1, D2, F3, F4, F5)

Surface (clean, smooth):
- **S1. At most 16% of the skin in splotches; aim for 10% or less** (24 of 24 real: 0 to 15.8%; Minotaur 8.8%,
  Tekton 9.8%; smooth creatures median 4%). A small plane must be a mark (another material, or 6+ lit lightness
  from its surround) or a facet (a third of its border a hard fold or a strong step). Otherwise remove it.
- **S2. No tone step under 6 lit lightness (about 8 paint lightness) on a plane under 0.2% of the skin unless the
  step lies on a hard fold.** This limits `PAINT.md` rule 5: [quoted words withheld] applies to planes of 0.2% of the skin
  or more; smaller planes share their neighbour's tone or become marks. (Skin in small weak one-colour patches:
  24 of 24 real at most 0.110, Minotaur 0.027; the repaint made 0.117.)
- **S3. A patch of skin between two creases is ONE smooth surface**: folds inside it under 30 degrees, shaded by
  the light alone (see D1).
- **S4. Choose a place on the scale and say it in the design**: smooth (marks + facets under 20% of the skin,
  marks about 3%, as the Minotaur 15% / 5%) or detailed (20 to 45%, marks 6 to 18%, as Tekton 35% / 18.5%). A
  detailed creature earns its small planes with marks in another material or strong tone, and facets on hard folds.

Definition:
- **D1. Commit every fold: under 30 degrees (smooth) or 50 or more (a crease).** Hard-fold length at least 1.10
  times firm-fold length (24 of 24; median 1.61; aim 1.4 or more); firm folds at most 0.20 of in-material edge
  length (24 of 24; median 0.155). A firm fold is allowed where it carries a paint change.
- **D2. A horn, tusk or spike is ridged and meets the head at a crease**: at least 0.38 of its inner edge length
  hard and at most 0.10 firm; a fold of 42 degrees or more where it meets the skull and a tone step of 27 grey
  levels or more there (a dark band, as the Minotaur's); 3.7 or more times as long as wide (3 of 3 measured).
- **D3. The horn is the skull's own skin** drawn out (5 of 5 looked at: Minotaur, Greater demon, Graardor's tusks,
  Bull, Tormented Demon). No collar piece.

Construction (features):
- **F1. An eye is 1 to 8 faces of the head's own skin (2 is normal), repainted**, lying within 0.5% of the
  creature's height of the surface, 12 or more grey levels from its surround (median about 50), under a brow that
  is a fold of the same skin (11 of 12; the twelfth is a two-face open card). Cut the skin so the eye's outline is
  edges; do not add geometry on top.
- **F2. A nostril is 1 or 2 dark faces of the snout's skin**, flat (3 of 4; the fourth a 10-face inset nose). A
  mouth is a dark inset or a painted line of skin faces.
- **F3. No closed lump at eye scale** (no closed piece shorter than 7% of the height): 20 of 24 real have none,
  none has more than 2.
- **F4. One skin.** The largest welded piece holds at least 0.655 of the faces (24 of 24; median 0.993). A separate
  piece is a rigid thing of one label (plate, tooth, ring, weapon): open where it meets the body, or a slab; closed
  pieces sunk in the skin hold at most 0.124 of the faces (24 of 24; 21 of 24 have none).
- **F5. 98% of small contrasting features are faces of the skin** (1,508 of 1,536): flat paint 68%, inset 14%,
  raised 16%. At most 3.4% may be closed pieces (24 of 24; 23 of 24 have none).

## 7. The mechanical test (`clean_watcher.py`, `watcher-clean/`)

The repainted watcher, by script only: (A) small closed lumps removed and their footprints cut into the skin as
two flat faces each, mirrored left and right (10 lumps inset: eyes, sockets, nostrils, mouth corners, lip; 2
skin-coloured brow lumps removed; 5 could not be cut: three scar studs and two loin plates, kept); (B) 214 hidden
faces of the larger pieces deleted so they are open where they meet the skin; (C) 108 weak small paint patches
given their neighbour's tone; (D) 133 vertices away from hard folds, paint boundaries and other pieces relaxed
along their normals (at most 1.2% of the height). No vertex removed or renumbered: 131 labels before and after;
standing, walk and attack draw whole in the harness (`watcher-clean/board-clean.png`, same layout as board-65).

| line | real band | before | after | |
|---|---|---|---|---|
| 1 splotch | 0-0.158 | 0.229 | 0.140 | now INSIDE (still above Minotaur 0.088 and Tekton 0.098) |
| 2 hard / firm folds | 1.10-3.83 | 0.70 | 0.82 | still OUTSIDE |
| 3 firm folds | 0.058-0.199 | 0.227 | 0.209 | still OUTSIDE |
| 4 largest welded piece | 0.655-1 | 0.495 | 0.580 | still OUTSIDE |
| 5 stuck-on closed pieces | 0-0.124 | 0.384 | 0.021 | INSIDE, **but by deleting unseen faces: the look did not change** |
| 6 small closed lumps | 0-2 | 17 | 5 | still OUTSIDE |
| 7 features as closed pieces | 0-0.034 | 0.231 | 0.000 | now INSIDE |

Three of seven move inside; one of those (line 5) moved without any change a viewer could see, which shows that
line can be satisfied cheaply and should be read with line 4. Folds and welding cannot be reached by a script: an
earlier version of step D that relaxed every vertex away from hard folds reached 0.98 and 0.185 on lines 2 and 3
but pushed the skin through the shoulder pieces (dark nicks; `work/diag.png`), so it was withdrawn.

**How it looks to me (my view, not a measurement).** In profile the head is plainly cleaner: the two balls on the
snout and the bulging eye are gone and the outline is one form (`watcher-clean/close-head.png`, lower row). The
chest carries fewer faint patches (`close-torso.png`). From the front the face is NOT better and may be worse: the
cut eyes are thin slivers inside dark shards, smaller and less shaped than the lumps they replace, because a
script can only cut the footprint the lump happened to have into the triangles the head happened to have. At game
size on the board I can barely tell the two apart. So: the construction rule removes the [quoted words withheld] reading, but a
well-shaped eye, brow and nostril have to be DESIGNED into the head's own faces; and the fold and one-skin lines
need the body rebuilt, not filtered.

## 8. Tried and dropped (none separates all real from the watcher; real range, then watcher first paint / repaint)

- face-to-neighbour roughness of lit lightness, rms: 0.9-10.7; 2.2 / 2.0. Paint alone 0.4-11.7; 1.6 / 1.5. Light
  alone 0.7-2.9; 1.2 / 1.1.
- vertex light number against its neighbours' mean: 6.5-13.8; 9.0 / 9.0; local extrema of the light number
  (prominence 3, 6, 10): inside.
- single-face lit tone islands (2, 4, 6 lightness): share of faces 0.001-0.145; 0.034 / 0.022.
- one-colour tone islands of 1, 2, up to 4 faces: inside. Paint boundaries: median turn, sharp turns, junctions,
  dangling ends: inside (median turn 36-90 degrees; 83 / 62).
- fold distribution: flat, gentle and "mush" 15-30 degree shares, median fold, concave share of soft and of firm
  folds: inside or not on the bad side. Vertices where the fold changes sign four or more times round the ring
  ("crumple"): real 0.21-0.45; watcher 0.215 (LESS than nearly all).
- slivers (aspect over 5): 0.02-0.21; 0.10. Neighbour area ratio, area spread, tiny faces: inside.
- welding: split vertices 0 everywhere; near-coincident unwelded vertices 0-0.50; 0.07.
- picture measures (front and back at 420 px per height): high-pass rms inside one material 2.2-15.6; 6.3;
  weak / middling / strong edge shares, share of tone change carried by strong edges, calm-region sizes: inside.
  Gini of the gradient: all four of ours just under the real 10th centile, but the Minotaur is too.
- small weak planes without the border test: 0.012-0.428; 0.400 / 0.440 (the repaint just outside, Sol Heredit
  0.43): it needs the committed-border test to separate cleanly.
- skin in small planes, kink shares, planes per 100 faces, light-only and paint-only line shares: inside.
- crease chains (share of hard-fold or strong-line length in long / short chains, single edges): inside or within
  a hundredth. Hard folds that also carry a tone step: 0.04-0.53; 0.31 / 0.38.
- head measures on 11 humanoid heads: nothing.
- firm folds with no paint change over hard-fold length: 0.18-0.70; 1.00 / 0.75: separates, but only just for the
  repaint; shown by the tool as information and as the yellow lines of its picture, not as a verdict.

## 9. Not proven, and what I am unsure of

- **Chosen after looking.** About 150 measures were tried on 24 + 4 creatures and seven kept because they
  separate. With that many tries some separation is expected by chance. Lines 4 to 7 separate by a factor of 3 to
  8 and have a plain mechanism; line 1 by 1.45 and has six thresholds (stable across ten variants, but all tried
  on the same creatures); lines 2 and 3 by 1.6 and 1.14.
- **The fold lines also fail star hound 3, which the owner graded Strong.** So they are not the owner's eye; they
  may matter less on a furred quadruped than on a stone humanoid, or the hound may be read as lacking definition
  too if it were put beside a real one with these words in mind. Not known.
- **No creature built to these rules has been shown to the owner.** That splotch, committed folds and built-in
  features are what the owner meant is inference from one criticised creature in two paints.
- The band is the minimum to maximum of 24 hand-picked creatures; one new real creature can move an edge. Tekton
  alone sets the upper edge of line 5, Akkha the lower edge of line 4, Nex and Sol Heredit the top of line 1.
- The two styles are a cut I placed at 0.20 on a continuous scale; 5 of my 24 labels by looking disagree with it.
- Eyes and nostrils were named by me from tables and small pictures on 12 and 4 creatures; Callisto's eye was left
  out as uncertain; K'ril, the gorilla and Dusk were not named (their heads are not mirrored vertex for vertex).
  "Height above the skin" uses a plane fitted to the feature's border and is unreliable for long things (horns).
- "Sunk" and "touching" use ray casting against the largest shell, which is assumed closed.
- The light kink is my construction (gradient change across an edge over 2% of the height), not a client quantity.
- Translucent faces are left out everywhere; bone-weighted creatures are measured as rest meshes like the rest.
- Horns: three real horns measured as objects, two more looked at.
- The cleaned watcher was judged by me from my own renders and the harness board; nobody else has seen it.
