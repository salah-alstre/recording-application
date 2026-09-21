import { targets, connect } from "./cdp.mjs";
const ts = await targets(); const main = ts.find(t => t.type === "page" && !/overlay|hud|region/.test(t.url));
const c = await connect(main); const sleep = ms => new Promise(r => setTimeout(r, ms));
const inv = (cmd, a) => c.ev(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(a||{})}).then(()=>'ok')`);
const state = () => c.ev(`JSON.stringify({vis: document.visibilityState, page: !!document.querySelector('.content-inner .page'), ci: document.querySelector('.content-inner')?.getAttribute('style'), kids: document.querySelector('.content')?.children.length, dh: document.documentElement.dataset.hidden})`);
const nav = i => c.ev(`document.querySelectorAll('.sidebar .nav-item')[${i}].click()`);
for (const mode of ["hide", "minimize"]) {
  await nav(0); await sleep(600);
  console.log(mode, "->", await inv(mode === "hide" ? "plugin:window|hide" : "plugin:window|minimize", { label: "main" }));
  await sleep(800); console.log(" hidden state:", await state());
  for (const i of [1, 2, 3, 4, 0, 1]) { await nav(i); await sleep(150); }
  console.log(" after nav while hidden:", await state());
  await inv(mode === "hide" ? "plugin:window|show" : "plugin:window|unminimize", { label: "main" }); await sleep(2500);
  console.log(" after restore:", await state());
  await nav(2); await sleep(1000); console.log(" nav after restore:", await state());
}
c.close();
