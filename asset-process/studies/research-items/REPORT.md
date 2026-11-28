# Meridian equipment investigation · revision 239

All eight originals now have an inventory/drop model, fitted icon view, female worn model, encoded item definition, client-decoded readback and comparison board. The barbute also has separate male and female dialogue models. **The female legs need an authoring pass at the rear waist in the run.** The largest two connected patches of draw-order disagreement there contain 57 and 26 pixels. The test compares two renderings of the same pose and isolates the affected plates. [Enlarged actor/depth comparison](completed/legs/run-layering.png)

The [item authoring guide](ITEMS.md) contains the cache measurements, definitions of the mesh comparisons and slot-by-slot rules. The [completed directory](completed/) contains the binary assets and evidence. Inside each item folder, models.json maps model IDs to actual files, including the original male mesh. spec.json is the encoder input, item.def is the cache-format payload, decoded.json is client readback, and inventory.view.json records the eight neighbours, camera and fit. All deliverables are local.

## What survived the interruption

| Found on takeover | Verification in this investigation |
|---|---|
| research_items.rs and its release executable, plus itemcodec.py and itemlib.py | Ran the cache exporter, icon comparator and both definition decoder entry points. Re-exported 33,746 definitions. The JSONL SHA-256 matched the inherited export. |
| measure_items.py, measure_items2.py, measure_fit.py, q1.py–q5.py, cached models, measurement tables, kit and seam surveys | Reran both large surveys from an empty temporary data directory, fetching models from the cache. The resulting measurement JSON and family lists matched the inherited files by SHA-256. Reran the five follow-up programs. |
| research_pose.rs, fpose.py, female_warp.py, the fitted landmark archive and several early female renders | Reproduced the original male renderer across 450,000 pixels. Tested female real-item assembly against independently prepared file inputs in seven controls. Ran the warp's hold-out check and applied the warp to all eight originals. |
| make_inventory.py | Ran its pose and neighbour-fitting code on all eight. The first drops exposed missing grip segments and the face inside the empty helm. The finished inventory path handles those cases and tests signed offsets with the actual renderer. |
| capes/CAPES.md, two cape programs, measurements, a red cape re-encode and render sheets | Reran the three-cape measurement and the red-cape equivalence test. The earlier 16-sheet cape layering sweep is inherited evidence. |
| The fpose docstring referred to an absent chathead.py. The eight completed packages, boards and item report were missing. | Built the original-item packages, dialogue head builder, ground renderer, repair and inspection tools, and both reports. |

Inherited file paths, sizes and hashes are in the [source inventory](completed/inherited-files.txt). Fresh survey output includes the filter counts, fitted distributions, sample sizes and the three hash comparisons. [Cache-wide results](completed/measure_items.txt) · [Family results](completed/measure_items2.txt) · [Follow-up and cape results](completed/assembly.txt)

## Boards and measured pixels

The fifth icon and middle ground item are Meridian. The four figure columns show male stand, male walk, female stand and female walk. Icon pixels include the outline and shadow in a 36×32 cell. Ground pixels changed when the middle item was added to the flat-tile render. The four worn counts compare each equipped column to its bare control at the identical pose, before captions. They include body geometry hidden by equipping the item.

| Board | Item ID | Icon px | Icon x / y, inclusive | Ground px | Male stand / walk | Female stand / walk | Definition bytes |
|---|---:|---:|---|---:|---:|---:|---:|
| [Longsword](../../render/research-items-meridian/longsword.png) | 60000 | 253 | 3–28 / 2–30 | 1,461 | 828 / 873 | 795 / 894 | 97 |
| [Glaive](../../render/research-items-meridian/glaive.png) | 60001 | 144 | 2–29 / 1–28 | 2,250 | 747 / 806 | 793 / 818 | 93 |
| [Staff](../../render/research-items-meridian/staff.png) | 60002 | 251 | 3–28 / 2–31 | 4,931 | 1,131 / 1,207 | 1,108 / 1,140 | 90 |
| [Cuirass v2](../../render/research-items-meridian/cuirass.png) | 60003 | 674 | 0–31 / 4–27 | 5,550 | 3,196 / 3,483 | 2,397 / 2,637 | 90 |
| [Gauntlets](../../render/research-items-meridian/gauntlets.png) | 60004 | 262 | 2–31 / 4–27 | 1,363 | 679 / 635 | 582 / 541 | 93 |
| [Legs](../../render/research-items-meridian/legs.png) | 60005 | 287 | 10–22 / 2–29 | 5,082 | 3,831 / 3,024 | 3,596 / 3,200 | 82 |
| [Boots](../../render/research-items-meridian/boots.png) | 60006 | 531 | 3–28 / 3–29 | 2,182 | 1,717 / 1,612 | 1,375 / 1,305 | 89 |
| [Barbute B](../../render/research-items-meridian/barbute.png) | 60007 | 566 | 4–28 / 3–30 | 2,368 | 887 / 869 | 905 / 909 | 103 |

The final readback resolved every model in each definition, compared every decoded field and compared all six image regions on each board to their source renders. All 48 region comparisons returned zero different pixels. The original male filenames and SHA-256 values are in each build.txt. The eight definitions use local item IDs 60000–60007 and model groups beginning at 1000100, separate from the server copy's katana at 1000000–1000001. I chose preview IDs, examine text, default economics and Wield/Wear actions for the asset test. Gameplay balance and production ID allocation still belong to the server owner. [Final readback](completed/delivery.txt) · [Machine-readable counts](completed/counts.json)

## What the comparisons establish

**Inventory icons.** For real items the tool calls Icons::get. For a supplied file it decodes the same model format, applies scale and materials, then uses the client's light, icon vertex projection, depth bounds, triangle rasterizer and sprite outline/shadow. verify compared packed pixels before background composition. With quantity 1, outline 1, shadow 0x302020 and standard size, the Rune platelegs, Rune platebody, Rune med helm, Rune longsword, Staff of air, Dragon halberd, Rune boots, Fire cape, Barrows gloves and Attack cape had 330, 626, 569, 235, 193, 187, 580, 494, 459 and 394 drawn pixels respectively. **0 of 11,520 pixels differed, including 4,067 drawn pixels.** The custom-file replica test covers these plain records. Real note, bought and placeholder requests go through the actual client function. The transparent PNG wrapper was independently compared to the board raster for a real and a custom longsword. Both comparisons returned 0/1,152. [Verification transcript](completed/verification.txt) · [PNG readback](completed/delivery.txt)

I also extracted the ten actual model files, re-encoded their item definitions and fed them through the same file-input interface used for Meridian. On each of two backgrounds all ten matched Icons::get, including the textured Fire cape. This exercises file decoding, field encoding, materials and transparency together. [File-input commands and pixel comparisons](completed/file-icons.txt)

**Definitions.** Encoding and decoding all 33,746 real records reproduced all 34 compared fields, totalling 1,147,364 equal comparisons. Exactly 14,324 payloads also matched the original bytes. Opcode order, redundant defaults and discarded fields can change while the decoded object stays equal. For each original, itemcodec wrote the wide model-ID forms and both Definition::decode_local and Definition::decode returned the requested fields. This is semantic equality for the client schema. A cache record's ignored bytes are outside that guarantee. [Cache round trip](completed/verification.txt) · [Example with both head models](completed/barbute/build.txt)

**Female figure.** For real items the renderer uses Appearance::standard with gender 1 and the client's Appearance::mesh. The file-input path substitutes a model at its actual slot in the same assembly order and applies player colours. I exported female source parts with their item colours and offsets baked in, rendered the real definition and the file side by side, and compared the unannotated cells. Rune longsword, Staff of air, Rune platelegs, Rune med helm, Rune boots, Barrows gloves and Rune platebody each gave 0/50,000 differences at bind, walk frame 1 and walk frame 4. **0 of 1,050,000 pixels differed.** This includes the case in which the body item supplies the sleeves. The assembly includes the cache-standard goatee on both sexes. [Exact commands, model IDs, bounds and pixels](completed/pose-controls.txt)

**Female derivation.** The weapon files carry the measured modal rigid displacement (+4,−5,+1) in x, height, z, with definition offsets zero. Armour takes displacement from nearby real same-limb male/female landmarks and snaps authored seam coordinates and labels to the target rings. With each tested donor excluded by item and male model IDs, the median held-out vertex errors in cache units were 1.73 head, 3.16 body, 2.00 legs, 1.41 hands and 1.00 feet. The body check had only four distinct same-topology sources, and its worst tested vertex missed by 28.1 units. These figures measure the landmark estimate on compatible topology. They cannot validate a sculpted breastplate by themselves. [Fit results and sample sizes](completed/verification.txt)

Integer rounding in the first pass left 8, 2, 6, 1 and 1 previously nondegenerate triangles with non-positive rest-normal dot product on head, body, hands, legs and feet respectively. The local repair moved 5, 2, 5, 1 and 1 vertices by at most one unit per axis, excluding every measured seam point. All five final derivatives have zero newly collapsed faces, zero non-positive rest-normal comparisons and zero duplicate positions. The uncorrected derivative is retained as female-first-pass.bin. Each build log records the exact edited vertex indices.

**Fit in motion.** The final full female outfit shares four collar, eight wrist, four waist and eight ankle rest coordinates. The male set shares 4, 8, 2 and 8. The female collar and waist use the different labels found in the real female slots. Full-outfit walk, run and slash sheets include frames 1, 3 and 5 with a real outfit in the first column. For individual pieces, geometry-motion.json contains per-frame bare-control differences and enclosed-background counts. An enclosed background region can be the space between arm and torso or the legs. It requires visual inspection before calling it a hole. [Seam locations and labels](completed/assembly.txt) · [Walk](completed/set-1-walk.png) · [Run](completed/set-1-run.png) · [Slash](completed/set-1-slash.png)

**Ground.** The new example loads real items with Models::inventory and accepts a local definition plus file using the same inventory preparation. It uses the ground-pile lighting from world_actors.rs and that source's model submission geometry, texture mapping, depth bias and alpha, drawn through Frame::draw. The test layout places three items on adjacent level tiles with the Meridian file between two real cache items. The tile material and camera are harness choices. A live world drop, stacked-pile selection, slopes, bridges and raised supports were outside the render test. [Ground example](../../client/rust-client/examples/meridian_ground.rs) · [Longsword ground command and output](completed/longsword/ground.txt)

**Dialogue.** The barbute keeps the actual jaw. Its 356 shell triangles scale fourfold onto a stock 286-triangle dialogue face in place of the 121 embedded worn-face triangles. The male/female translations in cache coordinates were (0,711,24) and (0,697,10). The dialogue face retains its expression rig, and the shell takes rigid group 15. Both appeared beside a real Rune med helm in the widget pass, in bind and two frames of the server's default expression 591. The test covers the selected jaw kit and expression. [Male dialogue](completed/barbute/dialogue-0.png) · [Female dialogue](completed/barbute/dialogue-1.png) · [Build log](completed/barbute/build.txt)

## Faults and disposition

For each piece I compared the rear-quarter actor draw with a true-depth draw at walk frame 1, run frame 3 and slash frame 3. Counts below cover the whole figure. The real reference is from the same slot and identical pose and camera. A disagreement measures draw order. Whether it produces a visible fault depends on where it lands. The front-quarter and rear-quarter images are in each item's female-motion.png. [All counts, including bare controls](completed/layering.json)

| Piece | Meridian walk / run / slash | Real walk / run / slash | Decision from these renders |
|---|---|---|---|
| Longsword | 26 / 46 / 22 | 21 / 40 / 31 | The grip tracks the hand and the blade stays rigid. Eight added wood triangles close the authored fist gap in the drop. Some close-range attack crops cut off the blade tip. The boards show the complete standing and walking weapon. |
| Glaive | 20 / 20 / 21 | 10 / 16 / 11 | Its second eight-triangle bridge closes the shaft on the ground. The blade is slimmer in the icon than the halberd controls. The held shaft reads continuously through the sampled poses. Two-hand contact in every attack remains unverified. |
| Staff | 25 / 28 / 22 | 15 / 14 / 10 | The ring stays attached in both locomotion sequences and the slash stress pose. The ring and stone increase icon coverage relative to Staff of air. There is no casting or off-hand test. |
| Cuirass | 18 / 23 / 24 | 18 / 53 / 21 | The shoulders, sleeves and waist remain joined in the inspected frames. The correction removes two predicted vertex collisions. The sternum and belt are visibly compressed on the female figure. An author should judge plate volume and emblem proportions before art approval. |
| Gauntlets | 21 / 44 / 15 | 21 / 42 / 8 | The wrist rings meet and the fists follow the attacking hand. The two splayed mitts occupy 262 icon pixels versus 459 for Barrows gloves. Their fingerless block silhouette reads differently from the reference. An inventory sculpt is warranted if separate fingers are intended. |
| Legs | 26 / 138 / 31 | 21 / 53 / 9 | **Needs an author at the rear waist.** In the raised-knee run, a plate paints over the upper blue waist panel. The heat map separates this from bare-arm differences. Correct the relative plate depth, overlap and priority locally, then repeat the full-set run. No large open waist seam appeared on the front full-set sheets. |
| Boots | 40 / 50 / 8 | 24 / 40 / 8 | Both boots follow their feet and the ankle rings meet. The repaired derivative removes one collapsed face. The cuff and greave overlap deserves another rear and side view with alternate legs. |
| Barbute | 26 / 53 / 14 | 21 / 40 / 9 | The crown, cheek plates and open jaw stay together on the standard head. The empty drop now excludes the kit face. The dialogue version moves with the stock face in expression 591. Other beard lengths, expressions and neck-covering capes still need inspection. |

## Three cape patterns

The Red cape has 61 male vertices and 162 faces, with the outside at priority 5 and dark lining at 1. The Fire cape has 42 vertices, 63 faces and 24 textured faces. The Attack cape has 311 vertices and 636 faces, a priority-0 lining, a priority-5 outside and 404 priority-6 hood/tie faces. All three occupy slot 1 and leave other slots available. The cloth follows labelled rows down the back, anchored at the shoulder/collar. The female Red cape hem rises 16 units from the male minimum height; the female Attack cape retains the male vertex and face counts with changes concentrated at the top.

A cape author needs separate inside/outside winding and paint, explicit row labels, a folded inventory mesh and an independently fitted female silhouette. The skillcape ties need their own labels. The current Python encoder rejects the Fire cape's texture channels. On the untextured Red cape the inherited re-encode matched the cache item on the running male figure in all four tested frames, 0/480,000 different pixels. No original cape was requested or built. [Construction and earlier layering sheets](capes/CAPES.md) · [Repeated measurements](completed/verification.txt) · [Four-frame proof](completed/assembly.txt)

## Reproduce and use

Run from {ROOT}/studies/research-items with the cache present at the path in itemlib.py. The pose, icon and ground executables are under ../../client/rust-client/target/release/examples. The new example built with the stable GNU toolchain. The source and output directories are filesystem snapshots without Git metadata. [Cargo command and output](completed/ground-build.txt)

~~~powershell
Set-Location {CLIENT_SRC}
$env:PATH = "{CARGO_BIN};{GIT_DIR}/usr/bin;$env:PATH"
cargo +stable-x86_64-pc-windows-gnu build --release --example meridian_ground

Set-Location {ROOT}/studies/research-items
$py = '{PYTHON}'
& $py -I recheck_survey.py
& $py -I finish_items.py verify
& $py -I file_icon_controls.py
& $py -I finish_items.py assets
& $py -I finish_items.py motion
& $py -I pose_controls.py controls
& $py -I pose_controls.py review chat
& $py -I assembly_check.py
& $py -I item_boards.py
& $py -I inspect_layering.py
& $py -I verify_delivery.py
~~~

The individual commands and stdout are retained in each item's build.txt, motion.txt, layering.txt and ground.txt. The final view search is deterministic and each build log includes its trial count, starting result, chosen result and target. The following commands show the public tool interface. The delivery check ran equivalent commands against temporary output paths and verified the pixels.

~~~powershell
& $py -I item_view.py icon real.png --item 1303
& $py -I item_view.py icon original.png --model completed/longsword/inventory.bin --view completed/longsword/inventory.view.json
& $py -I item_view.py ground ground.png --model completed/longsword/inventory.bin --view completed/longsword/inventory.view.json --neighbours 1293 1305
& $py -I fpose.py female-walk.png "none,item:1127,model:completed/cuirass/female.bin@body+arms" --gender 1 --seq 819 --frames bind,1,4
~~~

| Authoring task | Tool |
|---|---|
| Prepare the drop geometry, fit a view and inspect it at 36×32 | [Inventory-specific geometry](inventory_parts.py), [inventory poser](make_inventory.py), [row fitter](fit_row.py), [PNG and ground viewer](item_view.py) |
| Derive and check the female worn file | [Slot warp and seam tables](female_warp.py), [rounding repair](repair_warp.py), [female figure and pose CLI](fpose.py), [controls and layering](pose_controls.py) |
| Construct the barbute dialogue mesh | [Dialogue head builder](derive_head.py) |
| Encode the item and check the cache/client fields | [Definition encoder](itemcodec.py), [eight item recipes and model manifests](finish_items.py) |
| Build and verify the evidence | [Boards](item_boards.py), [full outfit](assembly_check.py), [delivery checks](verify_delivery.py) |

## Next authoring pass

Correct the female leg overlap first, using the supplied rear-quarter run crop as the regression frame. Compare the revised waist against both the cuirass and the stock torso. Then inspect alternate jaw and body kits, weapon-specific attacks and casts, and additional dialogue expressions. The current proof covers the local revision-239 render and definition paths. Server installation, live drop/pickup, economic values, combat bonuses, every animation frame and visual acceptance by the art owner remain untested.

