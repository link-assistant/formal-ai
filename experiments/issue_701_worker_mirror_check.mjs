// Issue #701: check the compacted `extractTermInformationRequest` in the JS
// worker mirror behaves exactly like the three-loop version it replaced.
// Usage: node experiments/issue_701_worker_mirror_check.mjs
import { readFileSync, readdirSync } from "node:fs";
import vm from "node:vm";

const dir = "src/web/worker";
const source = readdirSync(dir)
  .filter((f) => f.endsWith(".js"))
  .sort()
  .map((f) => readFileSync(`${dir}/${f}`, "utf8"))
  .join("\n");

const context = vm.createContext({
  self: {},
  console,
  postMessage() {},
  fetch: () => Promise.reject(new Error("offline")),
  TextEncoder,
  TextDecoder,
  crypto,
  URL,
  setTimeout,
  indexedDB: undefined,
});
vm.runInContext(source, context, { filename: "worker-bundle.js" });

const reference = vm.runInContext(
  `(function (prompt, normalized) {
     if (conceptLookupResolves(prompt) || termInformationPromptIsLocalContext(normalized)) return "";
     const text = String(normalized || "");
     const markers = webSearchMarkers();
     const candidates = [];
     for (const prefix of markers.termInformationPrefixes) {
       if (text.startsWith(prefix)) candidates.push(text.slice(prefix.length));
     }
     for (const suffix of markers.termInformationSuffixes) {
       if (suffix && text.endsWith(suffix)) candidates.push(text.slice(0, text.length - suffix.length));
     }
     for (const { before, after } of markers.termInformationCircumfixes) {
       if (text.startsWith(before) && after && text.endsWith(after)) {
         const inner = text.slice(before.length, text.length - after.length);
         if (inner) candidates.push(inner);
       }
     }
     for (const candidate of candidates) {
       if (termInformationQueryIsLocalContext(candidate)) return "";
       const query = validSearchQuery(candidate);
       if (query) return query;
     }
     return "";
   })`,
  context,
);

const prompts = JSON.parse(readFileSync(process.argv[2] ?? "experiments/issue_701_prompts.json", "utf8"));
let checked = 0;
let mismatched = 0;
for (const prompt of prompts) {
  const normalized = vm.runInContext("normalizePrompt", context)(prompt);
  const a = reference(prompt, normalized);
  const b = context.extractTermInformationRequest(prompt, normalized);
  checked += 1;
  if (a !== b) {
    mismatched += 1;
    console.log(`MISMATCH ${JSON.stringify(prompt)}\n  loops   = ${JSON.stringify(a)}\n  compact = ${JSON.stringify(b)}`);
  }
}
console.log(`checked=${checked} mismatched=${mismatched}`);
process.exit(mismatched === 0 ? 0 : 1);
