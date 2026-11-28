# Brief: rebuild a real piece of armour from its reference sheet

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server. The owner's
standard: an asset must read at once as the game's own art, and within that be as well made as possible. Quality
matters; speed does not.

Our method has been proved on furniture and on weapons: authors rebuilt real pieces that the owner could not tell
from the real ones, and made originals the owner graded Strong. Armour has never been tried. You are one of the first
two authors to rebuild a real piece of armour, as a test of whether the method carries. Your piece, its slot and your
folder name are in the task line that came with this brief.

## What is different about armour

- **It replaces part of the body.** The game deletes the body part in the piece's slot (and any others the item
  names) and draws the piece instead. Whatever your piece leaves open is a hole in the figure. It must meet the
  neighbouring body parts at their seam rings, at exact shared coordinates.
- **It is worn space.** Feet at height 0, head top near 205; the character's right is negative x, front is negative z.
- **It must move.** Every vertex carries an animation label saying which part of the body it follows. A wrong label
  only shows when the figure moves: a spike, a tear, a part left behind. An unlabelled piece also deforms the wearer.
  `author-pack/labels/LABELS.md` and the atlas pictures beside it say which labels a slot uses and where; the tool
  `label_armour.py` assigns them by rule and tells you what it was unsure of. You are responsible for the result.
- **It is lit and drawn as part of the figure**: from above and in front, with several draw priorities on one piece
  in some cases.

## Read first, completely

1. `{ROOT}/author-pack/RULES.md`, including "Weapons and other worn equipment" (worn space, one unit
   is one pixel in the hand, the winding rule in `blender_kit.add_mesh`, reading the sheet by script).
2. `{ROOT}/author-pack/labels/LABELS.md` and the atlas pictures for your slot.
3. `{ROOT}/author-pack/tools/README.md`.
4. The two weapon rebuilds' notes, for how a worn model is built and judged:
   `{ROOT}/builds/weapon-fang-33174/notes.md`, `weapon-crossbow-33460/notes.md`.
5. The reference sheet of your piece: `{REFERENCE_ARCHIVE}/items/SLOTDIR/ID.png` (given in
   your task line). Its first block is the inventory model (ignore it); the second is the MALE WORN model, with bounds
   in the game's coordinates (the second number is height with the sign reversed). Rebuild the male worn model.
   Crop and enlarge that block with Pillow (`{PYTHON} -I`) into your `ref/` folder.

## Tools (from Git Bash; Python is `{PYTHON} -I`)

- Build by Blender script with `blender_kit.py`; export with `blender_export.py` (plain export, no `--weapon`).
- Label: `python {ROOT}/tools/label_armour.py YOURS.bin SLOT YOURS-labelled.bin` (SLOT as in your task
  line). Read its histogram, pivot lines, warnings and unsure list. Use the LABELLED file everywhere below.
- Standing, three angles: `python {ROOT}/tools/figure.py OUT.png "item:ID,model:PATH@SLOTSPEC,none" [--zoom 300]`
  where SLOTSPEC is given in your task line (it names the slot and the body parts hidden).
- In motion: `python {ROOT}/tools/pose.py OUT.png "item:ID,model:PATH@SLOTSPEC,none" --seq 819 --frames bind,1,3,5,7 --cell 400x400 --diff 1:2`
  (819 walk, 824 run, 390 slash, 386 stab; add `--yaw 1216` for a back view). `--diff 1:2` counts differing pixels
  between the real column and yours per frame.
- `px2col.py` reads colours off any of these pictures.

## How to work

1. Work only in `{ROOT}/builds/NAME/` and `{ROOT}/render/NAME-*`. Write
   `build_blender.py`, save `NAME.blend`, export `NAME.bin`, label to `NAME-labelled.bin`. Scripted only. Write files
   with your file-writing tool, never through shell heredocs.
2. Order of work: position, size and outline (standing, three angles); the seams with the body (no gap at any angle);
   colours and tone placement by sampling the same surface on both; then MOTION: walk, run, slash and stab, front and
   back, every frame you can; then how the surface is split.
3. Keep `notes.md`: per round, what gave yours away and what you changed; the label histogram and anything you changed
   by hand after the tool; the worst frame in motion and what caused it.
4. At least six rounds, at most fourteen. Stop when you cannot name a difference you know how to remove.

## Not allowed

- Reading this item's mesh, model data or definition from the game cache (no `osrscache.py`, `reencode_item.py`,
  `compare.py`, `real_dump.py`, `relabel.py`, `label_armour.py --check` or `--template` with this item or another item
  that shares its model, `asset_pose info` on this item, or the cache files). Drawing the real one with `figure.py`
  and `pose.py` is allowed and required.
- Modifying anything under `{ROOT}/tools/`, `author-pack/` or `client-build/`. If a tool fails or a rule
  seems wrong for armour, report it; that is as valuable as the model.
- Sending anything to an outside service or repository; the reference material is private.

## Report back (this is all the Teacher will see)

- Paths: final labelled `.bin`, `build_blender.py`, `notes.md`, the final standing picture and the final motion pictures.
- The exporter's final report and the labelling tool's final histogram and warnings.
- One line per round: what differed, what you changed.
- In motion: `--diff` pixel counts per frame for walk, run, slash and stab against the real one, and what the worst
  frame shows.
- What still differs from the real piece, standing and moving, and how you checked.
- For armour specifically: which rules or tools were wrong, useless or missing, and what you would tell the next
  armour author.
- How many build-export-figure cycles you ran.
