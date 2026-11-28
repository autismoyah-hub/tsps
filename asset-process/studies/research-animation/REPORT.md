# New animation authoring on the revision 239 cache

**The classic authoring route works through the client's ordinary sequence loader.** Walk 819 and slash 390 survived decode, re-encode and rendering with zero differing pixels in all 72 tested cells. Five new sequence IDs then loaded from a private cache copy and reproduced their loose-file poses across 58 playback frames. The source cache, server, client source, and production tools were left untouched. No server was started and no assets were distributed.

## What the handoff already contained

The previous engineer had written a cache reader, classic codecs, sparse JSON keyframe sampler, player joint map, a two-hand grip solver, pose and measurement tools, four complete source clips, loose frames and a packed cache copy. The evidence establishes what the files can do today. It cannot establish which commands that engineer personally ran.

I opened the inherited packed copy read-only. Its definitions and every referenced frame for 20001–20004 matched the corresponding inherited loose bytes, for 11, 15, 11 and 16 playback entries respectively. All 14,410 stock sequence definitions matched the source cache. The older player sheets were development passes. In particular the file labelled [quoted words withheld] in the first crosscut draft supplied no separate yaw. The inherited cow already had comparison sheets. A 140-file hash and size inventory identifies the handoff, including the binary payloads. [Inherited checks](verification/audit.log) · [Inventory](verification/inherited-inventory.tsv)

I retained the four motion concepts. I rebuilt the evidence from decoded data, moved the rest keys to visible frames, revised the glaive grip, added a short goblin test, traced the halberd attack selection, and tested real cached skeletal playback. [Format and authoring guide](FORMAT.md)

## Round trip in pixels

The reference path used the existing release `asset_pose.exe` against the source cache. The test path ran `seqtool.py export` **without** `--raw`, then `asset_anim.exe anim=DIR` against the freshly encoded files. Both drew the same combined figure in the default actor mode. Each numbered frame was drawn bare, with item 1323, with the Meridian sword, and wearing items 1127 + 1079 + 1163. Each cell was 300 × 400 and contained over 2,000 non-background pixels. RGB pixels were compared before saving the PNGs.

| Sequence | Byte-identical exported files | Playback frames × outfits | Pixels compared | Worst cell | Total differences |
|---|---:|---:|---:|---:|---:|
| Walk 819 | 9/9 | 8 × 4 | 3,840,000 | 0 | 0 |
| Slash 390 | 11/11 | 10 × 4 | 4,800,000 | 0 | 0 |

The negative control raised group 30's x rotation by one stored unit in slash frame 1. The four cells changed by **391, 1,108, 1,709 and 341 pixels**. All other rows stayed identical. The loose frame was restored. The per-frame matrix, both command lines and their complete output are in [pixels.log](verification/pixels.log). The reference, reconstruction and control images are in [the render directory](../../render/research-animation-verified).

The full-cache test has a narrower result. Seven definitions re-encode differently and one frame group has 64 decoder failures. The successful walk and slash tests do not establish byte fidelity for every sequence. [Census output and exception IDs](verification/census.log)

## Timing rules from real actions

The sample includes two one-handed sword attacks, four polearm attacks, the Fremennik salute and the bow. The measures are local group rotations between two consecutive stored poses. For each pair the tool composes Z, X, Y rotations and measures the angle of the relative rotation. A rate divides this angle by the preceding hold. It expresses how abruptly the pose changes over the allotted time. It is not a measurement of a continuously moving joint.

| Reference | Hold lengths in 20 ms cycles | Largest single step, in degrees |
|---|---|---|
| Sword slash 390 | 4,4,4,4,8,2,1,1,8,3 | Right shoulder 34.2, elbow 62.8, hand 25.3. Left shoulder 9.9. |
| Sword stab 386 | 4,4,6,3,18,4 | Right shoulder 80.4, elbow 66.1, hand 40.8. Right-hand translation moves up to 8.5 units. |
| Halberd 440 | 5,4,4,4,5,4,16,3,13,5,4,3,5 | Right shoulder 49.2, elbow 12.7, hand 88.6. Left shoulder 63.0, elbow 65.2. |
| Fremennik salute 2112 | 4,3,2,1,24,1,2,3,4 | Right shoulder 30.9, elbow 39.6. Head 4.2, torso 1.4. |
| Bow 858 | 4,4,6,4,4,4,20,4,4,4,6,4,4 | Right shoulder 40.0, elbow 23.9. Left shoulder 6.0. |

1. **Give a sword attack at least two legible silhouettes.** In 390, four four-cycle entries lead into an eight-cycle preparation at cycle 16. The 2,1,1 passage takes only 80 ms before the next eight-cycle hold. Stab 386 spends 18 of the same 39 cycles on one extension. Start with the preparation and finish poses, then allocate the small number of transitional drawings. Turn the weapon arm and torso together. Add a small opposing shoulder and head motion where it improves the silhouette. Slash also moves root turn and cape groups. Stab uses a much narrower set.
2. **Stage both hands for a polearm.** Reference 440 brings the body round for 26 cycles, holds frame 6 for 16, crosses through a three-cycle frame and holds frame 8 for 13. Turn the torso and both shoulders, then check separation and contact in model space. The left elbow's 65.2° step is larger than the right elbow's 12.7°. Keeping the off arm in the idle pose loses this relationship. Weapon mass does not imply one fixed tempo. Spear 2080 lasts 72 cycles, swipe 2081 lasts 66, and crush 2082 uses six consecutive one-cycle entries in a 35-cycle clip.
3. **Spend emote time on the sign the player should recognize.** The salute reaches its 24-cycle hold after ten cycles and the bow holds its middle drawing for 20. Use a short approach, an unmistakable held outline, and enough recovery drawings to return cleanly. In 2112 the right elbow travels as much as 145.8° from its initial rotation while the torso barely changes. Its priority is 2, it hides held items and has no classic interleave. Both reference sword attacks and 440 use priority 6 and a 40-group interleave.
4. **Review frame holds and joint travel together.** In 390 the fastest right elbow change is 39.6° per preceding cycle. A large return step after a long held pose can still look like a pop. Inspect the frame sequence with its durations and compare the last displayed pose with the intended idle. Copy a compatible rest pose to preserve pivots and leave the feet planted unless a step is deliberately authored.

The [677-row joint table](verification/joint-steps.tsv) gives **each** adjacent-frame angle for every changing rotation group in all eight references. The [measurement transcript](verification/measurements.log) also records moving translations, moving origins, held groups and reference values.

## Final clips and visual judgment

The links show every playback frame. The first row is the original and following rows are real cache references on the same figure and equipment. Compare by the printed hold lengths. Equal column numbers need not represent equal elapsed time. The game-size preview uses a 52-pixel-tall standing player at zoom 2000. This is a representative screen size, since the live camera can zoom. Close views use a larger native render. Player draw mode is actor, NPC mode is auto.

| ID / JSON | Comparison | Game size | Close |
|---|---|---|---|
| 20001 [Meridian crosscut](verification/meridian_crosscut.json) | Meridian sword against slash 390 and stab 386 | [Sheet](../../render/research-animation-verified/meridian_crosscut-game.png) | [Sheet](../../render/research-animation-verified/meridian_crosscut-close.png) |
| 20002 [Meridian overhead](verification/meridian_fall-grip.json) | Meridian glaive against ordinary halberd 440 | [Sheet](../../render/research-animation-verified/meridian_fall-grip-game.png) | [Sheet](../../render/research-animation-verified/meridian_fall-grip-close.png) |
| 20003 [Warden's salute](verification/warden_salute.json) | Bare player against Fremennik salute 2112 | [Sheet](../../render/research-animation-verified/warden_salute-game.png) | [Sheet](../../render/research-animation-verified/warden_salute-close.png) |
| 20004 [Cow fly shake](verification/cow_flyshake.json) | Cow 2790 against graze 5854 | [Sheet](../../render/research-animation-verified/cow_flyshake-game.png) | [Sheet](../../render/research-animation-verified/cow_flyshake-close.png) |

**Crosscut.** The blade starts across the chest on the opposite side, passes through a broad lateral sweep and finishes extended to the figure's right. Slash 390 raises the blade high, and stab 386 projects it forward. The crosscut reads distinctly with the specified sword. Its wrist changes by up to 66.1° in a single step, versus 25.3° for 390. The extended hold and short return make it look snappier and somewhat wrist-led. The quiet lower body makes the attack look light. I retained that timing for a one-handed attack.

**Overhead.** The long preparation high beside the head and the low finish distinguish this from 440's horizontal sweep. The inherited free hand fell well away from its intended grip at the high hold. I turned the weapon shoulder inward by 25 stored y units at the attack keys, shortened the target interval along the haft and revised the grab/release blend. At frames 4, 5 and 6 the solver's distances to the allowed shaft segment fell from **21.1, 23.1, 22.1** model units to **0.2, 0.2, 1.4**. The two hands now read together at the top. Mid-strike frames 7–9 still score 9.0, 8.7 and 8.1. This metric includes error along the target interval and approximates the weapon by a line through the weapon hand. It does not measure skin-to-haft collision. The weapon foreshortens near the head in this view, and the planted hips look stiff beneath the fast drop. The release is deliberate. I would refine this clip first. [First grip solve](verification/meridian_fall/anim.txt) · [Final grip solve](verification/meridian_fall-grip/anim.txt)

The server's item lookup gives dragon halberd 3204 stand 809, walk 1146, run 1210 and block 430. It has no item attack override. The halberd entry in `WeaponProfile.ts` supplies reach. Its ordinary style fallback comes from `data/definitions/item-combat-styles.json`, where **Jab, Swipe and Fend all specify 440**. Special attacks have separate handlers. This is a read of the owner's server copy. [Item lookup](verification/measurements.log) · [Exact style records](verification/audit.log)

**Salute.** The fist comes to the chest first, gives slightly, and the head bows into the hold. The head continues moving after the fist arrives, which distinguishes this 52-cycle gesture from the 44-cycle reference. At close size the two actions are clear. At game size the fist overlaps the dark torso and the tiny rebound is easy to miss. The 24-cycle hold reads as formal stillness, though it also makes the otherwise quiet body look rigid. The final recovery now reaches idle on screen.

**Sliding and endpoints.** I tracked both toe-label centroids (45 and 46) through every frame of all three player clips. Each stayed at its own fixed position to the client's printed precision of 0.1 model units. This tests planted feet on a stationary model. Locomotion blending is a separate check. Comparing inherited first and last renders gave 4,554 differing pixels for the sword, 6,493 for the glaive and 2,416 for the salute. The three final first/last comparisons each gave **0**. The cow already gave 0. Its sampled rest key made no visible endpoint improvement. [Toe tracks](verification/review.log) · [Endpoint comparisons](verification/final_checks.log)

## Creatures and skeletal playback

| NPC | Classic frame base | Result |
|---|---|---|
| Goblin 3029, humanoid | 1415, 170 groups. 669 vertices, 52 vertex labels. Stand 6181 and walk 6180. 73 classic sequences use this base. | The same author and loose/packed loaders work. The five-frame toe tap changes g9 around its real squat rest. It is deliberately small. The affected group covers 30 vertices near the foot. [JSON](verification/goblin.json) · [Sheet](../../render/research-animation-verified/goblin-check.png) |
| Cow 2790, beast | 1338, 135 groups. 475 vertices, 74 vertex labels. Stand 5852 and walk 5848. 44 classic sequences use this base. | The inherited fly shake drives neck/head g91/g88, ears g82/g86 and tail g15/g9. The close view shows the alternating head and tail directions. At game size it is a subtle idle, visibly smaller than the real graze. The even three-cycle spacing makes it fairly mechanical. The real idle holds seven cycles, the graze five. |

Both jobs use numbered groups on their own base with their own rest pose. No changes to the encoder were necessary. The [creature measurements](verification/measurements.log) list group types, target labels, counts and vertex bounds. For a weighted counterexample, NPC 15740 used base 2653 and sequence 13926. The client reported a cached animation on both samples and the frames differed by 1,963 pixels. [Rendered samples](../../render/research-animation-verified/skeletal-example.png)

## Packing and reaching the game

The final local cache is **[verification/final-cache](verification/final-cache)**. The packer created a fresh copy of the source cache files. For each new group it joined the frame files, wrote a container and trailer into a new sector chain in dat2, and indexed the chain in idx0. It rewrote group 12 in archive 2 with the new sequence definitions, then updated the reference tables for archives 0 and 2 in archive 255, including file IDs, CRCs, versions and sizes. Both original reference tables re-encoded identically before modification (protocol 7, flags 4). It reuses the existing bases. **Give the inherited packer a new, nonexistent destination**. Its startup code recursively removes an existing destination.

| ID | Final packed vs loose, differing / compared pixels |
|---|---:|
| 20001 | 0 / 1,320,000 |
| 20002, improved grip | 0 / 1,800,000 |
| 20003 | 0 / 1,320,000 |
| 20004 | 0 / 1,920,000 |
| 20005 | 0 / 600,000 |

For players, the packed path was ordinary `asset_pose.exe seq=ID`. For NPCs it was the cache branch of `asset_anim.exe seq=ID`, which supports NPC models. Every frame was compared against `anim=DIR` on the source cache. The final copy contains 14,415 definitions. All 14,410 stock definitions match byte-for-byte. As rendered with item 1323, stock walk and slash also match the source cache over 960,000 and 1,200,000 pixels respectively. The earlier `verification/new-cache` and the sheets named `meridian_fall` without `-grip` belong to the intermediate grip pass.

A game installation needs the same definitions, frame groups and reference tables available to its client, and server logic choosing the new sequence ID. For an attack the existing item/style selection is a candidate integration point. A salute needs an emote action mapping, and an NPC needs the appropriate action or definition selection. The private IDs are free in this cache snapshot. This work did not exercise network cache delivery, a server-to-client animation update, interrupt/restart behavior, combat-hit synchronization, moving actors, other body kits, or authoring a new archive-22 asset. The visible weapon in the tests was a local model injection. Installing the Meridian item models as real server items is a separate prerequisite for seeing that equipment in game.

**Next experiment.** In an authorized disposable client/server pair, install the exact final files, bind one test action to 20002, and record the sent ID, received assets, rendered frames and hit cycle while stationary and walking. Review the glaive from the side at normal speed, particularly hand contact on frames 7–9 and the return from the low hold. That would test the remaining gap between a readable cache and a shipped action.

## Exact reproduction and outputs

Working directory throughout was `{ROOT}/studies/research-animation`. Commands below are in dependency order. The wrappers print the executable and every argument of each child command into the corresponding transcript. Python used the specified interpreter in isolated mode. The additional `-B` suppresses bytecode writes. I used the existing release example binaries. I did not build Rust or rerun the previous engineer's original fixed-output pixel/pack check scripts. The new comparator keeps its own output paths and checks the result with assertions.

```powershell
$py = '{PYTHON}'
& $py -I -B verify_takeover.py census
& $py -I -B verify_takeover.py pixels
& $py -I -B verify_takeover.py measurements
& $py -I -B verify_takeover.py authoring
& $py -I -B verify_takeover.py pack
& $py -I -B verify_takeover.py packed_pixels
& $py -I -B finish_motion.py audit
& $py -I -B finish_motion.py refine
& $py -I -B finish_motion.py final_checks
& $py -I -B finish_motion.py review
```

The intermediate `pack` command intentionally requires a fresh destination. On this already-completed run, retain the existing copies and rerun the readback checks directly.

| Command | Recorded output |
|---|---|
| census, pixels | [Whole-cache codec counts](verification/census.log) · [Per-frame RGB comparisons](verification/pixels.log) |
| measurements, authoring | [Real motion and creature groups](verification/measurements.log) · [Four source rebuilds and comparison sheets](verification/authoring.log) |
| pack, packed_pixels | [Intermediate cache creation](verification/pack.log) · [Intermediate pixel checks](verification/packed_pixels.log) |
| audit, refine | [Inherited cache, server styles, joint steps and goblin](verification/audit.log) · [Glaive correction](verification/refine.log) |
| final_checks, review | [Final pack, stock regression, endpoint and skeletal checks](verification/final_checks.log) · [Inventory, game scale, toe tracks and final definitions](verification/review.log) |
