# PREREG: the gap between our best creatures and real ones, measures fixed before any is computed

Written 2026-11-27 before any measure below was computed on any creature. Read beforehand: COUNT.md, NPC-PLAN.md,
FINDINGS 39 45 54 60 64 69 70 79 80 85 87, CREATURES.md, CLEAN.md, HELM3.md and its PREREG. Run beforehand: only
`probe.py` (does each entry load; face and label counts; no measure). Nothing below may be changed after the
computation; anything added later is POST-HOC and cannot count. Words in quotes are the owner's (CORPUS.md).

## Lessons taken from HELM3 (0 of 17 survived)

1. Its measures were whole-shell averages; the faults it then SAW were local (a crest, a face, a cheek). Here two
   measures are about small features (C11) and part junctions (C6), and the boards are read for places.
2. Its separators were things the earlier rejects did wrong and the last one had fixed. Here a measure must call our
   BEST five ours, not only the rejects: the owner now puts those five [quoted words withheld] too (F10).
3. It had no construction measure of how the surface is split. Here seven of seventeen are (C1 to C8).

## Groups

- REAL (30): Tekton 7541 (2017, named by the owner), Minotaur 12812, Lynx 11068, Nex 11278, Vardorvis 12223, Branda
  12596, Eldric 14147, Araxxor 13668 (the newest classic bosses in AUTHORSHIP.md: Araxxor and the Royal Titans), Jaguar
  12976, Tormented Demon 13599, Basilisk Knight 9293, Sol Heredit 12821, Akkha 11789, Ba-Ba 11778, Duke Sucellus 12191,
  Yama 14176, Bull 15625, Alchemical Hydra 8615, The Nightmare 9425, Araxyte 11175, The Leviathan 12214, The Whisperer
  12204, The Hueycoatl 14009, Doom of Mokhaiotl 14707, Maggot King 15742, Callisto 6609, Zebak 11730, Kephri 11719,
  Amoxliatl 13685, Elidinis' Warden 11753. All model ids 36,000 or more except Tekton (and Yama's, known to sit in old
  gaps). Names checked in the census. Weighted ones measured as rest meshes.
- OURS (11) = BEST (5): Warden-General III, stone watcher III, star hound 3, Armillary sentinel (r8 model, NPC 16360),
  Zodiac ray (r8 export); and REJECTED (6): stone watcher I and II, star hound 2, Warden-General I and II, the orrery
  spider. Paths in `gaplib.py`.

## Common definitions

- Entry loaded by `research-clean/cleanlib.load` (the client's assembly and light). SKIN = drawn faces (alpha not 255,
  render type under 2, area over 0) that are opaque (alpha 0). Translucent faces are left out of every mesh measure.
- H = vertical extent of the skin's vertices. Edges: two-face edges of the drawn mesh (`cleanlib.topology`) with both
  faces in the skin. fold = absolute dihedral angle; concave = negative signed dihedral.
- LINE edge: the two faces differ in paint, or fold >= 30. HARD fold >= 50 degrees (CLEAN.md).
- PARTS (label-free, so weighted creatures count): k-means on skin face centroids / H, area-weighted, k = 16, seed 0,
  30 iterations, initial centres = 16 faces drawn with probability by area (numpy default_rng(0)). ELONGATED part:
  area-weighted PCA of its face centroids, sqrt(l1 / l2) >= 2; its axis = first eigenvector.
- PLANE (as CLEAN.md): faces joined across edges that are not lines.
- Pictures: `cleanlib.view` (client light). CLOSE: size 420, pitch 10, yaw 0, 45, 90. GAME: ppu = 0.445 x scale / 128
  (the harness's zoom 1150), pitch 39, same yaws. BODY pixels = pixels whose face is in the skin. d = |grey step|
  between 4-neighbour body pixels (grey = 0.299 R + 0.587 G + 0.114 B).

## The 17 measures, each with the direction the owner's words predict (REAL against OURS)

| # | owner's words (fault) | id | definition | REAL |
|---|---|---|---|---|
| C1 | [quoted words withheld], [quoted words withheld] (F8); edge flow along forms | `flow_aligned` | over edges with both faces in one elongated part: share of edge length within 25 degrees of the part's axis or more than 65 degrees from it (along the form, or a ring round it) | higher |
| C2 | rings and loops placed by anatomy (construction) | `ring_cv` | per elongated part, vertex positions along the axis, sorted; a new ring where the gap exceeds 2% of the part's axial length; with 4 or more rings, coefficient of variation of the gaps between ring centres; area-weighted median over parts | higher |
| C3 | plane sizes by part (construction); "detail" placed (F6) | `plane_by_part` | log10 (p90 / p10) of the 16 parts' median face areas | higher |
| C4 | [quoted words withheld]; fewer, bigger planes (F4, F8) | `big_planes` | share of skin area in planes larger than 2% of the skin area | higher |
| C5 | how silhouettes are cut; "crisp" (F8) | `sil_cut` | 8 views (yaw 0, 45 .. 315, level): contour edges (the two face normals point to opposite sides of the view); share of contour edge length with fold >= 50 | higher |
| C6 | how parts meet; [quoted words withheld] (F5) | `join_line` | classic creatures only (more than one label): face label = commonest of its vertex labels (tie: lowest); edges between faces of different labels: share of their length that is a LINE | higher |
| C7 | [quoted words withheld], "intention" (F8) | `crease_chain` | hard-fold edges chained through a vertex when exactly the continuing hard edge turns 35 degrees or less from the incoming direction (the straightest one if several); share of hard-fold length in chains of total length >= 0.10 H | higher |
| C8 | [quoted words withheld], [quoted words withheld] (F8, F9) | `dangling` | the same chains: ends at a vertex that has no other hard-fold edge and is not on an open edge, per (hard-fold length / H) | lower |
| C9 | [quoted words withheld] (F2) | `concave` | of edge length inside one paint, the share that is concave with fold >= 30 | higher |
| C10 | [quoted words withheld] (F8) | `relief` | area-weighted mean distance of skin vertices from their position after 10 rounds of Laplacian smoothing (lambda 0.5, umbrella over skin-edge neighbours), / H | higher |
| C11 | [quoted words withheld], "playdough" (F5) | `mark_seat` | MARK = connected skin faces of one paint, area <= 0.5% of the skin, area-weighted mean lit lightness (mean of the face's three lit corners) 12 or more from its ring R1 (skin faces sharing a vertex with it); R2 = skin faces sharing a vertex with R1, not in mark or R1. Seated if the R1 and R2 means differ by 4 or more (a socket, a frame or a graded edge). Share of marks seated (needs 3 marks) | higher |
| C12 | [quoted words withheld]: geometry spent where it cannot be seen (F9) | `subpixel` | share of skin faces whose area x ppu(game)^2 is under 0.5 pixel | lower |
| C13 | "uneven" (F8) | `asym` | 800 area-weighted surface samples (seed 0), mirrored x -> 2 xc - x (xc = middle of the skin's x range): mean distance to the nearest sample / H | lower |
| P1 | "crisp", "sharpness" (F8, F7) | `crisp_close` | CLOSE, mean over views: of body pairs with d >= 12, the share with d >= 40 | higher |
| P2 | "definition", [quoted words withheld] (F6) | `define_game` | GAME, mean over views: share of body pixels with a body neighbour at d >= 12 | higher |
| P3 | "depth" (F2, F7) | `depth_close` | CLOSE, mean over views: grey p90 - p10 of body pixels | higher |
| P4 | how silhouettes are cut; "misshaped" (F8) | `sil_turn` | CLOSE, mean over views: outer contour of all drawn pixels, `cv2.approxPolyDP` at 1.5 px; of the total absolute turning at the polygon's vertices, the share at vertices turning 45 degrees or more | higher |

## Test (as HELM3, adjusted for unequal groups)

1. The direction must match the table, or the measure does not count.
2. Leave-one-out: for each creature, on the others choose the single cut (midpoints between sorted values, predicted
   direction only) with the fewest BALANCED errors (each error weighted by 1 / its group's size; ties: the middle of
   the tied cuts); classify the left-out one. A measure SURVIVES if (a) LOO balanced accuracy >= 0.80 and (b) at least
   4 of the 5 BEST are called OURS in LOO (it must explain the verdict on our best, not only on the rejects).
   Creatures where a measure is undefined are left out of that measure (n reported).
3. Chance: 20,000 random relabellings (fixed seed) of the same creatures, 11 drawn as OURS and 5 of those as BEST;
   the probability each measure passes the rule; expected survivors by chance = the sum. The measures are correlated,
   so the effective number of chances is lower; said, not corrected.
4. Bands for `gapcheck.py`: REAL p10..p90 and min..max on the predicted side, and the full-sample cut.

## Look (step 4 of the task), fixed now

- Boards: each BEST beside its nearest real one (Warden-General III / Tekton, watcher III / Minotaur, hound 3 / Lynx;
  for the sentinel and the ray, the nearest in kind and size picked from a contact board of REAL BEFORE any number,
  recorded in LOOK.md), at game size and close, three views, plus a black-silhouette row.
- LOOK.md: where on ours I see the owner's words, written before I open the measure results. Then whether the
  surviving measures point at those places; seen faults no measure catches become REVIEW QUESTIONS in GAP.md.
