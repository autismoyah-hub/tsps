# Brief: rebuild a real creature from its reference sheet

Status: first use. The tools are proven on a ceiling case and on degraded copies; no creature has been rebuilt yet.

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server. The owner's
standard: an asset must read at once as the game's own art, and within that be as well made as possible. Quality
matters; speed does not.

Our method (measure real assets, REBUILD real ones held out, prove in blind line-ups that the rebuild cannot be told
from the real one, only then make originals) has been proved on furniture, weapons and armour. For creatures the
rebuild step was skipped, and the three originals made without it were graded "usable", one of them [quoted words withheld].
You are one of the first three authors to rebuild a real creature. Your creature (NPC id), its folder under
`targets/` and your build name NAME are in the task line that came with this brief.

## What is different about a creature

- **It is one welded skin that bends.** Not parts that meet: the same surface, with the vertex label changing along
  it. Every vertex carries a label naming the bone it follows; the faces where the label changes stretch.
- **It is lit ONCE, in the rest pose, and keeps those tones in every frame.** Top, head end (-z) and the -x flank are
  bright; the underside is near black unless the definition sets ambient. A bend never changes the light.
- **It is depth-tested per pixel like scenery.** Face priorities do nothing. A face lying in another's plane needs a
  depth offset or must be part of the skin (split the skin and repaint: that is how the real ones do nearly all
  their detail).
- **It is judged moving**: stand, walk, attack and death, from the front and from behind, beside the real one.

## Read first, completely

1. `{ROOT}/studies/research-creatures/CREATURES.md`: sections 2, 4, 5 and 6. Section 4 is the rules.
2. `{ROOT}/studies/research-creatures/LIGHTING.md`: "What the client does" and "Rules for a creature author".
3. `{ROOT}/author-pack/RULES.md`: "How the game lights a model" and "How the game draws".
4. `{ROOT}/studies/research-npc/NPCS.md`, and the docstrings of `npc.py`, `atlas.py`, `creature_kit.py`,
   `meshio.py`, `celldiff.py` in that folder.
5. Your sheet: `{ROOT}/studies/research-creatures/targets/npc-ID/sheet-ID.png` and `sheet-ID.json`
   (paint maps, label maps, palette, bounds, counts, definition, sequences, the label table), `archive-ID.png` (the
   same creature lit, six views), and `atlas/` (label atlases and rest meshes of the PERMITTED siblings on the same
   skeleton: these you may study as meshes, to see how real joints, rings and pivot markers are built).
6. The notes of the creature made before this test existed, for its mistakes: `builds/creature-star-hound-2/notes.md`.

## The three targets

| NPC | what it tests | skeleton, sequences (stand, walk, attack, death) | definition | sheet folder | meshes you may study | meshes you may NOT read |
|---|---|---|---|---|---|---|
| Lynx 11068 | a recent quadruped: 592 faces, one skin, 20 colours of one hue, 65 labels, 5 undrawn pivot faces | 1969; 9091, 9090, 9092, 9093 | scale 102, ambient 0, contrast 0 | `targets/npc-11068/` | Ocelot 14545 | every NPC named Lynx (11068 to 11070, 11197), Strangled Lynx 12290, and every jaguar of any colour, cub or spirit (12976 to 12979, 13021, 14512, 14585, 14590: a later target) |
| Minotaur 12812 | a recent humanoid monster: 1,798 faces, one skin, 18 colours, 131 labels, 29 undrawn pivot faces | 1758; 10840, 10842, 10843, 10846 | scale 128, ambient 10, contrast 20 | `targets/npc-12812/` | Porazdir 7515 | every Minotaur on this skeleton (12812, 12813, 13814, 13815, 15387 to 15389) |
| Araxyte 11175 | a mid-sized recent creature with painted detail: 1,020 faces, 11 colours (dark body, orange and green marks), 62 labels | 1800; 9137, 9138, 9140, 9141 | scale 36, ambient 10, contrast 30 | `targets/npc-11175/` | Sarachnis 8713 | every NPC whose name contains Araxyte (11174 to 11176, 13666, 13667, 13671, 13673, 13675, 13680), Rax (13682, 13684), Araxxor (13668, 13669) |

Lynx attack 9092 and death 9093 are the Teacher's reading of the skeleton's sequence list (the server tables do not
give them); use them as given. The JSON's numbers win over this table if they differ.

## Tools (Git Bash; `P` = `{PYTHON} -I`; `N` = `{ROOT}/studies/research-npc`; `C` = `{ROOT}/studies/research-creatures`)

- **Build** by script with `N/creature_kit.py` (welds by whole unit, first label wins; faces oriented outward; check
  concave places yourself) or directly as a dict through `N/meshio.py save`. An undrawn pivot face is a face with
  `alpha` 255: take `m = kit.model()`, set `m['alpha'][i] = 255` for those faces, `meshio.save(m, path)`.
- **Definition**: `NAME.json` beside your `.bin`, as in `N/npc.py`'s docstring, with the sheet's scale, ambient,
  contrast, size and its four sequences (`stand`, `walk`, `attack`, `death`).
- **Draw** (allowed with the real target: pictures are your reference):
  `P N/npc.py OUT.png "npc:ID;attack=A;death=D,def:PATH/NAME.json" --rows bind,stand:0,walk:2,walk:6,attack:3,attack:6,death:5,death:12 --zoom Z --cell 380x380 --diff 1:2`
  (`--game` for game size; `--yaw 1216` from behind; `--pitch 128`). `--diff 1:2` counts differing pixels per row.
  `P N/celldiff.py SHEET.png COLS ROWS 1:2 ROW OUT.png` enlarges one pair with the difference marked.
- **Predict the light** of YOUR mesh before a render: `P C/light_npc.py dump "file:PATH.bin;ambient=A;contrast=C" OUT.txt`.
- **Check your own mesh against the rules**: `P C/style_npc.py def:PATH/NAME.json`, and `P N/meshio.py NAME.bin`.
- **The gate is the Teacher's**: `P C/gate_npc.py rebuild-npc ID PATH/NAME.json --json gate.json --out RENDERDIR`.
  You do not run it (it reads the target's mesh). Its checks: the definition equal to the sheet's; bounding box within
  1.5% of the largest dimension; faces within 10%; colours at most half as many again as the real count; colour
  distance at most 2.5; rest outline overlap at least 0.93 in side, front and top views; no split vertices; flat
  faces no more than the real one's share + 0.03; label centroids within 3 units at the median; then, over bind and six
  frames each of stand, walk, attack and death, front and rear: outline overlap at least 0.88 on average and 0.78 in
  the worst cell, no more than 0.05 lost against the standing picture, tone difference at most 8 grey levels.
  All provisional.

## How to work

1. Work only in `{ROOT}/builds/NAME/` and `{ROOT}/render/NAME-*`. Scripted builds only;
   write files with your file-writing tool, never shell heredocs.
2. Order of work.
   a. **Read the sheet by script**: outline per pixel row and column in each paint map (the grid is 20 model units;
      the scale is printed), the palette, the label table. Write `plan.md`: the parts, their sizes in model units,
      each label's region and centroid, where each tone sits, the face budget per part (the sheet gives the total).
   b. **Skin and outline**, standing, three angles, against the real one. One welded skin. Split it by hand: fans,
      rings of unequal count, large faces beside small (rule 3).
   c. **Labels**: every vertex, by the label map and table. Put the centroid of each label where the table says, to
      a unit or two; where the sheet lists labels that sit only on undrawn faces, build those faces (alpha 255) so
      the centroid lands. Then MOTION: all four sequences, front and rear, every frame you can; a spike, a tear or a
      limb left behind is a label fault.
   d. **Paint**: the sheet's colours exactly, placed by the paint maps: by part and marking, not by facing (rule 4).
      Detail (eyes, teeth, claws, marks) is inlaid: split the skin and repaint (rule 10).
   e. **Tone in the renderer**: where yours and the real one differ on the same surface with the same paint, the
      cause is votes (how the surface is split, hidden faces), not paint.
3. Keep `notes.md`: per round what gave yours away and what you changed; the `--diff` counts for the standard rows;
   the worst frame and its cause; your own `style_npc.py` result.
4. At least six rounds, at most fourteen. Stop when you cannot name a difference you know how to remove.

## Not allowed

- Reading the target's mesh, or that of any creature in the last column of the table: no `asset_npc merged` or `dump`,
  `npc.py --dump`, `atlas.py`, `meshio.py`, `osrscache.py`, `census.py --npc`, `light_npc.py dump npc:..`, and none of
  `cmesh.py`, `palette.py`, `era.py`, `compare_npc.py`, `gate_npc.py`, `reencode_npc.py`, `sheet_npc.py`,
  `style_npc.py npc:..` on them; nothing under `C/data/`, `C/proof/` or `C/light-data/`. Drawing them with `npc.py`
  is allowed and required.
- Copying, importing or tracing any real mesh, including the permitted siblings: they are for study.
- Modifying anything under `tools/`, `author-pack/`, `client-build/`, `studies/research-npc/`, `studies/research-creatures/`
  or another build folder. If a tool fails or a rule seems wrong, report it; that is as valuable as the model.
- Sending anything to an outside service or repository.

## Report (to `{ROOT}/builds/NAME/REPORT.md`; it is all the Teacher will see)

- Paths: final `.bin`, `NAME.json`, the build script, `plan.md`, `notes.md`, the final boards (close, game size, from
  behind; real beside yours; all four sequences).
- Counts of yours against the sheet: size, vertices, faces, colours, labels, undrawn faces; `meshio.py`'s printout;
  your `style_npc.py` result.
- One line per round: what differed, what you changed.
- Motion: `--diff` counts per row for stand, walk, attack and death, front and rear, and what the worst frame shows;
  any tear, hole, spike or wrong layering in any frame.
- What still differs from the real creature, standing and moving, and how you checked.
- What the sheet did not tell you that you needed, and what it told you that made the task too easy.
- Which rules or tools were wrong, useless or missing; what you would tell the next creature author.
- How many build-draw cycles you ran.

## A boss: faces first, then paint, never the other way

Added 2026-11-28 from the tone study (`{ROOT}/studies/research-tone/`; untested until a boss is rebuilt
this way). Three boss rebuilds matched outline, face count and motion and failed on tone; the owner found one by
[quoted words withheld]; no repaint of them can pass. The same paint tool on a mesh whose faces sit where the real
ones do gives tone 0. (T = {ROOT}/studies/research-tone; P = the venv Python with -I)

1. Read the boss sheet (version 2, in `T/sheets/NAME/`): sheet.json, views/ (18 rest views: paint, lit, light, ink,
   translucent layer), pose/ (rest and 10 frames from 8 directions), enlargements/.
2. Every white line of an ink map is an EDGE of your mesh. Build the plates and markings first, as flat faces whose
   borders are those lines; then join them. Never fit an envelope to outlines and reduce it, and never let a script
   move vertices to match a picture: both leave a crumpled skin (median fold 35 to 40 degrees where the real boss has
   15 to 18; sheet.json gives the real figure, style_npc.py yours; stay within 5 degrees).
3. Budget faces by what each pose shows: `P T/layers.py --mine NAME.json SHEET_DIR` prints your faces shown at rest
   only, alive, in death only, never, beside the real counts. A rest pose can hold layers the living boss never shows.
4. Keep every vertex inside the sheet's bounds, the floor included.
5. Check the light before painting: `P T/normalfit.py aid NAME.json SHEET_DIR RENDER/aid`. Red and blue are where your
   SHAPE is wrong. Re-split and flatten that region by hand until the mean error is under 5 light numbers (a proposed
   mark, not a measured one).
6. Paint with the tool: `P T/paintfit.py NAME.json SHEET_DIR OUT --mode paint`. Its "contested" surface should be under
   5 percent (real mesh 0; the failed rebuilds 23 to 39). Each contested face lies across an ink line: cut it there or
   rebuild that plate.
7. Never repair tone with paint. If the lit picture is wrong where the paint is right, the shape is wrong: go to 5.

"Definition", in one sentence: every marking of the real boss is one unbroken shape whose border runs along edges of
the mesh, between flat plates each lit in one smooth tone; so make each ink line an edge, keep each plate flat, and
never let a face cross a line or a patch of paint stand in for a shadow.
