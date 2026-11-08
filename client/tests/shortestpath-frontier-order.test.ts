import { strict as assert } from "node:assert";
import { packWorldPoint as p } from "../game/plugins/shortestpath/WorldPointUtil";
import { CollisionMap } from "../game/plugins/shortestpath/pathfinder/CollisionMap";
import { Pathfinder } from "../game/plugins/shortestpath/pathfinder/Pathfinder";
import type { PathfinderConfig, PathTransport } from "../game/plugins/shortestpath/pathfinder/PathfinderConfig";
import { FLAG_EAST, FLAG_NORTH, packRegionPosition, SplitFlagMap } from "../game/plugins/shortestpath/pathfinder/SplitFlagMap";

// Synthetic open rectangle crosses x=3200 (region edge) and y=3525 (wilderness edge).
// It is wholly on plane 0; the start and target are walkable and no real map is loaded.
const regions = new Map<number, Uint8Array>();
function openEdge(x: number, y: number, flag: number): void {
    const rx = Math.floor(x / 64), ry = Math.floor(y / 64);
    const key = packRegionPosition(rx, ry);
    let bytes = regions.get(key);
    if (!bytes) regions.set(key, (bytes = new Uint8Array(64 * 64 * 4 / 8)));
    const bit = ((y % 64) * 64 + x % 64) * 4 + flag;
    bytes[bit >> 3] |= 1 << (bit & 7);
}
for (let x = 3190; x <= 3206; x++) {
    for (let y = 3520; y <= 3530; y++) {
        if (x < 3206) openEdge(x, y, FLAG_EAST);
        if (y < 3530) openEdge(x, y, FLAG_NORTH);
    }
}
const map = new CollisionMap(new SplitFlagMap(regions));
const start = p(3200, 3525, 0);
const outsideStart = p(3200, 3524, 0);
const target = p(3197, 3524, 0);
const teleport: PathTransport = {
    index: 7, origin: -1, destination: target, duration: 1, additionalCost: 0,
    delayedVisit: false, differentialCost: 0, isTeleport: true, maxWildernessLevel: 0,
};
function search(from: number, teleports: readonly PathTransport[]) {
    const config: PathfinderConfig = {
        map, transports: new Map(), teleports, transportDestinations: new Set(teleports.map(t => t.destination)),
        avoidWilderness: true, unreachableTargetDistance: 0, collisionAwareBlockedTargets: false,
        calculationCutoffMillis: 60_000,
    };
    return new Pathfinder(config, from, [target]).run();
}

// Controls: open-ground Chebyshev walking optimum is 3, and a teleport is admitted
// immediately from outside wilderness. Both also check target and route reconstruction.
const walk = search(start, []);
assert.equal(walk.reached, true);
assert.equal(walk.cost, 3);
assert.equal(walk.path.at(-1)?.packed, target);
assert.ok(walk.path.every(step => step.transport === -1));
const outside = search(outsideStart, [teleport]);
assert.equal(outside.reached, true);
assert.equal(outside.cost, 1);
assert.equal(outside.path.at(-1)?.packed, target);
assert.equal(outside.path.at(-1)?.transport, teleport.index);

// Intended contract: walk one step south, then teleport for one more cost unit.
// Source inspection predicts a cost-3 walk because the zero-cost abstract node is
// queued behind cost-2 walking nodes that mark the target visited before it expands.
const exitThenTeleport = search(start, [teleport]);
assert.equal(exitThenTeleport.reached, true);
assert.equal(exitThenTeleport.cost, 2);
assert.equal(exitThenTeleport.path.at(-1)?.packed, target);
assert.equal(exitThenTeleport.path.at(-1)?.transport, teleport.index);
