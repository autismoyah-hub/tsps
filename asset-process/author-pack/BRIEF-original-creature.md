# Brief: an original creature on a borrowed skeleton (second version)

STATUS 2026-11-30, READ FIRST: PASSED ONCE EACH for a humanoid creature and a boss. Built by the helm's process plus the last part of this file ([quoted words withheld]), the stone watcher III was graded Strong ([quoted words withheld]) and the Warden-General III, by `night/prompt-boss-original.txt`, Strong ([quoted words withheld] to the first). Worked examples: `builds/creature-stone-watcher-3/`, `builds/boss-warden-general-3/`. Still open, in the owner's answers: the watcher's eyes, nostrils and horns STILL read as stuck on although they are faces of the skin (the checker does not measure this); and the earlier, more detailed paint [quoted words withheld]: clean must not mean bare. The status lines below are history.

Status 2026-11-29: NOT A PROVEN PROCESS. Two originals were made under it: a star hound graded Strong and a stone watcher graded REWORK by the owner ([quoted words withheld]) though it passed the style check with nothing out. The style check measures counts in the recent band, not quality. Until this brief is revised, an original creature must also go through the helm's lesson (`BRIEF-player-helm.md` steps 4 to 6): measure shape, depth and detail on two or three real creatures of the same body plan and on yours, write the numbers down, and close the gaps, judged beside them every round. Earlier status, 2026-11-28: passed once. A star hound made under this brief was graded Strong by the owner ([quoted words withheld]; [quoted words withheld]): `{ROOT}/builds/creature-star-hound-3/` is the worked example of an original. Rewritten 2026-11-27 from measurements. The first version of this brief produced three
creatures the owner graded Usable and called, of one, [quoted words withheld].
Measurement of 111 real creatures showed that brief wrong in fourteen places. Since then three real creatures have
been REBUILT from sheets and passed the creature gate, and the owner graded the two shown Strong.

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server. The owner's
standard, in the owner's words: [quoted words withheld]. Quality matters;
speed does not. A creature here is a NEW mesh of your own, labelled so that it moves with the animations of a real
creature whose skeleton it borrows. You never copy, import or trace a real creature's mesh. Your creature, the
skeleton it borrows, its idea and its ranges are in the task line that came with this brief.

## Read first, completely

1. `{ROOT}/studies/research-creatures/CREATURES.md`: all of it. Section 4 is the rules you build to
   (one welded skin; smooth shading; an irregular hand split; tone by part and marking, never by facing; soft steps
   and many tones; lit once at rest; labels as the skeleton's real creatures; and the recent band: faces, colours,
   painted detail inlaid in the skin, rounder forms). Section 3 scores the first three creatures and says in numbers
   why they read as dated: do not repeat them.
2. `{ROOT}/studies/research-creatures/LIGHTING.md`: a creature is lit ONCE in its rest pose and keeps
   those tones in every frame; the underside is already near black, so never paint it darker.
3. `{ROOT}/studies/research-creatures/BRIEF-creature-rebuild.md` and the three rebuilds made under it,
   which passed: `{ROOT}/builds/creature-rebuild-11068/` (Lynx), `-12812/` (Minotaur), `-11175/`
   (Araxyte): `notes.md` and `REPORT.md` in each. They show how a recent creature is actually built here: the skin,
   the label rings, the undrawn pivot faces, the inlaid paint. The one whose body plan is yours is your worked example.
4. `{ROOT}/author-pack/RULES.md`, "How the game draws" (an ordinary NPC is depth-tested like scenery).
5. The tools (read each docstring): in `studies/research-creatures/`: `style_npc.py` (the style check: run it every
   round), `cmesh.py`, `palette.py`, `contact.py`, `sheet_npc.py`, `lineup_npc.py`, `light_npc.py`; in
   `studies/research-npc/`: `npc.py`, `atlas.py`, `creature_kit.py` (it writes alpha 0 on every face: undrawn pivot
   faces need the rebuild authors' method), `meshio.py`. Python: `{PYTHON} -I`.
   Blender by script with `--python-exit-code 1` before `-P`.

## Study before designing

Draw and measure (`cmesh.py`, `palette.py`) at least three real creatures from 2019 on of YOUR body plan, whatever
their skeleton, and the real creatures on your skeleton. Write in `design.md` BEFORE building: their face counts,
colours, tones per hue, largest one-colour patch, inlaid details and what those details are; how their joints are
built; then your creature in words with sizes in game units, its idea in two words, its palette with each colour's
job and where each tone sits BY PART, the list of inlaid details you will paint (at least sixteen: eyes, teeth,
claws, nostrils, markings, straps), ONE signature for the outline, the ambient and contrast you choose and why, and
whose sequences it borrows.

## How to work

Work only in `{ROOT}/builds/NAME/` and `{ROOT}/render/NAME-*`. Each round: build,
export, run `style_npc.py` on your definition (every every-era limit must pass; recent bands should pass), draw
beside two recent real creatures of your body plan with `npc.py` (standing, walk, attack, death; game size and
close; front and back), READ the pictures, and write in `notes.md` what still gives it away beside the real ones and
what you changed. At least eight rounds, at most fourteen. Write files with your file-writing tool, never heredocs.

## Paint: a plan before any colour (added 2026-11-29 from `{ROOT}/studies/research-paint/PAINT.md`; untested by the owner)

Do this after shape and depth are right (the helm's process, in the status line above), not before.

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

## Not allowed

Copying, importing or tracing a real mesh. Modifying anything under `tools/`, `author-pack/`, `client-build/`,
`builds/research-*` or another build folder. Sending anything outside this machine.

## Report (to `{ROOT}/builds/NAME/REPORT.md`)

Paths of the final model, definition JSON, build script, notes and boards (close, game size, from behind; columns:
two recent real creatures of the body plan, yours); the final `style_npc.py` output in full; the table of your
measures beside the recent band; one line per round; any tear, hole or wrong layering in any frame; what still reads
as not-the-game or as dated, honestly; the three decisions you are least sure of; where this brief or a tool failed
you; cycles run.

# Additions of 2026-11-29: build the skin clean (these override anything above that they contradict)

Applied by the Teacher from the cleanness study. Where a section below says "replace" or [quoted words withheld], read it as already done.

Source: `{ROOT}/studies/research-clean/CLEAN.md` (24 real creatures; the watcher in two paints).
Status of every line below: UNTESTED BY THE OWNER. No creature built to these rules has been shown. The checker
passes the Minotaur, Tekton and 22 others on all seven lines and fails the watcher on all seven; it also fails star
hound 3 (graded Strong) on its two fold lines, so it is a guard, not the judge.

## A. Status line (replace the first sentence of the status paragraph)

Status: NOT A PROVEN PROCESS. The stone watcher, remade by the helm's process and then repainted to a paint plan,
was judged [quoted words withheld] but "lacking compared to the minitour and tekton references. Both
references are very clean, well defined and smooth ... The eyes and nostrils make me think of playdough and
playdough features stuck on a clay bound. The texturing I would describe as splotchy rather than smooth (minitour)
or thoughtfully detailed (tekton)." It passed `style_npc.py` and `stylecheck.py`. Measured causes
(`studies/research-clean/CLEAN.md`): its face was twelve closed lumps pressed onto a head; half its faces were not
in its skin; its folds were uncommitted; and a quarter of its skin was small weak patches of tone with no crease
or line to justify them, some of them added by the paint step of this brief.

## B. Add to "Read first"

6. `{ROOT}/studies/research-clean/CLEAN.md`: all of it, and its three boards in `boards/`. Run
   `P cleancheck.py YOURFOLDER --pic YOURRENDER/clean.png` every round from the first grey model on
   (`P` = `{PYTHON} -I`, in `studies/research-clean/`). The picture shows WHERE:
   magenta = splotch planes, red = closed lumps and stuck-on pieces, yellow = uncommitted folds.

## C. Add to "Study before designing"

For each of your three real creatures run `P features.py npc:ID --box UPLO,UPHI --pic OUT.png` on the head (and
`--zfrac 0,0.3` for a quadruped) and write in `design.md`, feature by feature (eye, brow, nostril, mouth, teeth,
horn or ear, each fitting): how many faces, whether it is faces of the skin (paint / inset / relief) or a separate
piece, how far it stands off the surface as a share of the creature's height, and its contrast. Then write the same
table for the creature you are about to build, BEFORE building. Say where your creature sits on the scale from
smooth (the Minotaur: marks and facets 15% of the skin) to detailed (Tekton: 35%), and what its marks will be.

## D. New section, before "Paint": Build the skin clean (rules; numbers are the band of 24 real creatures)

1. **One skin.** At least 0.655 of the faces in one welded piece (real median 0.99). Shoulders, forearms, shins,
   a chest or back plate of the creature's own substance are FORMS OF THE SKIN (a raised plane bounded by a hard
   fold and a tone step), not shells laid on it. A separate piece is a rigid thing of one label (a tooth, a ring,
   a weapon, a slab of armour), open where it meets the body. Closed pieces sunk into the skin: at most 0.124 of
   the faces (21 of 24 real creatures have none).
2. **Features are faces of the skin.** 98% of small contrasting features on real creatures are (1,508 of 1,536).
   - An **eye** is 1 to 8 faces of the head's own skin (2 is normal), repainted, lying within 0.5% of the
     creature's height of the surface, under a brow that is a fold of the same skin. Split the head so the eye's
     outline is edges. Never a ball, never a lump.
   - A **nostril** is 1 or 2 dark faces of the snout, flat. A **mouth** is a dark inset or a line of skin faces.
   - A **horn, tusk or spike** is the skull's skin drawn out: ridged (four to six sides; at least 0.38 of its inner
     edge length folded 50 degrees or more), 3.7 or more times as long as wide, meeting the head at a fold of 42
     degrees or more with a tone step of 27 grey levels or more there (a dark band at the base). No collar piece.
   - No closed piece shorter than 7% of the height anywhere (20 of 24 real creatures have none; none has over 2).
3. **Commit every fold.** Between two faces of one material: under 30 degrees (a smooth surface) or 50 or more (a
   crease, a ridge, a plane change). Hard-fold length at least 1.10 times firm-fold (30 to 50 degree) length
   (real median 1.61; aim 1.4); firm folds at most 0.20 of edge length. A firm fold is allowed where the paint
   changes across it. Never let a script jitter, carve or "sculpt" vertices on a finished surface: that is where
   the watcher's uncommitted folds came from.
4. **A patch of skin between creases is one smooth surface.** At most 16% of the skin in splotches (small planes
   under 0.2% of the skin that differ a little from their neighbours with no hard fold, strong step or other
   material at their border); aim for 10% or less (Minotaur 8.8%, Tekton 9.8%; smooth creatures median 4%).

## E. Change in "Paint: plan it, then lay it", step 3, first bullet

Replace "tone BY PLANE: each plane, plate or muscle one tone, the tone changing AT the fold; neighbouring planes
one ramp step apart" with:

- tone BY PLANE for planes of at least 0.2% of the skin: each such plane, plate or muscle one tone, the tone
  changing at a HARD fold (50 degrees or more); neighbouring planes one ramp step apart. A plane smaller than that
  takes its neighbour's tone, unless it is a MARK: another material, or at least 8 paint lightness (6 as lit) from
  its surround, two faces or more. One soft step on every small plane is a mottle (the repaint raised the skin in
  small weak patches from 0.063 to 0.117; no real creature of 24 is above 0.110, the Minotaur is at 0.027).

## F. Add to step 4 of the paint section ("Check every round")

`P cleancheck.py YOURS.json --pic OUT.png` must end "INSIDE the real band on all 7 lines", and its style line must
name the style your design chose. The lines, with the real band: splotch at most 0.158; hard / firm fold length at
least 1.10; firm folds at most 0.199; largest welded piece at least 0.655; faces in closed pieces sunk into the
skin at most 0.124; small closed lumps at most 2; small contrasting features built as closed pieces at most 0.034.
Line 5 can be satisfied by deleting unseen faces without changing the look: read it together with line 4.

## G. Add to "Judge beside the three real creatures"

Draw the head from the SIDE, close, beside the real heads (`features.py --pic`, third view; or `crop.py OUT.png
A,B,C --band 0.72,1.0 --yaw 15,90`). Anything that stands off the head's outline in profile and is not a horn, an
ear, a tusk or a crest is a lump: cut it into the skin. Answer in `notes.md` each round: which of my features
would come off if the head were shaken?

## H. Add to the report

The full `cleancheck.py` output and its picture for the final model; the feature table of section C for your
creature as built; one line for every verdict line that is OUTSIDE.

## I. What a script can and cannot repair (from the mechanical test, `CLEAN.md` section 7)

A script can cut a lump's footprint into the skin, delete hidden faces and absorb weak tone patches: on the watcher
that moved splotch from 0.229 to 0.140 and removed the lumps on the profile. It cannot shape an eye (the cut eyes
came out as slivers), commit folds (0.70 to 0.82; needs 1.10) or weld armour into the skin (0.495 to 0.580; needs
0.655). These three are design decisions: make them in the first grey model, not at the end.



## Additions of 2026-11-28 (from `studies/research-gap/GAP.md`, FINDINGS 89): PRODUCED IMPROVEMENTS ONCE EACH

Owner, 2026-11-28, of watcher IV, Warden-General IV and star hound IV, each reworked to these lines: [quoted words withheld]. Improvement over our own earlier versions is not yet the bar ([quoted words withheld]); one pass each is not proof. Worked examples: `builds/creature-stone-watcher-4/`, `builds/boss-warden-general-4/`, `builds/creature-star-hound-4/`.

The owner's open words: [quoted words withheld]; [quoted words withheld]; [quoted words withheld]; "definition"; [quoted words withheld]. One measure separates real creatures from ours (`gapcheck.py`: concave share at least 0.047); the rest are review questions. Inside every band is not passing; the owner's eye is the test.
- Carve the forms. At least 0.047 of the edge length inside one paint folds inward 30 degrees or more, placed BETWEEN masses (chest, stomach, shoulder, slab against slab), never as grooves along trim (that games the number and is the fault). Run `P {ROOT}/studies/research-gap/gapcheck.py YOUR.json` every round.
- Our best builds are TIDIER and more symmetric than real ones: do not make every part a neat tube or bevelled box; parts meet at a decided line with a fold or a colour change at the joint.
- Every eye, nostril and mouth has a form of skin around it (socket, brow, lip) that reads before any paint.
- Every colour region can be named as a part or a material; no paint that crosses forms in blobs.
- Big, decided planes; no patchwork of near-equal tones.
- The black silhouette at game size beside the nearest real creature: it tapers, and you can name three features in it.
- At game size the marks differ in value from the body and are larger than 2 pixels.
- Designed motifs with one focus.
Answer the seven review questions of GAP.md section 3 in notes.md every round, beside the real board in `studies/research-gap/boards/`.
