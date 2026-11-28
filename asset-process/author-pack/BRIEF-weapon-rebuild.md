# Brief: rebuild a real weapon from its reference sheet

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server. The owner's
standard: an asset must read at once as the game's own art, and within that be as well made as possible. Quality
matters; speed does not.

Our method has been proved on furniture (authors rebuilt real pieces that the owner could not tell from the real
ones, and made originals the owner graded Strong). Weapons have never been tried under it. You are one of the first
two authors to rebuild a real weapon, as a test of whether the method carries. Your real weapon and folder name are
in the task line that came with this brief.

## What is different about a weapon

- **It is a worn model.** It is authored in the body's own space, already in the hand: the figure's feet are at
  height 0, its hand near height 100, and the weapon's position and angle are part of the model. Your model must
  occupy the same place as the real one, or it will float beside the hand.
- **It is seen in a figure's hand, small.** A weapon is typically 70 to 260 units long, 20 to 80 wide and only
  8 to 45 thick; a blade is 4 to 12 thick. The earlier, failed work made weapons several times too thick.
- **Few faces, more colours per face.** Real weapons: median 250 faces (112 to 642 for most), about 10 colours with
  7 covering 90% of the surface, one draw priority for the whole model, almost never a flat-shaded face.
- **It is lit as part of the figure:** from above and in front (direction -30, -50, -30), stronger contrast base
  than scenery. Upward faces are the bright ones. There is no ambient or contrast value to choose.
- **Judged in a real figure's hand**, drawn by a build of the game client, beside the real weapon.

## Read first, completely

1. `{ROOT}/author-pack/RULES.md`. It was written from furniture; everything about welding, smooth
   shading, the one-vote-per-triangle lighting rule, tone steps, reading a sheet, and the tools applies. Its
   furniture figures (face counts, lightness, priorities by height, ground shading, lighting values) do not.
2. `{ROOT}/author-pack/tools/README.md`. `predict_light.py` can light a model from any direction.
3. The reference sheet of your weapon: `{REFERENCE_ARCHIVE}/items/slot-3/ID.png`.
   It has three blocks, each with six orthographic views and a header: the first is the inventory/ground model
   (ignore it); the second and third are the WORN model on male and female bodies. Rebuild the male worn model
   (second block). Its header gives the bounds in the game's coordinates, where the second number is height
   with the sign reversed: "min [-36, -163, -138], max [-24, -48, 102]" means x from -36 to -24, height from 48 to 163,
   z from -138 to 102. The header also lists recolour pairs when there are any (rule 9 in RULES.md decodes them).
   The sheet is tall; crop and enlarge its second block with Pillow (`{PYTHON} -I`)
   into your `ref/` folder to read it properly.

## Tools for weapons (all from Git Bash)

- Build by Blender script with `blender_kit.py` and export with `blender_export.py`, exactly as RULES.md describes.
  Give vertices in game units with y up, in WORN space (so heights around 40 to 170, not starting at 0). Ignore the
  exporter's warning that the base is not at height 0.
- **The judge's view:** `python {ROOT}/tools/figure.py OUT.png "item:ID,model:drive:/.../yours.bin"`
  draws a real figure holding each entry, one column per entry, from three angles, lit as the game lights worn
  gear. A different OUT.png each round, in `{ROOT}/render/NAME-rN/`. READ it, and crop and enlarge
  the weapon regions to compare.
- You may also view your model alone with `lineup.py` (it will stand in the air; that is fine) when you need a
  closer look at its construction, but judge in the figure's hand.

## How to work

1. Work only in `{ROOT}/builds/NAME/` and `{ROOT}/render/NAME-*`. Write
   `build_blender.py`, save `NAME.blend`, export `NAME.bin`. Scripted only. Write files with your file-writing tool,
   never through shell heredocs (backslashes get mangled here).
2. Order of work: position, size and outline in worn space first (compare the three angles of the figure picture);
   then colours (from the sheet's recolour pairs, or read off the real one with `px2col.py`); then tone placement by
   sampling the same surface on both; then how the surface is split.
3. Keep `notes.md`: per round, what gave yours away and what you changed; a tone table; why the blade or limb is
   split as it is.
4. At least five rounds, at most twelve. Use variants in one figure picture to settle choices. Stop when you cannot
   name a difference you know how to remove.

## Not allowed

- Reading this weapon's mesh, model data or definition from the game cache (no `osrscache.py`, `anatomy.py`,
  `compare.py`, `wire.py`, `real_dump.py`, `weapon_card.py`, or the cache files, for this item or its models).
  Drawing the real one with `figure.py` is allowed and required.
- Modifying anything under `{ROOT}/tools/`, `author-pack/` or `client-build/`. If a tool fails or a
  rule seems wrong for weapons, report it; that is as valuable as the model.
- Sending anything to an outside service or repository; the reference material is private.

## Report back (this is all the Teacher will see)

- Paths: final `.bin`, `build_blender.py`, `notes.md`, the final figure picture.
- The exporter's final report.
- One line per round: what differed, what you changed.
- What still differs from the real weapon, and how you checked.
- For weapons specifically: which rules in RULES.md were wrong or useless, what was missing, and what you would
  tell the next weapon author.
- How many build-export-figure cycles you ran.
