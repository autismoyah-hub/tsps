# Getting the project's custom assets into TSPS

Read-only investigation of the TSPS copy at
`{TSPS_CHECKOUT}/` (called `TSPS/` below).
Nothing in that copy was changed, built or run. The only program run was my own read-only cache comparison,
`{ROOT}/studies/research-tsps/compare_caches.py`, on the project's 239 cache and TSPS's 241 cache.

**Status of each statement:** "read" = I read the file and lines cited. "Inferred" = follows from what I read but
no code path was run. [quoted words withheld] = needs the client running.

## Headline

- The TSPS client is a **browser TypeScript client** that reads the cache over HTTP from the same directory the
  server uses. It is not the Rust tooling client and it has **none of that client's CRC / version-trailer / login-CRC
  checks** (I found none in its store code). So [quoted words withheld] is much easier than LIVE.md
  feared.
- Three custom-content channels exist that need no cache packing: **custom items with streamed models**
  (data file, works today), **definition bundles for NPC, object and item definitions** (works, but only a plugin
  can install one; there is no data file), and **world overlays** (object spawns, NPC spawns, terrain replacement
  of existing map squares).
- Nothing streams **sequences, frames, frame bases, spot animations, sprites or brand-new map squares**. Those
  need either a packed cache or new code.
- **Several of the project's ids collide with real 241 content** (models 62000..62202, objects 63000..63046,
  NPCs 16350/16351). They were free in 239 and are taken in 241.
- The 239 and 241 **binary formats are the same** for everything the project encodes; the differences are a few
  added opcodes and the on-disk version trailer (4 bytes in the project's cache, 2 in TSPS's).

## 1. What the TSPS client is

| question | answer | where (read) |
|---|---|---|
| language, lineage | TypeScript, React shell, fork of xRSPS | `TSPS/README.md:3,11`; `TSPS/docs/ARCHITECTURE.md:3` |
| renderer | WebGL through PicoGL (`client/picogl`, `client/package.json:25`), plus a WebGPU renderer (`client/render/webgpu/`, `docs/webgpu-renderer.md`, not read) | |
| how it gets the cache | **HTTP, static files.** It fetches `<base>/caches/caches.json`, picks the newest entry, then fetches `keys.json`, the `idx` files and `main_file_cache.dat2` from `<base>/caches/<name>/`. With Range support it loads the dat2 sparsely, group by group ([quoted words withheld]), and keeps fetched ranges in browser storage | `client/game/Caches.ts:15,58-78,120-183`; `client/config/clientEnv.ts:55-62`; `client/rs/cache/js5/Js5RangeClient.ts:21-27`; `client/rs/cache/CacheInfo.ts` (`getLatestCache`) |
| where those files come from | Development: the dev server serves `../server/caches` at `/caches`. Production: `ensure-cache --publish` copies the cache named in `server/target.txt` | `client/craco.config.js:191,234-235`; `docs/cache-revision-update.md:139-146` |
| revision | **241**: `server/target.txt` = `osrs-241_2026-10-20`; `server/caches/caches.json` and `info.json` say revision 241, OpenRS2 id 2746, `keys.json` = `{}` | verified from those files |
| how client and server agree | (1) Same directory: the server writes `caches.json` for its active cache (`CachePipeline.ts:86-99`) and the client reads it. (2) Login carries the revision **number**; the server refuses a mismatch ([quoted words withheld]) (`NetworkBuilder.ts:1128-1131`; client sends it at `client/network/serverConnection/connection/init.ts:105`). That is all: no per-archive CRCs at login | |
| integrity checks on the cache | The server accepts any directory that has `main_file_cache.dat2`, `idx255`, `info.json`, `keys.json` and a long enough `idx255`; otherwise it downloads from OpenRS2 (`CachePipeline.ts:9-14,75-80,105-108`). The name must match `osrs-<revision>_<yyyy-mm-dd>` (`CachePipeline.ts:34-40`). In the client's store code I found no CRC or group-version check (grep of `client/rs/cache/store/`, `Container.ts`; **inferred from absence**, not proven) | |
| stale browser copies | Persisted ranges are dropped when the dat2's `ETag` / `Last-Modified` / size changes (`Caches.ts:580-605`) | |
| the other client | `gateway/` lets the official 241 client connect through RSProx and "serves JS5 from the cache in `server/target.txt`"; its README says custom items and TSPS interfaces (group 30000 up) do not show there (`gateway/README.md:36,84-86`). Streamed content is browser-client only | |

Correction to LIVE.md: the "Custom items without cache packing" text is in `TSPS/server/README.md:69-99`, not the
root README. The other LIVE.md claims about this copy (241 cache, `custom-items.json`, ids 50000..65022, models
from 1000000) are confirmed.

## 2. Existing channels for custom content

All streamed content rides one packet: `sendContentData(source, datasets)`
(`server/.../net/packet/PacketSender.ts:1541`), which is deflated JSON in a `GAMEMODE_DATA` packet
(`ClientProtocol.ts:1009-1020`), unpacked by `loadFromPayload` in
`client/common/gamemode/GamemodeContentStore.ts:115` onward. **Each packet must fit 64 KiB after deflate.**

### 2a. Custom items and streamed models (works today, data file only)

- Data: `server/data/definitions/custom-items.json`; model files under `server/data/models/`.
  Record: `id` (50000..65022), `baseItemId` (a real item supplying equip slot, bonuses, requirements and
  animations), `objType` (any appearance field of the item definition: name, examine, `model`, `maleModel`,
  `maleModel1/2`, `femaleModel*`, head models, 2D zoom and angles, recolours, actions, note links), optional
  `itemDef` (gameplay overrides), `models` (`"<id>": "<path>.dat"`, ids from 1000000).
- Server: `server/plugins/items/ItemDefinitionLoader.plugin.js:262-318` (reads the file, checks each model is
  18 bytes..1 MiB and that its packet fits, checks an item does not name a missing streamed model), `:345-352`
  (sends `customModels` one per packet, then `customItems`, at login), `:371-373` (also HTTP `/api/custom-items`,
  `/api/custom-models`). Validation: `server/.../game/cache/CacheDefinitions.ts:436-517`.
- Client: `GamemodeContentStore.ts:257-275` fills `CustomItemRegistry` (`client/custom/items/CustomItemRegistry.ts:20-24`,
  same id range) and `CustomModelRegistry` (`client/custom/items/CustomModelRegistry.ts:6-17`). Items resolve
  through `CustomObjTypeLoader` (`client/rs/cache/loader/Dat2CacheLoaderFactory.ts:241-243`). **Models resolve in
  the general model loader, before the cache, for any caller**: `client/rs/model/ModelLoader.ts:30-35`. Render
  workers get both datasets (`client/game/worker/RenderDataWorkerPool.ts:75-77,177-183`).
- Carries: inventory, ground, worn male and female forms, head models, recolour and retexture to existing
  textures. Does not carry: its own animations (taken from the base item), textures
  (`server/README.md:94`).
- Tested against the content present: one item, **Dragonic katana**, id 50000, base 4587, models 1000000
  (`dragonic-katana/drop.dat`, 5131 bytes) and 1000001 (`equip.dat`, 6303 bytes). Both files are in the **old
  model format** (no `ff ff/fe/fd` trailer), which `ModelData.decode` accepts along with the three trailer formats
  (`client/rs/model/ModelData.ts:540-551`). A server test covers it (`server/tests/presets-persistence.test.cjs:211-246`).

### 2b. Definition bundles: NPC, object ("loc") and item definitions as native bytes

- Purpose as written: keep definitions that a newer cache removed ([quoted words withheld],
  `server/docs/definition-archives.md`).
- Format: a bundle `{ id, targetCache {name, game, revision}, sourceCache (full cache info), definitions: [{ kind:
  "npc"|"loc"|"obj", symbol, sourceId, id, dataBase64, crc }] }` (`client/rs/config/ArchivedDefinitionRegistry.ts:11-26`;
  the server has a mirror of the same 199-line file).
- Rules (same file): target must equal the active cache (`:85-87`); source revision must be `<=` the target's
  (`:88-94`); ids below 65535 (`:44`); NPC and object ids must be **at or above the cache's own count**, items may
  also replace an id whose current name is "null" (`:118-127`); payload CRC-32 checked (`:117`); each definition is
  decoded strictly, to its terminator, **with the source revision's rules** (`client/rs/config/ArchivedTypeLoader.ts:19-47`);
  links between definitions (transforms, notes, placeholders) must stay inside the bundle (`:138-165`).
- Server: `server/plugins/world/DefinitionArchives.plugin.js:1-45`. Installed only by a plugin emitting
  `definition-archives:install`; sent at login and after each install; the whole bundle list must fit one packet.
  **No data file and no installer exist in this copy**; the only callers are tests
  (`server/tests/archived-definition-registry.test.cjs:173-214`). The default is empty.
- Client: dataset `definitionArchives` (`GamemodeContentStore.ts:121-125`), feeding the NPC, object and item
  loaders (`Dat2CacheLoaderFactory.ts:205-243`).
- Carries: complete native NPC, object and item definitions, so a definition encoded for 239 can be delivered
  marked as revision 239. Does not carry: models ([quoted words withheld],
  `definition-archives.md:27-29`), sequences, anything else.
- **Inferred, not run:** a bundled NPC or object definition whose model fields hold ids from 1000000 (32-bit
  opcodes: object 6/7 at `client/rs/config/loctype/LocType.ts:250-277`, NPC 61/62 at `npctype/NpcType.ts:254-264`)
  would draw models streamed by channel 2a, because the model loader checks the streamed registry first. The
  streamed-model list is only fed from `custom-items.json`, but the loader does not require the listing item to
  use the model, so an item record can carry models for NPCs and objects.

### 2c. NPC model overrides

Server sends slot -> model id replacements for an existing NPC type; the client makes a synthetic type
(`client/custom/npcs/NpcModelOverrideRegistry.ts`, `CustomNpcTypeLoader.ts`; wire: `ClientProtocol.ts:164-165,2904,3149`,
`client/game/sync/NpcUpdateDecoder.ts:408-417`). Carries models only; name, size and animations stay the base
NPC's. Combined with 2a (inferred) this puts a custom model on an existing NPC with no new definition.

### 2d. World: spawns and terrain

- Objects: `server/data/definitions/object-spawns.json` (`[]` in this copy), rows `{ id, position {x,y,z}, type,
  face, remove }` (`.../definition/loader/impl/ObjectSpawnDefinitionLoader.ts:10-36`). Client datasets
  `worldLocSpawns`, `worldLocChanges` (`GamemodeContentStore.ts:8-28,277-297`).
- NPCs: `server/data/definitions/npc-spawns.json`, rows `{ id, x, y, level, wanderRadius }`.
  NPC type ids are 16 bits on TSPS's own wire (`ClientProtocol.ts:3526`), so the Rust client's 14-bit limit does
  not apply to the browser client.
- Terrain: `server/data/regions/<regionId>.pack` (28-byte header, object file, terrain file), loaded at start
  (`.../collision/RegionManager.ts:47-75`) and sent as a region replacement packet; plus dataset
  `worldTerrainOverrides`. **Only for map squares the active cache already has**: the pack's file ids must match
  the cache's (`.../collision/MapRegionReplacementManager.ts:346-352`), and the server's region list is built from
  the cache (`RegionManager.ts:43-46`). The directory does not exist in this copy.

### 2e. Interfaces and sprites

- Server-defined interfaces: `api.registerCustomInterface({ groupId, widgets, ... })`, served at
  `/api/interfaces/<groupId>` (`server/docs/agents/interfaces.md:1-13`;
  `.../game/interfaces/CustomInterfaceRegistry.ts:22-49`). Plugin code, not a data file.
- Sprites: the client has `client/widgets/gl/spriteOverrides.ts` (images drawn in place of a sprite id), used by
  a client plugin. I found no server channel feeding it (grep only; file not read in full).

### 2f. Not present

Sequences, frames, frame bases, spot animations, textures, new map squares, underlays: cache only. The sequence
loader has no custom wrapper (`Dat2CacheLoaderFactory.ts:261` onward; grep for "custom" in the sequence and
spot-animation loaders finds nothing).

## 3. Each kind of asset the project makes

(a) existing channel as is; (b) packed cache; (c) new code.

| asset | route | what it takes |
|---|---|---|
| **Items, all forms** (project ids 60000..60007) | **(a)** | Append records to `server/data/definitions/custom-items.json`; put model files under `server/data/models/<folder>/`; model ids from 1000000 (the project's research ids 1000100..1000174 fit). Item ids 60000..60007 are inside 50000..65022 and above 241's highest item (34731). Each needs a `baseItemId`. A weapon's stand, walk and attack sequences come from the base item, so only existing sequences |
| **Models for anything** | **(a)** as riders on an item record (inferred), or **(c)** tiny | Cleaner: let a file list models with no item. About 20 lines in `ItemDefinitionLoader.plugin.js` (or a `custom-models.json` read by the same loop). Likely acceptable upstream: it removes a quirk |
| **Object definitions** (project 63000..63046) | **(c) small** on top of 2b; or (b) | **Renumber**: 241 has objects up to 63306. Use 63307..65534. Needs an installer for bundles: a plugin of about 40 lines (`server/plugins/world/CustomDefinitions.plugin.js`) that reads e.g. `data/definitions/custom-definitions.json` and emits `definition-archives:install`. Then place with `object-spawns.json`. Models by 32-bit ids from 1000000. Not determined: whether the server-side mirror decodes and builds collision for such an object exactly as the client draws it |
| **NPC definitions** (project 16350, 16351) | **(c) small**, same installer; or (b) | **Renumber**: 241 has NPCs up to 16771. Use 16772 up. Spawn rows in `npc-spawns.json`; combat rows in `npc-combat-defs.json`. Stand and walk sequences inside the definition, and attack/death sequences in server data, must be sequences the cache has |
| **Sequences, frames, frame bases** (project sequences and frame groups 20001..20005) | **(b)** now; **(c) medium** later | (c) sketch: new dataset `customSeqs` (sequence definition bytes, frame group files, base files as base64); wrappers on the sequence loader, `Dat2SeqFrameLoader` (`client/rs/model/seq/SeqFrameLoader.ts:57`) and `IndexSeqBaseLoader` (`SeqBaseLoader.ts:11`) in the style of `CustomNpcTypeLoader`; forward the dataset to render workers (`RenderDataWorkerPool.ts:75-77`); server-side lookup for sequence lengths. Roughly 200..300 lines client, 80 server, plus tests. Frame groups can exceed 64 KiB, so it needs chunked delivery or an HTTP endpoint like `/api/custom-models`. Upstream: plausible as a general [quoted words withheld] feature, but it is the largest piece and would want discussion first |
| **Map squares** (project 4374, 4378; neither exists in 241) | **(b)** | Existing squares can be replaced by `.pack` files (a); new squares cannot. (c) would touch `CacheMaps`, `RegionManager`, `MapRegionReplacementManager` and the client scene loader; size not determined; not worth it now |
| **Underlay 300** | **(b)** | 241's highest underlay is 255, so 300 is free. No streaming channel |
| **Spot animations, projectiles** (coming) | **(b)**; **(c) small** once custom sequences exist | A spot animation is a model plus a sequence; a wrapper on the spot-animation loader is about 60 lines, but it depends on the sequence channel. Projectiles are sent by the server by spot-animation id (`.../game/model/Projectile.ts`, not read in detail) |
| **Interface sprites** (coming) | **(b)** (sprite archive 8), or **(c) small** | A dataset that calls the existing sprite-override function with image data: about 30..50 lines (inferred from names only). Custom interfaces themselves already have a channel (2e) |

### What route (b), a packed cache, takes

1. Re-target `pack.py` to the 241 cache (section 4) and write the result as a new directory
   `server/caches/osrs-241_<date>/` with `info.json` (revision 241) and `keys.json` (`{}`); the date makes the name
   differ from the stock cache so the server does not treat it as the download and browsers do not reuse stored
   ranges.
2. Put that name in `server/target.txt`. The server then regenerates `caches.json`, and the client takes the
   cache from there. Login still passes because only the number 241 is compared.
3. Everything in the pack then works through ordinary cache reads on both sides, including sequences and new map
   squares. The official client through the gateway would in principle receive it too (the gateway serves JS5 from
   the same cache); not examined.

Costs: every upstream cache update means re-packing; identifier generation (`scripts/generate-identifiers.ts`)
and startup scans will see the new ids; it is a private fork of the data and **cannot be contributed upstream**
(the cache is downloaded, not in the repository). Not determined without running: whether every server startup
scan tolerates the added entries.

## 4. Revision 239 against 241

Measured with `compare_caches.py` (project cache
`{OSRS_CACHE_239}` against
`TSPS/server/caches/osrs-241_2026-10-20`):

| | 239 (project) | 241 (TSPS) |
|---|---|---|
| reference tables, archives 0, 1, 2, 5, 7 | protocol 7, flags 4 | protocol 7, flags 4 (same) |
| **version trailer after each group** | **4 bytes** | **2 bytes** |
| models: groups / highest id | 61606 / 61605 | 62714 / **62775** |
| objects: highest id | 62200 | **63306** |
| NPCs: highest id | 16300 | **16771** |
| items: highest id | 65534 (an oddity of that copy) | 34731 |
| sequences: highest id | 14428 | 15162 |
| frame groups / bases: highest id | 14521 / 2673 | 14927 / 2786 |
| underlays: highest id | 241 | 255 |
| map squares | 2934, files 0..4 per square, unnamed | 2950, same layout |
| model format markers (sample, and newest 1500) | `fffe` and `fffd`; newest all `fffd` | identical mix |

Same ids, byte-for-byte equal in both: underlays 100%, objects 99.0%, NPCs 97.2%, sequences 99.6%, spot
animations 99.8%, items 82.9% (241 re-encoded many items); sampled models 1497 of 1502, frame groups 1557 of 1558,
frame bases 2655 of 2673. So Jagex did not re-encode these archives between the revisions.

From the code: TSPS's decoders for items, NPCs, objects, sequences, spot animations and models contain **no
revision test between 226 and 241** (grep of `client/rs/config/*type`, `client/rs/model`), and
`docs/cache-revision-update.md:156-163` lists what 241 added: item opcodes 15, 160, 251, later 99 and 161;
sequence opcode 19; NPC / object / spot-animation opcode 42; map files unencrypted. All additions; nothing the
project's encoders write changed meaning.

Re-targeting the packer therefore involves:
1. **Trailer**: write 2-byte version trailers (and read them) instead of 4.
2. **Ids**: move models off 62000..62202 (use 62776 up, or stay with streamed ids from 1000000), objects off
   63000..63046, NPCs off 16350/16351. Sequences and frame groups 20001..20005, underlay 300 and squares 4374 and
   4378 are free in 241 (checked for 4374, 4378, 20001, 20005).
3. Re-run the free-id check and the reference-table round trip against the 241 tables.
4. Proof: the Rust client refuses this cache layout (LIVE.md section 2), so proof on 241 means the TSPS client
   or its tests (`cache-config-decode` decodes every config entry strictly).
The encoders themselves need no change (inferred from the above; not proven by a decode in TSPS).

## 5. Recommendation

**Tier 1, no code changes**
- Items with all forms, through `custom-items.json` and `data/models/`.
- Placing existing or custom-modelled content in the world: `object-spawns.json`, `npc-spawns.json`, terrain packs
  for existing squares.
- If the owner accepts a private data fork: a packed 241 cache (route b) for sequences, map squares, underlays.

**Tier 2, a small contribution (candidates for a pull request)**
1. A data-file installer for definition bundles, so NPC and object definitions need no hand-written plugin
   (about 40 lines plus a documented JSON file and a test). Worth asking the maintainers first whether they want
   new content in the [quoted words withheld] registry or a sibling registry with honest naming; the provenance
   fields were designed for restored Jagex content.
2. Streamed models not tied to an item (about 20 lines).
3. Later, custom sequences / frames / bases, then spot animations and sprites on top. This is the piece that
   would [quoted words withheld], and the one to discuss before writing.

**Tier 3, not worth doing now**
- New map squares without a packed cache.
- Making streamed content appear in the official client through the gateway.
- Re-proving on the Rust client against 241.

### Cheapest end-to-end demonstration (for approval; nothing done)

One project item, visible in inventory, on the ground and worn, in the TSPS browser client. To be done in a
**working copy** of TSPS, not the cold-storage copy. Files written:

1. `server/data/models/<item-name>/inv.dat`, `male.dat`, `female.dat`: the project's model files as they are
   (any of the four model formats; each under 1 MiB and under 64 KiB once base64-encoded and deflated).
2. `server/data/models/<item-name>/README.md`: attribution, as the katana has.
3. `server/data/definitions/custom-items.json`: one record appended, e.g. id 60000, a `baseItemId` of the same
   equipment kind, `objType` with `name`, `model: 1000100`, `maleModel: 1000101`, `femaleModel: 1000102`, the
   secondary models set to -1, the 2D zoom and angles from the project's item definition, and
   `models: { "1000100": "<item-name>/inv.dat", ... }`.

Then start the server and client (`npm run setup`, `yarn start`, which downloads nothing new if the cache is
present), log in as an administrator, `::item 60000 1`, wield it, drop it. No cache is touched, and the change is
three data files that can be deleted to undo it.

What that will and will not prove: it proves the project's model encoding and item appearance fields in the real
client. Not determined until it runs: whether the project's models use anything `ModelData.decode` treats
differently from the Rust client (textures, priorities, labels for animation), and the icon framing.
