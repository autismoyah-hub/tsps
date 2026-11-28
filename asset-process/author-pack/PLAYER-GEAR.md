# Player gear: fit and definition (measured against the owner's verdicts)

Status 2026-11-27: written from measurements that had to account for every verdict the owner gave on our helms and sets (three rounds of helms rejected; the earlier set preferred as a whole). Its two measured findings (FIT and DEFINITION) are checkable with `{ROOT}/studies/research-fit/fit.py HELM-labelled.bin SLOTSPEC` and `studies/research-fit/definition.py OUTFIT_ENTRY` (run them where they live). NOT yet confirmed by the owner on a new piece. Supersedes `FASHIONSCAPE.md`.

---

# Player gear: fit and definition (a rewrite of FASHIONSCAPE.md from the owner's verdicts)

Status: a hypothesis with measurements behind it and NO owner verdict on gear built to it yet. It replaces the design
rules of `author-pack/FASHIONSCAPE.md`, which the owner's third helm verdict and the set verdict contradicted. It was
written by measuring the owner's five named helms and sets against our seven rejected helms and our two sets. Nothing
here was tested by building a helm. Private: cache-derived numbers.

Everything is reproducible from `{ROOT}/studies/research-fit/`:
`fit_study.py` (32 helms) -> `fit-measures.json`; `census.py` (every real head item) -> `census.json`; `overlay.py` ->
`render/research-fit-overlay/`; `motion_fit.py` -> `motion-fit.json`; `def_study.py` -> `def-measures.json` and
`render/research-fit-definition/`; `bands.py` -> `bands.json`; `mesh_cause.py` -> `mesh-cause.json`; the two tools
`fit.py` and `definition.py`; `run_tools.py` -> `tool-table.txt`.

## 0. The two findings in one paragraph each

**Fit.** Every one of our seven helms is a shell that stands 4 to 7 units off the bare head all the way round
(median stand-off 4.1 to 7.5; never closer than 2.5 to 4.4). The owner's five reference helms sit ON the head: median
stand-off 0.3 to 2.3, touching or inside the hair surface at their closest (-1.7 to 1.0). No helm of the 25 real ones
measured is as far off at its closest point as the closest of ours (real maximum 1.5, ours minimum 2.5). Among all 225
real head items that hide the hair and cover the head, the 25 that stand as far off as ours are the Fishbowl helmet,
the Deep sea helmet, the Snowglobe helmet, the Bucket helm, the pumpkin heads, the Frog, Bunnyman and Penguin masks,
the Present box hat, the Fighter hat, the Ancestral hat, the Mask of rebirth and two dragon masks: the game's joke and
costume heads, and two cloth hats. No metal war helm is among them. Our helms have the fit of a Bucket helm. The
silhouette width hides this: Torva is 28 to 30 wide at the skull and so are ours, but Torva's is the 20 to 22 wide head with a
shell 0.7 off each temple plus fins, and ours is a plain shell 28 wide.

**Definition.** At game size, over 34 pictures (standing, walk and run, front and back), our earlier set has 8.43
tone steps per 100 figure pixels, 4.35 line pixels per 100 and 54 readable regions; the redesigned set has 5.86, 2.25
and 35.5: 30 % fewer steps, 48 % fewer lines, a third fewer regions, with or without the helm, standing or moving.
The three light-coloured reference sets (Justiciar, and Bandos under the Serpentine helm and the faceguard) have 7.1
to 8.3 steps, 3.55 to 3.62 lines and 48.5 to 49 regions: the earlier set is with them, the redesigned set is below
all three. Torva has 6.34 / 2.58 / 39 and Oathplate 3.17 / 0.94 / 24.5: the two DARK sets are less divided, Oathplate
far less. So definition is what separates our two sets and what the light references share; it is not a law of every
desirable set. The means differ by set: Justiciar divides with a second material (gold trim and blue cloth: 41 % of
its steps are between materials), Torva with light edges on dark plate (7 % between materials; 1.7 light-line pixels
per 100 against 0.16 of a second material), the earlier Meridian set with both brass lines and dark lines. Both of
our sets lack darks: their darkest 5 % of pixels are at grey 14.3 (earlier) and 20.0 (redesigned) against 7.2 to 13.8
for the five; our paint never goes below lightness 18 where the references' goes to 0 to 12.

## 1. Rules for a helm

Each rule: what, the references that show it, the number, how a reviewer checks it. `fit.py` prints every number.

**H1. The shell sits on the head, not round it.** A helm that hides the hair REPLACES the hair kit; it does not
clear it. Build the shell on the bare head's own surface (`labels/body-kits.json`, hair + jaw: half-width 11 at
heights 180 to 184, 10 at 190, face at z -19, back of the skull z 4, crown 196) pushed out by 0 to 2 units.
- Shown by: all five. Torva: brow -0.2, temple 0.7, cheek 1.0, crown 1.9. Oathplate: brow -0.1, temple -0.1, cheek
  1.2. Serpentine: brow 0.3, temple -0.4, cheek -1.1 (inside the hair). Justiciar 1.1 / 2.2 / 0.7. The faceguard is
  the loosest of the five: 2.0 / 2.1 / 2.1, crown 3.0.
- Number: median stand-off 0.3 to 2.3; closest tenth -1.7 to 1.0; brow -0.2 to 2.0; temple -0.4 to 2.2; cheek -1.1
  to 2.1; crown 0.2 to 3.0; jaw -0.1 to 4.1. Ours: median 4.1 to 7.5, closest 2.5 to 4.4, brow 3.0 to 7.0, temple
  3.9 to 4.7, cheek 4.8 to 5.9, crown 3.4 to 13.9, jaw 4.8 to 7.7.
- Check: `fit.py`, lines `so_med so_p10 so_brow so_temple so_cheek so_crown so_jaw`; and look at the overlay
  (`overlay.py` draws it): the white cut line must lie on the red head line except where a named feature leaves it.

**H2. The face plane is the face.** In the side view the front of the helm between the brow and the chin is 0 to 2
units in front of the bare face (z -18 to -21) on Torva, Justiciar and Oathplate; a visor or snout that is the helm's
own feature reaches 4 to 6 (faceguard z -24, Serpentine z -25). Ours: the closed helm's visor z -27, all three round
3 helms z -26 to -28 over the WHOLE face, 6 to 9 units out, with nothing under it.
- Number: `chin_overhang` (how far the face of the helm stands in front of the neck and collar below it) 2.8 to 11.2
  for the five; ours 10 to 18 (six of seven over 11.2). `fore` 0 to 6; ours 7 to 8 on four.
- Check: `fit.py` lines `chin_overhang`, `fore`; side overlay.

**H3. In the side view the helm is not much deeper than the neck and collar under it.** Narrowest depth of helm plus
collar between heights 158 and 175, as a share of the shell's depth at the skull: 0.67 to 1.01 for the five; ours
0.38 to 0.58: a ball on a stalk. The five get there two ways: a shallow skull (Torva, the faceguard and Oathplate are
26.8 to 27.0 deep at the skull, the bare head is 22.5; ours 29.8 to 38.5) or a neck guard and collar that fill the
space (Justiciar's helm reaches down to 161 behind and is 25.5 deep at the neck; Bandos's collar stands to 171).
- Check: `fit.py` line `pinch_s_ratio`; `d_skull` for the cause.

**H4. The lower edge comes down to the collar in front.** Lowest point in front of the face: 163 to 168 for the
five (collar 164). Rounds 1 and 2: 169 to 172 (a chin and neck gap under the helm). Round 3 fixed this (165).
- Check: `fit.py` line `low_front`.

**H5. Size comes from the signature, not the shell.** The five are 1.27 to 1.55 head-widths wide, rise 5 to 16
above the crown and reach 7 to 21 behind the skull, all of it fins, horns, crowns, wings and crests standing on a
shell that obeys H1. Ours put the size in the shell and then added ornament on top (round 3: crescent 14 up, wings to
2.27 head-widths, a star 1.73 wide, on a shell 4.1 off the head).
- Number: `wid` 1.27 to 1.55, `above` 5 to 16, `w_skull` 21.8 to 30.2 (fins included), shell width at skull over
  shoulder width 0.29 to 0.41.
- Check: `fit.py` size lines (not marked `*`: a deliberate signature may leave them; say so in the design note).

**H6. All five cover the head.** The shell covers 0.96 to 1.00 of the head's directions; none of the five is an
open-faced helm, though two (Serpentine, faceguard) leave the jaw kit in place. Our open helms covered 0.83 to 0.95.
This is a fact about the five, not a rule against open helms: the Helm of neitiznot (0.88) is coveted.

What did NOT separate ours from the five, so there is no rule: whether the neck shows (Serpentine and the faceguard
show 370 to 665 skin pixels of neck and chest on Bandos; Torva, Justiciar, Oathplate none; ours 0 to 600); the drawn
neck pinch through walk and run (noisy, overlapping); helm to shoulder proportion (five 0.27 to 0.40 drawn, ours
0.38 to 0.40 except the winged one 0.49); overall height and depth ratios; the lowest point behind (all 161 to 165).

## 2. Rules for a whole set

`definition.py` prints every number. "Light" means the drawn figure's median grey is 35 or more (ours are 53 to 57).

**S1. A light set is divided into plates the eye can count at game size.** Shown by: Justiciar, Bandos (under both
helms). Number, median of 34 pictures at zoom 300: tone steps of 24 grey levels or more 7.1 to 8.3 per 100 figure
pixels; line pixels 3.55 to 3.62 per 100; readable regions 48.5 to 49; no undivided patch over 0.30 of the figure.
Earlier Meridian set: 8.43 / 4.35 / 54 / 0.17. Redesigned: 5.86 / 2.25 / 35.5 / 0.21.
- Check: `definition.py`, lines `step lines regions reg_big`. The pictures `render/research-fit-definition/contact-*.png`
  show the figure above what the measure sees.

**S2. Put the division on the trunk and arms; legs may be calm.** Standing, trunk and arms (heights 114 to 164):
Torva 9.2 steps per 100, Justiciar 8.1, Bandos 8.5; legs 3.1, 4.4, 3.9. The earlier set: 9.8 and 8.6. The redesigned
set: 5.9 and 5.75: below every reference but Oathplate where they are strongest. (`bands.py`.)

**S3. A boundary between plates is PAINTED: a line of a second material, a light edge or a dark line. A fold alone
does not draw it.** Mesh edges where the paint changes (material, or lightness by 12 or more), per square root of
surface: Torva body 17.4, Justiciar body 24.5, Bandos body 14.7; earlier Meridian body 11.5, legs 9.7, boots 10.8,
gauntlets 12.3; redesigned 10.5, 4.4, 6.1, 9.2. Folds of 30 degrees or more in one paint: redesigned legs 15.4
against the earlier legs' 8.0. The redesign traded painted boundaries for folds and the picture lost its lines; the
references have both (Torva body: folds 15.7 AND painted 17.4). RULES.md already says it for laps: "a tone step
alone is too weak"; it holds for the whole set.
- Which means: any of the three, by set. Second material: Justiciar (2.28 line pixels per 100 are a second material).
  Light edge on dark plate: Torva (1.70 light, 0.16 second material). Dark line: all (0.73 to 1.31). The earlier
  Meridian set: 1.27 second material, 2.09 light, 1.75 dark. Brass is one means, not the only one: if brass is cut,
  the line must come back as a dark line or a light edge.
- Check: `definition.py` prints the means; `mesh_cause.py` for a piece.

**S4. Darks.** The five reference figures have their darkest 5 % at grey 7.2 to 13.8 and 25 to 72 % of the figure
under grey 30. Ours: 14.3 and 24 % (earlier), 20.0 and 16 % (redesigned). Painted lightness at the 5th percentile of
surface: reference bodies and legs 0 to 16 (Bandos tassets alone 22); our bodies, legs, gauntlets and boots 16 to 20. Give recesses, linings and the lines between plates paint of
lightness 8 to 14. This is the one line where even the preferred earlier set is (just) outside the references.
- Check: `definition.py` lines `p05 dark range`.

**S5. More division than the references was not punished.** The earlier set is over the top of the five on steps,
level steps, lines and regions, and the owner chose it and graded it Strong. There is no measured upper limit. (The
owner's taste for restraint is about ornament; these lines are plate edges.)

## 3. The design instruction

**Take the nearest of the five references and keep its SHELL: change only the signature, the face treatment and the
trim.** Exactly:
- keep: every `*` line of `fit.py` inside the five's range (stand-off at brow, temple, cheek, crown and jaw within
  about 2 units of the bare head; face plane 0 to 2 units in front of the face; `chin_overhang` at most 11;
  `pinch_s_ratio` at least 0.67; lower front edge at 163 to 168). In practice: model the shell by moving the bare
  head's own vertices outward 0 to 2 units, and compare cut lines with the reference in `overlay.py`;
- free: what stands on the shell (crest, horns, fins, wings, crown: up to 16 above the crown, to 1.55 head-widths,
  to 21 behind), the eye shapes, the materials and where the trim runs.
Support: the five differ from each other in everything that is free (rise above the crown 5 to 16, reach behind 7 to
21, width 1.27 to 1.55, one to six materials, a made face or a visor) and agree in the shell (median stand-off inside
a 2-unit band). Our seven differ from the five in the shell and from each other in the free part: three rounds of
changing the free part did not change the verdict. What is NOT supported by any verdict yet: that a helm with a
reference's shell and our own signature will pass. That is the next test, and it is cheap: the five rebuilds now
being made give five shells.

For a set: keep the earlier set's painted plate boundaries on trunk and arms (S1 to S3), add darks (S4), and take the
redesigned body's outline if the owner's choice of that body is to be honoured (section 5).

## 4. Checklist

| # | check | tool, line | pass |
|---|---|---|---|
| 1 | shell on the head | `fit.py` so_med, so_p10 | 0.3 to 2.3; -1.7 to 1.0 |
| 2 | brow, temple, cheek, crown, jaw | `fit.py` so_brow .. so_jaw | each at most 2.0, 2.2, 2.1, 3.0, 4.1 |
| 3 | face plane | `fit.py` chin_overhang, fore | at most 11.2; at most 6 |
| 4 | not a ball on a stalk | `fit.py` pinch_s_ratio | at least 0.67 |
| 5 | lower edge at the collar | `fit.py` low_front | 163 to 168 |
| 6 | size is signature | `fit.py` wid, above, w_skull, helm_to_shoulder; overlay | inside the five, or the design note names the feature that leaves |
| 7 | cut lines on the head | `overlay.py` picture, front and side | white on red except at named features |
| 8 | set is divided (light sets) | `definition.py` step, lines, regions | at least 7.1, 3.55, 48.5 |
| 9 | no undivided mass | `definition.py` reg_big | at most 0.30 |
| 10 | trunk carries it | `bands.py` trunk step (a study script: add the set to its entry list) | at least 8 |
| 11 | boundaries are painted | `definition.py` means line; `mesh_cause.py` | painted edges per piece at least the earlier set's (about 10) |
| 12 | darks | `definition.py` p05, dark | at most 13.8; at least 0.25 |

Lines 1 to 5 are a gate (exit code of `fit.py`). Lines 8, 9 and 12 are among the ten lines behind the exit code
of `definition.py` (it fails if any of its ten is below the least divided reference of the set's value class). Note
that the earlier set itself fails three of the ten, narrowly, all on darks (p05 14.3 against 13.8, dark share 0.24
against 0.25, value range 97 against 102): treat those three as a direction, not yet a gate.

## 5. What the first study said that the verdicts contradict

1. **"Bulk test: sol_f <= 0.86 OR ex_f <= 0.46" and "facet test: crease >= 0.29" as the account of "goofy".** Rounds
   1 and 2 failed them, round 3 passed them (ex_f 0.50 to 1.04 with sol_f 0.63 to 0.74; crease 0.28 to 0.35), and the
   owner rejected round 3 in the same breath as [quoted words withheld]. So passing them does not make a helm
   acceptable. They can be passed without touching the fault: a crest or wings notch the outline (sol_f falls) while
   the shell stays 4 units off the head, and a ball can be creased. They measure the outline and the facets; the
   fault was the shell's distance from the head, which neither sees (Torva and our helms have the same silhouette
   width at the brow, 28 to 30). They remain true descriptions of real helms; they are not acceptance tests.
2. **R5 [quoted words withheld] and the set built on [quoted words withheld].** The set built to it lost: folds do
   not draw plate boundaries at game size; paint does (S3).
3. **R8 [quoted words withheld], extended to the body ("five level
   brass bands: trimmed rune").** Level steps per 100 pixels: Torva 3.33, Justiciar 3.45, Bandos 3.83 to 4.05, the
   earlier set 4.87, the redesigned 2.92. The reference sets have as many level lines as upright ones, and the owner
   chose the set with the most.
4. **[quoted words withheld] (FINDINGS 34) and "beside Torva and Justiciar it
   reads as reward gear where the earlier set reads as trimmed rune" (FINDINGS 35).** The owner chose the other one.
5. **"All four pass the palette rule: the house palette was never the fault."** Not contradicted on hue; but the
   palette has no darks (S4), which the first study did not measure.
6. **Section 1's [quoted words withheld].** Right word, wrong
   measure: the balloon is stand-off, and by that measure round 3 was still one.

What the verdicts do NOT contradict: R2 (a signature in the outline) and R7 (echo at the shoulders): the owner chose
the body redesigned to them for its [quoted words withheld], and called round 3 [quoted words withheld]. R6 (shared
palette, one accent) was never at issue.

## 6. What is still taste

- Which signature, and whether a helm is [quoted words withheld]. The owner called round 3 "strange" as well as ill-fitted;
  only the second word is measured here.
- How much of the earlier set's "traditional" look to keep against the redesigned body's "adventurous" one. The
  owner valued both in words.
- Dark sets. Oathplate is the least divided set measured (3.17 steps, 0.94 lines) and the owner named it. Nothing
  here says what makes a dark set desirable.
- Whether more division than the earlier set would still read as restraint.
- Power and rarity, as the first study said: all five references are or were best in slot.
