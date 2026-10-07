// Issue #563 (R345-R359) repository resource summarization: the JavaScript
// root's twin of rust/src/summarization/{mod,markdown,dialog,file,resource}.rs
// (js/agentic/crate/summarization*.mjs) is pinned to the Rust behaviour.
//
// Rust twins ported verbatim (same inputs, same assertions):
// * rust/tests/unit/specification/summarization_pipeline.rs (R345-R349,
//   R355-R358): the README / dialog / chat-title / sentence / resource tests;
// * rust/tests/source/source_tests/summarization/mod/tests.rs (R347-R351,
//   R356-R359): classification, summarize, compression, file and resource
//   tests;
// * rust/tests/unit/specification/issue_893_summarization_validation.rs: the
//   two bounded-evidence tests over `formalize_repository_file`;
// * rust/tests/unit/issue_844_statement_merge.rs: the identifier rung
//   (`the_identifier_rung_*`), which `label_for_mode` and
//   `SummarizationMode::Identifier` stand on;
// * rust/tests/unit/issue_858.rs: `summarize_dialog_plain`;
// * rust/examples/issue_563_folder_summary.rs: the example tree.
//
// Oracles. The sampled-file test reads the committed files the Rust test
// `include_str!`s (exact line and byte counts). The conversation fixtures were
// captured from the installed prebuilt binary (`formal-ai 0.347.0`):
// `formal-ai chat --prompt "Summarize conversation: <content>"` runs
// `formalize` -> `summarize_dialog` (Standard) and `generate_chat_title`
// (Topic), so each row pins the JavaScript pipeline against the real Rust
// binary, not against this port's own output. The JavaScript port was written
// from the 0.352.1 source; their agreement on all rows confirms both.
//
// Meta-language evidence: the JavaScript root mirrors the featureless Rust
// build (no parser installed) unless a host calls `installMetaLanguageParser`.
// The plumbing is tested with a stub parser, and the Rust assertions of
// `formalize_repository_file_rust_records_meta_language_and_symbols` run over
// the vendored tree-sitter-rust (summarization_meta_language.mjs). The exact
// link counts are derived from the meta-language 0.58.2 source, not compared
// with a native run, so they are asserted by shape (`\d+`), never by value.
// Assertions tagged `(source-derived)` were derived by hand from the Rust
// source rather than from a Rust run.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { after, before, describe, it } from 'node:test';
import { fileURLToPath } from 'node:url';

import { installHost } from '../../../js/agentic/host.mjs';
import { parseLino, readRepoFile } from '../../../js/server/lino.mjs';
import {
  DEFAULT_MAX_STATEMENTS, StatementKind, SummarizationMode, applyCompoundWords, applySemanticPrimes,
  classifySentence, defaultConfig, deformalize, effectiveMaxStatements, formalize, keepingBoilerplate,
  labelForMode, oneStepShorter, isLabelOnly, statement, statementKindFromSlug, summarize, targetPercent,
  toTopic, withLanguage, withMaxStatements, withMode,
} from '../../../js/agentic/crate/summarization.mjs';
import { meaningsWithRole } from '../../../js/agentic/crate/seed_meanings.mjs';
import {
  describeReadme, formalizeMarkdown, stripMarkdownNoise,
} from '../../../js/agentic/crate/summarization_markdown.mjs';
import {
  assistantTurn, formalizeDialog, generateChatTitle, summarizeDialog, summarizeDialogPlain, userTurn,
} from '../../../js/agentic/crate/summarization_dialog.mjs';
import * as webTreeSitter from '../../../js/vendor/tree-sitter/web-tree-sitter.mjs';
import { loadMetaLanguageParser } from '../../../js/agentic/crate/summarization_meta_language.mjs';
import {
  fileLinksNotation, formalizeRepositoryFile, installMetaLanguageParser, summarizeRepositoryFile,
} from '../../../js/agentic/crate/summarization_file.mjs';
import {
  formalizeRepositoryDirectory, formalizeRepositoryResource, repositoryDirectory, repositoryFile,
  resourceIsDirectory, resourceLinksNotation, resourcePath, summarizeRepositoryResource,
} from '../../../js/agentic/crate/summarization_resource.mjs';
import {
  NamingConvention, commitSubjectBudget, identifierBudget, isValidIdentifier,
  toIdentifier,
} from '../../../js/agentic/crate/summarization_identifier.mjs';

const root = new URL('../../../', import.meta.url);
const read = (path) => readFileSync(new URL(path, root), 'utf8');

before(() => {
  installHost({ readText: readRepoFile, parseLino, realm: {} });
});

after(() => installMetaLanguageParser(null));

const config = (mode) => withMode(defaultConfig(), mode);
const words = (text) => text.split(/\s+/u).filter(Boolean);
const texts = (statements) => statements.map((candidate) => candidate.text);

const sampleStatements = () => [
  statement('X is the AI that controls AIs.', StatementKind.Purpose, 100),
  statement('X is written in JavaScript.', StatementKind.Language, 60),
  statement('X orchestrates multiple agents.', StatementKind.Feature, 70),
  statement('Install X with npm install x.', StatementKind.Install, 10),
  statement('Run x --help for flags.', StatementKind.Example, 10),
];

describe('R345-R349: the statement pipeline (mod.rs)', () => {
  it('default_max_statements_is_thirty', () => {
    assert.equal(DEFAULT_MAX_STATEMENTS, 30);
  });

  it('summarization_mode_target_percent_matches_vision', () => {
    assert.equal(targetPercent(SummarizationMode.Topic), 0);
    assert.equal(targetPercent(SummarizationMode.Identifier), 0);
    assert.equal(targetPercent(SummarizationMode.Short), 20);
    assert.equal(targetPercent(SummarizationMode.Standard), 50);
    assert.equal(targetPercent(SummarizationMode.Full), 100);
    assert.equal(targetPercent(SummarizationMode.Expand), 200);
  });

  it('the_mode_ladder_steps_down_to_the_identifier_fixed_point (R357)', () => {
    assert.equal(oneStepShorter(SummarizationMode.Expand), SummarizationMode.Standard);
    assert.equal(oneStepShorter(SummarizationMode.Full), SummarizationMode.Standard);
    assert.equal(oneStepShorter(SummarizationMode.Standard), SummarizationMode.Short);
    assert.equal(oneStepShorter(SummarizationMode.Short), SummarizationMode.Topic);
    assert.equal(oneStepShorter(SummarizationMode.Topic), SummarizationMode.Identifier);
    assert.equal(oneStepShorter(SummarizationMode.Identifier), SummarizationMode.Identifier);
    assert.ok(isLabelOnly(SummarizationMode.Identifier));
    assert.ok(isLabelOnly(SummarizationMode.Topic));
    assert.ok(!isLabelOnly(SummarizationMode.Short));
  });

  it('formalize_splits_on_punctuation', () => {
    const statements = formalize('Foo is bar. Foo helps you ship! What is foo?');
    assert.equal(statements.length, 3);
    assert.ok(statements[0].text.endsWith('.'));
    assert.ok(statements[1].text.endsWith('!'));
    assert.ok(statements[2].text.endsWith('?'));
  });

  it('formalize_keeps_decimal_points_inside_sentences', () => {
    assert.deepEqual(
      texts(formalize('The adapter supplies 19.5 V. Its plug is 3.5 x 1.35 mm.')),
      ['The adapter supplies 19.5 V.', 'Its plug is 3.5 x 1.35 mm.'],
    );
  });

  it('sentence_formalization_preserves_decimals_and_initialisms', () => {
    assert.deepEqual(
      texts(formalize('The next U.S. general election is on Tuesday, November 3, 2026. Turnout was 63.7%.')),
      ['The next U.S. general election is on Tuesday, November 3, 2026.', 'Turnout was 63.7%.'],
    );
  });

  it('classify_picks_install_for_npm_install', () => {
    assert.equal(classifySentence('Install foo with npm install foo.'), StatementKind.Install);
  });

  it('classify_scans_summary_cue_meanings_in_declaration_order', () => {
    const slugs = meaningsWithRole('summary_classification_cue').map((meaning) => meaning.slug);
    assert.deepEqual(slugs, [
      'summary_kind_install',
      'summary_kind_example',
      'summary_kind_language',
      'summary_kind_stars',
      'summary_kind_purpose',
      'summary_kind_use_case',
      'summary_kind_feature',
    ]);
    for (const [slug, kind] of [
      ['summary_kind_install', StatementKind.Install],
      ['summary_kind_example', StatementKind.Example],
      ['summary_kind_language', StatementKind.Language],
      ['summary_kind_stars', StatementKind.Stars],
      ['summary_kind_purpose', StatementKind.Purpose],
      ['summary_kind_use_case', StatementKind.UseCase],
      ['summary_kind_feature', StatementKind.Feature],
    ]) {
      assert.equal(statementKindFromSlug(slug), kind, `from_slug(${slug})`);
    }
    assert.equal(statementKindFromSlug('not_a_summary_slug'), StatementKind.Misc);
  });

  it('classify_recognizes_each_kind_through_seed_surfaces', () => {
    for (const [sentence, kind] of [
      ['To install, run the setup script.', StatementKind.Install],
      ['For example, see the snippet below.', StatementKind.Example],
      ['Written in Rust.', StatementKind.Language],
      ['It has 5000 github stars.', StatementKind.Stars],
      ['It helps you ship faster.', StatementKind.Purpose],
      ['Use it when you deploy to production.', StatementKind.UseCase],
      ['It supports plugins.', StatementKind.Feature],
      ['Just a plain sentence about nothing.', StatementKind.Misc],
    ]) {
      assert.equal(classifySentence(sentence), kind, `classify(${sentence})`);
    }
  });

  it('classify_language_guard_falls_through_on_long_sentences', () => {
    assert.equal(classifySentence('X is a tool.'), StatementKind.Language);
    const long = 'This project is a comprehensive toolkit that supports plugins and provides many helpful features for everyone.';
    assert.ok(words(long).length > 12);
    assert.equal(classifySentence(long), StatementKind.Feature);
  });

  it('formalize_assigns_the_kind_weights_of_weight_for_kind', () => {
    const statements = formalize('Hive Mind is the AI that controls AIs. Install with npm install x.');
    assert.deepEqual(statements, [
      { text: 'Hive Mind is the AI that controls AIs.', kind: StatementKind.Purpose, weight: 100 },
      { text: 'Install with npm install x.', kind: StatementKind.Install, weight: 10 },
    ]);
  });

  it('summarize_short_keeps_highest_weight', () => {
    const out = summarize(sampleStatements(), config(SummarizationMode.Short));
    assert.ok(out.length > 0);
    assert.equal(out[0].kind, StatementKind.Purpose);
    // Short mode + 3 retained statements after dropping boilerplate:
    // effective_max_statements = max(1, round(3 * 0.2)) = 1.
    assert.equal(out.length, 1);
  });

  it('summarize_drops_install_and_example_by_default', () => {
    const out = summarize(sampleStatements(), config(SummarizationMode.Full));
    assert.ok(out.every((candidate) => candidate.kind !== StatementKind.Install));
    assert.ok(out.every((candidate) => candidate.kind !== StatementKind.Example));
    assert.equal(out.length, 3);
  });

  it('summarize_full_keeps_install_when_drop_boilerplate_false', () => {
    const out = summarize(sampleStatements(), keepingBoilerplate(config(SummarizationMode.Full)));
    assert.ok(out.some((candidate) => candidate.kind === StatementKind.Install));
    assert.ok(out.some((candidate) => candidate.kind === StatementKind.Example));
  });

  it('summarize_expand_with_primes_grows_output', () => {
    const expand = { ...config(SummarizationMode.Expand), use_semantic_primes: true };
    const out = summarize([statement('X orchestrates multiple agents.', StatementKind.Feature, 70)], expand);
    assert.deepEqual(out, [
      { text: 'X orchestrates multiple agents.', kind: StatementKind.Feature, weight: 70 },
      { text: 'X controls many multiple agents.', kind: StatementKind.Feature, weight: 65 },
    ]);
  });

  it('summarize_topic_returns_at_most_one_statement', () => {
    assert.ok(summarize(sampleStatements(), config(SummarizationMode.Topic)).length <= 1);
  });

  it('summarize_orders_equal_weights_stably', () => {
    const tied = ['One.', 'Two.', 'Three.'].map((text) => statement(text, StatementKind.Feature, 50));
    assert.deepEqual(texts(summarize(tied, config(SummarizationMode.Full))), ['One.', 'Two.', 'Three.']);
  });

  it('deformalize_joins_statements_with_period_terminator', () => {
    const rendered = deformalize([
      statement('Hello world', StatementKind.Identity, 100),
      statement('Foo bars', StatementKind.Misc, 50),
    ]);
    assert.equal(rendered, 'Hello world. Foo bars.');
  });

  it('to_topic_clamps_to_five_words', () => {
    assert.ok(words(toTopic('', sampleStatements())).length <= 5);
  });

  it('to_topic_uses_explicit_topic_when_present', () => {
    assert.equal(toTopic('Hive Mind', []), 'Hive Mind');
  });

  it('to_topic_takes_the_last_of_equally_weighted_statements (Iterator::max_by_key)', () => {
    const tied = [
      statement('First tied statement here.', StatementKind.Misc, 30),
      statement('Second tied statement here.', StatementKind.Misc, 30),
    ];
    assert.equal(toTopic('', tied), 'Second tied statement here');
  });

  it('apply_compound_words_shortens_english_phrases', () => {
    const result = applyCompoundWords('Run in order to ship the user interface.', 'en');
    assert.equal(result, 'Run to ship the UI.');
  });

  it('apply_semantic_primes_expands_orchestrates', () => {
    assert.equal(applySemanticPrimes('X orchestrates agents.', 'en'), 'X controls many agents.');
  });

  it('apply_semantic_primes_supports_russian', () => {
    assert.ok(applySemanticPrimes('X автоматизация всего.', 'ru').includes('когда машина делает'));
  });

  it('compound_words_and_semantic_primes_are_reversible_by_size', () => {
    const prose = 'the AI orchestrates multiple agents';
    assert.ok(words(applyCompoundWords(prose, 'en')).length <= words(prose).length);
    assert.ok(words(applySemanticPrimes(prose, 'en')).length >= words(prose).length);
  });

  it('effective_max_statements_clamps_explicit_cap', () => {
    const capped = withMaxStatements(config(SummarizationMode.Standard), 2);
    assert.equal(effectiveMaxStatements(capped, 10), 2);
    assert.equal(effectiveMaxStatements(capped, 0), 0);
  });

  it('effective_max_statements_topic_returns_one', () => {
    const topic = config(SummarizationMode.Topic);
    assert.equal(effectiveMaxStatements(topic, 10), 1);
    assert.equal(effectiveMaxStatements(topic, 0), 0);
  });

  it('formalize_summarize_deformalize_round_trip_keeps_meaning', () => {
    const statements = formalize('Formal AI is a symbolic engine. It runs offline. Install with cargo. Run with cargo run.');
    assert.ok(statements.length > 0);
    const kept = summarize(statements, withLanguage(config(SummarizationMode.Full), 'en'));
    assert.ok(deformalize(kept).includes('Formal AI is a symbolic engine'));
  });

  it('default_max_statements_constant_is_thirty', () => {
    assert.equal(DEFAULT_MAX_STATEMENTS, 30);
    const statements = Array.from({ length: 50 }, (_, index) => statement(`Sentence ${index}.`, StatementKind.Feature, 50));
    const out = summarize(statements, withMaxStatements(config(SummarizationMode.Full), DEFAULT_MAX_STATEMENTS));
    assert.equal(out.length, DEFAULT_MAX_STATEMENTS);
  });
});

describe('R349: README ingestion (markdown.rs)', () => {
  it('strip_markdown_noise_drops_badges_html_comments_and_code_blocks', () => {
    const markdown = '# Title\n\n[![ci](https://example.com/ci.svg)](https://example.com/ci)\n\n<!-- internal -->\n\nFormal AI is a deterministic symbolic engine.\n\n```bash\nnpm install formal-ai\n```\n\nIt runs offline.';
    const stripped = stripMarkdownNoise(markdown);
    assert.ok(!stripped.includes('![ci]'), stripped);
    assert.ok(!stripped.includes('<!--'), stripped);
    assert.ok(!stripped.includes('npm install'), stripped);
    assert.ok(stripped.includes('Formal AI is a deterministic symbolic engine.'), stripped);
  });

  it('strip_markdown_noise_has_one_output_line_per_input_line', () => {
    assert.equal(
      stripMarkdownNoise('# Title\n\n> It orchestrates multiple agents.\n- item `code` <b>bold</b>\n'),
      'Title\n\nIt orchestrates multiple agents.\nitem code bold\n',
    );
  });

  it('strip_markdown_noise_drops_headings_badges_and_code_blocks', () => {
    const markdown = '# Title\n\n[![ci](https://example.com/ci.svg)](https://example.com/ci)\n\nHive Mind is the AI that controls AIs.\n\n```bash\nnpm install hive-mind\n```\n\n> It orchestrates multiple agents.\n';
    const lower = stripMarkdownNoise(markdown).toLowerCase();
    assert.ok(lower.includes('hive mind'));
    assert.ok(lower.includes('orchestrates multiple agents'));
    assert.ok(!lower.includes('npm install'), lower);
    assert.ok(!lower.includes('[![ci]'), lower);
  });

  it('strip_markdown_noise_keeps_heading_text_when_no_marker', () => {
    const stripped = stripMarkdownNoise('## Section\n\nBody sentence one. Body sentence two.');
    assert.ok(stripped.includes('Section'));
    assert.ok(stripped.includes('Body sentence one'));
  });

  it('strip_markdown_noise_skips_html_comments', () => {
    const stripped = stripMarkdownNoise('<!-- internal note -->\n\nReal sentence.');
    assert.ok(!stripped.includes('internal note'));
    assert.ok(stripped.includes('Real sentence'));
  });

  it('strip_markdown_noise_keeps_the_rust_byte_offset_comment_quirk (source-derived)', () => {
    // Rust tests `text[out.len()..].starts_with("<!--")` at the byte length of
    // the output so far, so once one comment is removed a later multi-line
    // comment is no longer recognized and its tail line survives.
    assert.equal(stripMarkdownNoise('<!-- a -->\n<!-- b\nc -->\nTail'), '\n\nc -->\nTail\n');
  });

  it('formalize_markdown_classifies_install_and_purpose_lines', () => {
    const statements = formalizeMarkdown('Hive Mind is the AI that controls AIs.\n\nInstall with `cargo add hive-mind`.');
    assert.ok(statements.some((candidate) => candidate.kind === StatementKind.Purpose), JSON.stringify(statements));
    assert.ok(statements.some((candidate) => candidate.kind === StatementKind.Install), JSON.stringify(statements));
  });

  it('formalize_markdown_yields_statements_for_readme_prose', () => {
    const statements = formalizeMarkdown('# hive-mind\n\nHive Mind orchestrates multiple agents.\nInstall with npm install hive-mind.\n');
    assert.ok(statements.length > 0);
    assert.ok(statements.some((candidate) => candidate.text.toLowerCase().includes('orchestrates')));
    assert.ok(statements.some((candidate) => candidate.kind === StatementKind.Install));
  });

  it('describe_readme_short_keeps_purpose_drops_install', () => {
    const markdown = 'Formal AI is a deterministic symbolic engine.\n\nInstall with `cargo add formal-ai`.\n\nRun `formal-ai --help` for flags.';
    const summary = describeReadme('link-assistant/formal-ai', markdown, withLanguage(config(SummarizationMode.Short), 'en'));
    assert.equal(summary, 'Formal AI is a deterministic symbolic engine.');
  });

  it('describe_readme_strips_boilerplate_and_keeps_purpose', () => {
    const readme = '# command-stream\n\n![ci](https://example.com/ci.svg)\n\ncommand-stream is the streaming shell helper used by hive-mind.\nInstall with npm install command-stream.\n\n```bash\nnpm run example\n```\n';
    const description = describeReadme('link-foundation/command-stream', readme, config(SummarizationMode.Short));
    assert.ok(!description.toLowerCase().includes('npm install'), description);
    assert.ok(description.toLowerCase().includes('command-stream'));
  });

  it('describe_readme_topic_returns_repo_slug', () => {
    const topic = describeReadme('link-assistant/formal-ai', 'Formal AI is a deterministic symbolic engine.', withLanguage(config(SummarizationMode.Topic), 'en'));
    assert.equal(topic, 'link-assistant/formal-ai');
    const hive = describeReadme('link-assistant/hive-mind', '# Anything\n', config(SummarizationMode.Topic));
    assert.ok(words(hive).length <= 5);
    assert.ok(hive.includes('link-assistant/hive-mind') || hive.includes('hive-mind'));
  });

  it('describe_readme_identifier_returns_a_snake_case_name', () => {
    const name = describeReadme('link-assistant/formal-ai', 'Anything.', config(SummarizationMode.Identifier));
    assert.equal(isValidIdentifier(name, NamingConvention.SnakeCase), true, name);
    assert.equal(name, 'link_assistant_formal_ai');
  });
});

describe('dialog summarization and chat titles (dialog.rs)', () => {
  it('formalize_dialog_biases_user_turns_above_assistant_turns', () => {
    const statements = formalizeDialog([assistantTurn('Hi, how may I help you?'), userTurn('What is 2 + 2?')]);
    const user = statements.find((candidate) => candidate.text.includes('2 + 2'));
    const assistant = statements.find((candidate) => candidate.text.toLowerCase().includes('how may i help'));
    assert.ok(user.weight > assistant.weight, `user=${user.weight} assistant=${assistant.weight}`);
    assert.deepEqual([user.weight, assistant.weight], [50, 20]);
  });

  it('summarize_dialog_short_keeps_user_questions', () => {
    const turns = [
      userTurn('Tell me about Hive Mind.'),
      assistantTurn('Of course! Hive Mind orchestrates many subordinate agents and assigns goals.'),
    ];
    const summary = summarizeDialog(turns, withLanguage(config(SummarizationMode.Short), 'en'));
    assert.ok(summary.toLowerCase().includes('hive mind'), summary);
  });

  it('summarize_dialog_keeps_user_question_over_assistant_chatter', () => {
    const turns = [
      userTurn('What is Hive Mind?'),
      assistantTurn('Hive Mind is an AI orchestrator.'),
      userTurn('How do I install it?'),
    ];
    const summary = summarizeDialog(turns, withMaxStatements(config(SummarizationMode.Short), 2));
    assert.ok(summary.toLowerCase().includes('hive mind') || summary.toLowerCase().includes('install'), summary);
  });

  it('generate_chat_title_returns_five_or_fewer_words', () => {
    const title = generateChatTitle([
      userTurn('Tell me about Hive Mind.'),
      assistantTurn('Hive Mind orchestrates subordinate agents.'),
    ], 'en');
    assert.ok(title !== '');
    assert.ok(words(title).length <= 5, title);
    const single = generateChatTitle([userTurn('What is the Hive Mind project about?')], 'en');
    assert.ok(single !== '' && words(single).length <= 5, single);
  });

  it('plain_dialog_summary_removes_markdown_and_honors_budgets (issue_858)', () => {
    const turns = [
      userTurn('Implement the compact recap path in `src/anthropic.rs` and verify it with tests.'),
      assistantTurn('## Current status\n\n- The implementation is complete and the targeted regression passes.\n- A secondary cleanup can wait.'),
    ];
    const summary = summarizeDialogPlain(turns, 20, 2);
    assert.ok(words(summary).length <= 20, summary);
    assert.ok(!/[#`*]/u.test(summary), summary);
    assert.ok(summary.includes('compact recap'), summary);
    assert.ok(summary.includes('implementation is complete'), summary);
    assert.equal(summarizeDialogPlain(turns, 0, 2), '');
    assert.equal(summarizeDialogPlain([assistantTurn('No user here.')], 20, 2), '');
  });

  const CONVERSATION_FIXTURES = [
    {
      content: 'User asked about the weather. Assistant said it is sunny.',
      language: 'en',
      summary: 'User asked about the weather.',
      title: 'Assistant said it is sunny'
    },
    {
      content: 'Formal AI is a deterministic symbolic engine. It runs offline. Install with cargo add formal-ai. Run formal-ai --help for flags. It supports plugins and many helpful features. Written in Rust. It has 5000 github stars.',
      language: 'en',
      summary: 'It supports plugins and many helpful features. Formal AI is a deterministic symbolic engine. Written in Rust.',
      title: 'It supports plugins and many'
    },
    {
      content: 'The adapter supplies 19.5 V. The next U.S. general election is on Tuesday, November 3, 2026. Turnout was 63.7%.',
      language: 'en',
      summary: 'The adapter supplies 19.5 V. The next U.S. general election is on Tuesday, November 3, 2026.',
      title: 'Turnout was 63.7%'
    },
    {
      content: 'This project is a comprehensive toolkit that supports plugins and provides many helpful features for everyone. X is a tool. Use it when you deploy to production. For example, see the snippet below.',
      language: 'en',
      summary: 'This project is a comprehensive toolkit that supports plugins and provides many helpful features for everyone. Use it when you deploy to production.',
      title: 'This project is a comprehensive'
    },
    {
      content: 'Ассистент сказал, что солнечно. Это установка через npm install. Это инструмент для того чтобы помогать.',
      language: 'ru',
      summary: 'Ассистент сказал, что солнечно.',
      title: 'Это инструмент для того чтобы'
    },
    {
      content: 'बातचीत सारांश: उपयोगकर्ता ने मौसम के बारे में पूछा। सहायक ने कहा कि धूप है।',
      language: 'en',
      summary: 'बातचीत सारांश: उपयोगकर्ता ने मौसम के बारे में पूछा।',
      title: 'सहायक ने कहा कि धूप'
    },
    {
      content: '总结对话: 用户询问天气。助手说天气晴朗。我们需要安装。',
      language: 'zh',
      summary: '总结对话: 用户询问天气。',
      title: '助手说天气晴朗。'
    },
    {
      content: 'Hive Mind orchestrates many agents! Just a plain sentence about nothing. Another plain sentence. And one more thing is here.',
      language: 'en',
      summary: 'Hive Mind orchestrates many agents! Just a plain sentence about nothing.',
      title: 'Hive Mind orchestrates many agents'
    },
    {
      content: 'To install, run the setup script. It helps you ship faster. It supports plugins. Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron. Short one.',
      language: 'en',
      summary: 'It helps you ship faster. It supports plugins.',
      title: 'It helps you ship faster'
    },
    {
      content: 'The tool is a CLI for the user interface. Written in Rust, version 1.2.3. Done...',
      language: 'en',
      summary: 'The tool is a CLI for the user interface. Written in Rust, version 1.2.3. Done.',
      title: 'Written in Rust, version 1.2.3'
    },
    {
      content: 'Tell me about Hive Mind. Of course! Hive Mind orchestrates many subordinate agents and assigns goals. It is useful when you manage a fleet. Because it is deterministic.',
      language: 'en',
      summary: 'Hive Mind orchestrates many subordinate agents and assigns goals. It is useful when you manage a fleet. Tell me about Hive Mind.',
      title: 'Hive Mind orchestrates many subordinate'
    },
    {
      content: '   Leading spaces and a very very long first sentence that goes on and on and on and on and on and on forever. Second one.',
      language: 'en',
      summary: 'Leading spaces and a very very long first sentence that goes on and on and on and on and on and on forever.',
      title: 'Second one'
    },
    {
      content: 'Plain one. Plain two. Plain three. Plain four. Plain five. Plain six.',
      language: 'en',
      summary: 'Plain one. Plain two. Plain three.',
      title: 'Plain six'
    },
    {
      content: 'Alpha. Install with npm install x. Beta feature supports things. Gamma.',
      language: 'en',
      summary: 'Beta feature supports things. Alpha.',
      title: 'Beta feature supports things'
    }
  ];

  for (const [index, fixture] of CONVERSATION_FIXTURES.entries()) {
    it(`matches the Rust binary: conversation summary and title, fixture ${index} (${fixture.language})`, () => {
      const turns = [userTurn(fixture.content.trim())];
      const standard = withLanguage(config(SummarizationMode.Standard), fixture.language);
      assert.equal(summarizeDialog(turns, standard), fixture.summary);
      assert.equal(generateChatTitle(turns, fixture.language), fixture.title);
    });
  }
});

describe('R345-R348, R350: repository-file formalization (file.rs)', () => {
  const embeddedMarkdown = '# Summarization\n\nFormal AI summarizes repository files.\n\n```rust\npub fn summarize_file() -> &\'static str { "ok" }\n```\n\n```javascript\nexport function renderSummary() { return \'ok\'; }\n```\n';

  it('repository_file_summary_recurses_into_markdown_embedded_grammars (R345, R348, R349)', () => {
    const summary = summarizeRepositoryFile('docs/example.md', embeddedMarkdown, withLanguage(config(SummarizationMode.Standard), 'en'));
    assert.equal(
      summary,
      `docs/example.md is a Markdown file with ${embeddedMarkdown.split('\n').length - 1} lines and ${Buffer.byteLength(embeddedMarkdown)} bytes. It has embedded grammar blocks: rust, javascript. Key content: Formal AI summarizes repository files.`,
    );
  });

  it('the_sampled_repository_files_are_formalized_by_format_not_by_name (R346, R347)', () => {
    const identityPath = 'data/cache/wordnet/en/identity.json';
    const identity = formalizeRepositoryFile(identityPath, read(identityPath));
    assert.deepEqual([identity.format, identity.line_count, identity.byte_count], ['json', 57, 1524]);
    assert.deepEqual(texts(identity.statements), [
      'data/cache/wordnet/en/identity.json is a JSON data file',
      'Top-level keys: lemma, language, source, license, name, url, senses, id.',
    ]);

    const issuePath = 'docs/case-studies/issue-140/raw-data/issue-140.json';
    const issue = formalizeRepositoryFile(issuePath, read(issuePath));
    assert.deepEqual([issue.format, issue.line_count, issue.byte_count], ['json', 1, 5322]);
    assert.deepEqual(texts(issue.statements), ['docs/case-studies/issue-140/raw-data/issue-140.json is a JSON data file']);

    const note = formalizeRepositoryFile('notes/history.unknown', 'Formal AI keeps an add-only history.\n');
    assert.equal(note.format, 'text');
    assert.deepEqual(texts(note.statements), ['Formal AI keeps an add-only history.']);
  });

  it('formalize_repository_file_markdown_records_embedded_grammars (R347, R348, R350)', () => {
    const markdown = '# File summary\n\nFormal AI is designed to summarize repository files.\n\n```rust\npub fn summarize_file() -> &\'static str { "ok" }\n```\n\n```javascript\nexport function renderSummary() { return \'ok\'; }\n```\n';
    const formalized = formalizeRepositoryFile('docs/file-summary.md', markdown);
    assert.equal(formalized.format, 'markdown');
    assert.equal(formalized.embedded_grammars.length, 2);
    assert.equal(formalized.embedded_grammars[0].language, 'rust');
    assert.equal(formalized.embedded_grammars[1].language, 'javascript');
    const lino = fileLinksNotation(formalized);
    assert.ok(lino.includes('repository_file'));
    assert.ok(lino.includes('embedded_grammar'));
    assert.ok(lino.includes('language rust'));
    assert.ok(lino.includes('language javascript'));
    assert.equal(
      lino,
      [
        'repository_file',
        '  path docs/file-summary.md',
        '  format markdown',
        `  line_count ${markdown.split('\n').length - 1}`,
        `  byte_count ${Buffer.byteLength(markdown)}`,
        '  statement_count 2',
        '  statement',
        '    kind misc',
        '    weight 15',
        '    text File summary',
        '  statement',
        '    kind purpose',
        '    weight 100',
        '    text Formal AI is designed to summarize repository files.',
        '  embedded_grammar',
        '    language rust',
        '    line_count 1',
        '    statement_count 1',
        '  embedded_grammar',
        '    language javascript',
        '    line_count 1',
        '    statement_count 1',
      ].join('\n'),
    );
  });

  it('formalize_repository_file_markdown_closes_embedded_grammar_at_eof (R348)', () => {
    const formalized = formalizeRepositoryFile('docs/file-summary.md', '# File summary\n\n```rust\npub struct FileSummary;\n');
    assert.equal(formalized.embedded_grammars.length, 1);
    assert.equal(formalized.embedded_grammars[0].language, 'rust');
  });

  it('formalize_repository_file_rust_records_symbols_and_the_meta_language_slot (R347)', () => {
    const source = 'pub struct FileSummary;\n\npub fn summarize_file() -> &\'static str {\n"ok"\n}\n';
    const formalized = formalizeRepositoryFile('src/file_summary.rs', source);
    assert.equal(formalized.format, 'rust');
    assert.ok(formalized.statements.some((candidate) => candidate.text.includes('rust struct FileSummary')));
    assert.ok(formalized.statements.some((candidate) => candidate.text.includes('rust function summarize_file')));
    // No parser is installed: the featureless Rust build records no evidence.
    assert.equal(formalized.meta_language, null);
  });

  it('an_installed_meta_language_parser_supplies_bounded_parse_evidence (R347)', () => {
    const calls = [];
    installMetaLanguageParser((label, source) => {
      calls.push([label, source.length]);
      return { label, syntax_link_count: 7, total_link_count: 11, has_error: false, text_preserved: true };
    });
    try {
      const formalized = formalizeRepositoryFile('src/lib.rs', 'pub fn x() {}\n');
      assert.deepEqual(formalized.meta_language, { label: 'rust', syntax_link_count: 7, total_link_count: 11, has_error: false, text_preserved: true });
      assert.equal(
        summarizeRepositoryFile('src/lib.rs', 'pub fn x() {}\n', config(SummarizationMode.Full)),
        'src/lib.rs is a Rust file with 1 lines and 14 bytes. meta-language parsed it as rust with 7 syntax links. Key content: src/lib.rs is a rust source file. Defines rust function x.',
      );
      assert.ok(fileLinksNotation(formalized).includes('  meta_language\n    label rust\n    syntax_link_count 7\n    total_link_count 11\n    has_error false\n    text_preserved true'));
      // Markdown parses each fenced block under its own label; data formats too.
      const markdown = formalizeRepositoryFile('docs/loader.md', '# L\n\n```rust\nfn load() {}\n```\n\n~~~json\n{"ok": true}\n~~~\n');
      assert.deepEqual(markdown.embedded_grammars.map((block) => block.meta_language?.label ?? null), ['rust', 'json']);
      // A parse that errors is not reported as evidence in the prose summary.
      installMetaLanguageParser((label) => ({ label, syntax_link_count: 3, total_link_count: 5, has_error: true, text_preserved: true }));
      assert.ok(!summarizeRepositoryFile('src/lib.rs', 'pub fn x() {}\n', config(SummarizationMode.Full)).includes('meta-language parsed'));
      // issue_893: oversized structured evidence skips the parse entirely.
      calls.length = 0;
      installMetaLanguageParser((label, source) => {
        calls.push([label, source.length]);
        return { label, syntax_link_count: 1, total_link_count: 1, has_error: false, text_preserved: true };
      });
      const oversized = formalizeRepositoryFile('evidence.jsonl', `{"payload":"${'x'.repeat(32 * 1024)}"}`);
      assert.equal(oversized.meta_language, null);
      assert.deepEqual(calls, []);
      // ...and only jsonl: a `.json` file of the same size is also over the bound.
      assert.equal(formalizeRepositoryFile('evidence.json', `{"payload":"${'x'.repeat(32 * 1024)}"}`).meta_language, null);
      assert.ok(formalizeRepositoryFile('evidence.json', '{"payload":"x"}').meta_language !== null);
    } finally {
      installMetaLanguageParser(null);
    }
  });

  it('issue_893_oversized_structured_files_skip_the_unbounded_meta_language_parse', () => {
    const content = `{"payload":"${'x'.repeat(32 * 1024)}"}`;
    const formalized = formalizeRepositoryFile('evidence.jsonl', content);
    assert.equal(formalized.meta_language, null);
    assert.equal(formalized.byte_count, Buffer.byteLength(content));
    assert.ok(formalized.statements.length > 0);
  });

  it('issue_893_oversized_plain_text_keeps_bounded_head_and_tail_evidence', () => {
    const content = `head evidence\n${'a🙂\n'.repeat(12 * 1024)}tail evidence\n`;
    const formalized = formalizeRepositoryFile('evidence.log', content);
    assert.equal(formalized.byte_count, Buffer.byteLength(content));
    assert.equal(formalized.line_count, content.split('\n').length - 1);
    assert.ok(formalized.statements.length <= 256, String(formalized.statements.length));
    assert.ok(formalized.statements.some((candidate) => candidate.text === 'head evidence'));
    assert.ok(formalized.statements.some((candidate) => candidate.text === 'tail evidence'));
  });

  it('formats_are_detected_by_file_name_then_extension (source-derived)', () => {
    const formatOf = (path) => formalizeRepositoryFile(path, 'x\n').format;
    assert.deepEqual(
      ['Cargo.TOML', 'pkg/package.json', 'a/Dockerfile', 'a.RS', 'a.mjs', 'x.tsx', 'a.cc', 'a.h', 'a.svg', 'a.lino', 'a.sh', '.bashrc', 'README', 'a.', 'dir.d/readme', 'notes.unknown'].map(formatOf),
      ['toml', 'json', 'dockerfile', 'rust', 'javascript', 'typescript', 'cpp', 'c', 'xml', 'links_notation', 'shell', 'text', 'text', 'text', 'text', 'text'],
    );
  });

  it('empty_files_get_an_identity_statement_and_zero_lines', () => {
    const empty = formalizeRepositoryFile('docs/empty.md', '');
    assert.deepEqual([empty.line_count, empty.byte_count], [0, 0]);
    assert.deepEqual(empty.statements, [{ text: 'docs/empty.md is an empty Markdown file', kind: StatementKind.Identity, weight: 90 }]);
  });

  it('code_symbols_are_extracted_per_language_and_capped_at_eight (source-derived)', () => {
    const rust = 'pub(crate) fn hidden() {}\npub async fn run() {}\n// pub fn commented() {}\nstruct S;\nimpl S {}\nconst MAX: u32 = 1;\nmod m;\ntype T = u8;\nstatic G: u8 = 0;\ntrait Tr {}\nenum E {}\npub fn run() {}\n';
    assert.deepEqual(
      texts(formalizeRepositoryFile('a.rs', rust).statements),
      [
        'a.rs is a rust source file',
        'Defines rust function run.',
        'Defines rust struct S.',
        'Defines rust impl S.',
        'Defines rust module m.',
        'Defines rust type T.',
        'Defines rust static G.',
        'Defines rust trait Tr.',
        'Defines rust enum E.',
      ],
    );
    assert.deepEqual(
      texts(formalizeRepositoryFile('a.mjs', 'export default async function main() {}\nexport class K {}\nconst a = 1;\nlet b;\n').statements),
      ['a.mjs is a javascript source file', 'Defines javascript function main.', 'Defines javascript class K.', 'Defines javascript binding a.', 'Defines javascript binding b.'],
    );
    assert.deepEqual(
      texts(formalizeRepositoryFile('a.py', 'def f():\n  pass\nclass C:\n  pass\n').statements),
      ['a.py is a python source file', 'Defines python symbol f.', 'Defines python symbol C.'],
    );
    assert.deepEqual(
      texts(formalizeRepositoryFile('a.go', 'func main() {}\n').statements),
      ['a.go is a go source file', 'Defines go function main.'],
    );
    assert.deepEqual(
      texts(formalizeRepositoryFile('A.java', 'public static class A {}\ninterface I {}\nenum E {}\n').statements),
      ['A.java is a java source file', 'Defines class A.', 'Defines interface I.', 'Defines enum E.'],
    );
    assert.deepEqual(
      texts(formalizeRepositoryFile('a.c', 'int main(void) {\nint x;\nprintf("a");\nstatic int helper(int a)\n').statements),
      ['a.c is a c source file', 'Defines function main.', 'Defines function helper.'],
    );
  });

  it('structured_files_report_up_to_eight_top_level_keys_in_order (source-derived)', () => {
    assert.deepEqual(
      texts(formalizeRepositoryFile('a.toml', '[package]\nname = "x"\nversion = "1"\nname = "dup"\n"quoted" : 1\n').statements),
      ['a.toml is a TOML data file', 'Top-level keys: name, version, quoted.'],
    );
    const many = Array.from({ length: 12 }, (_, index) => `k${index}: v`).join('\n');
    assert.deepEqual(
      texts(formalizeRepositoryFile('a.yaml', many).statements),
      ['a.yaml is a YAML data file', 'Top-level keys: k0, k1, k2, k3, k4, k5, k6, k7.'],
    );
  });

  it('markdown_heading_fragments_lose_fifteen_weight_points', () => {
    const formalized = formalizeRepositoryFile('a.md', '# Title\n\nFormal AI summarizes repository files.\n');
    assert.deepEqual(formalized.statements.map((candidate) => [candidate.text, candidate.weight]), [['Title', 15], ['Formal AI summarizes repository files.', 30]]);
  });
});

describe('R355-R359: resource and folder summarization (resource.rs)', () => {
  const sampleTree = () => repositoryDirectory('src/summarization', [
    repositoryFile('src/summarization/mod.rs', '//! Summarization pipeline.\npub fn summarize() {}\n'),
    repositoryFile('src/summarization/file.rs', '//! File summary.\npub struct RepositoryFileFormalization;\npub fn formalize_repository_file() {}\n'),
    repositoryDirectory('src/summarization/nested', [
      repositoryFile('src/summarization/nested/readme.md', '# Nested\n\nThis explains nested things.\n\n```rust\npub fn x() {}\n```\n'),
    ]),
  ]);

  it('summarize_repository_resource_subsumes_file_summarization (R349, R355)', () => {
    const markdown = '# Summarization\n\nFormal AI summarizes repository files.\n';
    const standard = withLanguage(config(SummarizationMode.Standard), 'en');
    const viaFile = summarizeRepositoryFile('docs/example.md', markdown, standard);
    const viaResource = summarizeRepositoryResource(repositoryFile('docs/example.md', markdown), standard);
    assert.equal(viaFile, viaResource);
    assert.equal(viaFile, `docs/example.md is a Markdown file with 3 lines and ${Buffer.byteLength(markdown)} bytes. Key content: Formal AI summarizes repository files.`);
  });

  it('directory_topic_summary_is_a_single_identity_sentence (R357, R359)', () => {
    const summary = summarizeRepositoryResource(sampleTree(), withLanguage(config(SummarizationMode.Topic), 'en'));
    assert.equal(summary, 'src/summarization is a repository directory with 2 files and 1 subdirectory (12 lines total across 3 files).');
    assert.ok(!summary.includes('Contents:'));
  });

  it('directory_summary_reports_recursive_aggregate_counts (R356)', () => {
    const summary = summarizeRepositoryResource(sampleTree(), config(SummarizationMode.Topic));
    assert.ok(summary.includes('12 lines total across 3 files'), summary);
  });

  it('directory_identifier_summary_is_a_snake_case_name (R357, label_for_mode)', () => {
    const summary = summarizeRepositoryResource(sampleTree(), config(SummarizationMode.Identifier));
    assert.equal(summary, labelForMode(SummarizationMode.Identifier, 'src/summarization is a repository directory with 2 files and 1 subdirectory (12 lines total across 3 files).'));
    assert.equal(isValidIdentifier(summary, NamingConvention.SnakeCase), true, summary);
    assert.equal(summary, 'src_summarization_repository_directory');
  });

  it('directory_short_summary_bounds_listed_children (R357)', () => {
    const summary = summarizeRepositoryResource(sampleTree(), withLanguage(config(SummarizationMode.Short), 'en'));
    assert.ok(summary.includes('Contents:'), summary);
    assert.ok(summary.includes('1 more entry omitted for brevity.'), summary);
    assert.ok(summary.endsWith('1 more entry omitted for brevity.'), summary);
  });

  it('directory_summary_recurses_with_one_step_shorter_mode (R357)', () => {
    const summary = summarizeRepositoryResource(sampleTree(), withLanguage(config(SummarizationMode.Full), 'en'));
    assert.ok(summary.includes('src/summarization/nested is a repository directory'), summary);
    assert.ok(summary.includes('src/summarization/nested/readme.md is a Markdown file'), summary);
  });

  it('summarize_repository_resource_full_directory_recurses_into_nested_folder (R359)', () => {
    const tree = repositoryDirectory('src/summarization', [
      repositoryFile('src/summarization/mod.rs', '//! Summarization pipeline.\npub fn summarize() {}\n'),
      repositoryFile('src/summarization/file.rs', '//! File summary.\npub struct RepositoryFileFormalization;\n'),
      repositoryDirectory('src/summarization/nested', [
        repositoryFile('src/summarization/nested/readme.md', '# Nested\n\nThis explains nested things.\n'),
      ]),
    ]);
    const summary = summarizeRepositoryResource(tree, config(SummarizationMode.Full));
    assert.ok(summary.includes('Contents:'));
    assert.ok(summary.includes('src/summarization/nested is a repository directory'));
    assert.ok(summary.includes('src/summarization/nested/readme.md is a Markdown file'));
    assert.ok(summary.includes('is a repository directory with 2 files and 1 subdirectory'));
  });

  it('formalize_repository_resource_distinguishes_files_and_directories (R356)', () => {
    const formal = formalizeRepositoryResource(sampleTree());
    assert.equal(formal.kind, 'directory');
    const directory = formal.directory;
    assert.equal(directory.direct_file_count, 2);
    assert.equal(directory.direct_directory_count, 1);
    assert.equal(directory.total_file_count, 3);
    assert.equal(directory.total_directory_count, 1);
    assert.ok(directory.children.some(resourceIsDirectory));
  });

  it('formalize_repository_directory_aggregates_counts_recursively (R356)', () => {
    const tree = sampleTree();
    const formal = formalizeRepositoryDirectory('src/summarization', tree.children);
    assert.equal(formal.direct_file_count, 2);
    assert.equal(formal.direct_directory_count, 1);
    assert.equal(formal.total_file_count, 3);
    assert.equal(formal.total_directory_count, 1);
    assert.ok(formal.total_line_count > 0);
    assert.ok(formal.total_byte_count > 0);
    assert.equal(formal.total_byte_count, 50 + 96 + 66 + 0 + 0 + 0);
  });

  it('formalize_repository_resource_dispatches_on_entry_kind', () => {
    const file = formalizeRepositoryResource(repositoryFile('a.rs', 'fn x() {}\n'));
    assert.equal(resourceIsDirectory(file), false);
    assert.equal(resourcePath(file), 'a.rs');
    const dir = formalizeRepositoryResource(sampleTree());
    assert.equal(resourceIsDirectory(dir), true);
    assert.equal(resourcePath(dir), 'src/summarization');
  });

  it('directory_links_notation_lists_children_by_kind (R350, R358)', () => {
    const lino = resourceLinksNotation(formalizeRepositoryResource(sampleTree()));
    assert.ok(lino.startsWith('repository_directory'));
    assert.ok(lino.includes('direct_file_count 2'));
    assert.ok(lino.includes('file src/summarization/mod.rs'));
    assert.ok(lino.includes('directory src/summarization/nested'));
    assert.equal(
      lino,
      [
        'repository_directory',
        '  path src/summarization',
        '  direct_file_count 2',
        '  direct_directory_count 1',
        '  total_file_count 3',
        '  total_directory_count 1',
        '  total_line_count 12',
        '  total_byte_count 212',
        '  file src/summarization/mod.rs',
        '  file src/summarization/file.rs',
        '  directory src/summarization/nested',
      ].join('\n'),
    );
  });

  it('a_file_resource_renders_the_repository_file_block (R358)', () => {
    const lino = resourceLinksNotation(formalizeRepositoryResource(repositoryFile('a.json', '{\n  "k": 1\n}\n')));
    assert.equal(
      lino,
      'repository_file\n  path a.json\n  format json\n  line_count 3\n  byte_count 13\n  statement_count 2\n  statement\n    kind identity\n    weight 90\n    text a.json is a JSON data file\n  statement\n    kind feature\n    weight 70\n    text Top-level keys: k.',
    );
  });

  it('folder_summary_example_matches_the_example_tree_per_mode (R359, issue_563_folder_summary.rs)', () => {
    const tree = repositoryDirectory('src/summarization', [
      repositoryFile('src/summarization/mod.rs', '//! Summarization pipeline.\npub fn summarize() {}\n'),
      repositoryFile('src/summarization/file.rs', '//! File summary.\npub struct RepositoryFileFormalization;\npub fn formalize_repository_file() {}\n'),
      repositoryDirectory('src/summarization/nested', [
        repositoryFile('src/summarization/nested/readme.md', '# Nested\n\nThis explains nested things.\n\n```rust\npub fn x() {}\n```\n'),
      ]),
    ]);
    const identity = 'src/summarization is a repository directory with 2 files and 1 subdirectory (12 lines total across 3 files).';
    assert.equal(summarizeRepositoryResource(tree, config(SummarizationMode.Topic)), identity);
    // Each deeper rung lists more children, one mode shorter than its parent.
    const short = summarizeRepositoryResource(tree, config(SummarizationMode.Short));
    assert.ok(short.startsWith(`${identity} Contents: `));
    assert.ok(short.endsWith(' 1 more entry omitted for brevity.'));
    const standard = summarizeRepositoryResource(tree, config(SummarizationMode.Standard));
    assert.ok(standard.includes('src/summarization/nested is a repository directory with 1 file and 0 subdirectories (7 lines total across 1 file). Contents: '));
    assert.ok(!standard.includes('omitted for brevity'));
    const full = summarizeRepositoryResource(tree, config(SummarizationMode.Full));
    assert.ok(full.includes('Defines rust struct RepositoryFileFormalization.'));
    assert.ok(full.includes('It has embedded grammar blocks: rust.'));
  });
});

describe('the identifier rung (identifier.rs, vocabulary.rs; issue_844)', () => {
  it('the_identifier_rung_produces_valid_identifiers_under_a_length_budget', () => {
    const budget = identifierBudget();
    const phrases = [
      'The parser is fast',
      'A deterministic summarizer merges statements from many sources',
      '3 ways to install it',
      '解析器很快',
      "don't repeat yourself",
    ];
    for (const phrase of phrases) {
      for (const convention of [NamingConvention.SnakeCase, NamingConvention.ScreamingSnakeCase, NamingConvention.CamelCase, NamingConvention.PascalCase]) {
        const identifier = toIdentifier(phrase, convention, budget);
        assert.equal(isValidIdentifier(identifier, convention), true, `${phrase} as ${convention} gave ${identifier}`);
        assert.ok(Array.from(identifier).length <= budget.max_length, identifier);
        assert.ok(identifier.split('_').filter(Boolean).length <= budget.max_words, identifier);
      }
    }
    assert.equal(toIdentifier('the type', NamingConvention.SnakeCase, budget), 'type_');
    assert.equal(isValidIdentifier('type', NamingConvention.SnakeCase), false);
    assert.equal(toIdentifier('the type of a match', NamingConvention.SnakeCase, budget), 'type_match');
    const tight = identifierBudget(12, 4);
    const cut = toIdentifier('deterministic summarizer merges statements', NamingConvention.SnakeCase, tight);
    assert.ok(Array.from(cut).length <= 12, cut);
    assert.equal(isValidIdentifier(cut, NamingConvention.SnakeCase), true, cut);
    const subject = toIdentifier('the summarizer merges statements from many sources into one context', NamingConvention.CommitSubject, commitSubjectBudget());
    assert.equal(isValidIdentifier(subject, NamingConvention.CommitSubject), true, subject);
    assert.ok(Array.from(subject).length <= 50, subject);
    assert.ok(/^\p{Uppercase}/u.test(subject), subject);
  });
});

describe('R347: tree-sitter backed meta-language evidence (summarization_meta_language.mjs)', () => {
  let parser;

  before(async () => {
    parser = await loadMetaLanguageParser(webTreeSitter, {
      rust: fileURLToPath(new URL('js/vendor/tree-sitter/tree-sitter-rust.wasm', root)),
    });
    installMetaLanguageParser(parser);
  });

  after(() => installMetaLanguageParser(null));

  it('formalize_repository_file_rust_records_meta_language_and_symbols', () => {
    const source = 'pub struct FileSummary;\n\npub fn summarize_file() -> &\'static str {\n"ok"\n}\n';
    const formalized = formalizeRepositoryFile('src/file_summary.rs', source);
    assert.equal(formalized.format, 'rust');
    assert.ok(formalized.statements.some((candidate) => candidate.text.includes('rust struct FileSummary')));
    assert.ok(formalized.statements.some((candidate) => candidate.text.includes('rust function summarize_file')));
    const meta = formalized.meta_language;
    assert.notEqual(meta, null, 'Rust files should be parsed through the meta-language parser');
    assert.equal(meta.label, 'rust');
    assert.ok(meta.syntax_link_count > 0);
    assert.ok(meta.text_preserved);
    assert.equal(meta.has_error, false);
  });

  it('a_parsed_rust_file_reports_the_evidence_sentence_and_block (R347, R350)', () => {
    const source = 'pub fn x() {}\n';
    const summary = summarizeRepositoryFile('src/lib.rs', source, config(SummarizationMode.Full));
    assert.match(summary, /^src\/lib\.rs is a Rust file with 1 lines and 14 bytes\. meta-language parsed it as rust with \d+ syntax links\. Key content: /u);
    const lino = fileLinksNotation(formalizeRepositoryFile('src/lib.rs', source));
    assert.match(lino, /\n {2}meta_language\n {4}label rust\n {4}syntax_link_count \d+\n {4}total_link_count \d+\n {4}has_error false\n {4}text_preserved true\n/u);
  });

  it('markdown_embedded_rust_blocks_carry_their_own_evidence (R348)', () => {
    const formalized = formalizeRepositoryFile('docs/loader.md', '# L\n\n```rust\nfn load() {}\n```\n\n```go\nfunc main() {}\n```\n');
    const [rust, go] = formalized.embedded_grammars;
    assert.equal(rust.meta_language.label, 'rust');
    assert.ok(rust.meta_language.syntax_link_count > 0);
    // Only the loaded grammar produces evidence; see the module header for the gap.
    assert.equal(go.meta_language, null);
  });

  it('a_broken_rust_parse_is_not_reported_as_valid_evidence (source-derived)', () => {
    const meta = parser('rust', 'fn (');
    assert.equal(meta.has_error, true);
    const summary = summarizeRepositoryFile('broken.rs', 'fn (', config(SummarizationMode.Full));
    assert.ok(!summary.includes('meta-language parsed'), summary);
  });

  it('leading_blank_lines_are_not_covered_by_any_token (source-derived replay of reconstruct_text)', () => {
    // meta-language starts the root's gap tokens at the root's own start, so
    // text before the first node is lost from the reconstruction.
    assert.equal(parser('rust', 'fn x() {}').text_preserved, true);
    assert.equal(parser('rust', '\n\nfn x() {}\n').text_preserved, false);
  });
});
