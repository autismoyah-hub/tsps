# Gates and checkers

Run from the package root with `python -I`. A gate's last line is `VERDICT PASS|BORDERLINE|FAIL` (exit 0, 1, 2);
INFO lines never decide. A checker prints `holds | OUTSIDE` per rule against the measured real population: a guard,
not a judge. Before trusting any gate on a new kind, gate a real asset re-encoded through our encoder: it must pass.

## Gates

| Kind | Command |
|---|---|
| Scenery rebuild | `tools/gate.py rebuild-object REAL_ID FILE.bin --ambient A --contrast C [--shape 10/4/22]` |
| Scenery original | `tools/gate.py original-object FILE.bin --faces LO,HI --size X0,X1,Y0,Y1,Z0,Z1 [--shape N] [--bottom LO,HI] [--wall-plane X]` |
| Weapon rebuild | `tools/gate.py rebuild-weapon item:ID FILE.bin` |
| Worn gear | `tools/gate.py worn FILE-labelled.bin SLOTSPEC [--real item:ID] [--reference item:ID] [--steel 42,0] [--non-steel H,S,L]` |
| Creature rebuild | `studies/research-creatures/gate_npc.py rebuild-npc REAL_NPC_ID OURS.json [--json FILE] [--out DIR]` |
| Skeletal creature / boss rebuild | `studies/research-bosses/gate_bone.py rebuild-npc REAL_NPC_ID OURS.json`; via the rig route `studies/research-rig/rig_gate.py WORKDIR OURS.bin` |
| Titan sheets submissions | `studies/tools-titan-sheets/teacher_gate.py` (Teacher only) |
| Small inlaid detail | `studies/research-paint/detail/detailgate.py REAL_NPC_ID OURS.json` (proposed line) |
| Interface | `studies/research-interfaces/ifgate.py rebuild GROUP REBUILD.json [--detail]` and its original-interface checks |
| HD-era rebuild | `studies/tools-hd-gate/hdgate.py` (Teacher only) |
| RS3 rebuild | `studies/tools-rs3/rs3gate.py rebuild KIND ID OURS.bin` (read the VERDICT line, not the exit code) |

## Checkers

| What | Command |
|---|---|
| Room composition | `studies/research-rooms/roomcheck.py PLACEMENTS.csv --kind KIND` |
| Boss arena | `studies/research-arenas/arenacheck.py PLACEMENTS.csv FLOOR.json` |
| Fight structure | `studies/research-fights/fightcheck.py FIGHT.json` |
| Boss package | `studies/research-authorship/packagecheck.py PACKAGE.json` (`--calibrate` checks the bands) |
| Gap to real creatures | `studies/research-gap/gapcheck.py MODEL_OR_NPC_JSON` |
| Cleanness | `studies/research-clean/cleancheck.py ENTRY [--pic OUT.png]` |
| Paint style | `studies/research-paint/stylecheck.py DEF.json PLAN.json` |
| Helm fit and set definition | `studies/research-fit/fit.py HELM-labelled.bin SLOTSPEC`, `studies/research-fit/definition.py OUTFIT_ENTRY` |
| Helm measures | `studies/research-helm2/helmcheck.py`, `studies/research-helm3/helm3check.py HELM-labelled.bin` |
| Sitting picture presets | `studies/tools-render-quality/check_presets.py` |

Thresholds belong to the Teacher. A Producer or author never changes them; a proposed change comes with the
pieces, pictures and numbers that justify it.
