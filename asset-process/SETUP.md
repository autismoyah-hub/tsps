# Setting up the asset process


Everything here was used on one Windows 11 machine. The tools are Python scripts plus a Rust game client that
draws what the game draws. Nothing in this folder comes from the game's caches: you supply the caches yourself,
and every picture, sheet, measurement file and model the process makes is made on your machine from them.

## 1. What you need

| Thing | Version used | Notes |
|---|---|---|
| Windows 10 or 11, x86-64 | Windows 11 | The client builds and runs on Windows (Win32 window, GNU toolchain). A GPU is not needed: the "classic" look is drawn in software. |
| Python | 3.13 | With `numpy`, `pillow`, `scipy`, `shapely`, `opencv-python-headless`, `trimesh` (and `pytest` for the tests): `python -m pip install numpy pillow scipy shapely opencv-python-headless trimesh pytest`. Run scripts with `python -I` (isolated mode; every script finds `asset_paths.py` itself). |
| Blender | 5.2 LTS | Authors build in Blender; `tools/blender_export.py` and `tools/real_to_blender.py` run inside it (`blender -b FILE.blend -P tools/blender_export.py -- OUT.bin`). |
| Rust | stable, `stable-x86_64-pc-windows-gnu` (rustc 1.100 used) | Only to rebuild the client and the study renderers from source; prebuilt copies are a release download (section 3). Install with rustup from rustup.rs. |
| Git for Windows | any | Its bash runs the few `.sh` helpers. |
| Node.js | any recent | Only for `studies/tools-hd-era` (HD texture bridge) and the browser checks of `studies/research-interfaces` and `studies/tools-render-quality`. |
| Microsoft Edge | any | Headless screenshots of review pages and the TSPS web client checks (`EDGE` environment variable may point at `msedge.exe`). |
| An agent runner | - | The prompts were run with a command-line agent runner (`cuttle sessions spawn --runner {RUNNER} --model ... --prompt-file FILE`). Any runner that starts an agent with a prompt file and lets it run shell commands on this machine will do; replace the command. |

## 2. The game data you must supply

None of this is in the package. Download it yourself and keep it private.

- **OSRS cache, revision 239**: the cache every tool draws from (`OSRS_CACHE_239`). From the OpenRS2 archive
  (archive.openrs2.org), the oldschool cache of revision 239; you need the disk form (`main_file_cache.dat2`,
  `main_file_cache.idx*`) and the XTEA keys only if you decode maps. The tools never write to it.
- **OSRS cache, revision 242**: the one the TSPS server ships (`OSRS_CACHE_242`, default
  `TSPS_CHECKOUT/server/caches/osrs-242_2026-11-03`), used by the interface and HD-era studies. Also on OpenRS2.
- **HD-era caches** (`HD_CACHES`), for `studies/tools-hd-era` and `studies/tools-hd-gate`: OpenRS2 runescape
  caches of revision 561 (OpenRS2 id 14), 601 (id 263) and 614 (id 266), each unpacked as
  `HD_CACHES/r561/cache/cache/main_file_cache.dat2` (and `r601`, `r614`).
- **RS3 cache, build 951** (`RS3_CACHE`, `RS3_FLAT`, `RS3_DIR`), for `studies/tools-rs3`: OpenRS2 id 2773. Use the
  **flat-file** export (one file per group) as `RS3_FLAT`. The disk (`.dat2`) export of 951 is truncated: the
  dat2 format cannot address sectors past its limit, so groups late in large archives are missing from it;
  `rs3flat.py` reads the flat form for that reason. `RS3_DIR` holds `caches.json` (the list `audit.py` reads).
- **A TSPS checkout** (`TSPS_CHECKOUT`): this repository's own `server/` and `client/` (definitions such as
  `server/data/definitions/sequence-names.json` and `item-gameplay.json`, boss plugins, the web client sources
  that `studies/tools-hd-era/client_bridge.ts` bundles). Run `npm install` in `client/` once for the Node checks.
- **The reference archive** (`REFERENCE_ARCHIVE`): for every real object, NPC, item and body kit, a six-view
  sheet and a JSON file (model ids, counts, bounds), in `objects/`, `npcs/`, `items/` (objects by placement
  shape, e.g. `objects/shape-10/`). **You make it yourself** from your revision-239 cache with the
  reference-sheet program (`client/reference-sheet/`, prebuilt as `client/bin/reference_sheet.exe`):

  ```
  reference_sheet.exe CACHE OUTPUT TYPE START_ID END_ID
  ```

  `CACHE` is `OSRS_CACHE_239`, `OUTPUT` the archive folder (`REFERENCE_ARCHIVE`), `TYPE` one of `objects`,
  `npcs`, `items`, `kits`, and ids run from `START_ID` up to but not including `END_ID`. Run it once per type
  over the whole id range (or in ranges; it appends to `OUTPUT/events.jsonl` and keeps `progress.json`). It reads
  the cache only and needs no network or window. No archive output is in this package.
  The package's own sheet makers make the held-out sheets a blind rebuild needs (`studies/research-creatures/sheet_npc.py`
  for creatures, `studies/research-bosses/sheet_bone.py`, `studies/tools-titan-sheets/`,
  `studies/tools-hd-gate/hdsheet.py`); `tools/contact.py`, `tools/pick.py`, `tools/measure.py`,
  `tools/reencode_item.py` and `studies/research-rooms/rlib.py` expect the archive's layout.
- **A base scene** (`E114_SCENE`): a scene folder (`content.txt`, `package/`, `floor.bwr`, `bindings.txt`; the format is in
  `render/README.md`) that `tools/scene.py`, `measure.py`, `roomprobe.py` and `seams.py` use as their default room.
  Build one with `tools/scene.py OUT - "id:x:z:rot,..."` (a `-` base means none) and point the key at it.

## 3. The client and the renderer

`client/rust-client/` is the Rust client source with every example the tools call (`asset_figure.rs`,
`asset_npc.rs`, `asset_pose.rs`, `asset_anim.rs`, `asset_bone.rs`, `asset_skel.rs`, `asset_effect.rs`,
`asset_arena.rs`, `arena_view.rs`, `authored_scene_snapshot.rs`, `iface_*.rs`, `live_*.rs` ...).

The executables are not in git. Get them one of two ways:

- **Download** `client-bin.zip` from this branch's GitHub release named `asset-process-client-bin` and unpack it
  at the package root (its entries are `client/bin/...`, so it lands in `client/bin/`). Check the zip's SHA-256
  and each file's against `client/BUILD.md`.
- **Build** them from source with the commands in `client/BUILD.md` and put them in `client/bin/` (or point
  `CLIENT_BIN` at cargo's `target/release/examples/`).

`client/bin/` is in `.gitignore`. `client/README.md` says what the client is.

## 4. Configuration

Copy `config.example.toml` to `config.toml` beside `asset_paths.py` and fill it in, or set environment variables
of the same names (they win). `python -I asset_paths.py` prints what every key resolves to.

| Key | Meaning | Default |
|---|---|---|
| `ROOT` | the working folder: tools, author pack and studies are found here; outputs (`builds/NAME`, `render/`, `measure/`) are written here | this package |
| `LEGACY_ROOT` | optional: an older working folder in the original layout (`builds/research-*`, `night/`); a path missing under `ROOT` is read from there | none |
| `OSRS_CACHE_239` | revision-239 cache folder | none (required) |
| `OSRS_CACHE_242` | revision-242 cache folder | `TSPS_CHECKOUT/server/caches/osrs-242_2026-11-03` |
| `TSPS_CHECKOUT` | TSPS checkout | none (required for definitions, interfaces, HD bridge) |
| `RENDERER_BIN` | `authored_scene_snapshot.exe`, the scene renderer `tools/scene.py` runs | `client/bin/renderer/authored_scene_snapshot.exe` |
| `CLIENT_SRC` | the client source | `client/rust-client` |
| `CLIENT_BIN` | folder of the client's built examples, the study renderers and `reference_sheet.exe` (unpacked from the release zip, or built) | `client/bin` |
| `NATIVE_ASSETS` | the client's Python model codec (`native_codec.py`) | `CLIENT_SRC/tools/native-assets` |
| `BLENDER` | `blender.exe` | `blender` on PATH |
| `PYTHON` | the Python above | the interpreter running the tool |
| `HD_CACHES`, `RS3_DIR`, `RS3_CACHE`, `RS3_FLAT` | the HD and RS3 caches | none |
| `REFERENCE_ARCHIVE`, `E114_SCENE` | the reference archive and the base scene | none |
| `REVIEW_DIR` | where sitting pages are written | `ROOT/review` |
| `VERDICTS_DIR` | where collected sitting answers are filed | `ROOT/verdicts` |
| `FONT_DIR` | fonts for boards (Segoe UI, Arial, Consolas) | `%WINDIR%/Fonts` |
| `NODE`, `POWERSHELL`, `CARGO_BIN` | executables | `node`, `powershell`, `~/.cargo/bin` |

Shell helpers read the same keys: each staged `.sh` starts with `eval "$("${PYTHON:-python}" -I .../asset_paths.py --sh)"`.
A key that is not set expands to `UNSET-KEY`, so a tool that needs it stops with a file-not-found naming the key.

How paths work in the scripts: every machine path was replaced by a placeholder expanded at run time, e.g.
`_AP.fix(r'<OSRS_CACHE_239>')` or `_AP.fix('<ROOT>/studies/research-npc')`. Executables are found with
`_AP.exe('quality_npc.exe', HERE)` (the study folder first, then `CLIENT_BIN`). Prompts and documents use the same
names in braces: `{ROOT}`, `{PYTHON}`, `{BLENDER}`, `{TSPS_CHECKOUT}`, `{BUILD}` (the job's build folder,
`{ROOT}/builds/NAME`). Other brace names in documents only describe where something was on the original machine
(`{PROJECT_REPO}`: the private project repository holding the Teacher's record; `{TSPS_WORK}`: the folder holding
the TSPS checkout; `{EXTERNAL_DIR}`, `{ADAPTER_DIR}`, `{ASSET_ADAPTER_DIR}`, `{WORK_DIR}`, `{TEMP}`: other
private folders) and need no value.

## 5. Data the tools make for themselves

Large files measured from a cache are not shipped. Regenerate them on your machine before the tool that reads them:

| File | Read by | Make it with |
|---|---|---|
| `studies/research-npc/census/*.jsonl`, `by-skeleton.json` | `gapcheck.py`, `npc.py`, `census.py`, `research-creatures/find.py` | `client/bin/asset_npc.exe CACHE census studies/research-npc/census/`, then `census.py` |
| `studies/research-authorship/data/packages.json` | `packagecheck.py` | `studies/research-authorship/pkg_measure.py` |
| `author-pack/labels/body-kits.json` | `tools/pose.py`, `tools/figure.py`, `label_armour.py` | `studies/tools-gaps/make_body_kits.py` |
| `examples/barrel/barrel.bin` | `tools/lineup.py` (the model it stands in a line-up of real pieces only) | `python -I examples/barrel/build.py` |
| `measure/decor-real.json`, `measure/first-measure.json`, `measure/weapon-card.json` | `decor_bands.py`, `measure.py`, `weapon_card.py` | those tools |
| `studies/research-fights/TABLE.md`, `studies/research-arenas/data/tables.md` | people (reference tables) | `analyze.py`, `table.py` in those studies |
| `data/merged/` caches in creature studies | `cmesh.py` users | made on first use |
| reference sheets for a rebuild | authors | the sheet makers listed in section 2 |

Kept in the package because a checker cannot run without them, and small: `tools/decor_bands.json`,
`tools/material_bands.json` (gate band limits), `studies/research-fights/metrics.json`,
`studies/research-gap/data/stats.json`.

## 6. Check the setup

```
python -I asset_paths.py
python -I tools/gate.py rebuild-object 1100 builds/SOME/real-reencoded.bin --ambient 0 --contrast 0
python -I studies/research-fights/fightcheck.py YOUR-FIGHT.json   (format in fightcheck.py's docstring)
python -I studies/tools-hd-era/hdcache.py      (lists every HD cache's archives)
```

A real model re-encoded through our encoder must PASS its own gate (`tools/real_dump.py`, `tools/real_to_blender.py`,
`tools/blender_export.py`): that is the ceiling check, and the first thing to run on a new machine.
