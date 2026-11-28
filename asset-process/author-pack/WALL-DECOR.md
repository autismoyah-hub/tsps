# Wall and floor decorations: how they sit and how they are gated

This page is for pieces that do not stand on the floor like furniture: a **wall decoration** (placement shape 4:
shield, banner, plaque, torch bracket, shelf) and a **floor decoration** (placement shape 22: rug, runner, floor
plate). Furniture is shape 10 and nothing here changes it.

All numbers are measured from the real game (640 real wall decorations and 695 real floor decorations, each taken
once). They are made by `tools/decor_bands.py` and stored in `tools/decor_bands.json`.

    PY="{PYTHON} -I"
    TOOLS={ROOT}/tools

## How a wall decoration sits (shape 4)

Build it in the place it hangs. The game does not lift it or push it to the wall for you; the height and the
distance from the wall are in the model's own points.

Coordinates as in `blender_kit` (x, y up, z), game units, 128 to a tile, tile middle at x 0, z 0:

- **The wall is the plane x = -64**, the west edge of the tile. The wall runs along z.
- **The room is on the +x side.** The back of the piece is at x = -64, the front faces +x.
- **Width goes along z**, centred on z = 0.
- **Height is y**, measured from the floor of the tile. A shield hung at chest height has its lowest point near
  y = 94 and its highest near y = 191.
- **Rotation** is chosen when the piece is placed, not in the model: 0 = west edge, 1 = north, 2 = east, 3 = south.
  Always author it for the west edge. (Placed at rotation 1, a model point (x, z) lands at (z, -x).)
- One tile wide. Only 14 of the 640 real wall decorations cover two tiles.

Three real pieces, to fix the picture:

| real piece | x (wall to front) | height | z (along wall) |
|---|---|---|---|
| Square shield 13766 | -64 to -53 | 94 to 191 | -29 to 29 |
| Banner 24018 | -64 to -60 | 56 to 223 | -57 to 57 |
| Torch 13341 | -72 to -47 | 157 to 230 | -9 to 9 |

### What the real ones measure (640 pieces)

| measure | middle half (p25 to p75) | median | gate PASS | gate BORDERLINE |
|---|---|---|---|---|
| lowest point above the floor | 56 to 128 | 94 | 0 to 169 | -74 to 189 |
| highest point above the floor | 192 to 230 | 210 | 127 to 319 | 56 to 448 |
| back of the piece from the wall plane (negative = sunk into the wall) | 0 to 0 | 0 | -1 to 4 | -14 to 34 |
| front of the piece from the wall plane (how far proud) | 10 to 39 | 16 | 2 to 113 | -3 to 128 |
| thickness (x) | 9 to 40 | 14.5 | 2 to 83 | 0 to 128 |
| width along the wall (z) | 70 to 124 | 102 | 32 to 128 | 14 to 151 |
| middle of the piece along the wall (z) | 0 to 0 | 0 | -8.5 to 8.5 | -36 to 33 |
| face count | 114 to 289 | 182 | 27 to 810 | 2 to 1859 |
| distinct colours | 3 to 10 | 5 | at most 22 | at most 34 |
| main materials (hue and saturation pairs with 3% of the surface or more) | 2 to 4 | 3 | at most 6 | at most 8 |
| largest triangle's share of the surface | 0.04 to 0.16 | 0.08 | at most 0.25 | at most 0.33 |

Which percentiles: the back-from-wall row and the last two rows use PASS p10 to p90, BORDERLINE p2.5 to p97.5.
The other rows use PASS p2.5 to p97.5, BORDERLINE p0.5 to p99.5. Outside BORDERLINE is FAIL.

Facts worth knowing:

- Two in three real pieces have their back exactly on the wall plane (x = -64). Three in four are within 1 unit.
  About one in eleven is sunk more than 1 unit into the wall (a socket, a bookshelf back); about one in seven
  stands more than 1 unit off it.
- 78% hang with their lowest point more than 40 units above the floor. About one in ten reaches the floor.
- If the room's masonry stands proud of the tile edge, the task line gives the wall plane to use (for example
  x = -32). Build the back of the piece on that plane, not on -64.

### A face lying in another face's plane needs a depth offset

A painted emblem, a map's drawing, a stripe: when its faces lie exactly in the plane of the surface behind them,
the game hides them unless each such face carries a depth offset. 227 of the 640 real wall decorations use this
(the Square shield's white bird is 22 faces lying in the blue field, all with an offset).

Give those faces a material whose name ends in `_d2` (or `mat(h, s, l, depth=2)` in `blender_kit`). Real pieces
use 2 most often, sometimes 4, 1 or 6. `wire.py` prints these numbers doubled (2 is shown as 4).

### Exporter warning to ignore

`blender_export.py` prints `WARNING: the base is at height N, not 0: the object will float or sink` for every
wall decoration. That warning is for furniture. For a wall piece check instead that the printed base height is
the hanging height you meant.

## How a floor decoration sits (shape 22)

- It lies on its own tile, centred on x 0, z 0, inside x -64 to 64 and z -64 to 64. Rotation turns it about the
  tile middle.
- Its lowest point is at height 0. Its **visible surface must be above 0**: a surface at height 0 is not drawn on
  our ground (tested with real object 17648, which lies entirely at 0: nothing appears). Real rugs put their
  surface at 2 or at 8.
- One piece per tile. A rug larger than a tile is made of tile pieces (corner, edge, middle); of 1109 real floor
  decorations one covers more than a tile.

### What the real ones measure (523 pieces that lie on the ground)

172 more real floor decorations go below ground (trapdoors, pits, bridges over a drop). They are left out,
because our floor has no hole under them.

| measure | middle half (p25 to p75) | median | gate PASS | gate BORDERLINE |
|---|---|---|---|---|
| lowest point above the ground | 0 to 0 | 0 | 0 to 2 | 0 to 5 |
| highest point above the ground | 8 to 37 | 15 | 1 to 154 | 1 to 280 |
| footprint, x and z each | 97 to 128 | 125 | 45 to 130 | 14 to 157 |
| overhang past the tile edge | 0 to 0 | 0 | 0 | at most 3 |
| face count | 43 to 199 | 94 | 6 to 490 | 2 to 1094 |
| distinct colours | 1 to 5 | 3 | at most 11 | at most 12 |
| main materials | 1 to 3 | 2 | at most 4 | at most 5 |
| largest triangle's share of the surface | 0.03 to 0.14 | 0.06 | at most 0.32 | at most 0.50 |

Which percentiles: lowest point, overhang, main materials and largest triangle use PASS p10 to p90, BORDERLINE
p2.5 to p97.5. The others use PASS p2.5 to p97.5, BORDERLINE p0.5 to p99.5. The lower limit 1 on the highest
point is the visibility rule above, not a percentile.

The 148 flat real pieces (nothing higher than 8: rugs, mats, footprints, floor marks) have their surface at
2, 4, 7 or 8 in most cases, 36 faces at the median, 2 colours at the median, and never more than 5 main materials.
A floor piece may be entirely flat-shaded; many real ones are. The gate does not count that against it.

## Looking at your own piece as it is mounted

Append `@4` or `@22` to the model path so the line-up mounts it with that shape. A wall piece is put on the north
edge of its tile, facing the front view. The default camera aims at a point 45 units up in the middle of the
tile, which cuts a hanging piece out of the picture, so aim it:

    $PY $TOOLS/lineup.py OUT "MODEL.bin@4" --ambient A --contrast C --aim=0,58 --up 140 --yaws 0,45,315,60
    $PY $TOOLS/lineup.py OUT "MODEL.bin@22" --ambient A --contrast C --up 0 --views 4

`--aim=EAST,NORTH` moves the point the camera looks at, in game units from the tile middle (0,58 is just inside
the north edge, where the piece hangs). `--up` is that point's height: use the middle of your piece. `--yaws
0,45,315,60` gives four views from the room side. The usual `--views 4` turns the camera behind the wall for three
of its four views and shows the back of the piece, which nobody sees in a room.

A real neighbour beside yours: `"13750@4,MODEL.bin@4"`. `pairs.py` works on such a folder as it does for furniture.

## How they are gated

### Rebuild of a real piece

    $PY $TOOLS/gate.py rebuild-object REAL_ID MODEL.bin --ambient A --contrast C

The same command as for furniture. The gate reads the placement shape from the real object (4 or 22 when the game
holds it as a wall or floor decoration) and prints it on its first line. `--shape 4` or `--shape 22` forces it.
A and C are the real object's own values: `$PY $TOOLS/objlight.py REAL_ID`.

It measures what the furniture gate measures (size, face count, colours, colour distance, three outlines, the
pairs score), with both pieces mounted as the game mounts them, and adds:

- **position x, y, z**: each end of your piece against the same end of the real one. PASS within 1 unit,
  BORDERLINE within 2. A piece of the right size hung 12 units off the wall fails here.
- **drawn outline as mounted, worst of four views**: the share of drawn pixels the two pieces have in common.
  PASS 0.97 or more, BORDERLINE 0.95 or more.
- For a wall piece the four pairs views are from the room side (0, 45, 315 and 60 degrees). For a floor piece they
  are the usual four all round.

### An original piece

    $PY $TOOLS/gate.py original-object MODEL.bin --shape 4 --faces LO,HI --size X0,X1,Y0,Y1,Z0,Z1 --bottom LO,HI
    $PY $TOOLS/gate.py original-object MODEL.bin --shape 4 --faces LO,HI --size X0,X1,Y0,Y1,Z0,Z1 --bottom LO,HI --wall-plane -32
    $PY $TOOLS/gate.py original-object MODEL.bin --shape 22 --faces LO,HI --size X0,X1,Y0,Y1,Z0,Z1

`--shape` must be given: there is no real object to read it from, and without it the piece is gated as furniture
and fails [quoted words withheld]. `--size` is the allowed extent on each axis: x is thickness for a wall piece, y is how
tall the piece itself is, z is its width along the wall. `--bottom` is the allowed height of the lowest point,
which is how high a wall piece hangs. `--wall-plane` is needed only when the task line names a wall plane other
than -64.

With `--shape 4` or `--shape 22` the furniture check [quoted words withheld] is not made. In its place the gate checks
every row of the table above for that shape, and prints the limits and the percentiles on each line. The other
checks stay: no two vertices on one position, no zero-area triangle, face count and size inside the task line's
ranges, and for a wall piece not every face flat-shaded.

### What a task line must state

For a **rebuild**: the real object id, and its ambient and contrast. Nothing about shape; the gate reads it.

For an **original wall decoration**:

1. shape 4;
2. the wall plane, if it is not x = -64;
3. the hanging height: lowest point LO to HI (`--bottom`);
4. size ranges: thickness (x), height of the piece (y), width along the wall (z) (`--size`);
5. face range (`--faces`);
6. ambient and contrast for line-ups.

For an **original floor decoration**:

1. shape 22, and which tile piece it is if it belongs to a kit (corner, edge, middle);
2. size ranges: x, height (y, with the top at 1 or more), z (`--size`);
3. face range (`--faces`);
4. ambient and contrast for line-ups.

## What these tools cannot do yet

- Transparent faces and textured faces are not carried by the exporter. 132 of 640 real wall decorations (flames,
  glass) and 110 of 695 real floor decorations have transparent faces; a rebuild of one of those cannot match.
- A real floor piece lying entirely at height 0 cannot be drawn here, so it cannot be a rebuild target.
- Wall pieces two tiles wide, and the other wall-decoration shapes (5 to 8, corner and offset mounts), are not
  covered.
