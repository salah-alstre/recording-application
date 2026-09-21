import { targets, connect } from "./cdp.mjs";
const cycles = Number(process.argv[2] || 30), lang = process.argv[3] || "en", wait = Number(process.argv[4] || 350);
const ts = await targets(); const main = ts.find(t => t.url.includes("index.html") || (t.type === "page" && !/overlay|hud|region/.test(t.url)));
const c = await connect(main);
const probe = `(() => { const ci = document.querySelector('.content-inner'); if(!ci) return {ok:false, why:'no .content-inner'};
  const p = ci.querySelector('.page'); const nav = document.querySelector('.nav-item[aria-current]')?.textContent; const title = document.querySelector('.content-inner .page-title')?.textContent; if(nav!==title) return {ok:false, why:'state/render mismatch', nav, title}; if(!p) return {ok:false, why:'no .page', html: ci.innerHTML.slice(0,200), style: ci.getAttribute('style')};
  const r = p.getBoundingClientRect(); let o=1, el=p; while(el && el!==document.body){ o*=parseFloat(getComputedStyle(el).opacity); el=el.parentElement; }
  return {ok: r.width>100 && r.height>50 && o>0.95, w:r.width, h:r.height, opacity:o, ci: ci.getAttribute('style')}; })()`;
const click = (i) => c.ev(`document.querySelectorAll('.sidebar .nav-item')[${i}].click()`);
if (lang) await c.ev(`[...document.querySelectorAll('.sidebar-foot .seg button')].find(b=>b.textContent.trim()===${JSON.stringify(lang === "ar" ? "ع" : "EN")})?.click()`);
const order = [0, 1, 2, 3, 4, 0]; let fails = 0, n = 0;
for (let k = 0; k < cycles; k++) for (const i of order) { await click(i); await new Promise(r => setTimeout(r, wait)); n++; const p = await c.ev(probe); if (!p.ok) { fails++; if (fails < 4) console.log("FAIL nav#", n, "page", i, JSON.stringify(p)); } }
// rapid random navigation
for (let k = 0; k < 60; k++) { await click(Math.floor(Math.random() * 5)); await new Promise(r => setTimeout(r, Math.random() * 120)); }
await new Promise(r => setTimeout(r, 900)); const fin = await c.ev(probe); if (!fin.ok) { fails++; console.log("FAIL after rapid random", JSON.stringify(fin)); }
console.log(`lang=${lang} navigations=${n}+60 rapid, failures=${fails}, dir=${await c.ev("document.documentElement.dir")}`); c.close();
