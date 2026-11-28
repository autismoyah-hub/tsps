# Minor detail, measured: what a real boss carries that a passing rebuild lacks, and a gate line for it

Private to this machine (cache-derived). Step 2 of the paint research, 2026-11-26. Folder
`{ROOT}/studies/research-paint/detail/`; pictures `{ROOT}/render/research-paint-detail/`.
`P` = `{PYTHON} -I`, run in this folder. Everything below was run; what was not
is in the last section. All temporary storage of `tl.py`, `cmesh.py` and `rebuild_lib.py` is redirected to `work/`.

## 1. Method

**A mark** (`dlib.mesh_marks`) is a connected one-colour region of the drawn, opaque skin, at most 1.2% of the drawn
surface, that contrasts with what surrounds it: *strict* (cmesh's rule: every neighbouring region differs by 12 or
more paint lightness, or is another hue family) or *loose* (the tone study's rule: 20 or more grey levels from its
largest neighbour). Both are kept; each mark has its area, longest dimension and width in units, its paint contrast
and its body zone (`research-tone/decomp.py`'s zones).

**Followed into pictures.** The harness draws the real boss three ways under each camera: as the game draws it; with
every face in a code colour (the front face of every pixel); and with every mark painted out in the colour of its
largest unmarked neighbour (`dlib.paint_out`). So for every mark in every cell we know its pixel FOOTPRINT (pixels
where it is the front, opaque face) and its EFFECT (how many grey levels, 0-255, those pixels change when the marks
are painted out). Cameras: **game** (zoom 1150, pitch 220, 0.445 px per unit, yaws 192 / 704 / 1216 / 1728),
**close** (zoom 575, pitch 128, twice game size, two yaws), **board** (the blind line-up's own camera: 300 px cell,
pitch 128, yaw 192). Rows: rest pose, stand 0, walk 3, attack 4.

**Looked for in the rebuild.** A mark is VISIBLE in a cell at 2 or more pixels and 12 or more grey levels of effect.
It is KEPT there if the rebuild's picture, at that footprint, shows at least half of the mark's effect: the projection
of (rebuild minus real-with-marks-out, less its mean over the ring of unmarked pixels within 2 px: broad tone is not
detail) on (real minus real-with-marks-out), RGB. A shift of up to 2 px (game) is allowed, reduced per mark to 1 or 0
where the real-with-marks-out would itself answer a quarter of the mark at that shift (a neighbouring seam sliding into
the footprint is not this mark). Each mark counts once: the share of its visible cells in which it is kept.
The reverse is run too: the rebuild's own marks, looked for in the real picture ("spurious").

A first version (inside-minus-ring grey contrast, free 2 px shift) was discarded: the real Tekton with its loose
marks painted out still scored 0.58, because a painted-out seam found its neighbour.

## 2. Results

Checks of the measure (game camera): ceiling (real mesh through our encoder) 1.000 on both bosses; every mark
painted out 0.000 on both; the tone study's damaged copies fall in step: 10% of loose marks out 0.913, 50% 0.564,
100% 0.257 (the 82 strict-only marks remain).

| case | visible marks (game) | MISSING | recall, game | close | board | spurious (of ours' visible marks) | gate line | owner |
|---|---|---|---|---|---|---|---|---|
| Tekton, first rebuild | 123 | **93.1** | 0.243 +/- 0.030 | 0.291 | 0.235 | 132 of 245 | FAIL | found, [quoted words withheld] |
| Tekton, second | 123 | **95.9** | 0.221 +/- 0.029 | 0.241 | 0.192 | 110 of 210 | FAIL | not shown |
| Tekton, plate-first | 123 | **45.2** | 0.632 +/- 0.036 | 0.748 | 0.636 | 42 of 132 | FAIL | found, [quoted words withheld] |
| Nex rebuild | 87 | **31.9** | 0.634 +/- 0.038 | 0.681 | 0.557 | 114 of 350 | FAIL | found, [quoted words withheld] |
| Minotaur rebuild | 46 | **22.3** | 0.516 +/- 0.052 | 0.540 | 0.395 | 31 of 52 | PASS | taken for the real one |
| Lynx rebuild | 26 | 13.5 | 0.482 | 0.543 | 0.481 | 15 of 34 | PASS | no blind verdict recorded |
| Araxyte rebuild | 27 | 14.6 | 0.461 | 0.603 | 0.383 | 18 of 32 | PASS | found, by leg COLOUR (not marks) |

Mesh side (opaque marks, real / rebuild): Tekton 270 / 692, 695, 344; Nex 128 / 951. The first two Tekton rebuilds
and Nex's have 2.5 to 7 times the real's marks: fragments of paint fitted face by face, of which half are not in the
real picture. By longest dimension, real Tekton has NO mark under 16 units (10 of 16-32, 91 of 32-64, 169 over 64);
rebuild 1 has 145 under 32. Real Nex: 11 under 8 units, 19 of 8-16, 56 of 16-32, 42 larger; its rebuild 135 / 377 /
391 / 48. By paint contrast (grey 0-255: under 12 / 12-25 / 25-50 / 50+): Tekton 34 / 50 / 115 / 71; plate-first 36 /
53 / 116 / 139; Nex 33 / 26 / 39 / 30.

Recall by footprint at game size (the smaller the mark, the more often lost):

| | 2-3 px | 4-8 px | 9-20 px | 21+ px |
|---|---|---|---|---|
| Tekton first | 0.08 (n 6) | 0.15 (13) | 0.21 (9) | 0.27 (95) |
| Tekton second | 0.08 | 0.08 | 0.19 | 0.25 |
| Tekton plate-first | 0.25 | 0.37 | 0.44 | 0.71 |
| Nex | 0.25 (4) | 0.45 (9) | 0.50 (23) | 0.76 (51) |

By lit effect there is no gradient on the plate-first Tekton (12-25 grey: 0.68; 25-50: 0.63; 50+: 0.60): it does not
lose the faint marks, it loses the small ones and those of one zone. By zone: plate-first Tekton helmet 0.85, blade
arm 0.98, hammer arm 0.88, chest and apron 0.75, **legs 0.51 and 0.51** (70 of the 123 visible marks are on the
legs); first rebuild legs 0.09 and 0.30, helmet 0.17. Nex: **head and neck 0.23** (15 marks), arms and feathers 0.69
to 0.72, trunk 0.72, legs 0.74.

What the plate-first Tekton lacks, in numbers: 36 marks kept in under half their cells; median footprint 21 px at game
size (quartile 6), median effect 39 grey levels, median 80 by 14.5 units. They are not faint and not tiny in units:
they are the yellow and orange shin and thigh panels and their dark counterparts. Nex's 26: median 15 px, 28 grey,
20 by 3 units (the face, the collar).

Pictures (I read `tekton-close.png`, `nex-game.png`, the top half of `tekton-game.png` at half size, and
`recall-table.png`): on Tekton the plate-first rebuild keeps the visor slits, the chest seam, the six belt tongues,
the shoulder plate edges and the blade; the real's two long yellow shin panels and the orange thigh flashes are,
on the rebuild, small yellow chips on dark legs. The first two rebuilds show crumpled dark plates with orange chips
in other places: red nearly everywhere. On Nex the red is in the face and the breast plate; wings, tabard strips and
talons are green.

## 3. The smallest detail a viewer can still see at game size

No person was tested. What is measured:
- **In the picture, the floor is 2 pixels at 12 grey levels.** None of the 21 real marks whose largest footprint is 1
  pixel (Tekton 12, Nex 6, Araxyte 2, Minotaur 1) reaches 12 grey levels of effect in any of 16 cells; of the 2-3 px marks, 28 of
  43 do (Tekton 6 of 6, Nex 4 of 9, Lynx 6 of 7, Araxyte 11 of 20, Minotaur 1 of 1).
- **In units** (scale 128, 0.445 px per unit, so one pixel is about 5 square units seen square-on): the smallest real
  marks that register are Nex's face marks, 7.1 by 0.5 and 4.1 by 1.4 units (4 to 6 square units, 2 px, 13 to 22
  grey), and the Lynx's, 6.9 by 1.2 and 8.0 by 0.9 (7 to 8 square units at scale 102, 2 px, 20 to 35 grey); the
  Minotaur's eye-sized marks, 10 by 2.8 units (29 square units), give 4 to 6 px at 49 to 62 grey. Nex's six 1-px
  marks are 3 to 4 units long and about 1 wide (1 to 4 square units).
  **So: a detail under about 5 square units (roughly 4 by 1) is not in the game-size picture at all; from about 4 to
  8 square units with a lit contrast of 13 or more grey levels it is 2 pixels and registers; a detail meant to READ
  (an eye) is 25 to 30 square units, 4 to 6 pixels, at 50 grey levels.** On a creature drawn at another scale,
  multiply areas by (128 / scale) squared.
- **The owner's anchor.** The boards were `lineup.gif` at 300 by 300 px per figure (board-54, -61, -64 are 600 by 326
  GIFs). Tekton, 717 units tall, was drawn at 0.239 px per unit: about 171 px tall, HALF game size (at game size it is
  about 320 px). Nex at 0.398, the Minotaur at 0.328. At that scale 73 of Tekton's marks are visible by the rule and
  the plate-first rebuild lacks 26 of them: median 12 px on the board (quartile 8, smallest 2), 29 grey levels. The
  owner found it [quoted words withheld]. So marks of about 8 to 16 px at about 30 grey levels, 26 of them missing, are enough
  to be found at half game size. That is the only human evidence, and it cannot say which marks the owner saw.
- Whether the review page also showed `lineup.png` / `lineup-game.png` stills I did not establish.

## 4. The proposed gate line

**[quoted words withheld]** (`detailgate.py REAL_NPC_ID OURS.json`; exit 0 / 1 / 2):
MISSING = sum, over the real creature's opaque marks visible in at least one of the 16 game-size cells (4 yaws x rest,
stand 0, walk 3, attack 4; 2 px and 12 grey levels of effect), of (1 - the share of its visible cells in which the
rebuild keeps it, as defined in section 1). **PASS: MISSING at most 24. BORDERLINE: at most 30. FAIL: above.**
Printed with it, not gating: recall with a bootstrap standard error, recall by footprint class, the rebuild's own
visible marks that the real picture lacks.

Why a COUNT and not a share: as a share the line cannot be anchored. The Minotaur rebuild, which the owner took for
the real one, has recall 0.52; the plate-first Tekton, which the owner found, 0.63. What orders the owner's verdicts
is how many visible marks are absent: 22 (not told), 32 (found), 45 (found), 93 (found). A viewer finds a rebuild by
the number of places where something is missing; a boss has three times the marks, so the same share costs three
times the tells. The thresholds sit between the one [quoted words withheld] anchor (22.3) and the nearest "found" (31.9). For a
boss with 123 visible marks PASS means recall 0.80 or better.

Test (all through `detail.py`; the four marked * also through the `detailgate.py` command line, same numbers):

| case | MISSING | verdict |
|---|---|---|
| real Tekton through our encoder * | 0.0 | PASS (recall 1.000) |
| real Nex through our encoder | 0.0 | PASS |
| real Tekton, 10% of loose marks painted out | 10.7 | PASS |
| real Tekton, 50% out | 53.7 | FAIL |
| real Tekton, 100% of loose marks out (the copy that PASSES the tone line) | 91.4 | FAIL |
| real Tekton, every mark out * / real Nex, every mark out | 123.0 / 87.0 | FAIL |
| real Tekton, only marks under 9 px / under 21 px at game size out (77 / 92 marks, most of them hidden) | 18.0 / 23.9 | PASS / PASS |
| real Nex, only marks under 9 px / under 21 px out | 11.5 / 32.1 | PASS / FAIL |
| Tekton first, second, plate-first * | 93.1, 95.9, 45.2 | FAIL, FAIL, FAIL |
| Nex rebuild | 31.9 | FAIL |
| Minotaur * , Lynx, Araxyte rebuilds | 22.3, 13.5, 14.6 | PASS |

**The order of the three Tekton rebuilds:** first 93.1 and second 95.9 are the same within the error (recall 0.243
and 0.221, each +/- 0.03), then the plate-first far ahead at 45.2, still FAIL: the owner's order (found by
definition, then found by minor detail) is reproduced, and the passing rebuild does not pass this line. The measure
does NOT say the second rebuild improved on the first: its tone line fell from 12.1 to 9.6, its marks did not come
back. The order holds at shift tolerances of 2, 4 and 6 px (missing: Minotaur 22.3 / 19.0 / 16.3; Nex 31.9 / 27.6 /
26.1; plate-first 45.2 / 41.6 / 38.5; first 93.1 / 85.0 / 78.7).

## 5. Commands

```
P detail.py tekton1 tekton2 tekton3 nex minotaur lynx araxyte      mark-by-mark tables, work/results/TAG.json
P detail.py ceiling:7541 allout:7541 islands10:7541 islands50:7541 islands100:7541 under:7541:9 under:7541:21
P detail.py ceiling:11278 allout:11278 under:11278:9 under:11278:21
P detail.py --table [TAG..]                                        reprint tables from results
P detailgate.py 7541 {ROOT}/builds/boss-rebuild-7541-3/boss-rebuild-7541-3.json     the gate line (90 s)
P detailgate.py 7541 work/variants/7541-ceiling.json ; P detailgate.py 7541 work/variants/7541-allout.json
P detailgate.py --results                                          the line for every stored result
P board.py tekton nex table                                        the pictures
P explore.py ; P explore.py k 2,4,6 minotaur tekton3 tekton1 nex   other summaries; shift tolerance
```
Pictures: `tekton-game.png` (true size, x3, x3 with marks; front and rear quarter), `tekton-close.png`,
`nex-game.png`, `nex-close.png`, `recall-table.png`. Files: `dlib.py` (library), `detail.py`, `detailgate.py`,
`board.py`, `explore.py`.

## 6. Not proven

- No person was tested for visibility. "2 px at 12 grey levels" is a picture floor (below it no real 1-px mark has an
  effect) and a choice; the owner's verdicts say only that 26 missing marks of about 12 px and 30 grey were enough at
  half game size.
- The thresholds 24 / 30 rest on ONE [quoted words withheld] (Minotaur 22.3) and one nearest "found" (Nex 31.9), on different
  creatures. A count favours creatures with few marks: the Minotaur rebuild passes while keeping only half of 46.
  The line is a line for bosses; it needs a boss rebuild the owner cannot find before the numbers are calibrated.
- "Kept" is positional (2 px at game size). A mark present but displaced further counts as missing AND as spurious.
  On organic rebuilds (outline overlap 0.87 to 0.92) that lowers recall; the order survives 4 and 6 px.
- A mark here is any small contrasting one-colour region, up to 1.2% of the surface: on Tekton that includes whole
  small plate facets (median 75 by 14 units), not only what a person would call minor detail. The footprint classes
  are what is SEEN: Tekton's 2-3 px marks are large marks mostly hidden.
- Translucent marks (Tekton 3) and marks behind translucent faces are not scored. Death frames are not used.
  Only four rows of four yaws: a mark seen only in another frame is not counted.
- That [quoted words withheld] MEANT these marks is an inference: the measure shows where the plate-first
  rebuild differs most (small footprints, the legs), not what the owner looked at.
- Whether the Lynx rebuild had a blind verdict was not found in FINDINGS.md; it is not used as an anchor.
- Files under `studies/research-creatures/data/` dated 22:29 (stone watcher 2, Tekton 3) were not written by these
  tools (their cache is `work/data`); nothing else outside this folder and the render folder was touched.
