// Unit tests for the JavaScript ports of the formalization, meaning-detail,
// diagram and self-referential document recipes (js/agentic/formalize.mjs,
// lexicon.mjs, formalization_recipe.mjs, meaning_detail.mjs, diagram.mjs,
// self_heal.mjs, ledger.mjs, explain.mjs, change_request.mjs,
// repair_strategy.mjs, rebuild_plan.mjs, source_links.mjs,
// document_recipe.mjs and their crate/ dependencies).
//
// Each case mirrors a Rust test (rust/tests/unit/agentic-coding/agentic_coding.rs,
// agentic_surfaces.rs, issue_538_agentic.rs, custom_formalization_subject.rs, issue_558_*.rs,
// issue_1138_formalization_depth.rs); committed artifacts the Rust tests pin
// byte-for-byte (docs/diagrams/agentic-recipes.md, the issue-538 Agent CLI
// sessions, the seed meaning blocks) are the expectations here too.

import { loadRenameMap, resolveRenameChains } from '../../../experiments/formal_ai_subagent/rename-by-rule.mjs';
import assert from 'node:assert/strict';
import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { before, describe, it } from 'node:test';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { parseChatMessage } from '../../../js/server/chat-request.mjs';
import {
  FISHERMAN_DOC_ID, PRIMITIVE_KINDS, canonicalFishermanSynopsis, coversAllNine, formalizeTextToLinks, totalRecords,
} from '../../../js/agentic/formalize.mjs';
import { Lexicon, tokenize } from '../../../js/agentic/lexicon.mjs';
import {
  CANONICAL_SOURCE_URL, KB_PATH, isFormalizationTask, planFormalizationStep, searchQuery,
} from '../../../js/agentic/formalization_recipe.mjs';
import * as meaningDetail from '../../../js/agentic/meaning_detail.mjs';
import * as diagram from '../../../js/agentic/diagram.mjs';
import * as selfHeal from '../../../js/agentic/self_heal.mjs';
import * as ledger from '../../../js/agentic/ledger.mjs';
import * as explain from '../../../js/agentic/explain.mjs';
import * as changeRequest from '../../../js/agentic/change_request.mjs';
import * as repairStrategy from '../../../js/agentic/repair_strategy.mjs';
import * as rebuildPlan from '../../../js/agentic/rebuild_plan.mjs';
import * as sourceLinks from '../../../js/agentic/source_links.mjs';
import {
  SourceLinks, fastStableId, ownedFileCount, ownedManifest, ownedManifestContentId, ownedManifestNotation, ownedSourceFiles,
  ownedTotalBytes,
} from '../../../js/agentic/crate/self_source_links.mjs';
import { sourceCitation } from '../../../js/agentic/crate/self_explanation.mjs';
import { stableId } from '../../../js/agentic/crate/engine_stable_identifier.mjs';
import { sentences, clauses } from '../../../js/agentic/crate/formalization_segment.mjs';
import { unknownSurfaceSpans, unknownSurfaces } from '../../../js/agentic/crate/concept_lookup.mjs';
import { ConceptGraph, formalizeDeeply } from '../../../js/agentic/crate/formalization_concept_links.mjs';
import { offlineRegistryLookup } from '../../../js/agentic/crate/concept_lookup.mjs';

const ROOT = new URL('../../../', import.meta.url);
const repo = (path) => readFileSync(new URL(path, ROOT), 'utf8');

const user = (content) => parseChatMessage({ role: 'user', content });
const callMessage = (id, tool, args) => parseChatMessage({
  role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: tool, arguments: args } }],
});
const toolResult = (id, name, content) => parseChatMessage({ role: 'tool', tool_call_id: id, name, content });

/** Mirrors the Rust tests' `answer_tool_call`. */
function answerCall(messages, call, result) {
  const id = `call_${messages.length}`;
  messages.push(callMessage(id, call.tool, call.arguments));
  messages.push(toolResult(id, call.tool, result));
}
const single = (plan) => {
  assert.equal(plan?.kind, 'tool_calls', JSON.stringify(plan)?.slice(0, 400));
  assert.equal(plan.calls.length, 1);
  return plan.calls[0];
};
const final = (plan) => {
  assert.equal(plan?.kind, 'final', JSON.stringify(plan)?.slice(0, 400));
  return plan.answer;
};
const formalizeStep = (messages, tools) => planFormalizationStep(
  [...messages].reverse().find((message) => message.role === 'user').content, messages, tools,
);

const TASKS = {
  selfHeal: 'When you cannot answer an input, run your self-healing loop: reason about the failure, map it onto the source that would change with a source-to-links round-trip, learn a benchmark-gated lesson, and record the repair case in Links Notation for human approval.',
  ledger: 'Promote the approved lesson into your learning ledger and record the approved learning record in Links Notation so a repeated failure is answered from the ledger next time.',
  explain: 'Explain how Formal AI itself works, and ground the answer in its own source files, data artifacts, and tests rather than prose documentation.',
  change: 'I want to change Formal AI itself: please add a new capability to the system, and route my request through the same human-gated review loop that produces a requirement, a test, and a patch plan I can review as a pull request.',
  repair: 'Classify a failure and decide which part to repair: given a failure trace the system could not answer, determine whether the repair is a solver method, a data record, or a test change, then compose the grounded, human-gated repair strategy for it — for every class of failure, not just one.',
  rebuild: 'An improvement to Formal AI was accepted — now rebuild it and reattach the improved WebAssembly worker to the UI: give me the ordered, reversible steps to regenerate the worker, reattach it to the browser UI, hot-swap the local server, and verify the UI uses the accepted version.',
  sourceLinks: 'Translate the entire source code of our system to the links / meta language and back to source, and record the whole-repository source-to-links projection in Links Notation so we can recompile ourselves.',
  meaning: "Make the tomato meaning more detailed: pin every surface's part of speech and grammatical number, ground it in Wikidata, and add the missing plural to томат.",
  potato: 'Please make the potato word and meaning richer — record the singular/plural of each surface, add the missing plural form potatoes, and keep it grounded in Wikidata.',
  diagram: 'Generate the mermaid diagrams of our agentic recipes, split into parts, as a visual overview of how Formal AI drives its own tools.',
};

before(async () => {
  await installNodeHost(new WorkerHost());
});

describe('formalize (rust/tests/unit/agentic-coding/agentic_coding.rs)', () => {
  it('canonical_synopsis_covers_all_nine_primitives', () => {
    const { summary } = formalizeTextToLinks(canonicalFishermanSynopsis(), '');
    assert.ok(coversAllNine(summary));
    assert.deepEqual(summary.covered, [...PRIMITIVE_KINDS]);
    assert.equal(summary.doc_id, FISHERMAN_DOC_ID);
    assert.equal(summary.concepts, 3);
    assert.equal(summary.predicates, 6);
    assert.equal(summary.assertions, 7);
    assert.equal(summary.procedures, 1);
    assert.equal(summary.contexts, 2);
    assert.equal(summary.temporals, 3);
    assert.equal(summary.modals, 3);
    assert.equal(summary.annotations, 7);
    assert.equal(totalRecords(summary), 37);
    assert.equal(summary.needs_raised, 0);
  });

  it('every_output_record_is_links_notation / grounded_svo_extraction_is_faithful', () => {
    const document = formalizeTextToLinks(canonicalFishermanSynopsis(), '').links_notation;
    assert.ok(document.startsWith('knowledge_base\n  id "tale:fisherman-and-fish"'));
    assert.ok(!document.includes('\t'));
    for (const needle of ['\n  id "a:0"', 'subject "ent:old_man"', 'predicate "pred:catch"', 'object "ent:golden_fish"',
      'time "temporal:в-начале-сказки"', 'context "ctx:seaside"', 'modal "modal:commitment"',
      'provenance "tale:fisherman-and-fish@0:28"', 'object "стать владычицей морской"', 'object_kind "literal"',
      'span "0:28"', 'text "Старик поймал золотую рыбку."']) {
      assert.ok(document.includes(needle), needle);
    }
  });

  it('formalization_is_deterministic', () => {
    const first = formalizeTextToLinks(canonicalFishermanSynopsis(), '');
    const second = formalizeTextToLinks(canonicalFishermanSynopsis(), '');
    assert.deepEqual(first, second);
  });

  it('arbitrary_text_still_produces_a_valid_knowledge_base', () => {
    const formalized = formalizeTextToLinks('A cat sat on a mat. Then it slept.', 'doc:demo');
    const { summary } = formalized;
    assert.equal(summary.doc_id, 'doc:demo');
    assert.equal(summary.annotations, 2);
    assert.equal(summary.assertions, 0);
    assert.equal(summary.procedures, 0);
    assert.equal(summary.contexts, 0);
    assert.ok(!coversAllNine(summary));
    assert.ok(!formalized.links_notation.includes('pred:states'));
    assert.ok(formalized.links_notation.includes('preserved_span\n  id "preserved:0"'));
    assert.ok(formalized.links_notation.includes('language "en"'));
  });

  it('explicit_doc_id_overrides_the_default', () => {
    const formalized = formalizeTextToLinks(canonicalFishermanSynopsis(), 'kb:custom');
    assert.equal(formalized.summary.doc_id, 'kb:custom');
    assert.ok(formalized.links_notation.startsWith('knowledge_base\n  id "kb:custom"'));
  });

  it('lexicon resolves titles by identity and aliases', () => {
    const lexicon = Lexicon.standard();
    assert.equal(lexicon.workForTitle('СКАЗКА  О РЫБАКЕ И РЫБКЕ')?.doc_id, FISHERMAN_DOC_ID);
    assert.equal(lexicon.workForTitle('Сказка о рыбаке'), null);
    assert.equal(lexicon.bestWorkFor('formalize the fisherman tale')?.doc_id, FISHERMAN_DOC_ID);
    assert.equal(lexicon.bestWorkFor('Старик поймал золотую рыбку.'), null);
    assert.deepEqual(tokenize('Hello, мир! 42x'), ['hello', 'мир', '42x']);
  });
});

describe('formalization segmentation and needs (rust/src/formalization)', () => {
  it('segments with exact byte spans and the decimal-point rule', () => {
    const text = 'Pi is 3.14 today. ¿La palabra existe? 猫坐在垫子上。下一句';
    const segments = sentences(text);
    assert.deepEqual(segments.map((segment) => segment.text), ['Pi is 3.14 today.', '¿La palabra existe?', '猫坐在垫子上。', '下一句']);
    const bytes = new TextEncoder().encode(text);
    for (const segment of segments) {
      assert.equal(new TextDecoder().decode(bytes.slice(segment.start, segment.end)), segment.text);
    }
    assert.equal(segments[2].script, 'han');
    assert.deepEqual(clauses(sentences('First, second; third.')[0]).map((clause) => clause.text), ['First', 'second', 'third.']);
  });

  it('unknown surfaces skip quoted spans and seeded words', () => {
    const spans = unknownSurfaceSpans('the «quick brown fox» zyxwvut');
    assert.deepEqual(spans.map(([surface]) => surface), ['zyxwvut']);
    assert.ok(!unknownSurfaces('the "quick brown" zyxwvut', 'en').includes('quick'));
  });

  it('an offline deep pass reports every need unsatisfiable with the empty-structure identity', () => {
    const graph = formalizeDeeply('Старик поймал золотую рыбку.', 'doc:input', offlineRegistryLookup, null, 1);
    assert.ok(graph.needs.length > 0);
    assert.ok(graph.needs.every((need) => need.state === 'unsatisfiable'));
    assert.deepEqual(graph.groundedRatio(), [0, graph.needs.length]);
    assert.equal(graph.identity(), new ConceptGraph().identity());
    assert.ok(graph.toLinksNotation().startsWith(`concept_graph "${graph.identity()}"\n  document "doc:input"\n`));
  });
});

describe('formalization recipe (agentic_coding.rs, agentic_surfaces.rs, custom_formalization_subject.rs)', () => {
  const TOOLS = ['web_search', 'web_fetch', 'write_file', 'run_command'];

  it('recognises formalization tasks in every language', () => {
    assert.ok(isFormalizationTask('Formalize «Сказка о рыбаке и рыбке» into a Links Notation knowledge base.'));
    assert.ok(isFormalizationTask('formalize the fisherman tale'));
    assert.ok(isFormalizationTask('Формализуй «Кот сидел на коврике» в базу знаний Links Notation.'));
    assert.ok(isFormalizationTask('把《猫坐在垫子上》形式化为 Links Notation 知识库。'));
    assert.ok(!isFormalizationTask('What is the capital of France?'));
  });

  it('the acceptance walk: search query, then the canonical fetch, then the final report', () => {
    const task = 'Formalize «Сказка о рыбаке и рыбке» into a Links Notation knowledge base.';
    const messages = [user(task)];
    const search = single(planFormalizationStep(task, messages, ['web_search']));
    assert.equal(search.tool, 'web_search');
    assert.equal(search.arguments, '{"query":"Пушкин Сказка о рыбаке и рыбке полный текст"}');
    assert.equal(searchQuery(), 'Пушкин Сказка о рыбаке и рыбке полный текст');

    answerCall(messages, search, '1. ru.wikisource.org — full text');
    const fetch = single(planFormalizationStep(task, messages, ['web_search', 'web_fetch']));
    assert.equal(fetch.tool, 'web_fetch');
    assert.equal(fetch.arguments, `{"format":"text","url":"${CANONICAL_SOURCE_URL}"}`);

    const answer = final(planFormalizationStep(task, messages, ['web_search']));
    assert.ok(answer.startsWith('Formalized «Сказка о рыбаке и рыбке» into a Links Notation knowledge base: 37 records realising 9 of'), answer);
    assert.ok(answer.includes('9 of 9 protocol primitives'), answer);
    assert.ok(answer.includes('knowledge_base'));
    assert.ok(answer.includes(KB_PATH));
  });

  it('planner_walks_the_full_search_fetch_write_run_recipe', () => {
    const messages = [user('Please formalize «Сказка о рыбаке и рыбке» into a Links Notation knowledge base.')];
    let call = single(formalizeStep(messages, TOOLS));
    assert.equal(call.tool, 'web_search');
    answerCall(messages, call, '1. ru.wikisource.org');
    call = single(formalizeStep(messages, TOOLS));
    assert.equal(call.tool, 'web_fetch');
    assert.ok(call.arguments.includes(CANONICAL_SOURCE_URL));
    answerCall(messages, call, canonicalFishermanSynopsis());
    call = single(formalizeStep(messages, TOOLS));
    assert.equal(call.tool, 'write_file');
    assert.ok(call.arguments.includes(KB_PATH));
    assert.equal(JSON.parse(call.arguments).content, formalizeTextToLinks(canonicalFishermanSynopsis(), '').links_notation);
    answerCall(messages, call, `wrote ${KB_PATH}`);
    call = single(formalizeStep(messages, TOOLS));
    assert.equal(call.tool, 'run_command');
    assert.equal(call.arguments, `{"command":"cat ${KB_PATH}"}`);
    answerCall(messages, call, 'knowledge_base');
    const answer = final(formalizeStep(messages, TOOLS));
    assert.ok(answer.includes('9 of 9 protocol primitives'), answer);
    assert.ok(answer.includes('```lino\nknowledge_base'), answer);
  });

  it('planner_skips_capabilities_no_advertised_tool_provides / completes without tools', () => {
    const messages = [user('formalize the fisherman tale')];
    const call = single(formalizeStep(messages, ['write_file']));
    assert.equal(call.tool, 'write_file');
    answerCall(messages, call, 'ok');
    final(formalizeStep(messages, ['write_file']));
    assert.ok(final(formalizeStep([user('formalize the fisherman tale')], [])).includes('knowledge_base'));
  });

  it('planner_formalizes_the_fetched_text_when_fetch_succeeds', () => {
    const tools = ['web_fetch', 'write_file'];
    const messages = [user('formalize the fisherman tale')];
    const fetch = single(formalizeStep(messages, tools));
    assert.equal(fetch.tool, 'web_fetch');
    answerCall(messages, fetch, 'Старик поймал золотую рыбку.');
    const write = single(formalizeStep(messages, tools));
    assert.equal(write.tool, 'write_file');
    const content = JSON.parse(write.arguments).content;
    assert.ok(content.startsWith(formalizeTextToLinks('Старик поймал золотую рыбку.', '').links_notation));
    assert.ok(content.includes('\nneed '), content);
    assert.ok(content.includes('state unsatisfiable'), content);
  });

  it('planner_falls_back_to_the_synopsis_when_fetch_errors', () => {
    const tools = ['web_fetch', 'write_file'];
    const messages = [user('formalize the fisherman tale')];
    answerCall(messages, single(formalizeStep(messages, tools)), 'Error: 404 Not Found');
    const write = single(formalizeStep(messages, tools));
    assert.equal(JSON.parse(write.arguments).content, formalizeTextToLinks(canonicalFishermanSynopsis(), '').links_notation);
  });

  it('issue 956: a quoted custom source is formalized itself', () => {
    const task = 'Formalize «The cat sat on the mat» into a Links Notation knowledge base.';
    const answer = final(planFormalizationStep(task, [user(task)], []));
    assert.ok(answer.includes('The cat sat on the mat'), answer);
    assert.ok(!answer.includes('рыбке'));
    assert.ok(answer.includes('doc:input'));
    assert.ok(answer.includes('2 records realising 1 of 9 protocol primitives'), answer);
    const call = single(planFormalizationStep(task, [user(task)], ['web_search', 'web_fetch', 'write', 'bash']));
    assert.equal(call.tool, 'write');
    assert.ok(!call.arguments.includes('рыб'));
    for (const [prompt, expected] of [
      ['Формализуй «Кот сидел на коврике» в базу знаний Links Notation.', 'Кот сидел на коврике'],
      ['把《猫坐在垫子上》形式化为 Links Notation 知识库。', '猫坐在垫子上'],
      ['«बिल्ली चटाई पर बैठी» को Links Notation knowledge base में formalize करें।', 'बिल्ली चटाई पर बैठी'],
    ]) {
      const localized = final(planFormalizationStep(prompt, [user(prompt)], []));
      assert.ok(localized.includes(expected), localized);
      assert.ok(!localized.includes('рыбке'));
    }
    for (const prompt of [
      'Formalize «СКАЗКА  О РЫБАКЕ И РЫБКЕ» into a Links Notation knowledge base.',
      'Formalize “Сказка о рыбаке и рыбке” into a knowledge base with label «Reference».',
    ]) {
      assert.ok(final(planFormalizationStep(prompt, [user(prompt)], [])).includes('tale:fisherman-and-fish'));
    }
    const ordered = 'Formalize “First source clause.” into a knowledge base with label «Secondary label».';
    const knowledge = JSON.parse(single(planFormalizationStep(ordered, [user(ordered)], ['write'])).arguments).content;
    assert.ok(knowledge.includes('First source clause.'));
    assert.ok(!knowledge.includes('Secondary label'));
  });
});

describe('meaning detail (rust/tests/unit/agentic-coding/issue_538_agentic.rs)', () => {
  const seed = repo('data/seed/meanings-translation.lino');
  const block = (head, next) => {
    const start = seed.indexOf(`\n  ${head}\n`) + 1;
    const rest = seed.slice(start);
    return rest.slice(0, rest.indexOf(`\n  ${next}\n`) + 1);
  };

  it('recognises_the_meaning_detail_task / routes concepts', () => {
    assert.ok(meaningDetail.isMeaningDetailTask(TASKS.meaning));
    assert.ok(meaningDetail.isMeaningDetailTask('make the помидор meaning more detailed with grammatical number'));
    assert.ok(!meaningDetail.isMeaningDetailTask('What is the capital of France?'));
    assert.equal(meaningDetail.conceptForTask(TASKS.meaning).name, 'tomato');
    assert.equal(meaningDetail.conceptForTask(TASKS.potato).name, 'potato');
    assert.ok(meaningDetail.isMeaningDetailTask(TASKS.potato));
  });

  it('enriched blocks match the seed byte for byte, fetched or not', () => {
    const tomato = meaningDetail.enrichBlock(meaningDetail.TOMATO, null);
    assert.equal(tomato, block('tomato', 'cucumber'));
    assert.equal((tomato.match(/part_of_speech noun/g) || []).length, 9);
    assert.ok(tomato.includes('feature Q146786 # wikidata grammatical feature plural'));
    assert.equal(meaningDetail.enrichBlock(meaningDetail.TOMATO, meaningDetail.sourceBundle(meaningDetail.TOMATO)), tomato);
    assert.equal(meaningDetail.enrichBlock(meaningDetail.TOMATO, 'Error: 404'), tomato);
    assert.equal(meaningDetail.enrichBlock(meaningDetail.POTATO, null), block('potato', 'carrot'));
  });

  for (const [file, task] of [['agent-cli-session.json', TASKS.meaning], ['agent-cli-session-potato.json', TASKS.potato]]) {
    it(`replays the committed ${file} step for step`, async (t) => {
      const { importDocumentRecipe } = await documentRecipeModule();
      if (!importDocumentRecipe) return t.skip('document_recipe.mjs siblings not ported yet');
      const session = JSON.parse(repo(`docs/case-studies/issue-538/${file}`));
      assert.equal(session.task, task);
      const messages = [user(task)];
      for (const step of session.steps) {
        const call = single(await importDocumentRecipe.planMeaningDetailStep(task, messages, session.tools_advertised));
        assert.equal(call.tool, step.tool);
        assert.deepEqual(JSON.parse(call.arguments), step.arguments);
        answerCall(messages, call, step.result);
      }
      assert.equal(final(await importDocumentRecipe.planMeaningDetailStep(task, messages, session.tools_advertised)), session.final_answer);
    });
  }
});

/** document_recipe.mjs imports sibling recipe modules other agents own; skip until they exist. */
async function documentRecipeModule() {
  try {
    return { importDocumentRecipe: await import('../../../js/agentic/document_recipe.mjs') };
  } catch (error) {
    return { importDocumentRecipe: null, error };
  }
}

describe('diagram (rust/tests/unit/agentic-coding/issue_538_agentic.rs)', () => {
  it('recognises_the_diagram_task', () => {
    assert.ok(diagram.isDiagramTask(TASKS.diagram));
    assert.ok(diagram.isDiagramTask('please draw a mermaid flowchart of the recipes'));
    assert.ok(!diagram.isDiagramTask('make the tomato meaning more detailed'));
    assert.ok(!diagram.isDiagramTask('What is the capital of France?'));
    assert.ok(!diagram.isDiagramTask("Insert the line 'js/mermaid.bundle.js' after the line 'js/app.js' in .gitignore."), 'a quoted cue is payload');
  });

  it('committed_diagram_is_generated (docs/diagrams/agentic-recipes.md)', () => {
    const committed = repo('docs/diagrams/agentic-recipes.md');
    assert.equal(diagram.renderDocument(), committed);
    assert.equal((committed.match(/```mermaid/g) || []).length, 5);
  });

  it('replays the committed diagram Agent CLI session', async (t) => {
    const { importDocumentRecipe } = await documentRecipeModule();
    if (!importDocumentRecipe) return t.skip('document_recipe.mjs siblings not ported yet');
    const session = JSON.parse(repo('docs/case-studies/issue-538/agent-cli-session-diagram.json'));
    const messages = [user(session.task)];
    for (const step of session.steps) {
      const call = single(await importDocumentRecipe.planDiagramStep(messages, session.tools_advertised));
      assert.equal(call.tool, step.tool);
      assert.ok(session.tools_advertised.includes(call.tool));
      assert.deepEqual(JSON.parse(call.arguments), step.arguments);
      answerCall(messages, call, step.result);
    }
    assert.equal(final(await importDocumentRecipe.planDiagramStep(messages, session.tools_advertised)), session.final_answer);
  });
});

describe('issue-558 self-referential recipes (rust/tests/unit/issue_558_*.rs)', () => {
  it('task predicates route each canonical task to exactly its recipe', () => {
    const predicates = {
      selfHeal: selfHeal.isSelfHealTask, ledger: ledger.isLedgerTask, explain: explain.isExplainTask,
      change: changeRequest.isChangeRequestTask, repair: repairStrategy.isRepairStrategyTask,
      rebuild: rebuildPlan.isRebuildTask, sourceLinks: sourceLinks.isSourceLinksTask,
    };
    for (const [name, predicate] of Object.entries(predicates)) assert.ok(predicate(TASKS[name]), name);
    assert.ok(!predicates.rebuild(TASKS.selfHeal) && !predicates.rebuild(TASKS.change) && !predicates.rebuild(TASKS.repair));
    for (const sibling of ['selfHeal', 'sourceLinks', 'ledger', 'explain', 'change', 'repair']) {
      assert.ok(!predicates[sibling](TASKS.rebuild), sibling);
    }
    for (const sibling of ['selfHeal', 'sourceLinks', 'ledger', 'explain', 'change']) assert.ok(!predicates[sibling](TASKS.repair), sibling);
    for (const sibling of ['selfHeal', 'sourceLinks', 'ledger', 'explain']) assert.ok(!predicates[sibling](TASKS.change), sibling);
    assert.ok(!predicates.selfHeal(TASKS.ledger) && !predicates.sourceLinks(TASKS.ledger));
    assert.ok(predicates.selfHeal('Can you fix it yourself?'));
    assert.ok(!predicates.selfHeal('fix the bug in main.rs'));
    assert.ok(predicates.rebuild('Please reattach the improved worker to the UI.'));
    assert.ok(predicates.rebuild('Rebuild the wasm worker and hot-swap the local server.'));
    assert.ok(predicates.ledger('record the learning ledger'));
    assert.ok(predicates.ledger('promote the lesson into the promotion ledger'));
    assert.ok(predicates.repair('Please classify a failure and produce a repair strategy.'));
    assert.ok(predicates.repair('Decide which part to repair: solver method, a data record, or a test.'));
    assert.ok(predicates.change('Please change Formal AI itself.'));
    assert.ok(predicates.change('I want to add a new feature to the AI system.'));
    for (const predicate of Object.values(predicates)) {
      assert.ok(!predicate('what files are in this folder?'));
    }
    assert.ok(!predicates.change('change this line to uppercase'));
    assert.ok(!predicates.explain('Explain how Formal AI works\nhttps://github.com/o/r/issues/1\nline\nline\nline'));
  });

  it('committed documents render byte for byte with their final answers', () => {
    assert.equal(selfHeal.renderDocument(), repo('data/meta/self-healing-case.lino'));
    assert.ok(selfHeal.finalAnswer(selfHeal.renderDocument()).includes('mapped it onto src/agentic_coding/planner.rs'));
    assert.ok(selfHeal.finalAnswer(selfHeal.renderDocument()).includes('(awaiting_review)'));
    assert.equal(ledger.renderDocument(), repo('data/meta/learning-ledger.lino'));
    assert.ok(ledger.finalAnswer(ledger.renderDocument()).startsWith('Recorded the approved learning ledger: 1 lesson(s)'));
    assert.equal(repairStrategy.renderDocument(), repo('data/meta/repair-strategies.lino'));
    assert.ok(repairStrategy.finalAnswer(repairStrategy.renderDocument())
      .startsWith('Classified 3 failure traces, one per repair class (solver_method, data_record, test)'));
  });

  it('the change request and rebuild plan are derived from the canonical request', () => {
    const document = changeRequest.renderDocument();
    assert.ok(document.startsWith('change_request\n  id "change_request_'));
    assert.ok(document.includes('  derived_requirement "The system must add a new capability to Formal AI: let users ask it to reverse-sort a program\'s output."'));
    assert.ok(document.includes('  proposed_test "user_requested_change_please_add_a_new_capability_to_formal_ai"'));
    assert.equal(changeRequest.changeRequest().patch_plan.length, 5);
    assert.ok(changeRequest.finalAnswer(document).includes('a 5-step patch plan against the grounded module src/agentic_coding/planner.rs'));
    const plan = rebuildPlan.renderDocument();
    assert.ok(plan.includes(`  change_id "${changeRequest.changeRequest().id}"`));
    assert.equal((plan.match(/\n    step "/g) || []).length, 5);
    assert.ok(rebuildPlan.finalAnswer(plan).includes('a 5-step, reversible pipeline'));
    assert.ok(rebuildPlan.finalAnswer(plan).includes('reattaches it to 4 grounded UI artifacts'));
  });

  it('document recipes walk write -> verify -> final', async (t) => {
    const { importDocumentRecipe } = await documentRecipeModule();
    if (!importDocumentRecipe) return t.skip('document_recipe.mjs siblings not ported yet');
    const tools = ['write_file', 'run_command'];
    const messages = [user(TASKS.ledger)];
    let call = single(await importDocumentRecipe.planLedgerStep(messages, tools));
    assert.equal(call.tool, 'write_file');
    assert.equal(JSON.parse(call.arguments).content, ledger.renderDocument());
    answerCall(messages, call, 'ok');
    call = single(await importDocumentRecipe.planLedgerStep(messages, tools));
    assert.equal(call.arguments, '{"command":"cat learning-ledger.lino"}');
    answerCall(messages, call, ledger.renderDocument());
    assert.equal(final(await importDocumentRecipe.planLedgerStep(messages, tools)), ledger.finalAnswer(ledger.renderDocument()));
  });
});

/** Every `.rs` file under rust/src, enumerated independently of the port. */
function rustSourcePaths(dir = 'rust/src') {
  const out = [];
  for (const name of readdirSync(new URL(dir, ROOT))) {
    const path = `${dir}/${name}`;
    if (statSync(new URL(path, ROOT)).isDirectory()) out.push(...rustSourcePaths(path));
    else if (name.endsWith('.rs')) out.push(path.slice('rust/'.length));
  }
  return out;
}
const utf8 = (text) => new TextEncoder().encode(text).length;

describe('issue #558 owned manifest, self-explanation and source links (rust/tests/unit/issue_558_*.rs)', () => {
  it('owned_manifest_content_addresses_every_source_file', () => {
    const manifest = ownedManifest();
    const files = ownedSourceFiles();
    assert.equal(manifest.length, ownedFileCount());
    assert.equal(manifest.length, files.length);
    assert.ok(manifest.length > 150, 'the whole repository, not a corner');
    assert.deepEqual(manifest.map((digest) => digest.path), rustSourcePaths().sort());
    let total = 0;
    manifest.forEach((digest, index) => {
      const [path, source] = files[index];
      assert.equal(digest.path, path);
      assert.ok(digest.path.startsWith('src/') && digest.path.endsWith('.rs'), digest.path);
      assert.equal(digest.byte_len, utf8(source));
      if (index > 0) assert.ok(manifest[index - 1].path < digest.path, 'sorted and unique');
      total += utf8(source);
    });
    assert.equal(total, ownedTotalBytes());
    assert.equal(ownedManifestContentId(), stableId('source_tree', ownedManifestNotation()));
  });

  it('content ids equal the Rust-generated self-AST census for every undrifted module', () => {
    // data/meta/self-ast/src/**.lino is written by the Rust example
    // regenerate_self_ast_census from the same OWNED_SOURCE_FILES and pinned to
    // the sources by issue_673_self_ast_census.rs. A census whose byte_len no
    // longer matches has drifted with an unregenerated edit and is skipped.
    let compared = 0;
    for (const digest of ownedManifest()) {
      const census = new URL(`data/meta/self-ast/${digest.path.slice(0, -3)}.lino`, ROOT);
      if (!existsSync(census)) continue;
      const text = readFileSync(census, 'utf8');
      if (Number(/\n  byte_len (\d+)/.exec(text)[1]) !== digest.byte_len) continue;
      assert.equal(digest.content_id, /\n  content_id (\S+)/.exec(text)[1], digest.path);
      compared += 1;
    }
    assert.ok(compared > 100, `compared ${compared} modules`);
  });

  it('the fast FNV-1a equals crate::engine::stable_id', () => {
    for (const text of ['', 'a', 'source_tree', 'Привет ✓ 🎉', repo('rust/src/self_source_links.rs')]) {
      assert.equal(fastStableId('source_module', text), stableId('source_module', text));
    }
  });

  it('owned_manifest_renders_valid_links_notation', () => {
    const notation = ownedManifestNotation();
    assert.ok(notation.startsWith('source_manifest\n  engine meta_language\n  language rust\n'));
    assert.ok(notation.includes(`file_count ${ownedFileCount()}`));
    assert.ok(notation.includes(`total_bytes ${ownedTotalBytes()}`));
    assert.ok(notation.includes('src/self_source_links.rs'));
  });

  it('coverage_is_integer_permille_and_flags_unfaithful_modules', () => {
    const fake = (path, faithful) => ({ path, byte_len: 10, content_id: `id_${path}`, total_link_count: 5, named_node_count: 3, faithful });
    const graph = new SourceLinks([fake('a.rs', true), fake('b.rs', true), fake('c.rs', false), fake('d.rs', true)]);
    assert.equal(graph.moduleCount(), 4);
    assert.equal(graph.faithfulCount(), 3);
    assert.equal(graph.coveragePermille(), 750);
    assert.ok(!graph.isFullyFaithful());
    const empty = new SourceLinks([]);
    assert.equal(empty.coveragePermille(), 0);
    assert.ok(!empty.isFullyFaithful());
  });

  it('every source citation is grounded and a fabricated one cannot be constructed', () => {
    const current = explain.explanation();
    const manifest = ownedManifest();
    assert.equal(current.sectionCount(), 5);
    assert.equal(current.citationCount(), 13);
    for (const citation of current.citationsOf('source')) {
      assert.equal(citation.content_id, manifest.find((digest) => digest.path === citation.path).content_id);
    }
    for (const citation of current.citations().filter((entry) => entry.kind !== 'source')) {
      assert.ok(existsSync(new URL(citation.path, ROOT)), citation.path);
      assert.equal(citation.content_id, null);
    }
    assert.ok(current.citationsOf('data').length > 0 && current.citationsOf('test').length > 0);
    assert.throws(() => sourceCitation('src/this_module_does_not_exist.rs'), /self_explanation_unowned_source/);
  });

  it('the explanation document matches the committed Agent CLI evidence modulo the live ids', () => {
    // Preserve the captured Rust document and resolve only recorded path moves.
    // Live source ids are compared separately against the current manifest.
    const renamed = new Map(resolveRenameChains(loadRenameMap().renames).map(({ from, to }) => [from, to]));
    const live = (text) => text.replace(/ (content_id|source_file_count|source_manifest_content_id) .*/g, ' $1 X')
      .replace(/path "([^"]+)"/g, (whole, citation) => {
        const rooted = citation.startsWith('tests/') ? `rust/${citation}` : citation;
        const current = renamed.get(rooted) ?? rooted;
        if (current !== rooted) assert.ok(existsSync(new URL(current, ROOT)), current);
        return `path "${current}"`;
      });
    const document = explain.renderDocument();
    assert.equal(live(document), live(repo('docs/case-studies/issue-839/self-hosting-evidence/how-formal-ai-works.lino')));
    assert.ok(document.includes(`  source_file_count ${ownedFileCount()}\n  source_manifest_content_id "${ownedManifestContentId()}"\n`));
    assert.ok(document.endsWith('\n') && !document.endsWith('\n\n'));
    const answer = explain.finalAnswer(document);
    assert.ok(answer.startsWith('Here is how Formal AI works, grounded in its own source, data, and tests: across 5 topics with 13 citations'));
    assert.ok(answer.endsWith(`Generated document (how-formal-ai-works.lino):\n\n${document.trimEnd()}`));
  });

  it('the source-links document reports the whole repo over the representative slice', () => {
    const document = sourceLinks.renderDocument();
    assert.ok(document.startsWith(`self_source_links\n  engine meta_language\n  language rust\n  task translate_entire_source_to_links_and_back\n  entire_source\n    file_count ${ownedFileCount()}\n    total_bytes ${ownedTotalBytes()}\n    manifest_content_id "${ownedManifestContentId()}"\n  round_trip_proof\n    slice_size 6\n`));
    const files = ownedSourceFiles();
    const stride = Math.floor(files.length / 6);
    assert.deepEqual(sourceLinks.slice().modules.map((module) => module.path), [0, 1, 2, 3, 4, 5].map((index) => files[index * stride][0]));
    // Same shape as the Rust run committed in docs/case-studies/issue-841; the
    // parse census is the no-`meta-language`-feature build's (see the header
    // of js/agentic/crate/self_source_links.mjs).
    const shape = (text) => text.replace(/ (file_count|total_bytes|manifest_content_id|path|byte_len|content_id|total_link_count|named_node_count|total_named_node_count|faithful_count|slice_faithful_count|coverage_permille|fully_faithful|slice_fully_faithful|faithful) .*/g, ' $1 X');
    assert.equal(shape(document), shape(repo('docs/case-studies/issue-841/self-hosting-evidence/self-source-links.lino')));
    assert.ok(document.includes('    slice_fully_faithful false\n'));
    assert.ok(document.endsWith('\n') && !document.endsWith('\n\n'));
    assert.ok(sourceLinks.finalAnswer(document).includes(`content-addressed all ${ownedFileCount()} owned source files under one manifest id, and verified a representative slice of 6 modules`));
  });

  it('the explain and source-links recipes walk write -> verify -> final', async (t) => {
    const { importDocumentRecipe } = await documentRecipeModule();
    if (!importDocumentRecipe) return t.skip('document_recipe.mjs siblings not ported yet');
    const tools = ['write_file', 'run_command'];
    for (const [recipe, task, step, path] of [
      [explain, TASKS.explain, importDocumentRecipe.planExplainStep, 'how-formal-ai-works.lino'],
      [sourceLinks, TASKS.sourceLinks, importDocumentRecipe.planSourceLinksStep, 'self-source-links.lino'],
    ]) {
      const messages = [user(task)];
      let call = single(await step(messages, tools));
      assert.equal(call.tool, 'write_file');
      const written = JSON.parse(call.arguments);
      assert.equal(written.path, path);
      assert.equal(written.content, recipe.renderDocument());
      answerCall(messages, call, 'ok');
      call = single(await step(messages, tools));
      assert.equal(call.arguments, `{"command":"cat ${path}"}`);
      answerCall(messages, call, recipe.renderDocument());
      assert.equal(final(await step(messages, tools)), recipe.finalAnswer(recipe.renderDocument()));
    }
  });
});
