# Building an asset that belongs in the game: rules and tools

Private to this machine. Written by the asset Teacher from measurements of the real game meshes.

## Why earlier work failed

It was judged in Blender studio renders, flat-shaded, with bevels to catch highlights, one flat colour per part,
at whatever size looked good. The game does none of that. An asset built that way is picked out at once.

## The rules

1. **Smooth-shaded by default; flat faces are a deliberate tool.** The exporter writes faces smooth. Design
   for that: a broad flat-looking surface must really be flat (coplanar neighbours), because any change of
   angle becomes a soft gradient. About a quarter of real objects also carry some flat-shaded faces, usually a
   small share, where a crisp facet is wanted (a carved star, a cut gem). Mark those with `mat(h, s, l, flat=True)`.
   A whole object flat-shaded is the toy look.
2. **Welded: one vertex per position.** The exporter welds everything that rounds to the same whole game unit.
   Never rely on a split edge or a bevel for a hard edge. A crisp edge is two rings of vertices a few units apart
   (the lighting turns within that short strip), or a flat-marked facet, plus a step of tone between the surfaces.
3. **Few colours; the renderer does the lighting.** As a default, 4 to 9 colours for a piece of furniture, with
   3 to 5 covering 90% of the surface. This describes the body of a piece: a bookcase full of books, or anything
   carrying many small things, rightly uses many more (one of ours, graded Strong by the owner, has 27). The
   figures in these rules are defaults drawn from typical real pieces, not limits; depart from them for a reason
   you can state. Inside one hue, a step of lightness marks a different surface (and on real wood
   the saturation climbs as the lightness falls: 3 at lightness 45, 4 at 41, 5 at 33 and 28, 6 at 24 on one real piece):
   a joint or groove darker than its surface, an end grain or recess darker again, a band lighter on top.
   Do not paint in-between tones to imitate how light falls; that is the renderer's work, and it depends on the
   object's lighting values (rule 8). Lightness depends on the region: a median of 24 to 39 across periods, but
   pale sandstone-and-timber regions run to 57 and 70. Take it from real neighbours, not from this sentence.
   Saturation is mostly 1 to 4 on the 0 to 7 scale.
4. **Real size, in game units.** 128 units is one floor tile; a person stands about 200 tall. Build in whole
   units. The base sits at height 0.
5. **Spend faces on the outline and on parts that read from the game's camera** (above, about 45 degrees,
   the object perhaps 60 to 120 pixels tall). Not on bevels, not on detail smaller than about 3 units.
6. **Open where unseen.** Undersides resting on the floor are left open in real assets.
7. **It is judged only in the game's own renderer, in a line-up beside real assets.** Never judge from a
   Blender viewport or render.

8. **Lighting values belong to the object's definition, and regions differ.** Real definitions carry an ambient
   and a contrast value (seen so far: 10/30, 20/30, and none). The line-up gives ours `--ambient 10 --contrast 30`
   unless told otherwise; the brief says which values the real neighbours use. Settle the lighting values first,
   from the brief. Once they match, a surface that renders darker on the real one than on yours, facing the same
   way, IS painted a darker tone (or the surface has an edge you lack). Decide tones by sampling the same surface
   in the same view of the line-up, one surface at a time, and keep the table.
9. **What a reference sheet tells you.** Its header lists the bounds, the counts and the recolour pairs
   (original colour, colour in game) as 16-bit values: hue = value >> 10, saturation = (value >> 7) & 7,
   lightness = value & 127. Those are the real colours; use them.
   Many sheets list none. Then read the colours off the real object in the line-up: paint a first guess, sample
   the same lit surface on both with `px2col.py` (author-pack/tools), and shift hue, saturation and lightness
   until they agree. A fresh author recovered a nine-colour palette this way to within a few lightness steps.

## How the game lights a model (read from the client's own code; use it, do not fight it)

- **The light is nearly horizontal.** Its direction is (-50, -10, -50) in x, down, z. The lit faces are those
  turned toward -x, toward -z (the sheet's FRONT) and, weakly, upward; faces turned to +x and +z (the back) are
  unlit. So walls turned toward it are bright and walls turned away go very dark, close to black when the
  definition has no ambient: every tone you see on an unlit wall comes from its vertices' votes; the top of things gets little of it. Each
  object adds its definition's `ambient` to a base of 64 and its `contrast` (times five) to a base of 768; a
  higher contrast value flattens the difference between lit and unlit sides. Modern, rounder meshes carry
  values such as 20/30 for that reason; a round object with none goes black on its far side.
  The ambient and contrast are part of the asset's design: choose them with the mesh and record them.
- **A smooth vertex's light is the plain average of the normals of every triangle that uses it**, one vote per
  triangle, whatever its size. So triangulation and extra vertices are shading tools:
  - a vertex on the rim of a flat face, shared with side faces, is pulled toward the sides; a vertex in the
    middle of the face is not. A face with no interior vertices can only blend between its rim values. To make
    a surface show its own true tone in the middle and soften only at the rim, give it an interior vertex or an
    inset ring; that is where the real assets' soft dark or light patches come from;
  - which way a quad is split, and a fan against a strip, changes how many votes each side gets;
  - hidden faces vote too: a 1-unit face tucked under a board changes the light along that board's edge, and
    stopping a strip 1 unit short of a surface removes its votes from that surface's corners. Both are
    legitimate tools. `predict_light.py` (author-pack/tools) computes the result before you spend a render.
- **From light to the tone you see.** Lit lightness = painted lightness x light / 128, kept between 2 and 126;
  hue and saturation never change. Light = 64 + ambient + (the vertex's directional share), where the share
  runs to about +85 for a face square to the light and about -85 for one turned away, before contrast. So with
  no ambient a surface with no directional light shows half its painted lightness, the brightest about 1.16
  times, and one turned fully away goes to the floor. Ported exactly in `predict_light.py`.
- **The average is not re-normalised.** A vertex where faces meet at a sharp angle gets less directional light
  in both directions (a cube's corner about 58 percent): sharp corners are pulled toward the ambient tone,
  neither as bright as the lit face nor as dark as the unlit one.
- **Only smooth triangles vote.** A flat-marked face adds nothing to its vertices' normals, so marking a face
  flat also changes the shading of its smooth neighbours. A zero-area triangle still counts as a vote and dims
  its vertices toward ambient; do not leave degenerate faces.
- **Shading is drawn in blocks of 4 pixels along each row.** A smooth strip under about 8 pixels wide cannot
  show a clean gradient; do not rely on one.
- **A flat-marked face receives only two thirds of the directional light.** It is not [quoted words withheld]:
  turned toward the light it is darker than a smooth neighbour of the same tone, turned away it is lighter.
  Real assets use flat faces on small parts (a few percent of the surface), such as the edges of a carved star.
- **How the game draws, which decides every layering question** (read from the client and proved by test):
  - **Scenery and ordinary NPCs** are drawn with a per-pixel depth buffer, between models and inside a model.
    Priority does nothing. A face lying exactly in another's plane is mostly invisible unless it carries a
    per-face depth offset: `mat(h, s, l, depth=3)`. That is how inlays, labels, rugs' patterns and painted lines
    are done on scenery without standing proud. (Three in ten real objects carry such offsets.)
  - **A player with everything worn** is one model painted in priority order with no depth test inside the
    figure. Priority is a worn piece's whole layering: a higher-priority face shows through a lower one in front
    of it; a face lying in another's plane shows whole if its priority is higher, not at all if lower, and
    unreliably if equal. Depth offsets do nothing here. Weapons and shields are priority 10 (the exporter's
    `--weapon` writes it); a body piece mixes several (a shell at 3, its pattern above it, arms at 10); a helm
    is 6 with its visor at 7. Take the priorities from a real piece of the same slot.
  - Earlier versions of these rules said a coplanar strip never shows, and that there was no depth buffer between
    objects. Both were wrong; the first came from a figure tool that drew players with a depth test.
- **On scenery, real furniture also sets priorities by height band** (legs 0, seat 1, back 2). It has no effect in
  the game's scenery drawing; it is harmless to copy and not worth effort.
- **A single interior vertex or ring gives a faceted or framed patch, not a soft one.** Tested on the bench.
  Soft gradients across a flat surface on real assets come from how the whole surface is split, which a
  reference sheet does not show. Until the pack includes wireframes of real neighbours, do not chase them:
  match size, outline, colours and tone placement first.

## How real artists split a surface: look at the wireframes

`{ROOT}/author-pack/wireframes/real-ID.png` are study sheets of real assets: every edge drawn,
each face filled with its painted tone before lighting, flat-shaded faces outlined in red, and a second row
coloured by draw priority. Open several before you build (55076 bench, 15674 and 61111 barrels, 17987 crate,
27687 and 2333 tables). What they show:

- **The split is hand-made and irregular, on purpose.** A seat top is not a grid or one quad: it is fanned
  from a point placed off-centre on one edge, so that under the averaging rule the surface carries a slow
  diagonal gradient instead of a dead flat tone. Large faces sit next to slivers.
- **Faces are spent where the outline turns** and almost nowhere on flat expanses: a 328-face bench has about
  ten triangles on its whole seat top and dozens around a carved star.
- **Flat-marked faces are thin strips**: the sides of a joint between two boards, the lip of a seat. They keep
  one constant tone and read as a drawn line.
- **Tone by facing, per part:** upward faces the lightest tone, vertical faces one step down, undersides and
  feet darker; the same wood on a lower part is often one step darker than on the part above it.
- **Priority by height band:** legs, then seat, then back.

Triangulate by hand in the build script (give triangles, not quads to be split for you) wherever shading
matters, and say in your notes why each large surface is split the way it is.

## Reading a reference sheet, and what it hides

- A sheet is orthographic and lit differently from the game: trust its outlines, not its tones.
- Its scale is printed under each view (units per pixel); a feature's height is its pixel row times that, read
  at the row's centre. One author sat 1 unit low for five rounds by reading the row's edge.
- The game culls faces seen from behind. A real object may be an open shell you can see into on the sheet, and
  detail on a surface may vanish from one side because its faces point the other way. That is normal; copy it.
- A three-quarter view cannot tell a raised floor from a darker tone, or an undercut ledge from a thick slab.
  Check the side and back views for what lies beneath an overhang before you build it.
- At the line-up's default camera a unit of height is about 2 pixels and a unit of depth about 1.

## Techniques that worked on the first rebuilds

- **A seam between boards:** a 1-unit 45-degree chamfer where the boards meet. It is light on lit walls and
  faint on unlit ones, as the real seams are; a level ledge is equally bright on both and wrong.
- **Lines and panels on one plane:** split the plane into separately toned faces (no overlap). Overlapping
  coplanar strips do not show; partitioned faces do. Door frames, skirt strips and inlays are done this way.
- **Constant-tone marks** (writing on a page): small flat-shaded faces.
- **Grooves and bands:** two rings a few units apart with a tone step. A second vertex a few units from a
  corner on a long side does nothing useful and can leave a light wedge on a fanned top; do not sprinkle them.
- **No T-junctions, and no edge ending on another face's plane.** Where one face's edge meets the middle of
  another's edge, or a face stops exactly on the plane of a neighbour, the renderer shows a dotted crack. Split
  the neighbour so the vertices meet, or move one of them a unit.
- **A thin board with a sunk panel on each face:** if the two panel floors round to the same plane they weld,
  cancel each other's votes and are lit by ambient only. Make the board thick enough (6 units worked) for both.
- **Small parts just proud of a large face** (knobs, labels, handles) are cut into triangles by the depth sort if
  they share the large face's priority. Give them a higher one. Knob faces under about 6 units shade as two
  triangles when smooth; flat-mark them.
- **Shelving:** a shelf must draw over the things below it and under the things standing on it, so give each
  shelf and each row its own priority tier, rising with height. Behind a row of books it is the shelf top that
  shows, not the back wall, so the shelf top carries the interior's dark tone.
- **An evenly toned front** (a drawer, a door field): build it as the front face alone, with no top and bottom
  edge faces sharing its vertices, or the edges' votes sweep it from light to dark.
- **Things lying on a slope** round to ragged heights: give them about 1.5 units of clearance.
- **Props: few and large.** One or two that each take a real share of the surface read better than five small ones.
- **Flames and other things that should glow** go dark from behind unless they carry hidden faces turned to the
  light, or are flat-marked in a light tone.
- **Face budget:** aim within about a tenth of the real neighbours' count for a piece of that size. Well under
  means missing outline detail; well over means detail that will not read and shading you did not plan.
- **Compare variants, not rounds.** A line-up takes seconds: put several versions of your model in one call
  (`variants.md` in author-pack/tools) and keep the best, and use `pairs.py` for an enlarged real | ours |
  difference picture with a score per view. Scores on dark, unlit faces exaggerate: one lightness step there
  is a 10 to 15 percent difference.
- **When the same paint matches on lit faces and misses on unlit ones,** the cause is votes, not paint.

## Weapons and other worn equipment (from the first two weapon rebuilds)

Two authors rebuilt a real sword and a real crossbow from their sheets; in a figure's hand neither can be told from
the real one at game size. What they found, beyond the brief:

- **Worn space.** The model is authored where it is held: feet at height 0, the hand near 96 to 100. The part of a
  grip that is inside the fist is simply absent on real weapons.
- **One unit is one pixel in the figure's hand.** The furniture rule [quoted words withheld] does not hold: a
  1-unit chamfer, droop or offset is visible. But rings closer than about 10 units along a blade shade as kinks;
  put rings at crests and troughs only.
- **Lighting.** Worn gear is lit from above and in front (-30, -50, -30) with a stronger contrast base (850). In
  practice the front and one side are the brightest, tops a step below, undersides dark. A blade wants separate
  tones for its up-facing and down-facing faces; that is what the real ones do, and it is not [quoted words withheld].
- **One priority for the whole weapon, so the depth sort decides what shows.** A large face beside long thin ones
  is sorted over them (a guard cap cut a gold X into a blade). Faces that touch should be of similar size and share
  vertices. Keep small bright parts (studs, spikes) off the vertices of the part they hang from, or the weld dims them.
- **Hidden caps** at a junction (where a limb passes into a stock) give the junction's vertices the votes they need.
- **Reading the sheet.** Measure outlines by script, top and bottom edge separately, per pixel row; coordinates are
  whole units. The sheet's pixels are already palette colours, so `gamecol.nearest` on sheet pixels gives each
  region's hue and saturation even when no recolours are listed. The TOP view is mirrored left to right relative
  to the header's x. The sheet hides whatever sits behind another part: settle thin parts seen at an angle (a pommel,
  guard tabs) by variants in the figure's hand, not from the sheet.
- **Check for inside-out in round 1** (see the winding rule in `blender_kit.add_mesh`). A single-sided part such as
  a bowstring vanishing is the tell.
- **Tools:** `figure.py OUT.png "item:ID,model:PATH,none" [--zoom 260]` is the judge's view; put `none` (the bare
  figure) in as a column to mask the weapon exactly. `px2col.py` works on figure pictures. `pairs.py` and `lineup.py`
  do not apply. `predict_light.py`'s command line lights as scenery; for worn lighting call its `light()` function
  with `direction=(-30, -50, -30)` from a script, and treat the result as approximate (the contrast base differs).

## Armour (from the first rebuilds and the first original set)

- **An overlay at a higher priority must lie PARALLEL to the surface under it.** A player is painted in priority
  order with no depth test, so a higher-priority face is painted over everything lower, from any side. A flat device
  lying in its plate's plane is safe; a bevel, a raised boss or a rounded edge at a higher priority turns to face the
  camera from behind and is painted through the piece. Even one unit proud shows as a sliver at the outline side-on.
- **Test show-through, do not judge it by eye.** Draw the piece with and without the overlay from the rear
  (`--yaw 768`, `1024`, `1216`) and both sides (`640`, `1344`) through every frame of walk 819 and run 824, and
  count differing pixels (`pose.py --diff`). `builds/meridian-cuirass/through.sh` is a worked example.
- **Body pieces: stay at priorities 2, 3 and 4.** Priority 5 and above is painted after part of the arms and shows
  through a sleeve from above. Abdomen stub 2, shell and collar 3, flat devices, straps and buckles 4. Parts at one
  priority must not overlap each other: split a strap around its buckle.
- **Open helms:** the priority-6 part is the wearer's own head, rebuilt inside the helm (skin 4/3/70, hair-colour
  faces, eyes colour 0); metal is 7 and 8. A far wing or cheek guard painted over the face is the game's look.
- **Priority 0 does not hide a face.** It is painted first and shows wherever nothing later covers it. Faces kept
  only for their lighting vote must sit inside the outline and be tested: paint them a colour nothing else uses and
  count its pixels over the motion frames (`builds/armour-helm-10828/ghost_test.py`).
- **Split a picture difference into outline and tone before changing paint;** half of it is often outline. **Gloves** 10. **Boots** 0. Read a real piece of the slot before choosing.
- **Sleeved bodies (body and arms slots).** Arm tubes are always priority 10. A pauldron carries the shoulder label
  (21 / 25) so it swings with the arm, lies over the chest without sharing a coordinate with it, and needs a dark
  lining inside (its inside is culled; a tilted pauldron without one shows as a wire). Put the whole pauldron and
  its lining at 10: at 3 the near upper arm cuts it. How 10 is painted: a priority-10 face farther than the
  torso's mean depth is painted before the torso, the rest after everything. `label_armour.py`'s arm line
  (|x| >= 19) mislabels a platebody both ways; fix by part, by script, and check pivots against real armour
  (shoulder about (23.5, 156, 0)). Worked example: `builds/meridian-cuirass-v2/` (`label_fix.py`).
- **A boot cannot rise above the ankle ring;** the shin belongs to the legs piece. Every piece replaces its body
  part entirely and meets its neighbour at the neighbour's seam ring, at exact shared coordinates.
- **Laps between plates do not shade themselves,** and undersides are never seen. A lap reads only with a painted
  dark line about 2 units tall on a ring of its own under the upper plate's edge. A tone step alone is too weak.
- **Walls plumb where a device sits.** A leaning or rounded plate under a device rounds differently from it and
  leaks its colour onto neighbours in motion.
- **Silhouette first.** At game size a body piece is judged by its outline (shoulders, hem), then by two or three
  large soft tone changes, and only then by its device. A torso with no shoulders reads as a bib however well made.
- **House steel is hue 42, saturation 0** (the owner's choice, 2026-11-26, picked unlabelled over hue 21, which draws
  grey-green, and hue 0, the neutral grey of real iron and steel). Keep all steel on one hue and saturation so
  `tools/recolour.py` can move a whole set at once.
- `figure.py --zoom 300` gives a figure about 200 pixels tall: use it for the outline. Judge surfaces on
  `pose.py --aim SLOT --zoom 130 --cell 400x400`.

## Ids and what the server decides (from packing into the client, 2026-11-27)

- New ids: models 62000 and up but under 65536 (a higher model id loads from disk but cannot be fetched from a server);
  objects 63000 and up; items 60000 and up; NPCs must be UNDER 16384 (few are free: check); sequences 20001 and up.
  `studies/research-live/pack.py` checks every id free before writing and packs into a private copy of the cache only.
- A room belongs in the map square, not in server spawns: objects added after the map loads get no ground shading.
- A weapon's stand, walk, run and attack animations come from the server's item data, not from the item: say in your
  report which sequences your weapon needs. An NPC's attack and death likewise.
- Judge stone and other pale materials beside real ones in a real place: a wall that looks right alone can read near
  white beside the game's own stonework.

## What the TSPS client does differently (from a local run of the real client, 2026-11-29)

The owner's game runs on the TSPS browser client, not on our tooling client. Seen there with our own items:
- CORRECTED 2026-11-29 (the first version of this section blamed depth offsets and [quoted words withheld]; both were
  wrong; evidence `{TSPS_WORK}/notes/demo/DEMO.md` section 10 and `RULES-additions.md`, which you should read
  for the numbers).
- Per-face depth offsets do nothing in an inventory icon in EITHER client, and TSPS never reads them. Layer items
  with face priorities as real items do: overlay +1 to +4 priority over its base and 0.1 to 1 unit proud; never more
  than 5 units per priority step behind what covers it; avoid priorities 10 and 11 on inventory models. On scenery
  raise an overlay 1 to 2 units as geometry.
- Icons differ between the clients only on face edges (real items too, 10 to 17% of pixels): keep any colour feature
  that matters in an icon at least 2 icon pixels wide.
- TSPS LIGHTS models exactly as our tools do. It looks about 2 L* lighter because its default screen brightness is
  gamma 0.75 where our tools draw at 0.8 (a player's setting), and its camera looks down, showing the tops of
  shoulders and backs. Paint tops and backs knowing they will be seen.
- A chat head is cut by the dialogue layer, as in the native client; real tall hats overflow it too. To stay whole:
  nothing higher than 89 units above the head model's origin, nothing beyond 76 units from the vertical axis, less
  4 units for the talking animation.
- Wearable items go in `custom-items.json` (the server needs a gameplay definition: slot, style, animations) with a
  base item whose leaking fields must be reset; objects and NPCs go in `custom-definitions.json`. Steps, ids and
  commands: `{TSPS_WORK}/notes/demo/DEMO.md`.

## Tools (all run from Git Bash on this machine)

The author's own toolkit is in `{ROOT}/author-pack/tools/` with a README: `predict_light.py`
(what the game will do to your mesh, before a render), `pairs.py` (real | ours | difference, enlarged, with a
score per view, and several of your variants against one real), `px2col.py` (rendered pixels back to game
colours; runs of colour along a row or column), `variants.md`.


Python is `{PYTHON} -I` (there is no `python` on the path).
Blender is `"{BLENDER}"`.

- **Author in Blender by script** (no manual modelling). `{ROOT}/tools/blender_kit.py` lets a
  script work in game units with y up, as on the reference sheets, and makes materials named `hsl_H_S_L`
  (`mat(h, s, l)`, `add_mesh(name, verts, faces, colours)`, `reset()`, `save(path)`). Any Blender operation is
  allowed as long as every face ends up with an `hsl_H_S_L` material. Worked example of a welded, tone-painted
  object: `{ROOT}/examples/barrel/build.py` (direct) and `to_blender.py` beside it.
- **Export:** `blender -b YOUR.blend -P {ROOT}/tools/blender_export.py -- OUT.bin`
  It welds, smooths, and prints the size in game units and each colour's share of the surface. Read that
  report every time; it is the first check.
- **Line-up (the judge's view):**
  `python {ROOT}/tools/lineup.py OUT_DIR "REAL_ID,OUT.bin,REAL_ID,..." --views 2`
  Each entry is captured alone from the same camera in the game's renderer and tiled into `OUT_DIR/lineup.png`.
  `OUT_DIR` must not exist. Add `--dist 420 --elev 300 --crop 700` for larger objects (defaults 330, 250, 440),
  `--views 4` to see all sides (or `--yaws 0,45,315,180` to include the two front diagonals; a lower `--elev`
  gives a lower camera; `--small` also writes the line-up at about game size), and `--size 2x1` when the object covers two tiles, so yours stands where a real
  one of that size stands. Entries are real object ids or paths to your `.bin`. Ours is placed as a solid object,
  so the ground darkens around it as it does around real ones.
- Reference sheets of real objects: `{REFERENCE_ARCHIVE}/objects/shape-10/ID.png`
  (six orthographic views on a 32-unit grid, with the bounds in game units in the header).

## How to work

Build, export, line up, LOOK at the line-up picture, write down what gives yours away, change the build, repeat.
At least three rounds. An asset finished in minutes has not been looked at. Say plainly what still differs
when you stop.
