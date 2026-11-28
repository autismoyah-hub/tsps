# Producer tools handoff, 26 November 2026

## Using this delivery

Run the commands below from `{ROOT}/studies/tools-handoff2/`. The command blocks use PowerShell. The shell wrapper itself takes Git Bash arguments. All six tools and the test pack are local copies. The installed gate still imports the installed compare module. A gate score produced there can use the wrong colours for an affected object.

### Object lighting and comparison

`objlight.py` reads the real cache definition and prints the **signed, unmultiplied** ambient and contrast values to pass to lineup and gate. Contrast 15 goes on the command line as 15. The client multiplies it by five internally.

`compare.py` keeps the existing object and worn-item CLI and the dictionary returned by `recolours`, including the three scale keys. It shares the new lighting tool's definition walk. The authority was `client/rust-client/src/object.rs`, including action strings, conditional actions, 32-bit model lists, transforms, sound lists, typed parameters, local-only fields and the default no-op case. The old walk returned an apparently valid partial dictionary on its first unrecognised opcode. Drawers 39924 lost all three mappings after its Remove action. Drawers 25702 also lost both mappings. An early regression assertion that treated 25702 as an unchanged control failed and exposed that second case. The successful regression below checks five other objects with nonempty mappings against the old parser and independent archive JSON. The byte-stream test exercises lengths that those six examples do not cover.

### Held-out register

Run `held_out.py` to list definition type, ID, name, model IDs and all recorded builds. `--check ID` checks object, item and model namespaces together. For a specific namespace use `--check object:ID`, `item:ID` or `model:ID`. A match shows the assignment that reserved the model. “Free” means absent from the recorded rebuild/calibration assignments. For example, 2333 has a teaching wireframe and can be unsuitable for a blind brief even though it is free in this register.

I searched build reports and notes, comparison prompts, trial reports, states and prompts, and the night task lines and session records. The historical assignments are curated in the script. It also reads structured single-target rebuild prompts at runtime, including briefs with a named build and a target sheet before a binary exists. Keep the historical list current for briefs written in other formats. The Sol drawers brief counts as an assignment. The barrel script describes an original design. Style neighbours, camera controls and real meshes re-encoded by a Producer for a ceiling check do not establish an author rebuild. Item rows include inventory and alternate worn model IDs from the archive. Object lookups use the cache model lists, so aliases do not require a separate list.

### Blender build and export

From Git Bash run `bash blender_run.sh BUILD_SCRIPT.py OUT.blend OUT.bin [exporter options]`. The wrapper removes the previous binary, runs both Blender invocations with `--python-exit-code 1`, saves the built scene to the requested blend path and checks for a nonempty export. The failure cleanup also covers a partially written export. The test used the installed Blender executable and exporter. It built a coloured cube with `--weapon`, ran an intentionally raising build against that same destination, and attempted an export with a missing collection.

### Fable review pack

Open [the review index](review-pack-test/INDEX.md) for per-build gate measurements, author assessments, pictures and matches to the owner's answers. [Merged proposals](review-pack-test/PROPOSALS-ALL.md) groups reported issues while preserving their source wording. The Producer records and owner files are also in the pack. The parked list records archives. Astra's archived table remained eligible for the sitting at BORDERLINE under the revised procedure.

Run `review_pack.py YYYY-MM-DD OUT_DIR` with a dedicated output directory. The inclusive cutoff is local midnight, using filesystem modification times, including changed descendants of a build. Producer records are collected regardless of age. Known assignments and sitting keys disambiguate generic labels such as “table.” Unknown sessions and unmatched answers are marked in the index. A saved verdict is evidence of the indicated submission. Fresh checks use the installed gate with its temporary render work redirected inside OUT_DIR. The three v3 helm checks use the slot and steel declared in design.md, with the two declared moonstone tones exempted for B and C. Their motion lines are informational without a real comparison item. Reports with no identifiable assessment heading are reproduced in full. Time or token evidence is quoted where found. Missing accounting data cannot give a per-accepted-piece cost.

### Sitting runbook text

For a trial or concurrent Producer, pass `--root` on every build. For example, from the handoff directory run:

```powershell
& {PYTHON} -I make_sitting.py {ROOT}/trial/astra/sitting.json --root {ROOT}/trial/astra/review
```

Open `ROOT/sittings/SPEC-ID/index.html` for the individual page. Both its copied pictures and `ROOT/index.html` belong to that root. The corrected builder makes the root index a dated directory of all sitting pages, with the newest first. Historical production sitting IDs contain the date and a letter for the order within a day. For an undated trial ID, the displayed date comes from its page modification time.

If a trial has already repointed the shared shortcut, first identify the newest genuine sitting from the dated specs in `{ROOT}/sittings/`. At inspection it was `2026-11-26-g.json`. To restore the **original automatic redirect**, run the original builder on the genuine spec with the shared root:

```powershell
& {PYTHON} -I {PROJECT_REPO}/tools/make_sitting.py {ROOT}/sittings/2026-11-26-g.json --root {REVIEW_DIR}
```

To use the **all-sittings index** on the shared shortcut instead, run this from the handoff directory:

```powershell
& {PYTHON} -I make_sitting.py --index-only --root {REVIEW_DIR}
```

Check the first entry's date and title and follow its link. Index-only scans existing pages and keeps their original contents. If a trial was accidentally created under the shared root, it will also appear in this list. Use the genuine spec with the original builder when the shortcut must open a specific sitting directly. The shared index already redirected to sitting g on inspection. Neither shared-root command above was run.

## Test commands and observed output

### Lighting

```powershell
& {PYTHON} -I objlight.py 25702 22472 55076 2677 42083
```

Exit 0. Output:

```text
25702 'Drawers' ambient 20 contrast 15 models [25071] recolours 2
22472 'Lectern' ambient 30 contrast 0 models [22941] recolours 0
55076 'Bench' ambient 10 contrast 30 models [52304] recolours 7
2677 'Chest' ambient 15 contrast 0 models [12146] recolours 5
42083 'Table' ambient 10 contrast 10 models [42363] recolours 0
```

### Comparison regression, legacy parser against archive and corrected parser

```powershell
& {PYTHON} -I tests/definitions.py
```

Exit 0. Output:

```text
39924 {} -> {6443: 7830, 6682: 7954, 5908: 7830}
25702 {} -> {6443: 5532, 6682: 5421}
25 {41905: 960} -> {41905: 960}
26 {41905: 0} -> {41905: 0}
27 {41905: 127} -> {41905: 127}
29 {41905: 960} -> {41905: 960}
30 {41905: 0} -> {41905: 0}
```

### Cache and local opcode layouts

```powershell
& {PYTHON} -I tests/opcodes.py
```

Exit 0. Output:

```text
cache walk reached final recolour, scales and signed lighting
local walk reached final recolour, scales and signed lighting
```

### Normal comparison CLI

```powershell
& {PYTHON} -I compare.py 25702 ../cmp-drawers-astra/cmp-drawers-astra.bin
```

Exit 0. Output:

```text
size x, height, z                real [96, 141, 116]   ours [96, 141, 116]
vertices                         real 236   ours 242
faces                            real 356   ours 358
vertices on a shared position    real 0   ours 0
flat-shaded faces                real 28   ours 16
colours                          real 2   ours 2
colour shares (h,s,l: % of surface)
  real: (5, 2, 45):54, (5, 3, 28):46
  ours: (5, 2, 45):57, (5, 3, 28):43
colour distance (0 = identical; about 1 = one lightness step of 4 over the whole surface): 0.16
outline overlap (1 = identical): front 0.986, side 0.990, top 0.993
```

### Register inventory

```powershell
& {PYTHON} -I held_out.py
```

Exit 0. Output:

```text
definition | name | model IDs | builds
object:2677 | Chest | 12146 | trial-opus-chest-2677
object:7195 | Drawers | 25881 | obs2-cal-drawers-7195
object:13659 | Crystal ball | 12952 | obs1-cal-crystal-13659
object:15704 | Crate | 15510, 15516 | crate-15704
object:22472 | Lectern | 22941 | lectern-22472
object:25702 | Drawers | 25071 | cmp-drawers-astra cmp-drawers-claude cmp-drawers-opus cmp-drawers-sol
object:42083 | Table | 42363 | trial-astra-table
object:55076 | Bench | 52304 | bench-55076
item:1079 | Rune platelegs | 268, 432, 2582 | qual-legs-astra
item:1101 | Iron chainbody | 294, 458, 2439 | armour-chainbody-1101 qual-chainbody-astra
item:1127 | Rune platebody | 164, 306, 344, 468, 2605 | qual-platebody-astra
item:10828 | Helm of neitiznot | 21873, 21906, 21938, 22153, 22777 | armour-helm-10828
item:33174 | Osmumten's fang | 45278, 45285, 46468 | weapon-fang-33174 qual-fang-astra
item:33460 | Dragon crossbow | 34655, 34662, 42612 | weapon-crossbow-33460
```

### Direct assignment

```powershell
& {PYTHON} -I held_out.py --check 25702
```

Exit 0. Output:

```text
25702: used
definition | name | model IDs | builds
object:25702 | Drawers | 25071 | cmp-drawers-astra cmp-drawers-claude cmp-drawers-opus cmp-drawers-sol
```

### Shared-model alias

```powershell
& {PYTHON} -I held_out.py --check 23967
```

Exit 0. Output:

```text
23967: used
definition | name | model IDs | builds
object:7195 | Drawers | 25881 | obs2-cal-drawers-7195
```

### Direct model number

```powershell
& {PYTHON} -I held_out.py --check 25881
```

Exit 0. Output:

```text
25881: used
definition | name | model IDs | builds
object:7195 | Drawers | 25881 | obs2-cal-drawers-7195
```

### Free object control

```powershell
& {PYTHON} -I held_out.py --check object:2333
```

Exit 0. Output:

```text
object:2333: free
```

### Item identity

```powershell
& {PYTHON} -I held_out.py --check item:33174
```

Exit 0. Output:

```text
item:33174: used
definition | name | model IDs | builds
item:33174 | Osmumten's fang | 45278, 45285, 46468 | weapon-fang-33174 qual-fang-astra
```

### Item model

```powershell
& {PYTHON} -I held_out.py --check model:45278
```

Exit 0. Output:

```text
model:45278: used
definition | name | model IDs | builds
item:33174 | Osmumten's fang | 45278, 45285, 46468 | weapon-fang-33174 qual-fang-astra
```

### Blender success, build exception and export exception

```powershell
& {PYTHON} -I tests/blender.py
```

Exit 0. Output:

```text
good ['--weapon'] exit 0 bin bytes 158
bad [] exit 1 bin bytes absent
good ['--collection', 'missing_collection'] exit 1 bin bytes absent
```

### Review pack generation

```powershell
& {PYTHON} -I review_pack.py 2026-11-26 review-pack-test
```

Exit 0. Output:

```text
28 builds, 4 Producers, 2 owner files, 1 parked archives, 42 proposals
night/obs2: COUNT.md missing
meridian-helm-v3: REPORT.md missing.
qual-legs-astra: REPORT.md missing.
research-animation: No applicable piece gate.
research-animation: REPORT.md missing.
research-items: No applicable piece gate.
research-items: REPORT.md missing.
research-npc: No applicable piece gate.
research-npc: REPORT.md missing.
room-observatory: No applicable piece gate.
tools-handoff: no line-up or figure picture found
tools-handoff: No applicable piece gate.
tools-handoff2: no line-up or figure picture found
tools-handoff2: No applicable piece gate.
```

### Review pack readback

```powershell
& {PYTHON} -I tests/pack_check.py
```

Exit 0. Output:

```text
Verified 74 index links, 28 piece records, 26 byte-identical pictures, owner file bytes and all copied proposal entries.
```

### Sitting roots and archive index

```powershell
& {PYTHON} -I tests/sittings.py
```

Exit 0. Output:

```text
Two isolated sittings, both dated titles and targets verified, newest first after rebuilding older sitting and index.
9 existing sitting pages indexed on private copies; first sittings/2026-11-26-g/index.html; shared index byte-identical.
```

## Findings and verification limits

- The review snapshot contains 28 changed build folders, four Producers, 26 pictures, two owner-side files and one parked archive. The two owner files were the sitting-g answer JSON and the shared COUNT ledger. The trial-local states can still describe the owner as pending when the shared ledger already contains the answer. The pack retains both records. It found 42 source entries in the proposal files, including contextual Producer remarks. Several night records changed during development. The generation output above describes the delivered snapshot.
- The pack found no night/obs2 COUNT at collection time. Its output above enumerates every missing build report, picture and applicable gate. Research, room and tools folders are still indexed. Some historical reports have no author session ID or token accounting. No new owner answers, predictions or acceptance decisions were inferred.
- The original sitting builder overwrote the root index with a redirect to the last spec it processed, even if that spec was older. The original compare parser returned partial results without warning. Blender scripts can fail while a previous export remains usable by the next command. Each case has a local reproduction or readback above.
- The Producer proposal about using the [quoted words withheld] predates the current wording of PRODUCER step 5, which calls for a new session. The previous three-drawers gate measurements are historical. I tested the corrected comparison directly and did not recompute the shared scoring history, change thresholds, install the tools or repoint the shared index.
- All six requested deliverables were exercised. The tests cover the observed records, current cache format and the named failure cases. No visual quality grade, live shared-root change, exhaustive future prompt recognition or timing claim follows from these tests. No commit, push or external transfer was performed.
