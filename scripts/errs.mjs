import WebSocket from "ws"; import { targets } from "./cdp.mjs";
const ts = await targets(); const t = ts.find(t => t.type === "page" && !/overlay|hud|region/.test(t.url));
const ws = new WebSocket(t.webSocketDebuggerUrl); await new Promise(r => ws.once("open", r));
let id = 0; const send = (m, p = {}) => ws.send(JSON.stringify({ id: ++id, method: m, params: p }));
const seen = [];
ws.on("message", m => { const d = JSON.parse(m); if (d.method === "Runtime.exceptionThrown") seen.push("EXC " + JSON.stringify(d.params.exceptionDetails).slice(0, 400)); if (d.method === "Runtime.consoleAPICalled" && ["error", "warning"].includes(d.params.type)) seen.push(d.params.type + " " + d.params.args.map(a => a.value ?? a.description).join(" ").slice(0, 300)); });
send("Runtime.enable");
const sleep = ms => new Promise(r => setTimeout(r, ms));
const ev = (e) => new Promise(res => { const i = ++id; const h = m => { const d = JSON.parse(m); if (d.id === i) { ws.off("message", h); res(d.result?.result?.value); } }; ws.on("message", h); ws.send(JSON.stringify({ id: i, method: "Runtime.evaluate", params: { expression: e, awaitPromise: true, returnByValue: true } })); });
for (let k = 0; k < 8; k++) for (let i = 0; i < 5; i++) { await ev(`document.querySelectorAll('.sidebar .nav-item')[${i}].click()`); await sleep(500); }
// settings sub-nav
await ev(`document.querySelectorAll('.sidebar .nav-item')[4].click()`); await sleep(500);
for (let k = 0; k < 3; k++) for (let i = 0; i < 15; i++) { await ev(`document.querySelectorAll('.settings-nav .sub-item')[${i}].click()`); await sleep(350); const ok = await ev(`!!document.querySelector('.settings-body .settings-card, .settings-body .card')`); if (!ok) seen.push("SETTINGS BLANK at " + i); }
console.log([...new Set(seen)].slice(0, 20).join("\n") || "no errors"); ws.close();
