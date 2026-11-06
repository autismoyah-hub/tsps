// Source-only diagnostic fixture. Run solely through a qualified clienttest route.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { test } from 'node:test';
import ts from 'typescript';
import * as protocol from '../../server/src/main/typescript/elvarg/net/protocol/ClientProtocol';
import { PlayerDialogue } from '../../server/src/main/typescript/elvarg/game/model/dialogues/entries/impl/PlayerDialogue';
import { decodeServerPacket } from '../network/packet/ServerBinaryDecoder';
import { decodeAppearanceBinary } from '../game/sync/AppearanceDecoder';
import { PlayerAppearance } from '../rs/config/player/PlayerAppearance';
import { PlayerEcs } from '../game/ecs/PlayerEcs';
import { CacheSystem } from '../rs/cache/CacheSystem';
import { getCacheLoaderFactory } from '../rs/cache/loader/CacheLoaderFactory';
import { loadCache, loadCacheInfos } from '../scripts/cache/load-util';
import { WidgetManager } from '../widgets/WidgetManager';
import { PendingInterfaceUpdates } from '../widgets/custom/PendingInterfaceUpdates';
import { HeldWidgetProperties } from '../widgets/HeldWidgetProperties';
import { ChatheadFactory } from '../render/ChatheadFactory';
import { PlayerChatheadFactory } from '../render/PlayerChatheadFactory';

const read = (relative: string) => ts.createSourceFile(relative,
    fs.readFileSync(path.resolve(__dirname, relative), 'utf8'), ts.ScriptTarget.Latest, true);
function findOne(source: ts.SourceFile, select: (node: ts.Node) => string | undefined) {
    const found: string[] = [];
    const visit = (node: ts.Node) => { const text = select(node); if (text) found.push(text); ts.forEachChild(node, visit); };
    visit(source);
    assert.equal(found.length, 1, 'exactly one complete production seam');
    return found[0];
}
const compile = (code: string) => ts.transpileModule(code, {
    compilerOptions: { target: ts.ScriptTarget.ES2020, module: ts.ModuleKind.CommonJS },
}).outputText;
const clientSource = read('../game/OsrsClient.ts');
const widgetCallback = findOne(clientSource, n => ts.isVariableDeclaration(n) &&
    n.name.getText(clientSource) === 'handleWidgetPayload' ? n.initializer!.getText(clientSource) : undefined);
const method = (name: string) => findOne(clientSource, n => ts.isMethodDeclaration(n) &&
    n.name.getText(clientSource) === name ? n.getText(clientSource).replace(/^private /, '') : undefined);
const Receiver = new Function('PlayerAppearance', 'HELD_WIDGET_ACTIONS', compile('class Receiver {' +
    method('buildPlayerAppearanceFromPayload') + method('holdForUnloadedGroup') + '} return Receiver;'))(
    PlayerAppearance, new Set(['set_player_head']));
const overlaySource = read('../widgets/gl/widgetsOverlayFactory.ts');
const modelCallback = findOne(overlaySource, n => ts.isPropertyAssignment(n) &&
    n.name.getText(overlaySource) === 'getRenderModelCanvas' ? n.initializer.getText(overlaySource) : undefined);
const senderSource = read('../../server/src/main/typescript/elvarg/net/packet/PacketSender.ts');
const headMethod = findOne(senderSource, n => ts.isMethodDeclaration(n) &&
    n.name.getText(senderSource) === 'sendPlayerHeadOnInterface' ? n.getText(senderSource).replace(/^public /, '') : undefined);
const scriptMethod = findOne(senderSource, n => ts.isMethodDeclaration(n) &&
    n.name.getText(senderSource) === 'sendClientScript' ? n.getText(senderSource) : undefined);
const HeadSender = new Function('encodeWidgetSetPlayerHead', 'encodeRunClientScript',
    compile('class Sender {' + headMethod + scriptMethod + '} return Sender;'))(
        protocol.encodeWidgetSetPlayerHead, protocol.encodeRunClientScript);
const fingerprint = (m: any) => {
    assert.ok(m && m.verticesCount > 0 && m.faceCount > 0, 'native model prerequisites');
    return createHash('sha256').update(JSON.stringify([
        m.verticesCount, m.faceCount, ...['verticesX','verticesY','verticesZ','indices1','indices2','indices3',
        'faceColors1','faceColors2','faceColors3','faceTextures'].map(k => m[k] ? Array.from(m[k]) : null),
    ])).digest('hex');
};

test('native dialogue head uses transformed NPC geometry and restores player appearance', () => {
    const infos = loadCacheInfos();
    const info = infos.find(i => i.name === 'osrs-241_2026-10-20');
    assert.ok(info, 'the explicitly named active-cache version is available');
    assert.equal(info.revision, 241);
    const loaded = loadCache(info);
    const cache = CacheSystem.fromFiles(info, loaded.files);
    const loaders: any = getCacheLoaderFactory(info, cache);
    const modelLoader = loaders.getModelLoader(), textureLoader = loaders.getTextureLoader();
    const idkTypeLoader = loaders.getIdkTypeLoader(), objTypeLoader = loaders.getObjTypeLoader();
    const npcTypeLoader = loaders.getNpcTypeLoader();
    const wm = new WidgetManager(cache);
    wm.getGroup(217);
    const uid = (217 << 16) | 2;
    const widget = wm.getWidgetByUid(uid)!;
    assert.ok(widget, 'real dialogue head widget loaded');
    const ecs = new PlayerEcs();
    const index = ecs.allocatePlayer(1);
    const client: any = Object.assign(new Receiver(), { widgetManager: wm, playerEcs: ecs,
        controlledPlayerServerId: 1, pendingInterfaceUpdates: new PendingInterfaceUpdates(),
        heldWidgetProperties: new HeldWidgetProperties() });
    const handle = new Function(compile('return ' + widgetCallback)).call(client);
    const npcFactory = new ChatheadFactory(modelLoader, textureLoader);
    const playerFactory = new PlayerChatheadFactory(modelLoader, textureLoader, idkTypeLoader, objTypeLoader);
    // Only the final GPU/canvas sink is replaced; both selected model factories are real.
    const host: any = { osrsClient: { ...client, modelLoader, textureLoader, idkTypeLoader,
        objTypeLoader, npcTypeLoader }, model2DRenderer: {
            renderModelInstanceToCanvasExtents: (model: any) => model,
        } };
    const render = new Function('host','PlayerChatheadFactory','ChatheadFactory',
        compile('return (' + modelCallback + ')();'))(host, PlayerChatheadFactory, ChatheadFactory);
    const sent: Uint8Array[] = [];
    const sender: any = Object.assign(new HeadSender(), { player: { getSession: () => ({
        sendClientPacket: (p: Uint8Array) => sent.push(p),
    }) }, sendChatboxInterface(group: number) { assert.equal(group,217); return this; },
    sendInterfaceAnimation() { return this; }, sendString() { return this; },
    sendInterfaceFlagsRange() { return this; } });
    const dialoguePlayer: any = { getPacketSender: () => sender, getDisplayName: () => 'Protocol2410' };
    const normal = PlayerAppearance.defaultMale(idkTypeLoader);
    const failures: string[] = [], rows: any[] = [];
    function step(label: string, transform: number, colors = [0,0,0,0,0]) {
        const wire = protocol.encodePlayerAppearance({ gender: normal.gender, colors,
            kits: normal.kits, equip: normal.equip, npcTransformationId: transform },
            'Protocol2410', 3, 0, new Array(7).fill(-1));
        const decoded = decodeAppearanceBinary(wire)!;
        assert.ok(decoded, 'genuine encoded appearance decodes');
        assert.equal(decoded.npcTransformationId, transform);
        const appearance = client.buildPlayerAppearanceFromPayload({ ...decoded, equip: decoded.equipment });
        ecs.setAppearance(index, appearance);
        assert.equal(ecs.getAppearance(index)?.npcTransformationId, transform);
        sent.length = 0;
        PlayerDialogue.send(dialoguePlayer, "'obby-lobby' - what?", 588);
        const messages = sent.map(p => decodeServerPacket(p) as any);
        const heads = messages.filter(m => m?.type === 'widget' && m.payload?.action === 'set_player_head');
        assert.equal(heads.length, 1, 'real PlayerDialogue -> production PacketSender emits one head packet');
        assert.equal(sent.length, 2, 'dialogue also emits its real text-alignment script packet');
        const message: any = heads[0];
        assert.equal(message.type, 'widget');
        assert.equal(message.payload.action, 'set_player_head');
        assert.equal(message.payload.uid, uid);
        handle(message.payload);
        const actual = render(widget.modelId, { widget }, 100, 100);
        let expected;
        if (transform >= 0) {
            const npc = npcTypeLoader.load(transform);
            assert.ok(npc?.chatheadModelIds?.length, 'active-cache NPC has a real head');
            expected = npcFactory.get(npc);
        } else expected = playerFactory.get(appearance);
        const actualHash = fingerprint(actual), expectedHash = fingerprint(expected);
        rows.push({label,transform,actualHash,expectedHash,heads: transform < 0 ? undefined : npcTypeLoader.load(transform).chatheadModelIds});
        if (actualHash !== expectedHash) failures.push(label);
        return { actualHash, expectedHash };
    }
    const ordinary = step('ordinary', -1);
    const gnocci = step('gnocci', 7972);
    const repeat = step('repeat-gnocci', 7972);
    const bugs = step('different-npc', 7969);
    const npcColors = step('gnocci-player-colors-ignored', 7972, [1,0,0,0,1]);
    const restored = step('restored', -1);
    const recolored = step('ordinary-recolor', -1, [1,0,0,0,1]);
    const repeatOrdinary = step('repeat-restored', -1);
    console.log('CHATHEAD_PROTOCOL_2410 ' + JSON.stringify({ cache: info.name, rows }));
    assert.notEqual(gnocci.expectedHash, bugs.expectedHash, 'distinct actual NPC heads provide a non-vacuous identity control');
    assert.equal(gnocci.expectedHash, npcColors.expectedHash, 'player palettes do not recolor NPC head');
    assert.notEqual(ordinary.expectedHash, recolored.expectedHash, 'native normal player recolor control is effective');
    assert.equal(restored.actualHash, ordinary.actualHash);
    assert.equal(repeatOrdinary.actualHash, ordinary.actualHash);
    assert.equal(repeat.actualHash, gnocci.actualHash);
    assert.deepEqual(failures, [], 'full decoded head->widget->selected native model contract');
    ecs.reset();
});
