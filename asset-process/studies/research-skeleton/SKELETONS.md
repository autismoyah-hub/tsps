# How real skeletons (frame bases) are designed, revision 239 cache

Private to this machine. Everything here was read from the source cache with `skelread.py` and `census2.py` (this
folder), which use `research-animation/animlib.py` (the decoder proven byte-exact on 2,673 of 2,673 bases) and
`asset_npc.exe merged` for the meshes. Listings: `real/base-ID.txt`. Drawn rigs: `{ROOT}/render/research-skeleton-rigs/rig-ID.png`.

```
python skelread.py list BASE NPC      every group, its joint block, the joint tree
python skelread.py rig OUT.png BASE NPC
python skelread.py census             sizes by tile size          -> real/census.txt
python census2.py                     conventions over 565 bases  -> real/census2.txt
```

## 1. What a frame base is, and where the hierarchy hides

A base is a flat list of at most **255 groups** (the count is one byte; 14 real bases use exactly 255). A group is a
type (0 origin, 1 translate, 2 rotate, 3 scale, 5 alpha) and a list of labels (one byte each, so at most 256 labels;
the highest real label is 254). There are no parent pointers. The hierarchy is expressed three ways at once:

1. **Joint blocks.** An origin group followed by the non-origin groups up to the next origin is one joint. This is the
   format's own pairing, not a convention: `Frame::decode` inserts the nearest preceding origin before any non-origin
   transform a frame uses. The origin's labels are the PIVOT labels: the pivot is the centroid, on the mesh as
   deformed so far, of every vertex carrying them, plus the offset the frame gives.
2. **Cumulative label lists.** A joint's rotate group lists its own labels AND every label of every joint below it.
   Vorkath's neck: g127 lists 156 labels, g125 153, g123 150, g121 140, g119 101 ... the wing tip g22 lists 2. A chain
   is a sequence of shrinking lists. The parent of a joint is the joint with the smallest list that contains its list.
3. **Order of the groups.** Transforms run in file order on the current coordinates, so order decides what a joint's
   numbers MEAN (section 3). This is the part nobody had written down and the part that breaks things.

`skelread.py` recovers the tree from 1 and 2 alone and `skelc.py describe` writes it out; compiling that description
gives back the real base (section 5 of REPORT.md: 0 differing pixels on 1,651 frames of three skeletons).

## 2. Five real skeletons

| base | creature drawn | groups (origin / translate / rotate / scale / alpha) | joints | deepest chain | labels in the base | labels on the mesh, faces | sequences on it |
|---|---|---|---|---|---|---|---|
| 326 | Rat 1020 | 21 (9 / 3 / 6 / 1 / 2) | 8 | 3 | 19 | 18, 141 | 6 |
| 815 | Aberrant spectre 2 (floats, size 2) | 33 (13 / 3 / 11 / 3 / 3) | 13 | 4 | 34 | 32, 460 | 5 |
| 1491 | Wolf 106 (quadruped) | 76 (32 / 11 / 30 / 2 / 1) | 32 | 8 | 108 | 64, 892 | 43 |
| 1080 | Baby red dragon 137 (listing), Green dragon 260 (rig) | 194 (75 / 34 / 79 / 4 / 2) | 75 | 10 | 215 | 131, 2,030 | 30 |
| 1782 | Vorkath 8061 (boss, its own base) | 159 (76 / 5 / 75 / 2 / 1) | 76 | 19 | 253 | 251, 2,901 | 15 |
| 0 | the player (listing on Death spawn 10, rig on Man 3106) | 245 (97 / 19 / 83 / 21 / 25) | 96 | 19 | 218 | 45, 975 (Death spawn) | 3,904 |

(The spectre is a sixth, added because it is the floating neighbour of the creature made here.)

**Rat (326).** Root block first: origin on label 0 (three vertices under the feet), then translate, rotate, scale of
all 18 body labels and an alpha group. Then head (pivot = three neck labels, moves label 14), tail in two joints,
two ears (pivot = own label), and the legs as TRANSLATE-only joints (the feet slide, they do not turn). 8 joints is
enough for a creature of 141 faces.

**Wolf (1491).** Leaf joints first: paw g37, then shin g39, thigh g41, hip g43, ... the whole front body g65, and the
whole-body block (rotate 101 labels, translate, scale, alpha) LAST at g71 to g75. Four legs of four joints, a tail of
five, head with jaw and two ears. Most pivots sit on a label owned by the PARENT side of the joint (21 of 32).

**Dragon (1080).** Root first (translate, scale, rotate), then the limbs written parent before child inside each
limb, most joints with BOTH a translate and a rotate group (22 joints T+R), wings with paired rotate groups (`R+R`: a
joint that may be turned twice), neck and tail of five and six joints. 107 of its 194 groups are never mentioned by
any of its 30 sequences (mostly origins, which are implied, and spare channels).

**Vorkath (1782).** One rotate group per joint and nothing else for 70 of 76 joints; every joint written before its
parent; the whole-body block (alpha, rotate, scale, translate) last at g154 to g158. Wings: eleven joints that all
pivot on ONE label (26, the wing root) with lists that shrink by one finger at a time: a fan. Neck and tail are
chains of 10. Most pivots (61 of 76) are a single label owned by the parent.

**Player (0).** The oldest and the most patched: the original 60 or so groups are written parent first (root g0-2,
head g3-4, torso g5-7, legs, then shoulder g28-30, elbow g31-32, hand g33-35), and later additions (g115 and up) repeat
joints with extra label lists for capes, skirts, hair and effects, in mixed order. 15 origins carry a constant
offset in most frames (the pivot is [quoted words withheld]).

Each drawn rig colours every vertex by the joint that OWNS it (the last joint to move it), marks each pivot with a
cross numbered by its origin group, and joins it to its parent's pivot. Pivot offsets carried by frames are not
drawn (the spectre's pivots look low for that reason: its frames add up to 142 units to them).

## 3. Order decides what the numbers mean (measured, not read)

Two families exist, and they are not interchangeable.

- **Child-first** (every joint before its parent; root last). Wolf, Vorkath, most newer creatures. Each joint turns
  about its REST pivot, then its parent carries the result. A joint's angles are therefore LOCAL to its parent, as
  in any ordinary rig, and a pivot label may be anything that no earlier (deeper) joint moves.
- **Parent-first** (root first). The player, the rat, the dragon's limbs. Each pivot is recomputed after the parents
  have moved, so the pivot must RIDE on every joint above it; and a joint's angles are in MODEL axes, applied after
  the parents' turns.

Proof that it matters (`prove_compiler.py`, logs `proof-*/prove.log`): the same real frames re-targeted to the same
joints written in the other order.

| real base | re-ordered child-first | re-ordered parent-first |
|---|---|---|
| wolf 1491 (child-first as stored) | 0 of 57,657,600 pixels; 0 of 450,450 posed vertices differ | 1,921,673 pixels; 201,143 vertices differ, by up to 134 units |
| Vorkath 1782 (child-first as stored) | 0 of 32,774,400 pixels; 0 of 851,793 vertices | 1,212,827 pixels; 717,457 vertices, up to 994 units |
| player 0 (mostly parent-first as stored) | 82,979 pixels; 8,613 vertices, up to 58 units | 8,687 pixels; 1,474 vertices, up to 2 units (rounding and the patched groups) |

Rotations do not commute: a sequence belongs to the order of the base it was authored on.

Over the 565 bases that serve an NPC with a classic stand: 121 are purely child-first, 144 mostly; 49 purely
parent-first, 95 mostly; 123 mixed; 33 have no chains. A whole-body block is among the last two blocks in 251 and
among the first two in 145.

## 4. Conventions

- **Groups per joint.** One origin and one rotate is the norm: of 18,447 joint blocks, 9,379 are a lone rotate; 1,909
  rotate + translate; then the three- and four-group blocks used for roots (`A+R+S+T` 482, `R+T+S+A` 307).
  Translate on a limb joint is for stretch and settling (dragon), on a leaf it replaces rotation (rat's feet).
- **The whole body** is moved by one block that lists every label: rotate, translate, scale and usually the alpha
  group. In child-first bases it is last and written rotate (or alpha, rotate, scale) THEN translate, so the body
  turns about its pivot and is then carried. Its pivot is label 0 (vertices under the feet), a single hub label,
  or nothing (the pivot is then the frame's offset in model space).
- **Pivots.** By where the pivot labels sit: owned by the joint itself 8,244; owned by the parent side 7,255;
  inside the moved set 1,182; straddling 1,176; no labels 257. Either a ring (or one vertex) at the joint that the
  joint owns, or the end of the parent. Vorkath: 61 of 76 on the parent, usually ONE label. Some old bases instead
  give a big label set and a constant frame offset (spectre: every origin, offsets up to 142; 15 player origins).
- **Scale groups** exist in 487 of 571 bases but are rare per base: the whole body (spawn, death shrink: the spectre
  dies by scaling to nothing), a head or eye that pulses, a breath. Vorkath has 2, the wolf 2.
- **Alpha groups** (466 bases) act on FACE labels: the whole body for a fade, plus parts that glow or vanish
  (the spectre's cloak; 25 on the player for effects). One or two per creature.
- **Spare channels.** Real bases carry groups no sequence uses (wolf 38 of 76 unmentioned, Vorkath 68 of 159, dragon
  107 of 194; most are origins, which are implied, the rest spare translate / scale). A base is cheap; frames are not.
- **No hidden pivot vertices.** None of the six meshes has a vertex outside every face: pivots are made of real
  surface vertices.
- **Labels are free numbers.** Renumbering every label of the mesh and the base together changes nothing
  (0 pixels, section 5 of REPORT.md). Two labels on one rigid part are common: one for the joint ring, one for the rest.

## 5. How big real ones are

Distinct (base, model list) pairs standing on a classic sequence, 10th / 50th / 90th centile (`real/census.txt`;
the player base excluded):

| tiles | n | groups | joints | deepest chain 50 / 90 | labels on the mesh | faces |
|---|---|---|---|---|---|---|
| 1 | 1,864 | 35 / 67 / 159 | 13 / 25 / 61 | 7 / 11 | 13 / 41 / 78 | 321 / 632 / 1,352 |
| 2 | 522 | 41 / 81 / 231 | 15 / 33 / 65 | 8 / 11 | 16 / 53 / 106 | 457 / 945 / 1,703 |
| 3 | 250 | 35 / 81 / 231 | 8 / 34 / 76 | 7 / 13 | 7 / 67 / 161 | 437 / 1,120 / 2,387 |
| 4 | 50 | 63 / 194 / 195 | 24 / 75 / 75 | 10 / 13 | 48 / 122 / 163 | 1,000 / 2,033 / 2,626 |
| 5 | 83 | 70 / 194 / 255 | 21 / 65 / 88 | 10 / 17 | 22 / 144 / 242 | 646 / 2,334 / 3,654 |
| 6 and 7 | 32 | 147 / 238 / 255 | 39 to 53 / 82 to 106 / 106 | 13 to 21 | 53 / 209 to 230 / 251 | 1,792 / 3,108 to 3,231 / 4,723 |

A two-tile creature of the game has about 33 joints on 81 groups and 53 labels at the median; 15 joints on 41 groups
is its simple end (the spectre is 13 on 33). A boss is 75 to 106 joints and uses the whole label space.

## 6. Limits found

- 255 groups, labels 0 to 255, both hard.
- A pivot is an integer (the centroid is truncated) and every turn is fixed-point: on a chain four joints deep the
  edges of a RIGID part change length by up to 3.4 units from pose to pose (measured on the creature made here).
  Detail at the end of a chain should not be finer than about 4 units.
- Unlabelled vertices are label 0. On bases whose root pivot is label 0, anything unlabelled drags the root.
