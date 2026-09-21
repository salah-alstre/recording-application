import { targets, connect } from "./cdp.mjs";
const ts = await targets(); const main = ts.find(t => t.type === "page" && !/overlay|hud|region/.test(t.url));
const c = await connect(main); let bad = 0;
for (let round = 0; round < 12; round++) {
  for (let k = 0; k < 40; k++) { await c.ev(`document.querySelectorAll('.sidebar .nav-item')[${Math.floor(Math.random()*5)}].click()`); await new Promise(r => setTimeout(r, Math.random() * 60)); }
  await new Promise(r => setTimeout(r, 1500));
  const s = await c.ev(`JSON.stringify({page: !!document.querySelector('.content-inner .page'), ci: document.querySelector('.content-inner')?.getAttribute('style'), n: document.querySelector('.content')?.children.length})`);
  const j = JSON.parse(s); if (!j.page || j.ci?.includes("opacity: 0")) { bad++; console.log("BLANK/STUCK round", round, s); }
}
console.log("bad rounds:", bad); c.close();
