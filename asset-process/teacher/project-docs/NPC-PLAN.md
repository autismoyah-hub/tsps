# Plan: the original-NPC system, from base to exceptionally authored (interim Teacher, 2026-11-27, A-032)

The owner, after sitting 2026-11-30-c: [quoted words withheld] This plan follows the method set out (INTERIM.md, "How this project actually makes progress";
CAPABILITIES.md, "What the system has learned about itself"). No author brief is sent for a phase until the phase
before it has met its acceptance condition. Nothing below finished quality goes to a sitting (FINDINGS 87).

## Where the system stands (from the register, not from my eye)

| Step of the method | Creatures and bosses today | Evidence |
|---|---|---|
| 1. Harness | Done: classic and weighted NPCs drawn as the client draws them, in motion, packed by id | FINDINGS 38, 39, 45, 49 (0 differing pixels) |
| 2. Rules | Mid-sized creatures (CREATURES.md), bosses by count (BOSSES.md), paint (PAINT.md), cleanness (CLEAN.md), the package (AUTHORSHIP.md), fights (FIGHTS.md) | FINDINGS 39, 45, 60, 64, 81, 82 |
| 3. Rebuild | Mid-sized classic creatures: PASS and not told (Minotaur, Araxyte, Lynx). Classic bosses: Tekton passes plate-first, found by small detail. NEWEST-ERA bosses (own skeleton, 2023 on): NEVER REBUILT. Weighted: never rebuilt | FINDINGS 41, 54, 60, 66 |
| 4. Originals | Each kind passed once at the owner's earlier bar (hound, sentinel, ray, watcher III, Warden-General III); the owner's bar is now [quoted words withheld], and by it they are [quoted words withheld] | HANDOFF, COUNT |

Fable's own conclusion: [quoted words withheld] For the newest kind of boss, step 3 has been skipped. That is the hole this plan fills before the Unsetting.

## Phase 1. Diagnose the gap on our BEST work, in the owner's words (no new assets)

Why: the pitch sitting judged blockouts, so its words are about work made below the bar on purpose. The gap that matters
is between our best finished originals and real ones. Method (as for helms, rooms, effects): the owner's own words on
references, turned into measurements.
- 1a. A blind sitting of FINISHED assets only: each of our best originals (Warden-General III, watcher III, the star
  hound, the Armillary sentinel, the Zodiac ray) in a line-up with real creatures of its size and era, in motion, at game
  size and close. Questions: which is ours; how sure; and in words, what gave it away. Predictions in COUNT first.
- 1b. A study (Opus, pre-registered as in FINDINGS 80: at most 18 measures written before computing, leave-one-out) of
  the words the owner gives, on those assets against their real neighbours; plus the construction questions the record
  lists as unknown ([quoted words withheld], FINDINGS 39 and STATE): edge flow, plane sizes by part, how forms
  are turned (rings and loops), how silhouettes are cut.
- Accept when: the owner has answered; each answer is either a surviving measure with a band, or a seen fault written
  down with pictures (FINDINGS 80 showed some faults are seen, not counted; those become review questions).

## Phase 2. Rebuild the newest kind blind, the Unsetting's nearest relatives first

Why: no author here has reproduced a 2023-on boss. Until one has, an original of that kind has no construction to stand on.
- 2a. Choose two real targets by measurement, not by taste: the newest-era bosses closest to the Unsetting in kind (a
  floating, non-humanoid body of one simple mass with prominences or appendages; translucent or glowing material;
  effects-heavy), from the AUTHORSHIP.md package data and the cache. Also one mid-sized newest-era creature as a
  control. Build each a version-2 boss sheet (FINDINGS 54: rest views as paint, lit, light and ink; a pose set;
  enlargements), extended to translucency.
- 2b. Blind rebuilds by Astra authors under `BRIEF-creature-rebuild.md` with its boss section (plates and marks first,
  every ink line an edge, flat plates, faces budgeted by pose, light before paint, never repair tone with paint), its
  own skeleton included. Gates: `gate_npc.py`, `detailgate.py` (small marks), the tone split of FINDINGS 54.
- 2c. Every rebuild report carries "How this boss is built" (as the helm rebuilds did, FINDINGS 41): the construction
  an original must copy in kind.
- Accept when: each rebuild passes the gates AND the owner, blind in motion, cannot tell it or grades it Strong. If a
  target fails twice, the two-strike rule: change the approach (a smaller target, a better sheet), not a third try.

## Phase 3. The Unsetting, by the process that worked, carried by Phase 2's construction

- Brief: `BRIEF-original-creature.md` with its clean and paint sections, and the order of work of
  `night/prompt-boss-original.txt` (the Warden-General III order: measure at least five references; ONE reference for
  proportion and construction, which is now a Phase 2 rebuild; plates before paint; light before paint; at least twelve
  rounds judged beside the references; a worked rebuild as the model of the skin), plus Phase 1's measures and review
  questions, plus `builds/boss-pitch-k/` for the idea only (its blockout is discarded).
- Its own classic skeleton (the rigid route proven by the Armillary sentinel, FINDINGS 38); textures and transparency only
  where its references use them (FINDINGS 82, 83).
- Two authors, independently; the owner sees only finished models beside real bosses, in motion.
- Then, in order and only after the model is Strong at the new bar: its animations (qualified domain), its effects
  (qualified), its arena (ARENAS.md), its fight (FIGHTS.md), its pet.

## What I will not do

- Send any brief for Phase 2 or 3 before the phase before it is accepted.
- Show the owner anything unfinished, or a checker's pass as a sign of quality.
- Run many fast candidates for quality; parallel work only where each item gets the full process.
- Change a gate threshold, or declare a process proven, on my own.

## Status

Phase 1: the owner said go (2026-11-27). 1a built as sitting 2026-11-30-d, NOT blind (the owner has seen all three originals, so a pick would test memory): Warden-General III beside Tekton, watcher III beside the Minotaur, the star hound beside the Lynx, each under the real one's own sequences (lineup_npc.py, `{ROOT}/sittings/phase1/`); per creature, how close to a genuine OSRS asset and what most separates it, in words. The sentinel and the ray are left out: the line-up tool draws ours under a real creature's sequences, and they have skeletons of their own.

## Sprint while the owner is away (owner, 2026-11-27: [quoted words withheld])

Five workstreams run in parallel. Each starts with tools proven against a known answer, and an author is ordered only
after that stage's acceptance condition is met (the rule of this plan). Results wait for the owner as finished boards.

| | Workstream | Stage A (now) | Accept A when | Stage B (after A) |
|---|---|---|---|---|
| W1 | Render quality of sitting pictures ([quoted words withheld]) | Astra tools: audit our pictures against the client at its real resolution (resolution per tile, GIF's 256 colours, resampling, gamma); presets for faithful 1x, integer-magnified, and high-resolution (the same rasteriser at 2 to 4 times the pixels); animated WebP in place of GIF | a board where the faithful preset equals a client capture pixel for pixel, and every finding has a number | all later sitting pictures use the presets; re-render the open sitting 2026-11-30-d |
| W2 | Phase 1b: the gap on our best originals, from the owner's recorded words (no sitting) | Opus study: pre-registered measures of the owner's words across COUNT.md on Warden-General III, watcher III, the hound, the sentinel and the ray against real neighbours; how real artists split a surface (edge flow, plane sizes, loops) on the newest bosses | measures survive leave-one-out, or the seen faults are written as review questions with pictures | lines added to the creature brief, marked UNTESTED |
| W3 | Phase 2: rebuild the newest kind blind: the Royal Titans Branda 12596 and Eldric 14147 (2024, classic skeletons of their own, elemental, Eldric 68% translucent: the Unsetting's nearest kin by AUTHORSHIP.md) | Astra tools: version-2 boss sheets for both with a translucency layer; the creature gate's ceiling on both (real through our encoder = 0); rebuild folders | sheets complete; gate ceiling PASS at 0 on both | two independent Astra rebuilds of each, boss section of BRIEF-creature-rebuild.md; then Phase 3 (the Unsetting) from whichever passes |
| W4 | Learn from the HD era (rev 561 to 614) by full rebuilds, then originals | Astra tools: an HD rebuild sheet and an HD gate (rebuild scored against the HD model converted to classic, texture assignment included), proven on a real HD model against itself and on damaged copies | ceiling 0 and damaged copies FAIL | three Astra rebuilds from TECHNIQUES.md (lamp stall 22772: textures on translucent geometry; diving apparatus 7535: textured equipment; HD spot 2768: alpha bands); then originals by the same techniques |
| W5 | Learn from RS3 (build 951) | Astra tools: RS3 model and material reader, a Blender viewer with materials, a technique study as for the HD era, proven on models present in both caches | reader proof at 0 differing on shared models | rebuild targets chosen from its TECHNIQUES |

Sprint stage A dispatched 2026-11-27 (clock): W1 Astra d6cb8ba6 (`studies/tools-render-quality`), W2 Opus agent of the interim session (`studies/research-gap`), W3 Astra 99e3f84e (`studies/tools-titan-sheets`), W4 Astra d48971bd (`studies/tools-hd-gate`), W5 Astra 68037195 (`studies/tools-rs3`). Sitting 2026-11-30-d stays built but the owner will not take it now; W1 re-renders its pictures.

Phase 2 status (2026-11-28): Branda (12596) rebuilt blind by an Astra author PASSES the gate on every line (FINDINGS 102); blind moving pictures for the owner are ready (`{ROOT}/sittings/phase2-branda/`). Accept when the owner cannot tell it. Eldric (mostly translucent) fails for both authors. Phase 3 (the Unsetting) will stand on Branda's "How this boss is built" once the owner has seen the blind line-up.

## On the Astra reset (2026-11-29 09:51 local), in this order

1. Collect sitting 2026-11-30-e if the owner has answered (`python tools/collect_sitting.py 2026-11-30-e`); if they cannot tell Branda blind, Phase 2 is accepted and Phase 3 (the Unsetting) is briefed on `titan-rebuild-12596-b/REPORT.md` "How this boss is built" plus BRIEF-original-creature.md (with its UNTESTED gap lines) and the Warden-General III order of work; two authors, at least twelve rounds, nothing shown until finished.
2. Re-send the returns the usage limit cut (each prompt is in `{ROOT}/prompts/`, unchanged): `prompt-titan-rebuild-12596-a-return2.txt` (tone only), `prompt-titan-rebuild-14147-a-return2.txt`, `prompt-titan-rebuild-14147-b-return2.txt`, `prompt-hd-rebuild-lampstall-return2.txt`. Gate each with the commands in their READMEs.
3. RS3 blind rebuilds: one Astra author each for `builds/rs3-rebuild-crossbow`, `rs3-rebuild-havoc-hood`, `rs3-rebuild-kerapac` (briefs on the model of `night/prompt-hd-rebuild-*.txt`, stating plainly any faces the original leaves untextured, and a view-by-view silhouette check; gate `studies/tools-rs3/rs3gate.py`, read the VERDICT line, not the exit code).
4. HD originals from the technique that passed (textured equipment: the diving apparatus male worn form): one original by `BRIEF-original-armour.md` plus `studies/tools-hd-era/TECHNIQUES.md`, judged only when finished.
