# Texture and transparency support for ordinary models

**45 of 45 classic-renderer views matched pixel for pixel. All 2,185 stored opaque, untextured model files re-exported byte-identically.** The original crystal lantern passed the scenery gate. Revision 239 has 210 textures. [Model format and decoder references](FORMAT.md) · [Texture contact sheet](board-textures.png) · [Lantern in the classic renderer](lantern-classic.png).

## What changed

All paths below are relative to `{ROOT}/`. Each changed pre-existing Python tool under `tools/` has its original beside it as `NAME.pre-textures.py`. The asset directory is outside Git. Work and evidence were isolated in `studies/tools-textures-alpha/`. The shared tools were reserved under `the working folder-tools`.

| File in tools/ | Change |
|---|---|
| osrscache.py | Reads face selectors, ordered mapping bases, mapping types, weight data and optional original packing. Rebases mapping and vertex indices when combining parts. The faithful combine follows first-use coordinate welding and first-label ownership. |
| real_dump.py, real_to_blender.py | Rich dumps keep face data, texture-space vertices, labels, weights and depth offsets. The Blender mesh keeps face order, corner order and loose basis vertices. Indexed mesh attributes carry the selector and source vertex identities. |
| blender_kit.py, blender_export.py, author.py | Materials accept texture and transparency independently. Ordinary authored meshes still use the welded exporter. Optional channels are omitted when unused. |
| cache_material.py, model_packing.py | New indexed export and model packing helpers. Rebuild geometry and material streams from the evaluated mesh. Original compression instructions are available for archived binary fixtures. |
| gate.py, compare.py, material_bands.json | Report per-value face counts and the measured population. Material use is informational. Existing geometric, colour, outline and worn checks still decide their own results. |

The effects kit already passes its JSON face alpha array to the model encoder at `studies/research-effects2/fxkit.py:58–61`. The ordinary route now accepts a material assignment directly. All work and captures were local. No commit or external publication was made.

## Authoring in Blender

Use `mat(h, s, l, flat=False, prio=None, depth=None, texture=None, alpha=0)`. The same argument order works in the per-face tuples passed to `add_mesh`. An author can also assign an ordinary material named `hsl_H_S_L[_flat][_pN][_dN][_tID][_aBYTE]`. Omit either optional material channel independently. ID 0 is a valid texture. A new triangle uses its three ordered cache corners as its texture basis. Imported nonlocal bases follow their referenced mesh vertices. Author alpha is the **raw transparency byte**. Start at zero for opaque, use an intermediate value for glass, and consult the format notes before using 254 or 255.

```python
frame = mat(6, 4, 40, texture=0)
pane = mat(30, 5, 66, flat=True, alpha=100)
collar = mat(6, 4, 34, texture=0, alpha=48)
```

For a real copy, run `real_dump.py model MODEL_ID mesh.json --faithful`, open the dump through `real_to_blender.py -- mesh.json mesh.blend`, then run `blender_export.py -- mesh.bin` on the saved file. Rich materials automatically select the faithful route. The flag selects it for an opaque control as well. `real_dump.py object OBJECT_ID ... --shape N` bakes the existing dump's object colour/scale convention and the definition's retextures. `worn ITEM_ID` selects the male worn mesh. For exact placement of an object carrying definition transforms, export its raw model and keep the original definition, as in the scenery proof. An integer model cannot store fractional scaled vertices. Export an indexed import in its own collection and keep its mapping vertex IDs unique. Reassigning a material or moving an existing basis vertex is supported. Arbitrary topology changes can invalidate a stored basis.

## Real round trips

The scenery harness used the model selected by each actual shape-0 object definition. It imported the decoded geometry into Blender, saved and reopened the .blend, then exported a new binary. The cache-reference scene used the real model ID. The comparison scene changed only the definition's selected model ID to point to the export. Both used rotation 1, identical placement, lighting, recolours, retextures and camera. Each cell below is the number of full-frame RGB pixels with any channel different, before resizing or labelling. Scene frames are 1280 × 900 at yaws 0°, +45°, −45°. The worn comparisons used the classic actor painter on a standard figure with the item's declared slot, at native yaws 192, 704 and 1728 of 2048. Columns were compared before annotation. All reference assets changed visible pixels against the omission control in each view. The stored thumbnails are for inspection. [Test harness](proof.py) · [Per-view counts and visibility controls](proof.json).

| Asset | ID | Name | Raw texture IDs | Nonzero raw alpha | View 1 Δpx | View 2 Δpx | View 3 Δpx |
|---|---:|---|---|---|---:|---:|---:|
| object | 13228 | Stained-glass window | 2 | 150 | 0 | 0 | 0 |
| object | 13229 | Decorative window | 2 | 112, 150 | 0 | 0 | 0 |
| object | 1734 | Odd looking wall | 2 | 144 | 0 | 0 | 0 |
| object | 10325 | Door | 2 | — | 0 | 0 | 0 |
| object | 11721 | Gate | 12 | — | 0 | 0 | 0 |
| object | 11388 | Amethyst crystals | — | 50 | 0 | 0 | 0 |
| object | 12045 | Magic door | — | 160, 180, 200 | 0 | 0 | 0 |
| object | 13219 | Stained-glass window | — | 150 | 0 | 0 | 0 |
| object | 10043 | Door | — | — | 0 | 0 | 0 |
| object | 10172 | Gate | — | — | 0 | 0 | 0 |
| object | 10335 | Rat wall | — | — | 0 | 0 | 0 |
| object | 10060 | Grand Exchange booth | — | — | 0 | 0 | 0 |
| worn | 6570 | Fire cape | 40 | — | 0 | 0 | 0 |
| worn | 25712 | Clan cloak | 12, 88 | 235, 240, 245, 250 | 0 | 0 | 0 |
| worn | 22388 | Xeric's guard | — | 50 | 0 | 0 | 0 |

Object-definition retextures remain active in those scene frames. For example, object 1734's raw mesh uses ID 2 and its definition remaps it to ID 23. The stained-glass and clan-cloak tests exercise both material streams on a model. The lantern collar exercises both on the same face.

For an independent check of the direct object import, the stained-glass window (13228) passed through `real_dump.py object` and gave **0** changed front-view pixels. Editing its Blender materials to remove just transparency changed **22,563** pixels. Removing just the texture changed **102,160**. These controls exercise the material assignment and the renderer's use of the exported bytes. [Edit-and-render check](sensitivity.py) · [Measured results](sensitivity.log).

## Regression scope and evidence

The final inventory found 3,852 pre-existing `.bin` paths under `builds/`, excluding this proof's output tree. Of these, **2,185** decoded models had no textured faces and all transparency bytes zero. Their bytes comprised **1,678 distinct inputs**, spanning 2,147 skinned-extended and 38 skinned-legacy file paths. For each distinct input, the regression constructed an indexed Blender mesh with materials, labels and vertices, exported through `blender_export.py --preserve-packing`, and compared the resulting bytes against **each** stored path. All comparisons passed. The packing option regenerates vertex deltas, face indices and face data while recording the original strip instructions, stream flags and auxiliary fields. This covers raw cache fixtures and postprocessed labelled models even where an original Blender source is unavailable. [Inventory by digest and path](inventory.json) · [Regression driver](binary_regression.py) · [Results including retries](binary-regression.jsonl).

The other paths comprised 1,322 models with texture or nonzero raw alpha, 157 registry-declared non-model payloads, 63 Rust incremental compiler files and 125 files that did not decode as valid nonempty models. [Rejected payload paths and reasons](non-models.json).

A second test opened all **773** stored Blender source paths (762 distinct files). With the default options and locally recoverable label/priority options, **821 successful before/after exports** had identical bytes. Those exports directly reproduced 724 stored binary paths. Five distinct Blender inputs were outside the old exporter's input convention, detailed below. This test runs the ordinary authored-mesh path, independently of original binary packing. [Blender source and option comparison](blend_regression.py) · [Per-source results](blend-regression.jsonl).

## Original crystal lantern

From Git Bash in `{ROOT}`, the following commands reproduce the example and its gate. The build script uses the installed Blender 5.2. The wrapper supplies `--python-exit-code 1` before both script invocations.

```bash
B={ROOT}/studies/tools-textures-alpha
P={PYTHON}
tools/blender_run.sh "$B/lantern.py" "$B/lantern.blend" "$B/lantern.bin"
"$P" -I -B tools/gate.py original-object "$B/lantern.bin" --faces 200,230 --size 42,52,68,76,42,52
"$P" -I -B "$B/lantern_view.py"
```

Four textured uprights enclose an amber light behind eight translucent panes. The octagonal foot, collar and cap use the same cache texture. It measures 46 × 72 × 46 game units. Of its 216 triangles, 176 use texture 0, 16 pane triangles have alpha 100, and 16 textured collar triangles have alpha 48. The build declares a 200–230 face range, width/depth 42–52 and height 68–76. [Source](lantern.py) · [Blender file](lantern.blend) · [Exported model](lantern.bin) · [Three classic views](lantern-classic.png).

```text
cache materials | {[quoted words withheld]: {"0": 176}, "faces by nonzero transparency": {"48": 16, "100": 16}, "population": "shape-10", "real": {"total": 25485, "textured": 1697, "alpha": 6722, "both": 193, "ids": [0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 12, 15, 16, 17, 18, 19, 20, 22, 23, 25, 26, 28, 29, 30, 31, 32, 33, 34, 35, 38, 40, 41, 42, 46, 49, 59, 60, 88, 89, 90, 95, 99, 105, 107, 111, 123, 124, 127, 128, 190, 194, 196, 198, 199, 200, 201, 202, 203, 204, 205, 209, 210, 65535], "values": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 22, 23, 24, 25, 26, 29, 30, 32, 33, 35, 37, 38, 39, 40, 44, 46, 48, 49, 50, 55, 56, 58, 59, 60, 62, 64, 65, 67, 68, 69, 70, 71, 73, 75, 77, 79, 80, 81, 85, 86, 87, 88, 89, 90, 92, 95, 96, 97, 98, 99, 100, 101, 102, 105, 106, 107, 108, 110, 112, 113, 115, 117, 119, 120, 122, 124, 125, 126, 127, 128, 129, 130, 131, 132, 134, 135, 138, 139, 140, 143, 144, 145, 146, 148, 149, 150, 155, 156, 159, 160, 161, 162, 163, 164, 168, 169, 170, 173, 174, 175, 176, 178, 179, 180, 182, 185, 186, 187, 188, 189, 190, 191, 192, 194, 195, 196, 198, 200, 201, 203, 204, 205, 208, 209, 210, 214, 215, 216, 217, 218, 220, 224, 225, 226, 227, 230, 231, 232, 234, 235, 238, 240, 242, 243, 244, 245, 247, 248, 249, 250, 251, 252, 253, 254, 255], "textured_percent": 6.66, "alpha_percent": 26.38, "both_percent": 0.76}} | informational | INFO
duplicate vertices | 0 | 0 | PASS
base height | 0 | 0 | PASS
faces | 216 | 200..230 inclusive | PASS
size x | 46 | 42..52 inclusive | PASS
size y | 72 | 68..76 inclusive | PASS
size z | 46 | 42..52 inclusive | PASS
all hue/saturation pairs | 5 | informational | INFO
main materials (at least 3% of surface each) | 5 | PASS <= 6, BORDERLINE <= 8 | PASS
zero-area triangles | 0 | 0 | PASS
flat face share | 0.1111111111111111 | < 1 | PASS
largest triangle surface share | 0.01764336616854112 | <= 0.25 | PASS
VERDICT PASS
```

## Revision-239 populations

Each row counts an archived object definition at its folder's placement shape, or an item definition with at least one male worn model at its archive slot. Models are decoded from the cache. Each definition counts once per category even if it has multiple parts or shares geometry with another definition. Texture IDs include definition retextures. Alpha use means any nonzero raw byte, including the sentinel bytes. “Both” means both streams have an active value somewhere on the model. Empty slot folders have no denominator. These are the full archived shape populations, broader than the filtered mounted-decoration samples in WALL-DECOR.md. Value ranges below are inclusive and cover consecutive observed values only. [Measurement script](survey.py) · [Machine-readable bands](../../tools/material_bands.json).

| Kind | Definitions | Textured % | Alpha % | Both % | Texture IDs | Nonzero alpha bytes |
|---|---:|---:|---:|---:|---|---|
| slot--1 | 265 | 7.55 | 7.17 | 0.38 | 9, 11, 15, 19, 22 | 50, 60, 70, 75, 128, 160, 170, 176, 200, 217, 224 |
| slot-0 | 1049 | 0.29 | 5.62 | 0 | 18, 40 | 1, 10, 32, 40, 50, 64, 69, 75, 80, 100, 112, 128, 133, 140, 144, 149, 150, 190, 201, 255 |
| slot-1 | 374 | 5.08 | 8.29 | 2.14 | 12, 18, 40, 59, 88 | 13, 26, 50, 68, 80, 90, 99, 100, 140, 180, 200, 235, 240, 245, 250 |
| slot-10 | 237 | 0 | 7.59 | 0 | — | 50, 100, 120, 140, 150 |
| slot-11 | 192 | 0 | 83.33 | 0 | — | 16, 32, 180, 200 |
| slot-12 | 0 | — | — | — | — | — |
| slot-13 | 0 | — | — | — | — | — |
| slot-2 | 225 | 0 | 14.22 | 0 | — | 20, 30, 40, 48, 60, 80, 90, 100, 128, 130, 150, 180, 196, 200, 220 |
| slot-3 | 1832 | 0.82 | 11.9 | 0 | 0, 12, 19, 31, 34, 40, 53, 88 | 1, 16, 25, 30, 40, 48, 50, 60, 70, 75, 80, 85, 90, 96, 100, 105, 112, 120, 128, 130, 140, 144, 150, 160, 170, 176, 180, 192, 200, 208, 210, 220, 224, 225, 230, 240, 250, 253–255 |
| slot-4 | 619 | 0 | 7.75 | 0 | — | 30, 100, 140, 150, 220, 230, 255 |
| slot-5 | 441 | 0.68 | 14.97 | 0 | 11, 22 | 50, 64, 75, 90, 96, 100, 112, 125, 128, 130, 144, 150, 153, 170, 196, 200, 208, 210, 235 |
| slot-7 | 570 | 0 | 7.89 | 0 | — | 4, 20, 30, 60, 100, 128, 140, 150, 254 |
| slot-9 | 219 | 0 | 2.74 | 0 | — | 130, 140, 150 |
| shape-0 | 2286 | 7.7 | 22.92 | 0.92 | 0, 2–5, 11, 12, 15–17, 22, 23, 35, 42 | 1, 10, 32, 40, 50, 60, 64, 66, 80, 90, 95, 96, 100, 110, 112, 120, 125, 128, 130, 140, 144, 150, 160, 164, 170, 176, 180, 190, 192, 200, 208, 220, 224, 225, 240, 254, 255 |
| shape-10 | 25485 | 6.66 | 26.38 | 0.76 | 0–5, 7–12, 15–20, 22, 23, 25, 26, 28–35, 38, 40–42, 46, 49, 59, 60, 88–90, 95, 99, 105, 107, 111, 123, 124, 127, 128, 190, 194, 196, 198–205, 209, 210, 65535 | 1–20, 22–26, 29, 30, 32, 33, 35, 37–40, 44, 46, 48–50, 55, 56, 58–60, 62, 64, 65, 67–71, 73, 75, 77, 79–81, 85–90, 92, 95–102, 105–108, 110, 112, 113, 115, 117, 119, 120, 122, 124–132, 134, 135, 138–140, 143–146, 148–150, 155, 156, 159–164, 168–170, 173–176, 178–180, 182, 185–192, 194–196, 198, 200, 201, 203–205, 208–210, 214–218, 220, 224–227, 230–232, 234, 235, 238, 240, 242–245, 247–255 |
| shape-22 | 1109 | 4.69 | 20.56 | 0 | 9, 11, 16, 17, 30, 35, 45 | 30, 32, 35, 40, 50, 55, 60, 64, 75, 80, 85, 96, 100, 110, 113, 120, 125, 128, 135, 140, 144, 150, 160, 174, 175, 180, 192, 200, 210, 215, 224, 227, 230, 240, 250, 254, 255 |
| shape-3 | 4 | 0 | 50 | 0 | — | 175 |
| shape-4 | 908 | 6.28 | 27.53 | 0 | 2, 11–14, 16, 19, 22, 39, 48 | 1, 8, 20, 26, 32, 41, 48, 50, 60, 64, 75, 76, 85, 96–98, 100, 112, 115, 120, 128, 130, 140, 144, 148, 150, 160, 175, 176, 180, 181, 192, 200, 204, 208, 219, 224, 226, 227, 229, 240, 245, 250, 253–255 |
| shape-9 | 24 | 4.17 | 37.5 | 0 | 48 | 16, 64, 75, 80, 96, 112, 125, 128, 144, 160, 175, 176, 192, 208, 220, 224, 240 |
| worn | 6023 | 1 | 11.66 | 0.15 | 0, 9, 11, 12, 15, 18, 19, 22, 31, 34, 40, 53, 59, 88 | 1, 4, 10, 13, 16, 20, 25, 26, 30, 32, 40, 48, 50, 60, 64, 68–70, 75, 80, 85, 90, 96, 99, 100, 105, 112, 120, 125, 128, 130, 133, 140, 144, 149, 150, 153, 160, 170, 176, 180, 190, 192, 196, 200, 201, 208, 210, 217, 220, 224, 225, 230, 235, 240, 245, 250, 253–255 |

## TSPS client, source inspection

**Custom models support cache textures and per-face transparency. One controlled-player path drops face alpha on textured faces.** Paths here are relative to `{TSPS_CHECKOUT}/client/`.

- `custom/items/CustomModelRegistry.ts:6–15,37–39` retains the model bytes. `rs/model/ModelLoader.ts:34–37` hands a custom registry hit to the same ModelData decoder and applies optional gear visuals. `custom/items/GearVisualRegistry.ts:105–117` returns ordinary unregistered visual data intact and supports optional texture overrides and animated alpha.
- `rs/model/ModelData.ts:555–569,794–818,874–882` dispatches FF FD, reads alpha, texture ID minus one, mapped-face selector minus one and the basis vertices. `rs/model/TextureMapper.ts:34–72` chooses the face or its mapping and filters against the client's SD texture loader. `rs/model/ModelData.ts:2892–2905,3094–3097` computes UVs, retains supported texture IDs and passes alpha into the lit model. Texture availability therefore also depends on TSPS's active texture loader.
- `render/buffer/SceneBuffer.ts:645–655,718–758,858–895,938–955` supplies opacity and atlas index to geometry and classifies transparent faces using both the face byte and the texture. `render/webgpu/actors/actorPlayers.ts:403–404` uses the common filter. `render/player/PlayerRenderer.ts:1675–1681` uses the common path for non-local players.
- In the controlled-player loop, `render/player/PlayerRenderer.ts:1655–1673` starts every face at opacity 255 and updates only when textureId is −1. An otherwise translucent face with a cache texture keeps full face opacity in this path. The smallest PR would export/reuse `faceTransparencyToAlpha` from `SceneBuffer.ts:864–867` for every face with a transparency stream, then run the existing transparent-face classification. Add a regression using the same textured, partially transparent worn face on the local and remote player, including sentinel handling. This finding is source-only, as requested.

## Failed commands, recovery and limits

- The first direct shell override for the source reads returned “The syntax of the command is incorrect.” The reads succeeded using the explicit Git Bash executable through the host shell. A later exploratory Python `-c` command lost its string quoting and raised NameError. A file-backed inspection script replaced it. A separate source-search command reported a Git Bash unmatched quote and was rerun with simpler search terms. Two diagnostic tool-call expressions had an extra closing parenthesis and failed to parse before execution.
- `git rev-parse --show-toplevel` reported no repository, and a later `ls .git` confirmed it. The initial `ls studies/tools-textures-alpha` found no directory. The task folder was created. `cuttle orchestrator resource status {ROOT}/tools` rejected the path as an invalid key (HTTP 400). The project showed no reservations. The folder-form resource key was claimed and read back at position one before editing.
- One `tail -4` diagnostic on multiple logs failed on its option syntax. A source search for two assumed Rust pose module filenames found neither file. Subsequent reads used the actual figure example and renderer modules.
- Early scenery proof runs failed on the omission control: an empty package, then an object-only package, produced [quoted words withheld] Mounting a real cache model ID as custom produced [quoted words withheld] The corrected harness uses a fresh custom ID. An off-camera-only control produced [quoted words withheld] A fixed banker in every scene supplied a visible comparison background. Repeated per-asset failures are retained in [proof.log](proof.log).
- The first Grand Exchange booth dump rejected its weighted vertices. The indexed encoder now carries weights and the same case passed all three views. The first binary regression also rejected unused complex mappings in research-effects/real/spot-2014/model-43213.bin. Preserving the mapping-type sections fixed that case; the final full pass included it.
- The first band publication raised KeyError on empty slot folders; the dependent gate log did not exist on that attempt. Zero-denominator rows now report no percentage. The first lantern lineup used camera distance 180 and failed the snapshot pose bounds check. The supplied render uses distance 330.
- The old exporter could not export five Blender sources in the before/after comparison. arena1-barrier-a and arena1-pier-a use materials named “HSL … alpha …”, arena1-pier-b uses a numeric material name, original-arm2-helm-shelf-a has no exportable mesh at the default entry point, and research-rig/probe has a face without a material. Their exact paths and exceptions are in the source-regression log. The stored binary regression was independent of those source conventions.
- The comparison covers static scenery and male worn figures at the listed poses. The TSPS assessment used source inspection only. The contact sheet shows the cache's source sprites at rest. No live TSPS runtime, female worn variant, or animated texture tick sequence was part of the pixel comparison.
