# Choosing a review render

| Review question | Preset | How to read it |
|---|---|---|
| Does the shape or paint read during ordinary play? | **Faithful 1×** | Native fixed-client default on flat ground, 33.30 px/tile at the target. Open at natural size and browser zoom 100%. On the sitting page, click once to toggle natural size. |
| Which existing pixels form this edge or colour patch? | **Magnified**, usually 2× or 3× | Each native pixel becomes a square of identical pixels, including on diagonals. Use for inspecting what survived rendering at game size. |
| Is this a modelling defect, a face overlap or a detail too small to sample at 1×? | **High-resolution**, 2×, 3× or 4× | Same pose, lens angle, camera distance and software paint. The renderer gets more pixels: 66.60, 99.90 or 133.20 px/tile. Judge the extra detail alongside the faithful view. |
| Do the animation, feet and silhouette hold together? | **Faithful 1×, lossless animated WebP** | Both columns share camera, ground reference and sequence frame. The seed fixes their positions. Use the high-resolution preset with WebP when the question concerns fine moving detail. |

High-resolution multiplies both the focal length in pixels and raster dimensions. Geometry, lighting, gamma, depth ordering, textures, sequence choice and time are the same. Edge coverage, four-pixel shade spans and texture sampling can change. A real resize can also change the client's orbit and FOV; a 2× render is a resolution scale of this camera. Keep all panels at the same presentation scale. At fractional OS display scaling, a CSS pixel may cover multiple physical pixels.

## Commands in Git Bash

Use a new output name for each run. The following commands use the installed binaries and local cache.

```bash
py={PYTHON}
base={ROOT}
q=$base/studies/tools-render-quality
model=$base/builds/boss-warden-general-3/boss-warden-general-3.json

"$py" -I "$q/quality.py" "$q/tekton-game.png" 7541 "$model" --seed 4111 --preset faithful
"$py" -I "$q/quality.py" "$q/tekton-pixels.png" 7541 "$model" --seed 4111 --preset magnified --factor 3
"$py" -I "$q/quality.py" "$q/tekton-detail.png" 7541 "$model" --seed 4111 --preset hires --factor 4 --pose attack:3
"$py" -I "$q/quality.py" "$q/tekton-motion.webp" 7541 "$model" --seed 4111 --preset faithful
```

For the other pairs use NPC 12812, `builds/creature-stone-watcher-3/creature-stone-watcher-3.json`, seed 4112; or NPC 11068, `builds/creature-star-hound-3/creature-star-hound-3.json`, seed 4113. Paths to the JSON should be absolute. Pose names are bind, stand:N, walk:N, attack:N and death:N where the real NPC has that sequence. The output suffix selects still PNG or the complete moving sequence.

For a room or object line-up, pass an existing scene package and a tile to inspect. Yaw here is in 16384ths of a turn. The capture uses the ordinary authored snapshot's classic terrain, cutaway, lighting and fixed simulation tick.

```bash
shield=$base/night/armoury2/boards/cal-return-blind/content.txt
"$py" -I "$q/scene_quality.py" "$shield" "$q/shield-game.png" --authored --tile 20.5 60.95703
"$py" -I "$q/scene_quality.py" "$shield" "$q/shield-detail.png" --authored --tile 20.5 60.95703 --preset hires --factor 3
```

For a fight stage, give `scene_quality.py` an `asset_arena` script, omit `--authored`, and pass the target world tile, yaw and desired `--cycle`. Use `--cache drive:/.../cache` for a cache containing staged assets. The tool replaces size, focal, camera and render directives; it preserves the actor and event lines. Each frame comes from the client example. The NPC-only tool uses a plain background and a common off-centre crop so tall actors fit. For questions involving occlusion or terrain, use the world scene.

A film author with existing RGB frames can use the shared encoder after captions and panel composition. A duration is in milliseconds. Pass RGB images directly from the renderer, before GIF encoding.

```python
import sys
from pathlib import Path
sys.path.insert(0, "{ROOT}/studies/tools-render-quality")
from quality import save_motion
count, duration_ms = save_motion(Path("{ROOT}/studies/tools-render-quality/fight.webp"),
                                 frames, [100] * len(frames))
```

To rebuild the two private adapters after a client rebuild, use the existing native release libraries. No download is required.

```bash
export PATH={CARGO_BIN}:$PATH
"$py" -I "$q/build_renderer.py"
```
