import { targets, connect } from "./cdp.mjs";
const ts = await targets(); const main = ts.find(t => t.type === "page" && !/overlay|hud|region/.test(t.url));
const c = await connect(main); const sleep = ms => new Promise(r => setTimeout(r, ms));
const inv = (cmd, a) => c.ev(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(a||{})}).then(v=>JSON.stringify(v))`);
const imgs = () => c.ev(`JSON.stringify([...document.querySelectorAll('.clip-grid img')].map(i => i.complete && i.naturalWidth > 0))`);
if (process.argv[2] === "create") { await inv("start_recording"); await sleep(6000); await inv("take_screenshot", { mode: "display" }); await sleep(1500); await inv("stop_recording"); await sleep(9000); }
await c.ev(`document.querySelectorAll('.sidebar .nav-item')[1].click()`); await sleep(2500);
const list = JSON.parse(await imgs()); console.log("cards:", list.length, "images loaded:", list.filter(Boolean).length, "broken:", list.filter(x=>!x).length);
console.log("fallback/loader tiles:", await c.ev(`document.querySelectorAll('.thumb-fallback,.thumb-loading').length`));
const clips = JSON.parse(await inv("list_clips", { query: { section: "all", search: "", sort: "newest" } }));
console.log(clips.slice(0,3).map(x => `${x.kind} ${x.title} -> ${x.thumbPath.split("\\").slice(-2).join("\\")}`).join("\n"));
c.close();
