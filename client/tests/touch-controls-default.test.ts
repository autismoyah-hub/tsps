import assert from "node:assert/strict";
import { test } from "node:test";
import { ConfigChanged } from "@runelite/api/events";
import { ConfigManager } from "@runelite/client/config/ConfigManager";
import { PluginInjector } from "@runelite/client/plugins/PluginInjector";
import { GameState } from "../game/login/GameState";
import { PadMode, TouchControlsConfig } from "../game/plugins/touchcontrols/TouchControlsConfig";
import { TouchControlsPlugin, type TouchControlsClient } from "../game/plugins/touchcontrols/TouchControlsPlugin";

// Disclosed capability fixtures; these test the connected plugin/config state,
// not physical device classification. Browser coverage also checks the real HUD.
type Profile = { pointer: "coarse" | "fine" | "none"; hover: boolean; points: number; touchStart?: boolean };
const phone: Profile = { pointer: "coarse", hover: false, points: 1, touchStart: true };
const laptop: Profile = { pointer: "fine", hover: true, points: 10, touchStart: true };

async function fixture(profile: Profile, run: (plugin: TouchControlsPlugin, config: ConfigManager) => void,
    search = "", mediaSupport = true): Promise<void> {
    const savedWindow = Object.getOwnPropertyDescriptor(globalThis, "window");
    const savedNavigator = Object.getOwnPropertyDescriptor(globalThis, "navigator");
    const media: Record<string, boolean> = {
        "(pointer: coarse) and (hover: none)": profile.pointer === "coarse" && !profile.hover,
    };
    Object.defineProperty(globalThis, "window", { configurable: true, value: {
        location: { search }, addEventListener() {}, removeEventListener() {},
        ...(profile.touchStart ? { ontouchstart: null } : {}),
        ...(mediaSupport ? { matchMedia: (query: string) => ({ matches: media[query] ?? false }) } : {}),
    } });
    Object.defineProperty(globalThis, "navigator", { configurable: true, value: { maxTouchPoints: profile.points } });
    const client: TouchControlsClient = {
        camera: { getYaw: () => 0 } as TouchControlsClient["camera"],
        gameState: GameState.LOGGED_IN, isLoggedIn: () => true,
        widgetManager: { interfaceParents: new Map(), isEffectivelyHidden: () => false },
    };
    const config = new ConfigManager(undefined);
    const plugin = new PluginInjector(config).provide(ConfigManager, config).runWith(() => new TouchControlsPlugin(client));
    try {
        await plugin.start();
        plugin.onClientTick();
        run(plugin, config);
    } finally {
        await plugin.stop();
        if (savedWindow) Object.defineProperty(globalThis, "window", savedWindow);
        else Reflect.deleteProperty(globalThis, "window");
        if (savedNavigator) Object.defineProperty(globalThis, "navigator", savedNavigator);
        else Reflect.deleteProperty(globalThis, "navigator");
    }
}

test("Default hides the joystick on a touch-capable laptop with a fine hovering primary pointer", async () => {
    await fixture(phone, plugin => assert.equal(plugin.getUiState().visible, true,
        "positive control: logged-in touch-first plugin is enabled and can display controls"));
    await fixture(laptop, plugin => assert.equal(plugin.getUiState().visible, false,
        "TOUCH5133_FINE_PRIMARY_DEFAULT: secondary touchscreen must not enable the laptop joystick"));
});

for (const points of [0, 1, 10]) {
    test(`Default shows coarse non-hover primary controls even when maxTouchPoints is ${points}`, async () => {
        await fixture({ ...phone, points }, plugin => assert.equal(plugin.getUiState().visible, true));
    });
}

for (const [name, profile] of [
    ["non-touch desktop", { pointer: "fine", hover: true, points: 0 }],
    ["legacy touch event without touch-first input", { pointer: "fine", hover: true, points: 0, touchStart: true }],
    ["coarse hovering primary", { pointer: "coarse", hover: true, points: 1 }],
    ["no primary pointer", { pointer: "none", hover: false, points: 0 }],
] as const) {
    test(`Default hides controls for ${name}`, async () => {
        await fixture(profile, plugin => assert.equal(plugin.getUiState().visible, false));
    });
}

test("unknown media-query support stays hidden; explicit URL force still works", async () => {
    await fixture(phone, plugin => assert.equal(plugin.getUiState().visible, false), "", false);
    await fixture(laptop, plugin => assert.equal(plugin.getUiState().visible, true), "?touch=1", false);
});

test("URL overrides and live explicit On/Off keep precedence over Default", async () => {
    for (const [profile, search, defaultVisible] of [[laptop, "?touch=1", true], [phone, "?touch=0", false]] as const) {
        await fixture(profile, (plugin, config) => {
            assert.equal(plugin.getUiState().visible, defaultVisible);
            for (const [mode, expected] of [[PadMode.OFF, false], [PadMode.ON, true], [PadMode.DEFAULT, defaultVisible]] as const) {
                config.setConfigValue(TouchControlsConfig, "padMode", mode);
                plugin.onConfigChanged(new ConfigChanged(TouchControlsConfig.group, "padMode", undefined, mode));
                assert.equal(plugin.getUiState().visible, expected, `live ${mode} with ${search}`);
            }
        }, search);
    }
});
