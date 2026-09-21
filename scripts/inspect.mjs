import { targets, connect } from "./cdp.mjs";
const ts = await targets(); const main = ts.find(t => t.url.includes("index.html") || (t.type === "page" && !/overlay|hud|region/.test(t.url)));
const c = await connect(main);
console.log(await c.ev(process.argv[2]));
c.close();
