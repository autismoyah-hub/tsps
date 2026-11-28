# Brief: an original weapon for the Meridian set

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server. The owner's
standard, in the owner's words: [quoted words withheld]; "pushing the envelope
of making something that looks great (perhaps better than any assets authored by jagex) is a great direction". So: it
must read at once as the game's own art, and within that be as well made as you can make it. Quality matters; speed
does not.

Our method has produced furniture the owner graded Strong, and two authors have rebuilt real weapons that cannot be
told from the real ones in a figure's hand. You are making one of the project's first ORIGINAL weapons.

## The set

The **Meridian** arms: the weapons of the wardens of an astronomers' tower. The house style, which all pieces of the
set share and which you must keep to so that three authors' weapons agree:

- **Materials and palette.** Warm brass for fittings (hue 7 to 8, saturation 4 to 5, lightness about 45 to 70 in three
  or four tones); deep enamel blue as the accent (hue 37 to 38, saturation 4 to 5, lightness about 20 to 40); cool
  grey steel for blades (hue 42, saturation 0 only, the house steel the owner chose on 2026-11-26; lightness about 35 to 68); dark walnut for hafts and grips
  (hue 5 to 6, saturation 3 to 4, lightness about 14 to 26). One small pale accent is allowed (a moonstone:
  hue 30 to 34, saturation 1 to 2, lightness 90 to 110). Nothing else. Mostly steel and brass; blue is the accent.
- **Motifs, from an astronomer's instruments:** rings and arcs (an armillary's bands), a small sphere held in a ring,
  compass points and four- or eight-point stars, crescent cut-outs, graduated notches along an edge. Use one or two
  on your weapon, large enough to read at game size, not all of them.
- **Character:** precise and elegant, not brutal: slender, a little ceremonial, but a real weapon.

Your weapon, its class, its real templates for placement in the hand and its size range are in the task line that
came with this brief. It must be a NEW design: not a copy of any real weapon's layout, and no real mesh imported,
traced or pasted. Learning from real meshes is expected.

## Read first, completely

1. `{ROOT}/author-pack/RULES.md`, including its section "Weapons and other worn equipment".
2. `{ROOT}/author-pack/tools/README.md`.
3. The two weapon rebuilds' notes and scripts, for how a weapon is actually built and placed:
   `{ROOT}/builds/weapon-fang-33174/` and `{ROOT}/builds/weapon-crossbow-33460/`.

## Study real weapons of your class before designing (allowed and expected for an original)

- Sheets: `{REFERENCE_ARCHIVE}/items/slot-3/ID.png` (the second block is the male worn
  model, with its bounds). Your task line names several real weapons of your class.
- See them in the hand: `python {ROOT}/tools/figure.py OUT.png "item:ID,item:ID,..." [--zoom 260]`.
- You may read a real weapon's worn mesh to learn how it is placed, how thick it is, how its faces are spent and
  toned: `python {ROOT}/tools/real_dump.py worn ID OUT.json` writes vertices (x, height, z), faces,
  colours and flags. Use it to find where the hand is (the gap in the grip, or the grip's centre) and at what angle
  the class of weapon is held. You may not import or trace it into your model.
- Weapon facts from the real ones: median 250 faces (most 110 to 640), about 10 colours with 7 covering 90% of the
  surface, a blade 4 to 12 units thick, one draw priority, almost never a flat-shaded face.

Write `design.md` BEFORE building: what you learned from the real ones of your class (placement, sizes, thickness,
face spending, how they read at 60 to 150 pixels long); your design in words with sizes in game units and its
placement in worn space; the palette with each colour's job, from the house style; which one or two motifs you use
and how large; and why a player reads it as that class of weapon and as the game's own art.

## How to work

1. Work only in `{ROOT}/builds/NAME/` and `{ROOT}/render/NAME-*`. Write `build_blender.py`
   (Blender Python with `blender_kit.py`), save `NAME.blend`, export `NAME.bin` with `blender_export.py`; read its
   report (ignore the base-height warning). Scripted only. Write files with your file-writing tool, never through
   shell heredocs.
2. Judge only in the figure's hand: every round a figure picture with yours among three or four real weapons of the
   class and a bare figure (`none`), and a close one (`--zoom 260`). READ them; crop and enlarge the weapon.
3. Each round answer in `notes.md`: does it read as the game's own art, and what would make a careful player doubt
   it? Is it held convincingly (in the hand, at the angle of its class, not floating, not through the body)? Does it
   read as its class and show its motif at game size? Is it as good as the best real one beside it, and what does
   that one do better? Then change something visible.
4. At least six rounds, at most fourteen. Use variants in one figure picture to settle design choices (silhouette,
   motif size, palette balance). Stop when you cannot name an improvement you know how to make.

## Not allowed

- Copying, importing or tracing a real mesh; lifting one real weapon's layout.
- Modifying anything under `{ROOT}/tools/`, `author-pack/` or `client-build/`, or another piece's
  build folder. If a tool fails or a rule seems wrong, report it.
- Sending anything to an outside service or repository; the reference material is private.

## Report back (this is all the Teacher will see)

- Paths: `design.md`, final `.bin`, `build_blender.py`, `notes.md`, the final figure picture among real weapons and
  the final close one.
- The exporter's final report.
- One line per round: what was wrong, what changed.
- Honest answers: what still reads as not-the-game; whether it is held convincingly; what the best real one of its
  class still does better.
- The three design decisions you are least sure of.
- Rules or tools that helped, misled or were missing for an original weapon.
- How many build-export-figure cycles you ran.
