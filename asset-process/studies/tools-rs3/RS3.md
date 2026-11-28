# Reading build 951.1

The English cache catalogue on this machine dates archive 2773 to November 23, 2026. Its reference tables describe the complete archive. The supplied disk ends during group 48892 of index 40. Index 47 advertises 146,939 single-file groups; its local index file is zero bytes. A complete sector walk accounts for all 16,777,214 nonreserved sectors. The next sector required by the last audio chain is 16,777,215, just outside the file. The local catalogue describes a 25,353,455,332-byte archive, 49,838,978 blocks and `disk_store_valid=false`. A complete export needs a representation capable of addressing that size. [Sector evidence](disk-audit.json) · [Local catalogue]({RS3_DIR}/caches.json)

## Archive directory

Group and file counts come from archive 255. The last column counts positive-length disk index entries. It includes the incomplete chain in index 40. Blank archive numbers have no local reference-table entry. Older index names are carried forward only where the local client or the payload supports that assignment. Each unclassified row remains an open identification. [TSPS IndexType.ts]({TSPS_CHECKOUT}/client/rs/cache/IndexType.ts) · [Sample group headers](inventory.json)

| Index | Evidence or likely role | Advertised groups | Advertised files | Indexed locally |
|---:|---|---:|---:|---:|
| 1 | Animation skeletons / frame bases (TSPS assignment) | 5,331 | 5,331 | 5,331 |
| 2 | Configuration families | 40 | 148,682 | 40 |
| 3 | Interfaces | 1,889 | 104,277 | 1,889 |
| 5 | Maps | 8,715 | 41,740 | 8,715 |
| 8 | Sprites | 36,638 | 36,638 | 36,638 |
| 10 | Binary | 1 | 1 | 1 |
| 12 | Client scripts (data only) | 21,207 | 21,207 | 21,207 |
| 13 | Font metrics | 60 | 60 | 60 |
| 14 | Audio sample data (OggS signature in group 0) | 60,934 | 60,934 | 60,934 |
| 16 | Locations / objects | 554 | 141,509 | 554 |
| 17 | Enumerations (TSPS assignment) | 69 | 16,783 | 69 |
| 18 | NPCs | 258 | 32,953 | 258 |
| 19 | Items | 253 | 64,254 | 253 |
| 20 | Sequences | 300 | 38,317 | 300 |
| 21 | Spot effects | 38 | 9,491 | 38 |
| 22 | Config records, parameter maps observed; detailed family uncertain | 1,678 | 53,606 | 1,678 |
| 23 | World-map data (area names observed) | 5 | 8,362 | 5 |
| 24 | Message templates (texts observed) | 2 | 3,326 | 2 |
| 26 | Material definitions | 1 | 25,407 | 1 |
| 27 | Particle configuration (TSPS assignment) | 2 | 4,578 | 2 |
| 28 | Defaults (TSPS assignment) | 11 | 11 | 11 |
| 29 | Unclassified | 1 | 569 | 1 |
| 32 | Unclassified | 11 | 11 | 11 |
| 33 | Unclassified | 3 | 3 | 3 |
| 34 | Unclassified | 11 | 11 | 11 |
| 35 | Unclassified | 54 | 54 | 54 |
| 40 | Audio (OggS signature in group 0), last indexed group incomplete | 86,271 | 86,271 | 48,888 |
| 41 | Unclassified | 774 | 916 | 0 |
| 42 | Unclassified | 142 | 142 | 0 |
| 47 | NXT model catalogue (inferred from definition joins and build-950 documentation) | 146,939 | 146,939 | 0 |
| 48 | Candidate animation archive; contents unavailable | 33,548 | 988,130 | 0 |
| 49 | Unclassified | 265 | 559 | 0 |
| 52 | Texture catalogue variant; material ID joins | 39,919 | 39,919 | 0 |
| 53 | Texture catalogue variant; comparable group IDs | 39,919 | 39,919 | 0 |
| 54 | Texture catalogue variant; fewer groups | 39,868 | 39,868 | 0 |
| 55 | Texture catalogue variant; comparable group IDs | 39,919 | 39,919 | 0 |
| 56 | Unclassified | 3,083 | 3,083 | 0 |
| 57 | Unclassified | 41 | 5,138 | 0 |
| 58 | Unclassified | 225 | 225 | 0 |
| 59 | Unclassified | 5 | 5 | 0 |
| 60 | Unclassified | 235 | 235 | 0 |
| 61 | Unclassified | 1,509 | 1,509 | 0 |
| 62 | Unclassified | 7 | 7 | 0 |
| 65 | Unclassified | 2 | 30 | 0 |
| 66 | Unclassified | 9 | 9 | 0 |
| 67 | Unclassified | 36 | 36 | 0 |
| 255 | Reference tables | — | — | 46 |

## Disk containers and references

A six-byte index entry contains two big-endian 24-bit integers, stored size and first sector. Each 520-byte sector identifies its group, sequence number, next pointer and archive. Group IDs through 65535 use a two-byte group number and 512 data bytes. Larger IDs use four bytes and 510 data bytes. The reader checks each header before interpreting payload bytes. Container types supported are uncompressed, headerless BZip2, gzip and Jagex LZMA with a five-byte properties header. The decompression paths bound output size. Multi-file groups use the HD reader's cumulative signed chunk lengths. No script from the cache is evaluated. [Disk reader](rs3cache.py) · [HD reference and chunk reader](../tools-hd-era/hdcache.py)

The reference tables encode delta group and file IDs, optional name hashes, CRCs, sizes and version numbers. Large IDs in protocol 7 occupy four bytes with the high bit set. In the size-bearing tables the stored group length excludes the two-byte disk version trailer. Expanded lengths and CRC32s matched every group in 16, 18, 19, 21 and 26, including multi-file groups. This validates the reference fields used to select duplicate candidates. [Reference reader](references.py) · [Readback](verification.json)

## Definitions and lookup coverage

| Family | Address | Records | Name field reached | Model list reached | Reached terminator at exact end |
|---|---|---:|---:|---:|---:|
| Item | 19, group × 256 + file | 64,254 | 46,432 | 50,073 | 42,477 |
| NPC | 18, group × 128 + file | 32,953 | 26,079 | 26,602 | 25,253 |
| Object | 16, group × 256 + file | 141,509 | 46,976 | 122,612 | 114,997 |
| Spot | 21, group × 256 + file | 9,491 | — | 9,491 | 9,489 |

Item opcode 1 gives the inventory model. NPC opcode 1 gives a counted list. Object opcode 1 gives shape types and counted lists of models for each shape. The lookup preserves those lists and selects shape 10 when present. Model references use a two-byte value or a high-bit-tagged four-byte value. Short 32767 represents an empty slot. Opcode 2 holds a zero-terminated display name for the three named families, and a sequence reference for spots. Opcode 40 carries recolour pairs and 41 material replacement pairs. Item fields 23 and 25 give worn references. Derived items and state-driven object/NPC transforms can omit direct fields. The SQLite catalogue records the first unsupported opcode and byte offset for partial reads, and retains decoded prefixes. Search results cover direct extracted names. [Extractor](definitions951.py) · [Catalogue](assets.sqlite)

The HD definitions supplied the initial field-width table. TSPS item and NPC sources confirm the big-smart model convention, but widths differ between branches. For example, in 951 the male/female model fields can occur consecutively without the older byte offset after each one. Additional 951 widths are inferred from their alignment across records. Reaching the terminator constrains widths without proving field semantics. [HD definitions](../tools-hd-era/definitions.py) · [TSPS item reader]({TSPS_CHECKOUT}/client/rs/config/objtype/ObjType.ts) · [NPC reader]({TSPS_CHECKOUT}/client/rs/config/npctype/NpcType.ts) · [ByteBuffer.readBigSmart]({TSPS_CHECKOUT}/client/rs/io/ByteBuffer.ts)

## Materials and texture addressing

Group 0 of archive 26 holds 25,407 files. File ID is the material key. For example, echo NPC 28070 substitutes material 13920 for 13926. Material records contain numeric texture references. Across the two common layouts all 47,045 extracted references join advertised archive-52 groups. Archives 53–55 have overlapping group namespaces. Group membership validates the ID field, but does not determine channel meaning, texture encoding or image content. [Decoded fields and remaining bytes](materials.json) · [Observed material flag counts](control-check.json)

| First byte | Files | Inferred reference layout |
|---|---:|---|
| 0 | 16,276 | Two prefix bytes and a big-endian 32-bit flag word. Bit 0 selects a four-byte reference. Bits in mask 0x0a select one additional reference when either is present. Flag 0x10 selects a reference in the 168 records of that form. All 22,999 references resolve in the directory. |
| 1 | 9,130 | A 32-bit flag word followed by one tag byte and one big-endian 32-bit texture ID for each of bits 0x20, 0x40 and 0x80 that is set, in that order. Slot counts are 9,130, 8,190 and 6,726. |
| 2 | 1 | Material 23093. Its prefix resembles layout 1. The complete raw record is retained as unknown. |

Material 10456 has flag word 0x002340ec, three tag-2 references 19591, 19592 and 19593, and a property tail. Interpreting aligned words in this tail as big-endian floats yields 32, 10 and approximately 0.015. Material 10453 has two tagged references and the same three numerical values. The reader preserves the prefix, flags, tags and entire remaining tail. Sampler settings, alpha mode, diffuse/normal/emissive assignment and the meaning of these scalar properties require independent evidence. No diffuse image bytes were available for PNG decoding. Archive 8 contains sprites with a separate namespace. [Reader](materials951.py) · [Property evidence](materials.json)

## Mesh layout, coordinates and proof boundary

There are **zero observable 951 mesh headers**. Model version and flag histograms, vertex stream layouts, UV orientation, weight encoding, face ordering and differences against 239/HD cannot be measured here. A local guide for **950** reports four source units per OSRS unit, +Y up, stored material ID plus one, and skin flag 0x20 with u16 bone and byte weight streams. The inspection backend implements the proposed basis, mapping (x,y,z) to Blender (x,z,y)/4 and reversing winding. Its synthetic test exercises that convention. It provides no independent verification of the 951 basis. [Build-950 guide, section 10]({TEMP}/TSPS-RS3-Item-Porting-Guide.pdf) · [Blender backend](rs3blend.py)

No resident group in any supplied 951 archive matches an archive-47 or archive-52–55 reference by expanded length and CRC32. A separate sweep decompressed all 61,606 OSRS 239 models and 165,517 HD 561/601/614 models. Zero matched a 951 model reference by that pair. This rules out byte-identical fallback payloads in those model banks. Equal geometry can have different bytes. **Geometry comparisons performed: 0. Number of identical meshes: undetermined.** [In-cache scan](references.json) · [Comparison scan](candidate-proof.json)

