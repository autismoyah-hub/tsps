# Revision 239 model materials

The source references below are relative to `{CLIENT_SRC}/src/`. All multibyte integers in these streams are big-endian. The face index is the join key for colour, texture, mapping selector, transparency, shading, priority, label and depth offset. Reordering faces can change the picture even when those streams stay aligned.

## Disk layout

`model.rs:49–92` selects four layouts. Suffix FF FF selects extended, FF FE skinned legacy, FF FD skinned extended. The unmarked legacy footer occupies 18 bytes, the two middle formats 23 bytes each, and skinned extended 26 bytes. The suffix counts toward that size. The footer starts with two u16 counts (vertices and faces), then the u8 number of texture-space triangles. This last count is independent of how many faces use a texture.

The subsequent u8 fields in order are render-stream flag, priority (255 selects one byte per face), alpha flag, face-skin flag, face-texture flag in extended formats only, vertex-skin flag, and bone-weight flag in skinned formats only. Next come u16 byte lengths for X, Y, Z and face-index deltas. Extended layouts add a u16 selector-stream byte length. Skinned layouts add a u16 vertex-label/weight-stream length. Boolean presence flags are 0 or 1.

| Layout | Data sections in file order |
|---|---|
| Legacy and skinned legacy | Vertex delta flags, face strip opcodes, optional priorities, face labels, face render/texture flags, vertex labels/weights, face transparency, face index deltas, u16 face colours, texture basis triangles, X/Y/Z deltas, skinned trailer when applicable. |
| Extended and skinned extended | Mapping types, vertex delta flags, optional render types, face strip opcodes, optional priorities, face labels, vertex labels/weights, face transparency, face index deltas, face texture IDs, compact mapping selectors, u16 face colours, X/Y/Z deltas, simple bases, complex mapping sections, metadata marker/payload, skinned trailer when applicable. |

The decoder slices these sections at `model.rs:93–154`. A missing optional stream occupies zero bytes. Alpha uses exactly one unsigned byte per face when present. With the stream absent the accessors supply zero. Preserve the raw byte, including 254 and 255. The optional metadata after the mapping tables comprises a marker followed by three u16s and one u32 if present. The skinned trailer has a flag and optionally one signed depth-offset byte per face. See `model.rs:283–301`.

**Legacy texture selection.** In each face's render byte, bit 0 selects flat shading, bit 1 enables texture, and bits 2–7 select one of the mapping triangles. For a textured face the u16 colour field carries the texture ID and the decoded colour becomes 127. FFFF denotes absent texture. Mapping entries are ordered triples of u16 vertex indices. A basis identical to the ordered face indices is simplified to the implicit basis by the client. See `model.rs:214–240,303–328`.

**Extended texture selection.** An enabled texture stream contains one u16 per face. Zero means untextured, otherwise subtract one to obtain the ID. A textured face takes one selector byte only when the model has a mapping table. Zero selects that face's own corners. Other bytes are one plus the unsigned table index. Untextured faces consume no selector. When the table is empty, all textured faces use their own vertices. Render type and painted u16 colour are independent fields. See `model.rs:163–170,239–253` and `world_mesh.rs:482–497`.

**Basis triangles.** Each type-0 entry consumes three u16 vertex indices in P, M, N order. The points can belong to a different face, lie outside the face, or span several faces. P is the origin, M−P the U axis, N−P the V axis. The client solves the two-axis projection for each rendered corner and falls back to the unit triangle for a degenerate determinant. The classic rasterizer receives the camera-space basis for perspective sampling. See `model.rs:283–295`, `world_mesh.rs:482–515`, `render3d.rs:1259–1284,1445–1454,1481–1494,1618–1639`. Keeping only the texture ID or rotating the face corners can change the image.

Extended types 1, 2 and 3 have separate packed sections. For C nonzero types and K type-2 entries their lengths in order are 6C, 6C, 2C, C and 2C+2K bytes. This Rust decoder skips those sections and leaves their mapping triples zero. The local indexed importer retains the encoded sections and type order. See `model.rs:93–103,135–139,283–295`.

## Drawing

Textured faces use light intensity clamped to 2–126. HSL faces light their painted lightness component. Raw alpha 254 forces lighting kind 3 and 255 forces kind 2. Hidden faces have shade sentinel −2. Nontextured kind 3 uses flat shade 128. The model accessor normalizes 255 to 253, and near clipping can restore the raw byte. See `model_light.rs:144–188`, `model.rs:42–44`, `model_alpha.rs:2–18`, `render3d.rs:1336`.

For ordinary values, 0 is opaque and larger bytes admit more background. Untextured classic blending truncates foreground × (256−alpha)/256 and background × alpha/256 separately. Flat 254 has a special scanline shift. Opaque and alpha phases, face priorities and depth ordering matter. For a 128-pixel cache texture, the textured span samples perspective coordinates in blocks, skips zero texels in nonopaque textures, modulates the foreground by light and blends the background for nonzero face alpha. The alpha branch omits the opaque depth write. See `model_alpha.rs:9–44`, `render3d.rs:839–865,1192–1212,1445–1545`, `texture_span.rs:98–136`. Texture cutouts and face transparency can therefore occur independently or together.

## Cache inventory and contact sheet

Archive 9, group 0 contains **210** definition file IDs. The complete set is **0 through 53 and 55 through 210**, inclusive. ID 54 is absent. Texture 0 is valid. Each seven-byte definition stores u16 sprite ID, u16 average HSL, u8 opaque flag, u8 scroll direction and u8 speed. The referenced sprite is file 0 of archive 8, group equal to that sprite ID. See `texture.rs:14–51`. Sprite metadata, palette, storage order and optional alpha plane are decoded at `sprite.rs:22–108`.

[The contact sheet](board-textures.png) labels every definition by texture ID in ascending order, using its first sprite, offsets and native palette at rest. Black represents palette slot zero. These thumbnails show source colours before scene lighting, brightness correction or scrolling. [survey.py](survey.py) reads actual file IDs from the cache reference table.
