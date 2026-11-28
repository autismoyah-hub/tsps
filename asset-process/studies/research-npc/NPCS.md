# How the game's NPCs are put together (revision 239 cache)

Measured with `asset_npc.exe CACHE census census/` (the client's own decoders) and `census.py`; the full printout
is `census/survey.txt`. Private to this machine.

## How an NPC is drawn (client source, read only)

- `src/npc_model.rs:49 mesh`: every model id of the definition is decoded and, when there are several, joined by
  `Model::combine` into ONE model (equal coordinates become one vertex, first label wins); then the definition's
  recolours and retextures are applied.
- `src/world_actors.rs:457`: lit once at rest, `light(64 + ambient, 850 + contrast, (-30, -50, -30))`; `:466`
  animated; `:473` `npc_model::resize` (scale) AFTER animation; `:487-500` `place_actor(.., Mode::actor(false,
  def.z_buffer, use_player))`.
- `src/model_order.rs:68 Mode::actor`: Priority only if `player || !npc_z_buffer || player_override`, else Auto.
  `src/npc.rs:95` z_buffer defaults true; `:225` opcode 147 clears it.
- `src/render3d.rs:864`: Mode::Auto paints faces in model order with a per-pixel depth test; priorities are not
  consulted (`:890` sorts only for modes 2 and 3); per-face depth offsets apply (`Triangle.depth_bias`).

So an ordinary NPC, including one assembled from several models, is depth-tested like scenery and its face
priorities do nothing. 13,822 of the 13,836 definitions with models are ordinary. 14 carry opcode 147 and are
painted by priority exactly as a player (Thurgo 4733, Amergin 15602, Pete 15603, Lara 15605, ten Ivan Strom).

## Counts

16,298 definitions; 13,836 have models; 8,541 distinct looks (model list + recolours + scale); 6,602 distinct model lists.

| | definitions | distinct model lists |
|---|---|---|
| one model | 4,878 (35.3%) | 1,804 (27.3%) |
| several | 8,958 (64.7%) | 4,798 (72.7%) |

Models per definition: 2: 1,092; 3: 902; 4: 500; 5: 735; 6: 1,124; 7: 1,672; 8: 1,346; 9 and more: 1,587.

- **People.** The player's frame base is skeleton 0. 5,695 definitions (41.2%; 3,339 distinct model lists) stand on
  it. 5,171 of them are several models (most often 7, 8 or 9), 524 one model. 4,399 (77.2%) contain at least one
  player body-kit model or worn-item model; 795 are made ONLY of such models. 361 more definitions off the player
  base also borrow such parts. Example, White Knight 1798: seven models, helm 11744, the body of item 75, the arms
  of the Iron platebody, gloves, a Bronze 2h sword, Iron platelegs, Bronze boots, six recolours.
- **Recolours:** 8,480 definitions (61.3%), median 5 pairs. Retexture: 1.
- **Scale:** not 128/128 in 2,997 (21.7%); of those the y scale runs 48 / 110 / 162 at the 10th / 50th / 90th centile.
- **Size in tiles:** 1: 11,224; 2: 1,293; 3: 706; 4: 177; 5: 310; 6: 47; 7: 56; 8: 7; 9: 16.
- **Chat heads:** 8,314 definitions (60.1%) list head models (1: 5,326; 2: 2,097; 3: 745; 4: 143; 5: 3).
- **Sequences:** stand present in 12,570, walk in 12,133; stand and walk share a frame base in 12,096 of 12,099.
  Attack, block and death are NOT in the cache: the server sends them (TSPS `npc-combat-defs.json`,
  `npc-captured-animations.json`, `npc-animations.json`).
- **Lighting:** ambient and contrast are 0/0 in 11,442; 2,394 set them (15/0, 10/10, 30/30, 20/0, -25/0 commonest).
- **Bone weights:** 401 definitions carry weighted models; 390 stand in a cached (bone) animation. The rest use labels.
- **Layers:** 5,378 of 6,602 model lists carry more than one face priority (it does nothing on an ordinary NPC);
  3,126 carry per-face depth offsets (which do).

## Frame bases (skeletons)

634 distinct frame bases serve the definitions that have a stand or walk sequence (1,232 have neither).
By distinct model lists: 335 frame bases serve exactly one, 194 serve 2 to 4, 85 serve 5 to 19, 20 serve 20 or more.
86.2% of distinct model lists sit on a frame base that serves five or more; 5.4% on one of their own.
A new mesh on a borrowed skeleton is the game's common practice, not a rarity.

| skeleton | distinct model lists | definitions | groups | sequences on it | who |
|---|---|---|---|---|---|
| 0 | 3,339 | 5,695 | 245 | 3,904 | people: Guard, Citizen, Banker, Pirate, Soldier |
| 297 | 250 | 358 | 50 | 58 | dwarves |
| 2402 | 144 | 337 | 67 | 98 | children, gnomes |
| 1415 | 110 | 219 | 170 | 73 | goblins |
| 1288 | 98 | 202 | 77 | 50 | zombies |
| 1383 | 81 | 125 | 83 | 57 | cave goblins |
| 461 | 60 | 126 | 52 | 45 | trolls |
| 321 | 55 | 94 | 46 | 31 | ogres |
| 1081 | 52 | 103 | 231 | 37 | giants, cyclopes |
| 1291 | 49 | 109 | 84 | 58 | skeletons |
| 1491 | 49 | 122 | 76 | 43 | quadruped: Wolf, Hellhound, Guard dog, Fox, Jackal |
| 1914 / 1098 / 1097 | 43 / 24 / 16 | 99 / 121 / 34 | | 45 / 33 / 9 | vampyres |
| 1078 | 40 | 107 | 132 | 33 | winged biped: Lesser, Black, Greater demon, K'ril Tsutsaroth |
| 1080 | 34 | 117 | 194 | 30 | dragon: Green, Blue, Red, Black, Steel, baby dragons |
| 280 | 25 | 128 | 61 | 20 | cats |
| 1574 | 24 | 29 | 100 | 14 | aviansie, Commander Zilyana |
| 2248 / 1538 | 19 / 13 | 36 / 23 | 38 / 69 | 35 / 19 | birds: parrots; gull, heron, owl |
| 1247 | 18 | 58 | 52 | 13 | sheep, goats |
| 1888 | 17 | 32 | 255 | 41 | The Nightmare and its husks |
| 1338 | 15 | 30 | 135 | 44 | cow, buffalo |
| 1426 | 15 | 33 | 135 | 18 | scorpions |
| 1003 | 14 | 20 | 100 | 15 | orks, General Graardor |
| 1800 | 13 | 61 | 232 | 38 | spiders: Nylocas, Araxyte, Sraracha |

`census.py --skeleton ID` lists every creature and sequence on one; `census.py --npc ID` one definition with its
models (labels, priorities, depth offsets, which item or kit each comes from); `atlas.py OUTDIR ID...` draws where
each label sits on real creatures of a skeleton.

## Three bosses (what one needs)

| | General Graardor 2215 | Vorkath 8061 | Scurrius 7221 |
|---|---|---|---|
| size in tiles | 4 | 7 | 3 |
| models, faces | 2, 1,928 | 1, 2,901 | 4, 3,402 |
| box (w x h x l, after scale) | 445 x 484 x 287 | 1,424 x 284 x 1,122 | 188 x 236 x 747 |
| scale | 110 | 104 | 90 |
| movement | labels (52) | labels (251) | bone weights, no labels |
| skeleton | 1003, shared with 13 other model lists (orks) | 1782, its own | 2191, shared with one other |
| sequences on the skeleton | 15 | 15 | 22 (cached animations) |

Our method can now do a Graardor-kind boss: a new mesh of about 2,000 faces labelled to a shared skeleton and
played through that skeleton's sequences (the star hound is that at 654 faces). It cannot yet do a Vorkath-kind:
a unique skeleton needs new frames and sequences, which we do not write. It cannot do a Scurrius-kind at all:
we write no bone weights and no cached animations. Also not built or tested for any boss: a chat head, special
attack graphics and projectiles, a mesh over 1,000 faces, a labelling pass for 100 to 250 labels (the hound uses 46).
