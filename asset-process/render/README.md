# Rendering

Tools write scratch renders under `render/` in the working folder (this folder; it must exist, `tools/gate.py`
makes its temporary folders here). Nothing rendered is kept in the package.

## The scene renderer (`RENDERER_BIN`)

`authored_scene_snapshot.exe CONTENT CACHE X Z YAW DISTANCE ELEVATION TARGET_UP classic auto NEW_OUTPUT_DIRECTORY`

X and Z are tiles; yaw is in 16384ths of a turn; the output directory must not exist; it writes `view.bmp`. A room
takes under a second. `classic` is the game's own look and the one to judge in (a `gpu` enhanced mode exists in the
client source but needs a GPU and is not used for judging). The prebuilt copy is
`client/bin/renderer/authored_scene_snapshot.exe`; `client/bin/authored_scene_snapshot.exe` is the same program
built from `client/rust-client/examples/authored_scene_snapshot.rs`.

## A scene

A folder: `content.txt` (paths and SHA-256 pins), `package/` (`registry.txt` listing models and object definitions
with CRC-32, and their files), `floor.bwr` (text: one `V x z height underlay walkable` row per vertex of a 128 by 128
grid, then `O object x z shape rotation` placements), `bindings.txt`. `tools/scene.py OUT BASE "id:x:z:rot,..." "X Z
YAW DIST ELEV UP" ...` builds one from a base scene (or `-`) plus real objects by id, and captures each camera.

## Known limits

- Between objects there is a depth buffer; inside one model faces are painted by priority and depth sort.
- A face at ground height loses to the terrain: floor pieces sit at height 1 to 3, as ground decoration (shape 22).

## Pictures for review

`studies/tools-render-quality/` (`PRESETS.md`, `AUDIT.md`): `quality.py OUT.png|OUT.webp REAL_NPC MODEL --seed N
--preset faithful|magnified|hires` draws a creature pair at the client's real resolution (faithful), integer
magnified, or at 2 to 4 times the pixels by the same rasteriser; animated WebP replaces GIF (GIF's 256 colours
change the look). `scene_quality.py` does the same for scenes. Sittings use the faithful preset first.
