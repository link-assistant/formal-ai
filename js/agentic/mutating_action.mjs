// Verified mutating filesystem actions (issues #824 and #944): the JavaScript
// twin of rust/src/agentic_coding/mutating_action.rs.
//
// A mutating command is planned as the ordered recipe its seed intent declares
// (preconditions, preparation, the action, postconditions), each step observed
// before the next is planned.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { finalAnswer, jsonText, planOne } from './plan.mjs';
import { Progress } from './progress.mjs';
import { fill as fillSeededStep } from './work_item_steps.mjs';
import { StepOutcome, render, reportedExitCode, responseLanguage, stepOutcome } from './tool_result.mjs';
import { localizedResponse } from './crate/seed.mjs';
import { effectIsDeclared, shellIntentVocabulary } from './crate/seed_shell_intents.mjs';
import { maxByKey, replaceAllLiteral, rsplitOnce } from './crate/rust_str.mjs';

const PATH_PLACEHOLDER = '{path}';
const SOURCE_PLACEHOLDER = '{source}';
const DESTINATION_PLACEHOLDER = '{destination}';
const DESTINATION_PARENT_PLACEHOLDER = '{destination_parent}';
const ACTION_PLACEHOLDER = '{action}';
const CHECK_PLACEHOLDER = '{check}';
const CHECKS_PLACEHOLDER = '{checks}';
const EXIT_CODE_PLACEHOLDER = '{exit_code}';
const CURRENT_DIRECTORY = '.';

/**
 * Mirrors `fn expand` in rust/src/agentic_coding/mutating_action.rs:
 * `{steps, action}` (the action's index) or null.
 * @param {string} command
 */
export function expand(command) {
  return expandWith(command, shellIntentVocabulary());
}

function expandWith(command, vocab) {
  const candidates = vocab.intents.filter((intent) => effectIsDeclared(intent.effect)
    && command.startsWith(intent.command + ' '));
  const intent = maxByKey(candidates, (candidate) => candidate.command.length);
  if (!intent) return null;
  const parsed = effectOperands(command.slice(intent.command.length), intent.effect);
  if (!parsed || !parsed.operands.length) return null;
  const { operands, flags } = parsed;
  const effect = intent.effect;
  const before = flags.some((flag) => effect.reuse_options.includes(flag)) && effect.before_reuse.length
    ? effect.before_reuse : effect.before;
  const templates = [...before, ...effect.prepare, ...effect.after];
  const fill = (template, bindings) => bindings.reduce((text, [key, value]) => replaceAllLiteral(text, key, value), template);
  let groups;
  if (templates.some((template) => template.includes(PATH_PLACEHOLDER))) {
    groups = operands.map((path) => (template) => fill(template, [[PATH_PLACEHOLDER, path]]));
  } else {
    if (operands.length < 2 || (operands.length > 2 && !effect.directory_targets)) return null;
    const destination = operands.at(-1);
    const sources = operands.slice(0, -1);
    if (!effect.directory_targets && sources.some(hasShellExpansion)) return null;
    const collection = effect.directory_targets && (sources.length > 1
      || sources.some(hasShellExpansion) || destination.replace(/^['"]|['"]$/gu, '').endsWith('/'));
    if (collection) {
      const setup = fillSeededStep('filesystem-collection-setup', [['{destination}', destination]]);
      const bindings = [[SOURCE_PLACEHOLDER, '"$formal_ai_source"'],
        [DESTINATION_PARENT_PLACEHOLDER, '"$formal_ai_parent"'], [DESTINATION_PLACEHOLDER, '"$formal_ai_destination"']];
      groups = [(template) => fillSeededStep('filesystem-collection-check',
        [['{sources}', sources.join(' ')], ['{setup}', setup], ['{check}', fill(template, bindings)]])];
    } else {
      groups = [(template) => fill(template, [[SOURCE_PLACEHOLDER, sources[0]],
        [DESTINATION_PARENT_PLACEHOLDER, parentOf(destination)], [DESTINATION_PLACEHOLDER, destination]])];
    }
  }
  const expandTemplates = (items) => items.flatMap((template) => groups.map((bind) => bind(template)));
  const steps = [...expandTemplates(before), ...expandTemplates(effect.prepare)];
  const action = steps.length;
  steps.push(command, ...expandTemplates(effect.after));
  return { steps, action };
}

/** Preserve shell quoting while separating declared options from positional operands. */
function effectOperands(text, effect) {
  const words = shellWords(text);
  if (words === null) return null;
  const operands = [];
  const flags = [];
  let options = true;
  for (let index = 0; index < words.length; index += 1) {
    const raw = words[index];
    const word = raw.replace(/^['"]|['"]$/gu, '');
    if (options && word === '--') { options = false; continue; }
    if (options && word.startsWith('-') && word !== '-') {
      const valueOption = effect.value_options.find((option) => word === option || word.startsWith(option + '=')
        || (!option.startsWith('--') && word.startsWith(option) && word.length > option.length));
      if (valueOption) {
        if (word === valueOption && ++index >= words.length) return null;
        flags.push(valueOption);
      } else if (effect.flags.includes(word)) flags.push(word);
      else if (!word.startsWith('--') && [...word.slice(1)].every((letter) => effect.flags.includes('-' + letter))) {
        flags.push(...[...word.slice(1)].map((letter) => '-' + letter));
      } else return null;
    } else operands.push(raw);
  }
  return { operands, flags };
}

function shellWords(text) {
  const words = [];
  let start = null;
  let quote = null;
  let escaped = false;
  for (let index = 0; index < text.length; index += 1) {
    const character = text[index];
    if (start === null && /\s/u.test(character)) continue;
    if (start === null) start = index;
    if (escaped) { escaped = false; continue; }
    if (character === '\\' && quote !== "'") { escaped = true; continue; }
    if (quote !== null) { if (character === quote) quote = null; continue; }
    if (character === '"' || character === "'") { quote = character; continue; }
    if (/\s/u.test(character)) { words.push(text.slice(start, index)); start = null; }
    else if (';|&<>()`'.includes(character)) return null;
  }
  if (quote !== null || escaped) return null;
  if (start !== null) words.push(text.slice(start));
  return words;
}

function hasShellExpansion(word) {
  let quote = null;
  let escaped = false;
  for (const character of word) {
    if (escaped) { escaped = false; continue; }
    if (character === '\\' && quote !== "'") { escaped = true; continue; }
    if (quote !== null) { if (character === quote) quote = null; continue; }
    if (character === '"' || character === "'") quote = character;
    else if ('*?[$'.includes(character)) return true;
  }
  return false;
}

/** Mirrors `fn parent_of`. */
function parentOf(path) {
  if (/["']/u.test(path) || path.includes(String.fromCharCode(92))) return '"$(dirname -- ' + path + ')"';
  const split = rsplitOnce(path, '/');
  if (split === null) return CURRENT_DIRECTORY;
  return split[0] === '' ? '/' : split[0];
}

/**
 * Mirrors `fn plan_step` in rust/src/agentic_coding/mutating_action.rs.
 * @param {string} command
 * @param {object[]} messages
 * @param {string[]} toolNames
 * @param {string} prompt
 */
export function planStep(command, messages, toolNames, prompt) {
  const recipe = expand(command);
  if (recipe === null) return null;
  const tool = toolFor(toolNames, Capability.Run);
  if (tool === null) return null;
  const progress = Progress.scan(messages);
  const taken = progress.run_outputs.length;
  if (taken > 0) {
    const index = taken - 1;
    const observed = progress.run_outputs[index];
    if (stepOutcome(observed) === StepOutcome.Failed) {
      return finalAnswer(blockedReport(recipe, recipe.steps[index] ?? command, observed, prompt, index >= recipe.action));
    }
  }
  if (taken < recipe.steps.length) return planOne(tool, jsonText({ command: recipe.steps[taken] }));
  return finalAnswer(completedReport(recipe, prompt));
}

function blockedReport(recipe, check, observed, prompt, attempted) {
  const language = responseLanguage(prompt);
  let answer = localizedResponse(attempted ? 'mutating-action-observed-failure' : 'mutating_action_blocked', language);
  if (answer === null) return render(check, observed, prompt);
  answer = replaceAllLiteral(answer, ACTION_PLACEHOLDER, recipe.steps[recipe.action]);
  answer = replaceAllLiteral(answer, CHECK_PLACEHOLDER, check);
  const code = reportedExitCode(observed);
  return replaceAllLiteral(answer, EXIT_CODE_PLACEHOLDER, code === null ? '' : String(code));
}

function completedReport(recipe, prompt) {
  const language = responseLanguage(prompt);
  const checks = recipe.steps.slice(recipe.action + 1).map((check) => `\`${check}\``).join(', ');
  const answer = localizedResponse('mutating_action_completed', language);
  if (answer === null) return recipe.steps[recipe.action];
  return replaceAllLiteral(replaceAllLiteral(answer, ACTION_PLACEHOLDER, recipe.steps[recipe.action]), CHECKS_PLACEHOLDER, checks);
}

/**
 * Mirrors `fn verified_recipe` in rust/src/agentic_coding/mutating_action.rs.
 * @param {string} command
 * @returns {string[]|null}
 */
export function verifiedRecipe(command) {
  return expand(command)?.steps ?? null;
}
