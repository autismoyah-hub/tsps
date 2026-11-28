# Plan

Rewritten 2026-11-26 by the asset Teacher (Claude Fable 5.2) after the owner's mandate A-023: make the system as good as it can be in every area (objects, areas, rooms, weapons, armour, NPCs, animation), then target real content. [STATE.md](STATE.md) says which step is current. The plan of 2026-11-25 that this replaces is in version control.

## The aim, in a form that can be checked

An asset of ours, shown as the game draws it beside real assets, is one the owner takes for the game's own art and would keep, and at its best is better made than its real neighbours (A-022). For rebuilds of real pieces, used to prove a method on a new kind, the test is stricter: not told from the twin.

## The method, which is what the system is

Proved on furniture and on weapons (STATE.md). For every kind of asset the same five things, in this order:

1. **See it as the game sees it.** A harness that draws ours and real ones the same way: line-ups on terrain for scenery, a real figure for worn gear. Nothing is judged anywhere else.
2. **Read the real ones.** Measure the real meshes and the client's own code for that kind: sizes, face counts, colours, how it is lit, ordered, labelled and moved. Write what is found as rules in the author pack, as defaults with reasons.
3. **Rebuild before inventing.** Fresh authors rebuild two real assets of the kind from their sheets, without the mesh; the Teacher scores them against the real mesh. Their reports correct the rules and the tools. A kind passes when a fresh author matches a twin from the written pack alone.
4. **Originals as sets.** One brief, one house style, several authors; pieces judged together for whether they agree, beside the game's best of the kind.
5. **The owner's sitting.** Blind where a twin exists; grades and a ranking for originals; every answer filed and counted against the Teacher's prediction (`docs/verdicts/COUNT.md`).

What makes it work is in the author pack on the asset machine (`{ROOT}/author-pack/`): rules drawn from measurement, tools that enforce them, briefs that have produced accepted work. A change to the method is kept only if a score or a verdict moves.

## The route through the areas

Ordered by what each one unlocks and how sure the ground is. Status is in STATE.md.

**A. Objects (furniture, props).** Passed; owner's grade Strong. Remaining: playbooks from the briefs; kinds not yet tried (plants and other organic shapes, stone and metal, large architecture, anything with transparency or animation such as fires and fountains).

**B. Weapons.** Rebuilds passed; originals graded Strong. Remaining: prove they move with the hand (step C); more classes (bows, shields and off-hand items, thrown and two-handed heavy weapons); inventory icons and ground models, which every item also needs.

**C. Movement: labels, skeletons and frames.** The next investment and the one everything after it rests on. The game moves a model by labels on its vertices and faces; a weapon carries one label, a body many. Without them worn gear does not follow the body and nothing of ours can be animated.

- Harness: draw a real figure, real NPC or real object at a chosen frame of a real animation, with our model in place, using the dedicated client build.
- Read the real ones: how labels are assigned on weapons, armour, body kits and creatures; how a skeleton and its frames are stored; what the existing encoder for actions already writes.
- Rebuild: carry a real weapon and a real piece of armour through real animations with our labels and match the real one frame for frame.
- Then new movement: first variations of real sequences for our own models, then original sequences. This is the least certain ground in the plan; say so when reporting.

**D. Armour and worn equipment.** Built over the real body as weapons are built in the real hand, with labels from step C so each part follows its limb. Rebuild two real pieces first (a helmet and a body), then a set.

**E. People and creatures.** The game assembles people from body kits on one skeleton. Start by rebuilding a real figure from kit entries, then new figures by recolouring and exchanging parts, then by new parts, then creatures with their own skeletons, then a boss. Needs the owner's answer on whether an original may be made by editing real meshes (open in STATE.md); learning from them is already the practice.

**F. Rooms and areas.** A room from a set of ours: walls, floor, placement, light, judged from fixed cameras in the classic look and beside a real room of the game. Start with a scribes' hall from the set already graded Strong, then the tower lobby (A-015), then outdoor areas with terrain. Scale and the height of things come from real rooms, measured.

**G. The system itself.** Alongside the above, not after it:

- Playbooks under `teacher/playbooks/` for each brief that has produced accepted work.
- Authors compared: GPT-6.2 Astra and Sol against the Claude subagents on the same held-out rebuilds, same brief, scored (A-004).
- A reviewer that earns something: the first trial was at chance on telling ours from real; try it instead on naming defects that the owner or the scores confirm.
- Cost metered per accepted asset, from the runs' own usage figures.
- Concept pictures from real references compared with designing directly from real neighbours, on the same pieces (standing rule 2 and A-005).
- The private tools and pack kept in more than one place.

## Order of work from here

C first (movement), because B, D and E are not finished without it and it is the largest unknown. While authors and builds run, F's first room, since it uses what is already graded Strong and shows the owner an area. Then D, then E. G's items are taken whenever they unblock or de-risk the current step; the comparison of authors and the playbooks first.

## Not in this plan

- Catalogues and demonstrations built fast; the last one was unusable (A-017).
- Any judgment made on a Blender studio render, in the renderer's GPU mode, or on the Teacher's word alone.
- Speed as a goal, gates and receipts as progress, and praise of work the owner has not judged.
- Real content for the owner's game. The owner said it comes after the system is as good as it can be; a set made to prove a kind is a by-product, not a commission.
