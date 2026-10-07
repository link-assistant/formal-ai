// Issue #932: the box-language project corpus, pinned on the JavaScript root.
//
// scripts/generate-box-language-corpus.sh asks the solver for one program per
// language in four locales and for a shell script converted from each
// language's README installation guide; the box images then build and run what
// came back. rust/tests/unit/issue_932_box_language_projects.rs holds the Rust
// solver to that contract. This file holds the browser worker twin to the same
// contract, read from the same data/meta/box-language-projects.lino, so the
// corpus is a property of the seed and not of one root.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { REPO_ROOT, createWorkerContext, evaluate } from './support/browser-runtime.mjs';

let worker;

before(async () => {
  worker = createWorkerContext();
  await evaluate(worker, 'loadSeed()');
});

const solve = (prompt) => worker.solve(prompt, [], {}, {}, [], {});

/** The contract's top-level records, each `{id, fields, steps}` with `init_step_N`/`init_command_N` paired in order. */
function contractRecords() {
  const text = readFileSync(`${REPO_ROOT}/data/meta/box-language-projects.lino`, 'utf8');
  return text.split(/\n(?=\S)/).map((block) => {
    const [id, ...lines] = block.split('\n');
    const fields = {};
    for (const line of lines) {
      const match = line.match(/^ {2}(\w+) (?:"(.*)"|\((.*)\))$/);
      if (match) fields[match[1]] = match[2] ?? match[3].split(' ').map((item) => item.replace(/"/g, ''));
    }
    const steps = [];
    for (let index = 1; fields[`init_command_${index}`]; index += 1) {
      steps.push({ description: fields[`init_step_${index}`], command: fields[`init_command_${index}`] });
    }
    return { id, fields, steps };
  });
}

const RECORDS = contractRecords();
const CONTRACT = RECORDS.find((record) => record.fields.record_type === 'box_language_project_contract').fields;
const PROJECTS = RECORDS.filter((record) => record.fields.record_type === 'box_language_project');
const PROMPTS = RECORDS.filter((record) => record.fields.record_type === 'box_language_prompt').map((record) => record.fields);

/** First fenced block opened with exactly ```<fence>, as the corpus generator extracts it. */
function fencedBlock(answer, fence) {
  const lines = answer.split('\n');
  const start = lines.findIndex((line) => line.trimEnd() === '```' + fence);
  if (start === -1) return null;
  const block = [];
  for (const line of lines.slice(start + 1)) {
    if (line.trimEnd() === '```') break;
    block.push(line);
  }
  return block.length ? block.join('\n') : null;
}

/** The README guide exactly as `BoxLanguageProject::install_guide` builds it. */
function installGuide(project) {
  return ['## Installation', ...project.steps.map((step, index) => `${index + 1}. ${step.description}\n   \`${step.command}\``)].join('\n');
}

describe('R932-9: the language to image mapping is reviewable data', () => {
  test('every project names its image, fence, program file and traditional init commands', () => {
    assert.deepEqual(CONTRACT.prompt_locales, ['en', 'ru', 'hi', 'zh']);
    assert.deepEqual(PROJECTS.map((project) => project.fields.language),
      ['rust', 'python', 'javascript', 'typescript', 'go', 'java', 'ruby']);
    for (const project of PROJECTS) {
      assert.match(project.fields.image, /^konard\/box-/, project.id);
      assert.ok(project.fields.code_fence && project.fields.program_file, project.id);
      assert.ok(project.steps.length >= 4, `${project.id} carries its init steps`);
    }
  });
});

describe('R932-7: the corpus is the same program in english, russian, hindi and chinese', () => {
  for (const project of PROJECTS) {
    test(project.fields.language, async () => {
      const programs = [];
      for (const locale of CONTRACT.prompt_locales) {
        const prompt = PROMPTS.find((entry) => entry.locale === locale).template.replace('{display_name}', project.fields.display_name);
        const answer = await solve(prompt);
        const program = fencedBlock(answer.content, project.fields.code_fence);
        assert.ok(program, `${project.fields.language}/${locale} carried a fenced program: ${answer.content}`);
        assert.ok(program.includes(CONTRACT.expected_output), `${project.fields.language}/${locale} prints ${CONTRACT.expected_output}`);
        programs.push(program);
      }
      assert.deepEqual(programs, programs.map(() => programs[0]), 'byte-identical across the four locales');
    });
  }
});

describe('R932-2/R932-13: an installation guide converts into a script that keeps every traditional command', () => {
  for (const project of PROJECTS) {
    test(project.fields.language, async () => {
      const answer = await solve(`Convert this README.md installation guide into a sh script:\n\n\`\`\`markdown\n${installGuide(project)}\n\`\`\``);
      assert.equal(answer.intent, 'installation_conversion', answer.content);
      const script = fencedBlock(answer.content, 'bash');
      assert.ok(script, answer.content);
      for (const step of project.steps) {
        assert.ok(script.split('\n').includes(step.command), `${project.fields.language} keeps \`${step.command}\``);
      }
    });
  }
});
