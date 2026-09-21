// Minimal CDP client for the WebView2 remote-debugging port (test harness only).
import WebSocket from "ws";
export async function targets(port = 9222) {
  for (let i = 0; i < 60; i++) { try { const r = await fetch(`http://127.0.0.1:${port}/json`); const j = await r.json(); if (j.length) return j; } catch {} await new Promise(r => setTimeout(r, 500)); }
  throw new Error("no CDP targets");
}
export async function connect(t) {
  const ws = new WebSocket(t.webSocketDebuggerUrl); await new Promise(r => ws.once("open", r));
  let id = 0; const pend = new Map();
  ws.on("message", m => { const d = JSON.parse(m); if (d.id && pend.has(d.id)) { pend.get(d.id)(d); pend.delete(d.id); } });
  const send = (method, params = {}) => new Promise(res => { const i = ++id; pend.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
  const ev = async (expr) => { const r = await send("Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true }); if (r.result?.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails)); return r.result?.result?.value; };
  return { send, ev, close: () => ws.close() };
}
