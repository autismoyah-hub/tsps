# Building the client and the study renderers

## Getting the executables

They are not in git. Download `client-bin.zip` from this branch's GitHub release named `asset-process-client-bin`
and unpack it at the package root; its entries are `client/bin/...`. Zip SHA-256:
`5681ee434603abaaeb01cfb98bdd2e73ebed1669ebd5d61630564e6e3eda49c4` (96079900 bytes). Or build them with the commands below and put them in `client/bin/`.
`asset_paths.py` looks there by default (`CLIENT_BIN`, `RENDERER_BIN`).

Toolchain: Rust stable `stable-x86_64-pc-windows-gnu` (rustc 1.100 was used), Windows x86-64. The crates the
client needs are vendored in `rust-client/vendor/` except the optional `windows` crate (only for the
`enhanced-renderer` feature), which cargo fetches once; do not pass `--offline` the first time.

## The client's examples (what `CLIENT_BIN` holds)

From `client/rust-client/`:

```
cargo +stable-x86_64-pc-windows-gnu build --release --example NAME
```

for each NAME of `arena_view asset_anim asset_arena asset_bone asset_effect asset_figure asset_npc asset_pose asset_skel
authored_scene_snapshot creature_light iface_dump interface_preview live_readback live_scene live_world meridian_ground
research_items research_pose`. The executables appear in `target/release/examples/`; point `CLIENT_BIN` there, or
copy them to `client/bin/`. A full build takes about a minute. `authored_scene_snapshot.rs` has code behind the
`custom-assets` and `enhanced-renderer` features; the feature set the shipped copy was built with was not recorded,
so a rebuild may differ in hash (the classic drawing is what the tools use). `live_scene`, `live_world` and
`live_readback` drive a running client against a local server (`studies/research-live/LIVE.md`).

Some examples are patched by study scripts before building: `studies/research-arenas/rust/patch_arena_view.py`
(makes `arena_view.rs`), `studies/research-effects2/rust/patch_arena.py` (makes `asset_arena.rs` from `asset_effect.rs`).

## The study renderers

| Executable | Build |
|---|---|
| `quality_npc.exe`, `quality_scene.exe` | `python -I studies/tools-render-quality/build_renderer.py` (after a release build of the client: it links the client's built libraries with `rustc` and patched copies of `asset_npc.rs` and `authored_scene_snapshot.rs`; output in the study folder) |
| `arena_film.exe` | `python -I studies/tools-real-fight/build_view.py` (rustc on `arena_film.rs` against the built client library) |
| `hdclassic.exe` | `cargo +stable-x86_64-pc-windows-gnu build --release --manifest-path studies/tools-hd-era/Cargo.toml` |
| `sheet-render.exe` | `cargo +stable-x86_64-pc-windows-gnu build --release --manifest-path studies/tools-hd-gate/Cargo.toml` |

Tools look for a study renderer in its study folder (or its `target/release/`) first, then in `CLIENT_BIN`.

## The reference-sheet program

`reference-sheet/reference_sheet.rs` draws the reference archive (`SETUP.md`, section 2). It is a client example:
copy it to `rust-client/examples/` and, from `client/rust-client/`, run

```
cargo +stable-x86_64-pc-windows-gnu build --release --example reference_sheet -j4
```

(the original build set `CARGO_TARGET_DIR` to a separate folder and `CARGO_BUILD_JOBS=4`; rustc 1.99.1 was used).
The executable is `target/release/examples/reference_sheet.exe`. Usage: `reference_sheet.exe CACHE OUTPUT TYPE
START_ID END_ID` (TYPE `objects`, `npcs`, `items` or `kits`; END_ID exclusive). Built against this package's client
source with rustc 1.100, its sheets for objects 1100 to 1102 were byte-identical to the shipped copy's (only the
timings in `events.jsonl` differ). See `reference-sheet/README.md`.

## The scene renderer

`bin/renderer/authored_scene_snapshot.exe` is the prebuilt renderer `tools/scene.py` runs (`RENDERER_BIN`). It was
built on another machine from an earlier revision of the same client (features `custom-assets`); that source
revision is not in this package. `examples/authored_scene_snapshot.rs` is its current source.

## What is in `bin/` (the release zip)

Sizes in bytes; SHA-256 of the file as shipped.

| File | Built from | Size | SHA-256 |
|---|---|---|---|
| `client/bin/arena_film.exe` | studies/tools-real-fight/arena_film.rs via build_view.py | 14524758 | `1d6fe94f038467631b49138f15d46dd61aa6b9cfea6395e547cb7c2a5e6d833a` |
| `client/bin/arena_view.exe` | client example examples/arena_view.rs | 10218553 | `5bccdb97efefcc1dac3878e72261d0a78664cb35f475e045a3f4a414ab69af5f` |
| `client/bin/asset_anim.exe` | client example examples/asset_anim.rs | 9789451 | `0c9ede9a4d2e581773c4d3edd2ccda04183b62824b8fa545fde8680949f330c9` |
| `client/bin/asset_arena.exe` | client example examples/asset_arena.rs | 10218570 | `c60f1ff7f4074f2033acf80d46a7300fd69a03ea8bdd435cf8b7f958ade21022` |
| `client/bin/asset_bone.exe` | client example examples/asset_bone.rs | 9731367 | `712b1d83832949e750694658af360fcc9c8caff96b35687a37df129fde2ed78e` |
| `client/bin/asset_effect.exe` | client example examples/asset_effect.rs | 10609915 | `292404e8729c9dd2ce0685eae76b1ee061b1db1068cfb2d3cd894ac3792fcfd7` |
| `client/bin/asset_figure.exe` | client example examples/asset_figure.rs | 9717684 | `b2dff37da92aa77616aa99baa0602e1832583985966c7d3c27d558e04845ace7` |
| `client/bin/asset_npc.exe` | client example examples/asset_npc.rs | 9940976 | `c36e6fd09d90f76d236f1160b5f2860db1feacf27bc9eca4318d9c8c99913aec` |
| `client/bin/asset_pose.exe` | client example examples/asset_pose.rs | 9808642 | `921875b51705e2947d46124877d331432688ec1cf16b9f24985a1ffa90544e20` |
| `client/bin/asset_skel.exe` | client example examples/asset_skel.rs | 10137335 | `33033bb2f7c5597901bcb85be41613a0fa8b7ac8a7fe8c98c771385f1b7a6642` |
| `client/bin/authored_scene_snapshot.exe` | client example examples/authored_scene_snapshot.rs | 10219320 | `af6f5843e01739dd50b62226a67a034ca620728c9d27862fff0d7d631de8733c` |
| `client/bin/creature_light.exe` | client example examples/creature_light.rs | 9642803 | `32ec3ab57f8e32f5f5bf5c1c6c2f349585cb609b221d3ad75cbe76f307f10b2b` |
| `client/bin/hdclassic.exe` | studies/tools-hd-era: cargo build --release (classic.rs) | 9485076 | `b9eb7ec7ac7ae0d24c1ceb681a8cee8a59a17ff099939db99e894b0f34efcecf` |
| `client/bin/iface_dump.exe` | client example examples/iface_dump.rs | 9889618 | `4dd08cea83f76d690364005a8c79c9de8837e5bcfc7d09c8629ec179cc7e0781` |
| `client/bin/interface_preview.exe` | client example examples/interface_preview.rs | 9681121 | `27a3cb2c8547abd2d2abb802d110c62b48e6edaefaa5bc1255161aa2c8829a98` |
| `client/bin/live_readback.exe` | client example examples/live_readback.rs | 10057109 | `80e832dbb32cd0c7915901390a8f09c7da4e97d98537fe4711ba35fd1def4b3e` |
| `client/bin/live_scene.exe` | client example examples/live_scene.rs | 10198152 | `b13f742ab3a3ed3f3b0652bf306608762e111c3c945d085bb1ca04d3d1b6f38c` |
| `client/bin/live_world.exe` | client example examples/live_world.rs | 10485638 | `50aee7e6c267548d4a476c9cc9210313be4bf3ad41eccd756a30f97c963893ca` |
| `client/bin/meridian_ground.exe` | client example examples/meridian_ground.rs | 9674972 | `1de19f861591a6be1c5596d4eb9d9a20c96e2f026a25f0177a49b9db31102a99` |
| `client/bin/quality_npc.exe` | studies/tools-render-quality/build_renderer.py (patched asset_npc.rs) | 13788168 | `abc19358a289b38fedefc10fca13cabb12d388d2613c242a930c6c24713fc6ff` |
| `client/bin/quality_scene.exe` | studies/tools-render-quality/build_renderer.py (patched authored_scene_snapshot.rs) | 14061898 | `14aac4eb730919c5c58f85c6ecffce39475606048b4f5ceb34f92d820ab5d53f` |
| `client/bin/reference_sheet.exe` | client example reference-sheet/reference_sheet.rs (prebuilt, rustc 1.99.1) | 9832034 | `3ae5e40c3c3ae1f2c4ccd77183202764a7bb95648e1ea8e6ab37147ba38df22d` |
| `client/bin/renderer/authored_scene_snapshot.exe` | prebuilt scene renderer used by tools/scene.py (RENDERER_BIN); built elsewhere, source commit not present here | 10241680 | `96c362af76ce77a26a5ae6f078ecef52810f0ee64d5c1e1e9d3561ce4abce0ec` |
| `client/bin/research_items.exe` | client example examples/research_items.rs | 9748749 | `838d6640bcf1e8aa9d613b38f5bced032488401eb513d5cf5d072ad121241e9b` |
| `client/bin/research_pose.exe` | client example examples/research_pose.rs | 9812430 | `04ab4706ab293b56e7fa9aaa0851ee3a2c86c84808788efc587a030f59eca829` |
| `client/bin/sheet-render.exe` | studies/tools-hd-gate: cargo build --release (sheet_render.rs) | 9491887 | `879bd4180ddd2af8558a8abffeee3cdb8d2766393831ed804e2b944a5519d5bc` |

5 of these (`arena_film.exe`, `quality_npc.exe`, `quality_scene.exe`, `hdclassic.exe`, `sheet-render.exe`) embedded the build machine's source
folder in panic messages; those strings were overwritten with a placeholder of the same length (`build\asset-process\.`
and `build\tsps-checkout___`). Nothing else in them was changed; the proof runs in `STAGING-NOTES.md` used these
patched copies.
