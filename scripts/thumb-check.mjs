import { targets, connect } from "./cdp.mjs";
const record = process.argv[2] === "record";
const ts = await targets(); const main = ts.find(t => t.type === "page" && !/overlay|hud|region/.test(t.url));
const c = await connect(main); const sleep = ms => new Promise(r => setTimeout(r, ms));
const inv = (cmd, a) => c.ev(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(a||{})}).then(v=>JSON.stringify(v))`);
if (record) { console.log("start", await inv("start_recording").catch(e=>e.message)); await sleep(6000); await inv("stop_recording"); await sleep(9000); }
await c.ev(`document.querySelectorAll('.sidebar .nav-item')[1].click()`); await sleep(2500);
console.log(await c.ev(`JSON.stringify([...document.querySelectorAll('.clip-grid img')].map(i => ({src: i.src.slice(0,110), ok: i.complete && i.naturalWidth>0, w: i.naturalWidth})))`));
console.log("thumbnail placeholders:", await c.ev(`document.querySelectorAll('.thumb-fallback, .thumb-loading').length`));
const clips = JSON.parse(await inv("list_clips", { query: { section: "all", search: "", sort: "newest" } }));
console.log(clips.map(x => ({ title: x.title, thumb: x.thumbPath })));
c.close();
