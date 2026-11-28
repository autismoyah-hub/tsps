# Sitting render quality

**Nine of twelve published pictures have fewer source pixels per tile than the native fixed client at the camera target. Ten fall below the 1920×1080 resizable baseline.** Seven pitch boards shrink their captures to 23.6–32.4 px/tile. The fixed native baseline is 33.30, and the resizable baseline is 68.89. The strongest colour loss is in the encounter film: on the real side, 8,710 colours in the first scene frame become 163 in the published GIF. On the rebuilt side, 1,732 become 102. The page then scales every picture to the available width.

[Open the proof board](board-quality.png) at natural size. The 1350×3841 board shows Tekton 7541 and Warden-General III in the sitting's decoded standing frame, Faithful, Magnified 2× and High-resolution 2×/3×/4×. Every row includes a 1:1 detail crop of each actor. [Full audit, camera derivation and source lines](AUDIT.md) · [Author and Teacher commands](PRESETS.md)

## Published sample

At the target plane, a transverse tile is 128 model units. Values below apply at natural image size. The full audit includes px/unit, client ratios and computed displayed size at two page widths, including when browser enlargement hides a sampling deficit.

| Sitting picture | Source px/tile | Main intervention in the historical producer |
|---|---:|---|
| 2026-11-30-a / shield.png | 309.2 | 1280×900 → centred 440² crops. Four yaws. PNG; no resample. |
| 2026-11-30-c / pitch-a.png | 23.6 | 720×620 → 240×207, foreground matte then bounding crop. Lanczos; PNG. |
| 2026-11-30-c / pitch-b.png | 27.7 | Foreground bounds plus 10 px margin → 47%, including mask. Lanczos; PNG. |
| 2026-11-30-c / pitch-c.png | 28.7 | 666×630 → 222×210 after terrain-colour and plate mask. Lanczos; PNG. |
| 2026-11-30-c / pitch-d.png | 32.4 | Connected-component matte, bounding crop, dimensions rounded to thirds. Lanczos; PNG. |
| 2026-11-30-c / pitch-f.png | 24.8 | 590² → 200² following two-background matte. Lanczos; PNG. |
| 2026-11-30-c / pitch-g.png | 27.3 | 780² → 260² after plate subtraction. Lanczos; PNG. |
| 2026-11-30-c / pitch-h.png | 23.8 | 600² → 200² after matched plate subtraction. Lanczos; PNG. |
| 2026-11-30-b / encounter-read.gif | 32.8 | 608×432 per view, title and gutter → 1228×466. Two GIF generations. |
| 2026-11-30-d / boss-gap.gif | 30.6 | Direct 300² per actor. Auto-fit distance 2141; numbered strip → GIF. |
| 2026-11-30-d / watcher-gap.gif | 42.0 | Direct 300² per actor. Auto-fit distance 1559; numbered strip → GIF. |
| 2026-11-30-d / hound-gap.gif | 134.6 | Direct 300² per actor. Auto-fit distance 487; numbered strip → GIF. |

The moving boss was drawn 8% below fixed native scale. The hound was drawn at four times fixed native scale. Both sat in 300-pixel cells because the old camera auto-fits the reference creature. On a wide page those files expand by up to 2.44×. Presenter size, sampling density and asset size are separate quantities.

## Preset proofs

| Preset | Measured result |
|---|---|
| Faithful 1× | Direct unmodified `asset_arena.exe` with independently specified camera and the world preset both produced the same 512×334 Lumbridge scene, including Tekton. **0 of 171,008 pixels differed**; 4,856 distinct RGB colours. [Direct example](client-scene.png) · [Preset](faithful-scene.png). Both PNG SHA-256 values are `d2d33f5af5e38b581eb78daf3ee290e058ebede7dd207559b1216127f83f24cf`. |
| Magnified | The normal NPC CLI's 2× PNG compared with an independent integer repetition of the entire 1× PNG, including labels: **0 differing pixels**. |
| High-resolution | 2×, 3× and 4× use the same software triangle painter and camera-space geometry, with scaled lens and raster. At 2× on the two actors, the original `asset_npc.exe` supports exactly the required 512-pixel focal. Its independently rendered and cropped pair matched the new runner with **0 differing pixels**. The 2× rerender differed from nearest magnification at **51,264** pair-strip pixels. The authored scene CLI also rendered 1024×668, 1536×1002 and 2048×1336. |
| Motion | Every decoded WebP image was compared against the original RGB images over **every time interval**, with exact duration and infinite-loop checks. Edge 158.0.4370.45 displayed [the local img page](playback.html) at DPR 1. Two [first](edge-frame-0.png) and [later](edge-frame-1.png) screenshots matched distinct boss WebP frames, indices 5 and 16, pixel for pixel. |

The camera baseline for the presets is the native revision-239 software client with brightness 0.8. The comparison with TSPS uses its own source defaults. TSPS starts at a different pitch, uses gamma 0.75 at slider 50, and can apply GPU anti-aliasing. The pixel identity test covers the local native **example** at the same camera. No pixel identity with a logged-in TSPS WebGPU scene is claimed. The isolated actor view is a common crop on a plain background. The world and authored scene entry points include the environment.

## Delivered moving line-ups

I used Faithful 1× for the pending sitting so different species share an in-game scale. The seed order and 60 ms sampling clock come from the original line-up. During verification I moved the crop window to fit tall actors while holding the lens, target and orbit fixed. All three final files have zero foreground pixels touching their cell boundaries across the entire animation.

| File | Seed | Left / right | Decoded frames | Loop | Bytes |
|---|---:|---|---:|---:|---:|
| [Boss](../../sittings/phase1-hq/boss/lineup.webp) | 4111 | Tekton / Warden-General III | 87 | 7,440 ms | 961,928 |
| [Watcher](../../sittings/phase1-hq/watcher/lineup.webp) | 4112 | Minotaur / Stone Watcher III | 79 | 7,380 ms | 740,432 |
| [Hound](../../sittings/phase1-hq/hound/lineup.webp) | 4113 | Star Hound III / Lynx | 88 | 12,840 ms | 107,456 |

The original GIFs have the same respective durations. The hound has 90 stored GIF frames; at game scale some sampled poses decode to identical images, and WebP combines their holds. The boss WebP is about 5% larger than its GIF. Lossless colour and faithful scale were the delivery criteria.

## Files and regression checks

All authored work is confined to this new build directory and the three requested files in `sittings/phase1-hq/`. The asset workspace is outside Git. No commit or shared-tool edit was needed. The sitting spec and the published page were left in place.

| New files | Check |
|---|---|
| `quality.py`, `scene_quality.py` | NPC pose/motion, world arena, and authored-scene commands exercised. The normal boss PNG matches the delivered motion's first frame. The shared motion writer checks the decoded RGB timeline before publishing a file. |
| `build_renderer.py`, `quality_npc.exe`, `quality_scene.exe` | Built offline with the installed GNU toolchain and existing native release libraries. The NPC adapter changes only the isolated actor world's fixed focal parameter. The classic scene adapter parameterizes the raster and focal. Both build from temporary copies. At the historical shield camera, the new scene binary, unmodified client example and archived original produced **byte-identical BMPs**. SHA-256 `6a18e40ed1f87c2825d8789838f650287d39ed4aaff8ae2344307e903fa252ad`. |
| `proof.py`, `measurements.json`, `board-quality.png` | Re-rendered the control cameras; reproduced the three old first GIF frames, with zero differing decoded pixels after the same quantisation. Compared review copies by SHA-256. |
| `boss-before-gif.png`, `watcher-before-gif.png`, `hound-before-gif.png`, `real-tekton-before-gif.png`, `arena-arena1-before-gif.png` | Lossless frame-zero controls used for the audit. Fight samples exclude the caption rows and compare the same scene area before either GIF pass. |
| `client-scene.png`, `faithful-scene.png` | Independent scene camera specifications and identical PNG hashes, as above. |
| `edge-check.cjs`, `playback.html`, `edge-frame-0.png`, `edge-frame-1.png` | Local file playback in a private headless profile. Both screenshots match actual decoded frames. |
| `verify_delivery.py`, `delivery.json` | Original/new loop lengths, bytes, edge-touch counts, and screenshot matches. |
| `check_presets.py`, `preset-checks.json` | CLI replication proof, authored renderer identity, four raster sizes, RGB film-encoder round trip over a 300 ms test. |
| The three WebPs linked above | Each checked on decode before delivery. |
| `AUDIT.md`, `PRESETS.md`, `REPORT.md` | Audit of twelve distinct published files, source references, author commands and evidence inventory. |

To repeat the checks in Git Bash, define `py`, `base` and `q` as in PRESETS.md, then run:

```bash
"$py" -I "$q/proof.py"
"$py" -I "$q/check_presets.py"
node "$q/edge-check.cjs"
"$py" -I "$q/verify_delivery.py"
```

## Failed commands and recovered checks

- The first tool invocation with Git Bash selected as the shell routed `pwd` to the Windows command interpreter and returned 255. Subsequent commands explicitly launched `{GIT_BASH} -lc`.
- Initial `ls` probes of `studies/tools-render-quality` and `sittings/phase1-hq` found neither destination. `git rev-parse --show-toplevel` in the working folder reported that it was outside a repository. Work proceeded in the two new directories.
- `cuttle orchestrator resource status {ROOT}/{sittings/phase1-hq,studies/tools-render-quality}` as two absolute keys returned HTTP 400. The directory-name keys `phase1-hq` and `tools-render-quality` were valid; each was claimed and read back at position 1.
- Read-only source probes using nonexistent `examples/support/pose.rs`, `src/preferences.rs`, shell globs `client-build/*.{py,md}`, `builds/boss-pitch-{c,d,e,f,g,h,i}/capture.py`, and `target/release/deps/libgielinor*` produced missing-path diagnostics. The corresponding sources are `examples/asset_pose.rs`, `src/device_options.rs`, the pitch-specific board scripts, and `target/release/build/*/*/out/*.rlib`. Optional probe globs included absent shield `receipt*` and absent pitch-specific helper/source combinations. `which gcc` found no standalone GCC on the shell PATH; the installed Rust GNU toolchain linked both adapters.
- The inline `python.exe -I -c` dimensions probe lost quoting through the Windows shell wrapper and raised SyntaxError. `file` and the file-based Pillow checks supplied the dimensions. `tail -4` on both encounter logs produced an invalid-option diagnostic; subsequent reads used the individual logs and their explicit frame-count markers.
- The first `proof.py` inventory run stopped on the absent historic source `render/anim-hound-set-review/star-hound-set.gif`. Recovery used each actual published copy for its dimensions, with null for missing source comparisons. Four page entries across three distinct missing source paths are marked null. None belongs to the twelve-file sample.
- The first `check_presets.py` film fixture cropped a uniform region of the shield snapshot. WebP collapsed all three views to a still; the loop verification correctly failed. The revised fixture uses three distinct crops of the lossless encounter frame and passes.
- A first-pass boundary check found 90 foreground border pixels in the boss animation. Re-rendering from a taller raster with an offset crop retained the original projection. Maximum border counts on final readback are zero.

## Limits of the measurements

The client baselines are calculations at a specified target depth and flat-terrain default camera. Individual vertices sit at different depths. The browser table assumes a DPR-1 page with the stated content width. The actual display, zoom and saved graphics settings used by the owner were unavailable. The GIF colour measurements describe frame zero; the WebP fidelity and boundary checks cover the complete delivered loops. No live game login, publication step or external transfer was needed.
