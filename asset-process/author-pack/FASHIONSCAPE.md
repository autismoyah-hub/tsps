# What makes worn gear desirable to players (a hypothesis under test)

Status 2026-11-27: NOT CONFIRMED. The owner judged gear built to this study: all three helms rejected again ([quoted words withheld]; the test: No), and the whole set built to it lost to the earlier set ([quoted words withheld]). The measurements of real helms below stand; the design rules drawn from them do not have the owner's eye behind them. Do not design to this file until it is revised. Earlier status, 2026-11-26: written by a research author from measurements of 23 coveted and 18 plain real helms and our four rejected ones. The owner has NOT yet judged gear built to it. Required reading for any worn piece ordered as PLAYER gear; not required for NPC or guard gear. Measuring tools: `{ROOT}/builds/meridian-helm-v3/study/` (`measure.py`, `crease.py`) and `check.sh` beside it.

---

# What makes a worn helm desirable to players: a study, and rules to design and review by

Private to this machine. Written before the meridian-helm-v3 helms were designed. Every number here is reproducible:
`study/outfits.py` (the two groups), `study/measure.py` -> `study/measures.json`, `study/crease.py` -> `study/crease.json`.
Pictures: `{ROOT}/render/meridian-helm-v3-study/` (`C1..C4-close.png`, `P1..P4-close.png`, `O1-close.png`:
four views close; `C-game-all.png`, `P-game-all.png`: whole outfits at game size, three angles, heads enlarged).

## 0. What is knowledge, what is measurement, what is taste

- **Knowledge (not measured, mine, about the game's players):** which helms go in which group. Players wear a helm
  for three reasons that are hard to pull apart: it is the best in its slot (Torva, Masori, Ancestral, Neitiznot
  faceguard, Serpentine, Justiciar, Slayer helmet), it is rare or costly (3rd age, gilded, Statius), or it finishes a
  look (Armadyl, Dharok, Verac, Guthan, Berserker helm, Helm of neitiznot, Crystal). "Fashionscape" is the players'
  word for dressing for the look alone; its habits are: whole outfits in ONE palette, pieces of different sets joined
  by colour, a recognisable silhouette at the bank or the Grand Exchange, and showing what you have earned.
- **Measurement:** everything in the tables, from the cache meshes and from drawings of the standard figure.
- **Taste:** the reading of a helm as [quoted words withheld] (rule 1) and of its "gaze" (rule 4). I classed those by eye and
  give my classing per item so it can be disputed.

Every item id was found by NAME in the archive and its name confirmed in the cache with `asset_pose info`.

**Group C, coveted (23):** Armadyl helmet 11826, Torva full helm 26382, Neitiznot faceguard 24271, Helm of neitiznot
10828, Serpentine helm 12931, Justiciar faceguard 22326, Inquisitor's great helm 24419, Masori mask (f) 27235,
Ancestral hat 21018, Virtus mask 26241, Dragon full helm 11335, 3rd age full helmet 10350, Dharok's helm 4716, Verac's
helm 4753, Guthan's helm 4724, Slayer helmet (i) 11865, Oathplate helm 30750, Blood moon helm 29028, Statius's full
helm 22625, Crystal helm 23971, Obsidian helmet 21298, Sunfire fanatic helm 28933, Berserker helm 3751. Each drawn
with its own body and legs, or its usual partner (Bandos for the faceguard, serpentine and slayer helms; Fighter
torso for the two Fremennik helms).

**Group P, plain (18):** Bronze, Iron, Steel, Mithril, Adamant, Rune full helm 1155 1153 1157 1159 1161 1163; Iron,
Steel, Rune med helm 1137 1141 1147; Initiate sallet 5574, Proselyte sallet 9672, White full helm 6623, Decorative
helm 4071, Coif 1169, Rock-shell helm 6128, Granite helm 10589, Warrior helm 3753, Torag's helm 4745.

**Set aside, group X (3), because they argue against a simple story:** Gilded full helm 3486 (the Rune full helm's
mesh in gold: coveted for colour and rarity alone); Shayzien helm (5) 13379 and Spiny helmet 4551 (horns, a skull
face, spikes: every shape feature of group C, and nobody covets them). See section 5.

**Ours, group O (4, all rejected by the owner):** v1 open sallet with hoop, v2-a closed helm, v2-b barbute, v2-c
crested sallet.

## 1. The numbers

Geometry is orthographic from the mesh in whole game units against the bare head (hair + jaw kits: x +-11, crown 196,
face at z -20, back of skull z 4). `ex_f` = silhouette area outside the bare head's, as a share of the bare head's
area, front view. `sol_f` = silhouette area / area of its convex hull, front view (1 = a blob, lower = notched by
horns, wings, spikes). `peaks` = outline points of at least 3 units prominence on five outline profiles, less the
five a plain egg has. `fore` / `wide` / `top` = units beyond the bare head forward / sideways / upward. `crease` =
share of the mesh's interior edge length where the two faces meet at more than 30 degrees. `shared` = share of the
helm's lit pixels whose material (hue and saturation) also covers 3 % or more of the body and legs it is worn with.

| | C coveted (23) | P plain (18) | O ours, rejected (4) |
|---|---|---|---|
| ex_f (extra silhouette, front) | 0.17 .. 1.91, median 0.44 | 0.16 .. 0.41 (one 1.33), median 0.19 | 0.66 .. 0.96 |
| sol_f (solidity, front) | 0.55 .. 0.96, median 0.80 | median 0.83 (0.98 for med helms) | 0.91 .. 0.97 |
| **bulk test: sol_f <= 0.86 OR ex_f <= 0.46** | **23 of 23** | **18 of 18** (X: 3 of 3) | **0 of 4** |
| peaks | median 3 (quartiles 1 .. 4.5) | median 1 (0 .. 1) | 0, 0, 1, 0 |
| **signature test: peaks >= 2 OR fore >= 8 OR wide >= 6** | **20 of 23** | **2 of 18** | **0 of 4** |
| wide (half-width beyond the head) | median 4 | median 0.5 | 3 .. 4 |
| crease (share of edges over 30 degrees) | 0.29 .. 0.58, median 0.42 | 0.29 .. 0.52, median 0.47 | 0.10 .. 0.22 |
| **facet test: crease >= 0.29** | **23 of 23** | **18 of 18** | **0 of 4** |
| median fold angle between faces | 15.9 .. 39.5 | 8.1 (med helm) .. 30.8 | 7.6 .. 12.6 |
| faces | 94 .. 434, median 288 | 88 .. 371, median 157 | 382 .. 483 |
| **shared >= 0.80 (helm wears the body's materials)** | **17 of 23**, median 0.90 | **3 of 18**, median 0.74 | 4 of 4 |
| accent (smallest material of 1.5 to 15 % of the helm) | 2 % .. 11 %, median 7 % | a plume of 24 .. 41 % in a colour the body lacks | 6 .. 10 % |
| materials with 4 % or more of the helm | 1 .. 6, median 3 | 2 .. 3 | 3 |
| helm mean grey minus body mean grey | -37 .. +46, median +10 | -35 .. +42, median +3 | +8 .. +11 |
| skin shown (share of the bare face's skin pixels) | 0 for 13; 0.09 .. 0.64 for 10 | 0 for 9; 0.27 .. 0.78 for 9 | 0, 0.19, 0.26, 0.80 |
| mirror-symmetric vertices | 1.0 for 20; all are symmetric designs | 1.0 | 1.0 |

Two different things fall out of this table, and they must not be confused:

- **What separates coveted from plain** is the signature test (20 of 23 against 2 of 18) and the shared palette (17 of
  23 against 3 of 18). Face count follows (288 against 157) because signatures cost faces.
- **What separates OUR rejected helms from every real helm, coveted or plain,** is the bulk test (0 of 4 against 44 of
  44) and the facet test (0 of 4 against 44 of 44). Ours are the only helms measured that are both large and smooth:
  a balloon. That, I believe, is the word "goofy" in numbers. It is a fault of authenticity that we took for a fault
  of design; fixing it alone would give a helm fit for a guard, which is what the owner said of the barbute.

What did NOT separate the groups, so there is no rule about it: value contrast between helm and body (both groups
span about -35 to +45), the number of materials, how much face is shown (half of each group shows none), height
above the crown alone (the plain full helm's plume rises 14), solidity alone, face count alone.

## 2. Rules (design to these; a reviewer checks each line)

**R1. One idea you can name in two words, and it is a character, not a container.** (taste, classed by eye)
Hawk beak (Armadyl), demon knight (Torva), winged viking (Neitiznot, both), serpent head (Serpentine), crowned judge
(Justiciar), iron mask (Inquisitor), jackal king (Masori), wizard's hat (Ancestral), death mask (Virtus), dragon
head (Dragon), plumed hoplite (3rd age), spiked executioner (Dharok), flail cage (Verac), bull horns (Guthan), sun
face (Sunfire), jaguar mask (Blood moon), crystal blades (Crystal). The plain group names only its type: full helm,
med helm, coif. Check: black out the helm against the background at game size; a player should still be able to
name it. If the honest name is the armourer's type (sallet, barbute, armet, kettle hat), it is a guard's helm.

**R2. The signature is in the OUTLINE, pointed and rigid, and there is exactly one.** (measured)
Pass if `peaks >= 2` or `fore >= 8` or `wide >= 6`. Shown by: horns (Guthan wide 15, Berserker 17, Torva peaks 3),
wings (Neitiznot top 21, faceguard), a beak (Armadyl fore 14), a crown of points (Justiciar peaks 5, Inquisitor),
blades (Crystal peaks 4, Oathplate peaks 6), a spike (Dharok, Verac), rays (Sunfire wide 15), a tall cone (Ancestral
top 41). The three coveted helms that fail this (Dharok by 0 peaks but a 12-unit spike, Statius, Obsidian) are
compact angular masks with a made face (R4); none is a dome. The signature reads from the front AND from the back or
side: by eye, most of the 20 still show it in the back view (`C*-close.png`, bottom row); I did not measure this.
Things the coveted helms do NOT use for it: a soft plume (only the plain full helm has one), a wire (nothing in
either group is thinner than about 2 units across its width), a rounded comb.

**R3. Bulk test: either hug the head or break the outline; never a big smooth shape.** (measured, 44 of 44 real)
`sol_f <= 0.86` OR `ex_f <= 0.46`. A helm that adds more than 46 % to the head's front silhouette must have a notched
outline (Masori 1.12 / 0.85, Oathplate 0.85 / 0.74, Dragon 0.91 / 0.66, Sunfire 0.90 / 0.55). A helm with a smooth
outline must stay close to the skull (3rd age 0.31 / 0.96, Obsidian 0.35 / 0.95, Statius 0.28 / 0.92, Crystal
0.24 / 0.92, Justiciar 0.46 / 0.90). Shell width at the brow: coveted closed helms 20 to 30 units, but the ones at 28
to 30 (Torva, faceguard, Oathplate) carry horns or fins there; a plain wall 28 to 30 wide is ours alone.

**R4. The helm has a gaze.** (taste, classed by eye; 20 of 23)
Either a made face with slanted eye shapes under a V brow (11: Torva, Justiciar, Inquisitor, Masori, Virtus, Dragon,
Oathplate, Obsidian, Serpentine, Sunfire, Blood moon), or the wearer's own eyes in a deep hard frame (9: Armadyl,
both Neitiznot helms' kin Berserker and Helm of neitiznot, 3rd age, Dharok, Statius, Crystal, Slayer, Ancestral
under its brim). Only three are anonymous (Neitiznot faceguard's level visor, Verac's bars, Guthan's slot) and each
of those has the strongest outline of its kind (wings, flail spike, horns 15 wide). The plain group: vertical bars
with no eyes (full helm) or a whole unframed face (med helm, coif). Nothing in either group hides the eyes and shows
the mouth. Check: in the front view at game size, can you tell where it is looking and in what mood?

**R5. Hard planes.** (measured, 44 of 44 real)
`crease >= 0.29` and a median fold angle of 16 degrees or more. Real helms are cut from a few large planes that meet
at folds of 20 to 40 degrees; the smooth shading then draws each plane as one soft tone and each fold as a line.
An evenly rounded loft (ours: crease 0.10 to 0.22, median fold 8 to 13 degrees) shades as a ball. Face budget: 180
to 330 (the coveted quartiles), of which 8 to 18 % beyond the head's box (the signature).

**R6. The helm wears the body's materials and adds none; one small accent, repeated on the body.** (measured)
`shared >= 0.80` (17 of 23 coveted; 3 of 18 plain). Accent 2 to 11 % of the helm's lit pixels (median 7 %), and the
same accent appears on the body: Torva's violet eyes and chest gem, Justiciar's gold star on brow and breast,
Armadyl's gold crest and gold pauldron edges, Obsidian's red eyes and red plate edges, Virtus's violet. The plain
full helm does the opposite: a quarter to four tenths of it is a plume in a colour its body does not have.
(The six coveted helms under 0.80 are worn with other sets' bodies, or are masks.)

**R7. The helm's signature shape is repeated at the shoulders.** (by eye, own-set outfits only: 13 of 17)
Armadyl gold points on helm and pauldrons; Torva and Oathplate spikes on both; Dharok's spike and spiked shoulder;
Justiciar's crown points and pointed pauldrons; Masori's gold fins and gold collar wings; Crystal's blades on helm,
shoulders and elbows; Virtus, Ancestral, Dragon, Sunfire, Inquisitor, Blood moon likewise. Not: 3rd age, Statius,
Verac, Guthan. This is a rule about the SET: a helm cannot satisfy it alone (see the report on the cuirass).

**R8. Lines converge; nothing rings the skull level.** (by eye)
On the coveted helms the drawn lines (trim, ridges, the edges of the sights) run up the centre or slant toward it:
a V brow, a keel, a crest line, a pointed chin. Level bands round the skull belong to the plain group (the med
helm's red band, the decorative helm's band). Count on ours: v1 a blue band and a brass brow loop; v2-a a brass brow
band, a level sight and a level lap line.

**R9. Symmetric, and restrained.** Every helm in both groups is mirror-symmetric. Median three materials; the
coveted helms with more (Masori 6, Serpentine 6) are the busiest to my eye (taste). One motif.

### Checklist

| # | check | pass |
|---|---|---|
| 1 | named idea, two words, a character not an armourer's type | named by a reader from the silhouette |
| 2 | signature in the outline | peaks >= 2 or fore >= 8 or wide >= 6; visible from the back or side as well as the front |
| 3 | bulk | sol_f <= 0.86 or ex_f <= 0.46 |
| 4 | gaze | slanted made eyes under a V brow, or the wearer's eyes in a deep frame; never mouth without eyes |
| 5 | hard planes | crease >= 0.29; median fold >= 16 degrees; 180 to 330 faces |
| 6 | palette | shared >= 0.80; one accent of 2 to 11 %; the accent or the signature's material is on the body too |
| 7 | echo | the signature's shape recurs on the body piece (a set rule) |
| 8 | lines | no level band round the skull; lines converge on the centre |
| 9 | symmetry, restraint | symmetric; at most 3 materials of 4 % or more plus the accent; one motif |

`study/measure.py OUT.json key=PATH@SLOTSPEC --new` and `study/crease.py key=PATH` print lines 2, 3, 5, 6 for a new helm.

Note on line 5, added after building: the diagonal of a truly flat quad counts as an unfolded edge, so a helm made
of large flat planes scores lower than it looks. `crease.py` therefore also prints `crease_vis`, the same share
among edges that fold 3 degrees or more: real helms 0.34 to 0.66, our rejected four 0.17 to 0.33. The threshold of
line 5 was NOT changed; `crease_vis >= 0.34` is offered as a second reading, and both are reported for the new helms.

## 3. Why each of our four failed, by these findings

- **v1, open sallet with hoop.** Bulk: ex_f 0.96 at sol_f 0.93 (the largest smooth helm measured). Signature: peaks
  0; the hoop is a wire, which nothing real uses. Gaze: 80 % of the face bare and unframed, the med helm's look.
  Lines: a level blue band and a brass loop round the face. Planes: crease 0.22. Its honest name is [quoted words withheld].
- **v2-a, closed helm.** Bulk 0.66 / 0.94. Planes: crease 0.10, median fold 7.6 degrees, the smoothest helm of the 48
  measured. Gaze: a level letterbox sight. Lines: three level rings. Signature none (peaks 0, fore 7). Name: "closed
  helm", an armourer's type. It passes as real and it is a guard's.
- **v2-b, barbute.** Bulk 0.69 / 0.97 (the most solid outline of the 48). Planes 0.20. Signature none. Gaze: the
  face is in a slot too dark to read. Name: "barbute". The owner's [quoted words withheld] is exactly rule 1.
- **v2-c, crested sallet.** Bulk 0.78 / 0.91; side silhouette 1.24. Its crest is a rounded comb: peaks 0, so the
  outline gained area and no point. Gaze: it hides the eyes and shows the mouth, which no helm in either group does.
  Planes 0.18.

All four pass the palette rule (shared 1.00, accent 6 to 10 %): the house palette was never the fault.

## 4. What I am unsure of

- The groups are my knowledge of the players, and the coveted group is confounded with power: nearly every helm in
  it is or was best in slot or very rare. I cannot separate [quoted words withheld] from [quoted words withheld] with a cache.
- R1 and R4 are classed by my eye. Another reader might put Statius or the slayer helm elsewhere.
- The thresholds are fitted to 44 helms and then found to reject our 4: with four counter-examples a threshold can
  always be found. The bulk and facet tests are safe as descriptions of the game's art; as causes of the owner's
  "goofy" they are a hypothesis.
- Everything is on the male standard figure, standing.

## 5. What would falsify this

- **The owner prefers one of the rejected four to a helm that passes all nine lines.** Then the checklist measures
  the wrong thing.
- **A helm passes lines 1 to 9 and the owner still says "guard".** Then the missing ingredient is status (rarity,
  power, where it drops), which no mesh can carry. The X group already leans this way: the Shayzien (5) helm has
  horns, a skull face, peaks 3, crease 0.46 and is worn by nobody for looks; the Spiny helmet has peaks 5; and the
  Gilded full helm is the plain mesh, signature test failed, and is coveted. Shape is at most necessary.
- **A plain-looking helm becomes coveted on this server once it is best in slot.** Then looks follow power, and the
  cheapest way to a desirable helm is its stats.
- **The bulk test is wrong if** a real helm outside my 44 is both large and smooth (sol_f > 0.86 and ex_f > 0.46)
  and worn for looks. I looked for one among the helms I know and did not find it; the full list of 469 head
  pieces was not measured.
- **The echo rule is wrong if** the owner accepts one of the new helms on the present cuirass, whose shoulders are
  round and carry no point.
