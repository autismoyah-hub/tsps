# Paint: how the game's best creatures organise per-face colour, and rules to paint to

Private to this machine (cache-derived). Research of 2026-11-26. Folder `{ROOT}/studies/research-paint/`.
`P` = `{PYTHON} -I`, run in this folder. Pictures:
`{ROOT}/render/research-paint-sheets/`. Everything stated as a number was run; what was not is in
section 7. There are no textures in this art: "texturing" is how PAINT (one colour per face) is laid over the skin.

```
P measure_paint.py verify | run | table [--all] | identity     39 real creatures and ours -> data/paint.json, data/table.txt
P calibrate.py                                                 how many check lines each creature breaks
P show_mats.py ENTRY ..   P zones.py ENTRY ..                  a creature's ramps; its eight zones and their marks
P sheets.py contact|swatch|density OUT.png KEY,KEY..           paint-only sheets
P paintplan.py read ENTRY OUT.json [--parts humanoid]          the paint plan a creature follows
P stylecheck.py DEF.json PLAN.json                             the checker (exit 0 / 2)
```

## 1. What was measured, on what

39 real creatures, each id checked against the name in the cache's census (`P measure_paint.py verify`: 0
mismatches): the owner's 23 (Tekton, Vasa, Vespula, Vanguard, Muttadile; Maiden, Bloat, Nylocas Vasilias, Sotetseg,
Xarpus, Verzik 8370 / 8372 / 8374; Akkha, Ba-Ba, Kephri, Zebak, both Wardens; Nex; the Nightmare; the Doom of
Mokhaiotl; the Maggot King), 13 other bosses (Vorkath, Hydra, Cerberus, Duke, Leviathan, Whisperer, Vardorvis, Sol
Heredit, Araxxor, Hueycoatl, Yama, Corporeal Beast, Zulrah) and the three humanoids nearest the watcher (Minotaur,
Tormented Demon, Basilisk Knight). Ours: the two stone watchers, the star hound, the Zodiac ray, the second
Warden-General. The Great Olm (three objects, not an NPC) was left out.

Measured on the drawn, OPAQUE skin of the rest mesh (translucent faces, a tenth to two thirds of the area on bosses
with a ground disc or veil, are left out). Terms (`paintlib.py`):
- **material**: colours near one seed colour in hue and saturation (2 steps; greys together): one painted substance.
  Its colours dark to light are its **ramp**.
- **part**: one of eight zones cut from the face centres alone (the same cut for labelled and bone-weighted
  creatures); a paint plan may name parts by vertex labels instead (head, trunk, arms, legs).
- **patch**: a connected region of one colour. **mark**: a patch up to 1.2% of the skin that contrasts with what
  surrounds it (8 lightness, or another material). Six **motifs**: light / dark / accent (another material), each as
  a line (three times longer than wide) or a spot.

## 2. The first thing found: "consistent" does not mean [quoted words withheld]

I expected real creatures to carry the same paint statistics on head, trunk and limbs, and ours not. That is NOT
what the numbers say.
- Colours two parts have in common (same colours in the same shares; 1 = identical): real 0.24 to 0.59, median
  0.39. The second watcher 0.43, the first 0.40: inside.
- A creature's parts compared with each other against other creatures' parts, on ten per-part statistics
  (`P measure_paint.py identity`; under 1 = its parts resemble each other more than they resemble other creatures):
  real median 0.68 (0.49 to 0.87). Watcher 2: 0.71. Watcher 1: 0.61. Verzik's three forms: 0.84 to 1.05.
- Tone step per part, patch size per part, edge treatment per part: the spread across parts of the watchers sits
  inside the real spread on every one (`data/table.txt`).
A real boss is not uniform: the Minotaur's left leg uses the four darkest tones of its ramp and its arms the five
palest; half its marks are on its head and none on its legs. What real creatures share is HOW paint is built, below.

## 3. Where real creatures agree and ours stand apart (10th to 90th centile of the 39, median in brackets)

| measure | real | watcher 1 (Rework) | watcher 2 (Usable) | star hound (Strong) | Zodiac ray | Warden-General 2 |
|---|---|---|---|---|---|---|
| marks that are ONE triangle | 0.07-0.31 (0.17) | **0.42** | **0.48** | 0.16 | **0.68** | 0.29 |
| skin in one-triangle patches | 0.004-0.051 (0.017) | **0.065** | 0.043 | 0.013 | **0.079** | **0.069** |
| faces in the median mark | 2-5.2 (3) | 2 | 2 | 4 | **1** | 2 |
| mirrored surface painted alike (within 4 lightness) | 0.76-0.995 (0.96) | 0.93 | **0.67** | 0.98 | 0.95 | 0.83 |
| fold length (30 degrees or more) whose two planes differ in paint | 0.30-0.61 (0.49) | **0.30** | 0.32 | **0.21** | **0.30** | 0.48 |
| paint changes at folds / inside flat surfaces | 1.3-3.4 (2.1) | **1.02** | 1.50 | **1.03** | **1.12** | 3.2 |
| share of all marks on the densest of eight zones | 0.24-0.59 (0.36) | **0.23** | **0.21** | 0.31 | **0.18** | **0.22** |
| spread of paint-boundary density over zones | 0.21-0.76 (0.36) | 0.32 | **0.21** | 0.67 | **0.16** | 0.24 |
| second material, share of skin | 0.11-0.37 (0.20) | **0.03** | **0.08** | **0.06** | 0.25 | 0.20 |
| unevenness of ramp steps (sd / mean) | 0.21-0.88 (0.56) | 0.56 | 0.22 | **0.17** | 0.51 | (1.7) |

Bold = outside the real band. In words:
1. **Real marks are built, ours are splinters.** A real mark is three faces at the median and one in six is a
   single triangle; on the watchers nearly half the marks are single triangles, on the ray two thirds. The step 2
   study found the same thing from the other side (`detail/DETAIL.md`): the real Tekton has NO mark under 16 units
   long, and the rebuilds the owner found carry 2.5 to 7 times the real's marks as fragments.
2. **Real paint is mirrored.** Where the body is mirrored, 96% of it is painted alike at the median; asymmetry is a
   mark (a scar, one glowing arm), not a mass. Watcher 2: 67% (one pale breast and shoulder, patches that differ).
3. **Real tone follows form.** Half of all fold length separates two differently painted planes, and paint changes
   twice as often at a fold as inside a flat surface. On our four originals paint changes about as often inside a
   flat surface as at a fold (1.0 to 1.5): tone is laid as mottle over the form, not as planes of it. This is not
   tone by facing (the real correlation of lightness with "up" stays about 0.1): each plane or plate gets ITS tone.
4. **Real detail has a focus.** The densest of eight zones holds a third of all marks at the median (Minotaur 49%
   on the head, Hydra 95%); arms and legs are calm (Minotaur: legs 0 marks). Ours spread marks evenly: 18 to 23%.
5. **A second material is a mass** (a fifth of the skin at the median), placed, not sprinkled: ours 3 to 8%.
6. **Real ramps are not ruler-made.** Steps vary (4, 6, 2, 6, 7 ..), and in half the creatures hue or saturation
   moves along the ramp (main ramp: 3 hues, 2 saturations at the median; Minotaur skin 4.3.49 down to 3.4.20 and
   5.2.12; Doom 0.4.44 to 61.5.11). Watcher 2's three ramps step 6-6-6, 10-10-12 and 7-7. A weak separator alone.

**Calibration** (`P calibrate.py`): of the six lines marked FAIL in `stylecheck.py` (the first five rows above, two
for form), 24 of 39 real creatures break none, 10 break one, 5 break two or three (the Nightmare, Zulrah, the
Whisperer, Elidinis' Warden, Cerberus). Watcher 1 breaks 5, the ray 5, watcher 2 breaks 3, the star hound 2, the
Warden-General 2. The order of the two watchers is the owner's (Rework, then Usable); the star hound (Strong) breaks
only the two form lines; the ray's Strong was given for a first weighted creature that [quoted words withheld].

What does NOT separate (measured, inside the band for all of ours): materials (real 2 to 5, median 4); tones in the
main ramp (4 to 18, median 8); median tone step (4 to 11, median 8); motifs used (4 to 6 of six); share of parts
carrying each used motif (0.44 to 0.95); light tone on convex folds (0.27 to 0.56) and dark tone in concave folds
(0.34 to 0.73); gradient along a part (0.08 to 0.33); patch-size spread; number of marks (76 to 406).

## 4. Rules (each number is the real band; `stylecheck.py` checks them)

1. **Write a paint plan before painting** (section 5): materials as named ramps, parts, per part its ramps, tone
   range, motifs, whether focal or calm; edge treatment; symmetry.
2. **Two to five materials**, the largest 38 to 83% of the skin, the second 11 to 37%, a third up to 20%. Every
   material is one ramp used wherever that substance appears. No colour outside the plan's ramps.
3. **A ramp**: 4 to 18 tones for the main material (median 8); median step 2 to 6 lightness; steps uneven; let hue
   or saturation move one or two steps from dark to light.
4. **Each part takes a RANGE of each ramp**, not the whole of it (Minotaur: arms tones 7 to 11, left leg 0 to 3).
   Parts differ by range (value masses); they do not differ by material unless the plan says so.
5. **Tone by plane.** Give each plane, plate or muscle one tone and change tone at the fold: at least 0.30 of fold
   length (30 degrees or more) must separate two paints (aim 0.45 to 0.6), and paint must change at least 1.3 times
   as often at folds as inside flat surfaces (aim 2 or more). Neighbouring planes differ by one ramp step (median
   step 4 to 11 across any tone boundary). Never by facing.
6. **No splinters.** A mark is two faces or more (median 3); at most 0.31 of marks may be single triangles (aim
   under 0.2) and at most 5% of the skin may lie in one-triangle patches (aim under 2%).
7. **A focus and calm.** One zone of eight holds at least 24% of all marks (aim a third or more): the face, a
   chest or back emblem, a weapon. Limbs are calm: fewer boundaries, no small marks beyond their fittings.
8. **Mirror the paint.** Where the body is mirrored, at least 0.76 of it is painted alike (aim 0.9 or more).
   Asymmetry is one named mark in the plan.
9. **Edges** (weak rule: real creatures vary, ours already sit inside): a paler tone on 0.27 to 0.56 of convex fold
   length, a darker tone in 0.34 to 0.73 of concave fold length, on every part that has plates.
10. **Motifs recur.** Use four to six of the six kinds; each kind you use should appear on 44% or more of the parts.

## 5. The paint plan and the checker

Format and reader: `paintplan.py` (docstring has the JSON format). `P paintplan.py read npc:ID plans/real-ID.json
--parts humanoid` writes the plan a real creature follows: its ramps with exact colours, per part the ramps and tone
ranges it uses, its motifs, its share of the marks, whether it is a focus, the edge shares, the symmetry. Three are
written for the watcher's build: `plans/real-12812.json` (Minotaur), `real-7541.json` (Tekton), `real-13599.json`
(Tormented Demon). `P sheets.py swatch OUT.png KEY,.. [--plan PLAN.json]` draws a creature's parts and one palette
strip per part.

`P stylecheck.py DEF.json PLAN.json` prints PLAN lines (the creature against its own plan, part by part, in words:
[quoted words withheld], "no light edge on 84 percent of convex fold length on
the left leg", "'accent spot' marks appear on the trunk only", "head is a focal part and holds 13% of the marks on
22% of the skin") and BAND lines (against section 3's bands). PASS = no PLAN line fails and at most one BAND line
fails (34 of 39 real creatures). That allowance was set after seeing the repaint's result; with none allowed, 15 of
39 real creatures would not pass.

## 6. Sheets

`contact-owner-1.png`, `contact-owner-2.png`, `contact-other.png`, `contact-ours.png`: every creature measured,
paint only, front and back, no light. `swatch-humanoids.png` (Minotaur, Tekton, Tormented Demon, watcher 2, the
repaint), `swatch-bosses.png` (Nex, Nightmare, Doom, Maggot King, Sotetseg, Zebak), `swatch-ours.png`,
`swatch-plan-parts.png` (parts by label): each part's palette strip with its numbers. `density.png`: where the
paint boundaries are. I looked at `swatch-humanoids.png`, an early contact sheet of eight and a three-creature close
sheet; the other sheets were drawn and not read.

## 7. Not proven, not run

- No measure here has been put to the owner. That splinters, mirroring, planes, focus are what the owner meant by
  [quoted words withheld] and [quoted words withheld] is inference from five of our creatures.
- The bands are 10th to 90th centiles of 39 hand-picked creatures; a tenth of real creatures fail each line by
  construction. The star hound, graded Strong, fails both form lines.
- Parts are zones by position, not anatomy; "focal" is the densest zone, wherever it is. Humanoid label parts are
  cut by label centres with fixed fractions and were checked by eye on the watcher only.
- Materials are found by a hue and saturation rule; on some bosses it splits one substance in two (Doom's reds)
  or joins two (Nex's pale skin and violet). Counts of materials are soft.
- Motifs are six classes by shape and contrast. Stripes, scales, cracks, veins as named patterns were NOT
  recognised; [quoted words withheld] here is line or spot, light, dark or accent. Wear and irregularity were measured
  only as patch-size spread and ramp-step unevenness.
- Lit pictures were not measured: all of this is paint. Shading noise from a mesh's own triangulation (the second
  watcher's welded body) would read as "texturing" too and is not in any number here.
- Edge treatment is measured on folds of 30 degrees or more, not on plate borders recognised as such.
