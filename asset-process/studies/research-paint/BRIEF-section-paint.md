## Paint: plan it, then lay it (after shape and depth are closed)

Status 2026-11-26: UNTESTED BY THE OWNER. Written from measurements of 39 real high-end creatures
(`{ROOT}/studies/research-paint/PAINT.md`: read all of it) and proven only on a repaint of the second
stone watcher that the owner has not seen. The owner's words this section answers: "inconsistent texturing and a
lack of well-defined artistic styling across the model". There are no textures: texturing is how per-face colour is
laid. Do this step only when `style_npc.py` passes and the shape and depth gaps of the helm's process are closed;
paint cannot repair shape, and a repaint must not move a vertex.

`P` = `{PYTHON} -I`, tools in `{ROOT}/studies/research-paint/`.

1. **Study three plans.** For three real creatures of your build run
   `P paintplan.py read npc:ID YOURFOLDER/plan-ID.json --parts humanoid` (omit `--parts` for other builds) and
   `P sheets.py swatch YOURRENDER/swatch.png ID,ID,ID`. Write in `design.md` what each does: its ramps colour by
   colour, which tones each part uses, where its marks are, what its one asymmetry is.
2. **Write your paint plan** (`paint-plan.json`, format in `paintplan.py`; worked example
   `studies/research-paint/plans/watcher-2-plan.json`) BEFORE any face is coloured:
   - two to five materials, each a named ramp of exact colours with its job; main material 38 to 83% of the skin,
     second 11 to 37%; main ramp 4 to 18 tones, steps of 2 to 6 lightness that are NOT equal, hue or saturation
     moving a step or two from dark to light;
   - parts by vertex label (head, trunk, each limb, a weapon); for each part the ramps it may use and the tone
     RANGE of each (parts differ by range, as the Minotaur's pale arms and dark legs do), its base tone, its motifs
     (light, dark or accent lines and spots), and whether it is FOCAL or calm, with its share of the marks;
   - one or two focal parts holding at least a third of the marks; limbs calm;
   - the edge treatment (which edges get a paler line, which recesses a darker one) and the one allowed asymmetry.
3. **Lay the paint by the plan:**
   - tone BY PLANE: each plane, plate or muscle one tone, the tone changing AT the fold; neighbouring planes one
     ramp step apart; raised planes a step paler, recesses a step darker; never by facing;
   - every mark two faces or more, its boundary on edges of the mesh: re-split faces where a mark needs an edge;
     no single-triangle marks, no chips;
   - paint the -x side, mirror it to +x, then add the one asymmetry;
   - no colour that is not in a ramp of the plan.
4. **Check every round:** `P stylecheck.py YOURS.json paint-plan.json` must end in PASS: no PLAN line failing, at
   most one BAND line failing. It names what is inconsistent and where. The lines that separated our rejected
   creatures from real ones, with the real band: single-triangle marks at most 0.31 of marks (real median 0.17);
   skin in one-triangle patches at most 5%; mirrored surface painted alike at least 0.76 (median 0.96); fold length
   separating two paints at least 0.30 (median 0.49); paint changing at folds at least 1.3 times as often as inside
   flat surfaces (median 2.1); the densest of eight zones holding at least 24% of the marks (median 36%).
   Run `style_npc.py` as before: it must still pass.
5. **Judge beside the three real creatures**, paint-only (`sheets.py contact`) and lit (`npc.py`: standing, walk,
   attack; game size and close; front and back). Answer in `notes.md` each round: where is mine patchier, busier
   or flatter than theirs, and what did I change?

The checker is a guard, not the judge: it passed nothing the owner has seen. A creature can pass it and be dull
(the watcher's repaint lost its pale face and half its small marks to pass). Keep the idea readable in two or three
hues at game size (BOSSES.md section 4) and say in the report what the plan gave up.

Report, in addition: the paint plan, the three real plans studied, the final `stylecheck.py` output in full, the
swatch sheet of yours beside the three, and one line on each BAND line that is WARN or FAIL.
