// Issue #386: functional-parity check for the role-query user-intent recognisers
// in the hand-written browser worker (src/web/formal_ai_worker.js). The refactor
// rewired four functions to reason over the meaning lexicon by role + slot
// instead of hardcoded per-language word lists:
//   historyMentionsWebSearch  (web_search_history_signal substring markers)
//   hasProofRequestShape      (proof_directive / proof_request_lead / proof_marker)
//   extractProofClaim         (proof_claim_scaffold prefixes, declaration order)
//   isWhoIsPrompt             (who_question_lead prefixes + who_question_tail suffixes)
//
// This harness loads BOTH the committed baseline (git show HEAD:…) and the
// working-tree worker into separate vm sandboxes and asserts the four functions
// return byte-identical results across a broad multilingual matrix — proving the
// refactor is behaviour-preserving, including the prover/proven boundary guard
// and the first-matching-prefix claim extraction. It also spot-checks that the
// new worker still embeds the proof lexicon (a non-empty proof_directive role).
//
// Run with: node experiments/issue-386-worker-user-intent-parity.mjs

import { readFileSync } from "node:fs";
import { execSync } from "node:child_process";
import vm from "node:vm";

function loadWorker(source, label) {
  const sandbox = {
    self: { location: { search: "" } },
    importScripts: () => {
      throw new Error("no importScripts in node harness");
    },
    postMessage: () => {},
    console,
    TextEncoder,
    TextDecoder,
    WebAssembly,
    fetch: () => Promise.reject(new Error("offline")),
    setTimeout,
    clearTimeout,
  };
  sandbox.globalThis = sandbox;
  vm.createContext(sandbox);
  vm.runInContext(source, sandbox, { filename: `formal_ai_worker.js (${label})` });
  return sandbox;
}

const newSource = readFileSync(
  new URL("../src/web/formal_ai_worker.js", import.meta.url),
  "utf8",
);
const oldSource = execSync("git show HEAD:src/web/formal_ai_worker.js", {
  cwd: new URL("..", import.meta.url),
  encoding: "utf8",
  maxBuffer: 64 * 1024 * 1024,
});

const next = loadWorker(newSource, "working tree");
const base = loadWorker(oldSource, "HEAD");

for (const name of [
  "normalizePrompt",
  "historyMentionsWebSearch",
  "hasProofRequestShape",
  "extractProofClaim",
  "isWhoIsPrompt",
]) {
  if (typeof next[name] !== "function") throw new Error(`new worker missing ${name}`);
  if (typeof base[name] !== "function") throw new Error(`HEAD worker missing ${name}`);
}

let failures = 0;
let checks = 0;
function eq(label, got, want) {
  checks += 1;
  const g = JSON.stringify(got);
  const w = JSON.stringify(want);
  if (g !== w) {
    failures += 1;
    console.error(`FAIL ${label}: new=${g} old=${w}`);
  }
}

// Prompts fed (after normalization) to the three normalized-input recognisers.
// A deliberately wide net: every composition arm in all four languages, the
// boundary guards (prover/proven/improve), claim extraction with leading noise
// and mid-sentence intros, who-is in head-initial and head-final orders, plus
// non-matching controls. We assert NEW === OLD per function, so any quirk of the
// original hardcoded logic must be preserved exactly.
const PROMPTS = [
  // proof: bare directive verbs (en/ru), clause-initial
  "prove that sqrt(2) is irrational",
  "Prove the Pythagorean theorem",
  "proof of the infinitude of primes",
  "proof that 2 is prime",
  "show that there are infinitely many primes",
  "demonstrate that 1 + 1 = 2",
  "demonstrate the claim",
  "докажи что 2 + 2 = 4",
  "докажите теорему пифагора",
  "доказать что корень из двух иррационален",
  "доказательство теоремы ферма",
  // proof: English request-frame leads (no `that`)
  "can you prove the riemann hypothesis",
  "can you prove that god exists",
  "could you prove fermat's little theorem",
  "could you prove that p equals np",
  "please prove this statement",
  "please prove that the set is finite",
  "give me a proof of euclid's theorem",
  "give me a proof that there is no largest prime",
  "give a proof of the claim",
  "give a proof that two is prime",
  // proof: mid-sentence / non-Latin assertion markers
  "now prove that x is positive",
  "i wrote the proof of god yesterday",
  "रीमान परिकल्पना साबित करो कि सत्य है",
  "कृपया सिद्ध कीजिए कि यह सही है",
  "प्रमाण दो",
  "请证明勾股定理",
  "證明這個定理",
  // proof: boundary-guard negatives (must stay non-proof in both)
  "what is a prover",
  "the proven results are solid",
  "improve the code quality",
  "approve the request",
  "professor explains topology",
  // who-is: head-initial leads (en/ru)
  "who is elon musk",
  "who was albert einstein",
  "who are the beatles",
  "кто такой пушкин",
  "кто такая ада лавлейс",
  "кто это",
  "кто придумал интеграл",
  "кто-то украл велосипед",
  // who-is: head-final tails (hi/zh)
  "अल्बर्ट आइंस्टीन कौन है",
  "वे लोग कौन हैं",
  "爱因斯坦是谁",
  "愛因斯坦是誰",
  // controls: neither proof nor who-is
  "what is the capital of france",
  "сколько будет два плюс два",
  "translate hello into german",
  "summarize this conversation",
  "",
  "   ",
];

for (const prompt of PROMPTS) {
  const nNorm = next.normalizePrompt(prompt);
  const oNorm = base.normalizePrompt(prompt);
  eq(`normalize "${prompt}"`, nNorm, oNorm);
  eq(`hasProofRequestShape "${prompt}"`, next.hasProofRequestShape(nNorm), base.hasProofRequestShape(oNorm));
  eq(`extractProofClaim "${prompt}"`, next.extractProofClaim(nNorm), base.extractProofClaim(oNorm));
  eq(`isWhoIsPrompt "${prompt}"`, next.isWhoIsPrompt(nNorm), base.isWhoIsPrompt(oNorm));
}

// historyMentionsWebSearch takes a turn array (raw .content, lowercased inside).
const HISTORIES = [
  [{ role: "user", content: "let me search the internet for that" }],
  [{ role: "assistant", content: "I used DuckDuckGo to find it" }],
  [{ role: "user", content: "do a Web Search please" }],
  [{ role: "assistant", content: "веб-поиск показал результаты" }],
  [{ role: "user", content: "веб поиск дал ответ" }],
  [{ role: "assistant", content: "в интернете нашлось" }],
  [{ role: "user", content: "hello world, how are you" }],
  [{ role: "user", content: "improve my code" }],
  [],
  null,
  undefined,
  [{ role: "user", content: null }],
  [{ role: "user" }],
  [
    { role: "user", content: "first turn unrelated" },
    { role: "assistant", content: "second turn used a web search" },
  ],
];
for (let i = 0; i < HISTORIES.length; i += 1) {
  const h = HISTORIES[i];
  eq(`historyMentionsWebSearch #${i}`, next.historyMentionsWebSearch(h), base.historyMentionsWebSearch(h));
}

// Sanity: the new worker must actually embed the proof lexicon now (otherwise
// parity could pass vacuously if both sides recognised nothing). proof_directive
// must resolve to a non-empty bare-literal set on the working-tree worker.
if (typeof next.bareLiterals === "function") {
  const verbs = next.bareLiterals("proof_directive");
  if (!Array.isArray(verbs) || verbs.length === 0) {
    failures += 1;
    console.error("FAIL new worker proof_directive bare literals are empty — lexicon not embedded");
  } else {
    console.log(`ok   new worker proof_directive bare literals: ${JSON.stringify(verbs)}`);
  }
}
// And at least one proof + one who-is + one history case must be TRUE on the new
// worker, so the matrix is exercising live recognition rather than all-false.
{
  const proofTrue = next.hasProofRequestShape(next.normalizePrompt("prove that sqrt(2) is irrational"));
  const whoTrue = next.isWhoIsPrompt(next.normalizePrompt("who is elon musk"));
  const histTrue = next.historyMentionsWebSearch([{ content: "let me search the internet" }]);
  eq("live proof recognition", proofTrue, true);
  eq("live who-is recognition", whoTrue, true);
  eq("live history recognition", histTrue, true);
}

if (failures) {
  console.error(`\n${failures}/${checks} parity check(s) FAILED`);
  process.exit(1);
}
console.log(`\nall ${checks} worker user-intent parity checks passed (NEW === HEAD)`);
