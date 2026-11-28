# Brief: an original bone-weighted creature with its own skeleton and animations (DRAFT)

Status 2026-11-26: DRAFT from one creature made by this route (the Zodiac ray, `research-rig2/ray/`, not yet seen by
the owner). Not a qualified domain.

You are an authoring agent in a project that makes 3D assets for an Old School RuneScape private server. The owner's
standard: [quoted words withheld]. Quality matters; speed does not. A
weighted creature here is a NEW mesh of your own on a NEW bone skeleton of your own, every vertex weighted to its
bones, with its own keyframe animations: what the game's newest creatures are. It is for creatures that must BEND:
a long neck or tail, wings, tentacles, a soft body. (A creature of rigid parts is cheaper and better made by labels:
`author-pack/BRIEF-original-creature.md`.) You never copy, import or trace a real creature's mesh, weights or keys.
Your creature's idea, size and ranges are in the task line that came with this brief.

## Read first, completely

1. `{ROOT}/studies/research-rig/RIG.md` (the route, what a scene holds, the paint rule for a mesh lit
   in its bind pose, the rules for an author) and `{ROOT}/studies/research-rig2/RIG2.md` (the ring
   rule, the pose helpers, the bend, what is not proven).
2. `{ROOT}/studies/research-creatures/CREATURES.md` section 4 (the rules a creature mesh is built to)
   and `LIGHTING.md` beside it.
3. The worked example, all of it: `{ROOT}/studies/research-rig2/ray/`: `build_ray.py` (armature and
   skin from a script), `ray.parts.json`, `anim_ray.py`, `round.sh`, `notes.md` (what went wrong each round).
4. The tools in `{ROOT}/studies/research-rig2/` (read each docstring): `rig_weights.py`, `wlib.py`
   (`paint_rings`), `rig_pose2.py`, `rig_export.py`, `rig_timing.py`, `rig_light.py`, `style_rig.py`, `rig_view.py`,
   `rig_show.py`, `gif_frames.py`, `bend_view.py`, `rigpack.py`, `rig_byid.py`, `rig_fetch.py`.
   Python: `{PYTHON} -I`. Blender 5.2 headless:
   `"{BLENDER}" -b --python-exit-code 1 -P SCRIPT.py`.

## Study before designing

Fetch two real WEIGHTED creatures of your size and nearest body plan (`rig_fetch.py NPC work/NAME`; RIG2.md lists
49 by softness; `research-rig/data/rig-census.json` has all 157). For each: `rig_wstats.py` (how it is weighted),
`rig_view.py` (its rig card), `rig_timing.py` on its stand, walk, attack and death: length, key spacing, mean speed,
where the fastest moment sits, the near-still stretch, how many bones turn over 5 degrees, and draw them
(`rig_show.py`). Write in `design.md` BEFORE building:
- the idea in two words and why it needs weights (what bends);
- sizes in game units, the definition's scale, ambient and contrast (take the neighbours' unless you have a reason);
- the SKELETON as a tree with joint positions: 35 to 70 bones for one or two tiles; one root (not deforming); a chain
  of 4 or more bones wherever something must curl; bones drawn head to tail along the part; which joints are hinges;
- the BIND POSE: the pose it stands in, opened only as far as weighting needs; name the surfaces that must read
  bright and check they face up or front in it (RIG.md section 6);
- the PARTS LIST: every part of the skin, the bones it may use, rigid or bending. This becomes the parts file and is
  written before weighting;
- the palette BY PART (at least 11 colours, 4 to 9 tones in the main hue, steps of 4 to 9) and the list of at least
  16 inlaid details; paint anything that shows only when it turns over (a belly) far paler than it should look;
- the sequences: stand, move, attack, flinch, death; for each its length, key spacing, the pose it starts and ends
  on, and the measured neighbour it takes its timing from. If you cannot find a real flinch, say so.

## Build

1. One script builds the armature and the skin in Blender (no weights). ONE welded skin; vertex counts that change
   from ring to ring (a loft of equal rings fails the style check); details inlaid by splitting a face and
   repainting, never floating shells; whole game units.
2. `rig_weights.py --method nearest --shift 0 --rings soft|medium|hard --webs distance --max 3 --parts YOURS.json`.
   The method only chooses ONE bone per vertex; the ring rule paints the joints. Soft for what must bend like a
   tail or a wing, medium for a jointed limb (and for any hinge that closes past 90 degrees), hard or a `rigid`
   part for what must not shear. A membrane between two chains must come out with web vertices: if the tool prints
   0, your parts file is keeping the chains apart. Then look at `rig_view.py`.
3. `rig_pose2.py`: name your chains once (`Groups`), build WHOLE-BODY named poses, mix and key them, add
   follow-through down every chain, put a layer under everything, window the layers of every action that is not a
   loop, hold planted feet. Start every action but the stand on the stand's first pose.
4. `rig_export.py --base-id N`: no residual over 0.0001, no vertex whose weights do not sum to 255.

## Each round (at least eight, at most fourteen)

Build, weigh, animate, export; then `style_rig.py` (every GAME limit must pass; RECENT bands should, and so should
their upper sides: at most 45 colours, paint boundaries under 0.24), `rig_light.py` on the stand (changed by 16 or
more on at most a quarter of the surface), `rig_timing.py` on every sequence beside its real neighbour's, and for
EVERY sequence, beside the real neighbour: a contact sheet, a moving picture, and the moving picture's frames as a
sheet (`gif_frames.py`), from two sides, close and at game size. READ the frame sheets, every sequence, every round,
and write in `notes.md` which pictures you read, what gives it away beside the real one, any tear, crease, collapse
or wrong layering and the frame it is in, and what you changed. For a sharp joint use `bend_view.py`.
Last round: `rigpack.py` into a private cache copy under free ids and `rig_byid.py`: 0 differing pixels.

## Not allowed

Copying, importing or tracing a real mesh, real weights or real keys. Modifying anything under `tools/`,
`author-pack/`, `client-build/`, `builds/research-*` or another build folder. Touching any cache but your private
copy. Sending anything outside this machine. Shell heredocs (write files with your file-writing tool).

## Report (to `{ROOT}/builds/NAME/REPORT.md`)

Paths of the build, parts and pose scripts, the final model, frame base, keyframe files and definition, the packed
cache and the by-id line; the final `style_rig.py` output in full; bones, vertices, faces, influences per vertex,
joints ringed and web vertices; the timing table beside the real neighbours; boards (close, game size, two sides,
every sequence beside the real neighbour); one line per round naming the pictures read; any tear, hole, collapse or
wrong layering in any frame; what still reads as not-the-game, honestly; the three decisions you are least sure
of; where this brief or a tool failed you; what was not run.
