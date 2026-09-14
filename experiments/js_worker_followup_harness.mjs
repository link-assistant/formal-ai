// Node harness that boots the browser worker and exercises the
// response-language follow-up (issue #556) end to end. The seed .lino bundle is
// served from the canonical `data/seed/` tree so meaningsWithRole() is
// populated exactly as in the browser.
//
// `createWorkerContext` runs `src/web/formal_ai_worker.js` itself, so the entry
// point decides what to load: `seed-files.js`, `seed_loader.js`,
// `worker-modules.js`, then every module the last one lists. Issue #991 made
// those inventories generated, union-merged files so that nothing else has to
// name them -- a harness that rebuilt the load order by hand silently went
// stale as soon as a worker module was added, which is what happened here.
import { createWorkerContext, evaluate } from "../tests/web/support/browser-runtime.mjs";

const sandbox = createWorkerContext();

async function main() {
  // Hydrate through the real loadSeed(), exactly as the browser worker does.
  await evaluate(sandbox, "loadSeed()");

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
    {
      // Non-English context retargeted to a third language: the forced-language
      // seam is target-driven, it does not assume an English source.
      label: "RU context retargets identity to Chinese",
      first: "что ты такое",
      followup: "用中文",
      wantLang: "zh",
      wantIntent: "identity",
      mustContain: "确定性的符号化 AI",
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
