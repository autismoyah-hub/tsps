# Runbook

How the Teacher's work is carried out on the asset machine, command by command. Started 2026-11-25 by the Teacher of the agent-systems project from the asset Teacher's record; the asset Teacher completes and owns it. The role is in [SKILL.md](../SKILL.md); this file is only the mechanics here. Update a procedure when it changes; do not add a second version beside it.

## 1. Starting a session

1. Work in the checkout of this repository on the asset machine. `git fetch origin` and `git merge --ff-only origin/main` before reading anything.
2. Read `teacher/SKILL.md`, the six `core/` files one at a time (a combined read was cut short by the terminal on the first day and the gap went unnoticed), then `teacher/project-docs/PROFILE.md`, `STATE.md`, `PLAN.md` and `DECISIONS.md`, then `docs/knowledge/OSRS-ASSETS-METHOD.md`.
3. Check the anchors in STATE.md against the machine.

## 2. Git

- Commit to `main` and push. `git push origin main` works with the saved credential.
- The platform's branch reservation and runner lookup returned HTTP 401 ([quoted words withheld]) in a session more than a day old. That error does not block git: a plain push worked. Do not wait on it and do not ask the owner about it; a fresh session gets a fresh token.
- The asset system is kept in this repository under `asset-system/` (A-024): run `sh tools/sync_asset_system.sh` before committing; it copies the tools, author pack, findings, boards, sitting specs and each build's scripts and final model from `{ROOT}/`, which stays the working copy. Renders, the game cache and raw dumps of real meshes never go into a commit. After committing, run `sh tools/verify_asset_system.sh`: this repository converts line endings in text files, and the first commit of the asset system had its images and Blender files altered until they were marked binary in `.gitattributes`; a new kind of binary file must be added there first.

## 3. Running a blind pair

For the five-object run, make one detached git worktree per author. Verify every reference and design-intent file against {ADAPTER_DIR}/panel-inputs.csv before copying it to trial/<object>/input. Start each author with cuttle sessions spawn --runner <subscription Codex provider from --list-runners> --model gpt-6.2-<astra or sol> --effort medium --prompt <matched brief>. Use the bench brief at {ADAPTER_DIR}/timber-bench-trial/entry-1/brief.txt, substituting the five object names for the bench. Request trial/<object>/final.blend. Allow one revision and 60 active minutes per object. Actual five-object durations await delivery.

For each frozen scene, run Blender in background with the scene, then -P {ADAPTER_DIR}/timber-bench-trial/presentation.py -- <absolute PNG output>. The script computes bounds in camera space and fixes orthographic framing, material colours, studio light and 800-square output. The original private bench assignment is in teacher-order.json beside this script. Preserve that assignment when comparing bench verdicts. Generate independent private assignments for new pairs.

## 4. Collecting a verdict

The owner clicks [quoted words withheld] on the review page. `python tools/collect_panel_verdict.py` imports it from Downloads into `docs/verdicts/`; identical exports are safe to collect twice, conflicting ones need reconciling. The owner's words are kept exactly.

## 5. The review page

Every sitting shows, for each item judged, the real game asset or assets it should stand beside, at the same size (PROFILE.md, [quoted words withheld]). One sitting is about ten minutes. Pages are local files under `{WORK_DIR}/`; open them in Edge on the desktop.

**Sittings since 2026-11-25** (the owner asked for a page to answer on and to tell the Teacher things). Write a spec (`{ROOT}/sittings/ID.json`; the format is in the docstring of `tools/make_sitting.py`), then `python tools/make_sitting.py SPEC`. It copies the pictures to `{REVIEW_DIR}/sittings/ID/`, builds the page and points `{REVIEW_DIR}/index.html`, which the [quoted words withheld] desktop shortcut opens, at it. A line-up asks which is ours and how sure, hides the key until the pick is locked, then asks for a grade; a pair asks which side is closer to the game; a last box takes anything else. The owner clicks [quoted words withheld]; `python tools/collect_sitting.py ID` files the download in `docs/verdicts/` unchanged. Check a new page by screenshot before announcing it: `msedge --headless=new --window-size=1500,2600 --screenshot=OUT.png file:///{REVIEW_DIR}/sittings/ID/index.html` (Edge is under `{EDGE_DIR}/`).

## 6. What went wrong, so it is not repeated

- Asking the owner to approve the panel and the agents' inputs. Both were the Teacher's to decide.
- Asking the owner to re-authenticate the platform. The owner did not know what was meant, and it was not needed.
- A bench sitting with no real game asset on the page; the owner could not judge authenticity.
- Starting machine checks before the role's files had been read.


For the five-pair sitting use {WORK_DIR}/five-pair-sitting/presentation.py with the frozen scene and arguments -- TEXTURE or VERTEX followed by the output PNG. Each mode reads that scene's finished surface pass. five-pair-order.json stays private under {WORK_DIR}/. Save bench-anchored-verdict.json and five-pair-verdict.json verbatim from Downloads after inspecting their fields.

Publish every current sitting at {REVIEW_DIR}/index.html. The OSRS Asset Review desktop shortcut opens that fixed file. Keep sitting-specific images and verdict export names distinct. Verify every local image link before announcing the sitting.

## Autonomous catalog

The fixed review page at {REVIEW_DIR}/index.html opens catalog/index.html. Each detail page identifies the scene, hero render, supplied alternate views and references. All art stays private on this machine.

From {REVIEW_DIR}/catalog use Python with Pillow to run build_gallery.py followed by validate.py. The validator checks the generated links and opens every distinct scene in Blender. scene-validation.log contains the Blender readback. Rebuild renders with the author scripts kept in each family directory. The redesigned tower starts from the retained north-wall scene. The Furnace Ray and Cinder Sluice share a scene. Keep owner verdicts in docs/verdicts and Teacher decisions in the project register.

## The game's renderer and cache

Used and checked 2026-11-25 by the asset Teacher. Everything it produces is private; nothing below is committed except this description.

- **Python:** `{PYTHON}` (3.13, with Pillow and NumPy). There is no `python` on the path, and Blender's own Python does not start from this shell. Run scripts with `-I`.
- **Renderer:** `{RENDERER_BIN}`. Arguments: `CONTENT CACHE X Z YAW DISTANCE ELEVATION TARGET_UP classic auto NEW_OUTPUT_DIRECTORY`. X and Z are tiles; yaw is in 16384ths of a turn; the output directory must not exist. It writes `view.bmp`. A room takes under a second.
- **Cache:** `{OSRS_CACHE_239}` (revision 239). The revision-241 cache under `{EXTERNAL_DIR}/` is refused: [quoted words withheld].
- **Two looks.** `authored_scene_snapshot_gpu.exe` with `gpu auto` is an enhanced mode with cast shadows; E114's recorded captures used it and it needs a GPU this machine lacks. `classic` is the game's own look and is the one to judge in.
- **A scene** is a folder: `content.txt` (paths and SHA-256 pins), `package/` (`registry.txt` listing models and object definitions with CRC-32, and their files), `floor.bwr` (text: one `V x z height underlay walkable` row per vertex of a 128 by 128 grid, then `O object x z shape rotation` placements), `bindings.txt`. E114's is at `{E114_SCENE}`; its room is centred near tile 53.5, 46.5 and its three camera poses are in `capture-plan.json` there.
- **The Teacher's tools,** in `{ROOT}/tools/`:
  - `scene.py OUT BASE "id:x:z:rot,..." "X Z YAW DIST ELEV UP" ...` builds a scene from a base scene folder (or `-` for none) plus real objects by id, mounted with their own definitions, and captures each camera to a PNG. A negative id is a real NPC placed as a static figure, for scale; `-16298` is the Banker.
  - `osrscache.py` reads cache groups and decodes real models (vertices, faces, face colours, render types). `measure.py` and `bands.py` compute the style measures; `roomprobe.py` gives the size of whatever stands at a tile of the E114 room; `contact.py` makes a contact sheet of archive thumbnails; `board.py` makes a labelled side-by-side; `smooth.py` re-renders a scene with every face smooth-shaded.
  - `lineup.py OUT "entry,entry,..." [--shuffle SEED] [--views N] [--dist D] [--elev E] [--yardstick NPC]` is the way to compare: each entry (a real object id, or the path of one of our `.bin` models) is captured alone on terrain from the same camera and the crops are tiled into `lineup.png`, numbered; `key.txt` says which is ours and stays away from whoever judges.
  - `author.py` writes a mesh to the game's model format: one vertex per position, smooth shading, faces turned outward, 16-bit HSL face colours. `examples/barrel/build.py` is the worked example. `anatomy.py MODEL_ID` shows how a real model is built (rings, seams, colours and their share); `pick.py REGEX N OUT` tiles the three-quarter views of real objects of a name across all periods; `seams.py` measures welding and tone steps.
  - **The Blender route** (proved 2026-11-25: the barrel exported through Blender is identical, face for face, to the direct build): a build script uses `blender_kit.py` (game units, y up, materials named `hsl_H_S_L`), then `blender -b FILE.blend -P {ROOT}/tools/blender_export.py -- OUT.bin` welds, smooths and prints size and colour shares. Blender is `{BLENDER}`. The older `convert_glb.py` route exports flat and converts colour from RGB; do not use it.
  - `compare.py REAL_OBJECT_ID OURS.bin` scores a rebuild against the real mesh (size, counts, welding, colour shares and distance, outline overlap), applying the real definition's recolours and scale. A real model scored against a rounded copy of itself gives overlap 0.99 and colour distance 0.03; that is the ceiling. Authors of a held-out rebuild do not run it.
  - **Briefing an author:** give it `{ROOT}/author-pack/RULES.md`, one build folder under `builds/`, the reference sheet's path, and the rule that it may draw the real object in a line-up but may not read its mesh. The first brief is in the Teacher session of 2026-11-25; turn it into a playbook once it has produced an accepted result.
  - **A real asset in Blender:** `python tools/real_dump.py KIND ID OUT.json` (KIND is object, npc, item or model), then `blender -b -P {ROOT}/tools/real_to_blender.py -- OUT.json OUT.blend`. The mesh keeps real colours, flat faces and priorities as `hsl_` materials and exports back through `blender_export.py` unchanged apart from welding. `wire.py OBJECT_ID OUT.png` draws a wireframe study sheet (every edge, painted tones, flat faces, priority bands); the author pack holds these for real neighbours, never for a held-out piece. Files go in `{ROOT}/real-in-blender/`.
  - **The dedicated client build (A-021), built 2026-11-26.** Rust is installed for this user (`{CARGO_BIN}`, toolchain `stable-x86_64-pc-windows-gnu`; add that folder to PATH in the shell). A copy of the client's source is at `{CLIENT_SRC}`; build an example with `cargo build --release --example NAME` (not `--offline`: one optional crate is not vendored and is fetched once). A full build takes about a minute. The originals under `{ASSET_ADAPTER_DIR}/` are not touched.
  - **Weapons in a figure's hand:** `python tools/figure.py OUT.png "item:ID,model:PATH,none"` runs the Teacher's example `examples/asset_figure.rs` in that build: a standard figure per entry holding a real item or one of our models (authored in worn space), three angles, lit as the game lights worn gear. Checked: a re-encoded copy of a real halberd draws identically to the real item. `compare.py item:ID OURS.bin` scores a weapon rebuild against the real worn mesh in place. `weapon_card.py` is the weapon style card. The briefs for authors are `author-pack/BRIEF-weapon-rebuild.md` and `BRIEF-original-set-piece.md`.
  - `vignette.py OUT "WHAT:x:z:rot[:sx:sz];..." "X Z YAW DIST ELEV UP"` stands several pieces, real and ours, together on terrain with rotations: the test of a set. `family.py OBJECT_ID` lists real objects by closeness of palette, for choosing neighbours.
  - Findings and figures: `{ROOT}/FINDINGS.md`.
  - Write any file that contains a backslash with an editor, not through a Python string in a shell heredoc: that route wrote control characters into this runbook once.
- **Known limits.** Between objects the renderer has a depth buffer (an earlier line here said it had none; that was a wrong diagnosis). Inside one model faces are painted by priority and depth sort. A face at ground height loses to the terrain: floors sit at height 1 to 3, placed as ground decoration (shape 22). Choose real references by looking at their sheets; cache names such as "Table" or "Barrel" include set pieces that are nothing of the kind. The newest object ids are one dark region.
- The reference archive (`{REFERENCE_ARCHIVE}/`): for every real object, NPC, item and body kit, a six-view sheet and a JSON file with model ids, counts and bounds. The program that drew it is `{EXTERNAL_DIR}/osrs-cold/osrs-share/machine-b/jagex-reference-archive/receipts/build-003/reference_sheet.exe` (`CACHE OUTPUT TYPE START_ID END_ID`); not yet run here.

## Authors on less expensive models, and the gate (A-026)

- Start an author: write the prompt to a file (first line names the brief in `{ROOT}/author-pack/`, then the task line, then "Write your report to .../REPORT.md. Do not ask questions; decide and proceed."; examples in `{ROOT}/compare-authors/`), then `cuttle sessions spawn --runner {RUNNER} --model gpt-6.2-astra --effort medium --prompt-file FILE`. Wait: `cuttle sessions wait ID...`. Read the end: `cuttle sessions read ID`.
- A started session does not take a changed assignment from you. To return a piece, start a NEW session whose prompt names the build folder, pastes the gate's lines and says what to change.
- Gate (run from `{ROOT}`, Python as above; last line `VERDICT PASS|BORDERLINE|FAIL`, exit 0/1/2; INFO lines never decide):
  `tools/gate.py rebuild-object REAL_ID FILE --ambient A --contrast C` | `original-object FILE --faces LO,HI --size XLO,XHI,YLO,YHI,ZLO,ZHI` | `rebuild-weapon item:ID FILE` | `worn FILE-labelled.bin SLOTSPEC [--real item:ID] [--reference item:ID] [--steel 42,0] [--non-steel H,S,L]`.
  It does not judge design, or motion faults on original worn pieces (that ratio is INFO until calibrated): look at those yourself and leave them to the sitting.
- `tools/assemble.py OUT.png SET.json` draws a whole outfit; `tools/label_overrides.py` labels armour with per-part corrections; `tools/dump_worn_labels.py item:ID OUT.json` reads a real worn item with its labels.
- Close sessions when their work is read: a finished Astra session stays in memory as `waiting_for_input` until closed. `cuttle sessions kill ID` (its files stay on disk). After a batch, close every session you started; never close one you did not start.
