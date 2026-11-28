# Brief: an original piece of armour for the Meridian warden's set

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server. The owner's
standard, in the owner's words: [quoted words withheld]; "pushing the envelope
of making something that looks great (perhaps better than any assets authored by jagex) is a great direction". So: it
must read at once as the game's own art, and within that be as well made as you can make it. Quality matters; speed
does not.

Our method has produced furniture, a room and weapons the owner graded Strong, and an author has rebuilt a real
chainbody the owner could not tell from the real one. You are making one of the project's first ORIGINAL pieces of
armour. Four authors each make one piece of the same set; they will be shown together on one figure, standing and
moving, so yours must agree with the others through the house style below and meet them at the body's seams.

## The set

The **Meridian warden's armour**: what the wardens of an astronomers' tower wear with the Meridian arms (a longsword,
a glaive and a staff already made: `{ROOT}/builds/meridian-sword/`, `meridian-glaive/`, `meridian-staff/`,
each with `design.md`; look at them, they set the tone). Light, precise, a little ceremonial; a real fighter's kit,
not a costume. The house style:

- **Materials and palette.** Cool grey steel plates (hue 42, saturation 0 ONLY, the house steel the owner chose on 2026-11-26;
  lightness about 35 to 68, in three or four tones); warm brass trim and fittings (hue 7 to 8, saturation 4 to 5,
  lightness about 45 to 70); deep blue cloth or enamel as the accent (hue 37 to 38, saturation 4 to 5, lightness about
  20 to 40); dark leather straps and linings (hue 5 to 6, saturation 3 to 4, lightness 14 to 26). One small pale
  accent is allowed on the helm only (a moonstone: hue 30 to 34, saturation 1 to 2, lightness 90 to 110). Mostly
  steel; brass is the trim; blue is the accent. Bare skin, where your piece shows any, uses the colour 4/3/70 so that
  it takes the wearer's own skin colour.
- **Motifs, from an astronomer's instruments:** a ring or arc, a small sphere held in a ring, a four- or eight-point
  star, a crescent, graduated notches along an edge. One motif per piece, large enough to read at game size.
- **Trim rule, so four pieces agree:** every plate edge that ends a piece (a cuff, a hem, a collar, a rim) carries a
  brass band about 2 units wide; blue appears as one field per piece, not scattered.

Your piece, its slot, what it hides and its real templates are in the task line that came with this brief. It must
be a NEW design: not a copy of any real piece's layout, and no real mesh imported, traced or pasted. Learning from
real meshes is expected.

## What armour demands (read these before anything else)

1. `{ROOT}/author-pack/RULES.md`, especially "How the game draws" and "Weapons and other worn equipment".
   A player is painted in priority order with no depth test inside the figure: priority is your piece's whole layering.
2. `{ROOT}/author-pack/labels/LABELS.md` and the atlas pictures for your slot, and
   `{ROOT}/author-pack/labels/body-kits.json`: the exact geometry, labels, colours and priorities of the
   body part your piece replaces and of its neighbours. Your piece REPLACES that body part entirely: what you leave
   open is a hole, and where it meets a neighbour it must use the neighbour's seam ring at exact shared coordinates.
3. `{ROOT}/author-pack/tools/README.md`.
4. The notes of the two authors who rebuilt real armour, for how it is actually done:
   `{ROOT}/builds/armour-chainbody-1101/notes.md` (a body piece; coplanar pattern at a higher priority;
   walls kept plumb) and `{ROOT}/builds/armour-helm-10828/notes.md` (a helm that contains the wearer's
   head; thin double-sided parts and how they are lit; a camera calibration for overlaying a wireframe).

## Study real pieces of your slot before designing (allowed and expected for an original)

- See them worn: `python {ROOT}/tools/figure.py OUT.png "item:ID,item:ID,none" --zoom 300`, and close:
  `python {ROOT}/tools/pose.py OUT.png "item:ID,none" --aim head|body|hands|feet --zoom 130 --cell 400x400`.
- Read how a real piece is labelled and layered: `{CLIENT_BIN}/asset_pose.exe "{OSRS_CACHE_239}" info item:ID`
  prints its declared slots, its worn models' vertex and face labels and priorities.
- Read its mesh to learn construction: `python {ROOT}/tools/real_dump.py worn ID OUT.json`.
- Real armour by slot: median faces are helmet 264, gloves 92, boots 160, body 276 (the game's newest run to about
  twice that); a helm carries labels 1, 2, 3; gloves 19/27 and 22/28; boots 0, 32/48/45 and 38/47/46.

Write `design.md` BEFORE building: what you learned from the real pieces (how they close the body, their seams,
priorities, face spending, how they read at game size); your design in words with sizes in game units in worn space;
the palette with each colour's job; the motif and its size; the priorities you will use and why; and how your piece
will meet its neighbours in the set.

## Tools and how to work

- Python: `{PYTHON} -I`. Build by Blender script with `blender_kit.py`; export with
  `blender_export.py` (plain export; set priorities per face with `mat(h, s, l, prio=N)`).
- Label: `python {ROOT}/tools/label_armour.py YOURS.bin SLOT YOURS-labelled.bin` (add `--sleeveless` for a
  body piece that keeps the figure's arms). Read its histogram, pivot lines and warnings; compare the histogram with a
  real piece of the slot; fix by hand what the rule cannot know and say what you fixed. Use the LABELLED file below.
- Judge only on the figure, drawn as the game draws a player: `figure.py` standing (three angles) and `pose.py` in
  motion (`--seq 819` walk, `824` run, `390` slash, `386` stab; `--yaw 1216` for the back; `--aim` for a close view),
  with your piece as `model:PATH@SLOTSPEC` (SLOTSPEC in your task line), a real piece of the slot beside it and `none`.
- Work only in `{ROOT}/builds/NAME/` and `{ROOT}/render/NAME-*`. Write `build_blender.py`,
  save `NAME.blend`, export `NAME.bin`, label to `NAME-labelled.bin`. Scripted only. Write files with your
  file-writing tool, never through shell heredocs.
- Each round answer in `notes.md`: does it read as the game's own art, and what would make a careful player doubt
  it? Does it close the body with no hole or gap at any angle and in motion? Does anything show through that should
  not (priority)? Does the motif read at game size? Is it as good as the best real piece beside it? Then change
  something visible. At least six rounds, at most fourteen; use variants in one picture to settle choices.

## Not allowed

- Copying, importing or tracing a real mesh; lifting one real piece's layout.
- Modifying anything under `{ROOT}/tools/`, `author-pack/` or `client-build/`, or another build folder.
  If a tool fails or a rule seems wrong, report it.
- Sending anything to an outside service or repository; the reference material is private.

## Report back (this is all the Teacher will see)

- Paths: `design.md`, final labelled `.bin`, `build_blender.py`, `notes.md`, the final standing picture and the final
  motion pictures (front and back).
- The exporter's final report, the labelling tool's final histogram and warnings, and what you changed by hand.
- The exact SLOTSPEC to wear it with, its priorities, and the coordinates of every seam ring it shares with a
  neighbouring slot (so the set can be assembled).
- One line per round: what was wrong, what changed.
- Honest answers: what still reads as not-the-game; any hole, gap, tear or show-through, standing or moving; what the
  best real piece of the slot still does better.
- The three design decisions you are least sure of.
- Rules or tools that helped, misled or were missing for original armour.
- How many build-export-figure cycles you ran.
