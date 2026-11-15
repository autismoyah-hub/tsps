// #5586 MEASUREMENT: is the Raging Echoes Sage (Wiki old14144) in the cache, and which NPCs named Sage/Leagues Tutor exist?
const path = require("node:path");
const { before, test } = require("node:test");
require("../dist/Server").Server.installProductionPathResolver();
const { CachePipeline } = require("../dist/game/cache/CachePipeline");
const { CacheDefinitions } = require("../dist/game/cache/CacheDefinitions");
before(async () => { await CachePipeline.initialize(path.resolve(__dirname, "..")); });
test("sage npcs", () => {
  for (const id of [14144, 11232, 14145]) {
    let n; try { n = CacheDefinitions.getNpc(id); } catch (e) { console.log(`# SAGE id=${id} ERR ${e.message}`); continue; }
    console.log(`# SAGE id=${id} name=${n?.name} actions=${JSON.stringify(n?.actions ?? n?.options)}`);
  }
  for (const [id, name] of CacheDefinitions.getNpcNames()) if (/sage|tutor/i.test(name) && !/guard/i.test(name)) console.log(`# SAGE-NAME ${id} ${name} ${JSON.stringify(CacheDefinitions.getNpc(id)?.actions)}`);
  console.log(`# SAGE getNpcId league_6_tutor=${CacheDefinitions.getNpcId("league_6_tutor")} league_5_tutor=${CacheDefinitions.getNpcId("league_5_tutor")} league_5_sage=${CacheDefinitions.getNpcId("league_5_sage")}`);
});
