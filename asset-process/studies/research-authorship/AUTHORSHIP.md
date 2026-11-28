# How Jagex authors an entirely new end-game boss: the package, in numbers and in process

Private to this machine (cache-derived). Study of 2026-11-27, folder `{ROOT}/studies/research-authorship/`.
`P` = `{PYTHON} -I`, run in this folder. Revision 239 cache (`tools/osrscache.py` CACHE).
The TSPS checkout was read only for its boss plugins. Nothing outside this folder was written.

```
P pkg_measure.py               -> data/packages.json   (every package: NPCs, models, colour plan, skeleton, sequences, effects, objects)
P pkg_table.py > data/packages.txt   per-boss summary (data/summary.json);   P pkg_dump.py KEY [npcs colour skel seqs spots locs]
P beyond.py                    -> data/beyond.json     (three eras of boss meshes and boss effects compared)
P packagecheck.py PACKAGE.json         a package against the eight (holds / OUTSIDE, band, median)
P packagecheck.py --real KEY OUT.json  a real package written in the same format;   P packagecheck.py --calibrate
P names.py KIND REGEX                  grep Jagex's internal (gameval) names: npc, loc, seq, spot
```

**How a package was found.** Archive 24 of the cache holds Jagex's own internal names ("gamevals") for every NPC, object,
sequence and spot animation. A package is everything whose name carries the boss's stem (`maggot_king`, `dom_`/`vfx_doom`,
`yama`, `huey`, `amoxliatl`, `araxxor`, `rt_`/`branda`/`eldric`/`royal_titans`, `colosseum_sol`/`colossi_finalboss`), minus
quest, cutscene, Leagues and Deadman copies. To that are added every sequence on the boss's frame base and every spot
animation the server copy's boss plugin names (`server/plugins/bosses/<boss>/`). The plugin comments say these ids were
captured from the live game. This second source is the only way to see REUSED effects, which carry another boss's name or a
generic one. Numbers already measured elsewhere are cited, not re-measured: model measures come from BOSSES.md
(`research-bosses/data/bosses.json`), the spot census from EFFECTS.md, the animated-object census from EFFECTS2.md, arenas from
ARENAS.md, and fights from FIGHTS.md.

## 1. The eight bosses and what the cache holds of each

Chosen as the eight newest entirely new designs in this cache (2024 to 2026; release month is my knowledge):
- **Sol Heredit**, March 2024
- **Araxxor**, June 2024
- **Amoxliatl**, September 2024
- **The Hueycoatl**, September 2024
- **The Royal Titans**, November 2024
- **Yama**, May 2025
- **Doom of Mokhaiotl**, July 2025
- **Maggot King**, June 2026

Desert Treasure II's four bosses (2023) are left out as older. Their meshes are in BOSSES.md, and their fights are in
FIGHTS.md. Amoxliatl and the Hueycoatl are mid-level bosses (combat 263 and 642), not raid-tier. They are kept because the
owner named them, and they widen the low end. Amoxliatl is outside the band on more rules than any other (calibration, section 6).

### 1a. Model, colour, skeleton

The model columns are from BOSSES.md for the main form. "Fam" is hue families holding 3% or more, by cmesh's own rule
(neighbouring hues chain into one family). "Main" is the share of the largest family. The sector reading (eight fixed
45-degree hue sectors, unsaturated colours apart) is in `data/packages.json` under `sectors` and `accent`.

| boss | npc ids (fought forms) | tiles | models | faces (all models) | colours / 90% | fam / main | translucent area | inlaid islands | rig: base (groups/bones) | own skeleton? |
|---|---|---|---|---|---|---|---|---|---|---|
| Sol Heredit | 12821 (1); 5 ids | 5 | 4 | 3,867 | 66 / 19 | 3 / 0.79 gold | 0 | 195 | weighted 2245 (104/108) | yes; shared with its own wave Colossi (12817, 12819) |
| Araxxor | 13668 (1); 12 ids | 7 | 2 | 3,730 | 37 / 10 | 4 / 0.41 dark red | 0.24 | 315 | classic 2351 (222 groups) | yes |
| Amoxliatl | 13685 (1); 3 ids | 3 | 2 | 1,886 | 19 / 7 | 2 / 0.96 ice blue | 0.46 | 25 | weighted 2244 (12/214) | **no**: the Moons of Peril skeleton (2023: `npc_djinn_01..05` sequences, Sulphur and Frost Nagua). 9 of its own sequences among 50 on the base |
| The Hueycoatl | head 14009 / enraged 14013, tail 14014, body 14017 (4); 12 ids | 7 (tail 5) | 2 (+1 shield) | 4,003 (4,907) | 47 / 18 | 6 / 0.54 stone grey | 0.01 | 228 | weighted 2359 (16/163) + tail 2361 (56/49) | yes, two skeletons |
| Royal Titans | Branda 12596, Eldric 14147 (2); 11 ids | 3 each | 1 each | 2,510 / 3,094 (5,604) | 49 / 15 (Branda) | 1 / 0.99 fire red; Eldric 0.98 ice blue | 0.08 Branda; Eldric 68% of faces | 34 | classic 2400 (224) + 2395 (255) | yes, one per titan |
| Yama | 14176 (1); 9 ids | 5 | 3 | 3,070 | 31 / 9 | 3 / 0.50 red + 0.43 near-black | 0 | 61 | weighted 2404 (52/104), Bezier curves | yes |
| Doom of Mokhaiotl | 14707, shielded 14708, burrowed 14709 (3); 12 ids | 5 | 5 (3,012 body + two 391-face parts + 2 tiny) | 3,810 | 66 / 25 | 1 / 0.97 red (sectors: red 0.72, orange-gold 0.24) | 0.01 | 43 | weighted 2441 (88/90) | yes |
| Maggot King | 15742 (1); 4 ids | 5 | 1 | 3,733 | 37 / 14 | 4 / 0.57 dark teal-grey + 0.30 red | 0.08 | 37 | weighted 2653 (212/155) | yes |

No model of any of the eight has a textured face.

### 1b. Family, animation, effects, arena, fight

| boss | family on the skeleton | sequences on own skeleton (attacks; median / total cycles) | own effects (distinct models; projectiles) | effects reused (server-used) | own objects (animated) | fight (FIGHTS.md) |
|---|---|---|---|---|---|---|
| Sol | pet Smol Heredit (scale 50), seated state 12827, gib | 24 (9; 90 / 3,040) | 8 (5; 0) | none found | 0: his arena is the Colosseum's (364 `colosseum` objects, not his) | not measured |
| Araxxor | 3 Araxyte adds on the **Nylocas skeleton 1800 (2019)**, models 32073 to 32586 (ToB era); 3 eggs; pet Nid (scale 25) and Rax (Nylocas skeleton) | 18 (8; 60 / 1,358) | 6 (5; 3) | 4: Nylocas ranged projectile 1560, Venenatis web 2360, Araxyte explosions 2925/2926 | 79 (3): cave kit, webbed stalagmites, egg sacks, acid pool 54148 | 11 mechanics, 5 answers, 2 states, tells 3, final speed-up |
| Amoxliatl | pet Moxi (scale 32), unstable ice NPC | 9 own (3; 90 / 870) | 13 (8; 2) | none found | 20 (12): ice that appears, holds and vanishes | not measured |
| Hueycoatl | tail (own base), body hit points (10-face placeholder model 32530 at scale 128x350), broken tail, rubble, pet Huberte | 19 (8; 150 / 2,520; awaken 330) | 35 (11; 3) | none found | 46 (19): prayer pillars in 3 styles x 5 states, the phase-2 barrier (3 states), attack tiles 55209 | not measured |
| Royal Titans | 2 elemental adds on their own bases, inactive forms, 3 hit-box helpers, pets Bran and Ric (scale 26) | 32 over two bases (13; 90 / 3,600) | 17 (9; 3) | none found | 16 (10): spawn and death floor pieces per titan, fire and ice idle objects, ice throne; also the safe tile 56003 (EFFECTS2) | not measured |
| Yama | **Judge of Yama on base 1078 (A Kingdom Divided, 2020, model 42551)**, Imp on base 1240, Void Flare on base 2147, meteor NPC; pet Yami (scale 30) | 12 (5; 120 / 1,500) | 35 (24; 5; 6 skeletal and weighted) | 1 (a Wind Strike cast) | 16 (6): fire wall, smoke, shadow pool, portal; also glyphs 56335 to 56337 under a generic name (EFFECTS2) | 16 mechanics, 5 answers, 5 states, one lethal, 2 failures that feed him |
| Doom | larvae on their own base 2437 (giant ones are the same models at scale 300), volatile earth, earthen shield, pet Dom | 18 (9; 60 / 1,350) | 45 (9; 1): the 7 `vfx_dom_*` burrow effects and the area slam share ONE 2,021-face dust model with different sequences, 6 `vfx_doom_mound_*` share one 171-face model, and 16 blood splats share one 281-face model | 19: the new `vfx_standard_projectile_melee/magic/range` 3378 to 3380, a `vfx_rock_projectile_*` set, `vfx_beam_*`, the Leviathan's impacts 2490 to 2492, the DT2 guardian rock fall 2529, the Colossi stab dust 2699, holy water 192, the gargoyle shadow 2380 | 14 (1): acid pools, rock, entries | not measured |
| Maggot King | Ur-maggot larvae (an old grub base 678), corpse state, pet Maggot marquess (scale 25) | 15 (4; 90 / 1,440): a 7-way walk set (forward, back, left, right; walking while attacking), 3 deaths | 10 (5; 3) | gargoyle debris shadow 1446; the plugin also names Xarpus' acid spit 1555 as its magic projectile | 45 (0): a 23-piece `floorkit_maggot` floor kit, corpse piles of up to 3,091 faces, bile and blood pools | not measured |

Ids for every row are in `data/packages.json`, and `P pkg_dump.py KEY` lists them.

**Effect colour.** Effects are painted in the boss's own colours. The median effect has 0.89 to 1.0 of its colour in the
boss's own colour sectors (main, body or accent; rule E6). Three conventions are seen:
- **Fire and shadow kept apart.** Yama's fire effects take his red family. His shadow spikes and roots take his near-black second material.
- **One colour per attack style.** The Hueycoatl's melee effects are wholly in her red accent; ranged and magic effects take her stone and blue.
- **Generic effects coloured by style.** Doom's standard projectiles are also coloured by style.

## 2. Jagex's process, from its public posts

Collected by a web-reading agent and cut down here. I re-read two pages myself: the Hueycoatl quotes in the July 2024 Gazette
and the Desert Treasure II art-team quotes; both match the pages. The other quotes come through a page summariser; spot-check
them before reusing. "(wiki)" means the OSRS wiki, not a Jagex post.

**Concept and theme come first, from the region.**
- Sol Heredit: "Each drop of blood spilt is an offering to Ralos" ([Colosseum first look](https://secure.runescape.com/m=news/fortis-colosseum---first-look--rewards?oldschool=true)). The arena [quoted words withheld] (wiki).
- The Hueycoatl, by Mod Jerv: "I took inspiration from the shape language used in various murals and statuesque depictions of Quetzacoatl … I wanted to bring in elements of an animal that screams 'Roman!' as well; a lion." "She became a colourful winged lion-serpent with draconic influences." ([Gazette July 2024](https://oldschool.runescape.wiki/w/Update:Gielinor_Gazette_-_July_2024), wiki copy of the post). Varlamore is Aztec plus Roman, and the boss is both.
- Doom, by Mod Jerv: "We wanted the design to be insectoid and demonic". Early concepts took [quoted words withheld] (wiki, Doom of Mokhaiotl).
- Araxxor is a reinvention of an old RuneScape boss: [quoted words withheld]. Mod Arcane: [quoted words withheld] ([wiki Update page](https://oldschool.runescape.wiki/w/Update:New_Slayer_Boss_-_The_Araxyte)).

**Designer and concept artist together, then the model.**
- Mod Jerv on the Hueycoatl: "This time, Mod Squid and I jumped into it together as a blank slate", which implies they usually work in turn.
- Concept art is credited per boss: Mod Grub (Sol, Yama), Mod Jerv (Hueycoatl, Amoxliatl, Royal Titans, Doom, Maggot King, and most arenas), Mod Skylark (Araxxor).
- Then come a [quoted words withheld] (Araxxor), animations by Mod Harold, and rigging by Mod Hend (the Hueycoatl's barrier) (wiki galleries).

**The mechanics shape the body and the arena.** For the Hueycoatl: "Parts of this colossal serpent are poking out from caves
all over the mountain … When enough body parts have been damaged, the path to the peak will open … it'll shield itself with
its tail". The cache shows exactly this:
- separate body-part NPCs and a separate tail on its own skeleton;
- a `huey_shield_*` idle and attack set;
- the phase-2 barrier as three animated objects;
- prayer pillars in 3 styles x 5 states.

Doom [quoted words withheld] and descends into its burrow; the cache has a whole burrowed form with its own sequences and telegraph effects.

**Iteration is public and about readability.**
- The Hueycoatl's dragon reading: "A lot of players looked at the mighty Hueycoatl and saw a 'dragon' … this wasn't originally our intention".
- After release, Araxxor got [quoted words withheld].
- The Maggot King got "Improved the visual distinction between the Maggot King's projectile attacks and floor hazards" ([wiki tweaks page](https://oldschool.runescape.wiki/w/Update:The_Blood_Moon_Rises_Tweaks_&_Fixes)).
- Royal Titans: [quoted words withheld], and [quoted words withheld].
- Yama's contracts were changed after the preview.
- No public beta was found for any of the eight.

**Style is checked against the old library.** From the [DT2 art-team section](https://secure.runescape.com/m=news/desert-treasure-ii---awakened-bosses-blood-torva--old-schools-art?oldschool=true):
- "We use a vertex grid to design all our models"
- "While remaining true to our blocky, noughties roots"
- each asset [quoted words withheld]
- [quoted words withheld]

From Mod Deagle, Associate Art Director ([Gazette June 2020](https://oldschool.runescape.wiki/w/Update:Gielinor_Gazette:_June_2020), wiki copy):
- [quoted words withheld]
- his pipeline: a creative brief ([quoted words withheld], "Support the lore", "Keep the iconic features"), reference gathering, a moodboard, concept iterations, and a finalised concept where "All of the disciplines involved … align"; then "it is off to the 2D/3D character artists".

**Not found publicly:** polygon budgets, colour or telegraph rules, statements on skeletal animation, VFX technology or
textures for these bosses, and sound credits.

## 3. (a) The package as rules with bands

The band is min to max of the eight; the median is in brackets. A rule names who sets each end. `packagecheck.py` prints all
of this. These are the parts and budgets an entirely new boss of this era has; they are not a recipe for quality.

**Model**

| rule | band [median] | ends set by |
|---|---|---|
| M1 tiles | 3 to 7 [5] | |
| M2 faces of the main form | 1,886 to 4,003 [3,730] | Amoxliatl; Hueycoatl. The single-model ceiling of about 4,100 still holds; more only by more models or forms (Titans 5,604 over two, Hueycoatl 4,907) |
| M3 models in the definition | 1 to 5 [2] | the second model is a weapon, shield, translucent veil or tiny pivot |
| M4 colours | 19 to 66 [42] | |
| M5 colours covering 90% | 7 to 25 [14.5] | |
| M6 hue families of 3% or more | 1 to 6 [3] | |
| M7 main family share | 0.41 to 0.99 [0.68] | |
| M8 translucent share of the surface | 0 to 0.46 [0.04] | |
| M9 textured faces | 0 | |
| M10 inlaid islands | 25 to 315 [52] | |
| M11 largest welded shell | 0.64 to 0.89 [0.76] | |

The colour plan in words: one or two materials carry the body, laid in MANY tones. Doom has 34 tones in its red family,
Branda 39 and Sol 31. A third material or an accent sits at about 2 to 12%. Nearly one boss in two (Doom, the Titans,
Amoxliatl) is a single-hue design.

**Skeleton and family**

| rule | band [median] or count |
|---|---|
| S1 a skeleton of its own | 7 of 8 (not Amoxliatl) |
| S2 skeleton size | 90 to 214 bones when weighted (6 of 8); 222 to 255 groups when classic (Araxxor and the Titans, 2024) |
| S3 a scaled pet on the boss's own skeleton | 8 of 8; pet scale 25 to 50 |
| S4 NPC ids | 3 to 12 [10]: boss forms 1 to 4, state forms, adds, invisible helper NPCs (hit boxes, timers, projectiles as NPCs) |

**Animation**

| rule | band [median] or count |
|---|---|
| A1 own sequences | 9 to 32 [18] |
| A2 attack sequences | 3 to 13 [8] |
| A3 idle, attack and death | 8 of 8; plus spawn, emerge or awaken in 6 of 8, and phase or transition sequences in 5 |
| A4 median length | 60 to 150 cycles [90] (1.2 to 3 seconds) |
| A5 total | 870 to 3,600 cycles [1,470] |
| A6 sequences made for it | 100% on every real one |

**Effects**

| rule | band [median] or count |
|---|---|
| E1 own effects | 6 to 45 [15], drawn from only 5 to 24 distinct models |
| E2 projectiles | 0 to 5 [3] |
| E3 effect jobs | 2 to 5 [3] |
| E4 median faces | 257 to 1,550 [550] |
| E5 median size | 114 to 638 units [338] |
| E6 colour share in the boss's sectors | 0.89 to 1.0 [1.0] |
| E7 textured effects | 0 |

**Arena** (Sol excluded: his arena is the Colosseum's)

| rule | band [median] or count |
|---|---|
| O1 own objects | 14 to 79 [20] |
| O2 animated objects | 0 to 19 [6] |
| O3 at least one mechanic object | 7 of 7 |

For arena layout use ARENAS.md and `arenacheck.py`; for the fight use FIGHTS.md and `fightcheck.py`.

**Calibration** (`--calibrate`: each real boss against bands made from the other seven):

| boss | rules held of 31 |
|---|---|
| Maggot King | 30 |
| Yama | 28 |
| Hueycoatl | 27 |
| Sol | 26 |
| Araxxor | 26 |
| Doom | 26 |
| Titans | 24 |
| Amoxliatl | 20 |

So a real new boss typically breaks 1 to 7 rules. Read OUTSIDE as [quoted words withheld], not as "fail".

**Our Warden-General as built** (`PACKAGE-warden-general.json`, output in `check-warden-general.txt`) holds 21 of 31.
- All eleven model rules hold. Model III has 3,580 faces, 26 colours, 3 families, and a main family share of 0.75.
- What is outside is everything around the model:
  - a borrowed skeleton (Tekton's base 1668);
  - no pet and no other NPCs (1 id against 3 to 12);
  - 6 sequences, half of them Tekton's (393 cycles against 870 to 3,600);
  - 2 attacks against 3 to 13;
  - 5 own effects against 6 to 45, with colour less tied to the body (0.86 against 0.89 to 1.0);
  - 12 arena pieces against 14 to 79.
- Its fight holds 8 of 17 (FINDINGS 81).

This is the owner's [quoted words withheld], in numbers.

## 3. (b) The process as a checklist for our pipeline

Steps follow Jagex's order (section 2). Our tool, brief or gate for each step is named; **GAP** marks where we have none.

1. **Pitch: theme from the region and its lore** (Varlamore = Aztec plus Roman; Mokhaiotl = demon insect; Vampyrium = maggot
   parasite in a venator's body). The output is one paragraph: what it is, where it lives, why it fights here. **GAP:** no brief
   or template; the Warden-General had no regional source. Proposed: `BRIEF-boss-pitch.md` with the creative-brief headings
   Mod Deagle lists.
2. **Mechanics first, as a fight file.** Write the fight to FIGHTS.md's 17 rules before any art. The art then follows from
   the mechanics: one boss form per state, one effect per tell, one object per hazard.
   - Tool: `research-fights/fightcheck.py`, format `research-effects2/fight/fight.py` (REDESIGN.md's fight-v2 holds 17 of 17).
   - **GAP:** no tool turns a fight file into the asset list it implies. Proposed: `packagecheck` could read the fight and
     list the states, tells and hazards still without an asset.
3. **Concept: silhouette and colour plan on paper.** One idea readable in two or three hues (BOSSES.md section 4). Set the
   main family share (0.4 to 0.99) and the tones per main family (6 to 39). Decide the accent and where it sits. Decide
   attack-style colours for the effects.
   - Tools: `research-paint/paintplan.py` (a plan format) and `stylecheck.py`.
   - **GAP:** no concept-art step (we go from words straight to mesh). There is no silhouette review either: HELM3.md's
     proposal [quoted words withheld] is unbuilt. No reference moodboard step.
4. **Skeleton of its own, designed with its family.** Choose weighted (6 of 8) or classic. Plan the pet (the same model at
   scale 25 to 50) and the adds on the same skeleton or their own.
   - Tools: `research-skeleton/skelc.py` (classic); `research-bosses/bonecodec.py` and `research-rig2` (`rig_weights.py`,
     `rig_pose2.py`, `rig_export.py`, `BRIEF-weighted-creature.md`) for weighted.
   - **GAP:** ownership of weights by a parts file is untested on an unseen boss (FINDINGS 52). No weighted boss has been built.
5. **Model to the budget.** 3,000 to 4,000 faces at 5 tiles, 1 to 5 models (body plus weapon, veil or shield), no textures,
   translucency only where the concept says (veils, ice, webs), islands 25 to 315.
   - Tools: `BRIEF-original-creature.md` with its paint, clean and boss sections; `style_npc.py`, `cleancheck.py`,
     `detailgate.py`, `research-tone/paintfit.py` and `normalfit.py`; `packagecheck` M1 to M11.
6. **Animation set.**
   - idle, walk (or none for a stationary boss), 3 to 13 attacks of 1.2 to 3 seconds;
   - spawn or emerge and death (Hueycoatl awaken 330 cycles; Maggot King 3 deaths);
   - one sequence per state (Doom burrow set; Hueycoatl shield set);
   - movement variants where the fight needs them (Maggot King: walk while attacking, strafe).

   Tools: `bone_author.py`, `rig_pose2.py`, `research-animation` and the pose harness; `BRIEF-original-animation.md`.
   **GAP:** our attack poses travel a third of the real distance (FINDINGS 52). There is no measure of choreography (overlap,
   follow-through), and no long-sequence work (200 to 330 cycles) has been tried.
7. **Effects per tell, in the boss's colours.**
   - Build each attack as a family: cast or spot, projectile, impact; an 8-direction set from ONE model; spawn, idle and despawn.
   - Reuse a generic library where Jagex does (standard projectiles by style, rock, beam, slam dust).
   - Tools: `research-effects2` (`softkit.py`/`fxkit.py`, `style2.py`, `fxtools.py` gate), `BRIEF-original-effect.md`.
   - **GAP:**
     - We have no library of our own generic effects; each Warden effect was one-off.
     - No check yet ties effect colours to the boss (E6 is the first).
     - The owner's weakest point, [quoted words withheld] (FINDINGS 79), has no measured readability rule
       against the floor.
8. **Arena pieces and hazard objects.**
   - Build a named floor kit (Maggot King's 23 pieces), state objects for mechanics (pillars x 5 states, barriers x 3),
     spawn and death floor pieces, pools.
   - Tools: ARENAS.md, `arenacheck.py`, `compose_arena.py`, `tools-fight-arena` (packing), EFFECTS2 section 2 rules,
     `obtools.py` gate.
   - **GAP:** objects that change state with a phase are listed in EFFECTS2 section 10 and not built.
9. **Family NPCs.** Pet, adds, state forms (inactive, shielded, burrowed, defeated), and invisible helper NPCs.
   - **GAP:** no brief for a pet or a minion. No helper-NPC pattern in our fight data (FIGHTS uses objects and tiles).
10. **Sound.** Araxxor's fix was an audio cue. **GAP:** we have nothing for sound.
11. **Review in motion and in the arena, then iterate on readability.** Owner sittings with blind line-ups
    (`lineup_npc.py`, `research-arenas` boards, fight films).
    - **GAP:** the films do not yet show a solve and a punishment with a moving player, hitsplats or a health bar
      (FINDINGS 79, 81).
    - **GAP:** no playtest step with a human player.
12. **Package check before the sitting.** `packagecheck.py` (this study), `fightcheck.py`, `arenacheck.py`, then the owner.

## 3. (c) What is reused, and what is always new

**Always new on all eight:**
- the boss's mesh (no recoloured older model);
- its own idle, attack and death sequences (A6 = 100%);
- a pet built from the boss's own model scaled down;
- boss-named effects (6 to 45);
- objects named for the fight, except Sol's (14 to 79).
- On 7 of 8 also the skeleton.

**Reused, measured:**
- **Skeletons.** Amoxliatl stands on the Moons of Peril skeleton of 2023. It plays 9 own sequences among 41 of its kin, so
  it is the real counterpart of our Warden on Tekton. Sol's skeleton also carries the wave Colossi of his own release.
- **Adds.** Araxxor's Araxytes are Theatre of Blood-era models on the Nylocas skeleton (1800, 2019). Yama's Judge is the 2020
  quest NPC (model 42551, base 1078), and his imps stand on an old base (1240). The Maggot King's larvae use an old grub base (678).
- **Effects.**
  - Doom uses 19 effects it does not own: a new shared library (`vfx_standard_projectile_*`, `vfx_rock_projectile_*`,
    `vfx_beam_*`) plus the Leviathan's impacts, a DT2 rock fall, Colossi dust and holy water.
  - Araxxor uses Nylocas, Venenatis and Araxyte effects.
  - The Maggot King uses the Grotesque Guardians' falling-debris shadow and Xarpus' acid spit.
- **Meshes within a package.** Doom's 7 burrow effects and its slam are one 2,021-face model with different sequences; 16
  blood splats are one model. The Hueycoatl's 16 directional shockwaves are one 170-face model and its 8 impacts another;
  Yama's spawn, despawn and summon effects share two models.

**Rule for us:** a new boss owns its body, skeleton, core animations, pet and signature effects. Adds, shadows, generic
projectiles and impacts may be reused or drawn from a library, recoloured to the boss.

## 3. (d) Where Jagex goes beyond its older style (`P beyond.py`, data/beyond.json)

The three eras compared are:
- **A** = 2017 to 2020: CoX, ToB, Vorkath, Hydra, Nightmare (16 forms, 164 boss-named effects);
- **B** = 2022 to 2023: Nex, ToA, DT2, Muspah (14 forms, 187 effects);
- **C** = these eight (9 forms, 169 effects).

Figures are medians, with the 10th to 90th centile in brackets.

**Effects are where the newest bosses go past the old style:**

| measure | A | B | C |
|---|---|---|---|
| faces | 281 (60 to 927) | 460 | **907 (170 to 2,440)** |
| largest dimension, units | 130 | 244 | **378** |
| colours | 3 | 3 | **6** |
| effects with any translucency | 62% | 60% | **76%** |
| effects with flat-shaded (crisp) faces | 11% | 27% | **41%** |
| skeletal (weighted) effects | 0 | 41 | 6 (Yama's summon effects) |

Effects are 3.2 times the faces of the raid era and three times the size, with twice the colours. Each mesh is also used
more (169 effects on 76 models).

**Meshes grow a little and get calmer, not busier:**

| measure | A | B | C |
|---|---|---|---|
| faces | 3,035 | 3,263 | **3,729** (the ceiling of about 4,100 per model is unbroken) |
| models per definition | 1 | 1.5 | **2** |
| inlaid islands | 135 | 224 | **61** (less painted noise) |
| main hue share | 0.46 | 0.48 | **0.56** |
| colours | 31.5 | 56.5 | **37** |
| translucent faces at the top end (90th centile) | 0.30 | 0.11 | **0.68** (Amoxliatl, Eldric: whole-body ice) |
| sequences | 11.5 | 20 | **16** |

**What has not moved:**
- No era uses textures on a boss or its effects (0 in every row).
- Weighted rigs arrived in era B, but two of the eight (Araxxor, the Titans, 2024) went back to classic skeletons at the
  255-group limit.
- Yama alone uses true Bezier curves (SKELETAL.md).

**For a "beyond" track, then, the measured headroom is:**
- richer, larger, many-tone effects with crisp facets and layered translucency;
- several models per boss;
- whole-body translucent materials;
- long state sequences;
- more state objects in the arena.

It is not more inlaid detail or textures. Jagex itself stays at [quoted words withheld] and checks against the old library.

## 4. What I could not establish

- **Forms.** Which NPC id is which fought form is read from internal names plus combat level, not from a spawn table.
  NPC transform lists were not decoded.
- **Roles and package membership.** Sequence, effect and object roles come from name rules (approximate; e.g. some Sol and
  Hueycoatl effects fall in "other"). A piece of the fight under a generic name is missed: Yama's glyphs and the Titans' safe
  tile are added from EFFECTS2 by hand, and Sol's crystal beams are not counted as his.
- **Reuse.** Reused effects are only those the server copy's plugin names, by a regex over its source: a lower bound,
  approximate. The Maggot King's magic projectile 1555 was missed by the regex and taken from its JSON by hand.
- **Fight structure** is measured (FIGHTS.md) for Yama and Araxxor only; the other six are not.
- **Arenas** are measured (ARENAS.md) for the Maggot King, Doom and Araxxor only. Yama's floor is set by the server, and the
  Hueycoatl, Amoxliatl, Titans and Sol arenas were not drawn.
- **Colour.** cmesh's family rule chains neighbouring hues, so a red-to-gold ramp counts as one family (Doom, Branda). The
  sector reading is given beside it.
- **The bands** are eight bosses, min to max. The checker has met one package of ours. No owner grade is tied to it, and it
  cannot see quality.
- **Public process.** No public Jagex statement was found on polygon budgets, textures, VFX technology or skeletal animation
  for these bosses. Quotes in section 2 come partly through a page summariser; two pages were re-read.
- **Not seen.** No picture of any package was looked at in this study: the visual descriptions are BOSSES.md's.
- **Release dates** are my knowledge. The Maggot King's date is the web agent's reading of its overview post.
