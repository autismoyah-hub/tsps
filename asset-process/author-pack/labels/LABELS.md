# Animation labels for armour (player skeleton 0)

Pictures in this folder: `atlas-figure.png` (whole bare figure), `atlas-head.png`, `atlas-body.png`,
`atlas-legs.png`, `atlas-feet.png`, `atlas-hands.png`, `atlas-cape.png`. Each shows the real body kit and one or two
real armour pieces, front and right side, every vertex in the colour of its label, the label number on its region,
a grid every 10 units.

Tool: `tools/label_armour.py OURS.bin SLOT OUT.bin [--template ITEM_ID]`. It applies the rules below, prints the
label histogram, the joint pivots and every vertex it was unsure of.
**A body piece that keeps the figure's own arms** (a chainbody, a leather body: worn as `@body` WITHOUT `+arms`;
the real ones declare wear positions [4, -1, -1]) must be labelled with `--sleeveless`: only 2, 4, 5, 8, no arm
label whatever the x, seam labels only at the exact seam coordinates, no shoulder or elbow pivot (the arms kit
must stay). Tested on five real sleeveless bodies: 0 differing pixels from the real item in walk, run and slash.

## Coordinates

Worn space of the standard male figure. Height = minus the cache y. Feet at 0, waist seam 114, wrist seam 114
(arms hang), elbow 131, shoulder 156, collar 164, crown 196. **The character's right side is negative x** (the
weapon hand, label 27, is at x = -31). **Front is negative z** (toes at z = -18, cape at z = +13 and more).

## How a label moves a vertex (read this once)

A frame is a list of groups applied in order. A *pivot* group sets the pivot to the **centroid of all vertices on
the whole figure that carry its labels**; the *turn*, *move* and *scale* groups after it transform every vertex
whose label is in their list. Two consequences decide everything below.

1. **A label names the last bone a vertex follows.** Label 19 (right forearm) is in the lists of the upper-body
   turn, the right-shoulder turn and the right-elbow turn; label 20 (right upper arm) is in the first two only.
2. **A few labels also define a joint.** The elbow turns about the centroid of the label-17 vertices, the shoulder
   about the centroid of label 21, a knee about the centroid of labels 35+37 (34+31), a hip about label 40 (42), the
   upper body about label 29, the toes about label 47 (48), the head about labels 3+2, a hand about label 27 (28)
   itself. Those vertices must exist on the figure and their centroid must sit at the joint. Our piece replaces the
   body kit that carried them, so **our piece must carry them**. With no label-17 vertex anywhere the forearm turns
   about the origin: measured on a real platebody, vertices end 45 (walk) to 158 (run) units from where they belong.
   Moving the ring only shifts the limb: giving the platebody's whole upper arm the elbow label moved vertices by
   at most 13 units.

An unlabelled model (all 0) does not animate at all: label 0 is in no limb group and not even in the whole-body
shift of walking (measured: label-0 vertices travel 0.0 units in stand, walk, run, slash, stab), so the piece stays
at the rest pose while the figure walks out of it. Label 255 is in no group either.

## The standard rigid set, slot by slot

`R / L` = the character's right (x < 0) / left (x > 0). Heights are where real pieces change label most often
(fitted on every distinct real worn mesh of the slot); real families differ by a few units, see "limits".

### Head (slot 0): labels 1, 2, 3
- **1** everything: skull, face, visor, crest, plumes.
- **3** the neck: four vertices at (+-3, 169, -9) and (+-5, 173, 2).
- **2** the neck base ring, four vertices at exactly (+-7, 164, -8) and (+-7, 165, 4). The hair kit, most helms
  and 98% of body pieces all put a vertex at these four coordinates; the client merges equal coordinates, so the
  collar is one ring shared by head and body. A full helm hides hair and jaw, so it should carry 2 and 3.
- Optional **49** (15% of helms): a part that must rest on the chest and not turn with the head (a neck guard, a
  hood's collar). 69 and 94 are rarer variants of the same idea.
- 183 of 469 head pieces (hats, masks that hide nothing) use label 1 alone: the hair kit still supplies 2 and 3.

### Body (slot 4, with arms when the piece hides slot 6): labels 2, 8, 4, 5 and 21 20 17 19 (R), 25 26 23 22 (L)
- Chest **8**: everything with |x| < 19.
- Neck base ring **2** at the four collar coordinates above.
- Waist seam: **5** at (+-13, 114, 8); **4** at (+-11, 114, -8), (+-11, 123, 5), (+-8, 123, -8). Half the real
  bodies have these (the plate family); the other half use 8 or 86 there. In walk, run, slash and stab the
  difference is nil (measured 0 units on the platebody); 4/5 are left out of the upper-body shift used by 14 of
  the 93 sequences examined (stand 808 among them).
- Arm = |x| >= 19 (>= 22 below height 118, where robes flare). Along the arm:
  - **21 / 25 shoulder**: height >= 147. Its centroid is the shoulder pivot: real armour has it at
    (-+23.5, 156, 0).
  - **20 / 26 upper arm**: 135 to 147.
  - **17 / 23 elbow ring**: 130 to 135. Four vertices in the kit, at (-+31, 132, 4 and -3) and (-+23, 130, 4 and
    -3); centroid (-+27, 131, 1) in 80% of real bodies. This ring is the elbow.
  - **19 / 22 forearm**: below 130, ending in the wrist ring at exactly (-+32, 114, -2), (-+25, 114, -2),
    (-+25, 114, 4), (-+32, 114, 4), which 92% of bodies and 99% of gloves share.
- Convention at the elbow: the ring itself (17) belongs to the UPPER arm; everything below it is forearm.
  At the shoulder the cap (21) belongs to the arm, and its inner edge sits against the chest at |x| = 16 to 18
  without sharing coordinates with it.

### Legs (slot 7): 30 5 29 41 39, then 42 44 34 31 32 (R), 40 43 35 37 38 (L)
- **30** waist ring front (z < 0) and **5** waist ring back, height >= 113.5; seam points (+-13, 114, -8) and
  (+-13, 114, 8), the latter shared with the body.
- **29** hip ring, 108.5 to 113.5, kit points (+-13, 110, +-8). Its centroid, (0, 110, 0), is the point the whole
  upper body turns about.
- **41** pelvis, 101 to 108.5. Moves only with the whole body (no limb group contains it).
- 92 to 101: **39** crotch centre (|x| <= 2), **42 / 40** hip joints at the sides (kit: (-+17, 97, -9) and
  (-+17, 91, 9)). The centroid of 40 is the left hip pivot: real legs have it at (14, 95, 0).
- **44 / 43** thigh, 62 to 92.
- **34 / 35** upper knee ring, 56 to 62 (kit 58 to 61). Thigh side.
- **31 / 37** lower knee ring, 46 to 56 (kit 46 to 48). Shin side. The knee turns about the centroid of both
  rings together, (-+9.6, 54, -1) in real legs.
- **32 / 38** shin, below 46, ending in the ankle ring at exactly (-+12, 18, -4), (-+6, 18, -4), (-+12, 19, 6),
  (-+6, 19, 6), shared by 92% of legs and 90% of boots.
- Convention at the knee: two rings, one each side of the joint; a knee cap that stands forward of the knee
  (z <= -12) goes with the SHIN (real plate legs label it 33 / 36, which move as 31 / 37 do).

### Feet (slot 10): 0, then 32 48 45 (R), 38 47 46 (L)
- **32 / 38** ankle and heel: height >= 8, or height + z >= 1.5.
- **45 / 46** toes: z <= -10, or height + z <= -3.
- **48 / 47** the hinge between them: a ring of four vertices, in the kit at (-+6..15, 2, -3) on the instep and
  (-+3..15, -6, 5) under the heel. The toes turn about its centroid, (-+9.5, 0.5, 1).
- **0** root anchor: a triangle under the ground at exactly (-6, -6, -2), (6, -6, -2), (0, -6, 6). The feet kit
  and 83 of 141 boots carry it; whole-body turns (26 of 93 sequences, run 824 and slash 390 among them) pivot on
  its centroid. Include it.

### Hands (slot 9): 19 27 (R), 22 28 (L)
- **19 / 22** cuff: height >= 103, starting at the wrist ring coordinates above (height 114).
- **27 / 28** hand: below 103. The hand turns about its own centroid, (-+31, 96, 1).

### Cape (slot 1): 8, then rows 10, 11, 9, 16, 14, 15, 13, 12 downwards
A cape is a chain of four swinging segments. **8** is the collar on the chest. **10** is the top row (its centroid,
about (0, 157, 13), is the pivot of the whole swing). **9**, **14**, **13** are the rows where it bends (about
heights 112, 73, 46 in a plain cape) and **12** the hem; **11**, **16**, **15** are panel vertices between the
rows, moving exactly as 9, 14, 13 do. The red cape (item 1007) uses only the five rows. The tool guesses cape rows
from height alone and flags every vertex; `--template 1007` (or another cape of the same cut) settles the unsure
ones.

## Why families differ: the optional extras

| extra | what it is | how it moves | seen in |
|---|---|---|---|
| 18 / 24 | upper arm, upper half (robes, shirts) | exactly as 20 / 26 | 77% of bodies; Virtus robe top 26243, Mystic robe top 4091 |
| 86 | lower torso / hem | as chest 8 (follows the upper-body shift, unlike 4 / 5) | 45% of bodies; Virtus robe top, d'hide bodies 1135 (2 vertices at the back waist seam) |
| 59 / 60, 91 / 92 | shoulder pads (L / R) | as chest 8: they do NOT swing with the arm | Virtus robe top (59, 60), Void knight top 8839 |
| 58 / 93 | shoulder pads, old form | as 8 except in the newer upper-body group 206 | Dharok's platebody 4720 |
| 49, 69, 94 | helm or hood parts resting on the chest | as chest 8: they do not turn with the head | Dragon med helm 1149 (49), hoods |
| 33 / 36 | knee cap | as the lower knee ring 31 / 37 | 36% of legs; Rune platelegs 1079 |
| 79, 78, 168, 74, 75 | skirt centre panels | only with the whole body, as pelvis 41 | Virtus robe bottom 26245 (79), plateskirts (78) |
| 57 | skirt centre | follows the LEFT thigh | Rune plateskirt 1093 |
| 80 / 81, 166 / 167 | skirt, inner thigh columns (R / L) | as thigh 44 / 43 | Virtus and Mystic robe bottoms |
| 82 / 83 | skirt, inner columns at the knee (L / R) | as shin 38 / 32 | the same |
| 76 / 77 | skirt hem, inner (L / R) | as shin 38 / 32; they define the pivot of a skirt-swing group (218-222) that walk, run, slash and stab do not use | the same |
| 6, 7 | hanging front flap | 7 swings about the centroid of 6 | skillcape trims 9747, stoles 10470 |
| 67 / 68 | forearm, without the upper-body shift | as 19 / 22 otherwise | Bandos chestplate 11832, a few gloves |

So the robe top's "missing" labels are not cloth simulation. 18 and 24 are a second name for the upper arm; 86 is a
hem that stays with the chest; 59 and 60 are shoulder pads pinned to the chest. A robe bottom is one cone cut down
the middle: the outer columns carry the ordinary leg labels, the inner columns carry skirt labels that move exactly
like the leg beside them, and a few centre vertices at the top stay with the pelvis. The cloth "sways" only because
each half follows its own leg.

Labels that receive identical transforms in all 93 sequences examined (they differ only in which pivot they
define): {1, 3}, {2, 8, 10, 49, 59, 60, 69, 86, 91, 92, 94}, {4, 5, 6, 29, 30}, {17, 18, 20, 21}, {23, 24, 25, 26},
{31, 32, 48, 77, 83}, {34, 42, 44, 80, 166}, {35, 40, 43, 81, 167}, {37, 38, 47, 76, 82}, {41, 74, 75, 78, 79, 168},
{39, 57}, {9, 11}, {14, 16}, {13, 15}.

## Labels shared between slots

2 (collar: head, body, capes, amulets), 5 (back waist: body and legs), 19 / 22 (wrist ring: body and hands),
32 / 38 (ankle ring: legs and feet), 8 (anything fixed to the chest: body, capes, amulets). Where two slots meet,
real pieces put vertices at the SAME coordinates with the SAME label; the client merges them into one vertex, and
when the labels disagree the piece in the lower-numbered slot wins (head 0, cape 1, body 4, arms 6, legs 7,
hands 9, feet 10).

## Face labels

Face labels are used only by alpha groups, which fade faces (3 of the 93 sequences examined use one: 714, 2890,
6111). Walk, run and attacks ignore them. Real armour still carries them, by region: head 0, neck 1, chest 4,
abdomen 2, waist 17, R upper arm 9 (shoulder 11), R forearm 10, L upper arm 14 (shoulder 13), L forearm 12,
R hand 15, L hand 16, pelvis 22, L thigh 23, R thigh 24, L knee 20, R knee 19, L shin 21, R shin 18, skirt 31,
R foot 25, L foot 26, root triangle 27, cape 4 / 5 / 8 / 7 / 6 from the top down. The tool derives them from the
vertex labels (it agrees with the real face labels on 50% to 100% of faces in the armour test items, lowest
where the vertex labels themselves differ, and on only 40% for the cape, whose rows it assigns to the wrong
band).

## Limits of the rule (measured, not hoped)

Agreement of the rule alone with the real labels over every distinct real worn mesh (held-out test meshes apart):

| slot | meshes | exact label | same transforms in walk / run / attack |
|---|---|---|---|
| head | 467 | 97% | 99% |
| hands | 80 | 94% | 94% |
| feet | 140 | 85% | 91% |
| body | 325 | 79% | 87% |
| legs | 263 | 66% | 86% |
| cape (with rows) | 131 | 63% | 77% |

- **Shoulder seam.** Between |x| = 15 and 23 above height 135 real pieces disagree with each other: a platebody
  labels its pauldron 21 / 25 out to height 171, the Mystic robe top labels its shoulder wings 8. Position cannot
  tell them apart. The tool flags the zone.
- **Skirts.** A plateskirt labels its whole cone 43 / 44 (thigh) down to the hem; the rule sees shin heights and
  gives shin labels, so the lower skirt follows the shins (Rune plateskirt 1093: 51% of vertices on the right
  bone, about a third of the figure's pixels differ from the real item in every frame; no tear, but the hem
  kicks with the lower leg). A robe bottom does follow the shins below the knee, and the rule gets 93% of the
  Mystic robe bottom right. Decide which kind yours is; for the plateskirt kind relabel everything below the
  hips 43 / 44 by hand, keeping two knee rings on the leg stubs.
- **Templates only help when the shape is the same.** Copying labels from a differently shaped item of the
  same family moved the joint pivots 6 to 9 units and made the motion worse than the rule alone in six of
  twelve tests. The tool therefore measures how much of our mesh the template covers and, under 75%, takes from
  it only family sub-labels and vertices the rule was unsure of. Dragon boots from Rune boots (89% covered) came
  out identical to the real item in every frame.
- **Pivots.** Real items of one slot put the same pivot 3 to 5 units apart (knees most). A piece labelled by rule
  therefore moves a few units differently from any particular real item even where every vertex follows the
  right bone. That is a shift of a whole limb, not a tear.
- **Anything below the waist on a body piece** gets chest 8 (it hangs rigidly); real long tops use 30, 29, 41,
  43 / 44 there.

## Checklist for an armour author

1. Build in worn space; right side at negative x, front at negative z.
2. Put the seam rings at the exact shared coordinates: collar (+-7, 164, -8), (+-7, 165, 4); wrist
   (-+32 and -+25, 114, -2 and 4); back waist (+-13, 114, 8); ankle (-+12 and -+6, 18, -4), (same, 19, 6).
3. Give every joint the piece covers its ring: elbow ring at 130 to 132, two knee rings at 58 to 61 and 46 to 48,
   a toe hinge, a hip ring at 110. Do not let a joint fall in the middle of a long face.
4. Keep the chest inside |x| < 19 and start the arm outside it; decide on purpose whether a shoulder piece swings
   with the arm (21 / 25) or stays on the chest (59 / 60) and label it by hand if the tool flags it.
5. Run `label_armour.py`. Add `--template ID` when the piece was built on a real item's shape (it helps only
   then; the tool says how much of the mesh the template covers) and always for a cape. Read the pivot lines
   and the unsure list; fix by hand what the rule cannot know (skirt kind, chest-pinned shoulder pads, hood
   collars).
6. Boots: include the root triangle.

## What to check in pose.py before calling a piece done

```
pose.py OUT.png "none,model:PIECE.bin@body+arms,item:TEMPLATE" --seq 824 --frames 0,1,2,3,4,5,6,7 --overlap 1:2
```
- Run 824 (largest limb travel), walk 819, slash 390, stab 386; all frames, and a second view with `--yaw 1216`.
- No spike or stretched face at shoulder, elbow, knee, ankle (one vertex following the wrong bone).
- No gap opening at the wrist, waist, ankle or collar against the neighbouring kit or piece (seam label or
  coordinate wrong).
- The piece does not stay behind or float (a pivot label missing: the tool prints WARNING for it).
- Shoulder pieces do what you meant in slash frames 5 to 8, where the right arm is highest.
- Against a real item of the family in the next column the limbs sit within a few pixels.
- A helm or boots: add `--aim head` (or `feet`, `hands`, `legs`, `body`, or a height) with `--zoom 130 --cell 400x400`
  for a close view; `--tone A:B` (with a `none` column) gives the tone difference between two columns in grey levels.

## The body kits as numbers: `body-kits.json`

`body-kits.json` in this folder holds the bare figure the harness draws (the `none` entry of pose.py and figure.py:
the standard male figure), so nothing has to be read off a picture. For each body kit slot, `slots.hair`, `jaw`,
`body` (torso), `arms`, `hands`, `legs`, `feet`:

- `slot`, `kit_id`, `model_ids` (one cache model each: 230, 249, 28515, 26632, 176, 28285, 181), counts, bounds and
  histograms of labels and priorities;
- `vertices` in worn space as `[x, height, z]` (height up, feet 0; right side negative x; front negative z) and
  `vertex_labels`, one per vertex;
- `faces` (vertex indices, winding as stored), with `face_labels`, `face_priorities`, `face_colours` (16-bit
  values as the kit stores them, before the figure's skin and clothes recolours), `face_colours_hsl`, and
  `face_colours_drawn` (what this figure shows after its recolours, listed under `figure.player_recolours`).

Uses: the head's outline under a helm (hair 68 vertices, crown at 196, jaw 15 vertices), the exact seam rings,
the kit's priorities (hair and jaw 6, torso 3 with its abdomen stub 2, arms and hands 10, legs 1, feet 0), the
skin colour 4550 for a part that should take the player's skin. To write a model from it: cache vertex =
`[x, -height, z]`, everything else as listed. Verified by drawing: each of the seven slots rebuilt from the JSON
alone, re-encoded and worn as `model:PATH@SLOT`, differs from the bare figure by 0 pixels in the bind pose and
all eight walk frames, front and back views. `asset_pose.exe CACHE info kits` prints which kits the figure uses.
