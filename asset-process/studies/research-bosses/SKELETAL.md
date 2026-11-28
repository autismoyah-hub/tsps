# Skeletal (bone-weight, keyframe) animation in the revision 239 client, exactly

Private to this machine (cache-derived). Research of 2026-11-26/27, folder `{ROOT}/studies/research-bosses/`.
`P` = `{PYTHON} -I`, run in this folder. Everything stated as measured was run;
section 8 lists what was not. Source read: `client/rust-client/src/` `model.rs`, `model_merge.rs`,
`animation.rs` (Skeleton), `cached_animation.rs`, `curve.rs`, `skeletal.rs`, `animation_runtime.rs`,
`world_actors.rs`. Codec: `bonecodec.py`. Harness: `client/rust-client/examples/asset_bone.rs`.

## 1. The three files, and how a sequence reaches them

| file | cache address | what it holds |
|---|---|---|
| weighted model | archive 7, group = model id, file 0 | the mesh; per vertex a list of (bone, weight) |
| frame base | archive 1, group = base id, file 0 | classic groups (type, labels), then the BONE SECTION |
| keyframe file | archive 22, group = id >> 16, file = id & 65535 | curves per bone and per alpha group |
| sequence | archive 2, group 12, file = sequence id | opcode 13 = keyframe file id (u32), opcode 15 = range (u16 start, u16 end) |

A skeletal sequence has NO opcode 1 (no classic frames). In this cache: 944 sequences name a keyframe file, 914
keyframe files exist, 126 of 2,673 frame bases have bones, 374 of 61,606 models carry weights.
Weights and keyframes always come together: of the 48 boss forms measured in BOSSES.md, every weighted one has only
keyframe sequences on its base and every labelled one has none.

## 2. The weighted model

Layout `skinned` (the file ends `ff fd`; 26-byte footer). All 374 weighted models use it. Body sections in order:
texture types (nt) | vertex flags (nv) | render types (nf, if flag) | strip opcodes (nf) | priorities (nf, if the
footer priority is 255) | face labels (nf, if flag) | **vertex data (vlen)** | alpha (nf, if flag) | index data |
face textures (2 nf, if flag) | texture coordinates | colours (2 nf) | x | y | z deltas | simple texture triangles |
complex mapping bytes | metadata marker (+10 bytes when non-zero) | depth-offset marker (+nf bytes when 1).
Footer: u16 nv, u16 nf, u8 nt, u8 has render, u8 priority (255 = per face), u8 has alpha, u8 has face labels,
u8 has face textures, u8 has vertex labels, **u8 has bones**, u16 x, y, z, index, texture-coordinate and
**vertex-data** lengths, `ff fd`.

**Vertex data** = [nv label bytes, if [quoted words withheld]] then, if [quoted words withheld], for each vertex in order:
`u8 count`, then `count` pairs `u8 bone index, u8 weight`. That is all. Measured over the 374 models (261,783
vertices; `P bone_facts.py 13926 --weights`):
- influences per vertex: 1: 198,399 (76%); 2: 23,015; 3: 21,325; 4: 17,196; 5 to 9: 1,848. No vertex has none.
- weights sum to 255 on 261,775 vertices (254 on 8). The client does NOT normalise (it divides each weight by 255),
  so a sum under 255 pulls the vertex toward the origin by the missing share.
- pairs are not sorted (30,651 vertices are not heaviest-first); no zero weights are stored.
- 371 of 374 carry no vertex labels at all; highest bone index used 253 (one byte: at most 256 bones can be named;
  the largest real base has 255).
- strip opcodes: every one of the 34,616 `skinned` models is byte-identical when our encoder picks the opcodes
  itself (real files use opcode 1 only, three indices per face), so nothing about the triangulation is lost.
Several models of one NPC are merged by `Model::combine`: vertices at equal coordinates become one and keep the
FIRST one's weights (`model_merge.rs`).

## 3. The frame base's bone section

After the classic part (u8 n groups, n types, n label counts, the label lists):
`u16 bone count`; if it is 0 the file ends (2,546 bases end with `00 00`). Otherwise `u8 poses`, then per bone:
`u16 parent` (signed; negative = root), and per pose: 16 floats (a 4x4 matrix, COLUMN-major: element `[col*4+row]`,
translation in 12, 13, 14) followed by 3 more floats. All big-endian IEEE 32-bit.
Measured over the 126 bases with bones (8,822 bones; `data/census-bases.json`):
- poses = 1 in every base; every keyframe file names pose 0.
- exactly one root per base; **every bone's parent has a HIGHER index** (8,696 of 8,696): child first, root last
  (Maggot King: root is bone 154 of 155; deepest chain 21). The client resolves parents recursively, so order is free
  (proved: section 6, the renumbered copies).
- the matrix is the bone's REST transform LOCAL to its parent. 4,602 bones are pure translations; 4,220 also carry
  a rotation or scale.
- the 3 extra floats per pose are read and never used by this client (values such as -6.6e22 occur); they are
  carried bit for bit.
- bones: 2 to 255 per base, median 59. Owner's bosses: Zebak 47, Akkha and the Wardens 73, Doom 90, Maggot King
  155, Kephri 225.
- the classic part of a skeletal base: 38 bases have no groups at all; the rest hold PAIRS `(type 7, [label k])`,
  `(type 5, [label k])`, one pair per face label (Maggot King: 106 pairs). Type 5 groups are the targets of the
  alpha curves. Type 7 groups are the targets of [quoted words withheld] curves (channels 10 to 12; 16,332 in the cache), which
  this client decodes and never evaluates. Six bases also keep ordinary classic groups.
- one base (1941) has a bone count of 3 with 0 poses and 1,529 bytes left over: the client rejects it ("skeleton
  trailing bytes") and so does `bonecodec` (it is the base of the one frame group that never loaded, FORMAT.md).

## 4. The keyframe file

Header: `u8 version` (1 in all 914), `u16 base id`, `u16 range start`, `u16 range end`, `u8 pose` (0 in all),
`u16 channel count`. Then per channel:
`u8 kind`, `signed-smart target`, `u8 channel`, then the curve:
`u16 key count` (at least 1), `u8 (ignored by the client: 0 on rotation curves, 1 on translation, 3 on the rest)`,
`u8 before`, `u8 after` (what happens outside the keys: 0 hold, 1 linear, 2 and 3 repeat, 4 mirror; **0,0 on all
850,248 real curves**), `u8 weighted`, then per key 22 bytes: `u16 time` (signed), `f32 value`,
`f32 in.x, in.y`, `f32 out.x, out.y` (tangents as (time, value) vectors). Key times must not decrease.

| kind | channel | target | meaning |
|---|---|---|---|
| 1 | 1, 2, 3 | bone index | rotation about x, y, z in RADIANS |
| 1 | 4, 5, 6 | bone index | translation x, y, z in model units, local to the parent |
| 1 | 7, 8, 9 | bone index | scale x, y, z |
| 4 | 16 | index of a type-5 group of the base | alpha: value x 255 is ADDED to the alpha of every face whose face label is in the group, clamped 0..255 |
| 3 | 10, 11, 12 | index of a type-7 group | decoded, never evaluated here (only channel 10 is even kept) |

If two channels name the same slot the LATER one wins (none of the files tested does).
A real file gives every bone all nine curves (Maggot King stand: 155 x 9 + 106 alpha = 1,501 curves, 917 of them
constant).

**How real animations are keyed (measured, `P bone_facts.py SEQ`).** Not as sparse curves. Each bone curve has a key
at time -1 holding the REST value, then one key every 3 or 5 cycles whose out-tangent is (FLT_MAX, FLT_MAX), then
one closing key far beyond the range (time 586 or 19,465). In the client an out-tangent of (FLT_MAX, FLT_MAX)
means: at the key's own time return its value, at any later time before the next key return the NEXT key's value.
So a real skeletal creature holds each pose for 3 (or 5) cycles and jumps: the Maggot King's 90-cycle stand shows
31 distinct poses, its 120-cycle attack 41. Of 15.6 million real segments 13.7 million are these jumps, 0.2 million
are holds (out-tangent (0, 0): the value stays until the next key), 1.7 million are true curves.
True curves exist too: 135,819 curves in 176 files on 28 bases are flagged `weighted` (Yama's base 2404 has 16
such files) and are evaluated as cubic Beziers in time and value with a root search (`curve.rs`); unweighted
splines are evaluated as a cubic Hermite: slope out = out.y / out.x, slope in = in.y / in.x of the next key.

**Time.** The clock of a keyframe sequence counts FRAMES, one per client cycle (20 ms): `Clock::advance` adds one
to `frame` per tick; the frame count is `range end - range start` (opcode 15); at the end it loops back by
`loop_frames` or stops, as a classic sequence does. The curve is evaluated at time = frame index as a float
(`apply_observed` passes `clock.frame as f32`): key times ARE client cycles counted from the start of the sequence.
The range start is 0 in all 914 files; this client does not add it. Sound frames (opcode 14) index the same
frames (Maggot King attack: sounds at frames 10, 54, 103).

## 5. How the client poses and lights

`Rig::new` (once per keyframe file): for each bone the global rest matrix = parent's global x local; its INVERSE is
the inverse bind. From each local rest matrix it also takes the rest rotation (three angles from the inverse
matrix), rest scale (column lengths) and rest translation: these are the values of any channel the file does not
supply.
Per frame (`cached_pass`): every channel is evaluated at the frame time and overwrites its component; each bone's
local matrix is rebuilt as `compose`: a quaternion made by turning about **z, then x, then y** (half angles
multiplied in that order), converted to a matrix, its rows scaled, translation written to elements 12 to 14.
Global = parent global x local; skinning matrix = global x inverse bind.
`Model::skin`: for each vertex with weights, M = sum over its pairs of (weight / 255) x matrix[bone] (a pair naming
a bone outside the base is skipped silently); the vertex (x, y, z) is taken as (x, -y, -z), transformed by M, and
y and z negated back. Vertices with no pairs do not move. The result is kept as FLOATS (the classic path truncates
to integers). Then the alpha curves are applied, then `npc_model::resize` scales the model by the definition.
With a bone mask (sequence opcode 17) two keyframe sequences are mixed bone by bone on their LOCAL matrices
(`apply_blended`); a keyframe action over a classic walk, and the reverse, are also handled. Not exercised here.

**Lighting: once, at rest, for skeletal creatures too (confirmed in source and by the harness).**
`world_actors.rs` builds the NPC's mesh, calls `model.light(64 + ambient, 850 + contrast, (-30, -50, -30))` and
caches the mesh WITH its lit corner colours; every frame clones the mesh, animates the clone and paints it with the
cached colours. The same lines serve labelled and weighted NPCs. So a weighted creature is lit in its BIND pose,
which is a spread pose the game never shows (wings open, arms out, the Maggot King's maggot standing 170 units
behind the body): tones belong to the bind pose's facing, and whatever faces up in the bind pose is light for
ever. `asset_bone.rs` repeats these calls; its pictures equal the by-id pictures (section 6).

## 6. The encoders, and what is proven

`bonecodec.py`: `decode_model / encode_model`, `decode_base / encode_base`, `decode_anim / encode_anim`.

**Bytes** (`P bone_census.py all`; outputs `data/census-*.json`):
| encoder | real files | byte-identical |
|---|---|---|
| model, `skinned` layout, OUR OWN strip and flag choices (`hints=False`) | 374 weighted | 374 |
| the same, unweighted `skinned` models | 34,242 | 33,028; the other 1,214 differ in ONE byte (their metadata marker is 5, we write 1; any non-zero value means the same to the client); with that byte kept as a hint 34,242 |
| frame base with bone section | 2,673 | 2,672 (the 1 is base 1941, which the client rejects) |
| keyframe file | 914 | 914 |
The 26,990 models in the `skinned_legacy` layout (`ff fe`) are decoded, not re-encoded in their own layout.

**Pictures** (`P bone_rt.py NPC OUTDIR`; logs `rt/*.log`, numbers `rt/*/roundtrip.json`). Every frame of every
keyframe sequence on the creature's base, two views (yaw 192 and 704), 300 x 300 cells, three drawings: the real
NPC through the cache loaders; `plain` = our re-encoded model, base and keyframe files as LOOSE files; `renumbered`
= a new base id, the bones in a shuffled order, every model's vertices reversed, every file's channels reversed
(weights and targets remapped): files that share no section with the originals.

| creature | base (bones) | models | sequences / files | frames drawn | figure pixels | plain differing | renumbered differing |
|---|---|---|---|---|---|---|---|
| Maggot King 15742 (owner's) | 2653 (155) | 1 | 15 / 14 | 1,440 | 48,117,813 | 0 | 0 |
| Doom of Mokhaiotl 14707 (owner's) | 2441 (90) | 5 | 18 / 18 | 1,350 | 83,912,196 | 0 | 0 |
| Zebak 11730 (owner's, Tombs) | 2074 (47) | 1 | 10 / 10 | 1,110 | 50,116,000 | 0 | 0 |
| Kephri 11719 (owner's, Tombs) | 2079 (225) | 1 | 13 / 13 | 1,260 | 28,029,464 | 0 | 0 |
| Akkha 11789 (owner's, Tombs) | 2065 (73) | 3 | 38 / 34 | 2,640 | 83,555,440 | 0 | 0 |
| Vardorvis 12223 (two tiles) | 2151 (99) | 2 | 11 / 9 | 720 | 15,194,399 | 0 | 0 |
| Yama 14176 (weighted Bezier curves: 11,518) | 2404 (104) | 3 | 12 / 12 | 750 (every 2nd) | 28,065,398 | 0 | 0 |
Total 9,270 frames, 18,540 cells, 1,668,600,000 pixels, 336,990,710 figure pixels, 0 differing. In `plain` the
posed vertices are bit-equal to the real ones in every cell (hash of the float bits). In all seven the `plain`
files are also byte-identical to the cache, so `plain` proves the loose-file path and `renumbered` proves the
encoders write content, not a replay.

**By id** (`P bonepack.py rt/maggot-king cache-packed ...`, `P bone_byid.py cache-packed ...`; log
`rt/byid-maggot-king.log`): the renumbered Maggot King packed into a private copy of the cache as model 62500,
frame base 12653, 14 keyframe files (archive 22 groups 3000 to 3013), sequences 21000 to 21014 and NPC 16370, and
drawn by the client's ordinary loaders: 1,440 frames, 2,880 cells, 56,930,765 figure pixels, **0 differing**
against the real NPC on the real cache.

What is byte-identical: all three kinds, as above. What is only draw-identical: nothing needed that fallback; the
renumbered copies are draw-identical by design. What failed: nothing in these runs. What the round trip does NOT
exercise: section 8.

## 7. Writing one (`bone_author.py`), and the first authored animation

`P bone_author.py SPEC.json OUTDIR` writes a keyframe file from tracks of OFFSETS on named bones and channels,
starting from a real sequence's pose at a chosen frame (the bind pose is not a stance), eased between keys; a
`chains` entry gives a tail or neck follow-through (each bone later and larger). Two modes: `baked` (a key every 3
cycles stepping to the next: what real files do) and `spline` (sparse keys with (1, 0) tangents; the client
interpolates every cycle). `P bone_probe.py NPC SEQ OUT.png BONES` draws what each channel of a bone does, because
a bone's axes are not readable from the file.
Proofs: a file with no tracks draws the real stand's frame 0 exactly (0 of 90,000 pixels); the `spline` file at
cycles 0, 3, .. 57 equals the `baked` file (0 of 366,183 figure pixels; vertex bits differ in 16 of 20 cells, by
float rounding); the baked file shows 17 distinct poses in 60 cycles, the spline file 50; both packed as sequences
21100 and 21101 and played by id on the real Maggot King: 0 differing pixels against the loose files.
The animation: `authored/flinch.json`, a 60-cycle (1.2 s) flinch for the Maggot King: 16 moving channels (root
rears, chest and maggot recoil, nine maggot segments whip with one cycle of delay each, both wings lift).
Board: `{ROOT}/render/research-bosses-authored/flinch-board.png` and `.gif` (ours baked, ours
splined, real stand 13926, real attack 13928, real 13935).
Honestly: it is a readable recoil (up and back by cycle 6, forward past rest at 18, settled by 30) and it joins
the stand without a jump. It is not at the real animations' level: the real attack moves every part of the body
and both wings through large arcs with overlap; ours nods the maggot on a body that barely shifts, the wings move
too little to see at game size, and the last 30 cycles are nearly still. It proves writing.

## 8. Not proven, not run

- No NEW weighted mesh has been made: the model encoder is proven on real content (re-ordered), not on weights
  an author assigned. There is no tool to assign weights (REPORT.md, section 6).
- No new SKELETON: `encode_base` writes any bone tree, but no bone section has been designed here. The rest
  matrices' rotation convention was read from source, not exercised with a bone we placed.
- Bone masks (opcode 17) and mixing of two sequences were read, not drawn. A renumbered creature's masks would need
  remapping; `bonelib.renumber` does not do it (none of the packed sequences was drawn mixed).
- "Kind 3" curves and type 7 groups: carried byte for byte, meaning unknown, never evaluated by this client. If the
  official client uses them (a tint?), this client and every picture here lack that effect.
- Extrapolation modes 1 to 4 and curve `before` / `after`: no real file uses them; not tested.
- The official client was not compared: "identical" means identical in this Rust client, whose skeletal code the
  project had already checked only by one 1,963-pixel motion test (FORMAT.md).
- Nothing was played in a logged-in session or by the server.
- `skinned_legacy` models are not written; an authored model always comes out in the `skinned` layout.
