# Running the asset system without Fable

Owner's direction, 2026-11-26 (DECISIONS A-026): the system must run on less expensive models, including in the
Teacher's seat for the domains that are proven; Fable returns in bursts to review and improve, when the owner starts it.
This file is the design and its current status, and the Fable Teacher's to keep. A-027 narrows it: the Teacher's role (direction, planning, research, method, review) stays with Fable; what transfers is modelling and the running of PROVEN processes for modelling missions in qualified domains. A Producer runs a proven process; it is not a second Teacher.

## Three seats

| Seat | Who | Does | Does not |
|---|---|---|---|
| **Author** | a less expensive model (first: GPT-6.2 Astra), one run per piece | builds one piece from a brief in `asset-system/author-pack/` | change tools, rules or other builds |
| **Producer** | a less expensive model in the Teacher's seat, entry file `teacher/project-docs/PRODUCER.md` | takes the owner's orders in QUALIFIED domains; writes task lines; runs authors; runs the gate; sends back failures; assembles sittings; keeps STATE, AREAS, COUNT; folds authors' reports into a proposals file | change `RULES.md`, the briefs' method, the gate's thresholds or the domain register on its own; open a domain; call anything good without the gate and a sitting |
| **Fable review** | Fable, started by the owner as usage allows | reads the review pack, judges what the gate cannot, rules on the Producer's proposals, fixes method and tools, moves domains in the register, and does the research on unproven asset types | routine authoring |

Why the Producer may not change the method alone: the method was built by measuring real assets and by the owner's
sittings. A weaker judge drifting the rules is the one failure that would not show in any single asset. Proposals with
evidence are welcome and are ruled on at each Fable review; between reviews the owner's sittings are the judge.

## What replaces Fable's judgment day to day

1. **The gate** (`tools/gate.py`, being built): PASS, BORDERLINE or FAIL from measurements, not opinion. Thresholds are
   set from the owner's own sittings and only Fable review or the owner changes them. A piece that does not PASS does
   not reach the owner; it goes back to an author with the gate's lines, at most twice, then is parked for review.
2. **The owner's sittings**, unchanged: blind where a real twin exists, grades and a ranking for originals, the
   Producer's predictions recorded first, the page free of the Producer's opinions. `docs/verdicts/COUNT.md` continues.
3. **Calibration.** In each batch of pieces the Producer includes one held-out rebuild of a real asset of the same
   kind (the gate scores it against the real mesh). If calibration rebuilds start failing, authoring in that domain
   stops and waits for review: the authors or the pack have drifted.
4. **The known blind spot.** The gate measures closeness and cleanliness. It cannot tell a dignified design from a
   silly one: the Fable Teacher missed a helm the owner called [quoted words withheld]. For originals, design is judged by the owner,
   so originals go to sittings as two or three candidates, not one.

## Domain register (only Fable review or the owner edits this table)

| Domain | Status | Evidence | To qualify |
|---|---|---|---|
| Scenery objects, rebuild | **Qualified for Astra** | Drawers 25702: Astra's rebuild scored 2.76 in 12 minutes; the owner could not find the real one among four | |
| Scenery objects, original set pieces | **Qualified for Astra** | map table: the owner chose Astra's over Sol's (2026-11-26-e); house style followed | a grade from the owner on an Astra piece beside the Fable-made set |
| Weapons, rebuild | **Qualified for Astra** | Osmumten's fang: Astra's rebuild matches Fable's (outline closer, tone slightly behind; FINDINGS 31) | |
| Weapons, original | Provisional | Fable authors only (three graded Strong) | one Astra original beside the Meridian arms, graded by the owner |
| Armour, sleeveless and simple slots, rebuild | **Qualified for Astra** | Iron chainbody: Astra's rebuild equals or beats Fable's on every measure (FINDINGS 31) | |
| Armour, original single pieces | Provisional | Fable authors only | one Astra original piece for an existing set |
| Armour, sleeved bodies, legs, full sets | Research to provisional | one set by Fable authors, graded Strong; needed hand labelling and new rules on the way | the labelling tools (in build), then an Astra piece |
| Rooms | Provisional | one room by a Fable author, graded Strong | `room.py`; a second room by a less expensive author |
| Capes, skirts, the female figure, inventory and ground models | Research (Fable) | not started | |
| People, creatures, bosses | Research (Fable) | first measurements only | |
| New animation, bone-weight animation | Research (Fable) | not started | |
| Tools and system work, to a written specification | Provisional, leaning qualified | Astra built the gate and three tools exactly as specified, tested, with an exact report; the faults found were in the Teacher's specification | Fable's review of the corrected gate |

Sol (GPT-6.2) declines to rebuild real game objects, so it cannot run calibration; it can author originals.
Opus 5.5 did the rebuild but was the weakest of three on the one object tried.

## The Fable review, started by the owner

The owner starts a Fable session with: "Run the Fable review in teacher/project-docs/HANDOFF.md."
Fable then:

1. Runs `tools/review_pack.py` (to be built): everything since the last review in one folder: each piece's gate
   output and pictures, the owner's verdicts and the Producer's predictions, calibration results, pieces parked after
   two failures, the Producer's proposals file, tokens and time per accepted piece.
2. Looks at every piece the gate passed and names what the gate missed; checks the Producer's records against the
   files; rules on each proposal (adopt, reject, test); fixes rules, briefs, tools and thresholds; updates the register.
3. Spends what remains of the burst on the research domains, in the order of `PLAN.md`.
4. Leaves: a dated review note in `docs/reviews/`, the register and STATE current, and a short list for the Producer.

A review should cost a small part of what directing every asset costs, because it reads a prepared pack instead of
watching the work.

## Before the first Producer is accepted

1. Qualification runs for weapons and armour (running). 2. The gate, built and reviewed. 3. `PRODUCER.md` and
`review_pack.py` written. 4. **A trial:** a less expensive model takes the Producer's seat for one real order in a
qualified domain (a small furniture set), end to end, with the Fable Teacher watching and not helping. Accepted if
the gate passes, the records are true, and the owner grades the result as they graded the Fable-directed set.
5. The owner is told plainly what the trial showed, including where the Producer fell short.

## Which model in which seat (owner, 2026-11-26)

The owner: [quoted words withheld]
Reading: Astra by default, since it draws on a different subscription and leaves Claude usage for Fable; Opus only in
a seat where it measurably beats Astra. Authors: Astra (it beat Opus on the one object both did). Producer: unknown,
so the trial is run with both, on the same order, and the owner sees both results unlabelled.

Platform fact found 2026-11-26: a Cuttle session will not accept a changed assignment from the session that started
it (only from the owner). A piece that goes back to an author is therefore a NEW session whose prompt names the build
folder, the gate's lines and what to change.

## The trial's result and the seats (2026-11-26, after sitting 2026-11-26-g)

Both Producers ran the same order unaided (FINDINGS 32, 33). Both calibration rebuilds passed blind with the owner. The
owner chose an Opus-directed chest [quoted words withheld], called all four [quoted words withheld], and graded the choice Strong.

- **Authors: Astra.**
- **Producer, routine orders in qualified domains: Astra.** It is accepted for the seat. It keeps the stricter records
  and costs no Claude usage.
- **Producer, orders the owner marks as showpieces** (player reward gear, a set's centrepiece): **Opus**, when the owner
  chooses to spend the usage. It returned passing pieces that did not read as the thing ordered, and that showed in
  the result.
- What Opus did that the charter did not ask for is now asked of every Producer (PRODUCER.md steps 2 and 6).
- **Fable:** reviews in bursts, and the research domains, which now include what makes player gear desirable.

## Register changes after sitting 2026-11-27-a

| Domain | Status | Evidence |
|---|---|---|
| Rooms (shell and floor kit, original) | **Qualified for Astra** | observatory shell by an Astra author, graded Strong; furnished from three Producers' orders |
| Weapons, original | **Qualified for Astra** | war hammer graded Strong; one return needed on the mace for readability |
| Armour rebuilds, legs and sleeved bodies | **Qualified for Astra** | rune platebody and platelegs not told from the real ones in motion |
| Capes | Provisional | first cape graded Strong; one piece, no brief yet |
| Animation on an existing skeleton | Provisional, leaning qualified | four animations all [quoted words withheld], one from `BRIEF-original-animation.md` alone |
| NPC from our own worn pieces | Provisional | warden graded Strong |
| Creatures on a borrowed skeleton | Research | hound and spider Usable, boss Usable; the owner names this the hardest area and sees the hound as dated |
| Player reward gear (design) | Research | three rounds of helms rejected; the study is unconfirmed |
| Complete items (icon, ground, female, definition) | Provisional | two items read right; female forms of the new set in work |

## Register changes after sittings 2026-11-27-b and -c

| Domain | Status | Evidence |
|---|---|---|
| Player helms, original | Provisional (process proven once) | `author-pack/BRIEF-player-helm.md`: fit to a reference shell first (`fit.py`), then sharpness, depth and detail measured against the owner's five references; the Astra author's helm passed the fashionscape test (2026-11-27-c) |
| Helm rebuilds | **Qualified for Astra** | five reference helms pass the gate; the owner found two of the three shown, guessing, and graded all Strong |
| Creature rebuilds from a sheet | Provisional | Minotaur, Araxyte, Lynx pass `gate_npc.py`; the owner found one of two, guessing; both Strong |
| Creatures on their own skeleton, rigid | Provisional | the Armillary sentinel, graded Strong |
| Showpiece authoring | Astra | on one brief the owner chose the Astra author's helm over the Opus author's |

## Register changes after sitting 2026-11-28-a

| Domain | Status | Evidence |
|---|---|---|
| Player armour sets, original | Provisional, leaning qualified (process passed twice) | helm, then body, legs, gauntlets and boots by `author-pack/BRIEF-player-helm.md`'s process; the whole outfit passes the fashionscape test; all by Astra authors |
| Creatures on a borrowed skeleton, original | Provisional (process passed once) | star hound under the rewritten `BRIEF-original-creature.md`: Strong, [quoted words withheld] |
| Boss rebuilds, classic | Research to provisional | Tekton: gate FAIL on tone, found by the owner on the same fault, graded Strong; the gate's limit stands |
| Effects (spot animations, projectiles, floor marks), original | Provisional | the sentinel's fight set graded Strong; rebuilds owed (three running) |

## Register changes after sitting 2026-11-28-b

| Domain | Status | Evidence |
|---|---|---|
| Effect rebuilds from a sheet | **Qualified for Astra** | three pass the gate; two shown blind, neither found |
| Creature animation on an existing skeleton, from the brief | **Qualified for Astra** | player attack and a creature attack both [quoted words withheld]; five animations, all [quoted words withheld] |
| Creatures on a borrowed skeleton, original | Research (DOWN from provisional) | hound Strong, stone watcher Rework under the same brief; the style check did not see the difference |
| Rooms run end to end by a Producer | Provisional: orders yes, composition no | armoury Usable: [quoted words withheld] |
| Weighted creatures on their own skeleton, original | Research, one Strong | the Zodiac ray; weights by rule still do not pass the gate on a boss |
| Boss rebuilds | Research | Tekton and Nex found on [quoted words withheld]; plate-first method untested |

## Register changes after sitting 2026-11-29-a

| Domain | Status | Evidence |
|---|---|---|
| Effects, original sets from `studies/research-effects2/BRIEF-original-effect.md` | **Qualified for Astra** | two creatures' sets graded Strong, the second from the brief alone |
| Animated-object rebuilds | Provisional | three pass the object gate; not yet shown blind |
| Rooms composed to `author-pack/ROOMS.md` | Provisional (passed once) | armoury Usable to Strong, [quoted words withheld]; one room |
| Boss rebuilds, classic, plate-first | Provisional | Tekton passes the gate; the owner still finds it by [quoted words withheld]; Strong |
| Creatures, original, by the helm's process | Research, improving | watcher Rework to Usable; shape and depth fixed; paint style and detail are the open fault |

## Register changes after sitting 2026-11-29-b

| Domain | Status | Evidence |
|---|---|---|
| Our assets in the real TSPS client (worn set, icons, object, creature, weapons attacking), local demo | Seen and accepted | four questions, all Yes; `{TSPS_WORK}/notes/demo/DEMO.md` |
| Creatures, original: paint by plan (`studies/research-paint/PAINT.md`) | Research, improving | repaint chosen and [quoted words withheld]; still [quoted words withheld], features [quoted words withheld], less [quoted words withheld] than the references; study in `studies/research-clean/` |
| Wall and floor decorations | NOT gateable yet | the gate assumed floor-standing objects; repair in work; no Producer may order them until `author-pack/WALL-DECOR.md` exists |

## Register changes after sitting 2026-11-30-a

| Domain | Status | Evidence |
|---|---|---|
| Wall and floor decorations (`author-pack/WALL-DECOR.md`) | **Qualified for Astra** | shield rebuild not told from the real one, Strong; ten kinds of originals in a Strong room; the room-side tone limit is no longer provisional |
| Rooms composed to `ROOMS.md` by a Producer, with ordered pieces | **Qualified for a Producer** (passed twice) | armoury finished: preferred to the earlier Strong room; Strong |
| Animation sets and locomotion on an existing skeleton | Provisional (passed once) | hound idle, walk, death: Yes; brief section to be written from `builds/anim-hound-set/REPORT.md` |
| Creatures, original, humanoid, by the helm's process plus `studies/research-clean/CLEAN.md` | Provisional (passed once) | watcher III Strong ([quoted words withheld]); BUT its face features still read as stuck on (Yes), and the repaint [quoted words withheld] |
| Bosses, original, classic, on a borrowed skeleton (`night/prompt-boss-original.txt`) | Provisional (passed once) | Warden-General III Strong, [quoted words withheld] to the first; holds in motion |
| Player gear, a SECOND set from `BRIEF-player-helm.md` alone | **FAILED once** | Umbral helm A: fashionscape No: [quoted words withheld]; the brief stays [quoted words withheld]; do not order the other pieces |
| Interfaces | Research | harness exact; first three blind rebuilds from rules FAIL; rules being revised |


## Register changes after the owner's word of 2026-11-28 ([quoted words withheld])

| Domain | Status | Evidence |
|---|---|---|
| Creatures and bosses, original, reworked to the gap lines (`BRIEF-original-creature.md`, additions of 2026-11-28) | Provisional (improved once each) | watcher IV, Warden-General IV, star hound IV judged improvements on III; not yet [quoted words withheld] |
| Boss rebuilds of the newest kind (own classic skeleton, 2024) | Gate PASS once, owner's blind pick pending | Branda 12596 rebuilt blind passes every line (FINDINGS 102); Eldric (mostly translucent) fails |
| Review pictures | Presets adopted | `studies/tools-render-quality/PRESETS.md`: faithful 1x, magnified, high resolution, lossless WebP |
| HD-era rebuilds (rev 614) into our format | Research, one PASS | diving apparatus male worn PASS; ground BORDERLINE; three others fail (FINDINGS 91, 92, 99, 100) |
| RS3 (build 951) | Harness | all 146,939 models decode and re-encode byte-identical; rebuild packets ready (FINDINGS 103) |
