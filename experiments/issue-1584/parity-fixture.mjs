// Cross-runtime contract for the generalized CNL engine (issue #1584).
// JavaScript is the reference: this enumerates every language, meaning graph,
// profile and density, digests the projected text and validation report per
// (language, profile), and lists every controlled-dialogue turn verbatim. The
// Rust port (rust/src/cnl_output/) must reproduce the same digests and turns:
// rust/tests/unit/cnl_parity.rs and rust/tests/web/cnl-parity.test.mjs.
// One file per language keeps each fixture reviewable and under the text size limit.
// Usage: node experiments/issue-1584/parity-fixture.mjs [--write | --rows <language> <profile>]
import {createHash} from 'node:crypto';
import {mkdirSync, readFileSync, readdirSync, writeFileSync} from 'node:fs';
import {DENSITIES, dialogueGraphs, factGraphs, policy} from '../../rust/tests/web/support/cnl-matrix.mjs';
import {controlledDialogue, projectOutput, validateOutput} from '../../js/agentic/crate/cnl_output.mjs';
import {graphKey, realizeGraph} from '../../js/agentic/crate/cnl_grammar.mjs';

export const PARITY_DIR = 'data/parity/issue-1584-cnl';
export const parityPath = language => `${PARITY_DIR}/${language}.json`;
/** The languages with a committed fixture. */
export const fixtureLanguages = () => readdirSync(PARITY_DIR).filter(name => name.endsWith('.json')).map(name => name.slice(0, -5)).sort();

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

/**
 * Every question phrasing in every style, with precise terms, with the shared
 * surface forms and with input-only surfaces (both possibly ambiguous), a numbered choice after each
 * clarification, every task paraphrase, and every configuration.
 */
export function dialogueCases(language) {
  const grammar = policy.grammars[language];
  const cases = [];
  const prompts = new Set();
  for (const graph of factGraphs().filter(item => item.mood === 'question')) {
    for (const style of grammar.outputStyles) {
      for (const register of ['precise', 'forms', 'input']) {
        const prompt = realizeGraph(graph, style, register, grammar);
        if (prompt !== null) prompts.add(prompt);
      }
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

/** The languages whose grammar the engine loaded. */
export const engineLanguages = () => Object.keys(policy.grammars).sort();

/** One language's contract: a digest per profile and every dialogue turn. */
export function parityFixture(language) {
  const projections = Object.fromEntries(policy.profiles.map(profile => [profile.id, digest(projectionRows(language, profile.id))]));
  return {language, densities: DENSITIES, projections, dialogue: dialogueCases(language)};
}

// One case per line keeps the fixture reviewable.
export function formatFixture(fixture) {
  const lines = ['{', `  "language": ${JSON.stringify(fixture.language)},`, `  "densities": ${JSON.stringify(fixture.densities)},`, '  "projections": {'];
  const entries = Object.entries(fixture.projections);
  entries.forEach(([key, value], index) => lines.push(`    ${JSON.stringify(key)}: ${JSON.stringify(value)}${index < entries.length - 1 ? ',' : ''}`));
  lines.push('  },', '  "dialogue": [');
  fixture.dialogue.forEach((item, at) => lines.push(`    ${JSON.stringify(item)}${at < fixture.dialogue.length - 1 ? ',' : ''}`));
  lines.push('  ]', '}');
  return lines.join('\n') + '\n';
}

if (import.meta.url === `file://${process.argv[1]}` && process.argv[2] === '--rows') {
  // Debug a digest mismatch: --rows <language> <profile> prints the digested rows.
  console.log(projectionRows(process.argv[3], process.argv[4]).join('\n'));
} else if (import.meta.url === `file://${process.argv[1]}`) {
  mkdirSync(PARITY_DIR, {recursive: true});
  for (const language of engineLanguages()) {
    const text = formatFixture(parityFixture(language));
    const file = parityPath(language);
    if (process.argv.includes('--write')) {
      writeFileSync(file, text);
      console.log(`wrote ${file} (${text.split('\n').length - 1} lines)`);
    } else {
      const current = (() => { try { return readFileSync(file, 'utf8'); } catch { return ''; } })();
      console.log(`${file}: ${current === text ? 'current' : 'stale: run with --write'}`);
      if (current !== text) process.exitCode = 1;
    }
  }
}
