# Brief: an original effect set for one creature's fight

Status: PROVEN ONCE IN PART. A first set for the Armillary sentinel (five effects, made before any measurement of softness)
was graded Strong by the owner and read at one watching; its maker's own criticism was that it was too hard-edged, its
telegraph four times a real one's area, its pool a spot animation that cannot last. A second pass of the same set was made
under the rules below (`studies/research-effects2/set2/`: the worked example); the owner has NOT yet judged the second pass.
Three real effects have been rebuilt from sheets and pass the gate. So: the tools are proven, the rules are measured on 55 real
effects, the taste is the owner's and still to be confirmed.

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server (revision 239 cache).
The owner's standard is the game's end-game fights (the raids, Nex, the Nightmare, the Doom of Mokhaiotl) and, in the owner's
words for other assets, [quoted words withheld]. Quality matters; speed does not.
An effect set is what a fight shows that is not the boss or the room: a projectile, its impact, a warning on the floor, the hit
that follows, a hazard that stays. Your creature, its palette, the attacks to dress and your build name NAME are in the task
line that came with this brief. You never copy, import or trace a real effect's mesh or frames.

## What an effect is here (two sentences each; the documents have the proof)

- A **spot animation** is one mesh and one sequence; the PACKET makes it a projectile (flies -z first, loops), an effect on an
  actor or on a tile (removed at the first wrap of its sequence: it never loops). Lit once at rest, from above.
- An **animated object** is scenery with a sequence: it lasts until removed, loops by its sequence's loop field, starts at a
  random frame unless told not to, is lit from the SIDE. Anything that must stay (a pool, a cloud, a lit tile) is an object.
- There are no textures and no blending but per-face alpha (0 opaque .. 253; never 254). Motion is scale, turn, move and fade
  of LABELLED parts of a mesh that never changes. It is judged MOVING, at game size, where it is 100 to 2,500 pixels.

## Read first, completely

1. `{ROOT}/studies/research-effects/EFFECTS.md`: sections 1, 3, 4 (what effects are; 58 real ones by job with
   their on-screen numbers; twelve rules).
2. `{ROOT}/studies/research-effects2/EFFECTS2.md`: sections 1 and 2 (objects), 5 (how softness is made), 6 (the
   kit), 7 (the style check), 8 (a fight as data).
3. `{ROOT}/studies/research-effects2/set2/`: `build_bl.py` (the meshes, in Blender with the kit), `build.py`
   (skeletons, clips, definitions), `notes.md` (five rounds: what was wrong each time), `round.py`. This is your worked example.
4. `{ROOT}/studies/research-skeleton/SKELETONS.md` sections 1, 3, 4 and the docstrings of `skelc.py` and
   `skelanim.py` (a frame base and a sequence of your own, by joint name).
5. The docstrings of `softkit.py`, `fxkit.py`, `fx.py`, `ob.py`, `style2.py`, `fight/fight.py` in `studies/research-effects2/`.
6. The three effect rebuilds that passed, for how real ones are actually built: `{ROOT}/builds/effect-rebuild-2206/`
   (a projectile of nested translucent shells), `-1595/` (a floor mark), `-2008/` (an impact that grows 17 times): `REPORT.md`.

## The rules, by job, with numbers (medians of real effects of the reference fights; ranges in the documents)

All jobs: SMOOTH shading on everything that is mass (real: 42 of 55 wholly smooth); flat shading only for the creature's metal,
crystal, shards. Mass = many small rounded pieces (12 to 30 faces each, radius under 20 units at rest, 10 to 20 pixels on
screen), never one big faceted solid. Translucency in STEPS: an opaque or near-opaque core, 2 or 3 shells or rings at alpha
about 100, 150, 200 outward. Fade in time: come in from alpha 250 or grow from 10 to 30%; end by fading, not by shrinking alone.
3 or 4 colours close in tone; the creature's own palette; one crisp solid that is unmistakably the creature's, at most two.

| job | size on screen (pixels at largest; a player is 1,100) | life | must | built from |
|---|---|---|---|---|
| projectile | 150 to 400 (boss projectiles 73 to 741) | a looping sequence of 20 to 60 cycles; the flight is the server's (50 cycles over 6 tiles is typical) | be told from an arrow by colour and glow, not size; front is -z, centred on the origin; 45 to 200 units long | `glow_shell` core, the creature's solid, `streaks` behind (alpha 40 at the root to 220 at the tips) |
| impact (on an actor, height about 100) | 600 to 800 | 30 to 60 cycles, holds of 2 to 3 | start from nothing (first frame under 60 pixels), peak in the last third, end by alpha | a `glow_shell` flash that grows 2.5x in 5 cycles, a `lump_cloud` puff that rolls open and fades, small sparks |
| telegraph (on a tile) | 430 to 600 for one tile (116 units across); QUIET: distance from the scene 18 to 60 | the warning in game ticks x 30 cycles (2 to 10 ticks; 3 or 4 is usual) | grow or fill over its whole life, fast then slow; 1 or 2 colours; mostly translucent; up-facing faces at height 1 to 3 | `disc_falloff` (alpha 120 to 230 from centre to edge) that scales from 15% to full; one thin line of the creature's colour at the final edge |
| hit (same tile, the cycle the telegraph ends) | about 3 times the telegraph: 1,400 to 2,900 | 30 to 84 cycles | start LARGE and bright on its first frame; leave soft mass behind (dust, smoke) that outlives the flash | a `glow_shell` column or burst, a `disc_falloff` ground flash, a `dust_burst` thrown by scale about the centre, opaque then faded from 45% of the life |
| hazard that stays (an OBJECT, `.loc`) | 215 to 1,150 a tile | spawn 30 to 60 cycles, then an idle of 30 to 90 that loops (`loop_frames` = idle frames); a second sequence to despawn that ends invisible and holds (`loop_frames` 1) | walk-through, fixed start (`randomstart=0`), follow the ground (`contour=0`), ambient 20 to 40; a POOL is opaque and reads by shape (lobes, drips, a low dome 3 to 12 high, tone rings dark centre to bright rim), not by a soft edge | `pool`, or `lump_cloud` in 3 sets that breathe 10% out of step |

Counts that hold for all: 50 to 500 faces a projectile, 180 to 1,100 an impact, telegraph or hazard; a boss's signature hit to
2,000. A frame base has at most 255 groups: a joint with scale, move and alpha costs 4; share joints between sets of lumps.

## The kit and the route

`softkit.py` runs inside Blender; you write `NAME/build_bl.py` (meshes; game units, y is DOWN, a tile is 128) and
`NAME/build.py` (skeleton, clip, definition through `fxkit.finish`), exactly as `set2/` does. `P` =
`{PYTHON} -I -B`; `E2` = `{ROOT}/studies/research-effects2`.

- Mesh: `K.Kit()`, then `lump`, `lump_cloud`, `glow_shell`, `streaks`, `trail`, `dust_burst`, `pool`, `disc_falloff`, `torus`,
  `shard`, `star4`, `anchor`; `kit.export('mesh/X.json')`. Every part takes a vertex label NAME and a face label NAME.
- Skeleton: joints own vertex-label names; `{'kind': 'alpha', 'faces': [..]}` channels fade face-label names. A part thrown
  outward is scaled about an `anchor` at the centre; a part that grows in place pivots on its own label.
- Clip: keys per JOINT.CHANNEL; alpha is `[n, 0, 0]`, 8 per step, 32 = gone, negative brings in a part modelled at alpha 250.
  A projectile: `loop=True, sequence=dict(loop_frames=FRAMES)`. An object: `sequence=dict(loop_frames=IDLE_FRAMES)`.
- Ids: bases 2740 to 2799, sequences 20230 to 20299, free; say which you took. Spot and object ids under 65,536.
- Draw: `P E2/fx.py sheet|gif OUT EFFECT.. --job proj|actor|spot [--height 100] [--close]` with `def:PATH` (yours) and `id:N`
  (any real one); `P E2/ob.py sheet|gif|random` for `locf:PATH` and `loc:N`.
- Style check, EVERY round: `P E2/style2.py def:PATH KIND [--height 100]` or `locf:PATH object`; KIND = projectile, impact,
  telegraph, hit, hazard. An OUT line is a question you must answer in your notes, not a verdict.
- The fight: write `NAME/fight.json` (format in `E2/fight/fight.py`'s docstring; `E2/fight/sentinel.json` is the example),
  `P E2/fight/fight.py check`, then `play` each attack and `emit`. `check` must be clean: a telegraph's sequence must last
  warnTicks x 30 cycles; a projectile must land within a tick of its hit.

## Study before designing

For EACH effect choose two real effects of the same job from the reference fights (EFFECTS.md section 3 lists 58 by job;
EFFECTS2.md section 2 lists objects) and draw them at game size and close. Write in `design.md` BEFORE building: for each real
one its pixels, life, colours, what its mass is made of and how it ends (`P E2/soft.py one id:N JOB` prints it); then your set
in words: the creature's idea in two words; its palette with each colour's job; the ONE solid that marks the set as this
creature's and where it appears in each effect; per effect: parts, sizes in units, labels, motion as keys, face budget; the two
attacks as beats in ticks.

## How to work

1. Work only in `{ROOT}/builds/NAME/` and `{ROOT}/render/NAME-*`. Scripted builds only; write
   files with your file-writing tool, never shell heredocs.
2. Rounds (copy `E2/set2/round.py` and change its table): each round draws every effect of yours at game size and close BESIDE
   its two real effects, at six moments and as a moving picture at the game's timing, and prints pixels, distance and the
   softness line for all three. At least FIVE rounds an effect. LOOK at every sheet; a round you did not look at does not count.
   If you can only inspect stills, say so in the report.
3. Each round, in `notes.md`: what gave yours away beside the real ones (one line an effect), what you changed, the style lines.
4. Order: sizes and timing first (pixels and life inside the job's range), then mass and translucency, then the creature's
   solid, then colour. Then the fight: play both attacks and look at the set TOGETHER: it must read at one watching (who is
   attacked, where not to stand, when it lands) and every effect must belong to the creature.
5. Stop when you cannot name a difference you know how to remove.

## Not allowed

- Reading any real effect's or object's model, sequence, frames or base as a mesh (no `dump`, `reencode`, loaders). Drawing and
  `soft.py one` / `pal.py` numbers on real ones are allowed.
- Modifying anything under `tools/`, `author-pack/`, `client-build/`, `builds/research-*` or another build folder (copy a tool
  to change it). If a tool fails or a rule seems wrong, report it.
- Sending anything to an outside service or repository.

## Report (to `{ROOT}/builds/NAME/REPORT.md`; it is all the Teacher will see)

- Paths: per effect the `.bin`, `.def` or `.loc`, `skeleton.json`, `.anim.json`; the two build scripts; `design.md`, `notes.md`;
  the final round's sheets and moving pictures; `fight.json`, the two played attacks, the emitted server files.
- A table: per effect faces, pieces, smooth share, translucent share, alpha values, pixels at largest, distance, life, edge
  ratio, beside the same numbers of its two real effects.
- The final `style2.py` output for each effect, and your answer to every OUT line.
- One line per round and effect: what differed, what you changed.
- Your honest judgment of each effect beside its two real ones: what still gives it away.
- Whether you watched motion or inspected stills.
- Which rules or tools were wrong, useless or missing.
