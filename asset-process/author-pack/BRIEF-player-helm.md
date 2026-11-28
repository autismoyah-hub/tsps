# Brief: an original helm for players

Status 2026-11-30-b (interim Teacher): the lines marked UNTESTED below FAILED ONCE. Umbral helm C was built to all of them, was inside every band of helmcheck.py, and the owner preferred Umbral A: [quoted words withheld] (FINDINGS 79). Two helms of the second set from this brief alone have now failed: do not order a third on the same pattern. Status 2026-11-30 (interim Teacher): lines marked UNTESTED were added from the study `studies/research-helm2/HELM2.md` (FINDINGS 70); no helm built to them has been shown to the owner. Inside every band is not passing; the owner's test is.

Umbral A passed every check of this brief and failed the owner. Measured afterwards: 0.188 of its edge length was folded over 45 degrees against 0.254 to 0.447 for the five and the passed Meridian helm (its own report printed 18.8% against 27.5 to 43.2 and nothing was changed); it was painted ring by ring; its sides were painted darker than its front; its crest was the tallest allowed on the narrowest shell allowed; and the recoloured stand-in body was louder than the helm.

Status 2026-11-30: PROVEN ONCE, FAILED ONCE. A second set's helm made from this brief alone in one round (builds/umbral-helm-a) passed every check here and failed the owner's test: [quoted words withheld]; the owner prefers the Meridian helm, which came out of three rounds of the owner choosing among candidates. Passing this brief's checks is not passing. Status 2026-11-27: PROVEN ONCE. The process below produced the first original helm the owner passed ("The depth and
detail on 1 are great"; sharpness, depth and detail of the references: Yes; the fashionscape test: Yes), after seven
helms designed by eye and to an earlier study had been rejected in three rounds. Follow it in order. Do not skip a step
because the result looks right: every rejected helm looked right to its author.

Work under `BRIEF-original-armour.md` (what to read, tools, labelling, what is not allowed, the report) and read
`PLAYER-GEAR.md` (the measurements behind this brief). Your set, palette, signature direction and reference helm are
in the task line.

## The owner's references

Torva full helm (item 26382), Oathplate helm (30750), Justiciar faceguard (22326), Neitiznot faceguard (24271),
Serpentine helm (12931). A player helm is judged beside these, in whole outfits, never beside plain rune.
Rebuilds of all five, each with a section "How this helm is built", are in
`{ROOT}/builds/helm-rebuild-ID/REPORT.md`. The worked example of this whole brief is
`{ROOT}/builds/meridian-helm-v4-a/` (steps 1 to 3) and `builds/meridian-helm-v5-astra/` (steps 4 to 6):
read their `design.md`, `notes.md` and `REPORT.md`, and reuse `measure.py` from the second.

## The process

1. **Choose one reference for the shell** (the task line may name it). Study how it sits on the head: where the shell
   touches the hair surface, where it dips inside, where the face plane is, where the lower edge meets the collar, how
   deep it is at the skull. You may read real meshes to learn; you may not copy, import or trace one.
2. **Build the bare shell to that fit FIRST, with no ornament**, and run
   `python {ROOT}/studies/research-fit/fit.py YOURS-labelled.bin SLOTSPEC`. Every starred line must say
   PASS before anything else is added. The fault in all seven rejected helms was a shell standing 4 to 7 units off
   the head where real war helms stand within about 2: that is the fit of the game's bucket helms and costume heads.
3. **Add the signature and trim**: one signature for the outline, trim on ending edges, restraint. Run `fit.py` again
   after every round. It must still PASS at the end.
4. **Measure sharpness, depth and detail on the five references and on yours, and write the numbers in `design.md`
   before changing anything**:
   - sharpness: share of edge length folded over 45 degrees; crest lines and their fold angle; points and how acute;
     how thin fins and edges are; sharp folds where the paint also changes;
   - depth: how far the eye recess sits under its rim; plate steps in units; the black void's share; the painted
     lightness of the darkest recesses (references reach 0 to 16); the lightness range and tone count on one plate;
   - detail: tones per material; one-paint patches and the smallest that reads; lines of a second colour and where they
     run; the share of faces on face and brow against the back of the skull; tone steps and regions in close and
     game-size pictures.
   **UNTESTED, a gate, not a table:** run `P {ROOT}/studies/research-helm2/helmcheck.py
   YOURS-labelled.bin SLOTSPEC --rest "REST OF THE OUTFIT" --body CUIRASS.bin` on the bare shell, after the first
   decorated round and every round after. Before the report, the two SHARP lines must be inside: at least 0.254 of the
   shell's edge length folded more than 45 degrees, and at least 0.241 on the skull alone. Get there with
   construction, not ornament: plates that lap with a real step, a ridge, a cut sight, a brow shelf. A lathe of rings
   with paint on it is the shape of all seven rejected helms (0.03 to 0.21). These are numbers to CLOSE during the
   rounds, not to compute for the report.
   **UNTESTED, paint and proportion (warnings from `helmcheck.py`, each with its band):**
   - Paint by PLATE, never by ring. Level tone bands over upright ones in the side view at most 1.44; level steps over
     upright steps in front at most 0.95.
   - Do not paint the shade. Side and back steel gets the front's paint: at most 0.34 of side- and back-facing steel
     under lightness 20; the largest near-black patch in the unlit side view at most 0.19 of the helm.
   - The deepest recess is lightness 0 to 2.
   - Whole height at most 1.83 times the shell's width at the skull; rise above the crown at most 0.53 of that width.
     `fit.py` allows a 16-unit crest and a 22-unit shell separately; no accepted helm has both.
5. **Close each measured gap**, with the fit held. The detail the owner means is the references' kind: plates,
   ridges, recesses, a line of trim. Not added ornament.
6. **Judge every round** in the whole outfit beside your reference helm's outfit and two others of the five: at game
   size first, then the head close from the front, front three-quarter, side and behind; standing and in walk 819, run
   824 and slash 390. Each round answer in `notes.md`: beside the reference, where is mine still softer, flatter or
   plainer, and what did I change?

## Limits

Faces: the references run 214 to 377; go to about 420 only if each face earns its place. Steel with real darks. Hidden
slots as your design needs (`head+hair+jaw` for a closed helm). Test show-through by pixel count as `RULES.md` says.

**UNTESTED, the body the helm is shown on.** A stand-in body must not be louder than the helm. At game size from the
front: the body's tone range at most 122; the helm's range at least 0.9 of the body's; the helm's lights no more than
16 grey levels under the body's. A recolour that puts a second paint 60 or more lightness steps above the body's steel
fails this (the Umbral stand-in: silver 76 and 91 on steel 8 to 22). If the stand-in cannot be made to pass, show the
helm on two bodies.

## Report, in addition to the armour brief's

The final output of `fit.py`, and the same for the bare shell before ornament; the table of sharpness, depth and
detail numbers for the five references and yours; one board: your helm in the outfit beside three reference outfits,
whole figure front and back, then the head close from four sides. UNTESTED: the full output of `helmcheck.py` for
the final helm and for the bare shell; for any line outside, the reason in one sentence.
