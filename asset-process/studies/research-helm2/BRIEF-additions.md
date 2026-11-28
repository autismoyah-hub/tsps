# Proposed changes to `author-pack/BRIEF-player-helm.md` (not applied; the author pack is untouched)

Source: `{ROOT}/studies/research-helm2/HELM2.md` (six accepted helms, five more real ones, seven
rejected). Status of every line: UNTESTED BY THE OWNER. The test helm `research-helm2/umbral-helm-a2` closes the
gaps by measure and has not been shown. `helmcheck.py` is a guard, not a judge.

## A. Status paragraph: add

Umbral A passed every check of this brief and failed the owner. Measured afterwards (`research-helm2/HELM2.md`):
0.188 of its edge length was folded over 45 degrees against 0.254 to 0.447 for the five and the passed Meridian
helm (its own report printed 18.8% against 27.5 to 43.2 and nothing was changed); it was painted ring by ring; its
sides were painted darker than its front; its crest was the tallest allowed on the narrowest shell allowed; and
the recoloured stand-in body was louder than the helm.

## B. Step 4 becomes a gate, not a table

Run `P {ROOT}/studies/research-helm2/helmcheck.py YOURS-labelled.bin SLOTSPEC --rest "REST OF THE
OUTFIT" --body CUIRASS.bin` on the bare shell, after the first decorated round and every round after. Before the
report, the two SHARP lines must be inside: **at least 0.254 of the shell's edge length folded more than 45
degrees, and at least 0.241 on the skull alone.** Get there with construction, not ornament: plates that lap with a
real step, a ridge, a cut sight, a brow shelf. A lathe of rings with paint on it is the shape of all seven rejected
helms (0.03 to 0.21). The measures of step 4 are numbers to CLOSE during the rounds, not to compute for the report.

## C. Add to step 3 and 5 (paint and proportion; warnings from `helmcheck.py`, each with its band)

- Paint by PLATE, never by ring. Level tone bands over upright ones in the side view at most 1.44; level steps over
  upright steps in front at most 0.95.
- Do not paint the shade. Side and back steel gets the front's paint: at most 0.34 of side- and back-facing steel
  under lightness 20; the largest near-black patch in the unlit side view at most 0.19 of the helm.
- The deepest recess is lightness 0 to 2.
- Whole height at most 1.83 times the shell's width at the skull; rise above the crown at most 0.53 of that width.
  `fit.py` allows a 16-unit crest and a 22-unit shell separately; no accepted helm has both.

## D. Add to "Limits": the body the helm is shown on

A stand-in body must not be louder than the helm. At game size from the front: the body's tone range at most 122;
the helm's range at least 0.9 of the body's; the helm's lights no more than 16 grey levels under the body's. A
recolour that puts a second paint 60 or more lightness steps above the body's steel fails this (the Umbral
stand-in: silver 76 and 91 on steel 8 to 22). If the stand-in cannot be made to pass, show the helm on two bodies.

## E. Add to the report

The full output of `helmcheck.py` for the final helm and for the bare shell; for any line outside, the reason in
one sentence. And a reminder at the top of the brief: inside every band is not passing; the owner's test is.
