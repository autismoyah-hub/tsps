# Optional Maggot King trip resupply: exact-source result

The Sangvesti chest becomes an optional rebanking stop between Maggot King trips. It adds no lair-entry or grave-recovery prerequisite. A player with boostable 70 Construction, a hammer, a saw, 1 magic stone, 2 cupronickel bars, 2 adamantite bars and 4 ironwood planks can build the chest in three task ticks for 1,787 base Construction XP. Existing Construction helpers retain crystal-saw, carried-storage, material-discount and outfit behavior.

The ordinary reached Build operation defers all payment until its third valid tick and rechecks tools, materials, level, position, world object and player state before committing once. Movement and the registered interaction/logout/death/shutdown interruptions cancel pending work. Built state is per player and projects varbit15621; the existing Bank handler serves the built chest. Rejected copies of this root are consumed at dispatch so the generic Sailing amenity cannot pay after Maggot refusal; unrelated object roots retain ordinary delegation.

Behavior sources: [Maggot King/Strategies revision15402578](https://oldschool.runescape.wiki/w/Maggot_King/Strategies?oldid=15402578), [Bank chest (amenity) revision15402269](https://oldschool.runescape.wiki/w/Bank_chest_(amenity)?oldid=15402269), retained cache2757/rev242. Root62185 at2576,7833,0; visible62186/62187. Deferred commit order, presentation lines and interruption policy are disclosed repository choices, not captured retail timing or text.

## Exact source and actual result

Tested and intended draft PR head: [`fdec1c12aa7e1c4341eb30c690dffa3d70e46141`](https://github.com/autismoyah-hub/tsps/commit/fdec1c12aa7e1c4341eb30c690dffa3d70e46141); tree `2eab4e1d14ed4164e1a288cc5bfc205af1f6a30e`.

[Job6afa85a323985fb9cd8435f2](https://huggingface.co/jobs/Autismo-Yah/6afa85a323985fb9cd8435f2) was separately retained as **COMPLETED, retry0**. Each selected file ran in its own unfiltered Node process:

| Selected file | Actual passes / tests |
| --- | ---: |
| maggot-king-combat.test.cjs | 83 / 83 |
| maggot-king.test.cjs | 6 / 6 |
| shared-poison-counter.test.cjs | 32 / 32 |
| serpentine-helm.test.cjs | 21 / 21 |
| poison-orb.test.cjs | 10 / 10 |
| Total | 152 / 152 |

All five test stages exited0 with valid TAP and no selected file unstarted. Strengthened combat case82 passed, both strict ordered three-entry legacy registry expectations were in the actual tested bytes, and all six legacy test bodies completed. This supersedes the prior failures only on this exact source; their original results remain failures.

- [Complete raw returned log](maggot-resupply-after029.raw.log): 1,045,196bytes, SHA256 `758c0e8d502909908e3206f9b118937b38d61eb53f9b4280a9742873bf5cd6d3`; unchanged/unredacted.
- [Machine-readable result and full bindings](maggot-resupply-after029.result.json).
- [Original independent result-review text](maggot-resupply-after029.review.txt). It qualifies the exact selected fixture result and explicitly grants no Ready/publication/global-closure authority. The public mirror is documentary, not an execution contract.
- [Immutable public028 input manifest](https://raw.githubusercontent.com/autismoyah-hub/tsps/5ffa17531234704026e154a20a717f87d598e262/docs/evidence/maggot-resupply-intended028-input-manifest.json): 1,375,875bytes, SHA256 `526892053c66d1dd398d8e9d3afc827c3ac1d339605eac0baf416b826c0017a5`.

All6,036 actual runtime inputs matched the manifest; none were filled from expectations. The 724,557byte input event came from45 contiguous returned fragments. Per-fragment digests were computed during retention, not emitted as runtime-signed telemetry. The pinned per-row computed SHA/Gitblob guard precedes that event. Public028 URLs remain input DATA origin;029 was the distinct private execution case, not a source/ref/authority retarget.

Extract/install/ldd/native-load/build/cache all passed. Source9,193/build751/cache28 content seals stayed unchanged after success. Every stage group reported no timeout or remaining group; reported OOM/oomkill0 and peak2,588,311,552 under32,000,000,000bytes. These are content/group/provider observations, not independent PID1 or complete descendant/sandbox absence proof.

## Preserved failure history

- **026**, sourceb38f:82 combat passes,1 case82 refusal failure,69 later tests unstarted. [Complete extracted321-line failure block](maggot-resupply-parent026.failure.tap.txt); full original raw SHA256 `b55037d066fe77acb2ad9fafe7273a9bb5663c54bf915066a58b3e414be6b49a` remains frozen.
- **027**, source0891:83 combat+5 legacy passes,1 first registry assertion failure,63 later tests unstarted. [Complete extracted51-line failure block](maggot-resupply-parent027.failure.tap.txt); full original raw SHA256 `8d4bcef1b8c223de8c58236831b3c8de4d3ad8cbd4d8ac84d0aaad67a8f10cb0` remains frozen. Its later failed-test lair body was unreached.
- **020** was an older-parent83-pass combat result followed by6 before-hook failures/zero legacy bodies and63 unstarted cases.021–025 and028 were fatal pre-submission refusals/local operator no-job, not passed runtime tests or global provider-absence proofs. No old job was replayed, queried or given reused authority for this publication.

## Patch, provenance and remaining review

The feature PR patch has exactly five paths against merge-base25e34586: new Resupply module and documentation, Maggot registration, combat-fixture extension and the legacy fixture correction. All408 JSON paths/modes/blobs are naturally inherited unchanged from that base. No generated JSON, dumper override or output patch was authored; producer/import reproducibility remains a separate limitation.

Current main was inspected at083eb11d406c9ea99b2de53074554289339317ac. Its conflict-free Git merge withfdec yields tree83305eeeb6ab952076fa826b50266362b74e69ac, **not executed**:6,056 eligible inputs,20 added and102 changed versus the tested6,036. No rebase or source edit was made to turn old PASS into a claim about that tree. The exact input delta is in the JSON report. Final patch/evidence and current-main integration qualification remain with the original independent reviewer; this is a **draft, not Ready**.

The source document atfdec retains its dated source-preparation validation paragraph. This result report is the later actual029 outcome; keeping the exact source head avoids silently changing tested bytes.

[Bounded continuing-flight claim](https://github.com/RSPSApp/tsps/issues/153#issuecomment-6907569700); Refs#153 only. The umbrella remains open. Supplied setup, hybrid legacy callbacks, snapshot/JSON persistence, generated-producer/import, natural travel/encounter, decoded packets/rendered client/socket, retail equivalence, backend/disk, historical UNKNOWN/source006 and closure limits remain. Nothing here declares the whole Maggot King area complete.
