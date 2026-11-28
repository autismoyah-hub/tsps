# The asset process

How AI agents make game assets for an Old School RuneScape server that look as if they belong in the game, and how
one person, the owner, stays the only judge of whether they do. The method is the same for every kind of asset:
furniture, rooms, wall decorations, weapons, armour, capes, creatures, bosses, animations, effects, arena objects,
interfaces.

## 1. The method in four steps (and two more)

Each kind of asset climbs the same ladder. A kind does not move to the next step until the one below it is shown.

1. **A faithful harness.** Our tools must draw what the game client draws: the same model decoder, lighting,
   painter and camera, so that a picture we make of a real asset is the picture the game makes. Proven by drawing
   real assets both ways and counting differing pixels (the target is 0). Until this holds, every later judgement is
   about our renderer, not about the asset. `tools/` and `client/` are the harness.
2. **Rules with numbers.** Measure the real assets of a kind and write what they share as rules with numbers: face
   counts by size, how many colours and how they are spread, how edges are made (tight rings and tone steps, not
   bevels), welding, priorities, how rooms are furnished, how fights are paced. Each study in `studies/` holds its
   rules in its `*.md` file and the scripts that measured them. A rule is a hypothesis until an asset built to it
   passes a sitting.
3. **Blind rebuilds.** An author rebuilds a real asset from a reference sheet (pictures and numbers that show
   everything except the mesh) without reading the real mesh. A gate scores the rebuild against the real one. Then
   the owner sees it in a blind line-up beside the real one: can they tell which is ours? Rebuilding is what teaches
   the construction; in every kind where it was skipped, the first originals failed.
4. **Originals from briefs.** New designs, written from a brief that carries the rules and the construction the
   rebuilds proved, judged by the owner in a sitting beside the game's best of the kind.
5. **Transfers.** Less expensive author models do steps 3 and 4 from the brief alone, more than once. This is
   what makes the process run without its first authors.
6. **In the game.** Packed, loaded by id and seen in the client the server runs (`studies/research-live`,
   `studies/research-tsps`, `studies/research-items`).

## 2. Roles

- **The owner** judges pictures and nothing else: grades, picks, and words about what is wrong. The owner's time is
  for judging art; never bring them a tool problem or a question an agent could answer by trying.
- **The Teacher** directs the work: decides what is built next, designs every experiment, writes the briefs and the
  rules, builds the gates and the sittings, records predictions before a sitting and compares them after. The role
  is in `teacher/` (`SKILL.md`, `core/`, `templates/`, `playbooks/`, `exam/`); a project binds it in
  `teacher/project/` (start from `teacher/project-template/`).
- **A Producer** runs production in a kind that has reached step 5, on a less expensive model, without changing the
  method. See section 5.
- **Authors** build one piece each, in Blender or by script, from a brief and a task line, and write a report. They
  may draw real assets in line-ups; for a blind rebuild they may not read the real mesh or run its scorer.

## 3. The tools, by what they do

| Need | Tool |
|---|---|
| Read the cache, decode real models | `tools/osrscache.py`, `tools/anatomy.py`, `tools/real_dump.py` |
| Write our models in the game's format | `tools/author.py` (by script; worked example `examples/barrel/build.py`), `tools/blender_kit.py` + `tools/blender_export.py` (from Blender) |
| A real asset in Blender | `tools/real_dump.py KIND ID OUT.json`, then `blender -b -P tools/real_to_blender.py -- OUT.json OUT.blend` |
| Draw and compare | `tools/lineup.py` (objects alone on terrain, numbered, key kept aside), `tools/figure.py` and `tools/pose.py` (worn gear on a figure), `tools/assemble.py` (an outfit), `tools/vignette.py` (pieces together), `tools/scene.py` (a scene), `studies/research-creatures/lineup_npc.py` (creatures in motion) |
| Reference sheets for blind rebuilds | `studies/research-creatures/sheet_npc.py`, `studies/research-bosses/sheet_bone.py`, `studies/tools-titan-sheets/`, `studies/tools-hd-gate/hdsheet.py`, `studies/tools-rs3/rs3sheet.py`, `tools/wire.py` (wireframe study of real neighbours, never of the held-out piece) |
| Score | the gates in `gates/README.md` |
| Picture quality for sittings | `studies/tools-render-quality/quality.py` (presets: faithful 1x, integer-magnified, high resolution) |
| Review pages | `sittings/make_sitting.py`, `sittings/collect_sitting.py` |

## 4. Gates, checkers and sittings

A **gate** compares a piece with what it must match and prints one line per measure and a last line
`VERDICT PASS|BORDERLINE|FAIL` (exit 0, 1, 2). INFO lines never decide. Gates exist for rebuilt and original
scenery, worn gear, weapons, creatures, skeletal creatures, small details, interfaces, HD and RS3 rebuilds
(`gates/README.md`). A **checker** (rooms, arenas, fights, boss packages, cleanness, paint style, helm fit, the gap
to real creatures) puts a piece beside the measured real population: a guard, not a judge.

Rules about gates:
- **The ceiling first.** Before trusting a gate on a kind, gate a real asset re-encoded through our own encoder
  against itself. If the real one fails, the measure is wrong, not the author.
- **A gate does not judge design.** PASS is the floor. The owner's sitting decides.
- **Never loosen a threshold to pass a piece.** A threshold changes only with evidence across pieces, by the
  Teacher, recorded.
- **Authors of a blind rebuild never run its scorer.** The Teacher or Producer gates.

A **sitting** is a review page of about ten minutes (`sittings/README.md`). Every item judged stands beside the
real game asset it should stand beside, at the same size. A line-up asks which is ours and how sure, hides the key
until the pick is locked, then asks for a grade; a pair asks which side is closer to the game; a last box takes
anything else. Pictures use the faithful preset first (game size), then close. Nothing unfinished is shown, and
no agent's opinion is on the page.

## 5. Running production (the Producer)

One order, start to finish:

1. Restate the order in one sentence and name its kind. If the kind has not reached step 5, stop: it needs the
   Teacher.
2. Write the task lines, copying the form of an earlier task line for the same brief (the prompts in `prompts/`
   show what a task line gives: the piece, its NAME and folder, size and face ranges, lighting values, what it must
   meet). For an original, order two or three candidates from separate author runs and give each a different
   stated direction (form, closure, prop); separate runs otherwise converge on one design. For worn gear, say who
   wears it: a player (it must also be something a player wants to be seen in) or an NPC.
3. Add one **calibration rebuild** of a real asset of the same kind, never rebuilt before, to the batch.
4. Run the authors, one piece per run. Never edit an author's build yourself.
5. Gate every piece. PASS goes on. BORDERLINE or FAIL goes back as a NEW author session whose prompt names the
   build folder and pastes the gate's lines (a started session takes no changed assignment).
   **Two-strike rule:** at most two returns. After two, an original still not at PASS is parked with a note; a
   rebuild target that fails twice means change the approach (a smaller target, a better sheet), not a third try.
   If the calibration rebuild still FAILs, stop the batch: the authors or the pack have drifted. A BORDERLINE
   calibration on one measure is a near miss, not drift: record it and carry on.
6. Look at every passed piece yourself in the game's renderer beside real ones and beside its set. Ask first:
   with no caption, would a player name it as the thing ordered? You may return a piece once for what the gate does
   not measure, with your picture and your words. You may not pass a piece the gate failed. Before returning a
   rebuild for a failing score, check the score is reachable (gate the real one re-encoded).
7. Write your predictions down before the owner sees anything.
8. Build the sitting: a blind line-up where a real twin exists; candidates side by side beside the game's best of
   the kind; a grade and a ranking asked.
9. Collect the answers word for word, compare them with your predictions, and say plainly where you were wrong.
10. Close every author session you started once its report is read. Keep the records.

A Producer may write task lines, order runs, return and park pieces, fix a tool that crashes (with a test), and
propose rule or tool changes with evidence (the piece, the picture, the numbers, the author's words). A Producer
may not edit the rules, a brief's method, a gate or its thresholds, or call anything good on their own word.
Authors' reports often say a rule misled them or a tool is missing: copy each such line into the proposals with
the build's name; that is the system's best source of improvement.

**Rooms** are composed, not just ordered: count from the room's plan what it needs (by the measured density for
its kind, with repeats of the signature piece and wall decorations), place by `author-pack/ROOMS.md`, and run
`studies/research-rooms/roomcheck.py PLACEMENTS.csv --kind KIND`; its checklist is in `ROOMS.md` and in
`studies/research-rooms/PRODUCER-room-section.md`.

## 6. Measuring the owner's words

The owner's rejections are the most valuable data the process gets. When the owner says what is wrong (that a piece
looks soft, or its features look added on, or a room looks empty), do not guess a fix. Turn the words into measurements against the owner's
own references: pre-register a small number of candidate measures (at most about 18, written before computing),
measure them on the rejected pieces, the accepted pieces and the real references, keep only those that separate
them under leave-one-out, and write the survivors as rules with bands and a checker. Faults that are seen but not
counted become review questions with pictures. This is how the fit of helms, the definition of sets, plates
before paint on bosses, density and grouping in rooms, softness of effects and cleanness of creatures became rules
(`studies/research-helm2`, `research-helm3`, `research-fit`, `research-rooms`, `research-clean`, `research-gap`,
`research-paint`). No model's eye predicts the owner's: the process works when judgement is replaced by
measurement plus sittings.

## 7. Each kind of job: prompt, brief, gate

Prompts are in `prompts/` as templates: `{ROOT}` is your working folder, `{BUILD}` the job's build folder
(`{ROOT}/builds/NAME`), `{PYTHON}` and `{BLENDER}` the executables. The `*-TEMPLATE.txt` files are the forms to
copy; the others are filled-in examples. Paths to earlier pieces of the original project (`{ROOT}/builds/...`) name
that project's own models, which are not shipped: substitute your own earlier pieces or drop the line.

| Job | Prompt template / example | Brief | Gate or checker |
|---|---|---|---|
| Furniture or scenery rebuild | `prompt-object-rebuild-TEMPLATE.txt` (animated objects), `authors/prompt-astra.txt` | `author-pack/BRIEF-furniture-rebuild.md`, `studies/research-effects2/BRIEF-animated-object-rebuild.md` | `tools/gate.py rebuild-object REAL_ID FILE --ambient A --contrast C` |
| Original furniture, set piece, wall or floor decoration | `authors/prompt-orig-astra.txt`, `prompt-producer-*.txt` | `author-pack/BRIEF-original-set-piece.md`, `WALL-DECOR.md`, `BRIEF-observatory-set.md` | `tools/gate.py original-object FILE --faces LO,HI --size ... [--shape 4/22]` |
| A room | `prompt-room-observatory.txt`, `prompt-producer-armoury*.txt` | `author-pack/BRIEF-first-room.md`, `ROOMS.md` | `studies/research-rooms/roomcheck.py` |
| Weapon rebuild / original | `authors/prompt-qual-fang-astra.txt`, `prompt-mace-a.txt` | `BRIEF-weapon-rebuild.md`, `BRIEF-original-weapon.md` | `tools/gate.py rebuild-weapon item:ID FILE`; `worn` |
| Armour rebuild | `prompt-qual-platebody.txt`, `prompt-qual-legs.txt`, `prompt-helm-rebuild-TEMPLATE.txt` | `BRIEF-armour-rebuild.md` | `tools/gate.py worn FILE-labelled.bin SLOTSPEC --real item:ID` |
| Player gear, original | `prompt-set-v5-TEMPLATE.txt`, `prompt-helm-v4-TEMPLATE.txt`, `prompt-player-PIECE.txt`, `prompt-cape.txt` | `BRIEF-original-armour.md`, `BRIEF-player-helm.md`, `PLAYER-GEAR.md`, `FASHIONSCAPE.md` | `gate.py worn`, `studies/research-fit/fit.py`, `definition.py`, `research-helm2/helmcheck.py`, `research-helm3/helm3check.py` |
| Creature rebuild | `prompt-creature-rebuild-TEMPLATE.txt` | `studies/research-creatures/BRIEF-creature-rebuild.md` | `research-creatures/gate_npc.py rebuild-npc`, `research-paint/detail/detailgate.py` |
| Creature, original | `prompt-creature-v2.txt`, `prompt-creature-watcher.txt` | `author-pack/BRIEF-original-creature.md` | `research-gap/gapcheck.py`, `research-clean/cleancheck.py`, `research-paint/stylecheck.py` |
| Boss rebuild / original | `prompt-boss-rebuild-tekton.txt`, `prompt-titan-rebuild-*.txt`, `prompt-boss-original.txt`, `prompt-boss-pitch-*.txt` | boss section of `BRIEF-creature-rebuild.md`, `BRIEF-new-boss-pitch.md`, `studies/research-bosses/BOSSES.md` | `research-bosses/gate_bone.py`, `tools-titan-sheets/teacher_gate.py`, `research-authorship/packagecheck.py` |
| Weighted (skeletal) creature | `prompt-research-animation-astra.txt` | `studies/research-rig2/BRIEF-weighted-creature.md` | `research-rig/rig_gate.py` |
| Animation | `prompt-anim-*.txt` | `author-pack/BRIEF-original-animation.md`, `studies/research-animation/REPORT.md` | sitting (motion line-up) |
| Effects rebuild / original | `prompt-effect-rebuild-TEMPLATE.txt`, `prompt-fx-warden-general.txt` | `studies/research-effects/BRIEF-effect-rebuild.md`, `research-effects2/BRIEF-original-effect.md` | the brief's checks (`research-effects/fx.py`) |
| Arena and fight | `prompt-producer-arena1.txt`, `prompt-tools-fight-in-arena.txt` | `studies/research-arenas/ARENAS.md`, `research-fights/FIGHTS.md` | `research-arenas/arenacheck.py`, `research-fights/fightcheck.py` |
| Interface rebuild / original | `prompt-iface-rebuild-*.txt`, `prompt-iface-original-o1.txt` | `studies/research-interfaces/BRIEF-interface-rebuild.md`, `BRIEF-original-interface.md` | `research-interfaces/ifgate.py` |
| HD-era or RS3 rebuild | `prompt-hd-rebuild-*.txt` | `studies/tools-hd-era/TECHNIQUES.md`, `studies/tools-rs3/TECHNIQUES.md` | `tools-hd-gate/hdgate.py`, `tools-rs3/rs3gate.py` (Teacher only) |
| A new tool | `prompt-tools-*.txt`, `authors/prompt-tools-astra.txt` | `teacher/playbooks/build-tool.md` | proven against a known answer before use |
| A research study | `prompt-research-*.txt` | `teacher/playbooks/extract-method.md` | pre-registered measures, leave-one-out |

To start an author: write the prompt to a file (first line names the brief, then the task line, then "Write your
report to {BUILD}/REPORT.md. Do not ask questions; decide and proceed."), start it with your agent runner, wait,
read the report, gate. A return is a new session.

## 8. Lessons that cost the most to learn

- Read the role and the rules before touching the machine; a check started before the reading is usually wrong.
- Do not ask the owner to approve what is the Teacher's to decide (the panel, the inputs, the tools).
- A sitting with no real game asset on the page cannot be judged for authenticity.
- Cache names are not categories: "Table" or "Barrel" includes set pieces that are nothing of the kind. Choose
  references by looking at their sheets.
- A face at ground height loses to the terrain: floor pieces sit at height 1 to 3, placed as ground decoration.
- Two gate-passing pieces can still read as the wrong thing; look before the owner does.
- When two candidates differ in how much decoration they carry, do not assume the richer one wins. Ask; do not steer.
- Good pieces do not make a good room: composition has its own rules.
- Write any file that contains backslashes with a file editor, not through a shell heredoc.
- Say what was run and what was not. A gate that errored is not a pass.
