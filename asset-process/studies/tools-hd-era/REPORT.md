# RS2 HD assets in the classic tools

**The clearest technique to carry into 242 is material assignment on existing geometry.** The Sample table has the same 2,126 faces, four parts, palette size and alpha values in 614 and 239. The HD version textures 2,074 faces across 14 materials. The same-name Avatar of Destruction differs by one triangle between the two caches, yet HD textures 2,816 of its 2,831 faces. These are particularly good controls for a from-scratch rebuild. [Technique findings](TECHNIQUES.md) · [Paired figures and measurements](STUDY.md)

## What was built and verified

| Deliverable | Result |
|---|---|
| Cache and mesh reader | Read the reference tables from all three HD disks and both OSRS comparison disks. Decoded all **165,517** HD model records, including the 18-byte legacy layout, unversioned extended layout and explicit versions 14 and 15. No unrecognized model flag, connectivity opcode or unconsumed HD attachment tail was found. [Format notes](HDERA.md) · [Census](census.json) · [Reader](hdcache.py) · [Model translation](models.py) |
| Procedural materials | Evaluated **2,364** revision-specific texture records using the procedural generator in the local TSPS client. Read back every 128 × 128 RGBA PNG. Each archive includes the material and dependency table. [561 textures](textures-561.zip) · [601 textures](textures-601.zip) · [614 textures](textures-614.zip) |
| Classic viewer | Reads a selected model, object, NPC or inventory item and renders six separate angles with the local Rust client's lighting, face submission, priority ordering, blending and software rasterizer. The viewer supplies HD-generated texels and projected UVs where necessary. [Viewer](hdview.py) · [Renderer](classic.rs) · [Yk'Lagor, six views](yk-lagor-six.png) |
| Conversion | Writes a classic-format binary and reports the particular mesh's loss conditions and texture IDs requiring target materials. The converted r561 enriched snapdragon seed decodes identically to r242 model 53387 in the TSPS client model decoder for vertices, ordered faces and colours. [Converter](hdconvert.py) · [Converted example](seed-classic.bin) |
| Blender inspection | Reopened Yk'Lagor, 3,294 faces and ten packed textures, and Icy Bones, 1,409 faces and four packed textures. Verified face and vertex counts, priority and alpha arrays and packed image presence. Icy Bones exercises nonzero face alpha 25 and 75. Each file embeds the original mesh bytes, texture recipes, mapping and attachment data. [Importer](hdblend.py) · [Yk'Lagor](yk-lagor.blend) · [Icy Bones](icy-bones.blend) |

The HD caches place objects, NPCs, items, sequences and spots in separate numbered archives. OSRS 239 and 242 place those definitions in five groups of archive 2. Models stay in archive 7. HD uses a procedural material graph per texture group, and adds projected mappings, particle/billboard records and later subunit coordinates. The local 242 disk has 62,769 models and 216 sprite-backed texture definitions. [Detailed layout, archive counts, source lines and conversion losses](HDERA.md)

The UV cross-check covers all four mapping types. For six textured models with 12,170 faces total, the Python reader's geometry and mapping parameters produced the same per-corner UVs as a second decode of the original bytes by TSPS, within 0.00001. All reference UVs in these six meshes were finite. [UV results](uv-check.json)

## Pixel proof

Compared the ordered vertex coordinates, triangle indices and face colours first. For each match, the HD path then translated the mesh for the renderer. The control path independently read the original 239 model bytes and passed them directly to the Rust decoder. Both paths used the same camera, lighting, canvas and classic submission. The comparison used RGB pixels before sheet labels or image scaling.

**22 revision/model pairs × six views × 256 × 256 pixels = 8,650,752 pixel comparisons. Zero differed.** The images contained 1,028,355 foreground pixel observations. A few edge-on views can be empty. Every model pair had visible geometry. The 22 controls also had identical ordered vertices, faces and colours in the installed 242 cache. That last check was a geometry comparison. The pixel baseline was 239. [Per-angle counts, foreground counts and RGB hashes](pixel-proof.json) · [242 and sheet verification](verification.json)

| HD revision / model | 239 model | Identity in the HD definitions | Vertices | Faces | Differing pixels, six views |
|---|---:|---|---:|---:|---:|
| 561 / 44486 | 53744 | Barricade, objects 40317 and 40318 | 256 | 458 | 0 |
| 561 / 44579 | 53387 | Enriched snapdragon seed, item 14506 | 68 | 126 | 0 |
| 561 / 44595 | 53362 | Ruined dragon armour shard, item 14476 | 47 | 90 | 0 |
| 561 / 44607 | 53388 | Ruined dragon armour lump, item 14472 | 129 | 254 | 0 |
| 561 / 46657 | 50524 | Tumneken's shadow remains, item 14814 | 110 | 153 | 0 |

A whole-file fingerprint search found no identical decompressed model files between 561 and 239. Matching decoded content across model IDs recovered these examples. This proves the common geometry and classic draw path on untextured controls. It does not establish historical HD shader pixel parity. [Later control sheet](later-proof.png) · [Definition links](later-identities.json) · [Proof runner](proof.py)

## Study set

The eight contact sheets contain **46 paired examples**. All 92 model thumbnails have visible foreground. The tables give faces, vertices, part counts, texture IDs, textured-face counts, distinct nonzero alpha bytes, colour counts and available frame counts. [Full tables](STUDY.md) · [Additional compatible sequence families and frame counts](ANIMATIONS.md)

| Kind | HD coverage | Sheets |
|---|---|---|
| Bosses and large creatures | Ten definitions with combat level, footprint, body model IDs and measured base animations. Includes Nomad, Soul Wars and Dungeoneering examples. | [Creatures 1–8](sheets/npc-1.png) · [Creatures 9–10](sheets/npc-2.png) |
| Scenery | Six named, textured and translucent objects with same-ID, same-name 239 controls. Ranked by distinct material count, then textured-face count. | [Scenery](sheets/object-1.png) |
| Items | Twenty textured inventory models with same-name 239 controls. The inventory category includes construction previews as well as wearable equipment. | [Items 1–8](sheets/item-1.png) · [Items 9–16](sheets/item-2.png) · [Items 17–20](sheets/item-3.png) |
| Spot animations | Ten standalone spot definitions, with their actual model and sequence IDs, frame counts, and quantitatively similar 239 effects. Proposed encounter associations are labelled individually. | [Spots 1–8](sheets/spot-1.png) · [Spots 9–10](sheets/spot-2.png) |

**Encounter attribution remains unresolved.** The cache establishes the model and animation of each spot. It also establishes the face bindings for Yk'Lagor's two billboards and Flesh-Spoiler's three emitters. The selected bosses' stand/walk/idle frame bases and frame groups yielded zero direct joins to sequenced spots. The standalone spot sheets therefore report nearby model/sequence batches as candidates. Original server triggers or encounter captures would be needed to confirm ownership. The research-authorship study used server-side evidence for the same reason. [Batch distances](spot-study.json) · [Frame-base check](effects.py) · [Earlier authorship study](../research-authorship/AUTHORSHIP.md)

## Running the tools

Use Git Bash. The commands below write inside this build. Blender is launched in background mode by hdblend.py with the required Python exit-code behavior.

```bash
B='{ROOT}/studies/tools-hd-era'
P='{PYTHON}'
"$P" -I -B "$B/hdview.py" 11886 "$B/yk-lagor-six.png" --cache 614 --kind npc
"$P" -I -B "$B/hdview.py" 23526 "$B/sample-table.png" --cache 614 --kind object
"$P" -I -B "$B/hdview.py" 7535 "$B/diving-apparatus.png" --cache 614 --kind item
"$P" -I -B "$B/hdview.py" 46657 "$B/later-proof.png" --cache 561 --kind model
"$P" -I -B "$B/hdblend.py" 56819 "$B/icy-bones.blend" --cache 614
"$P" -I -B "$B/hdconvert.py" 44579 "$B/seed-classic.bin" --cache 561
"$P" -I -B "$B/proof.py"
"$P" -I -B "$B/verify.py"
```

The shipped renderer and bridge are ready to use. To rebuild both from local sources and regenerate the material archives:

```bash
B='{ROOT}/studies/tools-hd-era'
export PATH="{CARGO_BIN}:$PATH"
cargo +stable-x86_64-pc-windows-gnu build --offline --release --manifest-path "$B/Cargo.toml"
'{TSPS_CHECKOUT}/client/node_modules/@esbuild/win32-x64/esbuild.exe' "$B/client_bridge.ts" --bundle --platform=node --outfile="$B/client_bridge.cjs"
'{PYTHON}' -I -B "$B/bake.py"
```

The Rust harness links the existing client as a read-only dependency and uses this build's private Cargo target. The TypeScript bridge bundles trusted local client source. Cache records are parsed as data. No cache code was executed, no data was published and no commit was made.

## Scope of verification

The classic sheets show the bind pose with controlled lighting and independent framing. They omit terrain, scene lighting, runtime particles, animated scroll and HD shader passes. Blender preserves the source data and shows baked material colour and transparency. Its viewport uses Blender's lighting and draw order. The importer opens individual raw model IDs. The classic viewer resolves direct definition models, without simulating quest-state morphs or equipping worn variants. The technique judgements concern what 242 can encode or reconstruct. The converted seed was tested with its decoder. No authored asset was installed in a running 242 game. [Per-field transfer table](HDERA.md)

## Failed commands and recovery

The following includes failed command invocations, unsuccessful discovery probes, and intermediate failures that were corrected.

| Command or probe | Failure and resolution |
|---|---|
| Initial `pwd; ls …; command -v cuttle` with the shell override | The wrapper dispatched it to cmd and rejected pwd. Explicit Git Bash through PowerShell worked. |
| Explicit quoted Git Bash invocation through cmd | cmd rejected the escaped executable path. The short Windows executable path worked, followed by a PowerShell call operator with an absolute Git Bash path. |
| Initial listing of the requested build directory | The path did not exist yet. The file tool created the new isolated build there. |
| `git rev-parse --show-toplevel` under the working folder | The directory is not a Git repository. All output went into the requested new private build. |
| RUNBOOK grep with an apostrophe, then a later regex over definition error logs | The first command hit a PowerShell string terminator error. The later regex hit a Bash quote error. A simpler grep and an isolated Python file reader recovered the information. |
| Listings for client-build/target example paths, rust-client/target/release/deps, and a libgielinor glob | The guessed paths were absent. The existing examples were under rust-client/target/release/examples. The new harness built in the private target. |
| Listing research-npc/data and reading the user Cargo config | The census was under research-npc/census. The user-level config.toml was absent. The offline build resolved the local vendored dependencies. |
| Looking for esbuild in the TSPS root node_modules/.bin | The executable was under client/node_modules/@esbuild/win32-x64. |
| First offline Cargo build of hdclassic | Bounds expected floating-point positions. Corrected the literals, rebuilt and rendered the proof. |
| First standalone Python texture scan with -I | Its local module path was missing. Corrected the import. The exploratory evaluator was later replaced by the complete local client generator. Intermediate unsupported graph operations and projected mappings are absent from the final bake error lists. |
| Initial definition and item-selection scans | Incomplete field-width rules rejected records and an overly narrow control filter yielded zero items. Corrected the HD object shape lists, field widths, sequence prelude and item matching. Final field-extraction audits report zero failures in all three caches. |
| First full sheet rebuild against the client UV mapper | A diagnostic file called numbers.py shadowed the standard library module during a NumPy import. Renamed it, regenerated every sheet, and verified foreground in every cell. |

A first attempt to join effects by frame base used the version byte as part of the base ID and produced spurious candidates. Inspecting the frame headers identified the error. The corrected join returned zero and the earlier candidates were discarded. The final association table reports its independent proximity method explicitly.

