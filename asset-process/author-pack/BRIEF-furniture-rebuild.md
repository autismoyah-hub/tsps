# Brief: rebuild a real piece of furniture from its reference sheet

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server. The owner's
standard: an asset must read at once as the game's own art, and within that be as well made as possible. Quality
matters; speed does not. Take the time the model needs.

The task is a test of the authoring method and of you as an author: rebuild one real game object from its reference
sheet alone, so that in a line-up drawn by the game's own renderer it cannot be told from the real one. Several
authors are given this same brief and the same object; the results are scored against the real mesh, which you do
not get to see. Your object and your folder name are in the task line that came with this brief.

## Read first, completely

1. `{ROOT}/author-pack/RULES.md`: the rules (welded meshes, smooth shading with deliberate flat
   facets, few colours, how the game lights and draws, how real artists split a surface, techniques that worked,
   how to read a sheet) and the exact tool commands. Everything you need to know about the game's look is there.
2. `{ROOT}/author-pack/tools/README.md`: the author's tools (`predict_light.py`, `pairs.py`,
   `px2col.py`, `variants.md`).
3. The wireframe study sheets in `{ROOT}/author-pack/wireframes/` (real objects, not yours).
4. The reference sheet named in your task line: six orthographic views on a 32-unit grid, with bounds, counts and
   any recolour pairs in the header. It is your only source for the shape. View it at full detail: crop and enlarge
   regions with Pillow into your `ref/` folder.

## Tools (run from Git Bash on this machine)

- Python: `{PYTHON} -I` (there is no `python` on PATH).
- Blender: `"{BLENDER}" -b -P YOUR_SCRIPT.py` to build, then
  `blender -b YOUR.blend -P {ROOT}/tools/blender_export.py -- OUT.bin` to export. Build by script only,
  with `{ROOT}/tools/blender_kit.py` (game units, y up, materials named for the game's colours).
- Line-up: `python {ROOT}/tools/lineup.py OUT_DIR "REAL_ID,drive:/.../yours.bin" --views 4 --ambient A --contrast C`
  with the lighting values from your task line (`OUT_DIR` must not exist; other options are in RULES.md). Then
  LOOK at `OUT_DIR/lineup.png`.
- `python {ROOT}/author-pack/tools/pairs.py OUT_DIR` gives enlarged real | ours | difference pictures
  with a score per view.

## How to work

1. Work only in `{ROOT}/builds/NAME/` and `{ROOT}/render/NAME-*` (NAME from your task
   line). Write `build_blender.py`, save `NAME.blend`, export `NAME.bin`. Write files with your file-writing or
   patch tool, not through shell heredocs: backslashes get mangled on this machine.
2. Order of work: size and outline first; then colours (from the sheet's recolour pairs, or read off the real one
   with `px2col.py`); then tone placement by sampling the same surface on both; then how the surface is split and
   shaded; then flat facets and inlays.
3. Each round: build, export, line up against the real one, read the picture, write in `notes.md` what gives yours
   away and what you changed. At least six rounds, at most twelve. Use variants in one line-up to settle choices.
   Stop when you cannot name a difference you know how to remove.

## Not allowed

- Reading this object's mesh, model data or definition from the game cache: no `osrscache.py`, `anatomy.py`,
  `measure.py`, `compare.py`, `wire.py`, `real_dump.py`, `real_to_blender.py`, `scene.object_def`, or the cache files,
  for this object or its models. Drawing the real one in a line-up is allowed and required.
- Modifying anything under `{ROOT}/tools/`, `author-pack/` or `client-build/`, or any other build folder.
- Sending anything to an outside service or repository: the reference material is private. Do not commit or push anything.
- Reading other authors' build folders for the same object (`builds/cmp-*`).

## Report

When you finish, write your report to `{ROOT}/builds/NAME/REPORT.md` (and also give it as your final
answer). It must contain:

- paths of the final `.bin`, `build_blender.py`, `notes.md` and the final line-up picture;
- the exporter's final report (counts, size, colours);
- one line per round: what differed, what you changed;
- what still differs from the real object, honestly, and how you checked;
- which rules or tools helped, which were unclear, wrong or missing;
- how many build-export-line-up cycles you ran, and roughly how long you worked.
