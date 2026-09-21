import { targets, connect } from "./cdp.mjs"; import { execSync } from "node:child_process";
const ts = await targets(); const main = ts.find(t => t.type === "page" && !/overlay|hud|region/.test(t.url));
const c = await connect(main); const sleep = ms => new Promise(r => setTimeout(r, ms)); let fails = 0;
const inv = (cmd, a) => c.ev(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(cmd)}, ${JSON.stringify(a||{})}).then(()=>'ok')`);
const pageOk = async (label) => { const p = JSON.parse(await c.ev(`(()=>{const nav=document.querySelector('.nav-item[aria-current]')?.textContent; const t=document.querySelector('.content-inner .page-title')?.textContent; const r=document.querySelector('.content-inner .page')?.getBoundingClientRect(); return JSON.stringify({nav,t,w:r?.width,h:r?.height,op:getComputedStyle(document.querySelector('.content-inner')).opacity})})()`)); if (!(p.nav === p.t && p.w > 300 && p.h > 100 && p.op === "1")) { fails++; console.log("FAIL", label, JSON.stringify(p)); } };
const nav = i => c.ev(`document.querySelectorAll('.sidebar .nav-item')[${i}].click()`);
const setLang = l => c.ev(`[...document.querySelectorAll('.sidebar-foot .seg button')].find(b=>b.textContent.trim()===${JSON.stringify(l==="ar"?"ع":"EN")})?.click()`);
// 1. language switching while navigating
for (let k = 0; k < 24; k++) { await nav(Math.floor(Math.random()*5)); await sleep(80); await setLang(k % 2 ? "en" : "ar"); await sleep(300); await pageOk("lang switch #" + k); }
console.log("language switching done, dir=", await c.ev("document.documentElement.dir"), "lang=", await c.ev("document.documentElement.lang"));
// 2. settings sub-navigation, both languages
for (const lang of ["ar", "en"]) { await setLang(lang); await sleep(400); await nav(4); await sleep(400);
  for (let round = 0; round < 4; round++) for (let i = 0; i < 15; i++) { await c.ev(`document.querySelectorAll('.settings-nav .sub-item')[${i}].click()`); await sleep(220);
    const s = JSON.parse(await c.ev(`JSON.stringify({cur: document.querySelector('.settings-nav .sub-item[aria-current]')?.textContent, h: document.querySelector('.settings-body .section-title')?.textContent, cards: document.querySelectorAll('.settings-body .card').length, skel: document.querySelectorAll('.settings-body .skeleton').length, err: !!document.querySelector('.settings-body .banner.error')})`));
    if (s.cur !== s.h || s.cards < 1 || s.skel || s.err) { fails++; console.log("FAIL settings", lang, i, JSON.stringify(s)); } }
  console.log("settings 15 sections x4 done in", lang); }
// 3. window: maximize / restore / resize / minimize / restore
const ps = (cmd) => execSync(`powershell -NoProfile -Command "${cmd}"`, { encoding: "utf8" }).trim();
const win32 = `Add-Type -Name W -Namespace N -MemberDefinition '[DllImport(\\"user32.dll\\")] public static extern bool ShowWindow(IntPtr h,int c); [DllImport(\\"user32.dll\\")] public static extern bool MoveWindow(IntPtr h,int x,int y,int w,int ht,bool r);'; $p=Get-Process rimlight | ? {$_.MainWindowHandle -ne 0} | select -first 1;`;
for (const [label, cmd] of [["maximize", "[N.W]::ShowWindow($p.MainWindowHandle,3)"], ["restore", "[N.W]::ShowWindow($p.MainWindowHandle,9)"], ["resize small", "[N.W]::MoveWindow($p.MainWindowHandle,100,60,1000,660,$true)"], ["resize large", "[N.W]::MoveWindow($p.MainWindowHandle,60,40,1500,900,$true)"], ["minimize", "[N.W]::ShowWindow($p.MainWindowHandle,6)"], ["restore after minimize", "[N.W]::ShowWindow($p.MainWindowHandle,9)"]]) {
  ps(win32 + cmd + "|Out-Null"); await sleep(700); await pageOk(label + " (before nav)");
  for (const i of [1, 3, 4, 0]) { await nav(i); await sleep(300); await pageOk(label + " nav " + i); } console.log("window:", label, "ok");
}
console.log(fails ? "TOTAL FAILURES: " + fails : "ALL OK"); c.close();
