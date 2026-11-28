# HELM3: testing the owner's own words on the helms, pre-registered

Private to this machine (cache-derived). Folder `{ROOT}/studies/research-helm3/`.
`P` = `{PYTHON} -I`, run in this folder.

```
PREREG.md                    the 17 measures and the test, written before any helm was measured
P h3.py                      the 17 measures on 14 helms (+5 unjudged real helms for reference) -> data/h3.json
P stats.py                   leave-one-out, exact chance -> data/stats.json, data/table.md
P posthoc.py                 three POST-HOC looks (cannot count) -> data/posthoc.json
P look.py ; P board.py       pictures (work/look-*.png) and board-helm3.png
P helm3check.py HELM-labelled.bin [SLOTSPEC] [--rest OUTFIT]    prints the measures with bands (no gate: none survived)
```

The owner's judgement of C, paraphrased: the real helms read as crisp, well moulded and with defined features; C read as
uneven and misshapen, without visible intent. Groups: ACCEPTED 6 (Torva, Oathplate, Justiciar, Neitiznot faceguard, Serpentine, Meridian v5);
REJECTED 8 (Umbral A, B, C; Meridian v4-a/b/c; v3-a and v2-a, the two by-eye helms of HELM2).

## 1. Result: no measure survives

A measure counted only if (PREREG) its direction matched the owner's words, leave-one-out classified at least 12 of 14
helms, and C was called rejected. **0 of 17 survived.**

| word | measure | LOO /14 | C called REJ | note |
|---|---|---:|---|---|
| crisp | `cr_hard_step_close` (share of hard tone steps, close) | 8 | yes | 8/14 = calling everything rejected; no separation |
| crisp | `cr_ramp_game` (soft-ramp share, game size) | 7 | no | |
| crisp | `cr_soft_fold` (edge length folded 15-40 deg) | 5 | no | accepted helms have MORE soft folds (Torva 0.39) |
| molded | `mo_relief` (distance from smoothed shell) | 9 | yes | C 0.109 is inside the accepted 0.099-0.172 |
| molded | `mo_steel_shading` (steel p90-p10, close 3/4) | 7 | yes | |
| defined | `df_line_contrast_game` (p90 of neighbour steps) | 6 | no | C 19.2 is above Torva 18.2 |
| defined | `df_survive` (edges kept at game size / close) | 7 | yes | |
| defined | `df_dark_cut` (black cut against a light rim) | 10 | no | C 0.157 is in the accepted range |
| uneven | `un_edge_cv_plate` | 5 | no | |
| uneven | `un_face_logsd` | 8 | yes | |
| uneven | `un_asym` (left-right shell asymmetry) | 6 | yes | C 0.059 is just inside Oathplate's 0.060; A 0.068 is outside |
| misshaped | `ms_sil_front` / `ms_sil_side` (outline off a smooth fit) | 8 / 7 | yes | C 0.039 / 0.043, mid-band |
| misshaped | `ms_dents` | 6 | yes | DEGENERATE: smooth vertices are rare in all these meshes (C has 2 qualifying edges; its 1.0 means nothing) |
| intention | `in_long_lines` | 8 | yes | |
| intention | `in_dangling` (fold lines ending in nothing, per 100 u) | **12** | **no** | the one near-miss: accepted 0.49-1.53; A 3.21, B 2.09, v4 2.5-3.5, v2-a 9.6 outside; but **C 0.48 is the tidiest helm of all 14**, and v3-a 0.54 |
| intention | `in_mirror` (hard folds with a mirror partner) | 6 | yes | |

Full values: `data/table.md` (also the five unjudged real helms, which break most bands anyway: Dharok 0.93 hard steps,
Armadyl 1.32 face spread, Inquisitor 0.20 long lines).

**Chance.** Exact, by enumerating all 3003 ways of labelling 6 of the 14 values "accepted": the expected number of
measures passing the full rule by chance is **0.08** of 17; passing leave-one-out >= 12 alone, **0.14**. So one
survivor would have been notable, and none appeared. `in_dangling`'s 12/14 has a chance probability of 0.009 on its own
and is in the predicted direction, so it is probably real for the EARLIER rejected helms (A, B, v4, v2-a have fold
lines that stop in the middle of a surface) - and C, like v5, has none. It cannot explain C's verdict.

This repeats HELM2's pattern: the mesh/pixel properties that distinguished the earlier rejected helms (hard folds,
tidy fold lines) were things C's author was told to fix, C fixed them, and the owner still rejected C. Whatever the
owner sees on C is not in any of these 17 numbers.

## 2. What I see on C (by eye, written before the rings were drawn; `board-helm3.png`)

Beside A, Meridian v5 and Torva (front, 3/4, side at game size; front and 3/4 close):

1. **The crescent is lopsided and unrooted.** From the front one horn is tall and thick (wearer's right), the other
   short and thin; it is not seated on a ridge or a boss, it sits on the skull at an offset. v5's two fins and Torva's
   spike and fins are mirror pairs growing out of a crest line. C's own round notes saw this ("straight on it splits
   into two little horns", rounds 2, 4, 7) and it was never resolved. This is my best candidate for "misshaped" and "no
   intention".
2. **The face is a rounded egg, not a built face.** The central mask reads as one pale bulging lump with a soft
   gradient, with no frame, nasal or brow edge around it. Torva's face is a black T cut into a light raised frame;
   v5's is a barred grille under a brow band with a jewel. [quoted words withheld] on the real helms seems to mean planes that
   meet at decided edges around the face; C's face is soft where they are crisp.
3. **The cheek outline is broken by a dark wedge** on one side of the front view, so the left and right outlines of
   the shell differ. The accepted helms' front outlines are clean and paired.
4. **The temple and cheek are a patchwork** of small triangles of slightly different tone (3/4 views): the side "dart",
   cheek folds and laps each add a few facets, none long enough to read as a plate. Torva's side is two or three big
   planes; v5's are repeated, evenly spaced flutes.
5. **The side profile is kinked** (a hump at the nape lap, a seam down the front) rather than one curve.
6. **The eye slits are thin, unequal, and not seated** in a recess with a lit upper edge.

Do the measures point at these places? **No.** Asymmetry (`un_asym`, `in_mirror`) is computed over the whole shell and C
lands inside the accepted range (Oathplate is as asymmetric by this number); the crescent is excluded from the
silhouette measures by HELM2's 3.5-unit rule, so `ms_sil_*` never see item 1; the face lump (2) is smooth-shaded steel
with a normal tone range, so no tone statistic flags it; the patchwork (4) is below the scale of `un_face_logsd` and
`un_edge_cv_plate`, and C's many fold lines are long and connected (mesh row of the board: C has a dense web of
orange fold lines and few red dangling ends, the opposite of what the "intention" measures assumed). POST-HOC, I also
tried flat-shaded share, creases drawn with shared (smooth) normals, and crowded parallel folds: none separate
(`data/posthoc.json`; they cannot count anyway).

## 3. What the owner's words most likely refer to on C (opinion, with the evidence above)

- "misshaped" / [quoted words withheld]: the crescent (asymmetric in a way that reads as a mistake, not a design: no axis, no
  root, two unequal tips) and the front outline broken by one cheek wedge. Evidence: the front and 3/4 views; C's own
  notes; every accepted helm on the board is mirror-paired in its silhouette features. Not confirmed by a number.
- "uneven": the side and cheek patchwork of small unequal facets and tones (3/4 views), against two or three big planes
  on Torva. Not confirmed by `un_face_logsd` or `un_edge_cv_plate`, which work on the whole shell.
- not [quoted words withheld]: the face. The accepted helms' faces are hard-edged constructions (a
  cut T, a grille, a framed visor); C's is a soft lump with thin slits. HELM2 also failed to measure the face. This is
  the part I am least able to turn into a number and the most likely to matter, given how the owner speaks about
  "detail" and [quoted words withheld].

## 4. What an author would need to be told or shown (all UNTESTED proposals)

- UNTESTED: show the author the front views of Torva, v5, Justiciar beside their helm, at game size, and ask them to
  name the face's construction in one phrase (e.g. [quoted words withheld]) before modelling; reject any face
  that is one rounded surface without a frame edge around the dark.
- UNTESTED: any feature off the centre line must have a mirror partner, or be one piece that crosses the centre line
  with a single clear root (a boss, a ridge) - no oblique single blade without a seat.
- UNTESTED: per view, the author lists the plates a viewer can count at game size; a plate made of many small
  triangles of different tones counts as noise. Ask for fewer, bigger planes on the temple and cheek.
- UNTESTED: a silhouette review at game size, front and side, as solid black shapes beside the references, judged for
  "one decided curve per side, paired left/right".
- UNTESTED: stop building to measured bands alone. Two studies have now found numbers that the earlier rejected helms
  fail and that a new helm can satisfy while still being rejected. The owner's verdict appears to track design
  decisions (a face, a paired crest, few big planes) that a band does not encode.

## 5. What I could not establish

- No measure survived, so there is no measured account of C's rejection. Everything in sections 3 and 4 is my eye.
- Accepted group is 6 helms, one of ours; rejected 8, five of them older versions of one design line. Bands are thin.
- The face could not be measured in HELM2 and I did not pre-register a face construction measure (my error: it was
  the thing I most suspected after looking). A future pre-registration should define face measures (dark area framed
  by a lit edge on all sides; count of straight edges bounding the face at close size) and test them on a NEW
  rejected helm, not on these.
- `ms_dents` was ill-defined for low-poly helms (almost no vertices qualify); `un_asym` puts Oathplate (0.060, accepted)
  beside C (0.059); I did not check whether Oathplate's number is a real asymmetry or sampling noise, and a whole-shell
  number cannot tell a deliberate asymmetry from an accidental one, which may be the difference the owner sees.
- The owner judged C in an outfit and in motion; I measured bind pose only.
