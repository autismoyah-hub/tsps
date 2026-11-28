# Brief: an original piece for the scribes' hall furniture set

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server. The owner's
standard, in the owner's words: [quoted words withheld]; "pushing the envelope
of making something that looks great (perhaps better than any assets authored by jagex) is a great direction". So: it
must read at once as the game's own art (its shading, palette, construction), and within that it should be as well
made as you can make it. It does not have to match the plainness of the game's older furniture. Quality matters;
speed does not.

## The set

A furniture set for the back office and library of a civic hall, the [quoted words withheld] set. It has two members so far:

- the real game Lectern, object 22472 (the set's origin: golden-brown wood in five tones, panelled sides, small square
  posts with grooved feet and capitals, a plinth, an open book with rows of writing);
- our own original scribe's desk: `{ROOT}/builds/original-scribe-desk/` (`design.md`, `notes.md`,
  `build_blender.py`, `desk.bin`). The owner called it [quoted words withheld] and of the same set as the Lectern. Read
  its design and notes: its palette table, its post and panel dimensions and its tone placement are the set's house
  style, and yours must agree with them (same wood tones doing the same jobs, same post and plinth vocabulary, same
  lighting values: ambient 30, contrast 0).

Your piece is named in the task line that came with this brief, with its size range. It must be a NEW design: not a
copy of any real object's layout, and no real mesh imported, traced or pasted. Learning from real meshes is expected.

## Read first, completely

1. `{ROOT}/author-pack/RULES.md` (rules, how the game lights and orders faces, how real artists split
   surfaces, techniques, tool commands).
2. `{ROOT}/author-pack/tools/README.md` (predict_light, px2col, pairs, variants).
3. The wireframe sheets in `author-pack/wireframes/` (22472 is the Lectern).
4. The desk's `design.md` and `notes.md`.

## Study real neighbours of your kind before designing

Find real objects of your kind: sheets are `{REFERENCE_ARCHIVE}/objects/shape-10/ID.png`
with a JSON beside each (name, faces, bounds). To find candidates by name, list the JSON files whose `name` matches
(a few lines of Python with `{PYTHON} -I`); prefer the game's most detailed and
newest examples of the kind as well as two or three classic ones. You may draw any real object in a line-up and make
wireframe sheets of real neighbours (`python {ROOT}/tools/wire.py ID OUT.png`, into your `ref/`).
Names in the cache are unreliable: look at a sheet before trusting it.

Write `design.md` BEFORE building: what you learned from the real ones (sizes, proportions, where they spend faces,
how they read at game size); your design in words with sizes in game units; the palette with each colour's job, taken
from the set; props if any; and why a player reads it as that piece of furniture and as the desk's and Lectern's sibling.

## Lessons from the desk, to apply

- Props: few and large. Real furniture carries one or two props that each take a real share of the surface. The
  desk's five small ones were its weakest part.
- Detail smaller than about 3 units does not read; a stack of thin bands merges into one edge at game size.
- Compare whole-object tone against a set sibling with `px2col.py rect` on both captures (share of each hue and
  saturation, and typical lit lightness); the desk matched the Lectern's front-view mean colour to one unit.
- The exporter's [quoted words withheld] counts hidden faces; judge tone balance from the rendered pixels.
- "Upward faces lightest" makes a piece pale when a top surface dominates the view; balance by what is seen.
- Check it at game size (shrink a line-up so the piece is about 100 pixels wide) as well as enlarged.
- Use variants in one line-up to settle design choices.

## How to work

1. Work only in `{ROOT}/builds/original-NAME/` and `{ROOT}/render/original-NAME-*`
   (NAME is given in your task line). Write `build_blender.py` (Blender Python with `blender_kit.py`), save `NAME.blend`,
   export `NAME.bin` with `blender_export.py`; read its report each time. Scripted only. Write files with your
   file-writing tool, never through shell heredocs (backslashes get mangled here).
2. Judge only in line-ups in the game's renderer, every round, with `--ambient 30 --contrast 0` and the `--size`
   given in your task line: (a) beside the set, entries `"22472,{ROOT}/builds/original-scribe-desk/desk.bin,YOURS.bin"`
   (pieces of different footprint sit off-centre; that is fine); (b) among real objects of your kind. New output
   folder each round. READ every picture.
3. Each round, answer in `notes.md`: does it read as the game's own art, and what would make a careful player doubt
   it? Does it agree with the desk and the Lectern (tones, posts, plinth, level of detail)? Is it as good as the best
   real one of its kind, and what does that one do better? Then change something visible.
4. At least six rounds, at most fourteen. Stop when you cannot name an improvement you know how to make.

## Not allowed

- Copying, importing or tracing a real mesh; lifting one real object's layout.
- Modifying anything under `{ROOT}/tools/` or `author-pack/`, or another piece's build folder.
  If a tool fails or a rule seems wrong, report it.
- Sending anything to an outside service or repository; the reference material is private.

## Report back (this is all the Teacher will see)

- Paths: `design.md`, final `.bin`, `build_blender.py`, `notes.md`, the final line-up beside the set and the final one
  among real objects of the kind.
- The exporter's final report.
- One line per round: what was wrong, what changed.
- Honest answers: what still reads as not-the-game; where it disagrees with the desk or the Lectern; what the best
  real one of its kind still does better.
- The three design decisions you are least sure of.
- Rules or tools that helped, misled or were missing.
- How many build-export-lineup cycles you ran.
