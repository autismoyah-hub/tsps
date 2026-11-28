# Brief: the first room, a scribes' hall

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server. The owner's
standard: it must read at once as the game's own art, and within that be as well made as possible; "higher detail but
feels very authentically like an OSRS asset". Quality matters; speed does not.

Our method has produced furniture the owner graded Strong: a set of four originals for a [quoted words withheld] (desk, chair,
records cabinet, bookcase) that stands beside the real game Lectern (object 22472). Nobody has yet built a ROOM under
this method. You are building the first: the shell (floor, walls, door, windows) and the arrangement that turns that
set into a place. The earlier, failed work made rooms as one giant model at twice real size, flat-shaded; do none of that.

## What a room is in this game

- A grid of tiles, 128 units each. A person is about 200 tall. Real doors are about one tile wide and 240 tall; real
  walls about 240 to 330 tall; real pillars 300 to 520.
- Built from separate objects placed on tiles: floor pieces, wall pieces one tile long, corner pieces, a door piece,
  window pieces, pillars, then furniture. Every piece is an ordinary model that obeys the same rules as furniture.
- Seen from a raised camera looking down at about 45 degrees. Near walls would hide the room, so the game's rooms are
  judged with the walls on the far sides visible and the near sides open or low. Build all four sides but compose
  your views accordingly (you may leave the near wall out of a view, as a cutaway, and say so).

## Read first, completely

1. `{ROOT}/author-pack/RULES.md` (rules, lighting, techniques, tools).
2. `{ROOT}/author-pack/tools/README.md`.
3. The furniture set: `{ROOT}/builds/original-scribe-desk/`, `original-scribe-chair/`,
   `original-scribe-cabinet/`, `original-scribe-bookcase/` (each has `design.md`, `notes.md`, the `.bin`). Their palette
   and vocabulary are the house style; the room must suit them without competing with them.
4. `{ROOT}/tools/vignette.py` and `scene.py`: how several pieces, real and ours, are placed on tiles
   with rotations and captured from chosen cameras. `scene.build` takes `extra_models` entries
   `(model_id, model_bytes, object_definition_bytes, x, z, rotation[, shape])`.

## Facts about the renderer you need (found by test; build on them, and report anything that contradicts them)

- A face lying exactly in the ground plane (height 0) does not show: the terrain wins. A floor piece must sit at
  height 1 to 3.
- A floor piece can be placed as a [quoted words withheld] (shape 22) so that things standing on the same tile draw over
  it. Its object definition must list its model for that shape: bytes `[6, 1] + model_id (4 bytes, big-endian) + [22]`
  then `[2] + name + [0, 14, 1, 15, 1, 17, 19, 0, 0]`. For ordinary objects use `author.object_def`.
- The bare terrain is a dull grey-olive; a room needs its own floor pieces to have a floor colour at all.
- There is no reliable depth buffer between objects: what draws over what is decided tile by tile and by the order
  the game draws tiles (far to near from the camera). A model that spans many tiles is drawn all at once from its
  own tile and will cover or be covered wrongly. Keep every piece within its footprint (one tile, or its declared
  size), and test from several camera directions.
- Whether a wall should be an ordinary object (shape 10) one tile long and thin, or needs the game's wall shapes
  (0 to 3) to sort correctly against furniture on neighbouring tiles, is NOT yet known. Find out by experiment early:
  it decides how the kit is built. `rust-client/src` in `{CLIENT_SRC}` is the client's
  source if you need to read how shapes are placed and sorted; `examples/authored_scene_snapshot.rs` is the program
  that draws the scene. Do not modify the client.

## Study real rooms' pieces before designing

Real wall, floor, door, window, pillar and rug objects are in the archive: sheets at
`{REFERENCE_ARCHIVE}/objects/shape-10/ID.png` and, for the game's own wall pieces, the
sibling folders `shape-0`, `shape-3`, `shape-4`, `shape-9`, `shape-22` (each with a JSON per object: name, faces,
bounds, model ids). Find candidates by name with a few lines of Python (`{PYTHON} -I`),
look at the sheets, and draw real pieces in a line-up or a vignette to see how they tile and how tall they are.
`python {ROOT}/tools/family.py 22472` lists real objects whose palette is close to the Lectern's.
You may make wireframe sheets of real pieces (`tools/wire.py ID OUT.png` works for shape-10 objects). You may not
import or trace a real mesh, and the room's pieces must be new designs. Using REAL objects as placed pieces in your
room, beside ours, is allowed where it helps judge scale and fit (the Lectern, a real figure `-16298`), but the shell
itself must be ours.

Write `design.md` BEFORE building: what you learned from real rooms' pieces (heights, thickness, how a wall piece
meets a corner, how floors are coloured and patterned, how windows and doors are done, face counts); the room's plan
on a tile grid (about 9 to 11 tiles wide and 7 to 9 deep) with every placement; the kit (each piece, its size, face
budget and palette); the materials (suggested: pale dressed stone for walls, darker stone or timber for the floor,
the set's wood for the door and beams, one accent that picks up the furniture's blue; decide for yourself after
studying real rooms that hold this kind of furniture); and how the room will read from the game's camera.

## How to work

1. Work only in `{ROOT}/builds/room-scribes-hall/` and `{ROOT}/render/room-scribes-hall-*`.
   Build each kit piece by Blender script (`blender_kit.py`), export each to its own `.bin` with `blender_export.py`,
   and write your own `compose.py` there that places the kit, the furniture set, the real Lectern and a real figure
   (by calling `scene.build` and `scene.capture`, as `vignette.py` does) and captures fixed cameras. Scripted only.
   Write files with your file-writing tool, never through shell heredocs.
2. Fix three cameras in round 1 and keep them: an overview from the south, a three-quarter view, and a view at about
   the height and distance the game is played at (for example target the room's centre, distance 1024, elevation 896,
   then try lower). Capture all three every round and READ them.
3. Judge each kit piece alone as well: a line-up of your wall piece, door, window and floor piece beside real ones of
   the same kind, as furniture is judged.
4. Each round answer in `notes.md`: does it read as a room of this game, and what would make a player doubt it? Is
   anything drawn over something it should be behind? Does the shell suit the furniture or fight it? Is the scale
   right against the figure? Then change something visible.
5. At least eight rounds, at most eighteen. Use variants (two wall treatments, two floor patterns) in separate
   captures to settle choices. Stop when you cannot name an improvement you know how to make.

## Not allowed

- One big model for the room, or any piece larger than its footprint.
- Copying, importing or tracing a real mesh.
- Modifying anything under `{ROOT}/tools/`, `author-pack/`, `client-build/`, or another build folder.
  If a tool fails or a rule seems wrong for rooms, report it; that is as valuable as the room.
- Sending anything to an outside service or repository; the reference material is private.

## Report back (this is all the Teacher will see)

- Paths: `design.md`, `notes.md`, `compose.py`, the kit's `.bin` files, the three final captures, the kit line-ups.
- The plan (a small text grid) and the kit list with face counts and sizes.
- What you found about floors, walls and draw order in this renderer: what works, what does not, with the evidence.
  This is the main thing I need.
- One line per round: what was wrong, what changed.
- Honest answers: what still reads as not-the-game; what a real room of the game does better; draw-order faults that remain.
- The three design decisions you are least sure of.
- Rules or tools that helped, misled or were missing for a room.
- How many rounds and captures you ran.
