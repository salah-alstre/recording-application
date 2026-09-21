import { targets, connect } from "./cdp.mjs";
const ts = await targets(); const main = ts.find(t => t.type === "page" && !/overlay|hud|region/.test(t.url));
const c = await connect(main); const sleep = ms => new Promise(r => setTimeout(r, ms));
const state = () => c.ev(`JSON.stringify({nav: document.querySelector('.nav-item[aria-current]')?.textContent, title: document.querySelector('.content-inner .page-title')?.textContent, op: getComputedStyle(document.querySelector('.content-inner')).opacity})`);
let bad = 0;
const check = async (label, i) => { const s = JSON.parse(await state()); const ok = s.nav && s.title === s.nav && s.op === "1"; if (!ok) bad++; console.log(ok ? "ok  " : "FAIL", label, JSON.stringify(s)); };
await c.ev(`document.querySelectorAll('.sidebar .nav-item')[0].click()`); await sleep(500); await check("baseline");
await c.ev(`window.__raf = window.requestAnimationFrame; window.requestAnimationFrame = () => 0; 1`);   // frames stop, as when the WebView is occluded
for (const i of [1, 2, 3, 4, 0, 2, 4, 1]) { await c.ev(`document.querySelectorAll('.sidebar .nav-item')[${i}].click()`); await sleep(400); await check("frames stalled → nav " + i); }
await c.ev(`window.requestAnimationFrame = window.__raf; 1`); await sleep(600); await check("frames resumed");
console.log(bad ? "FAILURES: " + bad : "ALL OK"); c.close();
