// Issue #80: software project requests, pinned on the JavaScript root.
//
// The browser worker twin (`trySoftwareProjectRequest` and the approval
// follow-up in js/worker/) must formalize an open-ended request to build a
// software artifact into a reviewable Links Notation meaning record, derive the
// reasoning and plan from it, and generate starter code only after the user
// approves. The prompts and expectations are the ones
// rust/tests/unit/software_project.rs and rust/tests/unit/formal_ai.rs pin on
// the Rust solver; the two documented plans are compared byte for byte with the
// Rust constants, so the roots cannot drift apart silently.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { REPO_ROOT, createWorkerContext, evaluate } from './support/browser-runtime.mjs';

let worker;

before(async () => {
  worker = createWorkerContext();
  await evaluate(worker, 'loadSeed()');
});

const solve = (prompt, history = []) => worker.solve(prompt, history, {}, {}, [], {});

/** A `const NAME: &str = r#"…"#;` plan the Rust regression suite pins. */
function rustPlan(name) {
  const source = readFileSync(`${REPO_ROOT}/rust/tests/unit/formal_ai.rs`, 'utf8');
  const match = source.match(new RegExp(`const ${name}: &str = r#"([\\s\\S]*?)"#;`));
  assert.ok(match, `${name} is declared in rust/tests/unit/formal_ai.rs`);
  return match[1];
}

const OWLBEAR_PROMPT = 'Hi, can you write for me extension for owlbear? I am currently leading some dnd games '
  + 'and i want to try wargame. So, i need extensions that can track hp for different units, '
  + 'that can track Protection and Resistance stacks on unit an will reduce damage count on '
  + 'those stats. Also this extension should track cooldown of some abilities';

/** [prompt, artifact, delivery mode, language, starter label, code fence, implementation needle, extra gate]. */
const DIALOGUES = [
  ['Write an extension for Owlbear that tracks HP, Protection, Resistance, damage, and cooldowns', 'extension', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'mitigateDamage', 'generated_code'],
  ['Build a browser extension for reading progress that tracks pages and exports CSV', 'browser extension', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'applyCommand', 'generated_code'],
  ['Create a JavaScript Discord bot for scheduling game sessions with reminders', 'bot', 'code_generation', 'javascript', 'JavaScript', '```javascript', 'export function applyCommand', 'generated_code'],
  ['Implement a React web app for invoices that tracks overdue payments and exports reports', 'web app', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_code'],
  ['Make a plugin for a tabletop map that tracks unit status effects', 'plugin', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_code'],
  ['Develop a Rust command line tool for renaming photos by date', 'command-line tool', 'code_generation', 'rust', 'Rust', '```rust', 'pub enum ProjectCommand', 'generated_code'],
  ['Generate a mobile app for habit tracking with notifications and backups', 'mobile app', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_code'],
  ['Design a service for importing customer invoices and sending payment reminders', 'service', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_code'],
  ['Scaffold a website for event schedules that exports calendar data', 'website', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_code'],
  ['Create a Python API for tracking equipment status and maintenance dates', 'API', 'code_generation', 'python', 'Python', '```python', 'def apply_command', 'generated_code'],
  ['Build a bot for project reports that sends weekly notifications', 'bot', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_code'],
  ['Make an add-on for a tabletop token that tracks hp and damage', 'extension', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'mitigateDamage', 'generated_code'],
  ['Build a Python CLI tool for importing CSV tasks and exporting weekly reports with manual instructions', 'command-line tool', 'manual_instructions', 'python', 'Python', '```python', 'def apply_command', 'manual_instructions'],
  ['Write a Python scraper that imports product prices and stores history', 'scraper', 'code_generation', 'python', 'Python', '```python', 'def apply_command', 'generated_code'],
  ['Implement a Rust library for validating configuration files', 'library', 'code_generation', 'rust', 'Rust', '```rust', 'pub enum ProjectCommand', 'generated_code'],
  ['Build an admin dashboard that filters users and exports audit logs', 'dashboard', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_code'],
  ['Make a GitHub Action that checks changelog fragments on pull requests', 'action', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_code'],
  ['Implement a plugin for a design tool that syncs assets and reports conflicts', 'plugin', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_code'],
  ['Build a TypeScript SDK for uploading files with retries and progress events', 'SDK', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_code'],
  ['Create a Telegram bot that tracks expenses and sends weekly reports', 'bot', 'code_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_code'],
  ['Generate a command line tool with shell commands for backing up project files and validating upload status', 'command-line tool', 'script_generation', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'generated_script'],
  ['Develop a web app for incident reports, run commands in WebVM, and approve each step', 'web app', 'immediate_execution', 'typescript', 'TypeScript', '```typescript', 'export function applyCommand', 'each_step'],
];

describe('R156/R157/R162: the reported prompt formalizes into a reviewable plan', () => {
  test('the Owlbear request is the plan the Rust solver gives, byte for byte', async () => {
    const answer = await solve(OWLBEAR_PROMPT);
    assert.equal(answer.intent, 'software_project_plan');
    assert.equal(answer.content, rustPlan('OWLBEAR_PROJECT_PLAN'));
    assert.ok(!answer.content.includes('mitigateDamage'), 'no code before approval');
  });

  test('generalized variations never fall back to the unknown intent', async () => {
    const documented = await solve('Build a browser extension that tracks reading progress and exports CSV');
    assert.equal(documented.content, rustPlan('BROWSER_EXTENSION_PROJECT_PLAN'));
    for (const prompt of [
      'Build a browser extension that tracks reading progress and exports CSV',
      'Create a Discord bot for scheduling game sessions with reminders',
      'Implement a small web app for tracking invoices and overdue payments',
      'Make a plugin for a tabletop map that tracks unit status effects',
      'Develop a command line tool for renaming photos by date',
    ]) {
      const answer = await solve(prompt);
      assert.equal(answer.intent, 'software_project_plan', prompt);
      for (const section of ['Formalized meaning', 'Proposed plan', 'approve plan']) {
        assert.ok(answer.content.includes(section), `${prompt}: ${section}`);
      }
    }
  });
});

describe('R158/R159/R161/R163/R164: twenty-two dialogues, text to meaning to plan to approval to code', () => {
  test('the matrix holds at least the twenty dialogues issue #80 asked for', () => {
    assert.ok(DIALOGUES.length >= 20);
  });

  for (const [prompt, artifact, deliveryMode, language, starterLabel, codeFence, needle, extraGate] of DIALOGUES) {
    test(prompt, async () => {
      const plan = await solve(prompt);
      assert.equal(plan.intent, 'software_project_plan');
      for (const fragment of [
        '```lino', 'software_project_request', 'approval_state proposed', `artifact "${artifact}"`,
        `delivery_mode ${deliveryMode}`, `implementation_language "${language}"`,
        'approval_gate "task_formalization"', 'approval_gate "implementation_plan"', `approval_gate "${extraGate}"`,
        'requirement_category', 'requirement graph', 'implementation subtask(s)', 'Reasoning steps',
        'Classify the impulse as a request', 'Select delivery mode', 'Ask for approval', 'Requirement model',
        'Subtasks', 'Approval gates', 'Proposed plan', 'Review the formalized task', 'approve plan',
      ]) {
        assert.ok(plan.content.includes(fragment), `plan carries ${fragment}`);
      }
      assert.ok(!plan.content.includes(codeFence), 'the first turn generates no code before approval');

      const history = [{ role: 'user', content: prompt }, { role: 'assistant', content: plan.content, intent: plan.intent }];
      const implementation = await solve('approve plan', history);
      assert.equal(implementation.intent, 'software_project_implementation');
      for (const fragment of ['approval_state approved', 'software_project_request', 'Implementation steps',
        'Generated code checks', `Starter ${starterLabel} core`, codeFence, needle]) {
        assert.ok(implementation.content.includes(fragment), `implementation carries ${fragment}`);
      }
    });
  }
});

describe('R160: the game-unit tracker starter arrives only after approval', () => {
  test('HP, Protection, Resistance, mitigation and cooldown ticking in a TypeScript core', async () => {
    const prompt = DIALOGUES[0][0];
    const plan = await solve(prompt);
    assert.ok(!plan.content.includes('tickCooldowns'));
    const implementation = await solve('approve plan', [{ role: 'user', content: prompt }, { role: 'assistant', content: plan.content, intent: plan.intent }]);
    for (const symbol of ['UnitState', 'mitigateDamage', 'setStacks', 'tickCooldowns', '```typescript']) {
      assert.ok(implementation.content.includes(symbol), symbol);
    }
  });
});
