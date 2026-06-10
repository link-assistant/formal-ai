const fs = require("fs"); const path = require("path");
const root = process.cwd();
const code = fs.readFileSync(path.join(root,"src/web/seed_loader.js"),"utf8");
const g = {}; new Function("self","globalThis",code).call(null,g,g);
const Seed = g.FormalAiSeed;
const read = (p) => fs.readFileSync(path.join(root,"data/seed",p),"utf8");
const assert = (c,m) => { if(!c) throw new Error(m); };

// concept-contexts: multi-word + multi-script aliases (migrated quoted list)
const ctx = Seed.extractConceptContexts(Seed.parse(read("concept-contexts.lino")));
const ml = ctx.find((c)=>c.aliases.includes("machine learning"));
assert(ml,"machine learning alias not parsed");
assert(ml.aliases.includes("machine-learning research"),"multi-word alias lost");
assert(ml.aliases.includes("машинное обучение"),"cyrillic alias lost");

// concepts: top-level multi-word/script aliases + localized bare-migrated list + context_links
const concepts = Seed.extractConcepts(Seed.parse(read("concepts.lino")));
const color = concepts.find((c)=>c.aliases.includes("colour"));
assert(color,"colour alias not parsed");
assert(color.aliases.includes("color of light"),"multi-word top alias lost");
const locEn = color.localized.find((l)=>l.language==="en");
assert(locEn && locEn.aliases.includes("hue"),"localized bare-migrated alias lost");
const linked = concepts.find((c)=>c.contextLinks.includes("context_machine_learning"));
assert(linked,"context_links not parsed as list");

// personas: trigger phrase survives
const personas = Seed.extractPersonas(Seed.parse(read("personas.lino")));
assert(JSON.stringify(personas).includes("act as"),"persona trigger lost");

console.log("seed_loader JS list parsing OK:",
  "ctx="+ml.aliases.length, "color_top="+color.aliases.length,
  "color_loc_en="+locEn.aliases.length, "ctx_links="+linked.contextLinks.length);
