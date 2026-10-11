// Issue #918 (E71, R914-6), browser root: `conversation_topic`,
// `source_refresh` and `source_conflict` answer through the same
// data/seed/handler-rules.lino rule sets the native interpreter walks, with
// the `role_slot` and `stable_id` value sources and the `evidence` condition
// in js/worker/formal_ai_worker_handler_rules.js. Prompts and expected
// wording are the ones rust/tests/unit/issue_918_handler_rules_batch.rs
// asserts against the native engine.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await ready;
  return worker.solve(prompt, [], {}, {}, [], {});
}

async function rule(name, prompt) {
  await ready;
  return plain(evaluate(worker,
    `runHandlerRuleSet(${JSON.stringify(name)}, ${JSON.stringify(prompt)}, ${JSON.stringify(prompt.toLowerCase())}, [])`));
}

const TOPIC_EN = "We can talk about existence. I can start with a short definition, context, or a specific question; when web search is available, public facts can be checked against an external source.";
const TOPIC_RU = "Можем. Тема: бытие. Я могу начать с краткого определения, контекста или конкретного вопроса; если веб-поиск доступен, публичные факты можно уточнить через внешний источник.";
const TOPIC_HI = "हम बात कर सकते हैं. विषय: गणित. मैं छोटी परिभाषा, संदर्भ, या किसी ठोस प्रश्न से शुरू कर सकता हूँ; web search उपलब्ध हो तो public facts बाहरी स्रोत से जाँचे जा सकते हैं.";
const TOPIC_ZH = "可以聊。主题: 音乐。我可以从简短定义、上下文或具体问题开始; 如果 web search 可用, 公开事实可以通过外部来源核对。";
const CONFLICT_EN = "The sources you cite disagree: Wikipedia says X was born in 1880; Britannica says 1881. The disagreement is recorded as a conflict:source_disagreement link in the network rather than silently resolved.";
const CONFLICT_UNATTRIBUTED_EN = "No source is named for either answer, so no disagreement between sources can be recorded. Name each source and what it states, for example: Wikipedia says 1880, but Britannica says 1881.";
const REFRESH_EN = "Cached source source_e2db54b48c90e140 has been queued for refresh against its origin URL. The refresh event is appended to the audit log and a fresh fetched_at timestamp will be recorded once the new copy is verified.";
const REFRESH_UNNAMED_EN = "Cached source source_2a324f9681a3e3bd has been queued for refresh against its origin URL. The refresh event is appended to the audit log and a fresh fetched_at timestamp will be recorded once the new copy is verified.";

const EXECUTION_FAILURE = "Execution status: failed in isolated sandbox.\n```python\nundefined_function()\n```\nTraceback (most recent call last):\n  File 'main.py', line 1, in <module>\nNameError: name 'undefined_function' is not defined.\nThe failure trace is appended to the action log; see the trace link.";
const UNITS_EN = "meters measures length; kilogram measures mass. These are different physical dimensions and cannot be converted into each other. The incompatibility is recorded as a `unit_incompatibility` link in the network.";
const UNITS_RU = "метр измеряет length; килограмм измеряет mass. Это разные физические размерности, и их нельзя перевести друг в друга. Несовместимость записана в сети как связь `unit_incompatibility`.";
const KUPI_EN = "Buy an elephant is a well-known Russian children's word game. Whatever you reply, the answer comes back: everyone says that, but you buy an elephant! The traditional winning reply is: everyone has an elephant, but I do not.";
const KUPI_RU = "«Купи слона» — это известная русская детская фраза-игра. На любой ответ следует продолжение: «Все так говорят, а ты купи слона!» Правильный ответ по правилам игры: «У всех есть слон, а у меня нет».";

test("english and russian conversation topics answer unchanged", async () => {
  for (const [prompt, expected] of [
    ["Let's talk about existence", TOPIC_EN],
    ["Поговорим о бытие", TOPIC_RU],
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "conversation_topic", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

test("hindi and chinese openers capture their slot from the seed rule", async () => {
  for (const [prompt, expected, topic] of [
    ["चलो बात करें गणित", TOPIC_HI, "गणित"],
    ["聊聊音乐", TOPIC_ZH, "音乐"],
  ]) {
    const response = await rule("conversation_topic", prompt);
    assert.equal(response.content, expected, prompt);
    assert.ok(response.evidence.includes(`conversation_topic:${topic}`), prompt);
  }
});

test("the topic claim is the rule and an empty slot is no topic", async () => {
  await ready;
  const claims = (prompt) => evaluate(worker,
    `CLASS_CLAIM_EVIDENCE.conversation_topic_subject(${JSON.stringify(prompt)})`);
  assert.equal(claims("Let's talk about existence"), true);
  assert.equal(claims("Let's talk about ?!"), false);
  assert.equal(claims("What is the capital of France?"), false);
});

// Issue #1175 R3: the conflict names the two alternatives it attributes to
// sources; a disjunction that attributes neither is refused by name.
test("a source conflict names its attributed alternatives and refuses without them", async () => {
  const response = await solve("The sources conflict: Wikipedia says X was born in 1880, but Britannica says 1881.");
  assert.equal(response.intent, "source_conflict");
  assert.equal(response.content, CONFLICT_EN);
  assert.ok(response.evidence.some((link) => link.startsWith("conflict:source_disagreement")));
  const unattributed = await solve("Was X born in 1880 or 1881?");
  assert.equal(unattributed.intent, "source_conflict");
  assert.equal(unattributed.content, CONFLICT_UNATTRIBUTED_EN);
  assert.ok(unattributed.evidence.includes("source_conflict:refusal:no attributed alternatives"));
  assert.ok(!unattributed.evidence.some((link) => link.startsWith("conflict:source_disagreement")));
});

test("a source refresh answers unchanged and an unnamed one takes the refusal lane", async () => {
  const named = await solve("Refresh the cached page for example.com");
  assert.equal(named.intent, "source_refresh");
  assert.equal(named.content, REFRESH_EN);

  const unnamed = await rule("source_refresh", "Refresh the cache");
  assert.equal(unnamed.intent, "source_refresh");
  assert.equal(unnamed.content, REFRESH_UNNAMED_EN);
  assert.ok(unnamed.evidence.includes("source_refresh:refusal:no source named"));

  const url = await rule("source_refresh", "Refresh the cached page https://example.com/docs");
  assert.ok(!url.evidence.some((link) => link.startsWith("source_refresh:refusal")));
});

test("an execution failure answers unchanged in chat and agent mode", async () => {
  const chat = await solve("Write a Python script that calls undefined_function()");
  assert.equal(chat.intent, "execution_failure");
  assert.equal(chat.content, EXECUTION_FAILURE);
  const agent = await solve("[agent] Run a Python script that calls undefined_function()");
  assert.equal(agent.content, EXECUTION_FAILURE);
  assert.ok(agent.evidence.some((link) => link.startsWith("agent_mode:opted_in:")));
});

test("an incompatible unit pair answers unchanged from the seeded wording", async () => {
  for (const [prompt, expected] of [
    ["How many meters are in a kilogram?", UNITS_EN],
    ["Сколько метров в килограмме?", UNITS_RU],
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "unit_incompatibility", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

test("the buy-an-elephant idiom answers through its seed rule in the prompt language", async () => {
  // The browser used to answer every language with the Russian text inline;
  // the rule renders the seeded response the native interpreter renders.
  for (const [prompt, expected] of [
    ["Hey, buy an elephant!", KUPI_EN],
    ["Ну купи слона, пожалуйста", KUPI_RU],
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "kupi_slona", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

const RESEARCH_SEARCH = "Search for information about:\n1. Machine learning algorithms\n2. Deep learning vs traditional ML\n3. Neural networks basics";
const RESEARCH_TABLE = "Research comparison table (draft; verify claims against the source links from the preceding retrieval).\n\n| Topic | Key differences | Use cases | Advantages | Disadvantages |\n| --- | --- | --- | --- | --- |\n| Machine learning algorithms | Extract from the preceding source captures what distinguishes this topic from the others. | Extract the practical settings in which the preceding sources apply this topic. | Extract strengths supported by the preceding source captures; leave unsupported claims unverified. | Extract limitations supported by the preceding source captures; leave unsupported claims unverified. |\n| Deep learning vs traditional ML | Extract from the preceding source captures what distinguishes this topic from the others. | Extract the practical settings in which the preceding sources apply this topic. | Extract strengths supported by the preceding source captures; leave unsupported claims unverified. | Extract limitations supported by the preceding source captures; leave unsupported claims unverified. |\n| Neural networks basics | Extract from the preceding source captures what distinguishes this topic from the others. | Extract the practical settings in which the preceding sources apply this topic. | Extract strengths supported by the preceding source captures; leave unsupported claims unverified. | Extract limitations supported by the preceding source captures; leave unsupported claims unverified. |";
const RESEARCH_TASK = "Research task: What would be the economic impact if Rust replaced C++ in all major open-source projects by 2030?\nSteps required:\n1. Search for current C++ vs Rust usage statistics in open-source projects.";
const RESEARCH_PREVIEW = "Research task: What would be the economic impact if Rust replaced C++ in all major open-source projects by 2030? Steps required: 1. Search for current C++ vs Rust usage statistics in open-source projects.";
const RESEARCH_NO_RESULTS = `The result of the previous research step is: no CORS-readable web search results were returned. I do not have verified source data to complete the requested analysis, calculation, table, or sources list yet.\n\nPrior research task: \`${RESEARCH_PREVIEW}\`\n\nNext step: rerun the search with narrower queries or provide source links; then I can calculate the requested impact from those sources.`;
const RESEARCH_OPEN = `There is no verified final research result in the conversation yet. The prior turn was a research request, but I do not see a completed source-backed answer to report.\n\nPrior research task: \`${RESEARCH_PREVIEW}\`\n\nNext step: run the search or provide source links; then I can produce the requested result.`;

test("a research comparison table renders the seeded procedure, never a memorized fact", async () => {
  const search = await solve(RESEARCH_SEARCH);
  await ready;
  const response = await worker.solve(
    "create a comparison table showing:\n- Key differences\n- Use cases for each\n- Advantages and disadvantages",
    [{ role: "user", content: RESEARCH_SEARCH }, { role: "assistant", content: search.content }],
    {}, {}, [], {},
  );
  assert.equal(response.intent, "research_comparison_table");
  assert.equal(response.content, RESEARCH_TABLE);
});

test("a research result follow-up reads its status and wording from the seed", async () => {
  await ready;
  for (const [priorAnswer, expected, status] of [
    ["No CORS-enabled web search results were returned for `x`.\n\nProviders tried: DuckDuckGo.", RESEARCH_NO_RESULTS, "no_results"],
    ["Here is a summary I wrote.", RESEARCH_OPEN, "open_research"],
  ]) {
    const response = await worker.solve(
      "What is the result?",
      [{ role: "user", content: RESEARCH_TASK }, { role: "assistant", content: priorAnswer }],
      {}, {}, [], {},
    );
    assert.equal(response.intent, "research_result_followup", status);
    assert.equal(response.content, expected, status);
    assert.ok(response.evidence.includes(`research_result_followup:status:${status}`), status);
  }
});

test("the network query rules answer the snapshot, the introspection and the user filter", async () => {
  const snapshot = await solve("Export the network");
  assert.equal(snapshot.intent, "network_snapshot");
  await ready;
  const loaded = evaluate(worker, "networkSnapshotLinksNotation()");
  assert.equal(snapshot.content,
    `Here is the current link network as a links-notation snapshot:\n\n\`\`\`links\n${loaded}\n\`\`\``);
  const introspection = await rule("network_query", "What do you know about 'greeting'?");
  assert.equal(introspection.intent, "concept_introspection_greeting");
  assert.equal(introspection.content,
    "Here is what I know about 'greeting':\n\nintent: greeting\nrole: the network records 'greeting' as a concept with rules and example links.");
  const filter = await solve("List the facts I have contributed");
  assert.equal(filter.intent, "filter_user");
  assert.equal(filter.content,
    "No facts have been recorded under your user filter yet. Submit a 'teach this fact' request to start your personal contribution list.");
  assert.ok(filter.evidence.includes("filter:user:self"));
});

// The shell-command rewrite reads its loop and session templates, joiners,
// prompt markers, command heads and prose leads from data/seed/code-task-cues.lino
// in both runtimes; the prompts and answers are the ones
// rust/tests/unit/specification/shared_dialog_replay.rs pins natively.
const SHELL_LOOP = "while true; do sleep 30m && hive-cleanup -f; done";
const SHELL_SCREEN = "screen -dmS auto-cleanup bash -c 'while true; do sleep 30m && hive-cleanup -f; done'";

test("the shell loop rewrite answers unchanged in english, russian, hindi and chinese", async () => {
  for (const instruction of [
    "make a loop of that (infinite), answer with only single line",
    "сделай из этого бесконечный цикл, ответь одной строкой",
    "इसे अनंत लूप बनाओ, केवल एक पंक्ति में उत्तर दो",
    "把它做成无限循环, 只用一行回答",
  ]) {
    const prompt = `box@87ffc301f5eb:~$ sleep 30m && hive-cleanup -f\n\n${instruction}`;
    const response = await solve(prompt);
    assert.equal(response.intent, "shell_command_transform", instruction);
    assert.equal(response.content, SHELL_LOOP, instruction);
  }
  const lead = await solve("Make this a single line loop: sleep 5m && cleanup -f");
  assert.equal(lead.content, "while true; do sleep 5m && cleanup -f; done");
});

test("the screen session rewrite answers unchanged in english, russian, hindi and chinese", async () => {
  await ready;
  for (const followup of [
    "Use `screen -R auto-cleanup` to execute that line inside, answer in one line.",
    "Используй `screen -R auto-cleanup`, выполни эту строку внутри, ответь одной строкой.",
    "`screen -R auto-cleanup` का उपयोग करके उस पंक्ति को अंदर चलाओ, एक पंक्ति में उत्तर दो।",
    "使用 `screen -R auto-cleanup` 在里面执行那一行, 只用一行回答。",
  ]) {
    const response = evaluate(worker,
      `tryShellCommandTransform(${JSON.stringify(followup)}, [{ role: "assistant", content: ${JSON.stringify(SHELL_LOOP)} }])`);
    assert.equal(response.content, SHELL_SCREEN, followup);
  }
});

test("a terse language switch replays the prior answer under the seeded limit", async () => {
  await ready;
  assert.equal(evaluate(worker, 'handlerRulesPolicy("response_language_followup", "terse-word-limit")'), "4");
  const response = await worker.solve("用中文", [
    { role: "user", content: "что ты такое" },
    { role: "assistant", content: "Я formal-ai — детерминированный символьный ИИ, отвечающий по локальным правилам Links Notation." },
  ], {}, {}, [], {});
  assert.equal(response.intent, "identity");
  assert.equal(response.content,
    "我是 formal-ai —— 一个确定性的符号化 AI 系统,根据本地的 Links Notation 规则和兼容 OpenAI 的 API 形式作答。本演示不进行任何神经网络推理。");
  assert.ok(response.evidence.includes("response_language_followup:target:zh"));
});

const BRAINSTORM_FIVE = "1. A local Links Notation notebook with searchable traces.\n2. A deterministic code-review checklist generator.\n3. A multilingual prompt-variation test corpus.\n4. A CLI that converts issue requirements into traceable tests.\n5. A source-cache inspector for reproducible agent runs.";
const BRAINSTORM_TEN = `${BRAINSTORM_FIVE}\n6. A changelog-fragment consistency checker.\n7. A prompt-matrix generator for four-language smoke tests.\n8. A Wikidata anchor verifier for local seed records.\n9. A trace viewer that groups events by solver phase.\n10. A small offline issue-to-test planning tool.`;

test("the seeded brainstorm list reads its count policy from the seed", async () => {
  for (const [prompt, expected] of [
    ["Give me five ideas for an open-source side project.", BRAINSTORM_FIVE],
    ["Suggest ten open-source utilities for developers.", BRAINSTORM_TEN],
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "brainstorm_project_ideas", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

test("a roleplay frame renders the seeded persona and topic", async () => {
  // The prompts and answers rust/tests/unit/specification/prompt_variations_facts.rs pins natively.
  for (const [prompt, expected] of [
    ["Pretend you are Albert Einstein and explain relativity to a teenager.", "Roleplay frame recorded for Albert Einstein. I will keep the persona explicit and factual: relativity says measurements of space and time depend on the observer's motion, while the laws of physics stay consistent."],
    ["Roleplay as a teacher explaining relativity.", "Roleplay frame recorded for teacher. I will keep the persona explicit and factual: relativity says measurements of space and time depend on the observer's motion, while the laws of physics stay consistent."],
    ["Explain like you are Ada Lovelace teaching algorithms.", "Roleplay frame recorded for Ada Lovelace. I will keep the persona explicit and factual: an algorithm is a precise sequence of steps, so a reliable explanation names the inputs, the ordered operations, and the expected result."],
    ["Pretend you are a patient teacher and explain time dilation.", "Roleplay frame recorded for teacher. I will keep the persona explicit and factual: time dilation means clocks can measure different elapsed times when observers move differently or sit in different gravitational fields."],
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "roleplay_explanation", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

const SPIDER_EN = "Released title-role Spider-Man films in release order: 1. Spider-Man (2002); 2. Spider-Man 2 (2004); 3. Spider-Man 3 (2007); 4. The Amazing Spider-Man (2012); 5. The Amazing Spider-Man 2 (2014); 6. Spider-Man: Homecoming (2017); 7. Spider-Man: Into the Spider-Verse (2018); 8. Spider-Man: Far From Home (2019); 9. Spider-Man: No Way Home (2021); 10. Spider-Man: Across the Spider-Verse (2023); 11. Spider-Man: Brand New Day (2026). Announced but not yet released: Spider-Man: Beyond the Spider-Verse (2027-06-18). Source: Wikidata Query Service, snapshot taken 2026-08-04.";
const SPIDER_RU = "Вышедшие фильмы, где Человек-паук — главный герой в порядке выхода: 1. Человек-паук (2002); 2. Человек-паук 2 (2004); 3. Человек-паук 3: Враг в отражении (2007); 4. Новый Человек-паук (2012); 5. Новый Человек-паук. Высокое напряжение (2014); 6. Человек-паук: Возвращение домой (2017); 7. Человек-паук: Через вселенные (2018); 8. Человек-паук: Вдали от дома (2019); 9. Человек-паук: Нет пути домой (2021); 10. Человек-паук: Паутина вселенных (2023); 11. Человек-паук: Новый день (2026). Анонсированы, но ещё не вышли: Человек-паук: За пределами вселенных (2027-06-18). Источник: Wikidata Query Service, снимок данных от 2026-08-04.";

test("a release-timeline fact renders the snapshot against the day it is asked", async () => {
  await ready;
  assert.equal(plain(evaluate(worker, 'renderReleaseTimeline("spider_man_title_role_films", "en", "2026-08-04")')).text, SPIDER_EN);
  assert.equal(plain(evaluate(worker, 'renderReleaseTimeline("spider_man_title_role_films", "ru", "2026-08-04")')).text, SPIDER_RU);
  const response = await solve("List Spider-Man films in release order.");
  assert.equal(response.intent, "fact_lookup");
  const today = new Date().toISOString().slice(0, 10);
  assert.equal(response.content,
    plain(evaluate(worker, `renderReleaseTimeline("spider_man_title_role_films", "en", "${today}")`)).text);
});

test("the concept-query reader takes its vocabulary from the seed cue records", async () => {
  const response = await solve("What does Wikidata mean?");
  assert.equal(response.intent, "concept_lookup");
  assert.equal(response.content,
    "Wikidata (structured-knowledge): Wikidata is a collaboratively edited multilingual knowledge graph hosted by the Wikimedia Foundation. It stores structured data items that power Wikipedia infoboxes and external knowledge applications.\n\nSource: https://en.wikipedia.org/wiki/Wikidata (wikipedia).");
  await ready;
  for (const [prompt, term] of [
    ["Please tell me, what is an algorithm?", "algorithm"],
    ["What does the word lambda mean", "lambda"],
    ["What is the meaning of recursion?", "recursion"],
    ["What does LOL stand for?", "lol"],
    ["Who Ada Lovelace is", "ada lovelace"],
  ]) {
    assert.equal(plain(evaluate(worker, `extractConceptQuery(${JSON.stringify(prompt)})`)).term, term, prompt);
  }
  // A non-subject word is never the interrogated subject of a meaning question.
  assert.equal(evaluate(worker, 'extractMeaningQuestionBody("meaning of it", "meaning of it")'), null);
  assert.equal(evaluate(worker, 'normalizeConceptTerm("The Rust language")'), "rust language");
});

test("a topic summary is the seeded record, run after web search as the native row is", async () => {
  for (const [prompt, expected] of [
    ["Can you summarize Rust?", "Rust is a multi-paradigm, general-purpose programming language that emphasises performance, type safety, and concurrency. It enforces memory safety without using a garbage collector."],
    ["Please summarize formal-ai in one paragraph.", "formal-ai is a deterministic symbolic AI that answers without any neural-network inference."],
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "summarize_topic", prompt);
    assert.equal(response.content, expected, prompt);
  }
  const constrained = await solve("Please summarize formal-ai in one paragraph.");
  assert.ok(constrained.evidence.includes("summarization:constraint:one_paragraph"));
  const conversation = await solve("Summarize our conversation");
  assert.notEqual(conversation.intent, "summarize_topic");
});

const SORT_RUST = "Here is a reviewable sorting algorithm in rust:\n\n```rust\nfn sort(values: &mut Vec<i32>) {\n    values.sort();\n}\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable.";
const SORT_GO = "Вот проверяемый алгоритм сортировки на go:\n\n```python\ndef sort(values):\n    return sorted(values)\n\n```\n\nСтатус выполнения: недоступно в этой среде. Фрагмент предназначен для проверки и копирования.";
const SORT_RUST_HI = "यह rust में एक समीक्षा-योग्य sorting algorithm है:\n\n```rust\nfn sort(values: &mut Vec<i32>) {\n    values.sort();\n}\n```\n\nनिष्पादन स्थिति: इस runtime में उपलब्ध नहीं। यह snippet copy-paste करके समीक्षा करने के लिए है।";
const SORT_TYPESCRIPT_TESTS = "这是一个可审阅的 typescript 排序算法，附带测试：\n\n```typescript\nfunction sort(values) {\n  return [...values].sort((a, b) => a - b);\n}\n```\n\n测试：\n```typescript\nfunction test_sort_ascending() {\n  assert.deepEqual(sort([3,1,2]), [1,2,3]);\n}\n```\n\n执行状态：此运行环境不可用。该代码片段供复制粘贴审阅。";
const SORT_PYTHON_TESTS = "Here is a reviewable sorting algorithm in python with a test:\n\n```python\ndef sort(values):\n    return sorted(values)\n\n```\n\nTests:\n```python\ndef test_sort_ascending():\n    assert sort([3, 1, 2]) == [1, 2, 3]\n\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable.";
const SORT_PYTHON = "Here is a reviewable sorting algorithm in python:\n\n```python\ndef sort(values):\n    return sorted(values)\n\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable.";

// The algorithm rule set (issue #918): the sort operation or the word
// "algorithm", the language table and the seeded snippets answer exactly as
// the deleted handleAlgorithm / try_algorithm did, in every prompt language.
test("the algorithm rule set answers unchanged in english, russian, hindi and chinese", async () => {
  const english = await solve("Write me a sorting algorithm in Rust");
  assert.equal(english.intent, "algorithm_sort_rust");
  assert.equal(english.content, SORT_RUST);
  assert.ok(english.evidence.includes("execution_status:unavailable"));
  for (const [prompt, intent, expected] of [
    ["write a sorting algorithm in python with tests", "algorithm_sort_python", SORT_PYTHON_TESTS],
    ["Напиши сортировку списка на go с тестом", "algorithm_sort_go", SORT_GO],
    ["Rust में सॉर्टिंग एल्गोरिदम लिखो", "algorithm_sort_rust", SORT_RUST_HI],
    ["写一个 typescript 排序 algorithm 带 test", "algorithm_sort_typescript", SORT_TYPESCRIPT_TESTS],
  ]) {
    const response = await rule("algorithm", prompt);
    assert.equal(response.intent, intent, prompt);
    assert.equal(response.content, expected, prompt);
    assert.ok(!response.evidence.some((link) => link.startsWith("algorithm:refusal")), prompt);
  }
  const unnamed = await rule("algorithm", "explain the algorithm");
  assert.equal(unnamed.content, SORT_PYTHON);
  assert.ok(unnamed.evidence.includes("algorithm:refusal:no operation named"));
  assert.equal(await rule("algorithm", "What is the capital of France?"), null);
});

const PLAN_EN_PDF = "This is a document-generation request in PDF format. I am a deterministic symbolic solver: I cannot research arbitrary live data on the web and I do not render binary files directly, so I decompose the task into the formal plan the universal algorithm produces (decompose → tests → drafts → composition). The document workflow uses link-foundation/meta-language for txt, Markdown, HTML, PDF, and DOCX representation/conversion, with concept profiles for headings, paragraphs, lists, strong/bold text, emphasis, and hyperlinks:\n\n1. Scope the document and its criteria: which items to include and which attributes distinguish them.\n2. Collect the list of items from verifiable sources and record links to those sources.\n3. Classify each item against the stated criteria.\n4. Assemble the document structure: title, sections, and a table or list.\n5. Export the finished structure to the requested format.\n\nConfirm the plan or refine the criteria and sources and I will continue with concrete steps. If you need facts that are not in the local Links Notation memory, name a source and I will add it as a links rule.";
const PLAN_RU_PDF = "Это запрос на создание документа в формате PDF. Я детерминированный символьный решатель: у меня нет доступа к произвольным актуальным данным в вебе и я не рендерю бинарные файлы напрямую, поэтому я раскладываю задачу на формальный план по универсальному алгоритму (декомпозиция → проверки → черновики → композиция):\n\n1. Уточнить объём и критерии документа: какие элементы включать и по каким признакам их различать.\n2. Собрать список элементов из проверяемых источников и зафиксировать ссылки на эти источники.\n3. Классифицировать каждый элемент по заявленным критериям.\n4. Собрать структуру документа: заголовок, разделы и таблицу или список.\n5. Экспортировать готовую структуру в запрошенный формат.\n\nПодтвердите план или уточните критерии и источники — и я продолжу с конкретными шагами. Если нужны фактические данные, которых нет в локальной памяти Links Notation, укажите источник, и я добавлю его как правило связей.";
const PLAN_HI_PDF = "यह एक दस्तावेज़ बनाने का अनुरोध है (PDF प्रारूप में). मैं एक नियतात्मक प्रतीकात्मक हल करने वाला हूँ: मैं वेब पर मनमाना सजीव डेटा नहीं खोज सकता और बाइनरी फ़ाइलें सीधे नहीं बनाता, इसलिए मैं इस कार्य को सार्वभौमिक एल्गोरिदम की औपचारिक योजना में विभाजित करता हूँ (विभाजन → जाँच → मसौदे → रचना):\n\n1. दस्तावेज़ और उसके मानदंड का दायरा तय करें: कौन-सी वस्तुएँ शामिल करनी हैं और कौन-से गुण उन्हें अलग करते हैं।\n2. सत्यापन योग्य स्रोतों से वस्तुओं की सूची एकत्र करें और उन स्रोतों के लिंक दर्ज करें।\n3. प्रत्येक वस्तु को बताए गए मानदंड के अनुसार वर्गीकृत करें।\n4. दस्तावेज़ की संरचना बनाएँ: शीर्षक, अनुभाग और एक तालिका या सूची।\n5. तैयार संरचना को अनुरोधित प्रारूप में निर्यात करें।\n\nयोजना की पुष्टि करें या मानदंड और स्रोत स्पष्ट करें, और मैं ठोस चरणों के साथ आगे बढ़ूँगा।";
const PLAN_ZH_PDF = "这是一个生成文档的请求（PDF 格式）。我是一个确定性的符号求解器：我无法在网络上检索任意实时数据，也不会直接渲染二进制文件，因此我把任务分解为通用算法生成的形式化计划（分解 → 校验 → 草稿 → 组合）：\n\n1. 界定文档及其标准：包含哪些条目以及用哪些属性区分它们。\n2. 从可验证的来源收集条目清单，并记录这些来源的链接。\n3. 根据所述标准对每个条目进行分类。\n4. 组装文档结构：标题、章节以及表格或列表。\n5. 将完成的结构导出为所请求的格式。\n\n请确认计划或细化标准与来源，我将继续给出具体步骤。";
const PLAN_RU_DOCUMENT = "Это запрос на создание документа. Я детерминированный символьный решатель: у меня нет доступа к произвольным актуальным данным в вебе и я не рендерю бинарные файлы напрямую, поэтому я раскладываю задачу на формальный план по универсальному алгоритму (декомпозиция → проверки → черновики → композиция):\n\n1. Уточнить объём и критерии документа: какие элементы включать и по каким признакам их различать.\n2. Собрать список элементов из проверяемых источников и зафиксировать ссылки на эти источники.\n3. Классифицировать каждый элемент по заявленным критериям.\n4. Собрать структуру документа: заголовок, разделы и таблицу или список.\n5. Экспортировать готовую структуру в запрошенный формат.\n\nПодтвердите план или уточните критерии и источники — и я продолжу с конкретными шагами. Если нужны фактические данные, которых нет в локальной памяти Links Notation, укажите источник, и я добавлю его как правило связей.";

// The document-generation plan (issue #918): its software, authoring, format
// and noun cues are the document_* tables of data/seed/handler-rules.lino and
// its four-language plan the seeded document_generation_plan responses; every
// answer is byte-identical to the deleted inline templates.
test("the document plan renders the seeded plan in english, russian, hindi and chinese", async () => {
  const english = await solve("Make me a PDF document listing countries with food subsidies for low-income people.");
  assert.equal(english.intent, "document_generation_plan");
  assert.equal(english.content, PLAN_EN_PDF);
  await ready;
  for (const [prompt, expected, format] of [
    ["Сделай мне пдф файл со списком стран, где есть пособия/скидки на еду для малоимущих, как в виде прямых денежных дотаций, так и в косвенной форме, например, талоны.", PLAN_RU_PDF, "PDF"],
    ["गरीब लोगों के लिए खाद्य सब्सिडी वाले देशों की सूची के साथ एक PDF दस्तावेज़ बनाओ।", PLAN_HI_PDF, "PDF"],
    ["给我做一个包含为低收入者提供食品补贴的国家列表的PDF文档。", PLAN_ZH_PDF, "PDF"],
    ["Сделай отчёт по продажам", PLAN_RU_DOCUMENT, "document"],
  ]) {
    const response = plain(evaluate(worker, `tryDocumentGenerationPlan(${JSON.stringify(prompt)})`));
    assert.equal(response.intent, "document_generation_plan", prompt);
    assert.equal(response.content, expected, prompt);
    assert.ok(response.evidence.includes(`document_request:format:${format}`), prompt);
  }
  for (const prompt of ["Build me a CLI tool that generates PDF invoices", "[agent] create report.txt", "Create a list of prime numbers under 100"]) {
    assert.equal(plain(evaluate(worker, `tryDocumentGenerationPlan(${JSON.stringify(prompt)})`)), null, prompt);
  }
});

// The calendar rows (issue #918): the weekday names with the Russian case a
// direction phrase takes, the default event title and every cue list of the
// event parser are calendar_* tables of data/seed/handler-rules.lino; the
// relation, current-day and confirmation sentences are seeded responses.
// Every answer is byte-identical to the deleted inline templates, and the
// native twin pins the same strings in
// rust/tests/unit/issue_918_handler_rules_batch.rs.
const RELATIONS = [
  ["What day of the week comes after Tuesday?", "The day after Tuesday is Wednesday. I move Tuesday by +1 in the seven-day calendar cycle."],
  ["What day comes before Monday?", "The day before Monday is Sunday. I move Monday by -1 in the seven-day calendar cycle."],
  ["какой день недели перед средой", "Перед средой идёт вторник. Я сдвинул среда на -1 в семидневном календарном цикле."],
  ["следующий день после воскресенья", "После воскресенья наступает понедельник. Я сдвинул воскресенье на +1 в семидневном календарном цикле."],
  ["सोमवार के बाद कौन सा दिन आता है", "सोमवार के बाद मंगलवार आता है। मैं सात दिनों के कैलेंडर चक्र में सोमवार को +1 दिन सरकाता हूँ।"],
  ["सोमवार से पहले कौन सा दिन आता है", "सोमवार से पहले रविवार आता है। मैं सात दिनों के कैलेंडर चक्र में सोमवार को -1 दिन सरकाता हूँ।"],
  ["星期一之后是星期几", "星期一之后是星期二。我在七天的日历循环中将星期一移动+1天。"],
  ["星期三之前是星期几", "星期三之前是星期二。我在七天的日历循环中将星期三移动-1天。"],
  ["какой день будет через 100 дней после понедельника?", "Через 100 дней после понедельника — среда. 100 дней = 14 недель + 2 дня; понедельник + 2 дня = среда в семидневном календарном цикле."],
];

test("the weekday relations render the seeded names and sentences unchanged", async () => {
  for (const [prompt, expected] of RELATIONS) {
    const response = await solve(prompt);
    assert.equal(response.intent, "calendar_weekday_relation", prompt);
    assert.equal(response.content, expected, prompt);
  }
  await ready;
  assert.equal(evaluate(worker, `handlerRulesTableValue("calendar_weekday_label", "monday.ru.next")`), "понедельника");
  assert.equal(evaluate(worker, `handlerRulesTableValue("calendar_default_title", "es")`), "Event");
});

const CALENDAR_WEEKDAYS = {
  en: ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
  ru: ["воскресенье", "понедельник", "вторник", "среда", "четверг", "пятница", "суббота"],
  hi: ["रविवार", "सोमवार", "मंगलवार", "बुधवार", "गुरुवार", "शुक्रवार", "शनिवार"],
  zh: ["星期日", "星期一", "星期二", "星期三", "星期四", "星期五", "星期六"],
};

test("the current day renders the seeded sentence in every prompt language", async () => {
  await ready;
  const now = new Date();
  const iso = now.toISOString().slice(0, 10);
  const day = (language) => CALENDAR_WEEKDAYS[language][now.getUTCDay()];
  for (const [prompt, expected] of [
    ["What day is today?", `Today is ${day("en")}, ${iso} (UTC).`],
    ["Какой сегодня день?", `Сегодня ${day("ru")}, ${iso} (UTC).`],
    ["आज कौन सा दिन है?", `आज ${day("hi")} है, ${iso} (UTC).`],
    ["今天是星期几?", `今天是${day("zh")}，${iso}（UTC）。`],
  ]) {
    const response = await worker.solve(prompt, [], {}, { timeZone: "UTC" }, [], {});
    assert.equal(response.intent, "calendar_current_day", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

/** The ICS block, the Google Calendar link and the event date of a confirmation. */
function confirmationParts(content) {
  const end = "END:VCALENDAR\r\n";
  const ics = content.slice(content.indexOf("BEGIN:VCALENDAR"), content.indexOf(end) + end.length);
  const [, year, month, dayOfMonth] = ics.match(/DTSTART;TZID=[^:]+:(\d{4})(\d{2})(\d{2})T/u);
  return {
    ics,
    url: content.match(/https:\/\/calendar\.google\.com\S+/u)[0],
    date: `${year}-${month}-${dayOfMonth}`,
    day: String(Number(dayOfMonth)),
  };
}

test("an event confirmation renders the seeded sentence and the seeded title cues", async () => {
  const tomorrow = new Date(Date.now() + 86_400_000).toISOString().slice(0, 10);
  const cases = [
    ["schedule a call for tomorrow", ({ date, ics, url }) => `Create event «Call» on ${date}. Time: 17:00, timezone: UTC. Duration 60 minutes.\nImport this .ics file into any calendar:\n${ics}\nOr open it in Google Calendar (no login required):\n${url}\nReply 'yes' to confirm.`],
    ["поставь созвон на завтра", ({ date, day, ics, url }) => `Создать событие «Созвон» на ${day} число (${date}). Время: 17:00, часовой пояс: UTC. Длительность 60 минут.\nИмпортируйте этот файл .ics в любой календарь:\n${ics}\nИли откройте в Google Календаре (вход не требуется):\n${url}\nОтветьте «да», чтобы подтвердить.`],
    ["कल मीटिंग शेड्यूल करें", ({ date, ics, url }) => `${date} (17:00, समय क्षेत्र UTC) पर «मीटिंग» कार्यक्रम बनाएँ। अवधि 60 मिनट।\nइस .ics फ़ाइल को किसी भी कैलेंडर में आयात करें:\n${ics}\nया Google Calendar में खोलें (लॉगिन आवश्यक नहीं):\n${url}\nपुष्टि के लिए «हाँ» उत्तर दें।`],
    ["明天安排一个通话", ({ date, ics, url }) => `在 ${date}（17:00，时区 UTC）创建事件「通话」。时长 60 分钟。\n将此 .ics 文件导入任何日历：\n${ics}\n或在 Google 日历中打开（无需登录）：\n${url}\n回复「是」以确认。`],
  ];
  for (const [prompt, expected] of cases) {
    const response = await solve(prompt);
    assert.equal(response.intent, "calendar_create_event", prompt);
    const parts = confirmationParts(response.content);
    assert.equal(parts.date, tomorrow, prompt);
    assert.equal(response.content, expected(parts), prompt);
  }
  // No subject at all takes the seeded default title.
  const untitled = await solve("Book a review on Friday at 09:30");
  assert.ok(untitled.content.startsWith("Create event «Event» on "), untitled.content);
});

// Issue #918 (R918-2): the text_manipulation replacement cues are the text_*
// tables of data/seed/handler-rules.lino. A cue that names its target after
// it ("instead", "вместо") swaps the operands it sits between, so "X instead
// of Y" replaces Y with X; the four replace verbs keep the quoted order. The
// native twin pins the same answers in rust/tests/unit/issue_918_handler_rules_batch.rs.
const TEXT_REPLACEMENTS = [
  ["Write \"cat\" instead of \"dog\": the dog barks", "the cat barks"],
  ["Use \"cat\" instead of \"dog\" in \"the dog barks\"", "the cat barks"],
  ["In \"the dog barks\" write \"cat\" instead of \"dog\"", "the cat barks"],
  ["Instead of \"dog\" use \"cat\": the dog barks", "the cat barks"],
  ["Напиши «кот» вместо «пёс»: пёс лает", "кот лает"],
  ["Вместо «пёс» используй «кот» в тексте «пёс лает»", "кот лает"],
  ["Replace \"cat\" with \"dog\" in this text: \"cat sat with cat\"", "dog sat with dog"],
  ["\"cat sat\": replace \"cat\" with \"dog\"", "dog sat"],
  ["Замени «пёс» на «кот» в тексте «пёс лает»", "кот лает"],
  ["पाठ 'कुत्ता भौंकता है' में 'कुत्ता' को 'बिल्ली' से बदलें", "बिल्ली भौंकता है"],
  ["把文本「狗在叫」中的「狗」替换为「猫」", "猫在叫"],
  ["Uppercase \"replace me\"", "REPLACE ME"],
];

test("text replacements read their cues and operand order from the seed tables", async () => {
  for (const [prompt, expected] of TEXT_REPLACEMENTS) {
    const response = await solve(prompt);
    assert.equal(response.intent, "text_manipulation", prompt);
    assert.equal(response.content, expected, prompt);
  }
  await ready;
  const value = (name, key) => evaluate(worker, `handlerRulesTableValue(${JSON.stringify(name)}, ${JSON.stringify(key)})`);
  assert.equal(value("text_replacement_cue", "instead"), "target_follows_cue");
  assert.equal(value("text_replacement_cue", "замен"), "operands_in_order");
  assert.equal(value("text_input_context_cue", "in"), "ends_with");
});

// Issue #918 (R918-2): document_originality_check reads its attachment
// markers from the document_originality_marker table, its queries, sample
// word limit and default target from the document_originality_check policy,
// and its price-claim heading from seeded responses. The answers are the
// ones the inline literals produced; the native twin pins the same strings.
const ORIGINALITY_EN = "Workflow: read the supplied text, select stable passages, run exact web searches for those passages, then compare any found sources against the document for overlap, citations, and suspicious reuse. I will not claim a uniqueness percentage until matching sources are found and compared.";
const ORIGINALITY_RU = "Рабочий план: прочитать приложенный текст, взять устойчивые фрагменты, выполнить веб-поиск точных совпадений по этим фрагментам, затем сравнить найденные источники с документом и отметить совпадения, цитирования и подозрительные заимствования. Я не буду объявлять процент уникальности без найденных источников и сопоставления текста.";
const ORIGINALITY_HI = "योजना: संलग्न पाठ पढ़ना, स्थिर अंश चुनना, उन अंशों के सटीक मिलान के लिए वेब खोज करना, फिर मिले स्रोतों को दस्तावेज से मिलाकर overlap, citation और संदिग्ध copy को रिपोर्ट करना. स्रोतों से मिलान किए बिना uniqueness प्रतिशत घोषित नहीं किया जाएगा.";
const ORIGINALITY_ZH = "计划：读取附件文本，抽取稳定片段，对这些片段做精确网页搜索，再把找到的来源与文档逐段比对，报告重合、引用和可疑借用。在没有来源匹配前，不会直接给出唯一性百分比。";
const ETH_CLAIMS = "ETH in 2024: $1,700\nETH in 2021: $1,700";
const ETH_CONTRADICTED = "- ETH in 2024: $1,700 is contradicted: Binance ETHUSDT daily klines reports ETH USDT daily candles in 2024 stayed between $2100.00 on 2024-01-03 and $4107.80 on 2024-12-16.";
const ORIGINALITY_CASES = [
  [
    "Check this attached text for uniqueness and plagiarism\n\nAttached files:\n1. article.txt (text/plain, 12.0 KB)\nText excerpt: The tower opened in 1889 and remains a symbol of Paris.",
    `Recognized an originality and plagiarism check for \`article.txt\`.\n\n${ORIGINALITY_EN}\n\nClient text sample: present.`,
    "\"The tower opened in 1889 and remains a symbol of Paris.\" plagiarism originality",
  ],
  [
    "Check this attached text for uniqueness and plagiarism\n\nAttached files:\n1. article.txt (text/plain, 12.0 KB)\nText omitted: the file is too large",
    `Recognized an originality and plagiarism check for \`article.txt\`.\n\n${ORIGINALITY_EN}\n\nClient text sample: read from attachment.`,
    "article.txt plagiarism originality uniqueness",
  ],
  [
    "Check this text for plagiarism: The tower opened in 1889.",
    `Recognized an originality and plagiarism check for \`provided text\`.\n\n${ORIGINALITY_EN}\n\nClient text sample: read from attachment.`,
    "document plagiarism originality uniqueness",
  ],
  [
    `Verify the authenticity and factual accuracy of this attached document\n\nAttached files:\n1. claim.txt (text/plain, 3.0 KB)\nOCR text: ${ETH_CLAIMS}`,
    `Recognized an originality and plagiarism check for \`claim.txt\`.\n\n${ORIGINALITY_EN}\n\nClient text sample: present.\n\nPrice claim check:\n${ETH_CONTRADICTED}`,
    "\"ETH in 2024: $1,700 ETH in 2021: $1,700\" plagiarism originality",
  ],
  [
    `Проверь данный текст на уникальность и на плагиат\n\nAttached files:\n1. eth.txt (text/plain, 1.0 KB)\nText excerpt: ${ETH_CLAIMS}`,
    `Распознал проверку текста на уникальность и плагиат для \`eth.txt\`.\n\n${ORIGINALITY_RU}\n\nТекстовый фрагмент от клиента: получен.\n\nПроверка ценовых утверждений:\n${ETH_CONTRADICTED}`,
    "\"ETH in 2024: $1,700 ETH in 2021: $1,700\" plagiarism originality",
  ],
  [
    `संलग्न पाठ की मौलिकता और plagiarism जांचें\n\nAttached files:\n1. report.txt (text/plain, 8.0 KB)\nText sample: ${ETH_CLAIMS}`,
    `\`report.txt\` के लिए मौलिकता और plagiarism जांच पहचानी गई.\n\n${ORIGINALITY_HI}\n\nClient text sample: present.\n\nमूल्य दावों की जांच:\n${ETH_CONTRADICTED}`,
    "\"ETH in 2024: $1,700 ETH in 2021: $1,700\" plagiarism originality",
  ],
  [
    "检查这个附件文本的原创性和抄袭情况\n\nAttached files:\n1. manuscript.txt (text/plain, 9.0 KB)\nText excerpt: ETH in 2024: $1,700\nText unavailable: page two",
    `已识别 \`manuscript.txt\` 的原创性/抄袭检查请求。\n\n${ORIGINALITY_ZH}\n\nClient text sample: present.\n\n价格声明核查:\n${ETH_CONTRADICTED}`,
    "\"ETH in 2024: $1,700\" plagiarism originality",
  ],
];

test("an originality check reads its markers, queries and heading from the seed", async () => {
  for (const [prompt, expected, query] of ORIGINALITY_CASES) {
    const response = await solve(prompt);
    assert.equal(response.intent, "document_originality_check", prompt);
    assert.equal(response.content, expected, prompt);
    assert.ok(response.evidence.includes(`document_originality_check:request:${query}`), prompt);
  }
});

// Issue #918 (R918-2): software_project_followup renders every sentence from
// the seeded software_project_followup_* responses, its action verbs from the
// software_project_followup_action table and its gates and expected-output
// word limit from the policy of data/seed/handler-rules.lino. The expected
// output opener is any prefix form of the output_display_request role (the
// worker used to list four English openers, so "покажи результат" now records
// its output as the native row already did). The native twin pins the same
// sentences in rust/tests/unit/issue_918_handler_rules_batch.rs.
const SCRAPER_PLAN = "Design a simple web scraper in Python that:\n1. Fetches a webpage\n2. Extracts all headings (h1, h2, h3)\n3. Counts word frequency\n4. Generates a markdown summary";
const PRICES_PLAN = "Write a Python scraper that imports product prices and stores history";
const FOLLOW_UP_TEST = "Recorded a verification follow-up for the scraper from the active plan.\n\nFormalized meaning:\n```lino\nsoftware_project_followup\n  parent_request \"software_project_request_a85d2529097ceda5\"\n  parent_artifact \"scraper\"\n  action \"test\"\n  follow_up_kind verification\n  target_site \"wikipedia.org\"\n  expected_output \"the top 10 most frequent words\"\n  delivery_mode code_generation\n  implementation_language \"python\"\n  approval_state proposed\n  approval_required true\n  approval_gate \"generated_code\"\n  approval_gate \"test_execution\"\n  approval_gate \"network_access\"\n```\n\nReasoning steps:\n1. Recognize \"test\" as a verification request that exercises the scraper from the active plan, not a fact lookup.\n2. Bind the test target to wikipedia.org and keep live fetches behind the network_access gate.\n3. Record the expected output as \"the top 10 most frequent words\" so the test harness can assert it.\n4. Drive the artifact through a deterministic fixture before any host API or network call.\n5. Keep code execution behind approval gates because the sandbox cannot run untrusted code.\n\nVerification plan:\n1. Generate the scraper core plus a deterministic test harness with a captured wikipedia.org fixture.\n2. Assert each requirement (parsing, extraction, counting, summary) against the fixture.\n3. Surface the top 10 most frequent words from the fixture run.\n4. Run the python test command once the generated_code gate is approved.\n5. Promote the run to live wikipedia.org only after the test_execution and network_access gates pass.\n\nReply `approve plan` to generate the artifact plus this test harness. Running it live against the target needs the test_execution and network_access gates.";
const FOLLOW_UP_SHOW_RU = "Записано продолжение типа demonstration для scraper из активного плана.\n\nФормализованный смысл:\n```lino\nsoftware_project_followup\n  parent_request \"software_project_request_a85d2529097ceda5\"\n  parent_artifact \"scraper\"\n  action \"show\"\n  follow_up_kind demonstration\n  expected_output \"результат\"\n  delivery_mode code_generation\n  implementation_language \"python\"\n  approval_state proposed\n  approval_required true\n  approval_gate \"generated_code\"\n  approval_gate \"test_execution\"\n  approval_gate \"network_access\"\n```\n\nШаги рассуждения:\n1. Распознать «show» как запрос типа demonstration, который проверяет scraper из активного плана, а не поиск факта.\n2. Записать ожидаемый вывод как «результат», чтобы тестовый каркас мог его проверить.\n3. Прогнать артефакт через детерминированную фикстуру до любого вызова API хоста или сети.\n4. Держать выполнение кода за шлюзами одобрения, потому что песочница не может запускать недоверенный код.\n\nПлан проверки:\n1. Сгенерировать ядро scraper и детерминированный тестовый каркас с записанной фикстурой сайта (запрошенная цель).\n2. Проверить каждое требование (разбор, извлечение, подсчёт, сводку) на фикстуре.\n3. Показать результат из запуска на фикстуре.\n4. Запустить команду тестов python, как только шлюз generated_code будет одобрен.\n5. Переводить запуск на живой сайт (запрошенная цель) только после прохождения шлюзов test_execution и network_access.\n\nОтветьте `approve plan`, чтобы сгенерировать артефакт вместе с этим тестовым каркасом. Для живого запуска против цели нужны шлюзы test_execution и network_access.";
const FOLLOW_UP_APPROVED = "Recorded a verification follow-up for the scraper from the active plan.\n\nFormalized meaning:\n```lino\nsoftware_project_followup\n  parent_request \"software_project_request_8237833aba5f0f2d\"\n  parent_artifact \"scraper\"\n  action \"test\"\n  follow_up_kind verification\n  delivery_mode code_generation\n  implementation_language \"python\"\n  approval_state approved\n  approval_required true\n  approval_gate \"generated_code\"\n  approval_gate \"test_execution\"\n  approval_gate \"network_access\"\n```\n\nReasoning steps:\n1. Recognize \"test\" as a verification request that exercises the scraper from the active plan, not a fact lookup.\n2. Drive the artifact through a deterministic fixture before any host API or network call.\n3. Keep code execution behind approval gates because the sandbox cannot run untrusted code.\n\nVerification plan:\n1. Generate the scraper core plus a deterministic test harness with a captured the requested target fixture.\n2. Assert each requirement (parsing, extraction, counting, summary) against the fixture.\n3. Run the python test command once the generated_code gate is approved.\n4. Promote the run to live the requested target only after the test_execution and network_access gates pass.\n\nThe plan is approved, so the generated starter already includes this test harness. Running it live needs the test_execution and network_access gates.";

test("a software-project follow-up renders the seeded sentences in both approval states", async () => {
  await ready;
  const plan = await solve(SCRAPER_PLAN);
  const history = [{ role: "user", content: SCRAPER_PLAN }, { role: "assistant", content: plan.content }];
  for (const [prompt, expected] of [
    ["test it by scraping wikipedia.org and show me the top 10 most frequent words.", FOLLOW_UP_TEST],
    ["покажи результат", FOLLOW_UP_SHOW_RU],
  ]) {
    const response = await worker.solve(prompt, history, {}, {}, [], {});
    assert.equal(response.intent, "software_project_followup", prompt);
    assert.equal(response.content, expected, prompt);
  }
  const prices = await solve(PRICES_PLAN);
  const implementation = await worker.solve("approve plan",
    [{ role: "user", content: PRICES_PLAN }, { role: "assistant", content: prices.content }], {}, {}, [], {});
  const approved = await worker.solve("run it and verify the output",
    [{ role: "user", content: PRICES_PLAN }, { role: "assistant", content: implementation.content }], {}, {}, [], {});
  assert.equal(approved.intent, "software_project_followup");
  assert.equal(approved.content, FOLLOW_UP_APPROVED);
});
