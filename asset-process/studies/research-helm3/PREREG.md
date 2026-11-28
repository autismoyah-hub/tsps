# PREREG: helm3, measures fixed before any helm was measured

Written 2026-11-27 before any number was computed on any helm for this study and before looking at C's renders.
Only HELM2.md and the loaders were read. Nothing below may be changed after the computation; any later
measure is reported as POST-HOC and cannot count.

## Groups (14 helms)

- ACCEPTED (6): Torva 26382, Oathplate 30750, Justiciar faceguard 22326, Neitiznot faceguard 24271, Serpentine 12931,
  Meridian helm v5 (astra). Each in the outfit of HELM2 (h2.py).
- REJECTED (8): Umbral A, Umbral B, Umbral C, Meridian v4-a, v4-b, v4-c, v3-a, v2-a (the two by-eye helms of HELM2).
  C is worn on its own folder set (cuirass/legs/gauntlets/boots.bin in builds/umbral-helm-c).
- The five ACC+ helms of HELM2 are NOT used for the test (never judged). They may be printed for reference only.

## Common definitions

- Shell: the helm's faces that are not wearer faces (`fitlib.wearer_face`), degenerate faces dropped, vertices welded
  by position (3 decimals). Fold angle of an edge = angle between the normals of its two faces (two-face edges only).
- Plate: a connected set of shell faces joined across edges folded under 20 degrees (geometry only, paint ignored).
- Hard fold: welded edge folded over 45 degrees (35 where stated).
- Pictures: `tools/pose.py` through `h2.draw`, bind pose, head aimed. Game size = zoom 300 (cell 170x170); close =
  zoom 130 (cell 320x340). Views: front (yaw 0), quarter (256), side (512). Helm pixels = HELM2's `helm_cell` mask
  (differ with/without helm, not background, small stray pieces dropped). Grey = mean of RGB.
- Pixel pairs: 4-neighbour pairs with both pixels in the helm mask; d = |grey difference|.
- "Mean over views" = mean of front, quarter and side.

## The 17 measures and the predicted direction (ACC vs REJ)

| # | owner's word | id | definition | ACC predicted |
|---|---|---|---|---|
| 1 | crisp | `cr_hard_step_close` | close, mean over views: of pairs with d >= 12, the share with d >= 40 | higher |
| 2 | crisp | `cr_ramp_game` | game size, mean over views: of pairs with d >= 3, the share with 3 <= d < 12 (soft ramps) | lower |
| 3 | crisp | `cr_soft_fold` | mesh: share of shell edge length folded 15 to 40 degrees (neither flat nor decided) | lower |
| 4 | well molded | `mo_relief` | mesh: area-weighted mean distance of welded shell vertices from their position after 20 rounds of Laplacian smoothing (lambda 0.5, umbrella over welded neighbours), divided by sqrt(shell area) | higher |
| 5 | well molded | `mo_steel_shading` | close quarter view: grey p90 - p10 of helm pixels of the main paint family (the steel) only | higher |
| 6 | defined features | `df_line_contrast_game` | game size, mean over views: 90th percentile of d over all helm pairs | higher |
| 7 | defined features | `df_survive` | strong-edge pixel density (pixels with a pair d >= 24, per helm pixel) at game size / the same at close, mean over views | higher |
| 8 | defined features | `df_dark_cut` | game size front: share of near-black helm pixels (grey < 16) that touch (8-neighbour) a helm pixel lighter than the helm's grey p75 | higher |
| 9 | uneven | `un_edge_cv_plate` | mesh: for every plate of >= 4 faces, coefficient of variation of its welded edge lengths; median over plates weighted by plate area | lower |
| 10 | uneven | `un_face_logsd` | mesh: standard deviation of log(face area) over shell faces | lower |
| 11 | uneven | `un_asym` | mesh: mean distance from each mirrored (x -> -x) shell surface sample (600 area-weighted samples, fixed seed) to the nearest shell surface sample, divided by the shell's width | lower |
| 12 | misshaped | `ms_sil_front` | mesh silhouette, front (orthographic), of the shell minus the parts more than 3.5 units outside the bare head (HELM2's signature rule): radial outline r(theta) about the centroid at 360 angles; RMS of r minus its 6-harmonic Fourier low-pass, over mean r | lower |
| 13 | misshaped | `ms_sil_side` | the same seen from the side | lower |
| 14 | misshaped | `ms_dents` | mesh: on smooth welded shell vertices (every incident edge folded under 30 degrees), sign of (vertex - mean of neighbours) . vertex normal; share of smooth-vertex edges joining opposite signs (lumps and dents within a plate) | lower |
| 15 | no intention | `in_long_lines` | mesh: hard folds over 35 degrees chained into lines (through a vertex when one continuation turns under 35 degrees); share of hard-fold length in lines of total length >= 12 units | higher |
| 16 | no intention | `in_dangling` | the same chains: number of chain ends that are not on another hard fold or on the open boundary of the mesh, per 100 units of hard-fold length | lower |
| 17 | no intention | `in_mirror` | mesh: share of hard-fold (> 45 degrees) edge length whose mirrored (x -> -x) midpoint lies within 1.0 unit of the midpoint of another hard-fold edge | higher |

## Test

1. Direction must match the table. A measure with the wrong direction does not count whatever its separation.
2. Leave-one-out: for each of the 14 helms, take the other 13, choose the single cut (at a midpoint between sorted
   values, predicted direction only) with fewest errors on those 13 (ties: the middle of the tied cuts), classify the
   left-out helm. A measure SURVIVES if it classifies at least 12 of 14 correctly AND classifies Umbral C as rejected
   (C is the helm built to the last study's separator; a measure that calls C accepted cannot explain this verdict).
3. Chance: for each measure, the probability of surviving under random assignment of the 6 ACC labels to the 14
   values, computed exactly by enumerating all 3003 assignments (C's label fixed to rejected is NOT assumed; the C
   condition is evaluated on whichever helm gets C's role, i.e. over all assignments with C in REJ, weighted
   accordingly). Expected survivors by chance = sum over the 17. The measures are correlated, so the real number of
   independent chances is lower; this is said, not corrected.
4. Bands for helm3check.py: accepted min..max on the predicted side, and the full-sample cut.
5. The board: C, A, Meridian v5, Torva; front, three-quarter, side, close; hot spots of surviving measures marked.
   What I see on C is written down from the renders BEFORE the hot spots are drawn.
