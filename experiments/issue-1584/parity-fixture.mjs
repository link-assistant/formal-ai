// Cross-runtime contract for the generalized CNL engine (issue #1584).
// JavaScript is the reference: this enumerates every language, meaning graph,
// profile and density, digests the projected text and validation report per
// (language, profile), and lists every controlled-dialogue turn verbatim. The
// Rust port (rust/src/cnl_output/) must reproduce the same digests and turns:
// rust/tests/unit/cnl_parity.rs and rust/tests/web/cnl-parity.test.mjs.
// Usage: node experiments/issue-1584/parity-fixture.mjs [--write | --rows <language> <profile>]
import {createHash} from 'node:crypto';
import {readFileSync, writeFileSync} from 'node:fs';
import {DENSITIES, dialogueGraphs, factGraphs, policy} from '../../rust/tests/web/support/cnl-matrix.mjs';
import {controlledDialogue, projectOutput, validateOutput} from '../../js/agentic/crate/cnl_output.mjs';
import {graphKey, realizeGraph} from '../../js/agentic/crate/cnl_grammar.mjs';

export const PARITY_PATH = 'data/parity/issue-1584-cnl-parity.json';

/** One line per graph and density; the digest input both runtimes build identically. */
export function projectionRows(language, profile) {
  const rows = [];
  for (const graph of [...factGraphs(), ...dialogueGraphs()]) {
    for (const density of DENSITIES) {
      const configuration = {output_standard: profile, lexical_density: density};
      const text = projectOutput({language, source: '', parts: [{kind: 'graph', graph}]}, configuration, policy);
      const report = validateOutput(text, language, configuration, policy);
      rows.push([graphKey(graph), density, text, report.compliant, report.violations.join(','), report.unchecked.join(',')].join('\t'));
    }
  }
  return rows;
}

const digest = rows => createHash('sha256').update(rows.join('\n') + '\n').digest('hex');

/** Every question phrasing in every style, a numbered choice after each clarification, and every configuration. */
export function dialogueCases(language) {
  const grammar = policy.grammars[language];
  const cases = [];
  const prompts = new Set();
  for (const graph of factGraphs().filter(item => item.mood === 'question')) {
    for (const style of grammar.outputStyles) {
      const prompt = realizeGraph(graph, style, 'precise', grammar);
      if (prompt !== null) prompts.add(prompt);
    }
  }
  for (const template of grammar.templates.filter(item => item.styles.includes('task'))) {
    for (const [index, variant] of template.positive.entries()) {
      prompts.add(variant.replace(/\{left\.number\}/gu, String(7 + index)).replace(/\{right\.number\}/gu, '3'));
    }
  }
  for (const prompt of prompts) {
    const turn = controlledDialogue(prompt, [], {}, policy);
    cases.push({prompt, history: [], configuration: {}});
    if (turn?.intent === 'cnl_clarification') {
      cases.push({prompt: '1', history: [{role: 'user', content: prompt}, {role: 'assistant', content: turn.content}], configuration: {}});
    }
  }
  const sample = [...prompts][0];
  for (const profile of policy.profiles) {
    for (const density of DENSITIES) cases.push({prompt: sample, history: [], configuration: {output_standard: profile.id, lexical_density: density}});
  }
  return cases.map(item => {
    const turn = controlledDialogue(item.prompt, item.history, item.configuration, policy);
    return {...item, expected: turn === null ? null : {intent: turn.intent, language: turn.language,
      ...(turn.intent === 'cnl_task' ? {prompt: turn.prompt}
        : {content: turn.content, compliant: turn.outputProjection.report.compliant})}};
  });
}

export function parityFixture() {
  const languages = Object.keys(policy.grammars).sort();
  const projections = {};
  for (const language of languages) {
    for (const profile of policy.profiles) projections[`${language} ${profile.id}`] = digest(projectionRows(language, profile.id));
  }
  return {languages, densities: DENSITIES, projections,
    dialogue: Object.fromEntries(languages.map(language => [language, dialogueCases(language)]))};
}

// One case per line keeps the fixture reviewable and under the text size limit.
export function formatFixture(fixture) {
  const lines = ['{', `  "languages": ${JSON.stringify(fixture.languages)},`, `  "densities": ${JSON.stringify(fixture.densities)},`, '  "projections": {'];
  const entries = Object.entries(fixture.projections);
  entries.forEach(([key, value], index) => lines.push(`    ${JSON.stringify(key)}: ${JSON.stringify(value)}${index < entries.length - 1 ? ',' : ''}`));
  lines.push('  },', '  "dialogue": {');
  const languages = Object.entries(fixture.dialogue);
  languages.forEach(([language, cases], index) => {
    lines.push(`    ${JSON.stringify(language)}: [`);
    cases.forEach((item, at) => lines.push(`      ${JSON.stringify(item)}${at < cases.length - 1 ? ',' : ''}`));
    lines.push(`    ]${index < languages.length - 1 ? ',' : ''}`);
  });
  lines.push('  }', '}');
  return lines.join('\n') + '\n';
}

if (import.meta.url === `file://${process.argv[1]}` && process.argv[2] === '--rows') {
  // Debug a digest mismatch: --rows <language> <profile> prints the digested rows.
  console.log(projectionRows(process.argv[3], process.argv[4]).join('\n'));
} else if (import.meta.url === `file://${process.argv[1]}`) {
  const text = formatFixture(parityFixture());
  if (process.argv.includes('--write')) {
    writeFileSync(PARITY_PATH, text);
    console.log(`wrote ${PARITY_PATH} (${text.split('\n').length - 1} lines)`);
  } else {
    const current = (() => { try { return readFileSync(PARITY_PATH, 'utf8'); } catch { return ''; } })();
    console.log(current === text ? 'parity fixture is current' : 'parity fixture is stale: run with --write');
    if (current !== text) process.exitCode = 1;
  }
}
