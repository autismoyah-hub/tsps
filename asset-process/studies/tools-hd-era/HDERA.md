# Reading the HD caches

The read-only JS5 reader found 165,517 model groups across the three disks. Every group decompressed and passed the mesh decoder. The comparison disks are the local 239 reference cache and the installed 242 cache identified by its November 3, 2026 info.json. Counts below are records, except where a row explicitly says groups. [Machine-readable census](census.json) · [Raw footer flag counts](inventory.json)

| Content | HD archive and addressing | 561 | 601 | 614 | OSRS 239 | OSRS 242 |
|---|---|---:|---:|---:|---:|---:|
| Animation frame sets | 0, group = set, file = frame | 2,924 groups | 3,229 groups | 3,376 groups | 10,900 groups | 11,314 groups |
| Frame bases | 1 | 2,596 | 2,857 | 2,981 | 2,673 | 2,787 |
| Models | 7, one model per group | 48,609 | 56,930 | 59,978 | 61,606 | 62,769 |
| Textures | 9, one graph per group | 715 | 786 | 863 | 210 | 216 |
| Locations / objects | 16, ID = group × 256 + file | 45,073 | 52,253 | 55,297 | 62,194 | 63,320 |
| NPCs | 18, group × 128 + file | 9,075 | 11,258 | 12,189 | 16,298 | 16,858 |
| Items | 19, group × 256 + file | 15,259 | 18,634 | 19,708 | 33,746 | 34,766 |
| Sequences | 20, group × 128 + file | 12,190 | 13,799 | 14,628 | 14,410 | 15,167 |
| Spot effects | 21, group × 256 + file | 2,143 | 2,617 | 2,784 | 4,008 | 4,448 |

The OSRS disks put those five definition families into archive 2, groups 6, 9, 10, 12 and 13 respectively. Their textures are separate files in archive 9 group 0. HD archive 8 holds sprites. Archive 26 group 0 supplies material properties in parallel arrays. Archive 2 group 32 contains HD base animation sets. NPC opcode 127 selects one. The viewer resolves inventory models for items, body parts for NPCs, and shape 10 or the first listed shape for scenery. Revision 561 uses model/shape pairs on typed locations. The later disks group multiple models under each shape. [239 object reader, line 323]({CLIENT_SRC}/src/object.rs#L323) · [NPC, line 315]({CLIENT_SRC}/src/npc.rs#L315) · [Item, line 110]({CLIENT_SRC}/src/item.rs#L110) · [Sequence, line 156]({CLIENT_SRC}/src/animation.rs#L156) · [Spot, line 81]({CLIENT_SRC}/src/spot_animation.rs#L81)

## Mesh streams

| Layout | 561 | 601 | 614 |
|---|---:|---:|---:|
| Legacy, 18-byte footer | 5,726 | 5,676 | 0 |
| Extended, no explicit version | 42,883 | 51,254 | 39,969 |
| Extended, explicit 14 | 0 | 0 | 1,049 |
| Extended, explicit 15 | 0 | 0 | 18,960 |

The unversioned extended layout is labelled 12 in the reader and inventory. This is a convention for choosing the six-byte scale layout. The files contain no version byte there. An extended footer is 23 bytes including its FF FF marker. Vertex axis masks observed were 0–7, face connectivity opcodes 1–4, mapping types 0–3 and face render values 0–2 on all three disks. The observed feature bits are 1 for per-face render values, 2 for particle attachments, 4 for billboards and 8 for an explicit version immediately before the footer. No other model flag or connectivity opcode was observed. HD never selected the OSRS FF FE or FF FD skinned layouts in this census. [239 footer and sections, model.rs lines 48–138]({CLIENT_SRC}/src/model.rs#L48) · [HD flag interpretation, ModelData.ts line 1216]({TSPS_CHECKOUT}/client/rs/model/ModelData.ts#L1216)

Vertex signed-smart deltas, triangle strips, packed HSL, face priorities, transparency and skin labels share the classic layout. A simple texture record selects three vertex indices. Complex records add three orientation components, three scale values, rotation, direction and translation. Type 2 adds two translation bytes. The scale widths in bytes are 2/2/2, 2/3/2 and 3/3/3 for unversioned, 14 and 15. Rotation, direction and speed each occupy one byte. The 239 reader skips differently sized sections and expects a metadata marker. Feeding the HD bytes straight into it can move the footer or silently misinterpret a mapping. [239 complex skip, line 136]({CLIENT_SRC}/src/model.rs#L136) · [HD mapping widths, line 1551]({TSPS_CHECKOUT}/client/rs/model/ModelData.ts#L1551)

Versioned coordinates have four subunits per game unit. Model 25 has extents 108 × 150 × 194 in 601 and 432 × 600 × 776 in 614, with 169 vertices in both. The classic conversion follows the client's arithmetic shift. Blender retains the original subunit positions and raw values. The projected UVs use the local client's planar, cylindrical, box and spherical mapping, including its rotation, translation and seam handling. Degenerate bases with nonfinite UVs get a per-face unit triangle in Blender and are recorded in the embedded metadata. [Client scaling, lines 1186 and 1537]({TSPS_CHECKOUT}/client/rs/model/ModelData.ts#L1186) · [Client UV calculation]({TSPS_CHECKOUT}/client/rs/model/TextureMapper.ts#L6)

The effect tail consists of counted emitter and effector pairs, followed by counted six-byte billboard entries when their flag is set. The reader checks that it reaches the footer exactly. The interpretation of the pairs as type plus face or vertex binding is inferred from their values and conventional HD layout. The local model source confirms the feature bits. Counts are numbers of **models** carrying at least one record in each category.

| Revision | Emitters | Effectors | Billboards |
|---|---:|---:|---:|
| 561 | 0 | 0 | 0 |
| 601 | 29 | 4 | 6 |
| 614 | 126 | 4 | 59 |

## Materials and conversion

The local procedural client evaluated all 715 + 786 + 863 recipes at 128 × 128, brightness 1, at rest. Each ZIP contains lossless RGBA PNGs named by texture ID and a JSON table of material fields, operation names, sprite dependencies and texture dependencies. The Rust preview applies brightness 0.8 during upload. The 239 and 242 caches instead have seven-byte records identifying a sprite, average HSL, opacity and scroll parameters. Equal texture IDs across revisions do not establish material identity. [HD material reader, lines 17–161]({TSPS_CHECKOUT}/client/rs/texture/ProceduralTextureLoader.ts#L17) · [Operation graph reader, line 18]({TSPS_CHECKOUT}/client/rs/texture/procedural/ProceduralTexture.ts#L18) · [239 texture.rs, lines 13–42]({CLIENT_SRC}/src/texture.rs#L13) · [242 simplified texture reader, line 257]({TSPS_CHECKOUT}/client/rs/texture/SpriteTextureLoader.ts#L257)

| Transfer | Retained | Constraint |
|---|---|---|
| Legacy or unversioned simple mesh | Positions, ordered faces, colours, priorities, alpha, labels, simple mapping triangles | A texture reference still needs a matching target material. |
| Version 14/15 to classic | Integer game-unit geometry and face attributes | Discards the low two coordinate bits where present. |
| Complex mapping to a standalone classic binary | Face texture IDs and geometry | The 239 renderer skips the projection coefficients. The viewer supplies explicit per-corner UVs separately. A new 242 asset needs planar bases, a new texture bake, or equivalent client support. |
| Particles and billboards | Exact records in the Blender payload | Classic conversion omits them. The viewer draws the bind mesh. |
| Texture appearance | Baked RGBA, packed into the blend | Scroll, shader parameters, emissive response and multi-pass material composition need runtime behavior. The classic texel path uses binary coverage at alpha 128 and retains face-level blending. |
| Blender inspection | Raw source bytes, original per-face and per-vertex values, packed images, UVs, mapping parameters and effect records | Blender's material preview has its own lighting and sorting. Stored priority is a face attribute. |

The importer preserves the exact original model in a Blender text block, including fields outside the displayed mesh. Face transparency and texture opacity are independently editable. Priority, colour word, alpha, texture ID, mapping index and render type are integer face attributes. The custom-data values remain available when material slots are shared.

The config decoder extracts fields used by this study. The sequence reader stops after obtaining the frame-list length. Config coverage therefore describes successful field extraction, not a complete disassembly of every sequence command. Object transforms, appearance variants, script-triggered effects and animation playback are outside the static sheets. The OSRS-style layout agreement was tested by the comparisons in [pixel-proof.json](pixel-proof.json).

