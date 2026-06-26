// Issue #340: verify the pure-JS demo worker (`src/web/formal_ai_worker.js`)
// answers a composite `write_program` request the verified template catalog
// cannot resolve (HTTP GET -> parse JSON -> mean/median) with a *blueprint*
// instead of dead-ending at `write_program_unsupported`, and that the rendered
// program/plan/honest-execution report mirrors the Rust core
// (`src/coding/blueprint.rs`). Exercises the JS fallback path (no WASM loaded),
// which is what the GitHub Pages worker uses before/without the wasm module.
//
// Run with: node experiments/issue-340-worker-parity.mjs

import { readFileSync } from "node:fs";
import vm from "node:vm";

const source = readFileSync(
  new URL("../src/web/formal_ai_worker.js", import.meta.url),
  "utf8",
);

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
vm.runInContext(source, sandbox, { filename: "formal_ai_worker.js" });

let failures = 0;
function check(label, condition, detail) {
  if (condition) {
    console.log(`ok   ${label}`);
  } else {
    failures += 1;
    console.error(`FAIL ${label}${detail ? `\n     ${detail}` : ""}`);
  }
}

const {
  tryWriteProgram,
  tryProgramBlueprintFromPrompt,
  selectBlueprint,
  renderBlueprint,
  composeBlueprintProgram,
  normalizeBlueprintComposition,
  renderSelfFacts,
} = sandbox;

// 1. The exact issue #340 prompt (English, Rust).
const issuePrompt =
  "Write a Rust program that makes an HTTP GET request, parses the JSON " +
  "response, calculates the mean and median, and outputs the results, with " +
  "error handling and comments.";
const hit = tryWriteProgram(issuePrompt, [], "en");
check(
  "issue prompt is write_program (not unsupported)",
  hit && hit.intent === "write_program",
  hit && hit.intent,
);
check("blueprint embeds a fenced rust block", hit && hit.content.includes("```rust"));
check("blueprint embeds fn main()", hit && hit.content.includes("fn main()"));
check(
  "blueprint uses reqwest::blocking::get",
  hit && hit.content.includes("reqwest::blocking::get"),
);
check(
  "blueprint computes mean and median",
  hit && hit.content.includes("fn mean(") && hit.content.includes("fn median("),
);
check(
  "blueprint lists library prerequisites",
  hit && hit.content.includes("Required libraries:") && hit.content.includes("serde_json"),
);
check(
  "blueprint is honest: not run",
  hit && hit.content.includes("not run"),
);
check(
  "blueprint never claims it ran/compiled",
  hit && !/compiled and ran/i.test(hit.content),
);
check(
  "blueprint response evidence link is present",
  hit && hit.evidence.includes("response:write_program:blueprint:http_json_stats:rust"),
  hit && JSON.stringify(hit.evidence),
);
check(
  "blueprint records the recipe in evidence",
  hit && hit.evidence.includes("program_blueprint:recipe:http_json_stats"),
);
check(
  "blueprint execution status is unavailable",
  hit && hit.evidence.includes("execution_status:rust:unavailable"),
);

// 1b. Issue #342: a budget-calculator request includes web research, Python
// code, 50/30/20 budgeting, compound savings, a city comparison table, and a
// Markdown report. It must route to a Python blueprint before the generic web
// search handler can collapse the request to only "search".
const budgetPrompt =
  "I want to build a budget calculator. Here's what I need:\n\n" +
  "1. Search for average living costs in Moscow, Berlin, and New York\n" +
  "2. Write a Python program that takes monthly income as input, calculates " +
  "50/30/20 budget rule, and shows how much can be saved in each city\n" +
  "3. Calculate: If I save 20% of $3000 monthly at 8% annual return for " +
  "10 years, how much will I have?\n" +
  "4. Create a comparison table with city name, average rent, remaining " +
  "budget after expenses, and years to save $100,000\n" +
  "5. Export all this as a formatted markdown report with sources.";
const budget = tryWriteProgram(budgetPrompt, [], "en");
check(
  "budget request routes to write_program blueprint",
  budget && budget.intent === "write_program",
  budget && budget.intent,
);
check(
  "budget blueprint embeds Python report generator",
  budget &&
    budget.content.includes("```python") &&
    budget.content.includes("budget_50_30_20") &&
    budget.content.includes("budget_report.md"),
  budget && budget.content,
);
check(
  "budget blueprint covers cities and sources",
  budget &&
    budget.content.includes("Moscow") &&
    budget.content.includes("Berlin") &&
    budget.content.includes("New York") &&
    budget.content.includes("## Sources"),
  budget && budget.content,
);
check(
  "budget blueprint evidence link is present",
  budget &&
    budget.evidence.includes(
      "response:write_program:blueprint:personal_budget_report:python",
    ),
  budget && JSON.stringify(budget.evidence),
);
check(
  "budget blueprint records the recipe",
  budget &&
    budget.evidence.includes("program_blueprint:recipe:personal_budget_report"),
  budget && JSON.stringify(budget.evidence),
);

// 1c. Issue #459: class-based composite requests should route to the Python
// travel-planner blueprint before generic "Search:" bullets can take over.
const travelPrompt =
  'Build a "Smart Travel Planner" prototype:\n\n' +
  "1. Search: visa requirements for Russian citizens visiting Japan, UAE, Serbia\n" +
  "2. Search: average flight costs from Moscow to these destinations (next 3 months)\n" +
  "3. Write a Python class `TravelPlanner` with methods:\n" +
  "   - `add_destination(country: str, budget: float)`\n" +
  "   - `check_visa_requirements()` -> returns bool\n" +
  "   - `estimate_total_cost()` -> returns dict\n" +
  "   - `generate_itinerary(days: int)` -> returns markdown\n" +
  "4. Add business logic:\n" +
  "   - Prioritize destinations with visa-free access\n" +
  "   - Flag if budget < estimated cost\n" +
  "5. Generate sample output for: 7-day trip, $2000 budget\n" +
  "6. Output: class code + usage example + sample itinerary";
const travel = tryWriteProgram(travelPrompt, [], "en");
check(
  "travel planner request routes to write_program blueprint",
  travel && travel.intent === "write_program",
  travel && travel.intent,
);
check(
  "travel planner blueprint embeds the requested Python class",
  travel &&
    travel.content.includes("```python") &&
    travel.content.includes("class TravelPlanner") &&
    travel.content.includes("generate_itinerary") &&
    travel.content.includes("Budget warning"),
  travel && travel.content,
);
check(
  "travel planner blueprint evidence link is present",
  travel &&
    travel.evidence.includes(
      "response:write_program:blueprint:smart_travel_planner:python",
    ),
  travel && JSON.stringify(travel.evidence),
);
check(
  "travel planner blueprint records the recipe",
  travel && travel.evidence.includes("program_blueprint:recipe:smart_travel_planner"),
  travel && JSON.stringify(travel.evidence),
);
for (const { language, prompt, statusLabel } of [
  {
    language: "ru",
    prompt:
      "Найди источники по средней стоимости жизни и аренде в Москве, Берлине и Нью-Йорке. " +
      "Напиши программу на Python, которая принимает месячный доход, применяет правило " +
      "бюджета 50/30/20, считает накопить 20% от $3000, создает таблицу сравнения и " +
      "экспортирует markdown отчёт с источниками.",
    statusLabel: "Статус выполнения",
  },
  {
    language: "hi",
    prompt:
      "मास्को, बर्लिन और न्यूयॉर्क में औसत जीवन यापन लागत और किराया के स्रोत खोजो। " +
      "Python प्रोग्राम लिखो जो मासिक आय ले, 50/30/20 बजट नियम लगाए, $3000 का 20% " +
      "8% वार्षिक रिटर्न पर 10 साल बचत की गणना करे, तुलना तालिका बनाए और स्रोतों के " +
      "साथ मार्कडाउन रिपोर्ट निर्यात करे।",
    statusLabel: "निष्पादन स्थिति",
  },
  {
    language: "zh",
    prompt:
      "搜索莫斯科、柏林和纽约的平均生活成本和租金来源。编写 Python 程序，输入月收入，应用 " +
      "50/30/20 预算规则，计算每月存 $3000 的 20% 按 8% 年收益 10 年，创建比较表格，" +
      "并导出带来源的 Markdown 报告。",
    statusLabel: "执行状态",
  },
]) {
  const localizedBudget = tryWriteProgram(prompt, [], language);
  check(
    `budget request routes in ${language}`,
    localizedBudget &&
      localizedBudget.intent === "write_program" &&
      localizedBudget.content.includes(statusLabel) &&
      localizedBudget.evidence.includes(
        "program_blueprint:recipe:personal_budget_report",
      ),
    localizedBudget &&
      `${localizedBudget.intent} :: ${(localizedBudget.content || "").slice(0, 120)}`,
  );
}

// 1c. Issue #458: a crypto-portfolio tracker asks for "Search current prices"
// and "Write a Python script" in the same prompt. The direct blueprint probe is
// what lets the worker preempt broad search routing when the full program recipe
// is recognized.
const cryptoPrompt =
  "Simulate a crypto portfolio tracker:\n" +
  "1. Search current prices for: BTC, ETH, TON, USDT\n" +
  "2. Assume portfolio: 2.5 BTC, 15 ETH, 1000 TON, 5000 USDT\n" +
  "3. Calculate total value in USD, 24h change % for each asset, and portfolio weight distribution\n" +
  "4. Write a Python script that fetches prices from a public API (mock the endpoint), " +
  'implements alert logic: "Notify if any asset drops >5%", and logs results to a formatted string\n' +
  "5. Output: dashboard-style markdown + executable code";
const cryptoDirect = tryProgramBlueprintFromPrompt(cryptoPrompt, "en", "composed");
check(
  "crypto direct blueprint probe returns write_program",
  cryptoDirect && cryptoDirect.intent === "write_program",
  cryptoDirect && cryptoDirect.intent,
);
check(
  "crypto direct blueprint records recipe",
  cryptoDirect &&
    cryptoDirect.evidence.includes("program_blueprint:recipe:crypto_portfolio_tracker"),
  cryptoDirect && JSON.stringify(cryptoDirect.evidence),
);
const crypto = tryWriteProgram(cryptoPrompt, [], "en");
check(
  "crypto request routes to Python blueprint",
  crypto &&
    crypto.intent === "write_program" &&
    crypto.content.includes("```python") &&
    crypto.content.includes("# Crypto Portfolio Dashboard") &&
    crypto.content.includes("notify_alerts") &&
    crypto.content.includes("portfolio_weight"),
  crypto && `${crypto.intent} :: ${(crypto.content || "").slice(0, 160)}`,
);
check(
  "crypto blueprint evidence link is present",
  crypto &&
    crypto.evidence.includes(
      "response:write_program:blueprint:crypto_portfolio_tracker:python",
    ),
  crypto && JSON.stringify(crypto.evidence),
);

// 2. Python and JavaScript variants resolve too.
const py = tryWriteProgram(
  "Write a Python program that makes an HTTP GET request, parses the JSON, and computes the mean and median.",
  [],
  "en",
);
check(
  "python composite request routes to blueprint",
  py && py.intent === "write_program" && py.content.includes("```python") && py.content.includes("import requests"),
  py && py.intent,
);
const js = tryWriteProgram(
  "Write a JavaScript program that fetches JSON over HTTP and reports the mean and median.",
  [],
  "en",
);
check(
  "javascript composite request routes to blueprint",
  js && js.intent === "write_program" && js.content.includes("```javascript") && js.content.includes("await fetch("),
  js && js.intent,
);

// 3. Russian request is answered in Russian.
const ru = tryWriteProgram(
  "Напиши программу на Rust, которая делает HTTP запрос, разбирает JSON и считает среднее и медиану.",
  [],
  "ru",
);
check(
  "russian composite request routes to blueprint in russian",
  ru && ru.intent === "write_program" && ru.content.includes("Статус выполнения") && ru.content.includes("```rust"),
  ru && (ru.intent + " :: " + (ru.content || "").slice(0, 120)),
);

// 4. A partial request (http + json but NO statistics) stays unsupported —
//    the recipe's required capabilities are not all present.
const partial = tryWriteProgram(
  "Write a Rust program that makes an HTTP GET request and parses the JSON response.",
  [],
  "en",
);
check(
  "partial composite (no statistics) stays unsupported",
  partial && partial.intent === "write_program_unsupported",
  partial && partial.intent,
);

// 5. An unsupported language for the recipe (Go) stays unsupported.
const go = tryWriteProgram(
  "Write a Go program that makes an HTTP GET request, parses JSON, and computes the mean and median.",
  [],
  "en",
);
check(
  "go composite request stays unsupported (no curated go program)",
  go && go.intent === "write_program_unsupported",
  go && go.intent,
);

// 6. Cross-engine parity: the JS render must byte-match the Rust core. We assert
//    structural anchors here; the Rust test
//    `render_contains_plan_code_libraries_and_honest_execution` asserts the same
//    anchors on the Rust side, and the curated programs are verbatim copies.
const bp = selectBlueprint(
  "http get request parse json calculate mean median statistics",
  "rust",
);
check("selectBlueprint resolves rust http_json_stats", bp && bp.recipe.slug === "http_json_stats");
const rendered = renderBlueprint(bp, "en");
check("render numbers the decomposition plan", rendered.includes("1. Make an HTTP request"));
check("render localizes nothing in en intro", rendered.startsWith("Here is a Rust program"));

// 7. Compositional `comments` axis: a request that asks for comments keeps the
//    documented program; one that does not strips whole-line documentation, so
//    the synthesized program is a projection of the decomposition (not a frozen
//    string). Mirrors the Rust unit tests in `src/coding/blueprint.rs`.
const withComments = tryWriteProgram(
  "Write a Rust program that makes an HTTP GET request, parses JSON, computes mean and median, outputs the results, with comments.",
  [],
  "en",
);
check(
  "comments requested keeps the documented program",
  withComments && withComments.content.includes("// 1. Read the target URL"),
);
const noComments = tryWriteProgram(
  "Write a Rust program that makes an HTTP GET request, parses JSON, computes mean and median, and outputs the results.",
  [],
  "en",
);
const noCommentsCode = noComments && noComments.content.split("```rust\n")[1].split("\n```")[0];
check(
  "comments omitted strips whole-line documentation",
  noCommentsCode &&
    !noCommentsCode.split("\n").some((line) => line.trimStart().startsWith("//")),
  noCommentsCode,
);
check(
  "comments omitted keeps the core logic",
  noCommentsCode &&
    noCommentsCode.includes("reqwest::blocking::get") &&
    noCommentsCode.includes("fn median(") &&
    !/\n\n\n/.test(noCommentsCode),
);
const pyNoComments = tryWriteProgram(
  "Write a Python program that makes an HTTP GET request, parses JSON, and computes the mean and median.",
  [],
  "en",
);
const pyCode = pyNoComments && pyNoComments.content.split("```python\n")[1].split("\n```")[0];
check(
  "python comments omitted drops docstring and # lines",
  pyCode &&
    !pyCode.includes('"""') &&
    !pyCode.split("\n").some((line) => line.trimStart().startsWith("#")) &&
    pyCode.includes("requests.get"),
  pyCode,
);

// 8. Compositional `error_handling` axis: independent of `comments`, a request
//    that asks for error handling keeps the empty-numbers guard; one that does
//    not drops it (region body removed). Mirrors the Rust unit test
//    `error_handling_axis_composes_independently`.
const withErrors = tryWriteProgram(
  "Write a Rust program that makes an HTTP GET request, parses JSON, computes mean and median, outputs the results, with error handling.",
  [],
  "en",
);
const withErrorsCode =
  withErrors && withErrors.content.split("```rust\n")[1].split("\n```")[0];
check(
  "error handling requested keeps the empty-numbers guard",
  withErrorsCode && withErrorsCode.includes("contained no numbers"),
  withErrorsCode,
);
check(
  "error handling region markers never reach the user",
  withErrorsCode &&
    !withErrorsCode.includes("region:error_handling") &&
    !withErrorsCode.includes("endregion"),
  withErrorsCode,
);
check(
  "error handling omitted drops the guard body",
  noCommentsCode && !noCommentsCode.includes("contained no numbers"),
  noCommentsCode,
);
const pyWithErrors = tryWriteProgram(
  "Write a Python program that makes an HTTP GET request, parses JSON, computes mean and median, with error handling.",
  [],
  "en",
);
const pyErrCode =
  pyWithErrors && pyWithErrors.content.split("```python\n")[1].split("\n```")[0];
check(
  "python error handling requested keeps raise_for_status",
  pyErrCode && pyErrCode.includes("raise_for_status"),
  pyErrCode,
);
check(
  "python error handling omitted drops raise_for_status",
  pyCode && !pyCode.includes("raise_for_status"),
  pyCode,
);

// 9. `documented` strategy: regardless of which capabilities the request named,
//    the fully annotated program is emitted (every region present, comments
//    kept). Region marker lines are still always stripped. Mirrors the Rust
//    `region_directives_are_always_stripped_from_output` test and the
//    BlueprintComposition::Documented behavior.
const documented = renderBlueprint(bp, "en", "documented");
const documentedCode = documented.split("```rust\n")[1].split("\n```")[0];
check(
  "documented strategy keeps the comments",
  documentedCode.includes("// 1. Read the target URL"),
  documentedCode,
);
check(
  "documented strategy keeps every optional region body",
  documentedCode.includes("contained no numbers"),
  documentedCode,
);
check(
  "documented strategy still strips region marker lines",
  !documentedCode.includes("region:error_handling"),
  documentedCode,
);
check(
  "composeBlueprintProgram(documented) === renderable documented body",
  composeBlueprintProgram(bp, "documented").includes("contained no numbers"),
);
check(
  "normalizeBlueprintComposition maps aliases and defaults to composed",
  normalizeBlueprintComposition("full") === "documented" &&
    normalizeBlueprintComposition("verbatim") === "documented" &&
    normalizeBlueprintComposition("composed") === "composed" &&
    normalizeBlueprintComposition(undefined) === "composed" &&
    normalizeBlueprintComposition("nonsense") === "composed",
);

// 10. The active composition strategy is reported in the self-facts inventory,
//     mirroring the Rust `self_fact_blueprint_composition` line.
const facts = renderSelfFacts({ blueprintComposition: "documented" });
check(
  "self-facts report the blueprint composition setting",
  facts.includes('relation "blueprint_composition"') &&
    facts.includes('object "documented"'),
  facts,
);
check(
  "self-facts default the blueprint composition to composed",
  renderSelfFacts({}).includes('object "composed"'),
);

if (failures > 0) {
  console.error(`\n${failures} failure(s).`);
  process.exit(1);
}
console.log("\nIssue #340 worker parity checks passed.");
