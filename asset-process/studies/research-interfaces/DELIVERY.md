# Delivery: how an original interface reaches the game the owner runs (TSPS)

Read from `{TSPS_CHECKOUT}` (branch `local/meridian-demo`, base `main` 1e2fdbff; nothing changed there) and
`{TSPS_CHECKOUT}-plugins`. Line numbers are from those trees. The end-to-end test in section 3 ran in a
private scratch copy (`{ROOT}/tsps-scratch/tsps`), loopback only.

## 1. The channels that exist today

| Channel | Exists | Evidence | What it carries | Limits |
|---|---|---|---|---|
| **A. Server-defined interfaces** | yes | `server/src/main/typescript/elvarg/game/interfaces/CustomInterfaceRegistry.ts:12-55` (definition `{groupId, widgets[], lifecycle?}`; served at `GET /api/interfaces/<groupId>`); plugin API `registerCustomInterface` `server/src/.../plugins/PluginTypes.ts:1367`, `PluginManager.ts:4549-4555`; builder `server/plugins/interface/widgetGroup.js:32-80`; rules `server/docs/agents/interfaces.md` (line 120: `overlay: true` = non-modal panel) | a whole widget group (layers, rectangles, text, graphics, models) in the client's own widget fields | no script channel of its own (see E); 30000+ group ids are not in the official client behind the RuneLite gateway |
| B. Client fetch of a group it does not have | yes | `client/game/OsrsClient.ts:3048-3079`: an open for a group not in the cache fetches `fetchInterfaceDefinition`, `setCustomInterface`, mounts it and replays updates sent with the open | makes A need no client change | first open of a session waits on one HTTP fetch |
| **C. Server sprites** | yes | `server/plugins/world/CustomSprites.plugin.js:10-53` serves `server/data/sprites/<id>[-name].png` at `/api/sprites/<id>`; `client/custom/sprites/ServerSprites.ts:11,26` (ids from 30000) fetched from `client/widgets/gl/texture-cache.ts:271` when a sprite is neither cached nor overridden | new art for graphic widgets: `spriteId: 300xx` | PNG files cannot travel in a Plugin Hub ZIP (format 1: one `.plugin.js`, `tsps-plugins/README.md`); a second plugin registering `"sprites"` would replace this handler |
| D. Reusing cache interfaces with server data | yes | open/close/mount: `ClientProtocol.ts:2326` `encodeWidgetOpenSub` (no id whitelist), `PacketSender.ts:428-447, 1157, 1595-1623, 1742-1810` (open, hide, colour, sub-interface, events, `sendInterfaceScript`, `sendClientScript`) | any cache interface, its texts, colours, items, varps, and any cache client script | the look is the cache's; layout only as far as the cache scripts allow |
| D1. The boss health bar | yes | `server/plugins/interface/BossHud.plugin.js:1-69`: events `boss-hud:show {player, npcId, current, maximum, colours?}`, `update`, `fade`, `hide`, `reset`, `extra` (a second bar); interface 303 mounted at login (`ClientProtocol.ts:2803-2804`) | a working boss bar for any NPC today, name from the NPC definition | colours, fade and a second bar only; no timer, no custom art |
| E. Custom client scripts (CS2) | **no** | `client/game/cs2/ClientScriptLoader.ts` loads from the cache only; the only replaced scripts are bundled RuneLite patches checked by hash (`client/rs/cs2/runeliteScripts.ts`) | - | a main-repo change |
| F. `custom-definitions.json` kinds for interfaces or sprites | **no** | `client/custom/definitions/CustomDefinitionStore.ts:6,18`: only `"npc" | "loc" | "item"` | - | - |
| G. Client-drawn overlays (RuneLite-style client plugins) | yes, client code | `client/game/plugins/ClientPluginManager.ts:335-337` (`widgetOverlay`, `widgetOverlays()`); worked example `client/game/plugins/opponentinfo/OpponentInfoPlugin.ts` (reads `HPBAR_HUD_*` varps and varbits, draws with `drawTextGL`); server-to-overlay by varps: `server/plugins/interface/BuffTimers.plugin.js:8-12` writes 7990-7993, `client/game/plugins/statustimer/StatusTimerOverlay.tsx:30-33` reads them | anything drawn per frame (smooth timers, bars that move every frame) | only through a pull request to the main repo; a Hub plugin cannot ship client code (`notes/CONTRIBUTING-NOTES.md` section 1) |

**What decides what the server can open:** nothing filters group ids. `encodeWidgetOpenSub` writes `groupId & 0xffff`
(`ClientProtocol.ts:2326-2348`); `InterfaceLayoutRegistry` (`game/definition/InterfaceLayoutDefinition.ts`) is a
lookup of named layouts, not a validator. The client opens a cache group, or a server-defined group it can fetch,
and logs an error for anything else (`OsrsClient.ts:3066-3071`).

## 2. Which repository

- A server-defined interface (A), its behaviour and the events that drive it are plugin code. Bundled with TSPS:
  `tsps/server/plugins/...` ("Plugins first", `server/AGENTS.md`). As a Hub package in `tsps-plugins`: allowed if it
  needs no files besides its one `.plugin.js` (the widget JSON can live inside the JS).
- New art (C) needs PNG files in `server/data/sprites`: the main repo, or the owner's own server data folder. Not a
  Hub ZIP today.
- A client overlay (G) or a script channel (E): the main repo only, by pull request.

## 3. Proven end to end in this run (scratch copy, nothing pushed)

The proof rebuild of PET_INSURANCE (`proof/rebuild-148.json`, 49 static components) was converted by
`to_tsps.py` to TSPS widget JSON (`proof/rebuild-148-tsps-widgets.json`, group 30900) and registered by a 20-line
plugin placed only in the scratch copy (`tsps-scratch/tsps/server/plugins/research/IfaceResearch.plugin.js`).
`GET /api/interfaces/30900` served it; `::interface 30900` opened it in the main modal (161:16) of the TSPS
client. Screenshot `tsps/proof/g30900.png` against the real 148 in the same session (`tsps/proof/g148.png`):
**0 of 120,000 pixels differ**, and 0 against iface.py's drawing of the same JSON (`board-04-proof.png`). So an
interface authored in our format reaches the TSPS client, unchanged, through channel A, with cache sprites.
Not tested: a new sprite through channel C, an overlay (`overlay: true`) mount, clicks.

## 4. What a plugin-side boss HUD looks like

1. **Bar only, today (D1):** the boss's fight plugin emits `boss-hud:show {player, npcId, current, maximum}` on
   engage, `boss-hud:update {current}` on each hit, `boss-hud:hide` on death. Its name comes from the NPC
   definition; colours by `colours: [back, sliding, remaining]` in 15-bit RGB (default `[25600, 576, 800]`,
   `BossHud.plugin.js:54-55`). Hub-compatible.
2. **Mechanic timer or custom boss bar (A, optionally C):** a group 30xxx built from our rules (INTERFACES.md
   section 7: dark box, b12/p11 orange name, red-under-green bar, value centred), registered with
   `registerCustomInterface({groupId, widgets, overlay: true})`, opened into a HUD pane with the usual
   sub-interface packet, and driven each tick with `sendInterfacePosition`/size, `sendString` and
   `sendInterfaceColour`, then closed. Tick resolution (600 ms) unless a cache `onTimer` script can be borrowed
   (unproven). Art: cache sprites (Hub-compatible) or `data/sprites/300xx.png` (main repo or owner's data).
   Which HUD pane to mount into on each layout (161/164/548/601) was not settled in this run.
3. **Frame-smooth timer or bar (G):** a client plugin reading server-set varps, as `StatusTimerOverlay` does; a
   pull request to the main repo, with reserved varps.
