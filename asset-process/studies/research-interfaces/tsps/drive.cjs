// Demo driver (never committed): one headless Edge session logged in to the local TSPS client,
// while the scene is arranged through the repository's agent MCP (server started with AGENT_MCP=1).
// Modelled on client/tests/vision-capture.browser.cjs, but the browser stays logged in across
// steps, so MCP actions (equip, walk, attack, talk) show up live in the screenshots.
//
//   node drive.cjs --user meridm --pass vision1234 --steps steps.json --out <dir>
//
// Steps (JSON array, run in order):
//   { "mcp": "<tool>", "args": {...} }        call an agent MCP tool (127.0.0.1:49700)
//   { "sleep": ms }
//   { "camera": { "yaw": 0..2047, "pitch": 0..512 } }
//   { "wheel": { "x": px, "y": px, "deltaY": n, "count": k } }   mouse wheel (camera zoom)
//   { "click": { "x": px, "y": px, "button": "left"|"right" } }
//   { "key": "Escape" | "F1" ... }
//   { "eval": "<js>", "as": "name" }          evaluate in the page, result stored in the log
//   { "shot": "<name>" } | { "burst": { "name": n, "count": k, "everyMs": ms } }
const { spawn, execFileSync } = require("node:child_process");
const fs = require("node:fs");
const net = require("node:net");
const os = require("node:os");
const path = require("node:path");
const http = require("node:http");
const Ws = require((process.env.TSPS_CHECKOUT || "tsps") + "/client/node_modules/ws");

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const arg = (name, def) => {
    const i = process.argv.indexOf(`--${name}`);
    return i >= 0 && i + 1 < process.argv.length ? process.argv[i + 1] : def;
};
const EDGE = arg("browser", (process.env.EDGE || "msedge"));
const URL_BASE = arg("url", "http://localhost:3000/");
const MCP_PORT = Number(arg("mcp-port", "49700"));
const VIEW = (arg("viewport", "1280x800")).split("x").map(Number);
const OUT = arg("out", path.join(os.tmpdir(), "iface-research"));
fs.mkdirSync(OUT, { recursive: true });
const logFile = path.join(OUT, "drive.log");
const log = (line) => { const text = `${new Date().toISOString()} ${line}`; console.log(text); fs.appendFileSync(logFile, text + "\n"); };

function freePort() {
    return new Promise((resolve) => {
        const s = net.createServer();
        s.listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => resolve(p)); });
    });
}
function getJson(url) {
    return new Promise((resolve, reject) => {
        http.get(url, (res) => { let b = ""; res.on("data", (c) => (b += c)); res.on("end", () => { try { resolve(JSON.parse(b)); } catch (e) { reject(e); } }); }).on("error", reject);
    });
}
function mcp(tool, args) {
    const body = JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/call", params: { name: tool, arguments: args ?? {} } });
    return new Promise((resolve, reject) => {
        const req = http.request({ host: "127.0.0.1", port: MCP_PORT, path: "/mcp", method: "POST",
            headers: { "content-type": "application/json", accept: "application/json, text/event-stream" } }, (res) => {
            let b = ""; res.on("data", (c) => (b += c)); res.on("end", () => {
                const line = b.split("\n").find((l) => l.startsWith("data: "));
                try { resolve(JSON.parse(line ? line.slice(6) : b)); } catch { resolve({ raw: b }); }
            });
        });
        req.on("error", reject);
        req.end(body);
    });
}

function cdpClient(wsUrl) {
    const ws = new Ws(wsUrl, { maxPayload: 512 * 1024 * 1024 });
    let id = 0;
    const pending = new Map();
    ws.on("message", (data) => {
        let msg; try { msg = JSON.parse(data.toString()); } catch { return; }
        if (msg.id != null && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); }
    });
    const send = (method, params = {}) => new Promise((resolve, reject) => {
        const my = ++id; pending.set(my, resolve);
        ws.send(JSON.stringify({ id: my, method, params }), (e) => e && reject(e));
        setTimeout(() => { if (pending.has(my)) { pending.delete(my); reject(new Error(`cdp timeout ${method}`)); } }, 90000);
    });
    return {
        opened: new Promise((res, rej) => { ws.on("open", res); ws.on("error", rej); }),
        send,
        async evaluate(expression) {
            const r = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
            if (r.result?.exceptionDetails) throw new Error(r.result.exceptionDetails.exception?.description ?? r.result.exceptionDetails.text);
            return r.result?.result?.value;
        },
        close() { try { ws.close(); } catch {} },
    };
}

const CLIENT = `(window.osrsClient ?? window.__WEBGPU_RENDERER__?.osrsClient ?? window.__osrsClient)`;

async function main() {
    const steps = JSON.parse(fs.readFileSync(arg("steps"), "utf8"));
    const user = arg("user"), pass = arg("pass", "vision1234");
    const target = new URL(URL_BASE);
    target.searchParams.set("username", user);
    target.searchParams.set("password", pass);
    target.searchParams.set("renderer", arg("renderer", "webgpu"));
    const port = await freePort();
    const profile = fs.mkdtempSync(path.join(os.tmpdir(), "iface-research-edge-"));
    const proc = spawn(EDGE, ["--headless=new", "--no-first-run", "--no-default-browser-check", "--mute-audio", "--hide-scrollbars",
        `--window-size=${VIEW[0]},${VIEW[1]}`, `--remote-debugging-port=${port}`, `--user-data-dir=${profile}`, "about:blank"],
        { stdio: "ignore", detached: true });
    proc.unref();
    log(`edge pid ${proc.pid} debug port ${port}`);
    let cdp;
    const shutdown = () => {
        try { cdp?.close(); } catch {}
        try { execFileSync("taskkill", ["/PID", String(proc.pid), "/T", "/F"], { stdio: "ignore" }); } catch {}
        try {
            const out = execFileSync((process.env.POWERSHELL || "powershell"), ["-NoProfile", "-Command",
                `Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*${path.basename(profile)}*' } | ForEach-Object { $_.ProcessId }`], { encoding: "utf8" });
            for (const pid of out.split(/\r?\n/).map((s) => s.trim()).filter((s) => /^\d+$/.test(s))) {
                try { execFileSync("taskkill", ["/PID", pid, "/T", "/F"], { stdio: "ignore" }); } catch {}
            }
        } catch {}
        try { fs.rmSync(profile, { recursive: true, force: true }); } catch {}
    };
    try {
        let tab;
        for (let i = 0; i < 60 && !tab; i++) {
            try { tab = (await getJson(`http://127.0.0.1:${port}/json`)).find((t) => t.type === "page"); } catch {}
            if (!tab) await sleep(500);
        }
        cdp = cdpClient(tab.webSocketDebuggerUrl);
        await cdp.opened;
        await cdp.send("Page.enable");
        await cdp.send("Runtime.enable");
        await cdp.send("Page.navigate", { url: target.href });
        const deadline = Date.now() + Number(arg("login-timeout-ms", "240000"));
        let state = null;
        while (Date.now() < deadline) {
            try {
                state = await cdp.evaluate(`(() => { const c = ${CLIENT}; const r = window.__WEBGPU_RENDERER__ ?? (c && c.renderer);
                    return c && r ? { loggedIn: !!c.isLoggedIn?.(), gameState: c.gameState, maps: r.mapManager?.visibleMaps?.length ?? 0, frames: r.stats?.frameCount ?? 0 } : null; })()`);
            } catch { state = null; }
            if (state && state.loggedIn && state.gameState === 30 && state.maps > 0) break;
            await sleep(1000);
        }
        log(`client state ${JSON.stringify(state)}`);
        if (!state?.loggedIn) {
            await shot(cdp, "login-failed");
            throw new Error("not logged in");
        }
        await sleep(3000);
        // Dismiss the welcome screen the same way the repository's vision harness does.
        log(`welcome ${await cdp.evaluate(`(() => { const c = ${CLIENT}; if (!c?.widgetManager || c.widgetManager.rootInterface !== 378) return "n/a";
            const w = c.widgetManager.getWidgetByUid((378 << 16) | 72); if (!w) return "missing"; c.handleWidgetAction({ widget: w, option: "Play", opIndex: 1, source: "primary" }); return "dismissed"; })()`)}`);
        await sleep(1500);
        for (const step of steps) {
            if (step.mcp) {
                const call = mcp(step.mcp, step.args).then((result) => {
                    const text = result?.result?.content?.map((c) => c.text).join(" ") ?? JSON.stringify(result);
                    log(`mcp ${step.mcp} ${JSON.stringify(step.args ?? {})} -> ${text.slice(0, step.full ? 20000 : 600).replace(/\s+/g, " ")}`);
                }, (e) => log(`mcp ${step.mcp} failed ${e.message}`));
                // "async": fire and keep going (e.g. screenshots while a walk or a fight runs).
                if (!step.async) await call;
            } else if (step.sleep) {
                await sleep(step.sleep);
            } else if (step.camera) {
                const r = await cdp.evaluate(`(() => { const c = ${CLIENT}; ${step.camera.yaw !== undefined ? `c.camera.snapToYaw(${step.camera.yaw | 0});` : ""}
                    ${step.camera.pitch !== undefined ? `c.camera.snapToPitch(${step.camera.pitch | 0});` : ""} return { yaw: c.camera.yaw | 0, pitch: c.camera.pitch | 0 }; })()`);
                log(`camera ${JSON.stringify(r)}`);
                await sleep(step.settle ?? 800);
            } else if (step.wheel) {
                const { x, y, deltaY, count = 1 } = step.wheel;
                for (let i = 0; i < count; i++) {
                    await cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x, y });
                    await cdp.send("Input.dispatchMouseEvent", { type: "mouseWheel", x, y, deltaX: 0, deltaY });
                    await sleep(60);
                }
                await sleep(500);
            } else if (step.click) {
                const { x, y, button = "left" } = step.click;
                await cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x, y });
                await cdp.send("Input.dispatchMouseEvent", { type: "mousePressed", x, y, button, clickCount: 1 });
                await sleep(60);
                await cdp.send("Input.dispatchMouseEvent", { type: "mouseReleased", x, y, button, clickCount: 1 });
                await sleep(step.settle ?? 600);
            } else if (step.move) {
                await cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: step.move.x, y: step.move.y });
                await sleep(step.settle ?? 400);
            } else if (step.key) {
                await cdp.send("Input.dispatchKeyEvent", { type: "keyDown", key: step.key, code: step.key, windowsVirtualKeyCode: step.vk ?? 0 });
                await cdp.send("Input.dispatchKeyEvent", { type: "keyUp", key: step.key, code: step.key, windowsVirtualKeyCode: step.vk ?? 0 });
                await sleep(step.settle ?? 600);
            } else if (step.eval) {
                let value;
                try { value = await cdp.evaluate(step.eval); } catch (e) { value = `error: ${e.message}`; }
                log(`eval ${step.as ?? ""} -> ${JSON.stringify(value)?.slice(0, 4000)}`);
            } else if (step.shot) {
                await shot(cdp, step.shot);
            } else if (step.burst) {
                for (let i = 0; i < step.burst.count; i++) {
                    await shot(cdp, `${step.burst.name}-${String(i).padStart(2, "0")}`);
                    await sleep(step.burst.everyMs ?? 150);
                }
            }
        }
    } finally {
        shutdown();
        log("edge closed");
    }
}

async function shot(cdp, name) {
    const r = await cdp.send("Page.captureScreenshot", { format: "png" });
    const file = path.join(OUT, `${name}.png`);
    fs.writeFileSync(file, Buffer.from(r.result.data, "base64"));
    log(`shot ${file}`);
}

main().catch((e) => { log(`error ${e.stack ?? e}`); process.exitCode = 1; });
