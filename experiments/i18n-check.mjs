import fs from "node:fs";
import * as mod from "/home/box/.bun/install/global/node_modules/lino-i18n/src/index.js";
const text = fs.readFileSync("src/web/i18n-catalog.lino", "utf8");
const cat = mod.parseLinoCatalogs(text);
const obj = {};
cat.forEach((e) => { if (e && e.locale && e.translations) obj[e.locale] = e.translations; });
const i18n = mod.createI18n({ locales: obj, defaultLocale: "en", fallback: ["en"] });
for (const loc of ["en","ru","zh","hi"]) {
  const show = (k) => console.log(`[${loc}]`, k, "=>", JSON.stringify(i18n.t(k, {}, { locale: loc, defaultValue: "MISS" })));
  show("settings.responseLanguage");
  show("settings.responseLanguage.lastMessage");
  show("settings.responseLanguage.preferred");
  show("settings.responseLanguage.ui");
  show("settings.preferredLanguage");
}
