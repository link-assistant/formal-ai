// Node harness that loads the browser worker modules in one shared scope and
// exercises the response-language follow-up (issue #556) end to end. It reads
// the seed .lino bundle from disk so meaningsWithRole() is populated exactly
// as in the browser.
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { fileURLToPath } from "node:url";
import { TextEncoder, TextDecoder } from "node:util";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const webDir = path.join(root, "src", "web");
const seedDir = path.join(webDir, "seed");

// Build the raw seed map { "seed/<file>": text } for FormalAiSeed.loadAll.
function readSeedRaw() {
  const raw = {};
  for (const file of fs.readdirSync(seedDir)) {
    if (file.endsWith(".lino")) {
      raw[`seed/${file}`] = fs.readFileSync(path.join(seedDir, file), "utf8");
    }
  }
  return raw;
}

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.postMessage = () => {};
// Serve the seed .lino files from disk so FormalAiSeed.loadAll() hydrates the
// full seed (concepts, projects, tools, meanings) exactly as the browser does.
sandbox.fetch = async (url) => {
  const clean = String(url).split("?")[0];
  const file = path.join(webDir, clean);
  const text = fs.readFileSync(file, "utf8");
  return { ok: true, status: 200, async text() { return text; } };
};
sandbox.WebAssembly = { instantiate: async () => { throw new Error("no wasm"); } };
sandbox.location = { search: "" };
sandbox.setTimeout = setTimeout;
sandbox.clearTimeout = clearTimeout;
sandbox.TextEncoder = TextEncoder;
sandbox.TextDecoder = TextDecoder;
sandbox.URL = URL;
sandbox.URLSearchParams = URLSearchParams;
vm.createContext(sandbox);

function run(file) {
  const code = fs.readFileSync(file, "utf8");
  vm.runInContext(code, sandbox, { filename: file });
}

// Load the seed loader (defines self.FormalAiSeed).
run(path.join(webDir, "seed_loader.js"));

// Load every worker module into the shared scope, in order.
for (let i = 0; i <= 20; i++) {
  const name = `formal_ai_worker_${String(i).padStart(2, "0")}.js`;
  run(path.join(webDir, "worker", name));
}

// Hydrate the seed exactly like loadSeed(): stash a preloaded raw bundle and
// override FormalAiSeed.loadAll to return it.
void readSeedRaw;

async function main() {
  // Hydrate via the real FormalAiSeed.loadAll (served from disk through fetch),
  // exactly as loadSeed() does in the browser worker.
  await vm.runInContext("loadSeed()", sandbox);

  const solve = sandbox.solve;

  // Each case: answer a first request, then send a bare-language follow-up and
  // confirm the *whole* solver replays the prior answer in the target language
  // (issue #556) — not just project lookups. The first answer is produced by
  // the real solver so history mirrors production. `mustContain` is a phrase
  // that only appears in the target-language rendering, proving the retarget
  // actually localized the content (issue #526 round-trip spirit).
  const cases = [
    {
      label: "RU follow-up reanswers a GitHub project lookup",
      first: "ты можешь сделать кодревью https://github.com/netkeep80/anum_docs ?",
      followup: "я не понимаю по английски, напиши по русски",
      wantLang: "ru",
      wantIntent: "project_lookup",
      mustContain: "Это запрос о репозитории",
    },
    {
      label: "RU follow-up reanswers a capabilities answer",
      first: "what can you do",
      followup: "я не понимаю по английски, напиши по русски",
      wantLang: "ru",
      wantIntent: "capabilities",
      mustContain: "Вот что я умею",
    },
    {
      label: "ZH terse switch reanswers an identity answer",
      first: "what are you",
      followup: "用中文回答",
      wantLang: "zh",
      wantIntent: "identity",
      mustContain: "确定性",
    },
    {
      label: "HI comprehension failure reanswers capabilities",
      first: "what can you do",
      followup: "मुझे समझ नहीं आता, हिंदी में लिखें",
      wantLang: "hi",
      wantIntent: "capabilities",
      mustContain: "मैं यह कर सकता हूँ",
    },
  ];

  let failures = 0;
  for (const c of cases) {
    const first = await solve(c.first, [], {}, {}, [], {});
    const history = [
      { role: "user", content: c.first },
      { role: "assistant", content: String(first.content || "") },
    ];
    const ans = await solve(c.followup, history, {}, {}, [], {});
    const ev = Array.isArray(ans.evidence) ? ans.evidence : [];
    const hasTarget = ev.includes(`response_language_followup:target:${c.wantLang}`);
    const hasLangTo = ev.includes(`language_to:${c.wantLang}`);
    const handler = ev.find((e) => e.startsWith("response_language_followup:handler:"));
    const intentOk = ans.intent === c.wantIntent;
    const localized = String(ans.content || "").includes(c.mustContain);
    console.log(`\n=== ${c.label} ===`);
    console.log(`  first intent: ${first.intent}  followup intent: ${ans.intent} (want ${c.wantIntent})`);
    console.log(`  target marker: ${hasTarget}  language_to: ${hasLangTo}`);
    console.log(`  handler marker: ${handler}`);
    console.log(`  localized (${JSON.stringify(c.mustContain)}): ${localized}`);
    console.log(`  content[0:80]: ${String(ans.content || "").slice(0, 80).replace(/\n/g, " ")}`);
    if (!hasTarget || !hasLangTo || !handler || !intentOk || !localized) {
      console.log("  !! CASE FAILED");
      failures += 1;
    }
  }
  console.log(`\n${cases.length - failures}/${cases.length} cases passed.`);
  if (failures > 0) process.exit(1);
}

main().catch((e) => { console.error(e); process.exit(1); });
