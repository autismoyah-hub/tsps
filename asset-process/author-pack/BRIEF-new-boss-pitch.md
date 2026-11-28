# Brief: pitch an entirely new end-game boss (theme, fight, package plan, blockout)

Status 2026-11-27, after sitting 2026-11-30-c: FAILED. Do not use as written. Its blockouts (1,500 faces, flat colours, one session) were judged by the owner as assets, ten of eleven far below the bar (FINDINGS 87). It bypassed the creature, cleanliness and paint processes that produced Strong creatures. Kept for the record. Earlier status: NOT A PROVEN PROCESS. First use. The owner wants "entirely new
bosses/assets that are grounded in real jagex authorship design practices", and noted that our new assets, even
those heavily carried by a real one (our Warden-General on Tekton's skeleton), fall short of their reference. This
brief is the FIRST stage only: a pitch the owner chooses from. Full modelling, rigging, animation and effects come
after the owner's choice, by other briefs.

You are an authoring agent for an Old School RuneScape private server (TSPS client, cache revision 242). Your track
(FAITHFUL or BEYOND), your region seed and your build name NAME are in the task line.

## Read first (all on this machine)

1. `{ROOT}/studies/research-authorship/AUTHORSHIP.md`: what an entirely new Jagex boss consists of
   (the package rules with bands), how Jagex makes one (process, with quotes), and where Jagex itself goes "beyond".
2. `{ROOT}/studies/research-fights/FIGHTS.md` and `TABLE.md`: the structure of 16 real fights (17 rules:
   distinct answers, tells, punishment, failures that help the boss, conflicting overlaps, phases, escalation, pace).
3. `{ROOT}/studies/research-arenas/ARENAS.md` (how real arenas are built) and
   `{ROOT}/studies/research-creatures/CREATURES.md`, `research-bosses/BOSSES.md`,
   `research-clean/CLEAN.md`, `research-paint/PAINT.md` (how real creatures and bosses are built and painted).
4. `{ROOT}/author-pack/RULES.md`, `BRIEF-original-creature.md` (tools, the Blender route, what is
   not allowed).
5. FINDINGS 79 to 82 in `{ROOT}/FINDINGS.md`: the owner's latest words.

## The tracks

- FAITHFUL: everything inside the measured bands of AUTHORSHIP.md, FIGHTS.md and ARENAS.md. It must read at once as
  the game's own newest content.
- BEYOND: deliberately past what Jagex does, in the directions the owner named ("more geometry, textures,
  transparency, and richer animations/effects") and the ones AUTHORSHIP.md measured Jagex itself moving in
  (multiple models, whole-body translucency, far richer effects, long state animations, arena objects with states).
  Say which rule you leave and by how much, and why it makes the boss more striking. It must still read as RuneScape,
  not as another game.

## What you deliver, in `{ROOT}/builds/NAME/`

1. `PITCH.md`, one page: the boss's name; its theme and its place in the world (region, lore, why it is there, in a
   few sentences; original, not an existing Jagex character); what the player feels fighting it; its look in words
   (silhouette, materials, colour plan as hue families and shares); its forms or phases. No existing boss reskinned.
2. `fight-v1.json` in the fight format (read `{ROOT}/studies/research-effects2/fight/fight.py` and
   the optional fields documented in `research-fights/fightcheck.py`), with every mechanic's tell, solve and
   punishment, and the output of `python {ROOT}/studies/research-fights/fightcheck.py fight-v1.json`.
   FAITHFUL: all 17 rules hold. BEYOND: at most three outside, each explained.
3. `PACKAGE.json` in packagecheck's format (`research-authorship/packagecheck.py`), the plan of everything the boss
   needs (models, skeleton of its own, pet, NPC ids, sequences with lengths, effects, arena objects), and the output
   of packagecheck on it, with the same limits as the fight.
4. A BLOCKOUT: a low-detail model (at most 1,500 faces, flat colours by material) of the boss in its rest pose and in
   two key poses of its signature attack, made by script through the Blender route, at true game scale. Draw it with
   `{ROOT}/tools/lineup.py` beside three real bosses of similar size (choose them; say which): front,
   three-quarter and side at game size, and the same as BLACK SILHOUETTES (a sheet `silhouette.png`). A board
   `board.png`: rest pose and the two key poses beside the three real bosses.
5. `ASSETS.md`: the fight turned into the list of assets it needs (each model, sequence, effect, object, NPC id), each
   with its size, its role in the fight and which brief would make it.

## How to work

- Scripted builds only; write files with your file-writing tool, never shell heredocs; Python is
  `{PYTHON} -I`. Work only in your build folder and `render/NAME-*`.
- Up to six rounds on the blockout: each round, look at the silhouette sheet beside the real bosses and write in
  `notes.md`: does it read at once, from across a room, as one creature with one idea? What would a player call it?
- Not allowed: reading another author's pitch folder (several authors pitch in parallel, independently); copying,
  importing or tracing a real mesh; sending anything off this machine; killing a process you did not start.

## Report (to `{ROOT}/builds/NAME/REPORT.md`, or as your final message if your session may not write it)

The pitch in five lines; fightcheck and packagecheck outputs; the boards; for BEYOND, every rule left and why; what in
your own blockout still looks weakest beside the real bosses; where this brief was silent.
