# Capes: how real ones are built, labelled and layered (step 6, brief)

Three real capes: Red cape 1007 (plain, old), Fire cape 6570 (textured), Attack cape 9747 (skillcape: hood, front
ties, emblem). Scripts in this folder; pictures in `{ROOT}/render/research-items-capes/`.

Commands run (Python = `{PYTHON} -I`):
- `research_pose.exe CACHE info item:1007 item:6570 item:9747 item:9748 item:2412` -> `info.txt`
- `research_pose.exe CACHE info skeleton:0` -> `skeleton0.txt`
- `cape_measure.py` -> `cape-measure.txt` (rows, priorities, colours; cape groups used by sequences; travel per label)
- `cape_render.py` -> `cape-render.txt` and 16 sheets `{male,female}-yaw{192,704,1024,1344}-{actor,depth}.png`
- `tools/label_armour.py --check 1007 | 6570 | 9747 [--template 1007]`, `--check 1019 --template 1007`
- `tools/reencode_item.py 1007 red-cape-reencoded.bin`, then `tools/pose.py ... "item:1007,model:...@cape,none" --seq 824 --frames bind,1,3,5 --yaw 1024 --diff 1:2`

## 1. Definition

- Wear positions `[1, -1, -1]` for all three; over the cache 374 of 374 capes blank no slot
  (`data/measure-items.txt`). One worn model per body type, wear offsets 0 / 0, no head model.
- Female model: 100% of capes have one. Cache-wide 71% are a different mesh, 12% same topology reshaped, 8% a
  moved copy (down 2 to 9 units), 4% the same ids.
- Inventory model: a separate flat mesh lying on the ground (Red cape 63 vertices / 160 faces, 107 x 12 x 122;
  Fire cape 41 / 35, a single sheet 2 units up, textured; Attack cape 115 / 167, folded, 64 x 17 x 124), no labels.
  Icon views: (zoom, xan, yan, zan) 2140, 400, 948, 0 / 2086, 567, 2031, 0 / 2128, 504, 0, 0. Cache-wide 92.5% of
  cape inventory models are a different mesh from the worn one.
- Recolours do the families: Saradomin cape 2412 and Black cape 1019 are worn model 323 (the Red cape) recoloured;
  Attack cape (t) 9748 is model 18920 with other recolours.

| | Red cape 1007 | Fire cape 6570 | Attack cape 9747 |
|---|---|---|---|
| male worn model | 323: 61 v, 162 f | 9638: 42 v, 63 f, 24 textured faces | 18920: 311 v, 636 f |
| female worn model | 481: 59 v, 160 f | 9640: 39 v, 63 f | 18969: 311 v, 636 f (same counts, top moved) |
| male box (x, height, z) | -27..27, 37..165, -10..27 | -34..34, 30..165, -10..27 | -32..32, 18..177, -17..24 |
| female box | -27..27, 53..160, -8..25 | -31..31, 33..161, -10..28 | -32..32, 18..175, -19..24 |
| priorities (faces) | 1: 74, 5: 88 | 0: 1, 1: 24, 5: 38 | 0: 150, 5: 80, 6: 404, 7: 2 |
| face labels | 4, 5, 6, 7, 8 | 4, 5, 6, 7, 8 | 3, 4, 5, 6, 7, 8 |

## 2. How a cape is built

It is a two-sided sheet made of two layers of faces on the same vertices:
- the OUTSIDE (normals to +z, the back): priority 5, the cape's colours (Red cape 7700 and 11200 recoloured, hem
  band; Fire cape texture; Attack cape 920 and trim);
- the LINING (normals to -z, toward the body): priority 1 on the Red and Fire capes (one dark colour, 926 / 924;
  on the Red cape these 74 faces are flat-shaded, render type 1), priority 0 on the Attack cape (colour 54503);
- the COLLAR on the shoulders and chest: label 8, priority 5 (Red, Fire: 14 faces, colour 6032), and on the
  skillcape the whole hood and neck piece: 167 vertices of label 8, priority 6 (404 faces), two faces at 7.

Rows (male, height of the row; z grows away from the back as it falls: 12 at the neck to 24 to 27 at the hem):

| label | Red cape | Fire cape | Attack cape |
|---|---|---|---|
| 2 collar seam | 2 vertices at (+-7, 164, -8) | the same 2 | none |
| 8 collar / hood | 10, height 160 to 165 | 10, 159 to 165 | 167, 146 to 177 |
| 10 top row (swing pivot) | 4 at 158, z 12 to 13 | 6 at 158 | 9 at 149 to 159, centroid (0, 155, 13) |
| 11 | 1 at (0, 158, 13) | 2 at 158, |x| 31 (the wide corners) | 34, 98 to 139 (panel) |
| 9 first bend | 11 at 112 | 5 at 118 (+ one label 16) | 24 at 96 to 110 |
| 14 second bend | 11 at 73 | 6 at 80 | 24 at 65 to 70 |
| 13 third bend | 11 at 48 to 50 | 3 at 54 to 58 (+ two label 15) | 16 at 24 to 31 (+ eight label 15) |
| 12 hem | 11 at 37 to 39 | 5 at 30 to 40 | 9 at 18 to 24 |
| 6 / 7 front ties | none | none | 12 at height 110, 8 at 80 to 88, z -17 to -11 |

A plain cape is 5 rows of 11 vertices. A modern one is taller (hem at 18 against 37), carries a hood, and hangs two
ties down the chest (labels 6 and 7).

## 3. How it swings: authored in the animation, not simulated

Skeleton 0 holds a chain of pivot + turn groups for the cape (`skeleton0.txt`):
group 36 pivot [10] -> 37 turn [14, 9, 13, 15, 12, 11, 16]; 38 pivot [9] -> 39 turn [14, 13, 15, 12, 16];
40 pivot [14] -> 41 turn [13, 15, 12]; 42 pivot [13] -> 43 turn [12]; 44 pivot [6] -> 45 turn [7] (the ties).
(Groups 185 to 199 are a second copy of the chain with a scale group; 136, 139, 162, 165, 188 are fades.)
Each pivot is the centroid of that label's vertices on the figure, so the row labels 10, 9, 14, 13 are both the hinge
and its position. The frames carry the turns (largest value in the frame data, x axis only):

| sequence | 37 (whole cape) | 39 (below row 9) | 41 (below 14) | 43 (hem) | 45 (ties) |
|---|---|---|---|---|---|
| stand 808 | not used | 3, 12 of 12 frames | | | |
| walk 819 | 3, 6 of 8 frames | 4 | 8 | 15 | 7 |
| run 824 | 9, 8 of 8 | 6 | 8 | 15 | 16 |
| slash 390 | 4 | 5 | 6 | | |
| stab 386 | | 4 | | | |

Largest travel of a vertex (cape posed alone with `animlib.apply_frame`; whole-body groups then pivot on the cape's
own vertices, so the figures for labels 8 and 10 are approximate): walk, hem (12) 27 to 32 units against 9 to 11 for
rows 9 and 14; run, hem 52 to 59. Male and female agree within a few units. So the animator turned each segment by
hand, frame by frame; a cape with the standard rows swings in every sequence that has those groups, and a cape with
no row labels (all 8) would hang as a rigid board.

## 4. Layering (the game's painting of a player)

Priorities met: feet 0, legs kit 1, torso 3 with abdomen 2, armour 2 to 4, hair and helms 6 to 7, arms, hands and
weapons 10 (10 is slotted by depth). The cape puts its lining at 0 or 1 and its outside at 5:
- from the front the lining is painted first or with the legs (inside priority 1 faces go far to near, and the
  lining is farther), so legs, torso and armour paint over it;
- from behind the outside (5) is painted over torso, legs and body armour (2 to 4), and arms and weapons (10) are
  placed by depth: nearer the camera than the cape they paint over it, farther they are covered;
- a hood at 6 shares the helm's priority and is sorted against it by depth.

Looked at (`male-yaw192-actor.png`, `male-yaw704-actor.png`, `male-yaw1024-actor.png`, `female-yaw1024-actor.png`):
front quarter, the cape shows only beside and between the legs; side, it hangs clear behind the back and legs and
the staff / halberd shaft crosses in front of it; back, it covers torso and legs with the arms and the weapon over
its edges. Nothing of the body shows through in any of the 3 frames looked at.

Count (`cape-render.txt`): pixels where the game's painting differs from a true depth picture, walk frames bind, 1, 5,
with the cape | the same outfit without it (a figure covers about 7,000 to 8,000 pixels):
- male, Red cape on the bare figure: 11 to 50 | 11 to 50 at the back, 33 to 44 | 23 to 35 elsewhere: the plain cape
  adds at most about 20 pixels.
- male, Red cape over Rune platebody and platelegs: 48 to 102 | 32 to 94. Fire cape: 69 to 139. With a Dragon halberd:
  55 to 109 | 39 to 102.
- male, Red cape over the MERIDIAN LEGS (priority 2): 3 to 60 | 6 to 68 over four sides: no more than without the
  cape. The FINDINGS worry (legs at 2 untested against capes) is answered for the lining at 1 and the outside at 5:
  the legs paint over the lining from the front and under the outside from behind, as real platelegs do.
- Attack cape: 139 to 324 over Rune, 58 to 208 over the Meridian legs, against 32 to 94 without. The extra is the
  skillcape's own priority-6 hood and ties painted over its neighbours; it is how the real item draws.
- female: Red cape over Rune 15 to 175 | 7 to 29; the largest (119 to 201) are from behind and the sides. NOT
  examined close up; the female sheet looked at shows no visible fault at game size.
- with the Meridian staff: male 39 to 90 | 36 to 84. (On the female sheets the staff is the male file, not moved to
  the female hand; ignore that column's hand position.)

## 5. Female

- Red cape: the top stays at 158 to 160 but the cape is shorter: rows at 124, 84, 63, 53 against 112, 73, 49, 38
  (the hem 15 units higher), collar ring narrower in z (-8..8 against -10..12), no label-2 collar vertices.
- Fire cape: top row 155 against 158, lower rows within 1 to 3 units, width 31 against 34.
- Attack cape: same vertex and face counts; only label 8 and 10 vertices move (hood and top 2 units down, front 2
  to 3 forward); rows 9 to 12 identical.
- Anchors: male capes share the head/body collar points (+-7, 164, -8) with label 2 in 25% of 180 distinct meshes;
  female capes do not use the female collar points at all (most shared female coordinates: (0, 156, 14) and
  (0, 158, 13) label 8, (+-7, 160, -6) label 8, top row (+-15, 156, 12) and (0, 147, 15) label 10, each about 18 to
  20% of meshes). A cape has no seam it must meet: it lies over the shoulders.

## 6. Tools

| check | result |
|---|---|
| `label_armour.py --check 1007` (rule only) | exact label 96.7%, same transforms 98.4%, face labels 58.0% |
| `--check 1019 --template 1007` (same mesh) | 100%, 100%, face labels 59.9% |
| `--check 9747` (rule only) | exact 70.7%, same transforms 76.2%, face labels 77.2%; 74 vertices move wrongly (13->12, 9->16, 15->12, ties 6 and 7 -> 8) |
| `--check 9747 --template 1007` | exact 67.5%, same transforms 75.9%: the plain template does not help a modern cape |
| `--check 6570` | refused: [quoted words withheld] |
| `reencode_item.py 1007` then `pose.py ... model:...@cape` against `item:1007`, run 824 frames bind, 1, 3, 5, back view | 0 of 120,000 pixels differ in all four: our encoder and the pose tools carry a cape (labels, two priorities, flat lining) exactly |
| `reencode_item.py 6570` | refused (textured) |
| `gate.py` | no cape: `ALLOWED = dict(head, body, legs, hands, feet, weapon)`; a cape cannot be gated |
| `pose.py` / `fpose.py` | accept `item:ID` capes and `model:PATH@cape` on both body types |

What a cape author needs, and does not have:
1. A written cape pattern: two layers on shared vertices (outside priority 5, lining 0 or 1 in one dark colour,
   flat-shaded), collar label 8 at priority 5, rows 10 / 9 / 14 / 13 / 12 at the heights above, z 12 to 27.
   `LABELS.md` has the rows; it does not have the two layers or the priorities.
2. Labels by row index, not by height: the rule is right on a plain cape (97%) and wrong on a tall one (71%).
   The author knows which ring is which; `label_overrides.py` (tags or boxes) can carry that. Ties (6 / 7) only by hand.
3. A cape entry in `gate.py` (allowed priorities 0, 1, 5, 6; motion check from the back and sides).
4. The female cape: for a plain cape a shorter rebuild (rows 11 to 15 higher), not a shift; no tool derives it.
   `female_warp.py` has cape pairs (50 sources) but was not tested on a cape here.
5. Textures: an animated cape like the Fire cape is textured faces; our encoder and `reencode_item.py` do not write
   them, so that kind is out of reach until the encoder carries face textures and texture coordinates.
6. An inventory model: a separate flat or folded mesh (no tool; 92.5% of real ones are their own mesh).

## Not proven

- No cape of ours was built; everything above is from real capes.
- Travel per row was measured on the cape alone, not on the assembled figure.
- Layering was counted in walk frames only (not run or attacks), three frames per side, and judged by eye at game
  size on four of the sixteen sheets; the female back-view counts (up to 201 pixels) were not examined close up.
- Capes against a helm with a neck guard, a shield, or a body piece with priority-4 back devices: not tested.
- The female cape against our derived female armour: not tested (those files did not exist when this was run).
- Whether the bone-weighted (archive 22) animations move capes the same way: not looked at.
