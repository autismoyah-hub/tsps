# Effects, second part: animated objects, soft mass, a fight as data (revision 239 cache)

Private to this machine. Continues `../research-effects/EFFECTS.md` (read that first: sections 1 to 4). Everything here was
read from the client's source (`client/rust-client/src/`, read only) or measured by the scripts named.
`P` = `{PYTHON} -I -B`, run from `{ROOT}/studies/research-effects2/`.
`BL` = `"{BLENDER}" -b --python-exit-code 1 --python`.

```
P objcensus.py | summary | grep REGEX     every object with a sequence -> data/objanim.jsonl, data/objanim-summary.txt
P ob.py dump|faithful|sheet|gif|random    animated objects through the harness (section 3)
P obtools.py reencode|sheet|compare|gate|lineup      the rebuild test for animated objects (section 4)
P soft.py measure | one EFFECT JOB        how softness is made (section 5);  P softsum.py -> data/soft-summary.txt
softkit.py (inside Blender) + fxkit.py    the soft-mass kit (section 6);     P kitproof/kitproof.py build|draw|measure
P style2.py EFFECT KIND                   the style check for an original (section 7)
P fight/fight.py play|film|emit|check     a fight as data (section 8)
P set2/build.py [mesh] [a..e];  P set2/round.py N     the sentinel's set, second pass (section 9)
```

## 1. An object with a sequence: what the client does (`object.rs`, `world_mesh.rs`, `scene.rs`, `location_update.rs`, `location_animation.rs`, `live_runtime.rs`, `animation_runtime.rs`)

**Definition** (archive 2, group 6). The fields that matter for a hazard or an arena piece:

| opcode | field | notes |
|---|---|---|
| 1, 5, 6, 7 | models | a LIST (5 / 7) placed as shape 10, or pairs model + shape (1 / 6). With no shapes listed the object draws ONLY as shape 10 (11 = the same, turned 45 degrees); a floor decoration (shape 22) must list its model with shape 22 |
| 14, 15 | size in tiles | the model is centred on the footprint; a quarter turn swaps the two |
| 17 | walk-through | no collision and no projectile block: every pool, cloud, mark and light tile has it (1,298 of 5,408 animated objects) |
| 24 | sequence | what it plays from the moment it exists |
| 89 | fixed start | WITHOUT it (the default) each instance starts at a RANDOM frame and a random time inside that frame |
| 29, 39 | ambient, contrast | light = `light(64 + ambient, 768 + 5 x contrast, (-50, -10, -50))`: FROM THE SIDE, lower base contrast than a spot animation's `(850, (-30, -50, -30))`. The same mesh is lit differently as an object |
| 21 / 81 | contour | the posed mesh follows the ground under it (an up-facing pool on a slope) |
| 40, 41, 65-67, 70-72, 62 | recolours, scale x / height / z, offset, mirror | applied to the model once, before lighting |
| 22 | merged normals | ignored for an animated object (it is never baked with its neighbours) |
| 77 / 92 | transforms | the object is another object chosen by a variable: how one map object shows different states per player |

**How its sequence is chosen and started** (`world_mesh.rs` build and `advance`):
- at the moment the object enters the scene (map load, or `LOC_ADD_CHANGE`) the part gets a clock for opcode 24's sequence;
- `random_start = (no LOC_ANIM has been applied to it) and (no opcode 89)`, and then only if the sequence HAS a loop field:
  `frame = random x frame count`, `elapsed = random x that frame's hold` (`Clock::randomize`). Nine spores spawned in one tick
  are therefore out of step with each other by design; nine pools that must grow together need opcode 89;
- the clock advances one step per client cycle (20 ms), like everything else;
- **the loop field decides the life**: with `loop_frames = n` the sequence plays all its frames once and then repeats ITS LAST n
  FRAMES for ever (`next = frame - loop_frames`). n = frame count is an ordinary loop (4,065 of the 4,342 drawable animated
  objects set it). n smaller is [quoted words withheld]: the Moons of Peril blood pool `pmoon_boss_blood_pool` 51046 plays 28 frames
  (92 cycles) and loops the last 8 (32 cycles). n = 1 holds the last frame (Verzik's pillar 32688 collapses and stays down);
- **with NO loop field (277 objects) the sequence plays once and the object then stands in its REST pose** (the unanimated
  model: `flags & 9 == 9` stops the clock and the model is drawn unposed). A despawn that ends invisible must hold its last
  frame (loop 1) or the full object flashes back until the server removes it;
- the sequence's opcode 16 byte lifts the whole object while it plays, as it does an actor.

**How a server changes it** (zone packets; `live_runtime.rs` applies them in this order each cycle, then rebuilds the scene mesh
and carries every unaffected clock across, `retain_clocks`):

| packet | client call | effect |
|---|---|---|
| `LOC_ADD_CHANGE` (id, shape, rotation, tile) | `Scene::replace_location` | the object on that tile and layer (walls 0-3, wall decoration 4-8, scenery 9-21, floor decoration 22: one of each per tile) is REPLACED; the new one starts its own sequence (random or fixed start as above). A swap is this packet with another id |
| `LOC_DEL` | `replace_location(id -1)` | removed at once, whatever its sequence is doing |
| `LOC_ANIM` (sequence, shape, rotation, tile) | `Scene::animate_location` | the object's sequence is replaced by the one sent, started at frame 0 (never random). -1 stops it (rest pose). If the object is a pending spawn the animation is kept for when it appears. A sequence whose `restart` field (opcode 11) is 0 is not restarted when sent again |
| pending spawns | `location_update::Queue` | an added object can carry a delay and a lifetime in cycles after which the ORIGINAL object returns: the client's own timer (used with `LOC_MERGE`) |

Spawning over other scenery follows the scene's slot rules: a 5 x 5 object spawned over a bush hides the bush (seen in the proof,
section 3).

**Light and order.** Lit once, in its rest pose (at its placed rotation), tones kept through every frame; animation is applied in
the model's unrotated frame (`rotate_quarter(4 - r)`, animate, rotate back). Painted like all scenery: per-pixel depth,
opaque faces first, translucent faces of all models after, far to near. Nothing blends but per-face alpha.

## 2. Census of animated objects (`P objcensus.py summary`; `data/objanim-summary.txt`)

5,408 of 62,194 object definitions name a sequence; 4,342 have a drawable model for their shape and a readable sequence;
1,232 distinct sequences, 2,144 distinct model sets; 5,300 classic sequences, 108 skeletal.

| | value |
|---|---|
| random start (default) / fixed start (opcode 89) | 5,047 / 361 |
| loop field set / plays once and rests | 4,065 / 277 |
| walk-through / with a menu option / 1 x 1 / more than one model | 1,298 / 1,662 / 2,991 / 1,218 |
| faces, 10 / 50 / 90 centile | 100 / 494 / 1,666 |
| largest dimension, units | 122 / 255 / 539 |
| colours covering 90% of surface | 1 / 4 / 9 |
| any translucent face / wholly translucent / any flat face / textured | 55% / 10% / 28% / 4% |
| cycles of one pass | 20 / 65 / 239 (frames 3 / 12 / 37) |
| ambient / stored contrast | 0 / 10 / 55 and 0 / 0 / 45 |
| classic sequences: transform kinds | translate 2,978, alpha 2,840, scale 2,481, rotate 2,391 of 4,242; only 359 grow 1.5x (objects mostly stay their size: spots mostly grow) |

By job, from the names (a name rule, so approximate):

| job | n | faces 10/50/90 | largest dim | translucent / wholly | loops | random start | walk-through | cycles 10/50/90 |
|---|---|---|---|---|---|---|---|---|
| portal / rift / barrier | 554 | 152 / 808 / 1,431 | 287 | 90% / 23% | 98% | 92% | 8% | 60 / 97 / 106 |
| fire / flame / brazier | 539 | 121 / 323 / 1,212 | 195 | 61% / 5% | 96% | 98% | 13% | 20 / 33 / 128 |
| machine / lever / door | 287 | 68 / 442 / 2,711 | 332 | 49% / 1% | 98% | 98% | 8% | 20 / 64 / 156 |
| crystal / orb / light | 275 | 84 / 406 / 1,355 | 223 | 65% / 20% | 97% | 90% | 28% | 25 / 64 / 128 |
| water / fountain / fall | 264 | 32 / 370 / 1,298 | 245 | 65% / 20% | 97% | 100% | 38% | 19 / 64 / 300 |
| tile / floor mark / symbol | 195 | 103 / 456 / 1,535 | 221 | 35% / 7% | 73% | 89% | 56% | 10 / 60 / 10,053 |
| pool / liquid / acid | 144 | 72 / 296 / 1,178 | 200 | 49% / 9% | 98% | 95% | 22% | 32 / 41 / 175 |
| pillar / rock / falling | 135 | 118 / 462 / 1,059 | 221 | 38% / 2% | 93% | 96% | 17% | 2 / 81 / 259 |
| creature part / tentacle / egg | 126 | 225 / 510 / 2,032 | 290 | 48% / 3% | 92% | 98% | 3% | 17 / 56 / 193 |
| smoke / gas / cloud / vent | 114 | 32 / 378 / 3,183 | 253 | 85% / 53% | 99% | 94% | 34% | 19 / 80 / 320 |

**Verified examples from the owner's reference fights** (ids from the server copy's boss plugins and data; each read from the
cache by `P ids.py ID`; those marked * drawn through the harness):

| what | object | made of | sequence | notes |
|---|---|---|---|---|
| Moons of Peril blood pool * | 51046 | 1,286 faces, 375 pieces, alpha 25..125 on 34% | 10966, 28 frames 92 cycles, LOOP 8 | fixed start, walk-through; drops fall, the pool grows, then an idle of 32 cycles: THE pattern for a hazard |
| Nightmare spore * | 37738 then 37739 | 688 faces | 8629 (one frame: a held small pose), LOC_ANIM 8630 spawn, swap to 37739 idle 8631 (60 cycles), LOC_ANIM 8632 burst | random start; the server arms it at 5 ticks and bursts all at 30 |
| Sotetseg's light tile * | 33035 | 244 faces, shape 22, alpha 150 / 185 / 220 | 8143, 60 cycles | plain and dark tiles 33033 / 33034 are static: the maze is object SWAPS |
| Zulrah's venom cloud * | 11700 | 456 faces, 6 pieces, alpha 208 everywhere, 3 x 3, scale 160 | 1261 "misty", 51 cycles, scale only | soft mass as an object; 30 ticks on the server |
| Tekton's fire * | 30021 | 964 faces, 2 x 2 | 6645, 65 cycles | solid |
| Whisperer leech seed * | 47573 | 1,282 faces | 10270, 90 cycles | sent per player |
| Great Olm's head and hands * | 29881 / 29884 / 29887 | two models each, 5 x 8 tiles, 1,213 to 2,543 faces | idle 7336 / 7355 / 7351, 60 cycles | the boss IS objects; every attack is a LOC_ANIM |
| Verzik's collapsing pillar * | 32688 | 3,500 faces, 5 x 5 | 8052, 120 cycles, loop 1 | swapped in for the static pillar 32687 |
| Wardens' pyramid * | 45748 | 964 faces, 3 x 3, contoured | 9525 appear, 30 cycles, loop 1 | fixed start |
| Yama's glyphs | 56335 / 56336 / 56337 | 3 x 3 floor glyphs, wholly translucent | activate 30 cycles, loop 1 | swapped lit / out / inactive |
| Royal Titans' safe tile * | 56003 | 1,328 faces, shape 22 | 12015, 390 cycles, loop 1 | a 13-tick timer drawn as one object |
| Amoxliatl's ice * / Hueycoatl's tiles * | 54279 / 55209 | 1,282 / 2,016 faces | 60 / 120 cycles | 55209 has no loop field: it plays once and rests |
| Hydra's vents | 34568-34570 | 216 faces, alpha 128 | 8280 "off" (one frame) | the geyser is a LOC_ANIM |
| Muspah's spikes | 46693 / 46694 / 46697 | 270 faces | skeletal, 30 to 60 cycles | pre, post, homing: three objects on one model |
| NOT animated (static objects that appear and go) | Olm's acid 30032, Doom's acid 57283 and rock 57286, Maiden's blood 32984, Vorkath's acid 32000, Araxxor's acid 54148, Verzik's acid 41747, Nex's shadow 42942 and ice 42943 / 42944, Kephri's dung 45504 | | | most lasting pools in the game are STATIC models; the motion is the spot animation that lands on them |
| also one frame held | Xarpus acid 32744, Hydra pool 34511 (sequence 8067 [quoted words withheld]) | 281 faces, opaque | 1 frame | a sequence used as a POSE |

**What makes them read** (measured on the eight drawn in `data/soft.txt`, rows `object`): 215 to 1,142 pixels at game size for a
one-tile piece; edge ratio 0.59 (softer than spot-animation hazards, 0.93); 60% of their surface translucent at the median;
pieces 38; they keep their size (a looping breath of 5 to 20%), where a spot animation grows and goes.

Rules for an animated object:
1. Decide the life first: **loop** (frame count), **spawn then idle** (loop = idle frames), **hold** (loop 1), or once-and-rest.
2. Fixed start (opcode 89) whenever the sequence has a beginning (a spawn, a rise, a timer); random start only for pure idles.
3. Walk-through (opcode 17) for anything a player stands in; size in tiles = the area that hurts.
4. Build it for the SIDE light: an up-facing floor piece gets little light (direction y is -10 of 51): give it ambient 20 to 40;
   domes and lumps shade themselves.
5. A despawn is its own sequence, sent by LOC_ANIM before LOC_DEL; it ends invisible and HOLDS (loop 1).
6. One object per tile and layer: a hazard on a tile that already has scenery replaces it. Use shape 22 (list the model with
   shape 22) for floor pieces that must coexist with scenery, shape 10 otherwise.

## 3. The harness for objects (`ob.py`, `fx.py` job `loc`, `client/rust-client/examples/asset_arena.rs`)

`asset_arena` = `asset_effect` plus objects (regenerate with `P rust/patch_arena.py`, build with
`cargo +stable-x86_64-pc-windows-gnu build --release --example asset_arena`). By id it makes the client's own calls
(`replace_location`, `animate_location`, `WorldMesh::build`, `retain_clocks`, `advance`, one step a cycle); from files it repeats
`object_model.rs` and `world_mesh.rs` on loose files (`.loc`: model, sequence directory, ambient, contrast, scale, offset,
recolours, size, mirror, randomstart, contour). Script lines `loc`, `locanim`, `floc`, `flocanim`, `pin`, `shake` are in the
example's header. `pin` gives every animated object already in the map a LOC_ANIM of its own sequence, so the stage is the
same in every run (the map's own objects would otherwise start at random).

Proof (`P ob.py faithful 37739 33035 30021 11700 47573 32688 29881 29884 56264 45748 54279 47146 56003 32755 55209 30024`;
log `render/research-effects2-faithful/faithful-objects.out`): 16 real animated objects dumped to loose files with their own
bytes and drawn through the file path against the same objects by id, two rotations, close and game size, ten moments each.
**15 objects: 0 differing of 115,200,000 pixels over 600 frame pairs.** The sixteenth (Verzik's 5 x 5 pillar) differs by 4,380
pixels a frame, all inside one map bush that the client removes under the object's footprint when it is spawned BY ID and the
file path leaves standing (`diff-32688.png`): a difference of the stage, not of the object. First run: 3 objects differed; the
Wardens' pyramid by 387 pixels a frame on face edges: the file path had no ground contouring (opcode 21). Added; 0.
`P locanim_proof.py`: the Nightmare's spore as the server scripts it (appear, LOC_ANIM spawn, swap to the armed object,
LOC_ANIM burst, remove) by id against files: **0 differing of 20,736,000 pixels over 108 frame pairs.**
Not proven: the random start (the file path takes the two random numbers as `draws=`; the by-id path uses the client's own
generator, so pixels cannot be compared); skeletal sequences from files (108 objects draw by id only); several models combined
(the Olm's head: 0 pixels close, 14 at game size where it overlaps the player).

## 4. The rebuild test for animated objects (`obtools.py`)

`reencode ID DIR [damage]`, `sheet ID DIR`, `compare | gate ID OURS.loc`, `lineup OUT.gif EFFECT..`. The gate is fxtools' gate on
one pass of the sequence plus the object's own lines: loop field, start rule, size in tiles, ambient and contrast within 10,
walk-through. Proof (`proof/`, `proof/degraded.log`, pictures `render/research-effects2-proof/`):

| case | result |
|---|---|
| ceiling, 11700 Zulrah's cloud through our encoders | PASS; 0 differing of 2,160,000 pixels |
| ceiling, 33035 Sotetseg's light tile (shape 22) | PASS; 0 of 2,160,000 |
| ceiling, 51046 Moons blood pool (spawn then idle) | PASS; 0 of 2,160,000 |
| 11700 with every scale transform removed | FAIL: overlap 0.669, coverage 1.469 |
| 11700 stretched 130% | FAIL: box, posed size 312 against 240, overlap 0.561, coverage 1.698 |
| 33035 opaque | FAIL: translucent share 0 against 0.619, tone 16.0 |
| 51046 with the loop field dropped | FAIL on the loop line ONLY (-1 against 8): one pass is pixel-identical; without that line the gate would pass an object that rests after 92 cycles |
| 51046 at two thirds speed | FAIL: life 128 against 92, overlap 0.547, tone 29.8 |
| 51046 lightness +14 | FAIL: tone 16.0 |

Targets and sheets: `targets/loc-11700/`, `loc-33035/`, `loc-51046/` (`sheet-ID.png` six rows, `sheet-ID.json`,
`motion-ID.gif`, `motion-ID-quarter.gif`, `motion-ID-game.gif`, `field-ID.gif`); brief `BRIEF-animated-object-rebuild.md`.
No rebuild has been run.

## 5. How softness is made (`P soft.py measure`, `P softsum.py`; `data/soft.txt`, `data/soft-summary.txt`)

55 real effects and animated objects of the reference fights (15 projectiles, 8 impacts, 8 auras, 7 ground hazards, 6
telegraphs, 3 hits, 8 objects: Nex, the Nightmare, Verzik, the Olm, the Wardens, Kephri, Doom, Yama, Vorkath, Hydra, Duke,
Zulrah, the Royal Titans, the Whisperer). From the files: pieces (connected parts), nesting, alpha by distance from the centre,
facets, motion. On screen (close lens, the fullest moment): `edge ratio` = how much the effect changes the picture one pixel
inside its outline, over how much it changes it in its core (1.0 = a cut-out).

Format facts: no textures in effects (7 of 4,008), no blending but per-face alpha, lit once at rest. So softness is made of
exactly five things:

1. **Smooth shading.** 42 of 55 are 100% smooth-shaded; median share 1.0, lowest 0.18. The first-pass originals: 0.00 to 0.16.
   A flat-shaded face is a facet of crystal or metal; mass is never flat.
2. **Small rounded pieces, many of them.** Median 13 pieces an effect; 22 of 55 are clouds of 20 or more pieces of 12 faces
   each at the median (3 to 49), 98% of the pieces under a fifth of the whole. Doom's slam: 75 lumps of 27 faces. A lump is
   small ON SCREEN (10 to 20 pixels): the eye cannot find its facets. The same 20-face lump blown up to 100 units is a rock.
3. **Shells and steps of alpha.** 29 of 55 nest pieces inside pieces (depth 1: 15, 2: 5, 3: 7). Innermost level: opaque (median
   alpha 0); outermost: median 45, up to 208. The translucent values in use: 32 / 105 / 208 at the 10th, 50th and 90th centile;
   the commonest are 100, 150, 50, 75, 200, 64, 125. Within one effect 2 values at the median, up to 12 (the Nightmare's and
   Nex's magic projectile: tongues stepped 16, 32 .. 240 from root to tip). Alpha rises with distance from the centre in 16 of
   the 28 translucent ones (thinner outside), falls in 6 (a thin core inside a denser rim: bubbles, shields). The falling-rock
   shadow is the pure case: one flat disc, six rings 130 to 230, rank correlation of alpha with radius 0.97, edge ratio 0.32.
4. **Fade in time instead of in space.** 10 of the 17 effects that are wholly opaque at rest animate alpha: the dust of Doom's
   slam is opaque lumps that go from alpha 0 to 128 as the cloud grows from 128 to 648 units. Of the 16 that grow 1.5x or more:
   growth 3.2x at the median, the size peaks at 0.79 of the life, mean alpha runs 107, 137, 163 (first, at the peak, last).
5. **Few colours, close in tone.** 4 colours at the median. Lightness is not a core-to-edge rule (lighter inside in 10, lighter
   outside in 15 of 55): the bright rim is as common as the bright core. What is constant is that neighbouring pieces differ
   by small steps (the blood pools: lightness 30, 41, 51, 64 from centre to rim).

What softness is NOT: ground hazards are hard. Pools are opaque (0% translucent at the median), edge ratio 0.93, the most
saturated thing on the floor. They read as liquid by SHAPE (lobes, drips, a low dome, tone rings), not by a soft edge. The
earlier criticism [quoted words withheld] is right about the shape and wrong if it is read as [quoted words withheld].
A trail is not a fading ribbon either: 12 of 15 projectiles carry faces well behind the origin, 11 to 212 of them; only 5
thin toward the end and 4 fade toward it; the tail is tongues, shards or lumps, mostly opaque (median alpha near / far 11 / 44).

On screen by job (median edge ratio / core change / tonal levels): projectile 0.64 / 64 / 5; impact 0.54 / 59 / 5; aura 0.70 /
64 / 4; ground hazard 0.93 / 73 / 3; telegraph 0.56 / 36 / 3.5; hit 0.86 / 31 / 5; object 0.59 / 50 / 4.

## 6. The kit (`softkit.py` inside Blender; `fxkit.py` outside)

`softkit.Kit` builds in GAME units and axes (y down, -z forward); every face smooth unless asked; faces wound so the cross
product points out of the piece (the first kit pool was invisible and its lumps lit from inside until this was found).
`export` writes JSON; `fxkit.finish` compiles a skeleton (`research-skeleton/skelc.py`), saves the model, samples the clip
(`skelanim.py`) and writes a `.def` or a `.loc`. Label NAMES are shared between the mesh and the skeleton and checked.

| builder | makes | default numbers (from section 5) |
|---|---|---|
| `lump`, `lump_cloud` | rounded irregular lumps, 20 faces each (80 at level 2), in label sets | radius under 20 at rest; 2 or 3 tones by height; alpha thinner outward |
| `glow_shell` | a core and 2 to 4 closed shells | core alpha 0 to 40; shells 110, 160, 200; each 1.35x the last |
| `streaks`, `trail` | tapering tongues behind a projectile | alpha 40 at the root to 220 at the point, 3 segments |
| `dust_burst` | a ring of lumps on the floor in sector sets | thrown by scale about an anchor; opaque, then faded from 45% of the life |
| `pool` | lobed outline, 3 tone rings, bright rim, translucent skirt, ripples, bubbles, drips | r 62 = one tile; dome 3 to 12; skirt alpha 90 / 170 |
| `disc_falloff` | flat rings stepping in alpha | 130 to 230 in six steps |
| `torus`, `shard`, `star4`, `anchor` | the few crisp solids; an undrawn pivot | flat shading for metal and crystal only |

Proof (`P kitproof/kitproof.py all`; `render/research-effects2-kit/NAME-pair.png` (six moments, close and game size),
`NAME-pair-close.gif`, `NAME-pair-game.gif`, `NAME-blind.gif` + key; numbers `data/kitproof.txt`). Each piece is the kit's own
geometry in the size and palette of a real effect of the same kind, not a rebuild:

| kit piece | real | kit / real: faces, edge ratio | what I saw in the stills |
|---|---|---|---|
| `lump_cloud` as an object, 3 sets breathing | 11700 Zulrah's cloud | 301 / 456; 0.47 / 0.67 | the same kind at game size; the kit's is denser in the middle and its lumps a little larger |
| `glow_shell` as a projectile | 1340 Olm's orb | 80 / 110; 0.16 / 0.38 | reads as the same glowing orb; the kit's halo is SOFTER than the real one (three shells against two) |
| `glow_shell` + `streaks` | 2007 Nex's finale projectile | 207 / 130; 0.31 / 0.60 | a comet with tongues, as the real; the kit's tail is twice as long |
| `dust_burst` | 3370 Doom's slam | 801 / 2,021; 0.78 / 0.53 | the same smooth "popcorn" lumps and fade; the kit's ring is regular and visible from the first frame, the real one erupts in two clumps late |
| `pool` as an object, spawn then idle | 51046 Moons blood pool | 467 / 1,286; 0.46 / 1.58 | tone rings and bright rim as the real; the kit's is lobed with a translucent skirt (softer edge than any real pool), the real is a regular disc |

## 7. The style check (`P style2.py EFFECT KIND [--height H]`)

For an original: pixels at its largest, distance from the scene, life, start (an impact from nothing, a hit large, a telegraph
small and growing over whole ticks), smooth share at least 0.6, no alpha 254, edge ratio under 0.95 (1.2 for a pool), at least 3
tonal levels; each against the study set's range for the job. It reads and draws only the author's files.

## 8. A fight as data (`fight/fight.py`; format in its docstring)

`fight.json` names every beat of an attack by asset name: `npc_anim`, `projectile` (start, end in cycles; heights in the server's
units; angle; progress), `impact`, `telegraph` (warnTicks), `tile_graphic`, `hit`, `object` (lifetimeTicks, despawn sequence,
tickDamage), `camera_shake`. `play` draws it (the creature and the player by id, ours from files or anything by id);
`check` finds beats that cannot be right (a telegraph whose sequence is not warnTicks x 30 cycles, a projectile that lands with
no hit within a tick, an id over two bytes); `emit` writes `NAME.server.json` in the owner's data dialect (nex.json /
nightmare.json keys: `anim`, `projectile {id, start, end, angle, progress, startHeight, endHeight}`, `impact {id, height, delay}`,
`warnSpot`, `warnTicks`, `hitTicks`, `maxHit`, `hazard`, `cameraShake`) and `NAME.server.js`, the plugin calls for each beat
(`core.Projectile(..).withAngle().withProgress().withSource().sendProjectile()`, `performGraphic`, `sendGlobalGraphic`,
`new core.GameObject` + `ObjectManager.register / deregister`, `GameObject.performAnimation`, `sendCameraShake`). Files:
`fight/sentinel.json` (two attacks, second pass), `fight/sentinel-first.json`, `fight/nex-reference.json` (Nex's smoke attack
and shadow smash by id with the owner's own data values), `fight/emitted/`. The camera shake in playback is a stand-in, not
the client's CAM_SHAKE code. The emitted JS has not been run: the server has no shared loader, every boss has its own plugin.

**What the server must do for an animated hazard object** (the star-pool; emitted in `fight/emitted/armillary-sentinel.server.js`):
1. the object definition, its model, its sequences and its frame base are in the cache both sides load, the object id and the
   sequence ids under 65,536 (every packet names them in two bytes);
2. `new core.GameObject(id, tile, 10, 0, area)` and `ObjectManager.register(obj, true)` on the hit's tick (LOC_ADD_CHANGE to
   everyone in the area; it replaces any scenery object on that tile);
3. each tick while it stands: damage to players on its tile (the plugin's own tick task; the object is walk-through, so nothing
   else stops them);
4. one tick (30 cycles) before the end: `obj.performAnimation(new core.Animation(despawnSeq))` (LOC_ANIM);
5. at the end: `ObjectManager.deregister(obj, true)` (LOC_DEL). Players who enter the area later must be sent the object (whether the
   server's object manager does that for registered objects was not read) and will see it start at frame 0 of the spawn: the
   sequence is not synchronised to when it was made.

## 9. The sentinel's set, second pass (`set2/`; `set2/notes.md`)

See `set2/notes.md` for the five rounds and `REPORT.md` for the judgment. Final numbers (`set2/round5.log`, `set2/style.log`).

## 10. Still missing for arenas and mechanics, ranked

1. **Object swaps for phases, doors and barriers.** Feasible now: it is LOC_ADD_CHANGE with another id, and the harness draws it
   (`loc` / `floc` twice on one tile; proved on the Nightmare's spore). Missing: a wall-shaped object (shapes 0 to 3, 9) in the
   file path (only shapes 10, 11, 22 are placed) and a brief. Small.
2. **Tile markers and floor glyphs as shape-22 objects** (Yama's glyphs, Sotetseg's maze, the Titans' safe tile). Feasible now
   with `disc_falloff` / `star4` and a `.loc`; one rebuild target (33035) is waiting. Small.
3. **Falling objects with shadows.** The game does it as two spot animations by id (a shadow telegraph such as 1446, one per
   fall time, and a rock that is a projectile or a tile graphic with a long first hold). Feasible with today's tools; nothing
   built. Small to medium.
4. **Beams between two points.** The format has no beam: the game uses a projectile with a long thin model and slope 0, or a
   chain of tile graphics, or an NPC. A stretched projectile is drawable now (`proj`); a true two-ended beam would need the
   server to place and turn an NPC or objects per tile. Medium; needs a study of real ones (Verzik's webs, the Wardens' lasers).
5. **Attached, orbiting parts.** An actor holds several spot animations in numbered slots, each following the actor (drawn by
   the harness: `aspot ... SLOT`); orbit is a rotate transform in the sequence. A LOOPING aura is not possible as a spot
   animation on an actor (it ends at the first wrap): it is re-sent, or it is part of the NPC. Medium: needs the re-send
   cadence measured on real shields (Kephri 2141, Nylocas 1558).
6. **Breakable props.** Object swap (whole, damaged, broken) plus a spot animation burst: all drawable; nothing built. Small.
7. **Skeletal sequences for objects and effects from files** (108 objects, 138 effects use them). The file path poses classic
   sequences only; the project's skeletal route (`research-rig`) exists for creatures. Medium.
8. **Floor tiles that change (underlay / overlay swaps).** NOT a packet: floors are part of the map square. The game changes
   floors by instancing another map copy or by laying shape-22 objects over the floor (what every [quoted words withheld] mechanic
   here does). Treat it as item 2; a true floor change means a second map square. Large and probably unnecessary.
9. **Screen effects.** Camera shake and fades are packets, not assets (`sendCameraShake(slot, amplitude)`; the fader is client
   script 343 on the owner's server). The harness has only a stand-in shake. The client's own shake code is not ported into the
   harness. Small to port, no asset work.
10. **The boss health bar.** An interface drawn by client scripts; the owner's server already has a HUD plugin
   (`plugins/interface/BossHud.plugin.js`: `boss-hud:show {player, npcId, current, maximum}`). Reuse; no asset work.

## 11. Not proven

- Nothing here was seen in a logged-in client; no server sent a packet; the emitted plugin calls were not run.
- The moving pictures were judged from stills taken out of them at set moments (six to ten a life), not by watching motion.
- Random starts, skeletal sequences from files, multi-model objects at game size against actors: section 3.
- The object gate has met ceilings and synthetic damage only; no animated object has been rebuilt blind.
- The softness study's "pieces" and "nesting" come from a geometric rule (connected parts; box inside box); undrawn pivot faces
  appear in it as black levels. The on-screen numbers do not depend on it.
- The kit pieces were compared with one real effect each, by me, knowing which was which; the blind files exist and no one
  else has looked.
