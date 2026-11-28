# Brief: an ORIGINAL interface (boss HUD or reward panel) in the game's own style

Status 2026-11-30 (interim Teacher): **NOT A PROVEN PROCESS.** The second blind run from rules sheets (INTERFACES.md
v2) rebuilt all six held-out panels (4, 302, 422, 868, 206, 156) with 0 differing pixels outside icon boxes; the
gate's overall verdict was PASS for 3, BORDERLINE 1, FAIL 2, every difference inside anonymous icon art a rules sheet
cannot name. So the rules reproduce real panels; whether they make NEW panels that read as the game's own is what this
brief tests. O1 below is now for the Warden-General's encounter (the boss of the first whole encounter), not the
Armillary sentinel's.
Earlier status: no interface rebuild had yet passed blind (see `BRIEF-interface-rebuild.md`), so
nobody knew whether INTERFACES.md is enough to make new panels that read as the game's own. The checks of
`ifgate.py original` are measured conventions, not a proof: the real panels 148 and 476 pass them, the real HUD
303 passes all but the [quoted words withheld] line (its live state was captured without its pane). The owner judges the
result beside real interfaces; that judgement is the test.

You are an authoring agent in a project that makes original assets for an Old School RuneScape private server whose
game runs on the TSPS browser client (cache revision 242). The owner's standard: an asset must read at once as the
game's own, and within that be as well made as possible. Your order (O1 or O2) and your build name NAME are in the
task line that came with this brief.

## The orders (from NEXT.md section 2)

**O1. Boss HUD for the Warden-General's encounter** (fight data: `{ROOT}/builds/fx-warden-general/fight.json`;
the boss name drawn is "Warden-General"; the countdown is the Meridian Line's three warning ticks, one pip per tick). Group 30910, `"overlay": true`.
- The HPBAR_HUD plan (INTERFACES.md 7): a dark box (TRADEBACKING_DARK 897 inset 2, a 1px 191511 frame), the boss
  name in p11_full ff981f with shadow centred in the top 19px, a red-under-green bar (cc0000 under 00c800; the
  green's width is current/maximum), its value in p11_full eaeaea centred over the bar; plus one mechanic: an
  "alignment" countdown (a p11 or p12 ffffff number) and a row of three pips made as 9px-bevel boxes
  (INTERFACES.md 2.4), lit ff981f, under the bar.
- 150 to 180 wide, at most 70 tall, top-left in the HUD pane as 303 is in 242 (its box at (2,25)). The root fills the
  pane (size mode [1, 1]); the box is anchored by position modes, never absolute in a frame.
- Deliver: the component JSON; drawings of five states (full; half; 10%; timer 3; timer 0 with the pips lit), each
  its own JSON or produced by one script with a state argument; a line-up picture beside the real HUDs drawn by
  iface.py (303 from `sheets/303-picture/panel-1x.png`, and `iface.py 596`, `iface.py 413` with `--cache 242`);
  the plugin calls that would drive it each tick (`sendString`, size, colour, and which component each changes;
  DELIVERY.md section 4.2).

**O2. Reward chest panel for the same encounter.** Group 30911.
- A titled steel-border panel (INTERFACES.md 2.1) about 260x220, title "Armillary Cache", an item grid layer of 2
  rows of 4 laid out by the inv_init rule (section 6: 36x32 cells, first flush top-left, last flush bottom-right,
  whole-pixel pitch): declare it in the JSON as `"grids": [{"layer": FILE, "columns": 4, "rows": 2}]`; "Bank-all"
  and "Take-all" buttons (26x23 or 36x36, INTERFACES.md 5) in the bottom corners as on the CoX and ToA chests;
  no chest model (iface.py does not draw models).
- Deliver: the component JSON, its drawing empty and with slot backs drawn where items would sit (sprite 170, or
  marked boxes), and a line-up beside the real chests 539, 771, 23, 868 and 155 drawn by `iface.py G --cache 242`.

## Read first

1. `INTERFACES.md`, all of it; sections 2, 3, 5, 6, 7 and 10 are your rules.
2. `DELIVERY.md` sections 1, 3 and 4 (how a 30xxx group reaches TSPS; nothing is pushed anywhere).
3. `proof/build_rebuild.py` and `proof/rebuild-148.json`: a worked example of a steel panel built by script.
4. The docstrings of `iface.py` and `ifgate.py`.

## Tools (Git Bash; `P` = `{PYTHON} -I`; `I` = `{ROOT}/studies/research-interfaces`)

- Build by script; the component form and its fields are as in `BRIEF-interface-rebuild.md` ("Build").
- Draw: `P I/ifgate.py draw NAME.json OUT.png` (pane 512x334, or the 1230x708 HUD pane when the JSON has
  `"overlay": true` or `"pane": "hud"`). Draw any real group for the line-up with
  `P I/iface.py G OUT.png --cache 242` (you may read and draw real groups for an original: there is no held-out
  twin; do not copy a real group's component list wholesale).
- Gate: `P I/ifgate.py original NAME.json --out DIR`. Construction (one root, kinds, cache sprites and fonts only,
  fits, nothing outside the root, converts with to_tsps.py) and the measured conventions, each with its real band:
  text fonts; text colours against the measured set (INFO: real panels use others too); the shadow rule; for a
  panel: frame is a titled steel border, its 11 pieces where script 227 puts them, title font, colour, shadow, box
  and alignment, content margins (INFO), buttons as 9px bevel boxes with centred p12/p11 labels, item-grid pitch,
  panel size against the real distribution (INFO), inner boxes from the standard set; for a HUD: fill-parent root,
  the 191511 frame around 897, red under green, bar heights 9-21px, name and value fonts and colours, size against
  the real boss HUDs (INFO). `VERDICT` is the worst scored line; INFO lines never decide it.

## How to work

1. Work only in `{ROOT}/builds/NAME/`; scripted builds; write files with your file-writing tool, never
   shell heredocs. Nothing leaves this machine.
2. Write `plan.md` first: every part, the rule that sets it (section and words), and "SILENT: choice, reason" where
   no rule decides.
3. Draw, look at it beside the real ones at 1x and 3x, gate. Up to ten rounds; `notes.md` per round: the gate's lines,
   what you saw in the line-up, what you changed.
4. Not allowed: server sprites (new art) unless the order says so; client scripts (TSPS custom interfaces carry
   none); changing any file outside your build folder; anything sent off this machine.

## Report (to `{ROOT}/builds/NAME/REPORT.md`, or as your final message if your session may not write
that file)

- Paths: JSON(s), build script, drawings (each state or variant), line-up picture(s), `plan.md`, `notes.md`.
- The gate's lines for every round and the final VERDICT.
- Part by part: the rule of INTERFACES.md that set it, or SILENT with your choice.
- Where INTERFACES.md was silent, ambiguous or wrong for an original, and what a next measurement should settle.
- O1: the plugin calls per tick, and which pane you assumed (DELIVERY.md 4.2 leaves it open).
- What in the line-up still gives yours away, in your own judgement.
