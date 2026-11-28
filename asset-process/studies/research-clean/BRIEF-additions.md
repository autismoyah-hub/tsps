# Proposed additions to `author-pack/BRIEF-original-creature.md` (not applied; the author pack is untouched)

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
