# How an NPC is lit and what that means for joints and facets (revision 239 client and cache)

Private to this machine. Step 2 of the creature research, 2026-11-27. Tags: PROVEN = client source read AND a test
with numbers; SOURCE ONLY = read in the client, not separately tested; NOT PROVEN = stated as such.
Source paths are under `{CLIENT_SRC}/src/`.

## What the client does

1. **Lit once, at rest, then cached. PROVEN.** `world_actors.rs:444-461`: if the NPC is not in the cache `self.npcs`,
   `npc_model::mesh(..)` builds the rest mesh, then `let colors = model.light(64 + d.ambient, 850 + d.contrast,
   [-30, -50, -30])?;` (line 457) and `self.npcs.insert(key, (model, colors))` (key = definition id plus any
   server override; the cache holds 256 entries). Every frame after that (`:463-473`): `model.clone()`,
   `animations.apply_actor(..)`, `npc_model::resize(..)`, `place_actor(&model, colors, ..)` with the SAME cached
   colours. `npc_model.rs:1` says it in words: "Lighting precedes animation and body resizing." Nothing re-lights:
   `.light(` appears in `world_actors.rs` only at the cache-fill sites (113 players, 273, 393, 457, 595, 728, 1006,
   1323), never after `apply_actor`. Bone-weighted NPCs (cached animations) go through the same lines; there is no
   separate branch. Recolours are applied inside `mesh`, before lighting. A transform (another definition id) is
   another cache key, lit once in its own right.
2. **The numbers. PROVEN** (port equals client on every face, below). `npc.rs:169-170`: ambient is opcode 100 (signed
   byte), contrast opcode 101 (signed byte x 5). Light number at a smooth vertex =
   `ambient' + dot(sum of its faces' 256-length normals, (-30,-50,-30)) / (D x votes)`, integer division toward zero,
   with `ambient' = 64 + ambient`, `D = ((850 + 5 x contrast) x 65) >> 8` (65 = int(|direction|)); D = 215 with no
   definition values. One vote per SMOOTH triangle using the vertex, whatever its size; the sum is not re-normalised.
   Flat face (render type 1): `ambient' + dot(face normal, direction) / (D + D/2)`: two thirds of the directional
   part, and the face casts no vote. Drawn lightness = `(paint lightness x light number) >> 7`, clamped 2..126; hue
   and saturation untouched (`model_light.rs:83-183`).
   With 0/0 the directional part runs to +/-78 around 64: a surface square to the light draws at 1.11 x its paint,
   one facing straight up 0.96 x, the front (-z) and the -x side 0.77 x, the back (+z) and +x side 0.23 x, one facing
   straight down 0.04 x (clamped to lightness 2: black).
3. **The light is fixed to the BODY, not to the world. PROVEN for animation, SOURCE ONLY for turning.** Because the
   colours are computed in the rest pose and the model is turned to its facing afterwards (`place_actor(.., npc.angle,
   ..)`, `world_actors.rs:487-500`), the lit side is always the creature's own top, front (-z, the head end) and -x
   flank, wherever it faces and whatever the pose. Measured mean light number by the dominant direction of a
   vertex's summed normal, real creatures at rest (128 = paint as painted):
   | | up | down | front -z | back +z | -x flank | +x flank |
   |---|---|---|---|---|---|---|
   | Hellhound 104 | 104 | 25 | 75 | 42 | 87 | 40 |
   | Wolf 106 | 107 | 24 | 86 | 48 | 94 | 44 |
   | Cow 2790 | 106 | 22 | 85 | 36 | 92 | 40 |
   | Araxyte 11176 (ambient 10, contrast 30) | 107 | 45 | 93 | 58 | 90 | 59 |
4. **Faces that vote without being seen. PROVEN.** `normals()` (`model_light.rs:81-117`) looks only at the render
   type: 0 votes, 1 is flat, anything else neither votes nor is lit. It does not look at alpha, visibility or
   winding. `light_with_normals` then marks alpha 255 as not drawn (`colors[i][2] = -2`), alpha 254 as a fixed
   colour 128 flat. So: a face inside the body votes; a smooth face with alpha 255 votes and is never drawn; a flat
   face never votes. Real creatures carry such faces: Tormented Demon 1 undrawn face, General Graardor 2; over all
   7,801 NPC models 4,805 faces in 242 models have alpha 255.
5. **Several models become one welded model before lighting. PROVEN (counts) / SOURCE.** `model_merge.rs:54-82`:
   `Model::combine` keeps one vertex per whole-unit coordinate across ALL parts (first part's label wins), so parts
   that meet at equal coordinates share votes across the seam. Goblin 655: 994 part vertices become 936; Graardor
   1,038 become 1,012; Vyrewatch 308 become 304; Scurrius 1,762 stay 1,762. A one-model NPC is not combined: its
   vertices stay as authored.
6. **Players: the same light, different painting. SOURCE ONLY.** `world_actors.rs:113`: `light(64, 850, [-30,-50,-30])`
   once per appearance, cached; no ambient or contrast of its own. Painting differs (priority order, no depth test
   inside the figure); an ordinary NPC is depth-tested and ignores priorities (NPCS.md).
7. **Render types on NPC meshes. PROVEN (census).** Over the 7,801 models NPC definitions name (3,269,142 faces):
   type 0: 96.88%, type 1 (flat): 3.12% (101,886 faces, in 854 models = 10.9% of models), type 2: 14 faces (neither
   votes nor is drawn), nothing else. Alpha 254: 2,285 faces. All twelve creatures checked below are 100% smooth.

## Tests run

Tools: `examples/creature_light.rs` (new; prints `Model::light`'s own result for `npc:ID` through
`npc_model::mesh`, or for one of our files; with `seq=ID:FRAME` also the posed vertices and what a re-light would
give), `light_npc.py` (the port, the comparison and the joint statistics), `light_test_votes.py`. Data in
`light-data/`. `P` = `{PYTHON} -I`, run in this folder.

**(a) Port against client:** `P light_npc.py dump npc:ID light-data/npc-ID.txt [seq=S:F]` then
`P light_npc.py check light-data/npc-*.txt`. Twelve real creatures, 26,587 faces, 79,378 lit corners: 100.00%
identical, 0 corners differ. Hellhound 104, Wolf 106, Cow 2790, Goblin 655 (8 models, ambient 15), General Graardor
2215 (2 models), Vorkath 8061, Araxyte 11176 (10 / 30), Tormented Demon 13599 (10 / 25), Scurrius 7221 (4 models,
bone weights, 10 / 100), Dire gryphon 14859 (bone weights), Wyrd 16212 (bone weights, 10 / 20), Vyrewatch 16134
(3 models, ambient -25). Our own five files (hound 2, spider, boss 1, boss 2's two) also 100%.

**(b) Lit once:** the proof is the source (item 1). What the test adds is the size of the consequence:
`P light_npc.py relit light-data/npc-ID.txt` compares the rest colours (which are what is drawn) with what
`Model::light` would give on the posed mesh:
| creature, sequence frame | corners a re-light would change | by 8 or more steps | median / 90th centile / largest change |
|---|---|---|---|
| Hellhound 104, seq 6576 frame 6 | 78.5% | 5.3% | 2 / 5 / 30 |
| Wolf 106, death 6558 frame 6 | 94.6% | 50.4% | 8 / 21 / 57 |
| Cow 2790, death 5851 frame 5 | 76.3% | 22.9% | 2 / 14 / 31 |
| Goblin 655, death 6182 frame 5 | 84.8% | 16.9% | 3 / 11 / 39 |
| General Graardor 2215, death 7020 frame 8 | 82.7% | 22.7% | 2 / 13 / 49 |
| our star hound 2, death 6558 frame 6 | 89.8% | 18.3% | 3 / 10 / 20 |
Picture (harness, which repeats the client's steps; not independent evidence): `render/research-creatures-light/
rest-vs-death.png`: the cow and wolf lying on their sides keep their standing tones, dark belly and all.

**(c) Hidden faces vote:** `P light_test_votes.py light-data/votes`. A four-sided limb, paint lightness 64, lit by
the client; lit lightness at the four middle-ring vertices of the OUTER faces:
| version | lit lightness at the ring | |
|---|---|---|
| A nothing inside | 49, 32, 32, 14 | |
| B hidden smooth fan across the joint, facing up | 52, 39, 39, 26 | +3 to +12: it votes |
| C the same fan flat-marked | 49, 32, 32, 14 | equals A: flat faces do not vote |
| D the same fan smooth, alpha 255 (24 of 28 faces drawn) | 52, 39, 39, 26 | equals B: votes without being drawn |
Port equals client on all four.

**(d) How real joints are built:** `P light_npc.py joints light-data/*.txt`.
| creature | faces | labels | faces spanning 2+ labels | split vertices | connected pieces (largest, vertices) | open edges / loops |
|---|---|---|---|---|---|---|
| Hellhound 104 | 519 | 44 | 72.8% | 0 | 1 (266 of 266) | 12 / 1 |
| Wolf 106 | 892 | 64 | 75.2% | 0 | 1 (450 of 450) | 8 / 1 |
| Cow 2790 | 946 | 74 | 65.6% | 0 | 1 (475 of 475) | 0 |
| Araxyte 11176 | 1,020 | 62 | 62.3% | 0 | 1 (512 of 512) | 0 |
| Tormented Demon 13599 | 1,846 | 169 | 60.7% | 0 | 11 (835 of 928) | 4 / 2 |
| Vorkath 8061 | 2,901 | 251 | 63.4% | 0 | 21 (1,437 of 1,497) | 63 / 21, 20 tucked |
| Vyrewatch 16134 (3 models) | 632 | 86 | 73.1% | 0 | 3 (275 of 304) | 16 / 4 |
| General Graardor 2215 (2 models) | 1,928 | 52 | 34.4% | 0 | 9 (830 of 1,012) | 82 / 8, 6 tucked |
| Goblin 655 (8 models) | 1,910 | 53 | 33.1% | 0 | 5 (622 of 936) | 8 / 2 |
| Scurrius, Dire gryphon, Wyrd (bone weights) | | 0 | n/a | 0 | 17, 1, 1 (85%, 100%, 100% in one piece) | 66, 3, 0 |
| OURS star hound 2 | 674 | 47 | 66.8% | 0 | 3 (331 of 343) | 0 |
| OURS orrery spider | 764 | 54 | 60.2% | 0 | 2 (316 of 384) | 0 |
| OURS boss 1 | 2,266 | 52 | 41.4% | 0 | 19 (154 of 1,176) | 12 / 1 |
| OURS boss 2 (two files, each alone) | 1,226 + 1,368 | 27 + 29 | 34.8%, 35.7% | 0 | 22 and 20 (154 and 121) | 52 / 5, 40 / 4 |
("Tucked" is a proxy: the loop's centre lies inside another piece's bounding box.)

## Rules for a creature author

1. **A creature is ONE welded skin. PROVEN on the sample.** No real creature measured has a single split vertex,
   and 82 to 100% of its vertices are one connected piece (the eight-model Goblin: 66%); the only separate pieces are small rigid things carrying
   one label (claws, horns, spines, armour plates), open where they sink into the body. A joint is not two capped
   parts meeting: it is the same skin with the label changing along it, and the faces at the change span both
   labels (60 to 75% of ALL faces in the one-model beasts: labels change ring by ring, about one label per 5 to 8
   vertices). No gap can open because there is no seam to open; the bridge faces stretch instead.
   Our hound and spider are built this way. Our two bosses are not: 19 to 22 closed blocks, the largest 13% of the
   vertices, against 82 to 96% for Graardor, the Tormented Demon and Vorkath.
2. **A bend never changes the light. PROVEN.** Tones are fixed at rest. A crease, a raised limb, an opened jaw or a
   creature on its back keeps the tones of the rest pose (a re-light would change 76 to 95% of corners, a fifth of
   them by 8 steps or more). So: the rest pose is where every tone is decided; anything that must read in motion
   (a mouth interior, the underside shown in death, the inner face of a raised leg) must be right as PAINT in the
   rest pose; and a part that swings far (tail, jaw, wing) should not carry a strong light-to-dark sweep at rest,
   because that sweep will point the wrong way when it turns.
3. **Undersides are black unless you act. PROVEN.** With 0/0 a downward face draws at about 0.04 to 0.2 of its paint
   (light number 22 to 25 on real beasts); the +x flank and the rear at about 0.3. Definition ambient is the only
   thing that lifts them: recent creatures in the sample carry ambient 10 with contrast 20 to 100 (Araxyte's
   underside 45 against 22 to 25), the early beasts 0/0. Choose ambient and contrast with the mesh; do not paint
   a [quoted words withheld] on top of a 0/0 mesh, it is already near black, and do not expect belly detail to show.
4. **The creature has a lit side and it travels with it. PROVEN / SOURCE.** Top, head end and -x flank are bright;
   +x flank and rear are dim. An asymmetric feature reads better on the -x side; check the +x side and the rear
   in every review, since that is what the player sees half the time.
5. **Votes are a tool, and they work on NPCs exactly as on scenery. PROVEN.** Hidden smooth faces vote; alpha-255
   smooth faces vote and are not drawn; flat faces do not vote and get two thirds of the directional light; a
   vertex where faces meet sharply is pulled toward ambient. `light_npc.py` predicts any mesh exactly before a render.
6. **Facets come from the split, not from the flat flag. PROVEN on the sample.** All twelve real creatures checked
   are 100% smooth, and 89% of all NPC models carry no flat face at all. The faceted look of real creatures is large
   smooth triangles meeting at firm angles on a welded skin. Flat-marking is rare and small on NPCs; a creature
   leaning on flat faces for its planes (our boss 2: 144 flat faces) is not built as the real ones are.
7. **Parts given as several models weld where coordinates are equal. PROVEN (counts).** Use exact shared
   coordinates where two files should shade as one surface, and avoid accidental coincidences where they should not.

## Not proven

- That the live client's world path was exercised: `creature_light` calls `npc_model::mesh` and `Model::light` with
  the numbers of `world_actors.rs:457`; it does not run `npc_triangles` itself. "Lit once" rests on reading that
  function; the harness picture repeats the same steps and is not independent.
- The turning claim (rule 4, world facing): read from the call order only; `place` was not read line by line, and
  no test turned an NPC. The body-fixed numbers in the table are measured.
- The enhanced (GPU) renderer feature was not examined; everything here is the classic path.
- Hellhound sequence 6576 was taken from the server's list as a death; in the picture it looks like a howl. The
  numbers for it stand as [quoted words withheld], not as a death.
- The joint table is 12 real creatures chosen by hand, not a census; "tucked" is a bounding-box proxy; the
  bone-weighted creatures have no labels, so their joints were not measured at all.
- Whether ambient 10 with raised contrast is the recent norm: seen on 4 of 6 recent creatures here, not surveyed.
