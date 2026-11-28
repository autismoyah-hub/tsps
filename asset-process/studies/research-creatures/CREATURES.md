# Creatures: what the real ones measure, rules to build to, and the rebuild test

Private to this machine (cache-derived). Research of 2026-11-27. Everything measured here was run; what was not run
is listed at the end. Folder: `{ROOT}/studies/research-creatures/`. `P` =
`{PYTHON} -I`, run in this folder. Lighting has its own file: `LIGHTING.md`.

## 1. How the creatures were dated

- **Measured:** the model ids of each definition. Ids are handed out in order of creation.
- **Knowledge (mine, not the cache's):** each creature's release year. A creature is in a group only if both agree.
- Calibration, name -> model id (measured) -> release (known): TzTok-Jad 9319 (Oct 2005); Dagannoth Rex 9941 (2005);
  dragons 17407.., demons 17375.., giant rats 18888.., spiders 20309.., cows 23889.., goblins 24433.., wolves and
  hellhounds 26252..26266 (the 2006-07 remakes); General Graardor 27660 (Aug 2007, the end of the 2007 backup);
  Kraken 28231 (2014); Cerberus 29270 (2015); Vorkath 35023 (Jan 2018); Alchemical Hydra 36182.. (Jan 2019);
  Sarachnis 37292 (Jul 2019); The Nightmare 39196 (2020); Nex 43209 (2022); Vardorvis 49298 (2023); Scurrius 50144
  (2024); Araxxor 54289 (2024); Doom of Mokhaiotl 56469 (2025).
- **Three traps found.** (1) The file format dates nothing: every model in this cache is stored in the newest format.
  (2) Later models were put into old gaps (Zulrah 14407-9, Corporeal Beast 11056, Yama 10468): a low id alone is not
  age. (3) A high id is not always new art: Dire Wolf 37364 is the 2007 wolf re-exported (904 faces, same labels);
  Strongbones 43830 is the 2007 goblin kit merged into one model; Ice giant 54475 is a special flat, translucent
  mesh. Both were removed as rebuild targets for that reason.
- **The classic creatures named in the task are not "early" meshes in this cache.** Wolf, Hellhound, Cow, Goblin,
  Giant spider, the dragons and the demons are all 2006-07 remakes (ids 17,000 to 26,300; Wolf 892 faces, Goblin
  1,910). The 2004-05 look survives in other creatures (Unicorn 8986 = model 2933, the old Goblin 677 = 2951..,
  the Slayer monsters 5062-5086, the Fight Cave). So there are THREE groups, not two:

| group | rule | count | examples |
|---|---|---|---|
| E1 early | every model id under 12,500; a creature I know from 2004-05 | 28 | Unicorn, Ugthanki, Basilisk, Kurask, Crocodile, Bloodveld, old Goblin, Ogre, Yeti, Mountain troll, Abyssal demon, Gargoyle, Rock Crab, Dagannoth, TzTok-Jad, Dagannoth Rex |
| E2 late classic | ids 13,000 to 27,700; 2006 to Aug 2007 | 30 | Wolf, Hellhound (two), Cow, Giant rat, Black bear, Graahk, Kyatt, Terror dog, Giant and Huge spider, Scorpion, Lesser, Greater and Black demon, Goblin, Hill and Moss giant, Green dragon, King Black Dragon, Kalphite Queen, General Graardor |
| R recent | every id 36,000 or more; released 2019 or later | 53 | 16 quadrupeds (Lynx, Jaguar, Sourhog, Drake, Bull, Callisto, Skeleton Hellhound ..), 6 spiders (Sarachnis, Araxyte, Venenatis ..), 15 humanoid monsters (Basilisk Knight, Husk, Minotaur, Tormented Demon ..), 16 bosses (Hydra, Nightmare, Nex, Vardorvis, Scurrius, Sol Heredit, Araxxor ..) |

`P era.py list` prints every creature with its name, model ids and faces as the cache has them (the verification).
`P era.py measure` (90 seconds), `P era.py table [--plan q]`, `P era.py verdicts`. Data: `data/era.json`,
`data/era-table.txt`, `data/era-table-quadrupeds.txt`, `data/era-verdicts.txt`. The measures are in `cmesh.py`.

Contact sheets (each creature close above, at game size below; harness = the client's lighting and painting), in
`{ROOT}/render/research-creatures-era/`: `era-E1.png`, `era-E1-bosses.png`, `era-E2.png`,
`era-E2-bosses.png`, `era-R-quadrupeds.png`, `era-R-spiders.png`, `era-R-humanoids.png`, `era-R-bosses.png`, and ours
among like creatures: `like-quadrupeds.png`, `like-wolf-skeleton.png`, `like-spiders.png`, `like-bosses.png`.

## 2. What the measurements say

Median (10th to 90th centile). "Sep" is the chance that a random recent creature exceeds a random E1 / E2 creature
(0.5 = no difference, 1 or 0 = complete separation).

| measure | E1 2004-05 | E2 2006-07 | R 2019+ | sep E1:R | sep E2:R |
|---|---|---|---|---|---|
| faces | 502 (462-707) | 1,045 (530-2,001) | 1,607 (888-3,684) | 0.99 | 0.70 |
| vertices | 279 | 524 | 859 | 0.98 | 0.71 |
| median face, ten-thousandths of the surface | 13.2 (8.0-15.2) | 5.0 (2.8-12.1) | 4.3 (1.1-8.3) | 0.04 | 0.35 |
| colours | 8 (4-13) | 13 (8-27) | 22 (11-45) | 0.93 | 0.73 |
| colours covering 90% of the surface | 4 (2-6) | 6 (3-11) | 9 (4-18) | 0.88 | 0.69 |
| tones per hue family | 2.8 (1.0-6.6) | 4.6 (2.2-7.0) | 5.0 (2.9-11.0) | 0.75 | 0.61 |
| lightness span of the main hue | 22 | 25 | 33 (6-76) | 0.62 | 0.57 |
| median lightness step across a tone edge | 4 | 5 (4-9) | 6 (4-16) | 0.61 | 0.60 |
| largest one-colour patch, share of surface | 0.42 (0.15-0.83) | 0.17 (0.09-0.31) | 0.17 (0.06-0.45) | 0.24 | 0.50 |
| in-surface edges that are paint boundaries | 0.087 | 0.139 | 0.168 (0.107-0.235) | 0.82 | 0.63 |
| small contrasting islands inlaid in the skin | 6 (2-26) | 23 (10-106) | 49 (16-171) | 0.91 | 0.68 |
| surface in such islands | 0.040 | 0.061 | 0.111 (0.027-0.232) | 0.83 | 0.64 |
| surface on thin parts (limbs, horns, tails) | 0.28 | 0.48 | 0.47 (0.31-0.90) | 0.81 | 0.51 |
| median fold between neighbouring faces | 25.7 deg | 22.8 | 20.0 (12.3-28.4) | 0.24 | 0.32 |
| tone follows facing (correlation lightness ~ up) | 0.00 (-0.06-0.34) | 0.15 (-0.17-0.42) | 0.10 (-0.32-0.43) | 0.56 | 0.48 |
| tone by part (lightness variance explained by label) | 0.74 | 0.41 (0.17-0.82) | 0.48 (0.27-0.72) | 0.36 | 0.52 |
| vertices with exactly six faces | 0.30 (0.23-0.45) | 0.34 (0.28-0.40) | 0.32 (0.27-0.39) | 0.54 | 0.42 |
| spread of faces per vertex | 1.73 (1.43-2.31) | 1.56 (1.36-1.84) | 1.57 (1.31-2.03) | 0.39 | 0.49 |
| outline length / hull length (three views) | 1.49 | 1.63 | 1.58 (1.34-2.01) | 0.61 | 0.51 |
| vertex labels | 40 (30-62) | 74 (51-140) | 62 (0-168) | 0.63 | 0.41 |
| faces spanning two or more labels | 0.57 | 0.66 | 0.40 (0-0.74) | 0.28 | 0.23 |

Counts: flat-shaded faces at all: E1 1 of 28, E2 4 of 30, R 10 of 53 (over a tenth of the faces: 1, 0, 5: ice, crystal,
bone). Translucent faces: 3, 13, 31. Depth offsets: 6, 8, 13 (always a handful of faces: median share 0.000). Textures:
none in any group. Split vertices: 0, 0, 2 creatures. Definition ambient or contrast set: 5, 3, 29 of 53. Bone
weights instead of labels: 0, 0, 15 of 53. Undrawn faces (alpha 255) whose labels exist nowhere else: 1, 0, 15 of 53.

**What recent creatures do that early ones do not, plainly:**
1. **Three times the faces, spent on smaller faces everywhere** (502 -> 1,607; the median face is a third of the size
   relative to the body). Not concentrated: the share of faces that carry half the surface is the same (0.20 / 0.18).
2. **Nearly three times the colours and twice the tones per hue** (8 -> 22 colours; 4 -> 9 to cover 90%; 2.8 -> 5.0
   tones per hue). The big flat one-colour body is gone: the largest one-colour patch falls from 42% to 17%.
3. **Paint inside the surface.** Twice the share of in-surface edges are paint boundaries (0.087 -> 0.168), and eight
   times the small contrasting islands inlaid in the skin (6 -> 49): eyes, teeth, claws, scars, spots, stripes,
   straps. They are INLAID (the skin is split and the faces repainted, sharing vertices): depth offsets are not how
   it is done (median 0 offset islands in every group), and separate small shells are the minority (median 4).
4. **More of the body is limb** (surface on thin parts 0.28 -> 0.47) and the surface turns more gently between
   faces (median fold 25.7 -> 20.0 degrees): more, smaller faces round the forms. "Faceted" in the recent look is
   paint and outline, not sharper folds.
5. **Lighting values and translucency.** 29 of 53 recent definitions set ambient or contrast (commonly ambient 10 to
   25 with contrast 20 to 30); 5 of 28 and 3 of 30 early ones do. 31 of 53 carry some translucent faces.
6. **Pivot markers.** 15 of 53 recent creatures (15 of the 38 that move by labels) carry undrawn faces whose labels
   appear nowhere on the visible skin: Lynx 5 such labels, Minotaur 28, The Nightmare 37, Nex 30. The joint's pivot
   is set by an invisible triangle, not by a ring of the skin. 1 of 58 older creatures does this.
7. **Bone weights** in 15 of 53. We cannot write them; a rebuild target must move by labels.

**What does NOT separate the eras** (so it is not what "dated" means): welded skin (all), smooth shading (all),
priorities, textures, depth offsets, outline raggedness (0.61 / 0.51), tone following the facing (none do), hand-split
irregularity (all do), face aspect, mirror symmetry.

**The 2006-07 remakes are close to the recent creatures on every measure of style**; the best separations are faces
(0.70), vertices (0.71), colours (0.73), contrast and ambient set (0.72 / 0.70). The owner's "dated" is therefore most
likely the E1 look (about 500 faces, 4 colours over 90% of the body, one flat-coloured hide), not the wolf's.

## 3. Our four creatures, scored

| measure | real band used | hound 2 | spider | boss 1 | boss 2 |
|---|---|---|---|---|---|
| faces | R quadrupeds 888-2,041 (E1 quadrupeds 452-535, E2 519-1,132) | **674** | **764** | 2,266 | 2,594 |
| colours covering 90% of surface | R 4-18 (quadrupeds: E1 4, E2 5, R 6) | **3** | 7 | 18 | **32** |
| colours | R 11-45 | 12 | 12 | 37 | **85** |
| tones per hue family | R 2.9-11.0 | 4.0 | 3.4 | 10.6 | **16.8** |
| median face, ten-thousandths of the surface | R at most 8.3 (quadrupeds 8.9) | **8.9** | 8.4 | 3.1 | 2.6 |
| in-surface edges that are paint boundaries | R at least 0.107 | 0.190 | **0.096** | 0.152 | 0.206 |
| islands inlaid in the skin | R at least 16 | 34 | **2** | 34 | 23 |
| median fold, degrees | R at most 28.4 | 26.3 | **30.8** | 14.6 | 17.1 |
| edge length folded over 30 degrees | R quadrupeds 0.31-0.45 | **0.46** | **0.50** | 0.33 | 0.36 |
| tone follows facing | every era at most 0.45 | 0.42 | **0.65** | 0.17 | 0.07 |
| tone by part | every era at least 0.15 | **0.11** | **0.01** | 0.47 | 0.33 |
| median step across a tone edge | every era at most 16 | **18** | 10 | 11 | 8 |
| vertices with exactly six faces | every era at most 0.45 | **0.49** | **0.59** | **0.72** | **0.67** |
| spread of faces per vertex | every era at least 1.30 | **1.24** | **1.04** | **0.79** | **0.89** |
| largest shell, share of faces | R 0.48-1.00 | 0.98 | 0.82 | **0.13** | **0.12** |
| style gate: GAME limits broken / RECENT band broken | real R: median 0 / 0 | 4 / 3 | 4 / 5 | 2 / 0 | 2 / 0 |

Bold = outside the band. `P style_npc.py def:PATH.json` prints the list; `P style_npc.py --calibrate` shows that
30 of 53 recent creatures break no GAME limit and 41 of 53 break at most one RECENT band (E1: 2 of 28).

**Exactly which measures put the hound in the dated group** (`data/era-verdicts.txt`): of the 20 measures that
separate E1 from R at 0.72 or better, the hound is nearest the E1 median on five: faces (674: E1 494, E2 964,
R 1,126; below the recent quadrupeds' 10th centile of 888), vertices (343: E1 256, R 582), colours covering 90%
of the surface (3: E1 4, R 6; below R's 10th centile), median fold (26.3 degrees: E1 25.3, R 19.5) and edge length
folded over 30 degrees (0.46, outside R's band), plus translucent faces (none) and faces on middling parts. It is
nearest E2 on four (colours 12, tones per hue 4.0, median face size, surface on thin parts) and nearest R on eight
(the paint measures: islands 34, patches, in-surface boundaries 0.19).
In words: it has a recent creature's amount of paint detail on an early creature's budget and palette. 93% of its
surface is three dark blues (lightness 16, 28, 38: a span of 22 where the Wolf's greys span 85 and the Jaguar's
85), 674 faces where 15 of the 16 recent quadrupeds measured have 827 or more (the Lynx has 592, with 20 colours and 11
tones of one hue), and angular folds.
**And three things no era does**, which are not "dated" but [quoted words withheld]: its tone steps are harsh (median 18
lightness across a tone edge; real 4 to 6); its tones are not laid by part (0.11; real 0.3 to 0.7), they lean toward
the facing (0.42, at the top of every era's range; the first brief's [quoted words withheld] and the furniture rule
[quoted words withheld] are the cause); and its split is a regular loft (49% of vertices have exactly six faces;
real 30 to 34%, 90th centile 45%).
The spider shares all of that and lacks painted detail (2 inlaid islands against a recent spider's 36).
The bosses pass every dated-or-recent measure and fail on construction: grid-regular (0.72 and 0.67), built from 19
and 38 closed blocks (largest 13% of the faces; Graardor's largest shell holds 83% of its vertices), and version 2
has 85 colours and 16.8 tones per hue, beyond any real creature measured (90th centile 45 and 11.0).

## 4. Rules for a creature author (each derived above or in LIGHTING.md; thresholds are what `style_npc.py` checks)

Construction (every era does these; break one and it is unlike any real creature):
1. **One welded skin.** No split vertices (0 in 109 of 111). A joint is the same skin with the label changing along
   it, not two capped parts. Separate shells are small rigid things (claws, horns, plates). An armoured boss may be
   many shells, but its body is still one: largest shell at least half the faces.
2. **Smooth shaded.** Flat faces: none by default; up to a tenth for a deliberate material (ice, crystal, shards).
   Facets come from firm folds between smooth faces. Flat faces do not vote and take two thirds of the light.
3. **Split by hand, irregularly.** Vertices with exactly six faces: at most 0.45 (real 0.30 to 0.34); spread of
   faces per vertex at least 1.30. Do not loft every part from rings of equal count: fan, merge, vary the count from
   ring to ring, let large faces sit beside small ones. A loft is the first thing all four of ours share.
4. **Tone belongs to the part and the marking, never to the facing.** Correlation of lightness with "up" at most
   0.45 (real median 0.0 to 0.15); lightness variance explained by label at least 0.15 (real 0.4 to 0.7). The game
   already darkens the underside to near black (LIGHTING.md rule 3): painting it darker again is the furniture
   habit and it is wrong here. Paint a muzzle, a mane, a saddle, a sock, a belly PATCH with an edge of its own.
5. **Soft steps, many of them.** Median lightness step across a tone edge 4 to 9 (at most 16); a main hue carries 4
   to 9 tones over a span of 25 to 85 lightness. Hue may move one step with tone (Hellhound hue 0 and 1).
6. **Lit once at rest; the light is fixed to the body** (LIGHTING.md). Decide every tone in the rest pose; what
   shows only in motion (mouth, belly in death) must be right as paint. Set ambient and contrast with the mesh: 0/0
   is the classic look with a black belly; ambient 10 to 25 with contrast 20 to 30 is the common recent choice.
7. **Labels: follow the skeleton's real creatures part for part; pivots may be set by undrawn faces.** A face with
   alpha 255 is never drawn, still votes, and its vertices' labels count toward pivots: recent creatures use such
   triangles as pivot markers. Never leave a vertex on label 0 unless the real ones do.

The recent look (the band of the 53 creatures from 2019 on; outside it on the low side reads as dated):
8. **Faces:** at least 888 for anything of a tile or two (quadrupeds 888 to 2,041, median 1,126; spiders 986 to
   3,684; humanoid monsters 698 to 2,013; bosses 1,886 to 4,003). Median face at most 8.3 ten-thousandths of the surface.
9. **Colours:** at least 11, at least 4 to cover 90% of the surface (median 9), 2.9 or more tones per hue. No
   one-colour patch over 45% of the surface (median 17%).
10. **Painted detail inlaid in the skin:** at least 16 small contrasting islands (median 49): eyes, teeth, claws,
    nostrils, scars, spots, stripes, straps. Build them by splitting the skin and repainting faces that share the
    skin's vertices; not with depth offsets, not as floating shells. At least 0.107 of in-surface edge length should
    be a paint boundary.
11. **Rounder forms, longer limbs:** median fold at most 28.4 degrees (median 20); at least 0.31 of the surface on
    thin parts.
12. Upper bounds seen (do not exceed without a reason): 45 colours, 11 tones per hue, 219 islands.

Not rules (measured, no difference found): outline raggedness, priorities (they do nothing on an ordinary NPC),
depth offsets (rare), textures (none).

## 5. The reference sheet (`sheet_npc.py NPC_ID OUTDIR`)

The archive's NPC sheets (`{REFERENCE_ARCHIVE}/npcs/combat-LO-HI/ID.png` and `.json`)
show six lit orthographic views of the rest pose on a 32-unit grid with model ids, counts, recolours, scale, the
definition's animation slots and bounds; the JSON holds the same plus combat level, size and per-variant bounds.
They do NOT hold the palette, the labels, the lighting values, or the attack and death sequences. So a new sheet:
- **paint maps**: six orthographic views, each face in its painted colour, unlit, no edges, 20-unit grid in model units;
- **label maps** (side, top, front): faces coloured by vertex label, softened on purpose so regions read and face
  edges do not, with each label's number at its centroid; the JSON has every label's vertex count, centroid and
  extent, and lists the labels that sit only on undrawn faces;
- **palette** with shares; **numbers**: bounds, counts, definition (scale, ambient, contrast, size), skeleton, sequences;
- the archive's lit sheet is copied beside it.
Why paint, not lit views: the game lights a creature once at rest by a rule the author's tools reproduce exactly, so
a lit picture only adds an inversion exercise; where each tone is PAINTED is the art being rebuilt. Why labels: they
are the skeleton's contract, without which nothing moves. What stays hidden: vertex positions, the triangulation,
where faces are spent, how joints are built, the undrawn faces' shapes. Known leak: on a low-face creature the paint
patches outline many of the real faces (Lynx: 592 faces, 20 colours).

## 6. The rebuild test (`compare_npc.py`, `gate_npc.py`, `lineup_npc.py`, `reencode_npc.py`)

- `P compare_npc.py REAL_ID OURS.json [--out DIR]`: size, counts, palette and colour distance, rest outline overlap in
  body space, label sets and label-centroid gaps, then MOTION: bind plus 6 frames each of stand, walk, attack and
  death, front and rear quarter views, real against ours under the same sequences: outline overlap, share of
  pixels differing, mean grey difference where both are drawn. `--out` keeps the grids and a difference strip.
- `P gate_npc.py rebuild-npc REAL_ID OURS.json [--json F] [--out DIR]`: PASS / BORDERLINE / FAIL (exit 0 / 1 / 2).
- `P lineup_npc.py OUTDIR REAL_ID:OURS.json [more pairs] [--seed N]`: a shuffled animated board (`lineup.gif`), stills
  close and at game size, and `KEY.txt` aside. `REAL_ID:REAL_ID` is a control pair.
- `P reencode_npc.py NPC_ID OUTDIR [--jitter N --tones Q --lightshift K --hueshift K --unlabel A,B ..]`: the ceiling
  (the real creature through our encoder) and seeded degraded copies.

Provisional thresholds (PASS / BORDERLINE): definition equal; bounding box worst face 1.5% / 3% of the largest
dimension (at least 2 / 4 units); faces within 10% / 15%; colours at most real + 1 or half as many again / twice;
colour distance 2.5 / 3.5; rest outline each view 0.93 / 0.90; split vertices 0 / 4; flat share above the real one
0.03 / 0.08; label-centroid gap median 3 / 6 units; standing picture overlap 0.90 / 0.85 and tone 8 / 12 grey levels;
each sequence: mean overlap 0.88 / 0.82, worst cell 0.78 / 0.68, overlap lost against standing 0.05 / 0.10, tone 8 / 12.

Proof (all run; outputs in `proof/`, pictures in `{ROOT}/render/research-creatures-proof/`):
| case | result |
|---|---|
| ceiling: Lynx 11068, Araxyte 11175 (ambient 10, contrast 30, scaled), Ice giant 13796 (2 models, 9 recolours, flat and translucent), Minotaur 12812 (29 undrawn faces) | every one: 0 differing pixels in 50 cells, overlap 1.000, colour distance 0.00, label gap 0.0: PASS |
| Lynx, tones rounded to 8 | PASS: colour distance 0.37, tone 1.2 to 1.9, outline 1.000 |
| Lynx, vertices jittered 2 units, tones rounded to 16, lightness +6 | BORDERLINE: colour distance 2.82, tone 8.5 to 11.4, outline 0.94, motion overlap 0.92 |
| Lynx, jitter 5, tones rounded to 32, hue +3 | FAIL: colour distance 3.72, outline 0.84 to 0.92, tone 16 to 22, motion overlap 0.86 |
| Lynx, one foreleg's labels (25 to 28, 26 vertices) set to the head's | FAIL on motion only: at rest identical; death overlap 0.78 mean, 0.66 worst; overlap lost 0.11 to 0.22 |
The motion marks have one anchor (the ceiling) and one direction check (the degraded copies). They are not yet
calibrated against a rebuild a person could not tell from the real one; reset them after the first three rebuilds.

## 7. Not proven, not run

- No rebuild has been made; the gate's pass marks for motion are guesses between the ceiling and the degraded copies.
- The era groups are hand-picked (111 of about 6,600 distinct model lists); release years are my knowledge.
- "Dated" is the owner's word about one picture; that the E1 measures are what the owner saw is an inference.
  No original has been built to these rules, so nothing shows that meeting them removes the impression.
- Part shares use a thickness proxy (trunk, middling, thin), not head / hands / feet by name.
- Lynx attack 9092 and death 9093 are inferred from the skeleton's sequence list and frame counts, then looked at;
  the server tables do not list them. Jaguar attack likewise.
- Bone-weighted creatures were measured as meshes; their joints and movement were not examined, and they cannot be
  rebuilt with our encoder.
