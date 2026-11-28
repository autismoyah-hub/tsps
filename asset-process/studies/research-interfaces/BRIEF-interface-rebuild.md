# Brief: rebuild a real INTERFACE panel from its sheet (v2)

Status: second use. The first three blind rules runs (648, 383, 303) failed where INTERFACES.md v1 was silent or
wrong; their reports were turned into v2 (`CHANGES.md`): every construction is now a RECIPE with numbers, verified
against all real instances, the sheets say what no rule can (colours by palette name, wells' transparency and
mirroring, the header band's line, line pitch, draw order), and the gate prints a per-region verdict. A script that
uses only INTERFACES.md v2 (`ifkit.py`) and the v2 rules sheets rebuilds 648, 383, 303, 210, 68, 148 and 912 at 0
differing pixels (476 differs only in its icons). The gate is proven on controls: the real definitions of 14 panels
PASS at 0 pixels; of 139 damaged copies 122 FAIL and 9 are BORDERLINE (a 1px text shift, a 1px backing phase,
swapped icons), and the 8 that PASS are all the 1px text shift inside the 2px tolerance, as designed
(`gate-proof/summary-v2.txt`). Nothing here has been
judged by the owner.

You are an authoring agent in a project that makes original assets for an Old School RuneScape private server whose
game runs on the TSPS browser client (cache revision 242). The owner's standard: an asset must read at once as the
game's own, and within that be as well made as possible. Our method: measure real assets, REBUILD real ones held
out, prove the rebuild cannot be told from the real one, only then make originals. You are rebuilding an INTERFACE:
a panel (a window in the 512x334 main modal) as a list of the client's own components (layers, rectangles, texts,
sprites, lines). Your target group G and build name NAME are in the task line that came with this brief; the sheet
KIND is `rules`.

**What this test is for.** Is `INTERFACES.md` v2 enough to rebuild a real panel nobody has rebuilt, from a sheet that
gives sizes, words and named colours but no picture? Your report of where v2 was silent, ambiguous or wrong is the
most valuable thing you will deliver, more valuable than a PASS.

## Read first, completely

1. `{ROOT}/studies/research-interfaces/INTERFACES.md` (v2), all of it. Sections 2 (frame recipes), 3
   (wells), 5 (text), 6 (buttons), 13 (decision rules and what is undecidable) set almost every choice.
2. Your sheet: `{ROOT}/studies/research-interfaces/sheets/G-rules/`: `sheet.md`, `sheet.json` (the same),
   `wire.png` (drawn at 2x). New in v2: each region's `drawn` rank (draw order; later is on top), colours by their
   palette name (INTERFACES.md 12), wells' and backings' transparency and mirroring in words, the header band's line
   rectangle, each string's colour, line pitch, line alignment and the palette name of each emphasised part.
3. `ifkit.py` (the recipes as code: `steel_panel`, `steel_box`, `stone_panel`, `thin_box`, `bevel_box`, `tile_box`,
   `well`, `two_tone`, `hpbar_hud`) and `proof/build_rebuild.py` (a worked example; PET_INSURANCE, used up).
4. The docstrings of `iface.py` and `ifgate.py`.

## The targets (second test; none has been rebuilt or used to tune a recipe)

Chosen by a seeded random draw (seed 20261127, `recipes/heldout.json`) from the modal panels whose iface.py drawing
equals the TSPS capture, excluding every panel seen while writing v2. Return the group as 30910 + n.

| n | G | name | sheet | drawn for |
|---|---|---|---|---|
| 1 | 4 | DEATHKEEP | 500x322, 2 regions, 2 strings | titled panel with an inner box |
| 2 | 302 | HORROR_PRAYERBOOKS | 330x265, 13 regions, 13 strings | table or list |
| 3 | 422 | POH_VIEWER | 498x304, 15 regions, 8 strings | tabs or buttons |
| 4 | 868 | PMOON_REWARD | 188x190, 7 regions, 1 string | HUD or overlay (no unseen HUD is TSPS-exact: drawn from the rest) |
| 5 | 206 | NZONE_REWARDS | 488x300, 5 regions, 18 strings | item grid |
| 6 | 156 | BUGREPORT | 350x320, 5 regions, 4 strings | dialogue-like (no unseen dialogue is TSPS-exact: body text) |

## Not allowed (any of these voids the run)

- Reading, decoding or DRAWING your target group from the cache or from any file made from it: no `iface.py G ...`,
  no `ifcache.load_interface(cache, G)`, no `real/*`, `state/`, `state_modal/`, `state242/`, `tsps/`, `harness/`,
  `ref/`, `board-*.png`, `gate-proof/`, `recipes/` (the Teacher's census and tools) and above all
  `sheets-teacher-only/`.
- Opening any `sheets/*-picture/` folder; decoding or drawing ANY real interface group; any tool that matches cache
  sprites or fonts against a picture (tracing).
- Reading the gate's inputs, changing any file outside your build folder, running `ifsheet.py` or `ifgate_proof.py`.
- Sending anything to an outside service or repository.

You MAY: read INTERFACES.md (v2 and v1), DELIVERY.md, NEXT.md, CHANGES.md; use `ifkit.py` and `ifpalette.py`; draw
any cache SPRITE or FONT by id to look at it; draw any text you write in any font to measure its letters (that is how
5.1 finds the font); draw and gate your own JSON as often as you like within the rounds.

## Tools (Git Bash; `P` = `{PYTHON} -I`; `I` = `{ROOT}/studies/research-interfaces`)

- **Build** by a script (`ifkit.Kit`, or the `add()` helper of `proof/build_rebuild.py`). Every component needs
  `file`, `kind`, `parent` (a file number, or null for the one root), `raw_position`, `raw_size`; then by kind as in
  INTERFACES.md 1. Remember: draw order is the TREE (INTERFACES.md 1), so create each box's layer where it must be
  drawn among its siblings, and put a box's contents inside its layer.
- **Draw** exactly as the gate does: `P I/ifgate.py draw NAME.json OUT.png` (the pane and `OUT-crop3x.png`).
- **Gate**: `P I/ifgate.py rebuild G NAME.json` (counts only; the board goes to the Teacher). Its parts:
  - pixels: share of the panel's pixels that differ (models and item icons left out): PASS <= 0.5% (or 100 px),
    BORDERLINE <= 2% (or 300 px);
  - structure: texts (words, font, colour glyph by glyph, shadow, letters within 2px); border, divider, close and
    box-piece sprites within 2px and none extra; rectangles and lines (kind, colour, fill, transparency, within
    2px); backings and wells (sprite, flips, transparency, within 2px); icons BORDERLINE at worst (a rules sheet
    cannot name them); components that change no pixel are not scored;
  - **per-region verdict (new)**: one line per sheet region and string, `region R3 | matched` or `not matched`
    (every real component in it matched, and none of the pixels it shows differs). No further detail: use it to know
    WHERE to look, then find the rule that decides it; do not search values against it;
  - construction: one root, kinds 0/3/4/5/6/9, cache sprites and fonts only, fits the 512x334 modal, nothing outside
    the root, converts with `to_tsps.py`.
  - `VERDICT` is the worst scored line.

## How to work

1. Work only in `{ROOT}/builds/NAME/`. Scripted builds only; write files with your file-writing tool,
   never shell heredocs.
2. Order: (a) read the sheet; in `plan.md` write for each region the INTERFACES.md v2 recipe (section and pieces) and
   for each string the font (by 5.1), colour, box and alignment and the rule that says so; where no rule decides
   write "SILENT" and your choice. (b) Build in the sheet's `drawn` order: frame, inner boxes with their contents
   before their frame pieces (2.2, 2.5), buttons with their labels after their box, texts. (c) Draw, measure your
   letters against the sheet, gate. One round = one change set, one gate run.
3. Keep `notes.md`: per round all the gate's lines and what you changed and why.
4. **At most ten rounds.** Stop at PASS when you can name no difference you know how to remove, or at round ten. Do
   not tune to the gate: a change you cannot justify from the sheet or a rule is not allowed; a per-region verdict
   tells you where, never what.

## Report (as your final message; it is all the Teacher will see)

- Paths: the final JSON, build script, `plan.md`, `notes.md`, final drawing and 3x crop.
- The gate's lines, every line, for EVERY round, and the final VERDICT.
- Part by part (each region and string): which v2 recipe set it (section and words), or "SILENT: I chose X because Y".
- **Where INTERFACES.md v2 was silent, ambiguous or wrong** (the most valuable part), with the region or string, what
  the rule said, what you chose, and the per-region verdict after; and what the sheet did not tell you that you needed,
  or told you that made it too easy.
- Icons: what you chose for each letter and why (expect the icon line to be BORDERLINE).
- Which tools were wrong, useless or missing; how many cycles you ran.
