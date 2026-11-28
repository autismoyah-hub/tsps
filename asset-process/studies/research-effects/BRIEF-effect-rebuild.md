# Brief: rebuild a real effect from its reference sheet

Status: first use. The tools are proven on three ceilings (real effects through our encoders: 0 differing pixels,
PASS) and on degraded copies; no effect has been rebuilt yet. Nothing here has been judged by the owner.

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server. The owner's
standard: an asset must read at once as the game's own art, and within that be as well made as possible. Quality
matters; speed does not.

Our method (measure real assets, REBUILD real ones held out, prove in blind line-ups that the rebuild cannot be told
from the real one, only then make originals) has been proved on furniture, weapons, armour and creatures. Effects
(projectiles, impacts, floor warnings, hazards) are a new domain: the first five originals were made BEFORE any
rebuild, to open the domain, and are not a standard. You are one of the first three authors to rebuild a real effect.
Your effect (spot animation id), its folder under `targets/` and your build name NAME are in the task line that came
with this brief.

## What is different about an effect

- **It is one mesh and one sequence, and the sequence is half the work.** An effect that is right standing still and
  wrong in motion has failed. Parts are labelled; frames scale, turn, move and fade them. The mesh never changes.
- **Vertex labels move, FACE labels fade.** The alpha transform adds 8 x value to the transparency of every face
  whose face label it lists. A model with no vertex labels at all is not moved by any frame (only its alpha
  animates): if the sheet says `vertex_labels: 0`, write none (ask how: `fxtools.save_model(m, path, labelled=False)`).
- **Translucency is per face**: 0 opaque .. 253 gone. Never 254 (a special dark path in the client); 255 is undrawn.
- **It is lit once, at rest, from above**, with the definition's ambient and contrast, like an NPC, and depth-tested
  like scenery: face priorities do nothing; translucent faces are drawn after opaque ones, far to near.
- **Its job is given by the packet, not the definition.** A projectile flies with its -z end forward, centred on the
  flight point, and its sequence loops; on an actor or a tile the effect is removed at the first wrap of its sequence.
- **It is judged moving, at game size**, where it is 100 to 1,500 pixels: silhouette, colour, translucency and timing
  are what is seen; a 2-unit detail is not.

## Read first, completely

1. `{ROOT}/studies/research-effects/EFFECTS.md`: sections 1, 3 and 4. Section 4 is the rules.
2. `{ROOT}/studies/research-skeleton/SKELETONS.md` sections 1, 3 and 4, and the docstrings of
   `skelc.py` and `skelanim.py` there (a frame base and a sequence of your own).
3. `{ROOT}/studies/research-animation/FORMAT.md`, "Base, frame and model labels" (the five transforms).
4. `{ROOT}/author-pack/RULES.md`: "How the game lights a model".
5. Your sheet: `targets/spot-ID/sheet-ID.png` (six rows: in its job at game size, close, a quarter turn, from behind;
   alone on a tile from above and from ground level), `motion-ID.gif`, `motion-ID-game.gif`, and `sheet-ID.json`
   (definition, counts, palette with surface shares, alpha values, rest box, and PER FRAME: hold, posed box, top and
   bottom heights, mean alpha; the transform kinds used and the number of groups of each type in its base).
6. `{ROOT}/studies/research-effects/originals/build.py`: a worked example of the route (mesh with face
   labels and alpha, skeleton, clip, loose definition). It is an example of the TOOLS, not of quality.

## The three targets

| id, name | job (how it is drawn) | what it tests | definition | mesh | sequence |
|---|---|---|---|---|---|
| 2206 toa_wardens_prayer_ranged_travel | `proj` | a projectile that is all translucency: a pale shard in a soft shell | scale 128, ambient 40, contrast 60 | 95 vertices, 170 faces, 5 colours (white 95%), alpha 49 / 50 / 210, NO vertex labels, no face labels listed; box x -10..10, height -30..30, z -84..89 | 12 frames of 5 = 60 cycles, loop 12; only its alpha frames act |
| 1595 verzik_powerblast_safezone | `spot` | a floor mark that lives 3.66 s: rings and a rising mote, built by alpha, scale and translate | scale 128, ambient 0, contrast 0, 3 recolours (paint the sheet's palette directly) | 109 vertices, 180 faces, 3 yellows (10, 7, 47 / 55 / 61), every face alpha 100, 10 vertex labels, 9 face labels; box x -57..57, height 1..129, z -58..58 | 61 frames of 3 = 183 cycles, plays once; alpha in 500 frame-groups, translate 234, scale 94; 27 groups |
| 2008 nex_finale_attack_impact | `actor`, height 100 | an impact on a player: a burst that grows 17 times and ends by fading | scale 72, ambient 5, contrast 40, 2 recolours | 146 vertices, 288 faces, 3 colours (60, 7, 70 at 70%; 49, 5, 78; 10, 7, 64), alpha 64 / 80 on 96% of the surface, 14 vertex labels, 9 face labels; box x -63..57, height -48..48, z -60..60 | 12 frames, holds 2 then 3 = 35 cycles, plays once; scale, rotate, alpha; largest posed dimension 341; 19 groups |

The JSON's numbers win over this table if they differ. Related real effects you may NOT read as meshes: for 2206 every
effect on model 45301 (only 2206 itself; its sequence 7571 is shared with 20 others and is also closed to you); for
1595 every effect on model 22769 (1026, 1290, 1363, 1589, 3028); for 2008 every effect on model 3088 or sequence 664
(131, 134, 137, 140, 346, 1536, 1600, 1765, 1785, 1786, 2034, 2076, 2127, 2258, 2446, 2535, 2951, 3037, 3290, 3811).
You may DRAW any effect by id.

## Tools (Git Bash; `P` = `{PYTHON} -I -B`; `E` = `{ROOT}/studies/research-effects`; `S` = `{ROOT}/studies/research-skeleton`)

- **Mesh**: by script, `S/meshkit.py` (`MKit`: loops, tubes, toruses, bipyramids, face labels) or the `Fx` class in
  `E/originals/build.py` (adds per-face alpha, two-sided faces, flat rings and fans). Whole units; welded by position.
- **Skeleton**: a `skeleton.json` compiled by `P S/skelc.py compile skeleton.json skel/`; take vertex AND face label
  numbers from the compiled `skel/rig.json` (a face label gets the number of the vertex label of the same name).
  Choose a free base id 2710 to 2799 and say which.
- **Sequence**: `NAME.anim.json` sampled by `P S/skelanim.py NAME.anim.json anim/` (keys per JOINT.CHANNEL, eases,
  waves; `lengths` are the sheet's holds exactly). Sequence and frame-group id: free, 20120 to 20199.
- **Definition**: `NAME.def` beside the mesh: `name=`, `model=NAME.bin`, `anim=anim`, `scale=XZ,Y`, `ambient=`,
  `contrast=` (the sheet's values).
- **Draw** (allowed with the real target: pictures are your reference):
  `P E/fx.py sheet OUT.png def:PATH/NAME.def id:ID --job JOB [--height 100] [--close] --n 8`
  `P E/fx.py gif OUT.gif def:PATH/NAME.def id:ID --job JOB [--height 100] [--close]`
- **Check your own numbers**: `P E/fxtools.py style PATH/NAME.def JOB` (against the census; it reads only YOUR files).
- **The gate is the Teacher's**: `P E/fxtools.py gate ID PATH/NAME.def JOB [--height 100] --json gate.json --out RENDERDIR`.
  You do not run it or `compare` (they read the target's files). Its checks: life equal to the real one's (a
  projectile: loop period within 10%); faces within 25% or 24; colours at most real x 1.5 + 1; colour distance of
  the palettes at most 2.5; translucent share within 0.12; rest box within 6% of the largest dimension on each axis;
  largest posed dimension within 10%; then, at 10 moments from two sides: mask overlap at least 0.72 on average and
  0.45 in the worst cell, coverage at game size 0.80 to 1.25, tone difference at most 12 on average and 24 at worst.
  All provisional: they have met a ceiling and synthetic damage only.

## How to work

1. Work only in `{ROOT}/builds/NAME/` and `{ROOT}/render/NAME-*`. Scripted builds only;
   write files with your file-writing tool, never shell heredocs.
2. Order of work.
   a. **Read the sheet by script and by eye.** From the JSON: per frame, the posed box and mean alpha: plot them; they
      are the motion. From the pictures: the parts, and what each does over the life. Write `plan.md`: parts, sizes in
      units, which label each carries, each part's motion as keys (time in cycles, value), the face budget per part.
   b. **Rest mesh and palette**, drawn at the frame where the effect is largest, three sides, beside the real one.
   c. **Skeleton and sequence**: a first pass with the box and alpha curves right before any refinement of shape.
      Draw all moments beside the real one; at game size first.
   d. **Translucency and light**: alpha per part from the sheet's alpha values; the definition's ambient and contrast.
   e. **Shape refinement**, close.
3. Keep `notes.md`: per round what gave yours away and what you changed; your own `style` output.
4. At least six rounds, at most fourteen. Stop when you cannot name a difference you know how to remove.

## Not allowed

- Reading the target's model, sequence, frames or base, or those of the related effects named above: no
  `fx.py dump`, `fxtools.py reencode | compare | gate | sheet`, `census.py`, `summary.py grep`, `osrscache.py`,
  `animlib` loaders, nor anything under `E/real/`, `E/proof/`, `E/data/` for them. Drawing them is allowed and required.
- Copying, importing or tracing any real mesh or frame.
- Modifying anything under `tools/`, `author-pack/`, `client-build/`, `builds/research-*` or another build folder.
  If a tool fails or a rule seems wrong, report it; that is as valuable as the model.
- Sending anything to an outside service or repository.

## Report (to `{ROOT}/builds/NAME/REPORT.md`; it is all the Teacher will see)

- Paths: final `.bin`, `.def`, `skeleton.json`, `.anim.json`, the build script, `plan.md`, `notes.md`, the final
  sheets (game size and close, real beside yours) and moving picture.
- Counts of yours against the sheet: vertices, faces, colours, alpha values, labels, groups, frames, cycles.
- One line per round: what differed, what you changed.
- What still differs from the real effect, at rest and in motion, and how you checked.
- What the sheet did not tell you that you needed, and what it told you that made the task too easy.
- Which rules or tools were wrong, useless or missing; what you would tell the next effect author.
- How many build-draw cycles you ran.
