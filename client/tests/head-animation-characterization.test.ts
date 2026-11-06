// PRIVATE TEST-ONLY DRAFT. Intended location: client/tests/head-animation-characterization.test.ts.
// Prepared against ce1dd424; not executed/typechecked. No retail displacement expectations.
import assert from "node:assert/strict";
import { test } from "node:test";
import { readFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { createHash } from "node:crypto";
import { loadCacheFiles } from "../scripts/cache/load-util";
import { CacheSystem } from "../rs/cache/CacheSystem";
import { Dat2CacheLoaderFactory } from "../rs/cache/loader/Dat2CacheLoaderFactory";
import { PlayerAppearance } from "../rs/config/player/PlayerAppearance";
import { PlayerModelLoader } from "../rs/config/player/PlayerModelLoader";
import { PlayerPoseGeometry } from "../render/webgpu/actors/actorPlayers";
import { PlayerEcs } from "../game/ecs/PlayerEcs";
import { Model } from "../rs/model/Model";

const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
function moments(model: Model) {
    const arrays = [model.verticesX, model.verticesY, model.verticesZ];
    return arrays.map(a => {
        let min = Infinity, max = -Infinity, sum = 0;
        for (const value of a) { min = Math.min(min, value); max = Math.max(max, value); sum += value; }
        return { min, max, mean: sum / a.length };
    });
}

test("2386 native head poses: raw model, CPU submission and actor position are distinct observations", () => {
    // Existing clienttest runner preserves a fixed environment. Pin the attached cache path
    // relative to the intended client/tests location, not an unforwarded shell variable.
    const dir = resolve(__dirname, "../../server/caches/osrs-241_2026-10-20");
    assert.equal(digest(readFileSync(join(dir, "info.json"))),
        "a90a7f73ac2a4f0cc64f81605cfd15ebcdd53dd451b5e41cd1e12e5a170bac51", "retained r241 metadata");
    // Same native disk/metadata contract as loadCacheSystem(dir), without importing
    // the CS2 sweep's unused VM/widget/UI graph into this model-only fixture.
    const meta = JSON.parse(readFileSync(join(dir, "info.json"), "utf8"));
    const info: any = { name: basename(dir), game: meta.game, environment: meta.environment,
        revision: meta.builds[0].major, timestamp: meta.timestamp, size: meta.size };
    const cache = CacheSystem.fromFiles(info, loadCacheFiles(info));
    assert.equal(info.revision, 241);
    const factory = new Dat2CacheLoaderFactory(info, "dat2", cache);
    const idk = factory.getIdkTypeLoader(), obj = factory.getObjTypeLoader();
    const models = factory.getModelLoader(), textures = factory.getTextureLoader();
    const seqs = factory.getSeqTypeLoader(), frames = factory.getSeqFrameLoader();
    const app = PlayerAppearance.defaultMale(idk);
    const base = new PlayerModelLoader(idk, obj, models, textures).buildStaticModel(app);
    assert.ok(base && base.verticesCount > 0, "native appearance/model precondition");
    assert.ok(base.vertexLabels?.length, "native rig labels required");
    base.calculateBounds();
    const centerX = base.xMid | 0, centerZ = base.zMid | 0;
    const client: any = { textureLoader: textures, seqTypeLoader: seqs,
        seqFrameLoader: frames, skeletalSeqLoader: factory.getSkeletalSeqLoader() };
    const geometry: any = new PlayerPoseGeometry(client, new Map(), new Set(), () => {});
    const ecs = new PlayerEcs(), pid = ecs.allocatePlayer(1);
    ecs.teleport(pid, 100, 100, 0);
    const start = [ecs.getX(pid), ecs.getY(pid), ecs.getLevel(pid)];
    const samples: any[] = [];
    for (const seqId of [808, 819, 6654, 6655]) {
        const seq = seqs.load(seqId);
        assert.ok(seq, `${seqId} sequence loaded`);
        assert.equal(seq.isSkeletalSeq(), false, "this draft requires native keyframes; classify a mismatch, do not invent samples");
        assert.ok(seq.frameIds?.length && seq.frameLengths?.length === seq.frameIds.length);
        let rawDelayOffset = 0, changed = false;
        for (let frameIdx = 0; frameIdx < seq.frameIds.length; frameIdx++) {
            const frameId = seq.frameIds[frameIdx], frame = frames.load(frameId);
            assert.ok(frame, `${seqId}/${frameIdx} native frame and skeleton present`);
            const posed = Model.copyAnimated(base, false, true);
            posed.animate(frame, undefined, !!seq.op14);
            changed ||= posed.verticesX.some((x, i) => x !== base.verticesX[i]) ||
                posed.verticesY.some((y, i) => y !== base.verticesY[i]) ||
                posed.verticesZ.some((z, i) => z !== base.verticesZ[i]);
            const built = geometry.build(base, centerX, centerZ, { seqId, frameIdx });
            assert.ok(built && built[1], `${seqId}/${frameIdx} production CPU posing succeeded`);
            const cpu = built[0];
            assert.ok(cpu.opaqueVertices.length + cpu.alphaVertices.length > 0);
            const matrices = geometry.poseFor(`${seqId}/${frameIdx}`, {
                baseModel: base, baseCenterX: centerX, baseCenterZ: centerZ,
            }, { seqId, frameIdx });
            assert.deepEqual([ecs.getX(pid), ecs.getY(pid), ecs.getLevel(pid)], start,
                "geometry application has not advanced actor world position");
            samples.push({ seqId, frameIdx, frameId, rawDelayOffset, rawFrameDelay: seq.frameLengths[frameIdx],
                raw: moments(posed), submittedBounds: cpu.bounds,
                opaque: digest(cpu.opaqueVertices), alpha: digest(cpu.alphaVertices),
                labelMatrices: matrices ? { length: matrices.length,
                    sha256: digest(new Uint8Array(matrices.buffer, matrices.byteOffset, matrices.byteLength)) } : null,
                actorFine: start });
            rawDelayOffset += seq.frameLengths[frameIdx];
        }
        assert.ok(changed, `${seqId} positive animation control changes native geometry`);
        console.log(JSON.stringify({ seqId, rawFrameDelaySum: rawDelayOffset, frameCount: seq.frameIds.length }));
    }
    const translated = Model.copyAnimated(base, false, true);
    translated.translate(128, 0, 0);
    assert.equal(moments(translated)[0].min - moments(base)[0].min, 128,
        "model-space observation detects a known translation");
    assert.deepEqual([ecs.getX(pid), ecs.getY(pid), ecs.getLevel(pid)], start);
    ecs.teleport(pid, 101, 100, 0);
    assert.equal(ecs.getX(pid) - start[0], 128, "actor-world observation detects an explicit tile move");
    assert.equal(ecs.getY(pid), start[1]);
    console.log(JSON.stringify({ cache: info, appearance: {kits: app.kits, equip: app.equip}, samples }));
});
