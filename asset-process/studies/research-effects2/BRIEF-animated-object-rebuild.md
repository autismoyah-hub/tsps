# Brief: rebuild a real ANIMATED OBJECT from its reference sheet

Status: first use. The tools are proven on three ceilings (real objects through our encoders: 0 differing pixels, PASS) and on
six damaged copies (all FAIL); no animated object has been rebuilt yet. Three real spot animations HAVE been rebuilt from
sheets under the sister brief and passed their gate (`builds/effect-rebuild-2206/`, `-1595/`, `-2008/`): read their `REPORT.md`
and `notes.md` as worked examples of the route. Nothing here has been judged by the owner.

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server (revision 239 cache).
The owner's standard: an asset must read at once as the game's own art, and within that be as well made as possible. Quality
matters; speed does not. Our method: measure real assets, REBUILD real ones held out, prove the rebuild cannot be told from the
real one, only then make originals. Lasting hazards and arena pieces of boss fights (pools, clouds, tiles that light, pillars
that fall) are not spot animations: they are scenery OBJECTS with a sequence. You are one of the first three authors to rebuild
one. Your object id, its folder under `targets/` and your build name NAME are in the task line that came with this brief.

## What is different about an animated object (EFFECTS2.md section 1 has the client code behind each line)

- **It lasts.** A spot animation ends at the first wrap of its sequence; an object plays its sequence for as long as it stands.
  The sequence's LOOP FIELD is its life: `loop_frames = n` plays every frame once and then repeats the LAST n frames for ever.
  n = all frames is a loop; n smaller is a spawn followed by an idle; n = 1 holds the last frame; no loop field = it plays once
  and then stands in its REST pose (the unanimated mesh). The gate checks the field; a rebuild that looks right for one pass and
  has the wrong field fails.
- **It starts at a random frame unless its definition says otherwise** (`randomstart=0` in the `.loc` = opcode 89). The sheet
  says which; `field-ID.gif` shows nine instances with the real rule.
- **It is lit from the SIDE**: `light(64 + ambient, 768 + 5 x contrast, (-50, -10, -50))`, once, in its rest pose. An up-facing
  floor face gets little light. The same mesh looks different as an object and as a spot animation: use the sheet's ambient
  and contrast and judge only in the `loc` job.
- **It has a footprint** (size in tiles; the mesh is centred on it), a placement shape (10 scenery, 22 floor decoration: the
  sheet says), may be walk-through, may follow the ground (`contour=0`), and can be placed at four rotations (the sheet's
  fourth row).
- Otherwise it is an effect: one mesh, labelled parts, a frame base and a classic sequence; vertex labels move, FACE labels
  fade; alpha 0 opaque .. 253, never 254; judged MOVING at game size.

## Read first, completely

1. `{ROOT}/studies/research-effects/EFFECTS.md` sections 1 and 4, and `BRIEF-effect-rebuild.md` there (the route).
2. `{ROOT}/studies/research-effects2/EFFECTS2.md` sections 1, 2, 5 and 6.
3. `{ROOT}/studies/research-skeleton/SKELETONS.md` sections 1, 3, 4 and the docstrings of `skelc.py`, `skelanim.py`.
4. Your sheet in `targets/loc-ID/`: `sheet-ID.png` (rows: game size with a player two tiles away; close; camera a quarter turn
   round; the OBJECT at rotation 1; from above; level with the ground), `motion-ID.gif` (close), `motion-ID-quarter.gif` (two
   sides at once), `motion-ID-game.gif`, `field-ID.gif`, and `sheet-ID.json` (definition; counts; palette with surface shares;
   alpha values; rest box; the sequence: holds, loop_frames, idle cycles, per frame the posed box, top, bottom and mean alpha;
   transform kinds; groups in its base).
5. `{ROOT}/studies/research-effects2/set2/build.py` (function `star_pool`) and `set2/build_bl.py`: a worked
   example of an object with a spawn-then-idle sequence and a second sequence, built with the kit. An example of the TOOLS.

## The three targets

| id, name | what it tests | definition | mesh | sequence |
|---|---|---|---|---|
| 11700 snakeboss_poisoncloud (Zulrah's venom cloud) | soft mass as an object: translucent lumps that breathe | 3 x 3 tiles, shape 10, walk-through, RANDOM start, follows the ground, ambient 0, contrast 0, scale 160 on all axes, 1 recolour (paint the sheet's colour directly) | 240 vertices, 456 faces, one colour (30, 7, 20), every face alpha 208, none flat, 3 vertex labels, 3 face labels; box x -120..120, height 10..134, z -122..118 (before the scale) | 17 frames of 3 = 51 cycles, loop 17; scale only; 9 groups |
| 33035 tob_sotetseg_lighttile | an arena tile that lights: a floor piece built by alpha and translate | 1 x 1, SHAPE 22 (write `#shape=22` as a comment line in your `.loc`), walk-through, random start, ambient 5, contrast 5, offset 0, -2, 0 | 77 vertices, 244 faces, 3 colours ((0, 7, 80) 62%, (0, 0, 16) 27%, (0, 6, 33) 11%), alpha 150 / 185 / 220 on 62% of the surface, 44 flat faces, 5 vertex labels, 4 face labels; box 128 x 25 x 128 | 30 frames of 2 = 60 cycles, loop 30; translate and alpha; 9 groups |
| 51046 pmoon_boss_blood_pool (Moons of Peril) | a hazard with a SPAWN and an IDLE in one sequence | 1 x 1, shape 10, walk-through, FIXED start, ambient 30, contrast 0 | 1,675 vertices, 1,286 faces, 9 colours (four reds 30 / 41 / 51 / 64 light cover 96%), alpha 25..125 on 34% of the surface, 384 flat faces, 71 vertex labels, 52 face labels; box x -255..255, height -48..88, z -191..299 (the drops start far out) | 28 frames: 20 of 3 then 8 of 4 = 92 cycles, LOOP 8 (the idle is the last 32 cycles); translate, scale, alpha; 188 groups |

The JSON's numbers win over this table if they differ. 51046 is the hard one: a group budget of 255, 71 labels.
You may DRAW any object or effect by id. You may NOT read as meshes: your target's model, sequence, frames or base, nor any
object or spot animation on the same model or sequence (51046: model 51252, sequence 10966; 11700: model 4086, sequence 1261
"misty" and every object on it; 33035: model 35626, sequence 8143). The Teacher has not listed every sharer: if you are
unsure whether something shares them, draw it and do not read it.

## Tools (Git Bash; `P` = `{PYTHON} -I -B`; `E2` = `{ROOT}/studies/research-effects2`; `S` = `{ROOT}/studies/research-skeleton`)

- **Mesh**: by script. Either `E2/softkit.py` inside Blender (`E2/fxkit.py` `blender(script)` runs it; builders for lumps,
  shells, pools, falloff discs, rings; game units, y down) or `S/meshkit.py`. Whole units.
- **Skeleton, sequence, definition**: `fxkit.finish(outdir, NAME, mesh.json, skeleton, clip, kind='loc', ids=dict(base=B, seq=Q),
  ambient=.., contrast=.., size='X,Y', randomstart=0|1, contour=0)` compiles the skeleton, saves the model, samples the clip and
  writes `NAME.loc`. Base id: free, 2740 to 2799; sequence and frame-group id: free, 20230 to 20299; say which. The loop field
  is `sequence=dict(loop_frames=N)` in the clip; holds are `lengths=[..]` exactly as the sheet's.
- **Draw** (allowed with the real target: pictures are your reference):
  `P E2/ob.py sheet OUT.png locf:PATH/NAME.loc loc:ID --n 8 [--close] [--rot 1] [--life CYCLES]`
  `P E2/ob.py gif OUT.gif locf:PATH/NAME.loc loc:ID [--close] [--life CYCLES]`
  `P E2/ob.py random OUT.gif PATH/NAME.loc` (nine of yours with YOUR start rule)
- **Check your own numbers**: `P E2/style2.py locf:PATH/NAME.loc object` (reads and draws only yours).
- **The gate is the Teacher's**: `P E2/obtools.py gate ID PATH/NAME.loc --json gate.json --out RENDERDIR`. You do not run it,
  `compare`, `reencode` or `sheet` (they read the target's files). Its checks, on ONE PASS of the sequence at ten moments from
  two sides, close and at game size: life equal; faces within 25% or 24; colours at most real x 1.5 + 1; colour distance of the
  palettes at most 2.5; translucent share within 0.12; rest box within 6% of the largest dimension on each axis; largest posed
  dimension within 10%; mask overlap at least 0.72 on average and 0.45 in the worst cell; coverage at game size 0.80 to 1.25;
  tone difference at most 12 on average and 24 at worst; AND loop field equal, start rule equal, size in tiles equal, ambient
  and stored contrast within 10, walk-through equal. All provisional.

## How to work

1. Work only in `{ROOT}/builds/NAME/` and `{ROOT}/render/NAME-*`. Scripted builds only; write
   files with your file-writing tool, never shell heredocs.
2. Order of work.
   a. **Read the sheet by script and by eye.** Plot the per-frame box and mean alpha: they are the motion. Mark where the loop
      starts (frame count minus loop_frames) and what differs between the spawn and the idle. Write `plan.md`: parts, sizes,
      labels, each part's motion as keys, the face budget per part, the group budget.
   b. **Rest mesh and palette** in the `loc` job at the object's light, three sides and from above, beside the real one, at the
      frame where it is fullest.
   c. **Skeleton and sequence**: box and alpha curves right before any refinement; the loop seam (last frame to first idle
      frame) must not jump: draw two idles in a row (`--life`).
   d. **Translucency and light**; **the rotation row** (rotation 1 must look as the sheet's fourth row: the light does not turn
      with the object).
   e. **Shape refinement**, close.
3. Keep `notes.md`: per round what gave yours away and what you changed; your `style2.py` output.
4. At least six rounds, at most fourteen. Stop when you cannot name a difference you know how to remove.

## Not allowed

- Reading the target's model, sequence, frames or base, or those named above: no `ob.py dump`, `obtools.py`, `objcensus.py`,
  `ids.py`, `pal.py`, `soft.py` on them, `osrscache.py`, `animlib` loaders, nor anything under `E2/real/`, `E2/proof/`,
  `E2/data/` for them. Drawing them is allowed and required.
- Copying, importing or tracing any real mesh or frame.
- Modifying anything under `tools/`, `author-pack/`, `client-build/`, `builds/research-*` or another build folder. If a tool
  fails or a rule seems wrong, report it; that is as valuable as the model.
- Sending anything to an outside service or repository.

## Report (to `{ROOT}/builds/NAME/REPORT.md`; it is all the Teacher will see)

- Paths: final `.bin`, `.loc`, `skeleton.json`, `.anim.json`, the build scripts, `plan.md`, `notes.md`, the final sheets (game
  size and close, real beside yours, rotation 0 and 1) and moving pictures (one pass, and two idles in a row).
- Counts of yours against the sheet: vertices, faces, colours, alpha values, labels, groups, frames, cycles, loop field.
- One line per round: what differed, what you changed.
- What still differs from the real object at rest, in the spawn, in the idle and at the loop seam, and how you checked.
- What the sheet did not tell you that you needed, and what it told you that made the task too easy.
- Which rules or tools were wrong, useless or missing; what you would tell the next author.
- How many build-draw cycles you ran.
