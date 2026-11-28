# What [quoted words withheld] requires (revision 239 client, read from its source)

Source read: `{CLIENT_SRC}/src/` (`cache.rs`, `disk_cache.rs`, `cache_recovery.rs`,
`js5.rs`, `main.rs`, `viewport.rs`, `live_runtime.rs`, `scene.rs`, `terrain.rs`, `world_mesh.rs`, `world_actors.rs`,
`object.rs`, `npc.rs`, `item.rs`, `animation.rs`, `npc_info.rs`, `player_appearance.rs`, `location_update.rs`).
Everything below was then exercised on a private packed copy; the numbers are in `REPORT.md`.

## 1. The cache on disk

- `main_file_cache.dat2` holds every group as a chain of 520-byte sectors; `main_file_cache.idxN` holds, per group of
  archive N, six bytes: length (3) and first sector (3). A sector starts with group id (2 bytes, or 4 when the id is
  above 65535), chunk number (2), next sector (3), archive (1) (`cache.rs read_group_from`; a mismatch is
  [quoted words withheld]).
- A stored group is a container (1 byte compression 0/1/2, 4 bytes length, 4 bytes unpacked length when compressed,
  payload) followed on disk by a version trailer. In this cache every trailer is 4 bytes.
- `idx255` holds one reference table per archive: protocol 7, flags 4 in all the archives we touch (no names, no
  whirlpool; sizes present): the list of group ids, per group a CRC-32 of the container, a version, the two sizes,
  and the list of file ids. A group with several files ends with a chunk table (`cache.rs split_files`).

## 2. What validates it

| check | where | when it bites |
|---|---|---|
| group listed in the reference table | `disk_cache.rs base_group`: [quoted words withheld] | a group written to dat2 without a table entry is invisible |
| version trailer equals the table's version | `validate_container`: **[quoted words withheld]** | this is the message the revision-241 cache gave: a 2-byte trailer holding the low half of a larger version (the OpenRS2 / server disk layout) is refused. The same happens to any group whose trailer and table disagree |
| CRC-32 of the container equals the table's | `validate_container`: [quoted words withheld] | any byte changed without updating the table |
| file count of the group equals the table's | `split_files` | a config group rewritten with more files and an old table |
| reference table against the server's master index (CRC and version) | `disk_cache.rs reference`, `cache_recovery.rs validate_reference` | ONLY in the playable mode (`open_remote`): a table on disk that the server does not vouch for is discarded and fetched from the server |
| 23 reference-table CRCs sent at login | `disk_cache.rs login_crcs`, `live_runtime.rs:331` | the server may refuse a client whose tables differ from its own |

Offline (`DiskCache::open`, used by every example program and by `scene-view`, `scene-render`, the `verify-*`
commands) only the first four apply, and the packer satisfies them. Measured: adding our assets changes the login
CRCs of archives 0, 2, 5 and 7 and no other (`client-verify.log`).

`gielinor-client verify-cache` reports "checksum/version mismatch archive 0 group 0" on the SOURCE cache as well: it
assumes a 2-byte trailer and this cache has 4. It is not a check the game uses and says nothing about our copy.

## 3. Where each kind of asset lives, and which loader reads it

| asset | cache address | client loader | ids: width on the wire | our ids |
|---|---|---|---|---|
| model | archive 7, group = id, file 0 | `Model::decode` via `item_model::Models`, `npc_model::mesh`, `WorldMesh::build` | 32-bit in definitions; **16-bit group in a JS5 request** (`js5.rs request`) | 62000..62202 (+ 1000101 as a disk-only test) |
| object definition | archive 2, group 6, file = id | `object::Definition::load_all` (`decode_local`): models by opcode 6 (id + shape) or 7, 32-bit | 16-bit in `LOC_ADD_CHANGE_V2` (`location_update.rs`), smart-coded in map files | 63000..63046 |
| NPC definition | archive 2, group 9 | `npc::Definition::load_all`: models opcode 61 (32-bit) | **14 bits** in the NPC_INFO spawn record (`npc_info.rs:220`) | 16350, 16351 |
| item definition | archive 2, group 10 | `item::Definitions::load`: models 16-bit (opcodes 1, 23..26, 78, 79, 90..93) or 32-bit (44..54) | equipment value = item + 2048 in 16 bits (`player_appearance.rs`): items up to 63486 | 60000..60007 |
| sequence | archive 2, group 12 | `animation::Sequence::load_all` | 16-bit | 20001..20005 |
| frames | archive 0, group G, file F (a sequence names G<<16 \| F) | `animation_runtime.rs`; `Frame::decode` against the base named by the frame's first two bytes | | groups 20001..20005 |
| frame base | archive 1 | `Skeleton::decode` | | none added (ours borrow real ones) |
| floor underlay | archive 2, group 1 | `floor.rs` | map tile value = id + 1 | 300 |
| map square | archive 5, group = (x << 8) \| y, file 0 terrain, file 1 object placements | `Scene::load_normal` -> `terrain::Region::decode`, `terrain::locations` | | 4374, 4378 |

Three limits found this way, each of which a pack can silently break:
1. **NPC ids must be below 16384.** The source cache's highest is 16300, so 83 ids are free. My first pack used 17000
   and 17001: they read back and drew from the cache without complaint, and no server could ever have spawned them.
2. **Models should stay at or below 65535** if the cache is ever to be fetched from a server: a model packed as group
   1000101 reads from disk (proved) but a JS5 request cannot name it. The owner's server uses ids from 1000000 for
   models it STREAMS in its own packets, which is a different route (section 6).
3. **Worn items must stay at or below 63486**, and NPC, object and sequence ids at or below 65535.

Maps in this cache are not encrypted for this client: `Scene::load_normal` reads every square with the zero key, and
`gielinor-client verify-maps` decodes all 2,933 real squares that way. The square format is the newer one: one group
per square, numbered by position, with numbered files, not the named `m_x_y` / `l_x_y` groups.

## 4. The client's modes, and which need a server

| mode | command | cache opened with | server |
|---|---|---|---|
| playable | `gielinor-client live-view CACHE ADDRESS USER PASSWORD_FILE KEY`, the `authored-live` and `private-authored-test*` binaries, `live_session` | `open_remote` | **required**: JS5 master index, login, then every packet. There is no offline, local or replay switch in these paths; the [quoted words withheld] and the "offscreen" build drive a logged-in session, they do not replace the server |
| offline cache viewport (a real window) | `gielinor-client scene-view CACHE ZONE_X ZONE_Y PLANE` | `open` | none. Draws terrain and map objects of a location; no players, NPCs or interfaces |
| offline picture | `gielinor-client scene-render ...`, `normal_scene_snapshot`, `authored_scene_snapshot`, every `asset_*` / `research_*` / `live_*` example | `open` | none |

So without a server the client can show, in its own window, anything that is in the MAP of the packed cache; and its
world-actor code can be run on hand-made state (`examples/live_world.rs`) to draw NPCs and players. A logged-in
session cannot be had without a server.

## 5. What a server must send for our assets to appear in the playable client

| to show | packet (names from `protocol239.rs`) | fields that carry our ids |
|---|---|---|
| the cache itself | JS5 master index and groups; login reply accepting the CRCs | the server must hold the SAME packed cache: its reference tables are what the client validates the disk against |
| a place | `REBUILD_NORMAL_V2` (zone of the player) | the map square must be in archive 5 |
| an object not in the map | `UPDATE_ZONE_*` + `LOC_ADD_CHANGE_V2` | object id (16 bits), shape, rotation |
| an NPC | `NPC_INFO_SMALL_V5` / `NPC_INFO_LARGE_V5` | NPC id (14 bits); attack and death through the animation update (sequence id, 16 bits): they are NOT in the NPC definition |
| a player's gear | `PLAYER_INFO`, appearance block | twelve equipment values, item + 2048, with the item's 2nd and 3rd wear positions sent as 0; **seven movement sequences** (stand, turn, walk, three turns, run): the server chooses them per weapon |
| an attack or emote | animation update in `PLAYER_INFO` / `NPC_INFO` | sequence id |
| an item in the inventory or on the ground | inventory update; `OBJ_ADD` | item id; the icon is drawn client-side from the definition |

## 6. The owner's server copy (read only; nothing run)

`{TSPS_CHECKOUT}/`:
- `server/` is a TypeScript server speaking its own WebSocket protocol to a browser client (`client/`), and `gateway/`
  translates for the official revision-241 client and serves JS5 "from the cache in `server/target.txt`".
- `server/target.txt` names `osrs-241_2026-10-20`; `server/caches/osrs-241_2026-10-20/` is a **revision 241** cache
  with `keys.json` = `{}` (no map keys). `CachePipeline.ts` downloads that cache by name. The revision-239 client
  refuses a 241 cache (section 2). **Which client the owner plays with against which cache is therefore the first
  thing to settle**: a pack made from the 239 cache serves the 239 client; the server copy as found serves 241.
- Custom items without packing already exist: `server/data/definitions/custom-items.json` (ids 50000..65022, a base
  item for gameplay, `objType` appearance fields, models listed by id from 1000000 with files under
  `server/data/models/`), validated in `src/main/typescript/elvarg/game/cache/CacheDefinitions.ts` and streamed to
  the client before the first appearance update (README, "Custom items without cache packing"). It carries items
  only: [quoted words withheld], and there is no such route for objects, NPCs or maps. Our
  eight items fit that file's rules with their research ids 1000100..1000174 (`research-items/itemcodec.py` has a
  `--server` option for the record; not run here). I found no code in the revision-239 Rust client that receives that stream.
- Where the other data lives: NPC spawns `server/data/definitions/npc-spawns.json` (one row per spawn: id, x, y,
  level); NPC attack / death sequences `npc-combat-defs.json` (`anims`), `npc-captured-animations.json`,
  `npc-animations.json`; object spawns `object-spawns.json` (empty in this copy) and `PacketSender.ts sendObject`;
  a weapon's stand / walk / run / block sequences per item in `item-gameplay.json` (`standAnim`, `walkAnim`, ...);
  attack sequences by style in `item-combat-styles.json`; maps and collision from the cache (`CacheMaps.ts`).

## 7. The packer and the proof programs

`pack.py SOURCE_CACHE OUT_CACHE MANIFEST.json` (this folder) copies the cache and adds models, object, NPC and item
definitions, underlays, sequences with frames, and map squares, refusing any id that exists. `build_inputs.py OUTDIR`
writes the manifest from the project's assets. `readback.py` reads everything back through the client. The step
scripts `step3a.py`, `step3bd.py`, `step3c.py`, `step3e.py`, `step45.py` draw and compare; their logs are beside them.
New example programs in `client/rust-client/examples/` (no change to `src/`): `live_readback.rs`,
`live_scene.rs`, `live_world.rs`, `live_ground.rs`. Build: `cargo +stable-x86_64-pc-windows-gnu build --release
--features custom-assets --example live_scene --example live_readback --example live_world --example live_ground`,
and `--bin gielinor-client` for the client binary.
