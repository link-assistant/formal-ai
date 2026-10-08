// Issue #1175 R3, browser root: the claim rows of
// data/seed/capability-routing.lino are consulted before a handler runs
// (claimRouteAdmits in js/worker/formal_ai_worker_dispatch.js), as
// rust/tests/unit/capability-routing/issue_1175_claim_routing.rs pins for the native router.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

async function admits(handler, prompt) {
  await seeded;
  return plain(evaluate(worker, `claimRouteAdmits(${JSON.stringify(handler)}, ${JSON.stringify(prompt)})`));
}

test("the claim rows are read from the capability table", async () => {
  await seeded;
  assert.deepEqual(plain(evaluate(worker, "claimRouteRows()")), [
    { handler: "software_project", browserHandler: "trySoftwareProjectRequest", admitsOn: ["object_phrase_artifact", "approval_of_a_proposal"], refusalEvents: [] },
    { handler: "terminal_command", browserHandler: "tryTerminalCommand", admitsOn: ["shell_command_shape", "semantic_shell_task"], refusalEvents: [] },
    { handler: "repository_lineage", browserHandler: "", admitsOn: ["repository_subject"], refusalEvents: [] },
    { handler: "page_query_text", browserHandler: "tryPageQueryText", admitsOn: ["supplied_page"], refusalEvents: [] },
    { handler: "javascript_execution", browserHandler: "tryJavaScriptExecution", admitsOn: ["javascript_program"], refusalEvents: [] },
    { handler: "incompatible_units", browserHandler: "tryIncompatibleUnits", admitsOn: ["incompatible_unit_pair"], refusalEvents: [] },
    { handler: "http_fetch", browserHandler: "", admitsOn: ["fetch_url"], refusalEvents: [] },
    { handler: "url_navigate", browserHandler: "", admitsOn: ["navigation_url"], refusalEvents: [] },
    { handler: "calendar_create_event", browserHandler: "", admitsOn: ["calendar_date_signal"], refusalEvents: [] },
    { handler: "code_debugging", browserHandler: "tryCodeDebugging", admitsOn: ["code_artifact"], refusalEvents: [] },
    { handler: "code_explanation", browserHandler: "tryCodeExplanation", admitsOn: ["code_artifact"], refusalEvents: [] },
    { handler: "code_review", browserHandler: "tryCodeReview", admitsOn: ["code_artifact"], refusalEvents: [] },
    { handler: "summarization_text", browserHandler: "trySummarizationText", admitsOn: ["supplied_text"], refusalEvents: [] },
    { handler: "requirement_listing", browserHandler: "requirement_listing", admitsOn: ["supplied_text"], refusalEvents: [] },
    { handler: "text_rewrite", browserHandler: "tryTextRewrite", admitsOn: ["supplied_text"], refusalEvents: [] },
    { handler: "statistics", browserHandler: "tryStatistics", admitsOn: ["stated_number"], refusalEvents: [] },
    { handler: "word_problem", browserHandler: "tryWordProblem", admitsOn: ["stated_number"], refusalEvents: [] },
    { handler: "arithmetic", browserHandler: "tryArithmetic", admitsOn: ["calculation_expression", "currency_rate_basis"], refusalEvents: [] },
    { handler: "compound_interest", browserHandler: "tryCompoundInterest", admitsOn: ["investment_terms", "conversion_target_currency"], refusalEvents: [] },
    { handler: "number_constraint_reasoning", browserHandler: "tryNumberConstraintReasoning", admitsOn: ["interval_bounds"], refusalEvents: [] },
    { handler: "unit_conversion", browserHandler: "tryUnitConversion", admitsOn: ["measured_quantity"], refusalEvents: [] },
    { handler: "calendar_reasoning", browserHandler: "tryCalendarReasoning", admitsOn: ["calendar_date_signal", "calendar_anchor"], refusalEvents: [] },
    { handler: "test_generation", browserHandler: "tryTestGeneration", admitsOn: ["function_under_test"], refusalEvents: ["test_generation:refusal"] },
    { handler: "code_refactoring", browserHandler: "tryCodeRefactoring", admitsOn: ["code_artifact"], refusalEvents: ["code_refactoring:refusal"] },
    { handler: "format_conversion", browserHandler: "tryFormatConversion", admitsOn: ["structured_document"], refusalEvents: ["format_conversion:refusal"] },
    { handler: "brainstorm_composition", browserHandler: "tryBrainstormComposition", admitsOn: ["composition_topic"], refusalEvents: ["brainstorming:refusal"] },
    { handler: "creative_writing", browserHandler: "tryCreativeWritingRequest", admitsOn: ["composition_topic"], refusalEvents: ["creative_writing:refusal"] },
    { handler: "advice_request", browserHandler: "tryAdviceRequest", admitsOn: ["advice_topic"], refusalEvents: ["advice:refusal"] },
    { handler: "planning_request", browserHandler: "tryPlanningRequest", admitsOn: ["cached_destination"], refusalEvents: ["planning:refusal"] },
    { handler: "regex_synthesis", browserHandler: "tryRegexSynthesis", admitsOn: ["pattern_constraints"], refusalEvents: ["regex_synthesis:refusal"] },
    { handler: "sql_synthesis", browserHandler: "trySqlSynthesis", admitsOn: ["table_reference"], refusalEvents: ["sql_synthesis:refusal"] },
    { handler: "shell_command_compose", browserHandler: "tryShellCommandCompose", admitsOn: ["filesystem_object"], refusalEvents: ["shell_command_compose:refusal"] },
    { handler: "program_synthesis", browserHandler: "tryProgramSynthesis", admitsOn: ["function_spec"], refusalEvents: [] },
    { handler: "write_script", browserHandler: "tryWriteScript", admitsOn: ["script_language"], refusalEvents: [] },
    { handler: "document_generation_plan", browserHandler: "tryDocumentGenerationPlan", admitsOn: ["document_format"], refusalEvents: [] },
    { handler: "installation_conversion", browserHandler: "tryInstallationConversion", admitsOn: ["install_steps"], refusalEvents: [] },
    { handler: "execution_failure", browserHandler: "tryExecutionFailure", admitsOn: ["call_expression"], refusalEvents: [] },
    { handler: "concept_lookup", browserHandler: "tryConceptLookup", admitsOn: ["concept_subject"], refusalEvents: [] },
    { handler: "definition_merge", browserHandler: "tryDefinitionMerge", admitsOn: ["definition_merge_term"], refusalEvents: [] },
    { handler: "who_is", browserHandler: "tryWhoIsQuestion", admitsOn: ["concept_subject"], refusalEvents: [] },
    { handler: "how_it_works", browserHandler: "tryHowItWorks", admitsOn: ["mechanism_subject","prior_reply"], refusalEvents: ["how_it_works:refusal"] },
    { handler: "procedural_how_to", browserHandler: "tryProceduralHowTo", admitsOn: ["procedure_task"], refusalEvents: [] },
    { handler: "procedural_how_to_followup", browserHandler: "tryProceduralHowToFollowup", admitsOn: ["prior_procedure"], refusalEvents: [] },
    { handler: "web_search", browserHandler: "tryWebSearch", admitsOn: ["search_focus"], refusalEvents: [] },
    { handler: "conversation_topic", browserHandler: "", admitsOn: ["conversation_topic_subject"], refusalEvents: [] },
    { handler: "learn_from_source", browserHandler: "tryLearnFromSource", admitsOn: ["learnable_source"], refusalEvents: [] },
    { handler: "product_search", browserHandler: "tryProductSearch", admitsOn: ["marketplace_scope"], refusalEvents: ["product_search:refusal"] },
    { handler: "verifiable_task", browserHandler: "tryVerifiableTask", admitsOn: ["verifiable_spec"], refusalEvents: [] },
    { handler: "legality_warning", browserHandler: "tryLegalityWarning", admitsOn: ["legality_assessment"], refusalEvents: [] },
    { handler: "opinion_question", browserHandler: "tryOpinionQuestion", admitsOn: ["assistant_addressee"], refusalEvents: [] },
    { handler: "physical_action_question", browserHandler: "tryPhysicalActionQuestion", admitsOn: ["assistant_addressee"], refusalEvents: [] },
    { handler: "punctuation_only_prompt", browserHandler: "tryPunctuationOnlyPrompt", admitsOn: ["punctuation_only"], refusalEvents: [] },
    { handler: "ill_formed", browserHandler: "tryIllFormed", admitsOn: ["unbalanced_brackets"], refusalEvents: [] },
    { handler: "shell_refusal", browserHandler: "tryShellRefusal", admitsOn: ["shell_command_shape"], refusalEvents: [] },
    { handler: "link_native_synthesis", browserHandler: "tryLinkNativeSynthesis", admitsOn: ["stated_number"], refusalEvents: [] },
    { handler: "software_project_followup", browserHandler: "trySoftwareProjectFollowup", admitsOn: ["prior_software_project"], refusalEvents: [] },
    { handler: "research_result_followup", browserHandler: "tryResearchResultFollowup", admitsOn: ["prior_research_request"], refusalEvents: [] },
    { handler: "research_comparison_table", browserHandler: "tryResearchComparisonTable", admitsOn: ["prior_research_request"], refusalEvents: [] },
    { handler: "coreference", browserHandler: "tryCoreferenceFactLookup", admitsOn: ["coreference_antecedent"], refusalEvents: [] },
    { handler: "response_language_followup", browserHandler: "tryResponseLanguageFollowup", admitsOn: ["prior_user_request"], refusalEvents: [] },
    { handler: "numeric_list", browserHandler: "tryNumericList", admitsOn: ["list_items","prior_numeric_list"], refusalEvents: [] },
    { handler: "text_manipulation", browserHandler: "tryTextManipulation", admitsOn: ["text_operation"], refusalEvents: [] },
    { handler: "shell_command_transform", browserHandler: "tryShellCommandTransform", admitsOn: ["shell_command_operand","prior_reply"], refusalEvents: [] },
    { handler: "write_program_coreference", browserHandler: "tryWriteProgramCoreference", admitsOn: ["prior_program"], refusalEvents: [] },
    { handler: "write_program_concrete", browserHandler: "tryWriteProgramConcrete", admitsOn: ["program_task"], refusalEvents: [] },
    { handler: "program_blueprint_from_prompt", browserHandler: "tryProgramBlueprintFromPrompt", admitsOn: ["program_task"], refusalEvents: [] },
    { handler: "software_project_request", browserHandler: "trySoftwareProjectRequest", admitsOn: ["object_phrase_artifact","approval_of_a_proposal"], refusalEvents: [] },
    { handler: "memory_program", browserHandler: "tryMemoryProgram", admitsOn: ["memory_program_reading"], refusalEvents: [] },
    { handler: "memory_program_gap", browserHandler: "tryMemoryProgramGap", admitsOn: ["memory_program_reading"], refusalEvents: [] },
    { handler: "conversation_control", browserHandler: "", admitsOn: ["backticked_term","prior_reply"], refusalEvents: ["conversation_control:refusal"] },
    { handler: "agentic_continuation", browserHandler: "tryAgenticContinuation", admitsOn: ["prior_reply"], refusalEvents: ["agentic_continuation:refusal"] },
    { handler: "clarification", browserHandler: "tryClarification", admitsOn: ["prior_reply"], refusalEvents: ["clarification:refusal"] },
    { handler: "current_dialogue_fact_checking", browserHandler: "tryCurrentDialogueFactChecking", admitsOn: ["prior_user_request"], refusalEvents: ["current_dialogue_fact_checking:refusal"] },
    { handler: "historical", browserHandler: "tryHistorical", admitsOn: ["dialogue_turn","name_assignment"], refusalEvents: ["conversation_recall:refusal"] },
    { handler: "conversation_memory", browserHandler: "", admitsOn: ["dialogue_turn","name_assignment","recall_query_term","supplied_payload"], refusalEvents: ["conversation_recall:refusal"] },
    { handler: "summarization", browserHandler: "", admitsOn: ["summary_topic"], refusalEvents: ["summarization:refusal"] },
    { handler: "brainstorming", browserHandler: "tryBrainstormingRequest", admitsOn: ["brainstorm_category"], refusalEvents: ["brainstorming:refusal"] },
    { handler: "roleplay", browserHandler: "tryRoleplayRequest", admitsOn: ["persona_or_topic"], refusalEvents: ["roleplay:refusal"] },
    { handler: "document_originality_check", browserHandler: "tryDocumentOriginalityCheck", admitsOn: ["document_operand"], refusalEvents: ["document_originality_check:refusal"] },
    { handler: "translation", browserHandler: "tryTranslation", admitsOn: ["translation_text"], refusalEvents: ["translation:refusal"] },
    { handler: "triz_resolution", browserHandler: "tryTrizResolution", admitsOn: ["triz_precedent"], refusalEvents: ["triz_resolution:refusal"] },
    { handler: "algorithm", browserHandler: "tryAlgorithm", admitsOn: ["algorithm_operation"], refusalEvents: ["algorithm:refusal"] },
    { handler: "source_refresh", browserHandler: "trySourceRefresh", admitsOn: ["source_reference"], refusalEvents: ["source_refresh:refusal"] },
    { handler: "proof_request", browserHandler: "tryProofRequest", admitsOn: ["stated_claim"], refusalEvents: ["proof_request:refusal"] },
    { handler: "capabilities", browserHandler: "tryCapabilities", admitsOn: ["assistant_subject"], refusalEvents: [] },
    { handler: "meta_explanation", browserHandler: "tryMetaExplanation", admitsOn: ["assistant_subject"], refusalEvents: [] },
    { handler: "network_query", browserHandler: "tryNetworkSnapshot", admitsOn: ["assistant_subject"], refusalEvents: [] },
    { handler: "exact_memory_query", browserHandler: "tryExactMemoryQuery", admitsOn: ["memory_query_statement"], refusalEvents: [] },
    { handler: "fact_lookup", browserHandler: "", admitsOn: ["fact_subject"], refusalEvents: [] },
    { handler: "docs_method_explanation", browserHandler: "tryDocsMethodExplanation", admitsOn: ["documented_method"], refusalEvents: [] },
    { handler: "kupi_slona", browserHandler: "tryKupiSlona", admitsOn: ["idiom_utterance"], refusalEvents: [] },
    { handler: "source_conflict", browserHandler: "trySourceConflict", admitsOn: ["attributed_alternatives"], refusalEvents: ["source_conflict:refusal"] },
    { handler: "github_repository_traffic", browserHandler: "tryGithubRepositoryTraffic", admitsOn: ["named_repository"], refusalEvents: ["github_repository_traffic:refusal"] },
    { handler: "formalization_request", browserHandler: "tryFormalizationRequest", admitsOn: ["formalization_statement"], refusalEvents: ["formalization_request:refusal"] },
  ]);
});

test("every row names browser functions and evidence kinds the worker knows", async () => {
  await seeded;
  const unknown = plain(evaluate(worker, `claimRouteRows().flatMap((row) => [
    ...row.admitsOn.filter((kind) => typeof claimEvidence()[kind] !== "function"),
    ...(row.browserHandler && typeof self[row.browserHandler] !== "function"
      && !browserHandlerPrecedence().some((record) => record.name === row.browserHandler && (record.contextBinding || record.ruleSet))
      ? [row.browserHandler] : []),
  ])`));
  assert.deepEqual(unknown, []);
});

async function evidence(kind, prompt) {
  await seeded;
  return plain(evaluate(worker, `claimEvidence()[${JSON.stringify(kind)}](${JSON.stringify(prompt)}, normalizePrompt(${JSON.stringify(prompt)}))`));
}

test("the native-only rows read the same structure in the worker", async () => {
  assert.equal(await evidence("fetch_url", "Fetch https://example.com"), true);
  assert.equal(await evidence("fetch_url", "Fetch me a summary of the news"), false);
  assert.equal(await evidence("navigation_url", "Navigate to github.com"), true);
  assert.equal(await evidence("navigation_url", "Navigate the menu to the settings screen"), false);
  assert.equal(await evidence("calendar_date_signal", "Schedule a meeting with Anna tomorrow at 3pm"), true);
  assert.equal(await evidence("calendar_date_signal", "Explain how a calendar works."), false);
});

test("a surface word alone is not admitted", async () => {
  assert.equal(await admits("tryTerminalCommand", "Make a 3-day itinerary for a first visit to Rome."), false);
  assert.equal(await admits("tryTerminalCommand", "Find the bug: def average(xs): return sum(xs) / len(xs)"), false);
  assert.equal(await admits("trySoftwareProjectRequest", "Write a regex for a US ZIP code with an optional 4-digit extension"), false);
  assert.equal(await admits("tryPageQueryText", "What command builds the jar on this page?"), false);
  assert.equal(await admits("tryJavaScriptExecution", "Run this JavaScript"), false);
  assert.equal(await admits("tryIncompatibleUnits", "What does a unit test check?"), false);
  assert.equal(await admits("tryCodeDebugging", "Find the bug in my plan for the trip"), false);
  assert.equal(await admits("tryCodeExplanation", "Explain how a compiler works"), false);
  assert.equal(await admits("tryCodeReview", "Review my essay about the ocean"), false);
  assert.equal(await admits("trySummarizationText", "Summarize the rust language"), false);
  assert.equal(await admits("tryTextRewrite", "Make this more formal"), false);
  assert.equal(await admits("tryStatistics", "What is the mean of my test scores?"), false);
  assert.equal(await admits("tryStatistics", "Is this number prime?"), false);
  assert.equal(await admits("tryWordProblem", "How many apples are left if I eat some?"), false);
  assert.equal(await admits("tryArithmetic", "Explain how long division works"), false);
  assert.equal(await admits("tryCompoundInterest", "How does compound interest work when you invest?"), false);
  assert.equal(await admits("tryNumberConstraintReasoning", "Guess the hidden number I am thinking of"), false);
  assert.equal(await admits("tryUnitConversion", "Convert this recipe to metric units"), false);
  assert.equal(await admits("tryCalendarReasoning", "Explain how a calendar works."), false);
});

test("the structural evidence admits", async () => {
  assert.equal(await admits("tryTerminalCommand", "find . -name '*.log' -size +10M"), true);
  assert.equal(await admits("trySoftwareProjectRequest", "Build a web app for tracking habits"), true);
  assert.equal(await admits("tryPageQueryText", "What command builds the jar?\nRun kotlinc hello.kt -include-runtime -d hello.jar."), true);
  assert.equal(await admits("tryJavaScriptExecution", "Run this JavaScript: console.log(1 + 2)"), true);
  assert.equal(await admits("tryIncompatibleUnits", "How many meters are in a kilobyte?"), true);
  assert.equal(await admits("tryCodeDebugging", "Find the bug: def average(xs): return sum(xs) / len(xs)"), true);
  assert.equal(await admits("tryCodeExplanation", "Explain this code: print(sum(range(10)))"), true);
  assert.equal(await admits("tryCodeReview", "Review this code: `const total = items.map((item) => item.price)`"), true);
  assert.equal(await admits("trySummarizationText", "Summarize: The parser reads the file. It builds a tree. The tree is checked."), true);
  assert.equal(await admits("tryTextRewrite", "Make this formal: hey can you send me the file"), true);
  assert.equal(await admits("tryStatistics", "What is the mean of 3, 5 and 7?"), true);
  assert.equal(await admits("tryStatistics", "Is 97 a prime number?"), true);
  assert.equal(await admits("tryWordProblem", "Tom has 5 apples and gets 3 more. How many apples does he have?"), true);
  assert.equal(await admits("tryArithmetic", "What is 2 + 2?"), true);
  assert.equal(await admits("tryArithmetic", "what dollar exchange rate do you use for calculations?"), true);
  assert.equal(await admits("tryCompoundInterest", "If I invest $1000 at 8% annual interest compounded monthly for 5 years, how much will I have?"), true);
  assert.equal(await admits("tryCompoundInterest", "convert the final amount to EUR using current exchange rates from the web."), true);
  assert.equal(await admits("tryNumberConstraintReasoning", "Я загадал число больше 1 но меньше 3. что это за число?"), true);
  assert.equal(await admits("tryUnitConversion", "How many meters are in 3 km?"), true);
  assert.equal(await admits("tryCalendarReasoning", "What day comes after Monday?"), true);
  assert.equal(await admits("tryCalendarReasoning", "What day is today?"), true);
  assert.equal(await admits("tryCalendarReasoning", "What day of the week was 2024-02-29?"), true);
  assert.equal(await admits("tryCalendarReasoning", "What month is 2 months after January?"), true);
});

// The numeric rows must not change an admitted answer: each handler, run on
// its own probes, answers only where its row admits (and the one-number
// primality question still reaches statistics).
test("a numeric handler answers only where its row admits", async () => {
  await seeded;
  const answered = plain(evaluate(worker, `[
    ["tryStatistics", "Is 97 a prime number?"], ["tryStatistics", "What is the mean of my test scores?"],
    ["tryUnitConversion", "How many meters are in 3 km?"], ["tryUnitConversion", "Convert this recipe to metric units"],
    ["tryCalendarReasoning", "What day comes after Monday?"], ["tryCalendarReasoning", "Explain how a calendar works."],
  ].map(([name, prompt]) => {
    const normalized = normalizePrompt(prompt);
    const hit = self[name](prompt, normalized, detectLanguage(prompt));
    return [Boolean(hit), claimRouteAdmits(name, prompt, normalized)];
  })`));
  assert.deepEqual(answered, [[true, true], [false, false], [true, true], [false, false], [true, true], [false, false]]);
});

test("a handler with no claim row is admitted as before", async () => {
  assert.equal(await admits("tryAHandlerWithNoRow", "Купи слона"), true);
  assert.equal(await admits("tryKupiSlona", "Купи слона"), true);
});

test("R1175-3 refusal lane: a cued request without its operand keeps only its named refusal", async () => {
  await seeded;
  const admission = (handler, prompt) => plain(evaluate(worker, `claimRouteAdmission(${JSON.stringify(handler)}, ${JSON.stringify(prompt)})`));
  // Without the operand the handler is admitted to refuse only.
  assert.equal(await admission("tryTestGeneration", "Write unit tests"), "refusal");
  assert.equal(await admission("tryCodeRefactoring", "Refactor my morning routine"), "refusal");
  assert.equal(await admission("tryFormatConversion", "Convert this JSON to YAML"), "refusal");
  // With it, every answer the handler gives is admitted.
  assert.equal(await admission("tryTestGeneration", "Write tests for `square(n)`: square(3) returns 9"), "full");
  assert.equal(await admission("tryCodeRefactoring", "Refactor this promise chain with async/await:\n```javascript\nfetch(url).then(r => r.json());\n```"), "full");
  assert.equal(await admission("tryFormatConversion", "Convert this JSON to YAML:\n```json\n{\"a\": 1}\n```"), "full");
  // A refusal recorded by the handler stands; an answer without one is dropped.
  const refused = (handler, hit) => plain(evaluate(worker, `claimRouteRefused(${JSON.stringify(handler)}, ${JSON.stringify(hit)})`));
  assert.equal(await refused("tryTestGeneration", { evidence: ["test_generation:request:cued", "test_generation:refusal:function=none"] }), true);
  assert.equal(await refused("tryTestGeneration", { evidence: ["test_generation:request:cued", "test_generation:suite:case=3"] }), false);
  // End to end the named refusal is still the answer.
  const answer = plain(await evaluate(worker, "solve(\"Write unit tests\", [], {}, {}, [], {})"));
  assert.equal(answer.intent, "test_generation", JSON.stringify(answer));
  assert.ok(answer.content.startsWith("Recognized a test-writing request, but no function under test could be identified"), answer.content);
});

test("R1175-3: the refusal-group evidence reads the handlers' own operands", async () => {
  assert.equal(await evidence("function_under_test", "Write tests for `is_palindrome(s)`"), true);
  assert.equal(await evidence("function_under_test", "Write unit tests"), false);
  assert.equal(await evidence("structured_document", "Convert this YAML to JSON:\n```yaml\nname: formal-ai\n```"), true);
  assert.equal(await evidence("structured_document", "Convert this YAML to JSON"), false);
});

// Issue #1175 R3 classes (b), (c) and (e): a follow-up admits on the earlier
// turn it continues, a composer on its specification (without one it is
// admitted to its refusal lane), a lookup or policy on its subject or shape.
async function evidenceIn(kind, prompt, history = []) {
  await seeded;
  return plain(evaluate(worker, `claimEvidence()[${JSON.stringify(kind)}](${JSON.stringify(prompt)}, normalizePrompt(${JSON.stringify(prompt)}), ${JSON.stringify(history)})`));
}

test("R1175-3 class b: the dialogue evidence reads the earlier turns, never the prompt alone", async () => {
  const research = [{ role: "user", content: "Research the best laptops for programming" }, { role: "assistant", content: "Here is what I found." }];
  assert.equal(await evidenceIn("prior_research_request", "What is the result?", research), true);
  assert.equal(await evidenceIn("prior_research_request", "What is the result?"), false);
  const howTo = [{ role: "user", content: "How to tie a tie?" }, { role: "assistant", content: "Procedural discovery for tie a tie." }];
  assert.equal(await evidenceIn("prior_procedure", "Can you give me specific instructions?", howTo), true);
  assert.equal(await evidenceIn("prior_procedure", "Can you give me specific instructions?"), false);
  assert.equal(await evidenceIn("prior_user_request", "Answer in Russian", howTo), true);
  assert.equal(await evidenceIn("prior_user_request", "Answer in Russian"), false);
  assert.equal(await evidenceIn("prior_reply", "How does it work?", howTo), true);
  assert.equal(await evidenceIn("prior_software_project", "Now test it"), false);
  await seeded;
  assert.equal(plain(evaluate(worker, `claimRouteAdmission("tryResearchResultFollowup", "What is the result?", normalizePrompt("What is the result?"), ${JSON.stringify(research)})`)), "full");
  assert.equal(await admits("tryResearchResultFollowup", "What is the result?"), false);
});

test("R1175-3 class c: a composer without its specification is admitted to its refusal lane only", async () => {
  await seeded;
  const admission = (handler, prompt) => plain(evaluate(worker, `claimRouteAdmission(${JSON.stringify(handler)}, ${JSON.stringify(prompt)})`));
  for (const [handler, bare, specified] of [
    ["tryRegexSynthesis", "Write a regex", "Write a regular expression that matches five digits"],
    ["trySqlSynthesis", "Write a SQL query", "Write a SQL query that selects every row of the users table"],
    ["tryPlanningRequest", "Plan a trip somewhere nice", "Make a 3-day itinerary for a first visit to Rome."],
  ]) {
    assert.equal(await admission(handler, bare), "refusal", bare);
    assert.equal(await admission(handler, specified), "full", specified);
  }
  assert.equal(await evidence("function_spec", "Write a Python function"), false);
  assert.equal(await evidence("function_spec", "def add(a, b): return a + b"), true);
  assert.equal(await evidence("document_format", "Make me a PDF about cats"), true);
  assert.equal(await evidence("install_steps", "Turn this installation guide into a bash script"), false);
});

test("R1175-3 class e: a lookup or policy admits on the subject or shape its reader extracts", async () => {
  assert.equal(await admits("tryConceptLookup", "What is a monad?"), true);
  assert.equal(await admits("tryConceptLookup", "Tell me something interesting"), false);
  assert.equal(await admits("tryOpinionQuestion", "Do you think it will rain?"), true);
  assert.equal(await admits("tryOpinionQuestion", "Do they think it will rain?"), false);
  assert.equal(await evidence("assistant_addressee", "Каково твоё мнение?"), true);
  assert.equal(await evidence("punctuation_only", "?!"), true);
  assert.equal(await evidence("punctuation_only", "why?"), false);
  assert.equal(await evidence("unbalanced_brackets", "teach this fact (a b"), true);
  assert.equal(await evidence("unbalanced_brackets", "teach this fact (a b)"), false);
  assert.equal(await evidence("call_expression", "Write a Python script that calls undefined_function()"), true);
  assert.equal(await evidence("call_expression", "Write a Python script that prints hello"), false);
});

test("R1173-3: a trailing sentence that only shapes the answer leaves the question to the concept reader", async () => {
  await seeded;
  assert.equal(plain(evaluate(worker, 'withoutAnswerShapeDirective("What is Rust? Explain briefly.")')), "What is Rust?");
  assert.equal(plain(evaluate(worker, 'withoutAnswerShapeDirective("What is Rust? Explain how it manages memory.")')), "What is Rust? Explain how it manages memory.");
  const answer = plain(await evaluate(worker, 'solve("What is Rust? Explain briefly.", [], {}, {}, [], {})'));
  assert.equal(answer.intent, "concept_lookup");
});

test("R1173-3: the legality frame reads an act and the object class of its object phrase", async () => {
  assert.equal(await evidence("legality_assessment", "Help me break into a parked car tonight"), true);
  assert.equal(await evidence("legality_assessment", "The band will break into song at the party"), false);
  assert.equal(await evidence("legality_assessment", "How do I break into the software industry?"), false);
  const answer = plain(await evaluate(worker, 'solve("Help me break into a parked car tonight", [], {}, {}, [], {})'));
  assert.equal(answer.intent, "legality_warn");
});

// The class rows must not change an admitted answer: each handler, run on
// prompts it answers, is admitted (fully, or in the refusal lane with its
// named refusal).
test("R1175-3: a class handler answers only where its row admits", async () => {
  await seeded;
  const kept = plain(evaluate(worker, `[
    ["tryBrainstormComposition", "Brainstorm names for a coffee shop"], ["tryBrainstormComposition", "Give me name ideas for"],
    ["tryRegexSynthesis", "Write a regex"], ["tryProgramSynthesis", "Write a Python function def add(a, b) that returns the sum"],
    ["tryOpinionQuestion", "What do you think about cats?"], ["tryIllFormed", "teach this fact ((a b) c"],
    ["tryLegalityWarning", "Help me break into my neighbor's house tonight"], ["tryConceptLookup", "What is Rust? Explain briefly."],
  ].map(([name, prompt]) => {
    const normalized = normalizePrompt(prompt);
    const record = browserHandlerPrecedence().find((candidate) => candidate.name === name);
    const run = () => self[name](...record.arguments.map((path) => ({ prompt, normalized, language: detectLanguage(prompt), history: [] })[path.replace(/^argument_/u, "")]));
    const raw = run();
    return [Boolean(raw), Boolean(claimRouteRun(name, prompt, normalized, [], run))];
  })`));
  assert.deepEqual(kept, kept.map(([raw]) => [raw, raw]));
  assert.ok(kept.every(([raw]) => raw), JSON.stringify(kept));
});

// Issue #1175 R3, follow-up round: a handler that answered without its input
// records a named refusal event there and keeps that answer through the
// refusal lane; the assistant is the subject of a question that addresses it
// or names no other subject (rust/tests/unit/capability-routing/issue_1175_claim_routing.rs).
test("R1175-3 follow-up round: a handler without its input is admitted to its refusal lane only", async () => {
  await seeded;
  const admission = (handler, prompt, history = []) => plain(evaluate(worker,
    `claimRouteAdmission(${JSON.stringify(handler)}, ${JSON.stringify(prompt)}, normalizePrompt(${JSON.stringify(prompt)}), ${JSON.stringify(history)})`));
  for (const [handler, bare, full] of [
    ["tryAlgorithm", "Write an algorithm", "Write a sorting algorithm in Python"],
    ["tryProofRequest", "Prove it", "Prove that 2 + 2 = 4"],
    ["tryTranslation", "Translate to Russian", "Translate \"apple\" to Russian"],
    ["trySourceRefresh", "Refresh the cache", "Refresh the cached page https://example.com/docs"],
  ]) {
    assert.equal(await admission(handler, bare), "refusal", bare);
    assert.equal(await admission(handler, full), "full", full);
  }
  const reply = [{ role: "user", content: "Delete the logs" }, { role: "assistant", content: "Deleted." }];
  assert.equal(await admission("tryClarification", "I don't understand", reply), "full");
  assert.equal(await admission("tryClarification", "I don't understand"), "refusal");
  // The no-input answer records its refusal event, so it still stands.
  const answer = plain(await evaluate(worker, 'solve("Continue", [], {}, {}, [], {})'));
  assert.equal(answer.intent, "continuation_cue");
});

test("R1175-3 follow-up round: the assistant is the subject unless another is named", async () => {
  assert.equal(await evidence("assistant_subject", "What can you do?"), true);
  assert.equal(await evidence("assistant_subject", "show me the network"), true);
  assert.equal(await evidence("assistant_subject", "What is a monad?"), false);
  assert.equal(await evidence("memory_query_statement", "select content from memory"), true);
  assert.equal(await evidence("memory_query_statement", "remember that I like tea"), false);
});

test("R1173-3: a comma after the advice verb still reaches the advice handler", async () => {
  await seeded;
  const answer = plain(await evaluate(worker, 'solve("Посоветуй, как лучше спать", [], {}, {}, [], {})'));
  assert.equal(answer.intent, "advice");
});

// Issue #1175 R3, the last five rows: the handlers that read nothing but their
// cue now read the operand their answer is about (formal_ai_worker_claim_operands.js),
// so every precedence handler carries a row. Held-out probes in en, ru, hi and zh,
// as rust/tests/unit/capability-routing/issue_1175_claim_routing_operands.rs pins natively.
async function operands(kind, prompt) {
  await seeded;
  return plain(evaluate(worker, `CLAIM_OPERANDS[${JSON.stringify(kind)}](${JSON.stringify(prompt)})`));
}

async function admission(handler, prompt) {
  await seeded;
  return plain(evaluate(worker, `claimRouteAdmission(${JSON.stringify(handler)}, ${JSON.stringify(prompt)}, normalizePrompt(${JSON.stringify(prompt)}))`));
}

async function rule(handler, prompt) {
  await seeded;
  return plain(evaluate(worker, `runHandlerRuleSet(${JSON.stringify(handler)}, ${JSON.stringify(prompt)}, ${JSON.stringify(prompt.toLowerCase())}, [])`));
}

test("R1175-3 last five rows: every precedence handler carries a claim row", async () => {
  await seeded;
  const missing = plain(evaluate(worker, `(() => {
    const rows = new Set(claimRouteRows().map((row) => row.handler));
    return Object.keys(WORKER_HANDLER_REGISTRY.workerHandlers).filter((name) => !rows.has(name));
  })()`));
  assert.deepEqual(missing, []);
});

test("R1175-3 last five rows: the documented method is read from the prompt", async () => {
  for (const prompt of ["how does pandas DataFrame.join work?", "как работает pandas DataFrame.join?",
    "pandas DataFrame.join कैसे काम करता है?", "pandas DataFrame.join 如何工作？"]) {
    assert.equal((await operands("documented_method", prompt))[0], "pandas.DataFrame.join", prompt);
    assert.equal(await admission("tryDocsMethodExplanation", prompt), "full", prompt);
  }
  for (const prompt of ["Explain pandas DataFrame.merge", "объясни pandas DataFrame.merge",
    "pandas DataFrame.merge कैसे काम करता है?", "pandas DataFrame.merge 如何工作？"]) {
    assert.equal(await admission("tryDocsMethodExplanation", prompt), "denied", prompt);
  }
});

test("R1175-3 last five rows: the idiom claims only as the whole utterance", async () => {
  for (const [prompt, idiom] of [["Hey, buy an elephant!", "buy an elephant"], ["Ну купи слона, пожалуйста", "купи слона"],
    ["चलो हाथी खरीदो ना", "हाथी खरीदो"], ["快买大象吧", "买大象"]]) {
    assert.deepEqual(await operands("idiom_utterance", prompt), [idiom], prompt);
    assert.equal((await rule("kupi_slona", prompt)).intent, "kupi_slona", prompt);
  }
  for (const prompt of ["Where can I buy an elephant figurine?", "Мама сказала: купи слона в магазине игрушек",
    "दुकान से हाथी खरीदो और घर लाओ", "我想在动物园买大象玩具"]) {
    assert.equal(await admission("tryKupiSlona", prompt), "denied", prompt);
  }
});

test("R1175-3 last five rows: a source conflict names its attributed alternatives", async () => {
  for (const [prompt, first, second] of [
    ["The sources conflict: Wikipedia says Tesla was born in 1856, but an old almanac says 1857.", "Wikipedia says Tesla was born in 1856", "an old almanac says 1857"],
    ["Источники противоречат друг другу: по данным Википедии он родился в 1880 году, а по данным Британники в 1881.", "по данным Википедии он родился в 1880 году", "по данным Британники в 1881"],
    ["स्रोतों में विरोधाभास है: विकिपीडिया के अनुसार वह 1880 में पैदा हुआ, लेकिन ब्रिटानिका के अनुसार 1881 में।", "विकिपीडिया के अनुसार वह 1880 में पैदा हुआ", "ब्रिटानिका के अनुसार 1881 में"],
    ["来源之间有矛盾：维基百科说他生于1880年，大英百科说他生于1881年。", "维基百科说他生于1880年", "大英百科说他生于1881年"],
  ]) {
    assert.equal(await admission("trySourceConflict", prompt), "full", prompt);
    const answer = await rule("source_conflict", prompt);
    assert.ok(answer.content.includes(first) && answer.content.includes(second), answer.content);
  }
  for (const prompt of ["The sources conflict on this question.", "Источники противоречат друг другу.", "स्रोतों में विरोधाभास है।", "来源之间有矛盾。"]) {
    assert.equal(await admission("trySourceConflict", prompt), "refusal", prompt);
    const answer = await rule("source_conflict", prompt);
    assert.ok(answer.evidence.some((link) => link.startsWith("source_conflict:refusal")), prompt);
  }
});

test("R1175-3 last five rows: a traffic answer names the repository the prompt names", async () => {
  for (const [prompt, repository] of [
    ["Can I see who visited my GitHub repository konard/formal-ai?", "konard/formal-ai"],
    ["Можно ли узнать, кто заходил в репозиторий facebook/react на GitHub?", "facebook/react"],
    ["क्या मैं जान सकता हूँ कि GitHub रेपो rust-lang/rust में कौन आया?", "rust-lang/rust"],
    ["能知道谁访问过 GitHub 仓库 vercel/next.js 吗？", "vercel/next.js"],
    ["можно ли узнать заходил ли кто либо в твое репо на github?", "link-assistant/formal-ai"],
  ]) {
    assert.deepEqual(await operands("named_repository", prompt), [repository], prompt);
    assert.ok((await rule("github_repository_traffic", prompt)).content.includes(repository), prompt);
  }
  for (const prompt of ["Can I know who visited my GitHub repo?", "Можно ли узнать, кто заходил в мой репозиторий на GitHub?",
    "क्या मैं जान सकता हूँ कि मेरे GitHub रेपो में कौन आया?", "能知道谁访问过我的 GitHub 仓库吗？"]) {
    assert.equal(await admission("tryGithubRepositoryTraffic", prompt), "refusal", prompt);
    const answer = await rule("github_repository_traffic", prompt);
    assert.ok(!answer.content.includes("link-assistant/formal-ai"), answer.content);
    assert.ok(answer.evidence.some((link) => link.startsWith("github_repository_traffic:refusal")), prompt);
  }
});

test("R1175-3 last five rows: a formalization claims only with a statement", async () => {
  for (const prompt of ["Formalize: all humans are mortal", "Формализуй: все люди смертны",
    "औपचारिक बनाओ: हर छात्र जो पढ़ता है, परीक्षा पास करता है", "形式化：每个学习的学生都通过考试"]) {
    assert.equal(await admission("tryFormalizationRequest", prompt), "full", prompt);
  }
  for (const prompt of ["Formalize this in Lean", "Формализуй это", "इसे औपचारिक बनाओ", "请形式化这个"]) {
    assert.equal(await admission("tryFormalizationRequest", prompt), "refusal", prompt);
    const answer = plain(evaluate(worker, `tryFormalizationRequest(${JSON.stringify(prompt)}, normalizePrompt(${JSON.stringify(prompt)}))`));
    assert.ok(answer.evidence.some((link) => link.startsWith("formalization_request:refusal")), prompt);
  }
});
