# Reference-sheet program

`reference_sheet.rs` is a client example that draws the reference archive the tools read (`REFERENCE_ARCHIVE`):
for each real object, NPC, item or body kit, a six-view sheet and a JSON file with its model ids, counts and
bounds. It reads a local revision-239 cache only: no network, no window, no cache writes.

```
reference_sheet.exe CACHE OUTPUT TYPE START_ID END_ID
```

- `CACHE`: the revision-239 cache folder (`OSRS_CACHE_239`).
- `OUTPUT`: the archive folder; created if missing. Objects go under `objects/shape-N/`, the others under
  `npcs/`, `items/`, `kits/`. It appends one line per id to `OUTPUT/events.jsonl` and keeps `progress.json`.
- `TYPE`: `objects`, `npcs`, `items` or `kits`.
- `START_ID`, `END_ID`: the id range, `END_ID` exclusive.

## Build

Copy `reference_sheet.rs` to `client/rust-client/examples/`, then from `client/rust-client/`:

```
cargo +stable-x86_64-pc-windows-gnu build --release --example reference_sheet -j4
```

The shipped executable (`client/bin/reference_sheet.exe` in the release zip) was built that way with rustc 1.99.1
(`x86_64-pc-windows-gnu`, LLVM 23.1.1), with `CARGO_TARGET_DIR` set to a separate build folder and
`CARGO_BUILD_JOBS=4`, against an earlier revision of the same client; the build finished with warnings only
(unused imports and lifetimes in vendored crates). Rebuilt against the client in this package with rustc 1.100, its
sheets for a sample range were byte-identical to the shipped copy's.

The archive it makes is drawn from the game's cache: keep it private. None of it is in this package.
