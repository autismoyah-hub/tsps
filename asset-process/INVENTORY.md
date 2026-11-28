# Inventory

Everything considered for this package, where it came from, and the rule that decided it. `WF/` is the original
working folder, `PR/` the private project repository. Sizes are of the source. Excluded items stay private.

## Summary

| Decision | Entries |
|---|---|
| excluded | 352 |
| included | 257 |
| partly included | 35 |

| Decision | Rule | Entries |
|---|---|---|
| excluded | asset build: our model | 148 |
| excluded | asset build: rebuild of a real asset | 57 |
| excluded | sitting spec of the project | 22 |
| excluded | superseded copy | 13 |
| excluded | picture of a sitting or board | 13 |
| excluded | picture of a real asset | 8 |
| excluded | picture of real assets | 7 |
| excluded | Producer run folder | 7 |
| excluded | script that wrote prompts or state for one night run | 6 |
| excluded | model or helper of an asset build | 5 |
| excluded | private repository front matter | 5 |
| excluded | review of the project with the owner | 5 |
| excluded | Python cache | 4 |
| excluded | one-off board script of one sitting | 4 |
| excluded | build rounds of an original creature | 3 |
| excluded | sitting pictures | 3 |
| excluded | project knowledge base: project history and owner statements | 3 |
| excluded | reference sheets of real interfaces | 2 |
| excluded | animation rounds of one creature | 2 |
| excluded | reference sheets of real bosses | 2 |
| excluded | cargo build output | 2 |
| excluded | a Producer trial run | 2 |
| excluded | mirrors the private working copy into the private repository | 2 |
| excluded | one-off cleanup of this machine's disks | 1 |
| excluded | derived from the cache: regenerate with studies/tools-gaps/make_body_kits.py | 1 |
| excluded | gate outputs of one batch | 1 |
| excluded | duplicate snapshot of the study scripts with run outputs | 1 |
| excluded | build of an original effect set | 1 |
| excluded | build rounds of an original effect set | 1 |
| excluded | build of an original helm | 1 |
| excluded | files of an original NPC | 1 |
| excluded | scratch probes and patches of one run | 1 |
| excluded | superseded copies of tools/ kept for a before/after comparison | 1 |
| excluded | test output of review_pack.py: copies of owner records | 1 |
| excluded | reference sheets of real HD models | 1 |
| excluded | the owner's private record | 1 |
| excluded | copy of the working folder | 1 |
| excluded | owner's verdicts and COUNT.md | 1 |
| excluded | owner record | 1 |
| excluded | answer keys of blind line-ups | 1 |
| excluded | measurements derived from the cache | 1 |
| excluded | real meshes dumped into Blender files | 1 |
| excluded | reference pictures of real assets | 1 |
| excluded | renders | 1 |
| excluded | housekeeping list of one cleanup | 1 |
| excluded | pictures of a reviewer trial | 1 |
| excluded | third-party installer | 1 |
| excluded | a scratch TSPS copy and its logs | 1 |
| excluded | boards: pictures of real assets beside ours, and their answer keys | 1 |
| excluded | build output | 1 |
| included | prompt, as a template | 144 |
| included | tool | 50 |
| included | author pack: rules, briefs, tools | 27 |
| included | the Teacher role | 26 |
| included | sitting tool | 3 |
| included | measured band limits the gate reads | 2 |
| included | worked example named by RULES.md and the runbook | 1 |
| included | bands fightcheck.py reads: 16 measured fights, mechanics counts and timings from the TSPS server source | 1 |
| included | statistics gapcheck.py reads: per-measure bands, aggregate numbers | 1 |
| included | repo script: stamps STATE.md time | 1 |
| included | client source | 1 |
| partly included | study: rules and scripts kept, data out | 35 |

## Working folder: tools/

| Source | Decision | Rule | Size | In the package |
|---|---|---|---|---|
| `WF/tools/__pycache__` | excluded | Python cache | 166.1 KB |  |
| `WF/tools/anatomy.py` | included | tool | 2.0 KB | `tools/anatomy.py` |
| `WF/tools/anim_gif.py` | included | tool | 2.5 KB | `tools/anim_gif.py` |
| `WF/tools/armour_card.py` | included | tool | 2.2 KB | `tools/armour_card.py` |
| `WF/tools/assemble.py` | included | tool | 1.2 KB | `tools/assemble.py` |
| `WF/tools/author.pre-textures.py` | excluded | superseded copy | 4.5 KB |  |
| `WF/tools/author.py` | included | tool | 4.7 KB | `tools/author.py` |
| `WF/tools/bands.py` | included | tool | 1.1 KB | `tools/bands.py` |
| `WF/tools/blender_export.pre-textures.py` | excluded | superseded copy | 4.9 KB |  |
| `WF/tools/blender_export.py` | included | tool | 5.5 KB | `tools/blender_export.py` |
| `WF/tools/blender_kit.pre-textures.py` | excluded | superseded copy | 2.8 KB |  |
| `WF/tools/blender_kit.py` | included | tool | 3.3 KB | `tools/blender_kit.py` |
| `WF/tools/blender_run.sh` | included | tool | 963 B | `tools/blender_run.sh` |
| `WF/tools/board.py` | included | tool | 799 B | `tools/board.py` |
| `WF/tools/cache_material.py` | included | tool | 4.1 KB | `tools/cache_material.py` |
| `WF/tools/compare.before-recolour-fix.py` | excluded | superseded copy | 6.8 KB |  |
| `WF/tools/compare.pre-textures.py` | excluded | superseded copy | 6.4 KB |  |
| `WF/tools/compare.py` | included | tool | 7.7 KB | `tools/compare.py` |
| `WF/tools/compare.py.bak-walldecor` | excluded | superseded copy | 6.3 KB |  |
| `WF/tools/contact.py` | included | tool | 1.4 KB | `tools/contact.py` |
| `WF/tools/creature_card.py` | included | tool | 2.2 KB | `tools/creature_card.py` |
| `WF/tools/d_cleanup_20261127.py` | excluded | one-off cleanup of this machine's disks (machine paths only) | 1.7 KB |  |
| `WF/tools/decor_bands.json` | included | measured band limits the gate reads (aggregate numbers) | 3.5 KB | `tools/decor_bands.json` |
| `WF/tools/decor_bands.py` | included | tool | 10.3 KB | `tools/decor_bands.py` |
| `WF/tools/dsize.py` | included | tool | 1.2 KB | `tools/dsize.py` |
| `WF/tools/dump_worn_labels.py` | included | tool | 1.7 KB | `tools/dump_worn_labels.py` |
| `WF/tools/family.py` | included | tool | 1.0 KB | `tools/family.py` |
| `WF/tools/figure.py` | included | tool | 3.4 KB | `tools/figure.py` |
| `WF/tools/gate.pre-textures.py` | excluded | superseded copy | 21.8 KB |  |
| `WF/tools/gate.py` | included | tool | 22.5 KB | `tools/gate.py` |
| `WF/tools/gate.py.bak-walldecor` | excluded | superseded copy | 13.1 KB |  |
| `WF/tools/handoff.py` | included | tool | 1.4 KB | `tools/handoff.py` |
| `WF/tools/held_out.py` | included | tool | 4.3 KB | `tools/held_out.py` |
| `WF/tools/label_armour.py` | included | tool | 32.1 KB | `tools/label_armour.py` |
| `WF/tools/label_overrides.py` | included | tool | 4.6 KB | `tools/label_overrides.py` |
| `WF/tools/lineup.py` | included | tool | 7.4 KB | `tools/lineup.py` |
| `WF/tools/lineup.py.bak-walldecor` | excluded | superseded copy | 6.7 KB |  |
| `WF/tools/material_bands.json` | included | measured band limits the gate reads (aggregate numbers) | 9.3 KB | `tools/material_bands.json` |
| `WF/tools/measure.py` | included | tool | 4.6 KB | `tools/measure.py` |
| `WF/tools/minority.py` | included | tool | 1.1 KB | `tools/minority.py` |
| `WF/tools/model_packing.py` | included | tool | 4.5 KB | `tools/model_packing.py` |
| `WF/tools/objlight.py` | included | tool | 3.3 KB | `tools/objlight.py` |
| `WF/tools/osrscache.pre-textures.py` | excluded | superseded copy | 7.3 KB |  |
| `WF/tools/osrscache.py` | included | tool | 11.2 KB | `tools/osrscache.py` |
| `WF/tools/pick.py` | included | tool | 1.3 KB | `tools/pick.py` |
| `WF/tools/pose.py` | included | tool | 14.5 KB | `tools/pose.py` |
| `WF/tools/real_dump.pre-textures.py` | excluded | superseded copy | 2.4 KB |  |
| `WF/tools/real_dump.py` | included | tool | 3.2 KB | `tools/real_dump.py` |
| `WF/tools/real_to_blender.pre-textures.py` | excluded | superseded copy | 1.7 KB |  |
| `WF/tools/real_to_blender.py` | included | tool | 3.4 KB | `tools/real_to_blender.py` |
| `WF/tools/real_to_blender.py.bak-walldecor` | excluded | superseded copy | 987 B |  |
| `WF/tools/recolour.py` | included | tool | 1.8 KB | `tools/recolour.py` |
| `WF/tools/reencode_item.py` | included | tool | 6.1 KB | `tools/reencode_item.py` |
| `WF/tools/relabel.py` | included | tool | 3.0 KB | `tools/relabel.py` |
| `WF/tools/render_prune.py` | included | tool | 2.1 KB | `tools/render_prune.py` |
| `WF/tools/review_pack.py` | included | tool | 17.6 KB | `tools/review_pack.py` |
| `WF/tools/rmtree_long.py` | included | tool | 555 B | `tools/rmtree_long.py` |
| `WF/tools/roomprobe.py` | included | tool | 1.4 KB | `tools/roomprobe.py` |
| `WF/tools/scene.py` | included | tool | 9.0 KB | `tools/scene.py` |
| `WF/tools/seams.py` | included | tool | 2.0 KB | `tools/seams.py` |
| `WF/tools/smooth.py` | included | tool | 1.3 KB | `tools/smooth.py` |
| `WF/tools/treediff.py` | included | tool | 1.1 KB | `tools/treediff.py` |
| `WF/tools/tsps_anims.py` | included | tool | 1.2 KB | `tools/tsps_anims.py` |
| `WF/tools/vignette.py` | included | tool | 1.8 KB | `tools/vignette.py` |
| `WF/tools/weapon_axes.py` | included | tool | 1.2 KB | `tools/weapon_axes.py` |
| `WF/tools/weapon_card.py` | included | tool | 2.7 KB | `tools/weapon_card.py` |
| `WF/tools/wire.py` | included | tool | 6.8 KB | `tools/wire.py` |

## Working folder: author-pack/

| Source | Decision | Rule | Size | In the package |
|---|---|---|---|---|
| `WF/author-pack/BRIEF-armour-rebuild.md` | included | author pack: rules, briefs, tools | 6.0 KB | `author-pack/BRIEF-armour-rebuild.md` |
| `WF/author-pack/BRIEF-first-room.md` | included | author pack: rules, briefs, tools | 8.2 KB | `author-pack/BRIEF-first-room.md` |
| `WF/author-pack/BRIEF-furniture-rebuild.md` | included | author pack: rules, briefs, tools | 4.6 KB | `author-pack/BRIEF-furniture-rebuild.md` |
| `WF/author-pack/BRIEF-new-boss-pitch.md` | included | author pack: rules, briefs, tools | 5.5 KB | `author-pack/BRIEF-new-boss-pitch.md` |
| `WF/author-pack/BRIEF-observatory-set.md` | included | author pack: rules, briefs, tools | 2.4 KB | `author-pack/BRIEF-observatory-set.md` |
| `WF/author-pack/BRIEF-original-animation.md` | included | author pack: rules, briefs, tools | 4.7 KB | `author-pack/BRIEF-original-animation.md` |
| `WF/author-pack/BRIEF-original-armour.md` | included | author pack: rules, briefs, tools | 8.1 KB | `author-pack/BRIEF-original-armour.md` |
| `WF/author-pack/BRIEF-original-creature.md` | included | author pack: rules, briefs, tools | 20.5 KB | `author-pack/BRIEF-original-creature.md` |
| `WF/author-pack/BRIEF-original-set-piece.md` | included | author pack: rules, briefs, tools | 6.2 KB | `author-pack/BRIEF-original-set-piece.md` |
| `WF/author-pack/BRIEF-original-weapon.md` | included | author pack: rules, briefs, tools | 6.2 KB | `author-pack/BRIEF-original-weapon.md` |
| `WF/author-pack/BRIEF-player-helm.md` | included | author pack: rules, briefs, tools | 7.7 KB | `author-pack/BRIEF-player-helm.md` |
| `WF/author-pack/BRIEF-weapon-rebuild.md` | included | author pack: rules, briefs, tools | 5.8 KB | `author-pack/BRIEF-weapon-rebuild.md` |
| `WF/author-pack/FASHIONSCAPE.md` | included | author pack: rules, briefs, tools | 17.6 KB | `author-pack/FASHIONSCAPE.md` |
| `WF/author-pack/labels/atlas-body.png` | excluded | picture of real assets (label atlas of the game's body kits) | 149.6 KB |  |
| `WF/author-pack/labels/atlas-cape.png` | excluded | picture of real assets (label atlas of the game's body kits) | 102.8 KB |  |
| `WF/author-pack/labels/atlas-feet.png` | excluded | picture of real assets (label atlas of the game's body kits) | 72.9 KB |  |
| `WF/author-pack/labels/atlas-figure.png` | excluded | picture of real assets (label atlas of the game's body kits) | 158.8 KB |  |
| `WF/author-pack/labels/atlas-hands.png` | excluded | picture of real assets (label atlas of the game's body kits) | 53.7 KB |  |
| `WF/author-pack/labels/atlas-head.png` | excluded | picture of real assets (label atlas of the game's body kits) | 83.0 KB |  |
| `WF/author-pack/labels/atlas-legs.png` | excluded | picture of real assets (label atlas of the game's body kits) | 158.0 KB |  |
| `WF/author-pack/labels/body-kits.json` | excluded | derived from the cache: regenerate with studies/tools-gaps/make_body_kits.py | 29.8 KB |  |
| `WF/author-pack/labels/LABELS.md` | included | author pack: rules, briefs, tools | 17.8 KB | `author-pack/labels/LABELS.md` |
| `WF/author-pack/PLAYER-GEAR.md` | included | author pack: rules, briefs, tools | 17.4 KB | `author-pack/PLAYER-GEAR.md` |
| `WF/author-pack/ROOMS.md` | included | author pack: rules, briefs, tools | 17.5 KB | `author-pack/ROOMS.md` |
| `WF/author-pack/RULES.md` | included | author pack: rules, briefs, tools | 26.8 KB | `author-pack/RULES.md` |
| `WF/author-pack/tools/check_predict.py` | included | author pack: rules, briefs, tools | 5.5 KB | `author-pack/tools/check_predict.py` |
| `WF/author-pack/tools/gamecol.py` | included | author pack: rules, briefs, tools | 3.2 KB | `author-pack/tools/gamecol.py` |
| `WF/author-pack/tools/pairs.py` | included | author pack: rules, briefs, tools | 6.2 KB | `author-pack/tools/pairs.py` |
| `WF/author-pack/tools/predict_light.py` | included | author pack: rules, briefs, tools | 11.9 KB | `author-pack/tools/predict_light.py` |
| `WF/author-pack/tools/px2col.py` | included | author pack: rules, briefs, tools | 6.5 KB | `author-pack/tools/px2col.py` |
| `WF/author-pack/tools/README.md` | included | author pack: rules, briefs, tools | 7.1 KB | `author-pack/tools/README.md` |
| `WF/author-pack/tools/testdata/drum.bin` | included | author pack: rules, briefs, tools (synthetic test model made by make_drum.py) | 385 B | `author-pack/tools/testdata/drum.bin` |
| `WF/author-pack/tools/testdata/make_drum.py` | included | author pack: rules, briefs, tools | 880 B | `author-pack/tools/testdata/make_drum.py` |
| `WF/author-pack/tools/variants.md` | included | author pack: rules, briefs, tools | 3.0 KB | `author-pack/tools/variants.md` |
| `WF/author-pack/WALL-DECOR.md` | included | author pack: rules, briefs, tools | 11.2 KB | `author-pack/WALL-DECOR.md` |
| `WF/author-pack/wireframes/real-15674.png` | excluded | picture of a real asset | 82.3 KB |  |
| `WF/author-pack/wireframes/real-15704.png` | excluded | picture of a real asset | 72.9 KB |  |
| `WF/author-pack/wireframes/real-17987.png` | excluded | picture of a real asset | 55.7 KB |  |
| `WF/author-pack/wireframes/real-22472.png` | excluded | picture of a real asset | 65.6 KB |  |
| `WF/author-pack/wireframes/real-2333.png` | excluded | picture of a real asset | 61.0 KB |  |
| `WF/author-pack/wireframes/real-27687.png` | excluded | picture of a real asset | 49.8 KB |  |
| `WF/author-pack/wireframes/real-55076.png` | excluded | picture of a real asset | 50.9 KB |  |
| `WF/author-pack/wireframes/real-61111.png` | excluded | picture of a real asset | 107.1 KB |  |
| `WF/author-pack/tools/__pycache__` | excluded | Python cache | 50.3 KB |  |

## Working folder: builds/ (one line per build folder)

| Source | Decision | Rule | Size | In the package |
|---|---|---|---|---|
| `WF/builds/anim-hound-horn-toss` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1004.3 MB |  |
| `WF/builds/anim-hound-set` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 99.1 MB |  |
| `WF/builds/anim-meridian-hammer` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 31.2 KB |  |
| `WF/builds/anim-warden-general` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 6.8 MB |  |
| `WF/builds/arena1-armillary-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 193.7 KB |  |
| `WF/builds/arena1-armillary-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 197.4 KB |  |
| `WF/builds/arena1-barrier-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 161.0 KB |  |
| `WF/builds/arena1-barrier-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 52.0 KB |  |
| `WF/builds/arena1-cal-trapdoor` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 94.1 KB |  |
| `WF/builds/arena1-lane-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 220.3 KB |  |
| `WF/builds/arena1-north-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 132.4 KB |  |
| `WF/builds/arena1-north-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 170.5 KB |  |
| `WF/builds/arena1-parapet-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 715.9 KB |  |
| `WF/builds/arena1-pier-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 260.1 KB |  |
| `WF/builds/arena1-pier-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 236.8 KB |  |
| `WF/builds/arena1-ring-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 155.7 KB |  |
| `WF/builds/arena1-ring-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 232.9 KB |  |
| `WF/builds/arena1-roof-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 131.6 KB |  |
| `WF/builds/arena1-roof-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 145.8 KB |  |
| `WF/builds/arena1-slate-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 636.0 KB |  |
| `WF/builds/arena1-statue-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 266.8 KB |  |
| `WF/builds/arena1-statue-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 149.5 KB |  |
| `WF/builds/arena1r-roof-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 131.9 KB |  |
| `WF/builds/arena1r-roof-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 167.8 KB |  |
| `WF/builds/arena1r-roof-b-r1` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 149.9 KB |  |
| `WF/builds/arm-cal-stool` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 696.2 KB |  |
| `WF/builds/arm2-cal-square-shield` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 250.5 KB |  |
| `WF/builds/armour-chainbody-1101` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 4.4 MB |  |
| `WF/builds/armour-helm-10828` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 147.8 MB |  |
| `WF/builds/barrel/build.py` | included | worked example named by RULES.md and the runbook (script only; writes barrel.bin) | 5.5 KB | `examples/barrel/build.py` |
| `WF/builds/barrel/barrel-from-blender.bin` | excluded | model or helper of an asset build | 5.7 KB |  |
| `WF/builds/barrel/barrel.bin` | excluded | model or helper of an asset build | 5.6 KB |  |
| `WF/builds/barrel/barrel.blend` | excluded | model or helper of an asset build | 106.8 KB |  |
| `WF/builds/barrel/flipped.bin` | excluded | model or helper of an asset build | 5.6 KB |  |
| `WF/builds/barrel/to_blender.py` | excluded | model or helper of an asset build | 1.1 KB |  |
| `WF/builds/barrel/__pycache__` | excluded | Python cache | 9.8 KB |  |
| `WF/builds/bench-55076` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 45.4 MB |  |
| `WF/builds/boss-pitch-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 702.7 KB |  |
| `WF/builds/boss-pitch-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 722.9 KB |  |
| `WF/builds/boss-pitch-c` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 900.4 KB |  |
| `WF/builds/boss-pitch-d` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 818.2 KB |  |
| `WF/builds/boss-pitch-e` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 914.2 KB |  |
| `WF/builds/boss-pitch-f` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 751.9 KB |  |
| `WF/builds/boss-pitch-g` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.6 MB |  |
| `WF/builds/boss-pitch-h` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 884.9 KB |  |
| `WF/builds/boss-pitch-i` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.8 MB |  |
| `WF/builds/boss-pitch-j` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.1 MB |  |
| `WF/builds/boss-pitch-k` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 664.7 KB |  |
| `WF/builds/boss-rebuild-11278` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 10.1 MB |  |
| `WF/builds/boss-rebuild-7541` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 6.0 MB |  |
| `WF/builds/boss-rebuild-7541-2` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 19.3 MB |  |
| `WF/builds/boss-rebuild-7541-3` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 31.6 MB |  |
| `WF/builds/boss-rebuild-7541-4` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 78.3 MB |  |
| `WF/builds/boss-warden-general` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 5.5 MB |  |
| `WF/builds/boss-warden-general-2` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 11.9 MB |  |
| `WF/builds/boss-warden-general-3` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 2.9 MB |  |
| `WF/builds/boss-warden-general-4` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 6.0 MB |  |
| `WF/builds/cmp-drawers-astra` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 384.6 KB |  |
| `WF/builds/cmp-drawers-claude` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 5.2 MB |  |
| `WF/builds/cmp-drawers-opus` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.1 MB |  |
| `WF/builds/crate-15704` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 25.7 MB |  |
| `WF/builds/creature-orrery-spider` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.6 MB |  |
| `WF/builds/creature-rebuild-11068` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 666.6 KB |  |
| `WF/builds/creature-rebuild-11175` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 952.1 KB |  |
| `WF/builds/creature-rebuild-12812` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 1.5 MB |  |
| `WF/builds/creature-star-hound-2` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 147.4 KB |  |
| `WF/builds/creature-star-hound-3` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.0 MB |  |
| `WF/builds/creature-star-hound-4` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 28.8 MB |  |
| `WF/builds/creature-stone-watcher` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.5 MB |  |
| `WF/builds/creature-stone-watcher-2` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 3.6 MB |  |
| `WF/builds/creature-stone-watcher-3` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 3.4 MB |  |
| `WF/builds/creature-stone-watcher-4` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 7.1 MB |  |
| `WF/builds/effect-rebuild-1595` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 147.6 KB |  |
| `WF/builds/effect-rebuild-2008` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 81.0 KB |  |
| `WF/builds/effect-rebuild-2206` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 54.6 KB |  |
| `WF/builds/fx-stone-watcher` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 291.1 MB |  |
| `WF/builds/fx-warden-general` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 435.2 MB |  |
| `WF/builds/gate-batch-0128` | excluded | gate outputs of one batch (records) | 195.4 KB |  |
| `WF/builds/hd-rebuild-diving` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 219.9 MB |  |
| `WF/builds/hd-rebuild-lampstall` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 519.5 MB |  |
| `WF/builds/hd-rebuild-spot2768` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 274.1 MB |  |
| `WF/builds/helm-rebuild-12931` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 9.1 MB |  |
| `WF/builds/helm-rebuild-22326` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 885.2 KB |  |
| `WF/builds/helm-rebuild-24271` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 1.1 MB |  |
| `WF/builds/helm-rebuild-26382` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 727.8 KB |  |
| `WF/builds/helm-rebuild-30750` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 5.5 MB |  |
| `WF/builds/iface-original-o1` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 187.1 KB |  |
| `WF/builds/iface-rebuild-156` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 90.5 KB |  |
| `WF/builds/iface-rebuild-206` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 88.3 KB |  |
| `WF/builds/iface-rebuild-302` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 94.5 KB |  |
| `WF/builds/iface-rebuild-303` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 28.2 KB |  |
| `WF/builds/iface-rebuild-383` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 103.4 KB |  |
| `WF/builds/iface-rebuild-4` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 67.4 KB |  |
| `WF/builds/iface-rebuild-422` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 119.8 KB |  |
| `WF/builds/iface-rebuild-648` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 102.4 KB |  |
| `WF/builds/iface-rebuild-868` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 47.4 KB |  |
| `WF/builds/lectern-22472` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 4.0 MB |  |
| `WF/builds/meridian-boots` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 273.9 KB |  |
| `WF/builds/meridian-boots-v3` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 258.0 KB |  |
| `WF/builds/meridian-boots-v5` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 199.5 KB |  |
| `WF/builds/meridian-cape` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.2 MB |  |
| `WF/builds/meridian-crush-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 148.8 KB |  |
| `WF/builds/meridian-crush-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 116.1 KB |  |
| `WF/builds/meridian-cuirass` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 3.1 MB |  |
| `WF/builds/meridian-cuirass-v2` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 4.2 MB |  |
| `WF/builds/meridian-cuirass-v3` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 2.0 MB |  |
| `WF/builds/meridian-cuirass-v5` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 232.7 KB |  |
| `WF/builds/meridian-gauntlets` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 257.0 KB |  |
| `WF/builds/meridian-gauntlets-v3` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 643.5 KB |  |
| `WF/builds/meridian-gauntlets-v5` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 219.7 KB |  |
| `WF/builds/meridian-glaive` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 313.4 KB |  |
| `WF/builds/meridian-helm` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 309.4 KB |  |
| `WF/builds/meridian-helm-v2` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 691.5 KB |  |
| `WF/builds/meridian-helm-v3` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 4.8 MB |  |
| `WF/builds/meridian-helm-v4-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 159.4 KB |  |
| `WF/builds/meridian-helm-v4-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.7 MB |  |
| `WF/builds/meridian-helm-v4-c` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 194.8 KB |  |
| `WF/builds/meridian-helm-v5-astra` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 169.0 KB |  |
| `WF/builds/meridian-helm-v5-opus` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 261.1 KB |  |
| `WF/builds/meridian-legs` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 2.3 MB |  |
| `WF/builds/meridian-legs-v3` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 792.8 KB |  |
| `WF/builds/meridian-legs-v5` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.1 MB |  |
| `WF/builds/meridian-staff` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 342.2 KB |  |
| `WF/builds/meridian-sword` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 285.0 KB |  |
| `WF/builds/object-rebuild-11700` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 79.9 KB |  |
| `WF/builds/object-rebuild-33035` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 82.8 KB |  |
| `WF/builds/object-rebuild-51046` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 618.3 KB |  |
| `WF/builds/obs-dark` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 67.9 KB |  |
| `WF/builds/obs1-cal-crystal-13659` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 2.7 MB |  |
| `WF/builds/obs2-cal-drawers-7195` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 2.1 MB |  |
| `WF/builds/obs3-cal-barstool-1100` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 833.1 KB |  |
| `WF/builds/original-arm-bench-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 457.5 KB |  |
| `WF/builds/original-arm-bench-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 201.9 KB |  |
| `WF/builds/original-arm-chest-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 186.6 KB |  |
| `WF/builds/original-arm-chest-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 333.3 KB |  |
| `WF/builds/original-arm-dummy-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 196.7 KB |  |
| `WF/builds/original-arm-dummy-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 330.8 KB |  |
| `WF/builds/original-arm-pail-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 177.6 KB |  |
| `WF/builds/original-arm-pail-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 254.3 KB |  |
| `WF/builds/original-arm-rack-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 498.5 KB |  |
| `WF/builds/original-arm-rack-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 146.9 KB |  |
| `WF/builds/original-arm-stand-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 437.7 KB |  |
| `WF/builds/original-arm-stand-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 390.4 KB |  |
| `WF/builds/original-arm2-brazier-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 201.6 KB |  |
| `WF/builds/original-arm2-brazier-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 418.4 KB |  |
| `WF/builds/original-arm2-cabinet-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 187.6 KB |  |
| `WF/builds/original-arm2-cabinet-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 189.7 KB |  |
| `WF/builds/original-arm2-crate-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 306.6 KB |  |
| `WF/builds/original-arm2-helm-shelf-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 202.9 KB |  |
| `WF/builds/original-arm2-helm-shelf-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 498.4 KB |  |
| `WF/builds/original-arm2-plaque-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 341.9 KB |  |
| `WF/builds/original-arm2-plaque-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 345.3 KB |  |
| `WF/builds/original-arm2-rug-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 582.3 KB |  |
| `WF/builds/original-arm2-rug-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 141.3 KB |  |
| `WF/builds/original-arm2-shield-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 125.9 KB |  |
| `WF/builds/original-arm2-shield-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 121.3 KB |  |
| `WF/builds/original-arm2-standard-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 393.6 KB |  |
| `WF/builds/original-arm2-standard-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 432.5 KB |  |
| `WF/builds/original-arm2-torch-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 421.0 KB |  |
| `WF/builds/original-arm2-torch-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 173.3 KB |  |
| `WF/builds/original-arm2-weapons-barrel-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 217.3 KB |  |
| `WF/builds/original-arm2-weapons-barrel-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 180.4 KB |  |
| `WF/builds/original-maptable-astra` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 421.3 KB |  |
| `WF/builds/original-maptable-sol` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.7 MB |  |
| `WF/builds/original-obs1-orrery-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 218.0 KB |  |
| `WF/builds/original-obs1-orrery-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.6 MB |  |
| `WF/builds/original-obs1-telescope-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 293.3 KB |  |
| `WF/builds/original-obs1-telescope-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.8 MB |  |
| `WF/builds/original-obs2-cabinet-glazed` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 322.4 KB |  |
| `WF/builds/original-obs2-cabinet-open` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 536.6 KB |  |
| `WF/builds/original-obs2-table-pedestal` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 223.7 KB |  |
| `WF/builds/original-obs2-table-trestle` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 504.8 KB |  |
| `WF/builds/original-obs3-candle-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 317.8 KB |  |
| `WF/builds/original-obs3-candle-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 405.4 KB |  |
| `WF/builds/original-obs3-globe-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 442.2 KB |  |
| `WF/builds/original-obs3-globe-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 247.9 KB |  |
| `WF/builds/original-obs3-rug-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 199.0 KB |  |
| `WF/builds/original-obs3-rug-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 952.2 KB |  |
| `WF/builds/original-obs3-stool-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 383.4 KB |  |
| `WF/builds/original-obs3-stool-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 340.1 KB |  |
| `WF/builds/original-scribe-bookcase` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 7.8 MB |  |
| `WF/builds/original-scribe-cabinet` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 3.8 MB |  |
| `WF/builds/original-scribe-chair` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 4.4 MB |  |
| `WF/builds/original-scribe-desk` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 7.0 MB |  |
| `WF/builds/original-trial-astra-chest-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 804.9 KB |  |
| `WF/builds/original-trial-astra-chest-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 1.6 MB |  |
| `WF/builds/original-trial-opus-docchest-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 249.6 KB |  |
| `WF/builds/original-trial-opus-docchest-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 229.6 KB |  |
| `WF/builds/player-set-complete` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 313.3 MB |  |
| `WF/builds/player-set-v5-complete` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 342.4 MB |  |
| `WF/builds/qual-chainbody-astra` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 905.4 KB |  |
| `WF/builds/qual-fang-astra` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 265.6 KB |  |
| `WF/builds/qual-legs-astra` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 590.9 KB |  |
| `WF/builds/qual-platebody-astra` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 245.6 KB |  |
| `WF/builds/research-animation/verification` | excluded | duplicate snapshot of the study scripts with run outputs | 433.0 MB |  |
| `WF/builds/research-animation` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (19 files kept, 120 out: data or picture (derived from the game caches or run output) 120; Python cache 1) | 649.7 MB | `studies/research-animation` |
| `WF/builds/research-arenas` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (19 files kept, 80 out: data or picture (derived from the game caches or run output) 79; Python cache 1; bulk per-arena tables (regenerate with studies/research-arenas/table.py) 1) | 89.0 MB | `studies/research-arenas` |
| `WF/builds/research-authorship` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (8 files kept, 28 out: data or picture (derived from the game caches or run output) 28; Python cache 1) | 2.5 MB | `studies/research-authorship` |
| `WF/builds/research-bosses` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (22 files kept, 759 out: data or picture (derived from the game caches or run output) 759; Python cache 1) | 363.7 MB | `studies/research-bosses` |
| `WF/builds/research-clean` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (20 files kept, 152 out: data or picture (derived from the game caches or run output) 152; Python cache 1) | 18.6 MB | `studies/research-clean` |
| `WF/builds/research-creatures` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (18 files kept, 541 out: data or picture (derived from the game caches or run output) 541; Python cache 1) | 17.2 MB | `studies/research-creatures` |
| `WF/builds/research-effects/originals` | excluded | build of an original effect set (our asset) | 94.4 KB |  |
| `WF/builds/research-effects` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (12 files kept, 535 out: data or picture (derived from the game caches or run output) 535; Python cache 1) | 236.6 MB | `studies/research-effects` |
| `WF/builds/research-effects2/set2` | excluded | build rounds of an original effect set (our asset) | 395.3 KB |  |
| `WF/builds/research-effects2` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (28 files kept, 856 out: data or picture (derived from the game caches or run output) 856; Python cache 2) | 24.5 MB | `studies/research-effects2` |
| `WF/builds/research-fights/metrics.json` | included | bands fightcheck.py reads: 16 measured fights, mechanics counts and timings from the TSPS server source (10 KB) | 9.9 KB | `studies/research-fights/metrics.json` |
| `WF/builds/research-fights` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (8 files kept, 22 out: data or picture (derived from the game caches or run output) 21; bulk per-boss table (regenerate with studies/research-fights/analyze.py) 1) | 473.2 KB | `studies/research-fights` |
| `WF/builds/research-fit` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (16 files kept, 61 out: data or picture (derived from the game caches or run output) 61; Python cache 1) | 2.1 MB | `studies/research-fit` |
| `WF/builds/research-gap/data/stats.json` | included | statistics gapcheck.py reads: per-measure bands, aggregate numbers (50 KB) | 49.5 KB | `studies/research-gap/data/stats.json` |
| `WF/builds/research-gap` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (12 files kept, 99 out: data or picture (derived from the game caches or run output) 98; Python cache 1; corpus of the owner's recorded words (owner record) 1) | 13.4 MB | `studies/research-gap` |
| `WF/builds/research-helm2/umbral-helm-a2` | excluded | build of an original helm (our asset) | 167.1 KB |  |
| `WF/builds/research-helm2` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (14 files kept, 74 out: data or picture (derived from the game caches or run output) 74) | 4.4 MB | `studies/research-helm2` |
| `WF/builds/research-helm3` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (10 files kept, 60 out: data or picture (derived from the game caches or run output) 60) | 3.0 MB | `studies/research-helm3` |
| `WF/builds/research-interfaces/sheets` | excluded | reference sheets of real interfaces | 1.2 MB |  |
| `WF/builds/research-interfaces/sheets-teacher-only` | excluded | reference sheets of real interfaces (held out) | 3.9 MB |  |
| `WF/builds/research-interfaces` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (43 files kept, 1783 out: data or picture (derived from the game caches or run output) 1777; superseded copy or scratch 4; Python cache 2; record of a run (notes, state, proposals) 2) | 171.2 MB | `studies/research-interfaces` |
| `WF/builds/research-items` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (31 files kept, 292 out: data or picture (derived from the game caches or run output) 292; Python cache 1) | 227.6 MB | `studies/research-items` |
| `WF/builds/research-live` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (11 files kept, 557 out: data or picture (derived from the game caches or run output) 557) | 259.4 MB | `studies/research-live` |
| `WF/builds/research-npc/hound` | excluded | build rounds of an original creature (our asset) | 722.0 KB |  |
| `WF/builds/research-npc/warden` | excluded | files of an original NPC (our asset) | 15.9 KB |  |
| `WF/builds/research-npc` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (9 files kept, 55 out: data or picture (derived from the game caches or run output) 55; Python cache 1) | 12.2 MB | `studies/research-npc` |
| `WF/builds/research-paint` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (22 files kept, 563 out: data or picture (derived from the game caches or run output) 563; Python cache 2) | 53.7 MB | `studies/research-paint` |
| `WF/builds/research-rig/anim` | excluded | animation rounds of one creature | 85.0 MB |  |
| `WF/builds/research-rig` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (36 files kept, 370 out: data or picture (derived from the game caches or run output) 370; Python cache 1) | 449.0 MB | `studies/research-rig` |
| `WF/builds/research-rig2/anim` | excluded | animation rounds of one creature | 119.8 MB |  |
| `WF/builds/research-rig2/ray` | excluded | build rounds of an original creature (our asset) | 227.2 MB |  |
| `WF/builds/research-rig2/scratch` | excluded | scratch probes and patches of one run | 37.7 KB |  |
| `WF/builds/research-rig2` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (46 files kept, 492 out: data or picture (derived from the game caches or run output) 492; Python cache 1) | 504.5 MB | `studies/research-rig2` |
| `WF/builds/research-rooms` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (22 files kept, 130 out: data or picture (derived from the game caches or run output) 130; Python cache 1) | 23.5 MB | `studies/research-rooms` |
| `WF/builds/research-skeleton/sentinel` | excluded | build rounds of an original creature (our asset) | 606.5 KB |  |
| `WF/builds/research-skeleton` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (13 files kept, 7922 out: data or picture (derived from the game caches or run output) 7922; Python cache 1) | 437.7 MB | `studies/research-skeleton` |
| `WF/builds/research-tone/sheets` | excluded | reference sheets of real bosses | 30.5 MB |  |
| `WF/builds/research-tone` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (22 files kept, 1676 out: data or picture (derived from the game caches or run output) 1675; Python cache 1; superseded copy or scratch 1) | 66.7 MB | `studies/research-tone` |
| `WF/builds/research-tsps` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (2 files kept, 0 out: ) | 29.0 KB | `studies/research-tsps` |
| `WF/builds/room-armoury` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 204.2 KB |  |
| `WF/builds/room-observatory` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 527.6 KB |  |
| `WF/builds/room-observatory-furnished` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 147.3 KB |  |
| `WF/builds/room-scribes-hall` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 11.3 MB |  |
| `WF/builds/rs3-rebuild-crossbow` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 1.2 MB |  |
| `WF/builds/rs3-rebuild-havoc-hood` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 1.4 MB |  |
| `WF/builds/rs3-rebuild-kerapac` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 1.3 MB |  |
| `WF/builds/titan-rebuild-12596-a` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 366.0 MB |  |
| `WF/builds/titan-rebuild-12596-b` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 230.1 MB |  |
| `WF/builds/titan-rebuild-14147-a` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 305.0 MB |  |
| `WF/builds/titan-rebuild-14147-b` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 130.0 MB |  |
| `WF/builds/tools-fight-arena` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (4 files kept, 119 out: data or picture (derived from the game caches or run output) 119) | 452.3 MB | `studies/tools-fight-arena` |
| `WF/builds/tools-gaps/before` | excluded | superseded copies of tools/ kept for a before/after comparison | 192.8 KB |  |
| `WF/builds/tools-gaps` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (11 files kept, 306 out: data or picture (derived from the game caches or run output) 305; Python cache 1; executable: copied to client/bin when a tool calls it 1) | 131.8 MB | `studies/tools-gaps` |
| `WF/builds/tools-handoff` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (6 files kept, 30 out: data or picture (derived from the game caches or run output) 29; run log with owner records quoted (37 KB) 1) | 335.0 KB | `studies/tools-handoff` |
| `WF/builds/tools-handoff2/review-pack-test` | excluded | test output of review_pack.py: copies of owner records | 3.3 MB |  |
| `WF/builds/tools-handoff2` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (14 files kept, 0 out: ) | 3.3 MB | `studies/tools-handoff2` |
| `WF/builds/tools-hd-era/sheets` | excluded | reference sheets of real HD models | 1.0 MB |  |
| `WF/builds/tools-hd-era/target` | excluded | cargo build output (binaries that tools call are in client/bin) | 72.6 MB |  |
| `WF/builds/tools-hd-era` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (36 files kept, 31 out: data or picture (derived from the game caches or run output) 31; build output or cache 1) | 114.7 MB | `studies/tools-hd-era` |
| `WF/builds/tools-hd-gate/target` | excluded | cargo build output (binaries that tools call are in client/bin) | 72.6 MB |  |
| `WF/builds/tools-hd-gate` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (15 files kept, 81 out: data or picture (derived from the game caches or run output) 80; build output or cache 1; run log of one author session (135 KB) 1) | 74.7 MB | `studies/tools-hd-gate` |
| `WF/builds/tools-real-fight` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (6 files kept, 4 out: data or picture (derived from the game caches or run output) 3; executable: copied to client/bin when a tool calls it 1) | 14.4 MB | `studies/tools-real-fight` |
| `WF/builds/tools-render-quality` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (11 files kept, 15 out: data or picture (derived from the game caches or run output) 13; executable: copied to client/bin when a tool calls it 2; Python cache 1) | 27.8 MB | `studies/tools-render-quality` |
| `WF/builds/tools-rs3` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (39 files kept, 20975 out: data or picture (derived from the game caches or run output) 20965; superseded copy or scratch 10) | 1.4 GB | `studies/tools-rs3` |
| `WF/builds/tools-textures-alpha` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (16 files kept, 98 out: data or picture (derived from the game caches or run output) 98) | 6.9 MB | `studies/tools-textures-alpha` |
| `WF/builds/tools-titan-sheets/sheets` | excluded | reference sheets of real bosses | 39.6 MB |  |
| `WF/builds/tools-titan-sheets` | partly included | research or tools study: rules (*.md) and scripts kept; data, pictures, models out (13 files kept, 2029 out: data or picture (derived from the game caches or run output) 2028; run log of one author session (127 KB) 1) | 137.5 MB | `studies/tools-titan-sheets` |
| `WF/builds/trial-astra-table` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 873.4 KB |  |
| `WF/builds/trial-opus-chest-2677` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 24.9 MB |  |
| `WF/builds/umbral-helm-a` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 255.9 KB |  |
| `WF/builds/umbral-helm-b` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 235.7 KB |  |
| `WF/builds/umbral-helm-c` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 280.2 KB |  |
| `WF/builds/warden-set` | excluded | asset build: our model (original) or a rebuild of a real asset; not needed to run the process | 76.8 KB |  |
| `WF/builds/weapon-crossbow-33460` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 986.1 KB |  |
| `WF/builds/weapon-fang-33174` | excluded | asset build: rebuild of a real asset (model, pictures, sheets); not allowed | 1.6 MB |  |

## Working folder: night/ (prompts)

| Source | Decision | Rule | Size | In the package |
|---|---|---|---|---|
| `WF/night/arena1` | excluded | Producer run folder (records: COUNT, STATE, PROPOSALS, findings, boards and run scripts) | 970.5 MB |  |
| `WF/night/armoury` | excluded | Producer run folder (records: COUNT, STATE, PROPOSALS, findings, boards and run scripts) | 1.3 GB |  |
| `WF/night/armoury2` | excluded | Producer run folder (records: COUNT, STATE, PROPOSALS, findings, boards and run scripts) | 891.5 MB |  |
| `WF/night/fix_caps.py` | excluded | script that wrote prompts or state for one night run | 2.3 KB |  |
| `WF/night/make_wg_prompts.py` | excluded | script that wrote prompts or state for one night run | 7.7 KB |  |
| `WF/night/obs1` | excluded | Producer run folder (records: COUNT, STATE, PROPOSALS, findings, boards and run scripts) | 240.1 MB |  |
| `WF/night/obs2` | excluded | Producer run folder (records: COUNT, STATE, PROPOSALS, findings, boards and run scripts) | 334.4 MB |  |
| `WF/night/obs3` | excluded | Producer run folder (records: COUNT, STATE, PROPOSALS, findings, boards and run scripts) | 615.6 MB |  |
| `WF/night/prompt-anim-2.txt` | included | prompt, as a template | 1.5 KB | `prompts/prompt-anim-2.txt` |
| `WF/night/prompt-anim-hound-set.txt` | included | prompt, as a template | 3.9 KB | `prompts/prompt-anim-hound-set.txt` |
| `WF/night/prompt-anim-hound.txt` | included | prompt, as a template | 2.7 KB | `prompts/prompt-anim-hound.txt` |
| `WF/night/prompt-anim-warden-general.txt` | included | prompt, as a template | 3.9 KB | `prompts/prompt-anim-warden-general.txt` |
| `WF/night/prompt-boss-original.txt` | included | prompt, as a template | 7.4 KB | `prompts/prompt-boss-original.txt` |
| `WF/night/prompt-boss-pitch-a.txt` | included | prompt, as a template | 1011 B | `prompts/prompt-boss-pitch-a.txt` |
| `WF/night/prompt-boss-pitch-b.txt` | included | prompt, as a template | 987 B | `prompts/prompt-boss-pitch-b.txt` |
| `WF/night/prompt-boss-pitch-c.txt` | included | prompt, as a template | 986 B | `prompts/prompt-boss-pitch-c.txt` |
| `WF/night/prompt-boss-pitch-d.txt` | included | prompt, as a template | 984 B | `prompts/prompt-boss-pitch-d.txt` |
| `WF/night/prompt-boss-pitch-e.txt` | included | prompt, as a template | 990 B | `prompts/prompt-boss-pitch-e.txt` |
| `WF/night/prompt-boss-pitch-f.txt` | included | prompt, as a template | 992 B | `prompts/prompt-boss-pitch-f.txt` |
| `WF/night/prompt-boss-pitch-g.txt` | included | prompt, as a template | 1.3 KB | `prompts/prompt-boss-pitch-g.txt` |
| `WF/night/prompt-boss-pitch-h.txt` | included | prompt, as a template | 1.2 KB | `prompts/prompt-boss-pitch-h.txt` |
| `WF/night/prompt-boss-pitch-i.txt` | included | prompt, as a template | 1.2 KB | `prompts/prompt-boss-pitch-i.txt` |
| `WF/night/prompt-boss-pitch-j.txt` | included | prompt, as a template | 1.2 KB | `prompts/prompt-boss-pitch-j.txt` |
| `WF/night/prompt-boss-pitch-k.txt` | included | prompt, as a template | 1.2 KB | `prompts/prompt-boss-pitch-k.txt` |
| `WF/night/prompt-boss-rebuild-nex.txt` | included | prompt, as a template | 3.2 KB | `prompts/prompt-boss-rebuild-nex.txt` |
| `WF/night/prompt-boss-rebuild-tekton.txt` | included | prompt, as a template | 2.7 KB | `prompts/prompt-boss-rebuild-tekton.txt` |
| `WF/night/prompt-boss-return.txt` | included | prompt, as a template | 3.8 KB | `prompts/prompt-boss-return.txt` |
| `WF/night/prompt-boss-warden-general-4.txt` | included | prompt, as a template | 3.5 KB | `prompts/prompt-boss-warden-general-4.txt` |
| `WF/night/prompt-boss.txt` | included | prompt, as a template | 3.1 KB | `prompts/prompt-boss.txt` |
| `WF/night/prompt-cape.txt` | included | prompt, as a template | 4.2 KB | `prompts/prompt-cape.txt` |
| `WF/night/prompt-creature-2.txt` | included | prompt, as a template | 1.6 KB | `prompts/prompt-creature-2.txt` |
| `WF/night/prompt-creature-rebuild-11068.txt` | included | prompt, as a template | 1.6 KB | `prompts/prompt-creature-rebuild-11068.txt` |
| `WF/night/prompt-creature-rebuild-11175.txt` | included | prompt, as a template | 1.6 KB | `prompts/prompt-creature-rebuild-11175.txt` |
| `WF/night/prompt-creature-rebuild-12812.txt` | included | prompt, as a template | 1.6 KB | `prompts/prompt-creature-rebuild-12812.txt` |
| `WF/night/prompt-creature-rebuild-TEMPLATE.txt` | included | prompt, as a template | 1.6 KB | `prompts/prompt-creature-rebuild-TEMPLATE.txt` |
| `WF/night/prompt-creature-star-hound-4.txt` | included | prompt, as a template | 3.5 KB | `prompts/prompt-creature-star-hound-4.txt` |
| `WF/night/prompt-creature-stone-watcher-4.txt` | included | prompt, as a template | 3.5 KB | `prompts/prompt-creature-stone-watcher-4.txt` |
| `WF/night/prompt-creature-v2.txt` | included | prompt, as a template | 2.1 KB | `prompts/prompt-creature-v2.txt` |
| `WF/night/prompt-creature-watcher.txt` | included | prompt, as a template | 2.4 KB | `prompts/prompt-creature-watcher.txt` |
| `WF/night/prompt-effect-rebuild-1595.txt` | included | prompt, as a template | 1.8 KB | `prompts/prompt-effect-rebuild-1595.txt` |
| `WF/night/prompt-effect-rebuild-2008.txt` | included | prompt, as a template | 1.8 KB | `prompts/prompt-effect-rebuild-2008.txt` |
| `WF/night/prompt-effect-rebuild-2206.txt` | included | prompt, as a template | 1.8 KB | `prompts/prompt-effect-rebuild-2206.txt` |
| `WF/night/prompt-effect-rebuild-TEMPLATE.txt` | included | prompt, as a template | 1.7 KB | `prompts/prompt-effect-rebuild-TEMPLATE.txt` |
| `WF/night/prompt-effects-watcher.txt` | included | prompt, as a template | 3.5 KB | `prompts/prompt-effects-watcher.txt` |
| `WF/night/prompt-female-set-v5.txt` | included | prompt, as a template | 5.1 KB | `prompts/prompt-female-set-v5.txt` |
| `WF/night/prompt-female-set.txt` | included | prompt, as a template | 4.6 KB | `prompts/prompt-female-set.txt` |
| `WF/night/prompt-fx-warden-general.txt` | included | prompt, as a template | 4.5 KB | `prompts/prompt-fx-warden-general.txt` |
| `WF/night/prompt-hd-rebuild-diving-return.txt` | included | prompt, as a template | 2.0 KB | `prompts/prompt-hd-rebuild-diving-return.txt` |
| `WF/night/prompt-hd-rebuild-diving-return2.txt` | included | prompt, as a template | 1.7 KB | `prompts/prompt-hd-rebuild-diving-return2.txt` |
| `WF/night/prompt-hd-rebuild-diving.txt` | included | prompt, as a template | 3.5 KB | `prompts/prompt-hd-rebuild-diving.txt` |
| `WF/night/prompt-hd-rebuild-lampstall-return.txt` | included | prompt, as a template | 1.5 KB | `prompts/prompt-hd-rebuild-lampstall-return.txt` |
| `WF/night/prompt-hd-rebuild-lampstall-return2.txt` | included | prompt, as a template | 1.8 KB | `prompts/prompt-hd-rebuild-lampstall-return2.txt` |
| `WF/night/prompt-hd-rebuild-lampstall.txt` | included | prompt, as a template | 3.4 KB | `prompts/prompt-hd-rebuild-lampstall.txt` |
| `WF/night/prompt-hd-rebuild-spot2768-return.txt` | included | prompt, as a template | 1.5 KB | `prompts/prompt-hd-rebuild-spot2768-return.txt` |
| `WF/night/prompt-hd-rebuild-spot2768-return2.txt` | included | prompt, as a template | 1.9 KB | `prompts/prompt-hd-rebuild-spot2768-return2.txt` |
| `WF/night/prompt-hd-rebuild-spot2768.txt` | included | prompt, as a template | 3.5 KB | `prompts/prompt-hd-rebuild-spot2768.txt` |
| `WF/night/prompt-helm-rebuild-12931.txt` | included | prompt, as a template | 2.7 KB | `prompts/prompt-helm-rebuild-12931.txt` |
| `WF/night/prompt-helm-rebuild-22326.txt` | included | prompt, as a template | 2.8 KB | `prompts/prompt-helm-rebuild-22326.txt` |
| `WF/night/prompt-helm-rebuild-24271.txt` | included | prompt, as a template | 2.7 KB | `prompts/prompt-helm-rebuild-24271.txt` |
| `WF/night/prompt-helm-rebuild-26382.txt` | included | prompt, as a template | 2.8 KB | `prompts/prompt-helm-rebuild-26382.txt` |
| `WF/night/prompt-helm-rebuild-30750.txt` | included | prompt, as a template | 2.8 KB | `prompts/prompt-helm-rebuild-30750.txt` |
| `WF/night/prompt-helm-rebuild-TEMPLATE.txt` | included | prompt, as a template | 2.7 KB | `prompts/prompt-helm-rebuild-TEMPLATE.txt` |
| `WF/night/prompt-helm-v4-a.txt` | included | prompt, as a template | 5.5 KB | `prompts/prompt-helm-v4-a.txt` |
| `WF/night/prompt-helm-v4-b.txt` | included | prompt, as a template | 5.5 KB | `prompts/prompt-helm-v4-b.txt` |
| `WF/night/prompt-helm-v4-c.txt` | included | prompt, as a template | 5.8 KB | `prompts/prompt-helm-v4-c.txt` |
| `WF/night/prompt-helm-v4-TEMPLATE.txt` | included | prompt, as a template | 5.1 KB | `prompts/prompt-helm-v4-TEMPLATE.txt` |
| `WF/night/prompt-helm-v5-astra.txt` | included | prompt, as a template | 5.6 KB | `prompts/prompt-helm-v5-astra.txt` |
| `WF/night/prompt-helm-v5-opus.txt` | included | prompt, as a template | 5.6 KB | `prompts/prompt-helm-v5-opus.txt` |
| `WF/night/prompt-helm-v5.txt` | included | prompt, as a template | 5.6 KB | `prompts/prompt-helm-v5.txt` |
| `WF/night/prompt-hound-2.txt` | included | prompt, as a template | 4.4 KB | `prompts/prompt-hound-2.txt` |
| `WF/night/prompt-iface-original-o1.txt` | included | prompt, as a template | 1.3 KB | `prompts/prompt-iface-original-o1.txt` |
| `WF/night/prompt-iface-rebuild-156.txt` | included | prompt, as a template | 2.1 KB | `prompts/prompt-iface-rebuild-156.txt` |
| `WF/night/prompt-iface-rebuild-206.txt` | included | prompt, as a template | 2.1 KB | `prompts/prompt-iface-rebuild-206.txt` |
| `WF/night/prompt-iface-rebuild-302.txt` | included | prompt, as a template | 2.1 KB | `prompts/prompt-iface-rebuild-302.txt` |
| `WF/night/prompt-iface-rebuild-303.txt` | included | prompt, as a template | 1.7 KB | `prompts/prompt-iface-rebuild-303.txt` |
| `WF/night/prompt-iface-rebuild-383.txt` | included | prompt, as a template | 1.7 KB | `prompts/prompt-iface-rebuild-383.txt` |
| `WF/night/prompt-iface-rebuild-4.txt` | included | prompt, as a template | 2.1 KB | `prompts/prompt-iface-rebuild-4.txt` |
| `WF/night/prompt-iface-rebuild-422.txt` | included | prompt, as a template | 2.1 KB | `prompts/prompt-iface-rebuild-422.txt` |
| `WF/night/prompt-iface-rebuild-648.txt` | included | prompt, as a template | 1.7 KB | `prompts/prompt-iface-rebuild-648.txt` |
| `WF/night/prompt-iface-rebuild-868.txt` | included | prompt, as a template | 2.1 KB | `prompts/prompt-iface-rebuild-868.txt` |
| `WF/night/prompt-mace-a-return.txt` | included | prompt, as a template | 2.5 KB | `prompts/prompt-mace-a-return.txt` |
| `WF/night/prompt-mace-a.txt` | included | prompt, as a template | 1.9 KB | `prompts/prompt-mace-a.txt` |
| `WF/night/prompt-mace-b.txt` | included | prompt, as a template | 2.0 KB | `prompts/prompt-mace-b.txt` |
| `WF/night/prompt-object-rebuild-11700.txt` | included | prompt, as a template | 2.0 KB | `prompts/prompt-object-rebuild-11700.txt` |
| `WF/night/prompt-object-rebuild-33035.txt` | included | prompt, as a template | 2.0 KB | `prompts/prompt-object-rebuild-33035.txt` |
| `WF/night/prompt-object-rebuild-51046.txt` | included | prompt, as a template | 2.1 KB | `prompts/prompt-object-rebuild-51046.txt` |
| `WF/night/prompt-object-rebuild-TEMPLATE.txt` | included | prompt, as a template | 2.0 KB | `prompts/prompt-object-rebuild-TEMPLATE.txt` |
| `WF/night/prompt-player-boots.txt` | included | prompt, as a template | 4.5 KB | `prompts/prompt-player-boots.txt` |
| `WF/night/prompt-player-cuirass.txt` | included | prompt, as a template | 4.6 KB | `prompts/prompt-player-cuirass.txt` |
| `WF/night/prompt-player-gauntlets.txt` | included | prompt, as a template | 4.5 KB | `prompts/prompt-player-gauntlets.txt` |
| `WF/night/prompt-player-legs.txt` | included | prompt, as a template | 4.6 KB | `prompts/prompt-player-legs.txt` |
| `WF/night/prompt-player-PIECE.txt` | included | prompt, as a template | 4.3 KB | `prompts/prompt-player-PIECE.txt` |
| `WF/night/prompt-producer-arena1-return.txt` | included | prompt, as a template | 4.0 KB | `prompts/prompt-producer-arena1-return.txt` |
| `WF/night/prompt-producer-arena1.txt` | included | prompt, as a template | 6.0 KB | `prompts/prompt-producer-arena1.txt` |
| `WF/night/prompt-producer-armoury.txt` | included | prompt, as a template | 5.3 KB | `prompts/prompt-producer-armoury.txt` |
| `WF/night/prompt-producer-armoury2.txt` | included | prompt, as a template | 5.4 KB | `prompts/prompt-producer-armoury2.txt` |
| `WF/night/prompt-producer-armoury2b.txt` | included | prompt, as a template | 8.2 KB | `prompts/prompt-producer-armoury2b.txt` |
| `WF/night/prompt-producer-obs1.txt` | included | prompt, as a template | 3.5 KB | `prompts/prompt-producer-obs1.txt` |
| `WF/night/prompt-producer-obs2.txt` | included | prompt, as a template | 3.5 KB | `prompts/prompt-producer-obs2.txt` |
| `WF/night/prompt-producer-obs3.txt` | included | prompt, as a template | 3.9 KB | `prompts/prompt-producer-obs3.txt` |
| `WF/night/prompt-qual-legs.txt` | included | prompt, as a template | 1.7 KB | `prompts/prompt-qual-legs.txt` |
| `WF/night/prompt-qual-platebody.txt` | included | prompt, as a template | 2.0 KB | `prompts/prompt-qual-platebody.txt` |
| `WF/night/prompt-research-animation-astra.txt` | included | prompt, as a template | 4.5 KB | `prompts/prompt-research-animation-astra.txt` |
| `WF/night/prompt-research-items-astra.txt` | included | prompt, as a template | 4.9 KB | `prompts/prompt-research-items-astra.txt` |
| `WF/night/prompt-room-observatory.txt` | included | prompt, as a template | 4.1 KB | `prompts/prompt-room-observatory.txt` |
| `WF/night/prompt-set-v5-boots.txt` | included | prompt, as a template | 5.4 KB | `prompts/prompt-set-v5-boots.txt` |
| `WF/night/prompt-set-v5-cuirass.txt` | included | prompt, as a template | 5.4 KB | `prompts/prompt-set-v5-cuirass.txt` |
| `WF/night/prompt-set-v5-gauntlets.txt` | included | prompt, as a template | 5.5 KB | `prompts/prompt-set-v5-gauntlets.txt` |
| `WF/night/prompt-set-v5-legs.txt` | included | prompt, as a template | 5.4 KB | `prompts/prompt-set-v5-legs.txt` |
| `WF/night/prompt-set-v5-TEMPLATE.txt` | included | prompt, as a template | 5.3 KB | `prompts/prompt-set-v5-TEMPLATE.txt` |
| `WF/night/prompt-tekton-marks.txt` | included | prompt, as a template | 4.1 KB | `prompts/prompt-tekton-marks.txt` |
| `WF/night/prompt-tekton-plates.txt` | included | prompt, as a template | 3.7 KB | `prompts/prompt-tekton-plates.txt` |
| `WF/night/prompt-tekton-return.txt` | included | prompt, as a template | 3.2 KB | `prompts/prompt-tekton-return.txt` |
| `WF/night/prompt-titan-rebuild-12596-a-return.txt` | included | prompt, as a template | 1.7 KB | `prompts/prompt-titan-rebuild-12596-a-return.txt` |
| `WF/night/prompt-titan-rebuild-12596-a-return2.txt` | included | prompt, as a template | 1.4 KB | `prompts/prompt-titan-rebuild-12596-a-return2.txt` |
| `WF/night/prompt-titan-rebuild-12596-a.txt` | included | prompt, as a template | 4.6 KB | `prompts/prompt-titan-rebuild-12596-a.txt` |
| `WF/night/prompt-titan-rebuild-12596-b-return.txt` | included | prompt, as a template | 1.8 KB | `prompts/prompt-titan-rebuild-12596-b-return.txt` |
| `WF/night/prompt-titan-rebuild-12596-b-return2.txt` | included | prompt, as a template | 1.3 KB | `prompts/prompt-titan-rebuild-12596-b-return2.txt` |
| `WF/night/prompt-titan-rebuild-12596-b.txt` | included | prompt, as a template | 4.6 KB | `prompts/prompt-titan-rebuild-12596-b.txt` |
| `WF/night/prompt-titan-rebuild-14147-a-return.txt` | included | prompt, as a template | 1.7 KB | `prompts/prompt-titan-rebuild-14147-a-return.txt` |
| `WF/night/prompt-titan-rebuild-14147-a-return2.txt` | included | prompt, as a template | 1.1 KB | `prompts/prompt-titan-rebuild-14147-a-return2.txt` |
| `WF/night/prompt-titan-rebuild-14147-a.txt` | included | prompt, as a template | 4.6 KB | `prompts/prompt-titan-rebuild-14147-a.txt` |
| `WF/night/prompt-titan-rebuild-14147-b-return.txt` | included | prompt, as a template | 1.6 KB | `prompts/prompt-titan-rebuild-14147-b-return.txt` |
| `WF/night/prompt-titan-rebuild-14147-b-return2.txt` | included | prompt, as a template | 1.3 KB | `prompts/prompt-titan-rebuild-14147-b-return2.txt` |
| `WF/night/prompt-titan-rebuild-14147-b.txt` | included | prompt, as a template | 4.6 KB | `prompts/prompt-titan-rebuild-14147-b.txt` |
| `WF/night/prompt-tools-fight-in-arena.txt` | included | prompt, as a template | 4.0 KB | `prompts/prompt-tools-fight-in-arena.txt` |
| `WF/night/prompt-tools-hd-era.txt` | included | prompt, as a template | 4.6 KB | `prompts/prompt-tools-hd-era.txt` |
| `WF/night/prompt-tools-hd-gate.txt` | included | prompt, as a template | 4.7 KB | `prompts/prompt-tools-hd-gate.txt` |
| `WF/night/prompt-tools-real-fight.txt` | included | prompt, as a template | 3.4 KB | `prompts/prompt-tools-real-fight.txt` |
| `WF/night/prompt-tools-render-quality.txt` | included | prompt, as a template | 5.7 KB | `prompts/prompt-tools-render-quality.txt` |
| `WF/night/prompt-tools-rs3-models.txt` | included | prompt, as a template | 4.2 KB | `prompts/prompt-tools-rs3-models.txt` |
| `WF/night/prompt-tools-rs3.txt` | included | prompt, as a template | 3.9 KB | `prompts/prompt-tools-rs3.txt` |
| `WF/night/prompt-tools-textures-alpha.txt` | included | prompt, as a template | 4.6 KB | `prompts/prompt-tools-textures-alpha.txt` |
| `WF/night/prompt-tools-titan-sheets.txt` | included | prompt, as a template | 4.9 KB | `prompts/prompt-tools-titan-sheets.txt` |
| `WF/night/prompt-tools3.txt` | included | prompt, as a template | 4.0 KB | `prompts/prompt-tools3.txt` |
| `WF/night/prompt-umbral-helm-a.txt` | included | prompt, as a template | 4.6 KB | `prompts/prompt-umbral-helm-a.txt` |
| `WF/night/prompt-umbral-helm-b.txt` | included | prompt, as a template | 4.7 KB | `prompts/prompt-umbral-helm-b.txt` |
| `WF/night/prompt-umbral-helm-c.txt` | included | prompt, as a template | 5.0 KB | `prompts/prompt-umbral-helm-c.txt` |
| `WF/night/prompt-watcher-3.txt` | included | prompt, as a template | 5.2 KB | `prompts/prompt-watcher-3.txt` |
| `WF/night/prompt-watcher-return.txt` | included | prompt, as a template | 5.3 KB | `prompts/prompt-watcher-return.txt` |
| `WF/night/rec70.py` | excluded | script that wrote prompts or state for one night run | 2.1 KB |  |
| `WF/night/rec71.py` | excluded | script that wrote prompts or state for one night run | 2.3 KB |  |
| `WF/night/rec72.py` | excluded | script that wrote prompts or state for one night run | 2.6 KB |  |
| `WF/night/update_state_s18.py` | excluded | script that wrote prompts or state for one night run | 3.8 KB |  |
| `WF/night/walldecor` | excluded | Producer run folder (records: COUNT, STATE, PROPOSALS, findings, boards and run scripts) | 72.6 MB |  |

## Working folder: compare-authors/ and trial/ (prompts)

| Source | Decision | Rule | Size | In the package |
|---|---|---|---|---|
| `WF/compare-authors/followup-tools.txt` | included | prompt, as a template | 2.3 KB | `prompts/authors/followup-tools.txt` |
| `WF/compare-authors/prompt-astra.txt` | included | prompt, as a template | 1006 B | `prompts/authors/prompt-astra.txt` |
| `WF/compare-authors/prompt-orig-astra.txt` | included | prompt, as a template | 1.2 KB | `prompts/authors/prompt-orig-astra.txt` |
| `WF/compare-authors/prompt-orig-sol.txt` | included | prompt, as a template | 1.2 KB | `prompts/authors/prompt-orig-sol.txt` |
| `WF/compare-authors/prompt-qual-chainbody-astra.txt` | included | prompt, as a template | 1.6 KB | `prompts/authors/prompt-qual-chainbody-astra.txt` |
| `WF/compare-authors/prompt-qual-fang-astra.txt` | included | prompt, as a template | 1.2 KB | `prompts/authors/prompt-qual-fang-astra.txt` |
| `WF/compare-authors/prompt-sol.txt` | included | prompt, as a template | 994 B | `prompts/authors/prompt-sol.txt` |
| `WF/compare-authors/prompt-tools-astra.txt` | included | prompt, as a template | 5.3 KB | `prompts/authors/prompt-tools-astra.txt` |
| `WF/compare-authors/prompt-tools2-astra.txt` | included | prompt, as a template | 3.0 KB | `prompts/authors/prompt-tools2-astra.txt` |
| `WF/trial/astra` | excluded | a Producer trial run (records, builds) | 172.3 MB |  |
| `WF/trial/opus` | excluded | a Producer trial run (records, builds) | 4.1 MB |  |
| `WF/trial/prompt-producer-astra-2.txt` | included | prompt, as a template | 2.4 KB | `prompts/producers/prompt-producer-astra-2.txt` |
| `WF/trial/prompt-producer-astra.txt` | included | prompt, as a template | 3.8 KB | `prompts/producers/prompt-producer-astra.txt` |
| `WF/trial/prompt-producer-opus.txt` | included | prompt, as a template | 3.8 KB | `prompts/producers/prompt-producer-opus.txt` |

## Working folder: sittings/

| Source | Decision | Rule | Size | In the package |
|---|---|---|---|---|
| `WF/sittings/2026-11-25-a.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 3.5 KB |  |
| `WF/sittings/2026-11-25-b.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 3.6 KB |  |
| `WF/sittings/2026-11-26-a.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 2.3 KB |  |
| `WF/sittings/2026-11-26-b.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 3.0 KB |  |
| `WF/sittings/2026-11-26-c.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 2.3 KB |  |
| `WF/sittings/2026-11-26-d.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 2.3 KB |  |
| `WF/sittings/2026-11-26-e.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 3.1 KB |  |
| `WF/sittings/2026-11-26-f.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 880 B |  |
| `WF/sittings/2026-11-26-g.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 3.1 KB |  |
| `WF/sittings/2026-11-27-a.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 17.0 KB |  |
| `WF/sittings/2026-11-27-b.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 4.5 KB |  |
| `WF/sittings/2026-11-27-c.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 1.7 KB |  |
| `WF/sittings/2026-11-28-a.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 3.9 KB |  |
| `WF/sittings/2026-11-28-b.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 4.5 KB |  |
| `WF/sittings/2026-11-29-a.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 3.7 KB |  |
| `WF/sittings/2026-11-29-b.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 3.4 KB |  |
| `WF/sittings/2026-11-29-c.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 2.5 KB |  |
| `WF/sittings/2026-11-30-a.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 6.8 KB |  |
| `WF/sittings/2026-11-30-b.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 4.3 KB |  |
| `WF/sittings/2026-11-30-c.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 6.6 KB |  |
| `WF/sittings/2026-11-30-d.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 4.4 KB |  |
| `WF/sittings/2026-11-30-e.json` | excluded | sitting spec of the project (owner-facing questions, pictures of real assets); format kept as a sample | 5.3 KB |  |
| `WF/sittings/board-hud-o1.png` | excluded | picture of a sitting or board (shows real assets) | 22.2 KB |  |
| `WF/sittings/board_encounter.py` | excluded | one-off board script of one sitting | 1.2 KB |  |
| `WF/sittings/board_hud_o1.py` | excluded | one-off board script of one sitting | 1.7 KB |  |
| `WF/sittings/build_2026_11_27_a.py` | excluded | one-off board script of one sitting | 11.5 KB |  |
| `WF/sittings/build_2026_11_27_b.py` | excluded | one-off board script of one sitting | 4.0 KB |  |
| `WF/sittings/encounter-side-by-side.gif` | excluded | picture of a sitting or board (shows real assets) | 29.6 MB |  |
| `WF/sittings/page-check-2026-11-30-b-full.png` | excluded | picture of a sitting or board (shows real assets) | 4.5 MB |  |
| `WF/sittings/page-check-2026-11-30-b.png` | excluded | picture of a sitting or board (shows real assets) | 1.8 MB |  |
| `WF/sittings/page-check-2026-11-30-c.png` | excluded | picture of a sitting or board (shows real assets) | 2.4 MB |  |
| `WF/sittings/page-check-2026-11-30-d.png` | excluded | picture of a sitting or board (shows real assets) | 360.3 KB |  |
| `WF/sittings/page-check-2026-11-30-e.png` | excluded | picture of a sitting or board (shows real assets) | 209.5 KB |  |
| `WF/sittings/page-check-b.png` | excluded | picture of a sitting or board (shows real assets) | 319.6 KB |  |
| `WF/sittings/page-check-c-small.png` | excluded | picture of a sitting or board (shows real assets) | 538.1 KB |  |
| `WF/sittings/page-check-lower.png` | excluded | picture of a sitting or board (shows real assets) | 970.4 KB |  |
| `WF/sittings/page-check.png` | excluded | picture of a sitting or board (shows real assets) | 272.5 KB |  |
| `WF/sittings/phase1` | excluded | sitting pictures (real assets beside ours) | 4.5 MB |  |
| `WF/sittings/phase1-hq` | excluded | sitting pictures (real assets beside ours) | 1.7 MB |  |
| `WF/sittings/phase2-branda` | excluded | sitting pictures (real assets beside ours) | 2.2 MB |  |
| `WF/sittings/pitch-boards-all.png` | excluded | picture of a sitting or board (shows real assets) | 936.5 KB |  |
| `WF/sittings/pitch-boards-round2.png` | excluded | picture of a sitting or board (shows real assets) | 603.1 KB |  |

## Working folder: client-build/

| Source | Decision | Rule | Size | In the package |
|---|---|---|---|---|
| `WF/client-build/rust-client/target` | excluded | build output | 561.5 MB |  |
| `WF/client-build/rust-client/tools/native-assets/__pycache__` | excluded | Python cache | 15.2 KB |  |
| `WF/client-build/rust-client` | included | client source (owner instruction): source, examples, vendored crates, Cargo.toml/lock; no target/ (1579 files) | 13.7 MB | `client/rust-client` |

## Working folder: other top-level items

| Source | Decision | Rule | Size | In the package |
|---|---|---|---|---|
| `WF/FINDINGS.md` | excluded | owner record (findings with owner verdicts); measured rules it holds are in the studies' *.md | 179.5 KB |  |
| `WF/keys` | excluded | answer keys of blind line-ups (records) | 1.5 KB |  |
| `WF/measure` | excluded | measurements derived from the cache (regenerate with tools/measure.py, tools/decor_bands.py, tools/weapon_card.py) | 2.5 MB |  |
| `WF/real-in-blender` | excluded | real meshes dumped into Blender files | 126.1 KB |  |
| `WF/refs` | excluded | reference pictures of real assets | 222.0 KB |  |
| `WF/render` | excluded | renders (pictures of real and our assets) | 11.5 GB |  |
| `WF/render-prune-plan.txt` | excluded | housekeeping list of one cleanup | 105.7 KB |  |
| `WF/reviewer-trial-01` | excluded | pictures of a reviewer trial | 4.9 MB |  |
| `WF/rust-setup` | excluded | third-party installer (rustup-init.exe): get it from rustup.rs | 12.2 MB |  |
| `WF/tsps-scratch` | excluded | a scratch TSPS copy and its logs | 2.6 GB |  |
| `WF/board-*` | excluded | boards: pictures of real assets beside ours, and their answer keys (77 files) | 44.7 MB |  |

## Project repository: teacher/

| Source | Decision | Rule | Size | In the package |
|---|---|---|---|---|
| `PR/teacher/check.sh` | included | the Teacher role | 5.8 KB | `teacher/check.sh` |
| `PR/teacher/core/decisions.md` | included | the Teacher role | 8.7 KB | `teacher/core/decisions.md` |
| `PR/teacher/core/evidence.md` | included | the Teacher role | 7.3 KB | `teacher/core/evidence.md` |
| `PR/teacher/core/failure-modes.md` | included | the Teacher role | 8.7 KB | `teacher/core/failure-modes.md` |
| `PR/teacher/core/missions.md` | included | the Teacher role | 8.8 KB | `teacher/core/missions.md` |
| `PR/teacher/core/state-and-handoff.md` | included | the Teacher role | 6.5 KB | `teacher/core/state-and-handoff.md` |
| `PR/teacher/core/teaching.md` | included | the Teacher role | 3.3 KB | `teacher/core/teaching.md` |
| `PR/teacher/exam/ANSWERS.md` | included | the Teacher role | 15.4 KB | `teacher/exam/ANSWERS.md` |
| `PR/teacher/exam/QUESTIONS.md` | included | the Teacher role | 9.8 KB | `teacher/exam/QUESTIONS.md` |
| `PR/teacher/exam/README.md` | included | the Teacher role | 2.8 KB | `teacher/exam/README.md` |
| `PR/teacher/playbooks/build-tool.md` | included | the Teacher role | 3.9 KB | `teacher/playbooks/build-tool.md` |
| `PR/teacher/playbooks/cold-start-test.md` | included | the Teacher role | 3.4 KB | `teacher/playbooks/cold-start-test.md` |
| `PR/teacher/playbooks/extract-method.md` | included | the Teacher role | 4.4 KB | `teacher/playbooks/extract-method.md` |
| `PR/teacher/playbooks/port-kit.md` | included | the Teacher role | 4.1 KB | `teacher/playbooks/port-kit.md` |
| `PR/teacher/playbooks/README.md` | included | the Teacher role | 2.8 KB | `teacher/playbooks/README.md` |
| `PR/teacher/playbooks/review-port.md` | included | the Teacher role | 3.9 KB | `teacher/playbooks/review-port.md` |
| `PR/teacher/playbooks/save-before-delete.md` | included | the Teacher role | 5.5 KB | `teacher/playbooks/save-before-delete.md` |
| `PR/teacher/playbooks/select-task.md` | included | the Teacher role | 4.5 KB | `teacher/playbooks/select-task.md` |
| `PR/teacher/playbooks/survey-machine.md` | included | the Teacher role | 4.1 KB | `teacher/playbooks/survey-machine.md` |
| `PR/teacher/playbooks/task-pack.md` | included | the Teacher role | 4.3 KB | `teacher/playbooks/task-pack.md` |
| `PR/teacher/README.md` | included | the Teacher role | 6.3 KB | `teacher/README.md` |
| `PR/teacher/SKILL.md` | included | the Teacher role | 9.6 KB | `teacher/SKILL.md` |
| `PR/teacher/templates/decision-record.md` | included | the Teacher role | 1.6 KB | `teacher/templates/decision-record.md` |
| `PR/teacher/templates/mission-brief.md` | included | the Teacher role | 3.1 KB | `teacher/templates/mission-brief.md` |
| `PR/teacher/templates/owner-update.md` | included | the Teacher role | 956 B | `teacher/templates/owner-update.md` |
| `PR/teacher/templates/report-review.md` | included | the Teacher role | 1.7 KB | `teacher/templates/report-review.md` |
| `PR/teacher/project` | excluded | the owner's private record (profile, state, decisions, plans, handoffs) | 154.1 KB |  |

## Project repository: tools/

| Source | Decision | Rule | Size | In the package |
|---|---|---|---|---|
| `PR/tools/make_sitting.py` | included | sitting tool | 8.2 KB | `sittings/make_sitting.py` |
| `PR/tools/collect_sitting.py` | included | sitting tool | 1.4 KB | `sittings/collect_sitting.py` |
| `PR/tools/collect_panel_verdict.py` | included | sitting tool | 2.0 KB | `sittings/collect_panel_verdict.py` |
| `PR/tools/stamp_state.sh` | included | repo script: stamps STATE.md time | 313 B | `teacher/tools/stamp_state.sh` |
| `PR/tools/sync_asset_system.sh` | excluded | mirrors the private working copy into the private repository | 2.4 KB |  |
| `PR/tools/verify_asset_system.sh` | excluded | mirrors the private working copy into the private repository | 763 B |  |

## Project repository: other items

| Source | Decision | Rule | Size | In the package |
|---|---|---|---|---|
| `PR/.claude` | excluded | private repository front matter | 819 B |  |
| `PR/.gitattributes` | excluded | private repository front matter | 116 B |  |
| `PR/.gitignore` | excluded | private repository front matter | 62 B |  |
| `PR/AGENTS.md` | excluded | private repository front matter | 2.8 KB |  |
| `PR/asset-system` | excluded | copy of the working folder (staged from the working folder itself) | 40.6 MB |  |
| `PR/docs/verdicts` | excluded | owner's verdicts and COUNT.md (owner records) | 67.1 KB |  |
| `PR/docs/knowledge/OSRS-ASSETS-METHOD.md` | excluded | project knowledge base: project history and owner statements (method restated in PROCESS.md) | 20.6 KB |  |
| `PR/docs/knowledge/OSRS-ASSETS-SURVEY.md` | excluded | project knowledge base: project history and owner statements (method restated in PROCESS.md) | 22.3 KB |  |
| `PR/docs/knowledge/README.md` | excluded | project knowledge base: project history and owner statements (method restated in PROCESS.md) | 904 B |  |
| `PR/docs/lobby-coverage.md` | excluded | review of the project with the owner (record) | 5.6 KB |  |
| `PR/docs/panel.md` | excluded | review of the project with the owner (record) | 968 B |  |
| `PR/docs/review-of-the-demo-2026-11-25.md` | excluded | review of the project with the owner (record) | 5.6 KB |  |
| `PR/docs/teacher-decision-review.md` | excluded | review of the project with the owner (record) | 7.1 KB |  |
| `PR/docs/teacher-review-2026-11-25.md` | excluded | review of the project with the owner (record) | 5.9 KB |  |
| `PR/README.md` | excluded | private repository front matter | 1.3 KB |  |

## Executables added under client/bin (owner instruction)

| File | Built from | Size |
|---|---|---|
| `client/bin/arena_view.exe` | client example examples/arena_view.rs | 9.7 MB |
| `client/bin/asset_anim.exe` | client example examples/asset_anim.rs | 9.3 MB |
| `client/bin/asset_arena.exe` | client example examples/asset_arena.rs | 9.7 MB |
| `client/bin/asset_bone.exe` | client example examples/asset_bone.rs | 9.3 MB |
| `client/bin/asset_effect.exe` | client example examples/asset_effect.rs | 10.1 MB |
| `client/bin/asset_figure.exe` | client example examples/asset_figure.rs | 9.3 MB |
| `client/bin/asset_npc.exe` | client example examples/asset_npc.rs | 9.5 MB |
| `client/bin/asset_pose.exe` | client example examples/asset_pose.rs | 9.4 MB |
| `client/bin/asset_skel.exe` | client example examples/asset_skel.rs | 9.7 MB |
| `client/bin/authored_scene_snapshot.exe` | client example examples/authored_scene_snapshot.rs | 9.7 MB |
| `client/bin/creature_light.exe` | client example examples/creature_light.rs | 9.2 MB |
| `client/bin/iface_dump.exe` | client example examples/iface_dump.rs | 9.4 MB |
| `client/bin/interface_preview.exe` | client example examples/interface_preview.rs | 9.2 MB |
| `client/bin/live_readback.exe` | client example examples/live_readback.rs | 9.6 MB |
| `client/bin/live_scene.exe` | client example examples/live_scene.rs | 9.7 MB |
| `client/bin/live_world.exe` | client example examples/live_world.rs | 10.0 MB |
| `client/bin/meridian_ground.exe` | client example examples/meridian_ground.rs | 9.2 MB |
| `client/bin/research_items.exe` | client example examples/research_items.rs | 9.3 MB |
| `client/bin/research_pose.exe` | client example examples/research_pose.rs | 9.4 MB |
| `client/bin/arena_film.exe` | studies/tools-real-fight/arena_film.rs via build_view.py | 13.9 MB |
| `client/bin/quality_npc.exe` | studies/tools-render-quality/build_renderer.py (patched asset_npc.rs) | 13.1 MB |
| `client/bin/quality_scene.exe` | studies/tools-render-quality/build_renderer.py (patched authored_scene_snapshot.rs) | 13.4 MB |
| `client/bin/hdclassic.exe` | studies/tools-hd-era: cargo build --release (classic.rs) | 9.0 MB |
| `client/bin/sheet-render.exe` | studies/tools-hd-gate: cargo build --release (sheet_render.rs) | 9.1 MB |
| `client/bin/renderer/authored_scene_snapshot.exe` | prebuilt scene renderer used by tools/scene.py (RENDERER_BIN); built elsewhere, source commit not present here | 9.8 MB |

Left out of `client/bin`: client examples no staged tool calls (`iface_compose`, `interface_scripts`, `label_dump`,
`live_ground`, `player_models`), the main `gielinor-client.exe` (build it: `cargo build --release`), the GPU renderer
`authored_scene_snapshot_gpu.exe` (needs a GPU; not used for judging), one-off previews inside asset builds.

