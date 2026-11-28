# Build 951.1 asset study

**The local disk export stops in audio archive 40. It contains the model directory and none of the 146,939 model payloads.** The texture directories are present as well, with zero locally stored groups in 52–55. The strongest result available from this copy is at the definition level. The ordinary Eldritch crossbow and its Barrows variant share both inventory and worn geometry references, while the variant replaces five colours and one material. The project can rebuild that technique in 242 with original geometry and pixels. [Format and archive inventory](RS3.md) · [Rebuild briefs](TECHNIQUES.md)

## What was built

| Deliverable | Verified result |
|---|---|
| Disk reader and archive map | Read all 46 local reference tables. Walked every indexed sector chain and identified the precise physical cutoff. [Reader](rs3cache.py) · [Sector census](disk-audit.json) |
| Definition lookup | Indexed 248,207 item, NPC, object and spot records. The database stores direct names, model IDs, relevant properties and a stop reason with byte offset when decoding is partial. Checked every selected name and model list against the original definition bytes. [Lookup](definitions951.py) · [Catalogue](assets.sqlite) · [Per-family coverage](RS3.md) |
| Materials | Read all 25,407 records. Two common layouts yield 47,045 candidate texture references, all in the archive-52 directory. One further layout, material 23093, is retained as raw bytes. Property tails and flags remain inspectable. No diffuse PNG could be decoded. [Material records](materials.json) · [Flag counts](control-check.json) |
| Blender inspection backend | Reopened a synthetic scene with UVs, packed image, vertex RGBA, independent face transparency and vertex groups including two influences on one vertex. Rendered six views on the CPU. All six have foreground. Numeric RS3 model IDs report the missing group. Direct NXT import is unfinished. Blender Cycles supplies the lighting and rendering. [Importer backend](rs3blend.py) · [Viewer](rs3view.py) · [Fixture render](viewer-check.png) · [Editable fixture](viewer-check.blend) |
| Study | Selected ten modern boss/creature definitions, ten weapons, ten armour pieces and three spot definitions. The four sheets show the OSRS control geometry beside the 951 definition references. All 33 control thumbnails have foreground. RS3 mesh statistics and thumbnails are unavailable. [Tables](STUDY.md) · [Creatures](creatures.png) · [Weapons](weapons.png) · [Armour](armour.png) · [Effects](effects.png) |
| Technique transfer | Compared palette and material substitution, fitted armour parts, shared boss body assemblies, shared effect meshes, multiple image references and target rig limits. Each finding has a 242 feasibility judgement. [Technique assessment](TECHNIQUES.md) |

## Proof and recovery

The local archive catalogue identifies English build **951.1**, archive **2773**, November 23, 2026. It lists **25,353,455,332 bytes**, **49,838,978 blocks** and an invalid disk-store representation. The supplied dat2 is **8,724,151,800 bytes**, exactly 16,777,215 sector slots including reserved sector zero. Every nonreserved sector belongs to an indexed chain. Archive 40 group 48892 asks for sector 16,777,215 at chunk 117, one slot past the end. The three-byte JS5 next-sector field explains why this representation cannot store the catalogue's full block count. Reconstructing idx47 from sector headers would recover zero models. [Whole-disk audit](disk-audit.json)

Expanded size and CRC32 matched the master reference for **1,104** groups across the item, NPC, object, spot and material archives. For the missing archives, no group in 47 or 52–55 matched any resident group's expanded size and CRC32. The local TSPS source, server scripts, build trees and accessible local cache collections yielded no matching alternative 951 model bank or NXT reader. The local PDF describes a different machine's build-950 scripts. Two restricted E-drive project backup directories could not be inspected. [Verification](verification.json) · [Duplicate search](references.json) · [950 guide]({TEMP}/TSPS-RS3-Item-Porting-Guide.pdf)

The fallback search also decompressed **227,123** comparison models across OSRS 239 and HD 561, 601 and 614. **Zero** matched any 951 model's advertised expanded length and CRC32. This excludes raw-byte matches in the comparison banks. A mesh can have the same vertices and faces in a different binary encoding. **Zero pairs could be compared geometrically**, so the identical-geometry count and coordinate/winding differences remain unknown. [Cross-cache results](candidate-proof.json)

## Three proposed blind rebuilds

1. **Eldritch crossbow, item 47470**, for an inventory/worn pair and a five-colour, one-material finish variant.
2. **Vestments of havoc hood, item 53375**, for fitted part assembly, ten recolour rules and two independent material substitutions.
3. **Echo of Kerapac, NPC 28070**, for an alternate encounter appearance built from four shared body models with two colour and six material replacements.

These selections rely on direct definition evidence. The OSRS controls provide actual triangle budgets. The Leviathan control also provides a useful 242 rig check: 13 vertices have six nonzero bone weights in both 239 and 242. The local toolchain can represent multiple influences when the target rig and weight format are compatible. [Rebuild criteria](TECHNIQUES.md) · [Weight distributions](control-check.json)

## Running locally

Run from Git Bash. The SQLite query searches the existing catalogue. The fixture command regenerates both Blender outputs. The optional JSON interface accepts positions in source subunits, ordered triangles, linear RGBA per vertex, per-corner UVs, material IDs, face transparency bytes and per-vertex bone/weight-byte pairs. Images use the material ID as the PNG filename beside the JSON. The fixture script contains a complete working example. [Example input generator](blender_check.py)

```bash
P='{PYTHON}'
B='{ROOT}/studies/tools-rs3'
"$P" -I -B "$B/definitions951.py" --name 'Vestments of havoc hood'
"$P" -I -B "$B/verify951.py"
"$P" -I -B "$B/blender_check.py"
```

The data pipelines can be reproduced by running inventory.py, audit.py, references.py, definitions951.py --build, materials951.py, candidate_proof.py, study951.py, verify951.py and control_check.py in this build. Run blender_check.py before control_check.py when regenerating from scratch. Existing HD, OSRS and TSPS sources were read in place. New files were written under the requested build directory. No download, publication, commit or installation was performed.

## Failed commands and diagnostic probes

| Invocation or probe | Outcome and recovery |
|---|---|
| Initial shell override with pwd and ls | The command wrapper dispatched to cmd, which rejected pwd. Subsequent shell commands invoked the absolute Git Bash executable explicitly. |
| Initial ls of tools-rs3 | The destination did not yet exist. The file-writing tool created the dedicated build. |
| Whole-tree rg searches for RS3/NXT, 951 archive names and port scripts | The relevant TSPS code contains big-smart definition handling. Model.ts and preview.js matches such as faceColors3 were false positives. No build-951 mesh reader or matching alternative cache was found in the inspected locations. A broad search through dependencies took longer than the focused search. |
| rg against rs/config/obj, npc and loc paths | These paths were absent. The client uses objtype, npctype and loctype. The sources were located with a file listing. |
| Inline Python -c catalogue probe | Shell argument quoting left Python with an incomplete import. The file-based audit read the catalogue with the specified isolated Python. |
| Initial inventory.py group samples in 41, 42, 47–49, 52–62 and 65–67 | The first referenced groups had no local index entries. The final inventory records advertised counts and availability separately. |
| Initial audit.py | NumPy raised IndexError for sector 16,777,215. The audit now records the broken chain and continues through the remaining indexes. |
| Archive-name search using a slash/backslash regex, and searches of the E-drive backups | One rg expression had an unclosed character class. A subsequent filename-only search avoided the regex. The OS denied access to {EXTERNAL_DIR}/rust-client-project-only-20261105 and {EXTERNAL_DIR}/tsps-project-20261105. Other accessible cache and project locations were checked. |
| Initial references.py over numeric slots 0 through 67 | Reference 255/0 was absent. Enumerating actual idx files yielded all 46 local reference tables. |
| Probe for the working folder/AGENTS.md and the working folder/RUNBOOK.md | Neither root path existed. The requested build was isolated from existing tool files. |
| Initial definition and material field probes | Unsupported opcodes stop individual records. Successive width corrections improved item, object, NPC and spot coverage. A trial material layout read a third reference in 121 legacy records with flags 0x0b. Inspection showed a shared secondary slot for mask 0x0a. All final extracted IDs join the texture directory. |
| rs3view.py 122924 .../staff.png | The Fractured Staff inventory model is listed but absent. The viewer gave a missing-payload error. No staff image was generated. |
| Earlier verification runs | Pillow warned about getdata; the final verifier uses get_flattened_data. Blender 5.2 warned about the future deprecation of use_nodes during the successful render. |

## What would unblock the remaining work

For the first real import, the smallest missing input is **one verified build-951 archive-47 group plus the referenced texture payloads for that model**, supplied locally in flat-group or equivalent large-cache storage. The existing material definitions and reference CRCs can identify and check the inputs. The requested every-version/every-flag census and exhaustive geometry search require the complete 951 model bank. The diffuse decoder, property semantics, real RS3 six-view renders, geometry proof and RS3 half of the study remain unverified until those bytes are available.

