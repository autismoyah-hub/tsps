# Authoring animations for the revision 239 client

A classic action needs a sequence definition, a group of frame files, and a frame base compatible with the mesh. For these originals the first two are new. The base and labelled meshes already exist. Coordinates below are model coordinates. On the player, negative x is the character's right, negative z is forward, and height is −y.

## From a sequence ID to a pose

| Cache address | Contents |
|---|---|
| Archive 2, group 12, file = sequence ID | Definition. Opcode 1 supplies ordered frame references and hold lengths. Several entries can reference the same file. |
| Archive 0, group = high 16 bits, file = low 16 bits of a frame reference | Sparse transforms for one pose. The runtime reads the base ID from the first file and decodes the whole group against it. A bad member can prevent loading the group. |
| Archive 1, group = base ID, file 0 | Ordered transform types and their target label lists, followed by the optional bone data. |
| Archive 22, group/file = high/low halves of the opcode 13 ID | Cached skeletal animation channels. The asset itself identifies the base and bind pose. |

### Definition

Multibyte integers are big-endian. Each opcode is one byte. A zero byte terminates the definition. Revision 239 uses the following assignments.

| Opcode | Payload | Use |
|---|---|---|
| 1 | u16 count N, N u16 durations, N u16 file IDs, N u16 group IDs | Classic timeline. Durations are client cycles of about 20 ms. Arrays are adjacent chunks, not interleaved records. |
| 2, 8 | u16 loop-back frame count, u8 maximum loops respectively | Rewind from the end by the specified number of frames. Default loop-back −1, maximum 99. |
| 3 | u8 count, u8 group indices | Classic action/locomotion interleave. The client adds a sentinel. With a compatible secondary pose, the primary supplies the unlisted groups and the secondary the listed groups. Origins are processed in both passes. |
| 4, 5 | no payload, u8 respectively | Actor stretch flag and action replacement priority (default 5). |
| 6, 7 | u16 each | Shield and weapon appearance overrides. Omitted means keep equipped gear. Zero hides the slot. |
| 9, 10, 11 | u8 each | Movement precedence, moving priority, same-action restart mode. The first two default to 2 with an interleave or bone mask, otherwise 0. Restart defaults to 2. |
| 12 | u8 N, N u16 low halves, N u16 high halves | Secondary chat frame references. |
| 13 | u32 | Cached animation asset ID when signed value is nonnegative. |
| 14 | u16 N, then N records of u16 frame, u16 sound, four u8 fields | Per-frame sound location, loops, range and retain. The client filters unusable sound records. |
| 15, 16, 17, 19 | two u16 range endpoints, one byte, counted u8 bone indices, no payload respectively | Cached timeline length = end − start, extra signed byte in the client, blend mask, presence flag. |

In a classic frame the actor holds a pose. The JSON authoring program interpolates *before encoding*. Its frame-start grid is 0 followed by cumulative durations. The client clock increments elapsed time and advances while elapsed **exceeds** the current length. The grids in the sheets describe the authored holds. They do not measure server latency, hit timing or the initial clock phase.

### Base, frame and model labels

A base starts with u8 N, N type bytes, N label-count bytes, and then the N lists of u8 labels. Group number means an **index in this table**. A vertex label identifies a set of vertices on the combined mesh. For example, player group 30 turns the right shoulder chain. Label 50 belongs to the weapon and is included in upstream transforms so it travels with the arm. Player base 0 has 245 groups. A goblin's group 30 need not have the same role.

A frame contains u16 base ID, u8 mask count, that many one-byte masks, then the component values for all enabled mask bits in group order. Bits 0, 1 and 2 enable x, y and z. Signed-smart values −64…63 occupy one byte biased by 64. Other values in −16384…16383 occupy two bytes biased by 49152. A zero mask skips the group. Unset components default to 0 except scale, whose default is 128. Before an active non-origin transform, decoding inserts the nearest unused preceding origin since the last explicit transform if present, with zero offset.

Each frame deforms a fresh copy of the merged rest mesh. Groups execute in order against the *current* coordinates. A vertex can therefore move in several groups even though its classic skin has one label.

| Type | Operation |
|---|---|
| 0 origin | Recompute the pivot as the centroid of all vertices with listed labels, plus the frame offset. Empty selection uses the offset alone. The classic integer path truncates the centroid. |
| 1 translate | Add the three values to each selected vertex. |
| 2 rotate | Subtract the current pivot, turn about Z, X, then Y, restore the pivot. Each component wraps modulo 256. One unit is 1.40625°. The classic integer path uses fixed-point trigonometry and shifts between turns. |
| 3 scale | Scale displacement from pivot per axis by value / 128. |
| 5 alpha | Select faces by **face** skin labels. Add 8 × x to transparency, clamp to 0…255. |

Preserve the rest frame's small translations and pivot offsets. A missing elbow ring or a wearable carrying label 0 changes the centroid of an upstream group and can move the rest of the figure. The full cache also contains types 4, 7, 9 and 10. These five operations describe the classic deformation branches used for this work.

### Cached skeletal animation

The base tail can contain a u16 bone count, u8 pose count, and a signed parent index per bone followed by a 4×4 float matrix and three extra floats per pose. An archive-22 file has a version, base ID, range, pose index and typed curve channels. Each channel names a target, component and curve. The runtime samples rotation, translation and scale channels, composes local matrices through the parent hierarchy and applies inverse bind transforms. A vertex may carry multiple bone indices and byte weights. The skinning path sums matrices weighted by weight/255, transforms the vertex, and can apply a separate alpha curve. Vertices without weights receive no skeletal displacement.

This client implements the path. In the executed Maggot marquess test, NPC 15740 played sequence 13926, cached animation 52166656, base 2653. Frames 0 and 10 differed by **1,963 pixels**. The inherited loose animator explicitly accepts classic sequences only. Use the NPC harness for cached playback. Writing new curve assets and new weighted meshes remains outside the demonstrated route.

The census found 13,466 classic definitions and 944 cached definitions. None carried both. The 944 IDs are enumerated with group, file and range in [skeletal-sequences.tsv](verification/skeletal-sequences.tsv). IDs alone cannot classify them. The first is 2973 and 251 fall below 10000. Archive 22 contains 914 files. All 2,673 bases carry tail bytes, but only 127 have a positive bone count. Player base 0 has zero bones.

## Make and preview a classic action

Use [the revised salute](verification/warden_salute.json) as a working template. Pick a rest sequence on the required base, a classic `like` definition with suitable playback and equipment fields, an unused sequence ID and an unused frame group. The two IDs can differ. Set positive frame lengths. Write keys using frame indices or cycle times. A named channel or `gN` takes an absolute triple in frame units. Omitted channels retain their own interpolated timeline or rest values. Easing belongs to the segment *arriving* at the key. Linear interpolation operates on components, so unwrap angles deliberately when crossing the modulo boundary. A rest key at the last displayed frame makes the rest pose visible before the clip ends.

Run from this folder. These commands produce a loose directory and a two-row preview. The complete executed versions with absolute paths and output are in [authoring.log](verification/authoring.log).

```powershell
{PYTHON} -I -B author.py verification/warden_salute.json verification/warden_salute
{PYTHON} -I -B apose.py ../../render/research-animation-verified/warden_salute-close.png 'anim=verification/warden_salute;none;warden_salute' 'seq=2112;none;reference 2112' --zoom 520 --cell 480x530 --draw actor
```

The salute has 11 playback entries, nine distinct frame files and 52 cycles. The encoder deduplicates identical poses and writes an `anim.txt` containing every moving group. Sound and chat frames from the reference are omitted. Use numeric groups and a creature-specific rest pose for NPCs. The optional player polearm grip solver requires a labelled dump of the assembled figure with that particular weapon.

## Evidence and implementation references

From this folder run `{PYTHON} -I -B verify_takeover.py census`. Its [full output](verification/census.log) records 14,403/14,410 byte-identical sequence definitions, 2,673/2,673 byte-identical bases, and 209,191/209,255 byte-identical frame files with stored masks preserved. The seven definition exceptions contain repeated opcodes whose earlier values are overwritten in the Python dictionary. Frame group 12286 on base 1941 gives 64 trailing-byte failures, used by sequence 8850. The two required sequences are outside these exceptions.

The runtime evidence for the cached example, including the exact `asset_npc.exe` command, is in [final_checks.log](verification/final_checks.log). The implementation references under `../../client/rust-client/src/` are `animation.rs` (Sequence, Skeleton, Frame), `animation_runtime.rs` (Clock, frame group loading, apply_actor and cached_pass), `deform.rs` (classic transforms), `cached_animation.rs`, `curve.rs`, and `skeletal.rs` (channels, sampling and skinning).
