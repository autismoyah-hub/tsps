# Where the review pictures lose detail

Nine of the twelve sampled files contain fewer source pixels per tile than the native fixed client at the target plane. Seven are pitch boards. The shield is a close-up. The hound is also a close-up even though it shares the boss GIF's 600-pixel width. The largest measured colour loss is in the side-by-side encounter. Opening that film at natural size fixes display scaling, but its two palette conversions have already removed most of the original colours.

## Camera baseline

All densities below are local horizontal measurements at the camera target, in a direction parallel to the image plane. One tile is 128 world units. A transverse model span of length L projects to L × focal / view-depth. A ground tile seen along the viewing direction is foreshortened; its near and far edges have different depths. A single px/tile number cannot describe all faces of a boss or all tiles of an arena. For a point away from the target use its camera-space Z. Model units here are **after** any definition scale. Native projection is in `client/rust-client/src/render3d.rs:144–170,750–762`.

| Default on level ground, no shake or stretch, DPR 1 | 3D viewport | Pitch (2048/turn) | Focal px | Orbit units | px/unit | px/tile |
|---|---:|---:|---:|---:|---:|---:|
| Native fixed 765×503 | 512×334 | 128 | 256 | 984 | 0.2602 | 33.30 |
| Native resizable 1920×1080 | 1920×1080 | 128 | 662 | 1230 | 0.5382 | 68.89 |
| TSPS fixed 765×503 | 512×334 | 250.02 | 256 | 1350.06 | 0.1896 | 24.27 |
| TSPS resizable 1920×1080 | 1920×1080 | 250.02 | 662 | 1687.58 | 0.3923 | 50.21 |

The fixed window reserves space for the gameframe. Its 3D rectangle starts at (4,4) in TSPS `render/render/quality.ts:204–205`. The desktop calculation assumes the given dimensions are the drawable client area, a full-area resizable scene widget, and default script settings. Browser chrome, a smaller widget, saved camera settings, terrain pressure, wheel zoom and device-pixel ratio can change the result.

Native `src/camera.rs:347,612–625,747–757,1187–1190` supplies pitch 1024/8, orbit 600 + 3 × pitch, and the height-dependent distance multiplier 256/256 to 320/256. `src/client_preferences.rs:29` supplies a target 50 units up. `src/viewport_shape.rs:12–14,28–35,60` gives focal floor(height × FOV / 334), with FOV 256 at 334 high and 205 at heights ≥434. `src/live_runtime.rs:1912` feeds this focal to the root scene camera.

TSPS under `{TSPS_CHECKOUT}/client/` starts the camera with raw pitch 245 (`game/OsrsClient.ts:572,1124–1128`). `game/Camera.ts:154–157,460–465` converts that to 128 + 245 × 255/512. The height-dependent focal and perspective are in `game/Camera.ts:75,97–104,701–712,825–868`. The follow radius is in `render/render/camera2.ts:318–345,361–363` and `render/webgpu/camera.ts:248–280`. These are source-default calculations, not measurements of a connected game session.

## Twelve published files

Source scale means one stored image pixel per display pixel. Ratios below 1 indicate an asset smaller than the comparison client at the same target depth. Each ratio pair gives fixed / resizable. Sitting letters refer to 2026-11-30-a through -d. Source locations are relative to `{ROOT}/`. Each sampled review copy matched its source file byte for byte. Camera heights in scene snapshots are vertical offsets from the raised target; depth is hypot(distance, elevation). These snapshots use a 1000-pixel lens (`examples/authored_scene_snapshot.rs:28–33,77–82`). Arena scripts supply their own focal (`examples/asset_arena.rs:505–510`). The isolated actor harness uses 512 (`examples/support/actor_draw.rs:148`).

| Sitting / picture | px/unit / px/tile | Native scale | TSPS scale | Sampling, crop and encoding | Producer and line |
|---|---:|---:|---:|---|---|
| a / shield.png (880×1760) | 2.415 / 309.2 | 9.28× / 4.49× | 12.74× / 6.16× | 1280×900 → centred 440² crops. Four yaws. PNG; no resample. | `night/armoury2/boards/cal-return-blind/e0v0/scope.txt:3; tools/lineup.py:90–102` |
| c / pitch-a.png (1062×608) | 0.184 / 23.6 | 0.71× / 0.34× | 0.97× / 0.47× | 720×620 → 240×207, foreground matte then bounding crop. Lanczos; PNG. | `builds/boss-pitch-a/boards.py:24,47–60` |
| c / pitch-b.png (1320×648) | 0.216 / 27.7 | 0.83× / 0.40× | 1.14× / 0.55× | Foreground bounds plus 10 px margin → 47%, including mask. Lanczos; PNG. | `builds/boss-pitch-b/capture.py:19,46–57` |
| c / pitch-c.png (1412×788) | 0.224 / 28.7 | 0.86× / 0.42× | 1.18× / 0.57× | 666×630 → 222×210 after terrain-colour and plate mask. Lanczos; PNG. | `builds/boss-pitch-c/boards.py:19,31–40,52` |
| c / pitch-d.png (1262×634) | 0.253 / 32.4 | 0.97× / 0.47× | 1.33× / 0.65× | Connected-component matte, bounding crop, dimensions rounded to thirds. Lanczos; PNG. | `builds/boss-pitch-d/boards.py:19,64–86` |
| c / pitch-f.png (1200×704) | 0.194 / 24.8 | 0.75× / 0.36× | 1.02× / 0.49× | 590² → 200² following two-background matte. Lanczos; PNG. | `builds/boss-pitch-f/boards.py:27–38` |
| c / pitch-g.png (1668×868) | 0.213 / 27.3 | 0.82× / 0.40× | 1.13× / 0.54× | 780² → 260² after plate subtraction. Lanczos; PNG. | `builds/boss-pitch-g/boards.py:19–20,101–119` |
| c / pitch-h.png (1256×805) | 0.186 / 23.8 | 0.72× / 0.35× | 0.98× / 0.47× | 600² → 200² after matched plate subtraction. Lanczos; PNG. | `builds/boss-pitch-h/boards.py:78–100,111,144` |
| b / encounter-read.gif (1228×466) | 0.256 / 32.8 | 0.98× / 0.48× | 1.35× / 0.65× | 608×432 per view, title and gutter → 1228×466. Two GIF generations. | `render/fight-{real-tekton,arena-arena1}/encounter.log:1–4; sittings/board_encounter.py:8–22` |
| d / boss-gap.gif (600×326) | 0.239 / 30.6 | 0.92× / 0.44× | 1.26× / 0.61× | Direct 300² per actor. Auto-fit distance 2141; numbered strip → GIF. | `studies/research-creatures/lineup_npc.py:33–48,73–81` |
| d / watcher-gap.gif (600×326) | 0.328 / 42.0 | 1.26× / 0.61× | 1.73× / 0.84× | Direct 300² per actor. Auto-fit distance 1559; numbered strip → GIF. | `studies/research-creatures/lineup_npc.py:33–48,73–81` |
| d / hound-gap.gif (600×326) | 1.051 / 134.6 | 4.04× / 1.95× | 5.54× / 2.68× | Direct 300² per actor. Auto-fit distance 487; numbered strip → GIF. | `studies/research-creatures/lineup_npc.py:33–48,73–81` |

The pitch-board values use the nominal resize factor. Integer rounding in bounding-box resizes can change the result by a fraction of a pixel. Panel masks in A–H also change the background and can remove contact shadows or edge pixels. For a shape question, a cropped silhouette and a figure on the actual floor provide different evidence.

## What reaches the screen

The publisher copies images directly (`{ROOT}/sittings/make_sitting.py:40–43`). All four pages set body max-width 1500, body padding 24, section padding 18 with a 1 px border, and image width 100%. A click sets width auto (`{REVIEW_DIR}/sittings/2026-11-30-d/index.html:3–6`). Starting with one CSS pixel per display pixel, available widths of 765 and 1920 CSS px give image widths of 679 and 1462 px, respectively, before any scrollbar deduction. Browser zoom or UI scaling changes those numbers. The CSS leaves image-rendering at the browser default, so non-integer enlargement uses browser interpolation. Its precise kernel is implementation-dependent. No new raster detail is generated.

| Picture | Browser scale, 765 / 1920 | Display px/tile, 765 / 1920 | Compared with native fixed / resizable |
|---|---:|---:|---|
| shield.png | 0.77× / 1.66× | 238.6 / 513.7 | larger / larger |
| pitch-a.png | 0.64× / 1.38× | 15.1 / 32.5 | smaller / smaller |
| pitch-b.png | 0.51× / 1.11× | 14.2 / 30.7 | smaller / smaller |
| pitch-c.png | 0.48× / 1.04× | 13.8 / 29.8 | smaller / smaller |
| pitch-d.png | 0.54× / 1.16× | 17.4 / 37.5 | smaller / smaller |
| pitch-f.png | 0.57× / 1.22× | 14.0 / 30.2 | smaller / smaller |
| pitch-g.png | 0.41× / 0.88× | 11.1 / 23.9 | smaller / smaller |
| pitch-h.png | 0.54× / 1.16× | 12.9 / 27.7 | smaller / smaller |
| encounter-read.gif | 0.55× / 1.19× | 18.1 / 39.0 | smaller / smaller |
| boss-gap.gif | 1.13× / 2.44× | 34.6 / 74.6 | larger / larger |
| watcher-gap.gif | 1.13× / 2.44× | 47.6 / 102.4 | larger / larger |
| hound-gap.gif | 1.13× / 2.44× | 152.3 / 327.9 | larger / larger |

## GIF measurement at time zero

The lossless sources beside this audit were re-rendered from the saved camera, entries, seed and frame. For the three line-ups, re-encoding the lossless numbered strip as GIF reproduced every decoded pixel of the published first frame. For the fight, each panel was independently replayed at cycle zero using its logged stage and original cache. Counts exclude the 24-pixel in-panel caption and outer title/gutter. Each fight slice is 608×408. MAE averages absolute error over all three 8-bit channels and all pixels, including unchanged background.

| Image or panel | RGB colours before → first GIF → sitting | Pixels changed vs original in sitting | Mean / largest channel error |
|---|---|---:|---:|
| boss | 539 → 252 (one pass) | 1,308/195,600 (0.67%) | 0.018 / 42 |
| watcher | 420 → 253 (one pass) | 1,388/195,600 (0.71%) | 0.017 / 41 |
| hound | 568 → 252 (one pass) | 2,915/195,600 (1.49%) | 0.033 / 25 |
| real-tekton | 8,710 → 256 → 163 | 121,961/248,064 (49.17%) | 1.532 / 113 |
| arena-arena1 | 1,732 → 242 → 102 | 83,430/248,064 (33.63%) | 1.153 / 76 |

The real panel's first GIF already changes 96,930 pixels, the authored panel's 33,355. The sitting builder then decodes both GIFs, adds titles, and encodes the composite to another GIF (`sittings/board_encounter.py:8–22`). For future films, compose the RGB panels and captions first and encode that sequence directly to lossless WebP.

## Lighting and capture provenance

- **Shield and seven pitch boards.** The classic scene renderer loads textures and HSL palette at 0.8. Static objects use scene/object lighting. The reference shield has its cache definition. Authored objects use the ambient and contrast written by the builder (`tools/lineup.py:36,64,72`). The boards often use static object stand-ins for NPCs. The standard conversion copies model IDs into an object definition (`tools/scene.py:91–98`), so NPC-specific scale, recolour, pose and actor lighting cannot be assumed. Pitch H restores selected scale and lighting fields (`builds/boss-pitch-h/boards.py:41–47`). None of these pitch boards establishes how a live animated NPC is lit.
- **Three moving line-ups.** The NPC example lights once at rest with 64 + ambient, 850 + contrast and direction (−30,−50,−30), poses the mesh and applies definition scale afterwards (`examples/asset_npc.rs:372–375,401–407`). The actor painter uses palette/texture brightness 0.8 and world draw ordering (`examples/support/actor_draw.rs:72–98,146–149`). Its isolated camera uses the interface vertex transform; the native world scene also includes terrain, other actors and occlusion.
- **Fight.** Terrain, textures and actors go through the world renderer at brightness 0.8 (`examples/asset_arena.rs:514–516,731` and the corresponding `arena_view.rs` path). The snapshots are on the game's 20 ms cycle clock, sampled every 5 cycles. That is 10 fps. The creature line-ups sample their inherited average-frame-time schedules every 60 ms, at most 16.7 updates/s. Switching image format preserves those authored clocks.
- **TSPS.** Native device brightness defaults to 0.8 (`src/device_options.rs:23`). TSPS slider 50 defaults to gamma 0.75, and slider 40 gives 0.8 (`ui/ScreenBrightness.ts:13,25–27`). For an input channel of 0.25 the idealised mapping gives 84 vs 90 on the 0–255 scale. The demo confirmed both gamma values in the running client and measured worn gear at both settings (`{TSPS_WORK}/notes/demo/DEMO.md:461–498`). Matching the slider does not establish WebGPU/software pixel equivalence.
- **Capture conditions.** The TSPS driver starts an isolated headless Edge profile with WebGPU by default, window setting 1280×800, optional wheel and camera steps, and lossless CDP screenshots (`studies/research-interfaces/tsps/drive.cjs:11–16,33,96–103,207`). The demo includes fixed-yaw shots and zoom adjustments. Stretched mode is opt-in and changes logical sizing (`game/plugins/stretchedmode/StretchedModePlugin.ts:21–24,59–72`). The GPU plug-in offers FXAA and MSAA; its configured default is disabled on low-end hardware or 4× MSAA otherwise (`game/plugins/gpu/GpuConfig.ts:26–43`). These settings and shader paths are separate from software resolution scaling.
- **Other producers.** `tools/scene.py:160–170` converts a classic BMP to PNG, and `tools/vignette.py:35–37` uses that path. `tools/figure.py:29–45` delegates to the actor example and writes a numbered PNG; `examples/asset_figure.rs:30,53` uses brightness 0.8 and player lighting. `tools/board.py:7–18` resizes every input to the first panel's dimensions, which can stretch mismatched aspect ratios. `studies/research-effects2/fx.py:52–72` reads raw film BMPs; `fight/fight.py:161–192` chooses lens/camera, encodes GIF and downsizes stills; `studies/tools-fight-arena/play_arena.py:38–60` similarly encodes RGB frames and makes reduced contact sheets. Neither a screenshot of a board nor a GIF transcode can restore lost samples.

See [measurements.json](measurements.json) for raw comparisons and the page-copy inventory. The inventory records missing historical source paths as null. All twelve table entries have a matching source.
