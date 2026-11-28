# How real rooms are furnished (measured; a hypothesis until the owner judges a room composed to it)

Status 2026-11-29: measured from 2,074 real rooms (96 studied in detail) by an Opus research agent after the owner graded a Producer-composed room Usable ([quoted words withheld]). The checker is `{ROOT}/studies/research-rooms/roomcheck.py PLACEMENTS.csv --kind KIND` (run it where it lives). CAUTION: the checker measures closeness to the game's own rooms, not the owner's grade: it fails on some rules both rooms the owner graded Strong. Use it to catch a bare or aimless room, not as an acceptance test.

---

# How the game furnishes a room, and how to furnish ours

Measured, not remembered: 96 real interior rooms from the revision-239 map (16 armouries and guard halls, 10 libraries,
7 workshops, 6 smithies, 10 kitchens, 10 bedrooms, 6 throne rooms, 6 shops, 4 banks, 8 temples, 6 laboratories,
6 taverns, 1 store; 16 of them tower rooms; 65 classic, 28 recent), against a population of 2,074 distinct enclosed rooms.
How they were found and every number behind this page: `SCORES.md`, `kinds.txt`, `ranges.json`, `measures-study.csv`,
`objects-study.csv`, `companions.json`. Pictures: `render/research-rooms-sheets/`.

The checker is `roomcheck.py PLACEMENTS.csv --kind KIND` in this folder. It reads the compose table in the form
`night/armoury/compose.py` writes (`source, tile_x, tile_z, rotation, shape, width, depth`), finds the floor (every tile
with a shape-22 piece; tiles under a splayed corner are not floor), the doors, and what each piece is (from its file
name; columns `name, cls, func` override), computes the measures below and prints PASS or what is missing.
Every threshold is read from `ranges.json` at run time: the kind's own range when the study has 8 or more rooms of the
kind (armoury, library, kitchen, bedroom, temple), otherwise the range of all 96.

**What the checker is and is not.** It measures how close a room is to the way the game furnishes rooms of its kind.
It has NOT been calibrated against the owner's grades: there are only three graded rooms, and both Strong ones (the
observatory, the scribes' hall) also miss 8 to 10 of its 16 rules (they are sparser and stand further from the walls
than the game's rooms; `SCORES.md` and `scores/`). Use it as a floor and a to-do list, not as the grade.

## The words

- **Floor tiles**: tiles a player can stand on inside the walls. A tile under a diagonal wall is not floor.
- **Standing piece / furniture**: a named piece on the floor (shape 10/11) that is not a light or small clutter.
- **Wall edge**: one tile-length of wall round the floor, door edges excluded. A 9 x 9 tower has 35.
- **Against a wall**: some tile of the piece's footprint has a wall edge.
- **Touching**: footprints touch, sides or corners (no free tile between them).
- **Wall decoration**: a piece on a wall (shape 4 to 8) facing into the room: shields, banners, standards,
  paintings, candle brackets, plaques. Windows are NOT counted (in the game a window is part of a wall piece and
  cannot be told from a plain wall in the map), and nor are rails and beams that run the tile's whole width.
- **Centre zone**: the middle third of the room each way. **Far zone**: the last two rows of floor on the door's line,
  three tiles wide (five in rooms of 9 or more).

## The rules

Numbers are p25 (a quarter of real rooms of the kind are below) / median / p75 unless said; [quoted words withheld] means the
room's floor tiles. Each rule names its check in `roomcheck.py`.

### Density: is it sparse?

**D1. How many standing pieces.** pieces = rate x N / 10, with the rate between the kind's p25 and p90, aiming at
the median. Rates per 10 floor tiles (p25 / median / p90):

| kind | rate | 9 x 9 tower (69 tiles) | 11 x 8 hall (88) | real examples (id, place, world tile) |
|---|---|---|---|---|
| armoury, guard hall | 1.13 / 1.47 / 2.10 | 8 / 10 / 14 | 10 / 13 / 18 | #1955 Varrock palace (3222,3484); #114, #187 Lovakengj (1453,3747), (1524,3751); #1546 Burthorpe (2855,3536) |
| library, study | 2.15 / 2.83 / 4.06 | 15 / 20 / 28 | 19 / 25 / 35 | #1954 Varrock (3211,3494); #1790 Draynor manor (3090,3253); #104 Varlamore (1451,3196) |
| kitchen | 2.01 / 2.34 / 2.74 | 14 / 16 / 18 | 18 / 21 / 24 | #1917 Lumbridge (3208,3214); #1956 Varrock (3221,3494) |
| bedroom | 1.67 / 1.97 / 3.33 | 12 / 14 / 22 | 15 / 17 / 29 | #1802 Draynor (3100,3358); #1087 W Ardougne (2543,3288) |
| temple | 0.59 / 0.83 / 1.18 | 5 / 6 / 8 | 6 / 7 / 10 | #1923 Lumbridge church (3243,3210); #245 Lovakengj (1543,3808) |
| workshop, smithy | 1.20 to 1.36 / 1.57 to 1.78 / 2.27 to 2.77 | 9 / 11 / 16 | 11 / 15 / 20 | #1920 Lumbridge (3225,3254); #1938 Varrock (3230,3437) |
| throne, council | 0.91 / 1.47 / 2.04 | 7 / 10 / 14 | 8 / 13 / 17 | #1918 Lumbridge (3208,3222); #418 Hosidius (1781,3577) |
| shop, bank, tavern, lab | 1.06 to 2.50 / 1.69 to 2.72 / 2.0 to 2.9 | see ranges.json | | #1919 Lumbridge shop; #1474 Catherby bank; #1691 Falador inn; #1152 Yanille lab |
| rooms a player calls well furnished (top quarter, any kind) | 2.24 / 2.62 / 3.67 | 15 / 18 / 25 | | contact sheet `well-10.png` |
| bare rooms (bottom quarter) | 0.77 / 1.12 / 1.71 | | | `bare-10.png`: stair towers, empty halls |

Check: `D1 pieces` (between p25 and p90; prints [quoted words withheld] or [quoted words withheld]).

**D2. Floor used.** Furniture and standing lights cover at least the kind's p25 of the floor: armoury 0.13 (median
0.16), library 0.28 (0.38), kitchen 0.27 (0.33), bedroom 0.31 (0.41), all rooms 0.20 (0.28); well-furnished 0.35.
Check: `D2 floor used`.

**D3. Walls lined.** The share of wall edges with a piece against them is at least the kind's p25: armoury 0.17
(median 0.35), library 0.43 (0.53), kitchen 0.33 (0.50), bedroom 0.35 (0.57), all 0.28 (0.42); well-furnished 0.58.
Check: `D3 walls lined` (prints how many more wall tiles need a piece).

**D4. No long bare wall.** The longest stretch of one straight wall with nothing against it and nothing on it is at
most the kind's p75: armoury 5, library 3, kitchen 4, bedroom 3, all 4; well-furnished rooms 3, bare rooms 6.
Check: `D4 no bare wall`.

**D5. Wall decoration.** At least max(kind median, 1.0) decorations per 10 wall edges: armoury 0.82 so 1.0 (a 9 x 9
tower: 4), library 1.68, kitchen 1.98, bedroom 2.43, all 1.30; well-furnished 1.24 / 2.34 / 3.33 (5 to 11 in a
tower). The floor of 1.0 is ours, not the game's: a quarter of real armouries carry no separate decoration, but those
are mostly recent buildings whose wall pieces carry panels, frames and niches modelled in (Lovakengj, #114); our wall
kit is plain masonry. What hangs: shields and standards (Varrock palace #1955: 6 standards, 4 shields, 4 brackets on
108 tiles), tapestries and portraits (Lumbridge #1918), candle brackets. Check: `D5 wall decoration`.

**D6. Variety.** Distinct named things at least the kind's p25 (armoury 2, library 5, kitchen 6, all 4); median 5,
well-furnished 8. Real armouries are LOW in variety (median 4.5): they repeat one display piece. Check: `D6 variety`.

**D7. Corners.** At least the kind's p25 of the four corners have a standing piece within a tile (armoury 2,
library 3, bedroom 4, all 2; median 3 to 4). Check: `D7 corners`.

### Placement: is there a reason?

**P1. Against walls.** At least the kind's p25 of furniture touches a wall: armoury 0.52 (median 0.88), library 0.63
(0.68), kitchen 0.54 (0.71), bedroom 0.69 (0.83), shop 0.76 (0.94), all 0.46 (0.69). Check: `P1 against walls`.

**P2. Pieces that belong at a wall stand at a wall.** In the game these stand against a wall in at least three cases
of four (`companions.json`): bed 100%, chest 100%, statue 100%, plant 100%, light 95%, shelf/cabinet 84%, storage
(barrels, crates) 80%, machine 80%, rack/armour stand/dummy 78%, bookcase 75%. At least 3 in 4 of such pieces in a
room must touch a wall. Free-standing in the game: thrones (6% at a wall), altars (40%), seats (40%), tables (41%),
bank booths. Check: `P2 wall pieces at walls` (names each piece that is not).

**P3. Groups.** At least the kind's p25 of furniture touches another piece: library 0.81, kitchen 0.72, bedroom 0.53,
all 0.57 (median 0.83). Armouries are the exception (p25 0, median 0.77): their suits of armour stand ALONE at
intervals along the walls (a rack or stand is alone 55% of the time). What groups with what, as touching pairs over
what chance would give (lift): bookcase with bookcase 14x (runs), shelf with shelf 6x, storage with storage 5.3x
(barrels and crates in heaps), seat with table 4.2x, throne with table 4.0x, workstation with workstation 3.8x,
bed with storage 2.7x, light with altar or statue 6.5x (flanking), rack with rack 1.9x. Mixed pairs of unlike kinds
(rack with storage 0.25x, seat with storage 0.17x) are rarer than chance. Check: `P3 groups`.

**P4. Companions.** A seat touches a table, desk or workstation (68% of real seats do); a workstation has a companion
(storage, seat, table or a second station: 55%); a bed has a chest, cabinet or shelf beside it; a throne has a table or
lights. Check: `P4 companions`.

**P5. Repetition: the room's signature piece, several times.** The most-repeated piece appears at least the kind's p25
times: armoury 5 (median 6: suits of armour 4 to 8 times, in 7 of 16 armouries the most repeated piece), library 6
(bookcases, median 8), bedroom 4 (beds), all 3 (median 4). Repeats stand in rows (library) or at even intervals
(armoury), and in mirror pairs. Variants of one piece (our a / b) count as the same piece. Check: `P5 repetition`.

**P6. Something faces the door.** In 86 of 96 real rooms (90%) a piece or a wall decoration stands in the far zone on
the door's line: a decoration (40 rooms), table (18), seat (15), storage (15), shelf (14), workstation (11), bookcase
(7), rack (6), bed (6), altar and throne (2 each). Armouries: a decoration, a rack or a cabinet. Check: `P6 faces the door`.

**P7. The centre.** By kind: armouries keep it clear (15 of 16; p75 0.00), as do workshops, smithies, bedrooms and
temples (temples lay a carpet there: centre floor decoration median 0.83). Libraries (median 0.50), kitchens (0.50),
banks and throne rooms put their main table or group in the centre. Check: `P7 centre` (kind median under 0.2: centre
at most the kind's p75; otherwise at least its p25).

**P8. Paths.** Every piece has a free side tile reachable from the door (real: 100% at the median), and at least the
kind's p25 of pieces are reachable by a path two tiles wide (armoury 0.84, all 0.62). Check: `P8 reach`.

**P9. Free floor.** Free floor between the kind's p10 and p90: armoury 0.68 to 0.90, all 0.62 to 0.87. This is the
[quoted words withheld] guard. Check: `P9 free floor`.

### Measured and reported, not ruled

- **Symmetry.** Mirror symmetry of furniture about the room's better axis: median 0.67 (armoury 0.56, temple 1.00,
  bedroom 0.81, tavern 0.40); same piece mirrored: median 0.38. About the door's axis: armoury median 0.30. Rooms are
  ordered, not mirror images: the scribes' hall (0.90) is above the real libraries' p90 (0.85).
- **Lights.** Named lights are few: median 0 per room, p75 1; throne rooms 2 to 4 and temples 1 to 5 (candles beside
  altars and thrones). Many real wall lights are unnamed brackets, counted here as wall decoration.
- **Clutter.** Separate small clutter is rare (20 of 96 rooms, mean 0.6): the game models it INTO the furniture (cooking
  pots on shelves, tools on benches, books in cases). Prefer furniture that carries its clutter.
- **Floor decoration.** Floor pieces (patterns, carpets, rugs, runners) cover a median 0.31 of the floor (armoury 0.30,
  temple 0.53 as a runner to the altar, kitchen 0.26). Almost all are unnamed. Our kit floors are not counted as
  decoration; rugs, rings and inlays are.

## How to furnish a room of a given kind and size (the order of work)

1. Count the floor tiles N and the wall edges E (the checker prints both). Read the kind's rates above: pieces
   = 1.5 x N / 10 for an armoury, 2.8 x N / 10 for a library, and so on; decorations = at least E / 10, aim at 2 x E / 10.
2. Choose the signature piece (armoury: armour stands / suits; library: bookcases; bedroom: beds; temple: pews) and
   place it 4 to 6 times (library 6 to 10): along walls, at even intervals or in runs, in mirror pairs about the door's
   axis or the room's axis.
3. Put the room's main piece or its best decoration in the far zone, on the door's line.
4. Line the walls: every piece of the P2 list goes against a wall; seats go to tables, stools to workstations, storage
   in heaps of two or three in corners, lights flanking the main piece or the door.
5. Decide the centre by kind (P7): clear with a floor feature (armoury, workshop, temple, bedroom) or the main table
   group (library, kitchen, bank, throne room).
6. Keep a free path two tiles wide from the door to every piece, and keep free floor above the kind's p10.
7. Run `roomcheck.py`. Fix every MISS line or say in the report why it stays.

## The armoury, worked

`compose_armoury.py r2` (this folder) is the wardens' armoury recomposed by these steps with pieces that already exist:
69 floor tiles, 35 wall edges. 14 standing pieces (rule: 8 to 14, typically 10); five armour stands (two in the north
corners, three along the west wall opposite the door at even intervals, the middle one on the door's line); two weapon
racks flanking the north wall's central slit; two arms chests flanking the door along the east wall; the grindstone
bench, its pail, a stool and a barrel as the work corner (SW); the two training dummies side by side (S/SE); centre left
clear with the brass ring and the hatch; four candle brackets (the only wall decoration piece that exists) flanking the
door and over the outer suits. `roomcheck.py`: PASS on all 16 rules. Beside the real armouries it is now on the full
side: above their p90 on wall lining, grouping, variety and lights (`SCORES.md`). Pictures:
`render/research-rooms-armoury-r2/compare.jpg`.

## Missing pieces (an order ready to place)

Nobody has built these; the recomposed armoury uses candle brackets from the scribes' hall where a real armoury would
hang shields and standards. Sizes in units (128 = one tile); "place" names the tile and rotation in plan r2.

| piece | shape | size, height | count | place (tile, rotation) | why, from the real rooms |
|---|---|---|---|---|---|
| Wall shield, the wardens' arms (kite shield with the brass star) | 4, wall decoration | 70 wide, 90 tall, hung 150..240, 12 proud | 4 | west (40,44) r0 and (40,45) r0 above and beside the central suit; south (42,40) r3, (46,40) r3 | Varrock palace #1955/#1957: shields between suits; real armoury decoration median 0.82 per 10 edges, well-furnished 1.2 to 3.3 |
| Standard / banner (long cloth, house colours, pole at top) | 4 | 80 wide, 190 long, top at 290 | 2 | flanking the door: east (48,42) r2 and (48,46) r2, in place of the candle brackets there | #1955: 6 standards on 108 tiles; #1918 Lumbridge: tapestries and banners |
| Crossed weapons plaque (two swords or glaives on a board) | 4 | 100 wide, 80 tall | 1 | south (44,40) r3, over the pail (the north wall's bays are full of racks; one decoration per tile) | real weapon displays on walls; adds the armoury's function to the walls, not only the floor |
| Wall torch (iron bracket, flame) | 4 | 30 wide, 70 tall, at 160..230 | 2 | replaces the scribes' candle brackets over the outer suits, (40,42) r0 and (40,46) r0; the east brackets give way to the standards | the candle bracket is a domestic piece; torches suit a guard tower |
| Brazier (iron bowl on three legs, coals) | 10, 1 x 1, standing light | 100 wide, 120 tall | 1 or 2 | SE corner (47,41) in place of the second dummy, or flanking the west display | light that stands at a wall or flanks the main piece (light at a wall 95%, light with statue 6.5x) |
| Weapon cabinet / sword cabinet (tall, glazed or open front, blades inside) | 10, 1 x 1 | 120 wide, 210 tall, 60 deep | 2 | the north corners (41,47) r0 and (47,47) r0 in place of two of the five stands (the west wall's free tiles sit under a slit or carry the suits) | real "Sword cabinet", "Bow cabinet", "Axe cabinet" (Varrock, Falador, Port Sarim #1722: 6 axe cabinets); cabinets 1.6 per armoury |
| Rack of shields or helms (wall shelf with helms) | 4 or 10 | 200 wide, 60 deep | 1 | south wall above the work bench (42,40)..(43,40) | shelves along walls (shelf at a wall 84%) |
| Rug / runner (wardens' blue with brass border; corner, edge, middle pieces as the scribes' rug kit) | 22 | 3 x 4 tiles | 1 | in front of the west display, x 41..42 x z 43..45, short of the ring | real armoury floor decoration median 0.30 of the floor; ours 0.12 |
| Arms crates (lidded, hafts showing) and a weapons barrel | 10, 1 x 1 | 100 x 90 / 90 x 110 | 3 | heaped by the barrel in the SW corner: (41,42) and (42,41) after the stool moves | storage touches storage 59% (lift 5.3): heaps, not single barrels |

## Limits of these rules

- Kinds with fewer than 8 study rooms (workshop, smithy, throne, shop, bank, laboratory, tavern) use the all-rooms
  range; their own numbers are in `ranges.json` (`table.py`) but rest on 4 to 7 rooms.
- Kind names come from the names of what stands in the room. "Armoury" includes halls and tower rooms lined with suits
  of armour (Falador castle #1697, #1698), Brian's axe shop (#1722, Port Sarim) and Varrock palace rooms; real
  dedicated arms stores are few in the map.
- Windows, wall panels and decoration modelled into wall pieces are invisible to this study (it reads placements, not
  meshes); recent buildings are therefore under-counted on wall decoration.
- The rooms were drawn on a flat floor without their terrain overlays; their floors look plain in the pictures.
- The rules have been checked against three graded rooms only, and the recomposed armoury has not been seen by the owner.
