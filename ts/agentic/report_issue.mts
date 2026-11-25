// Confirm and execute agentic report requests with complete context (#822):
// rust/src/agentic_coding/report_issue.rs.
//
// Targets are 'harness_log' | 'server_log' | 'github_issue' | 'formal_ai';
// report contents are 'both' | 'harness' | 'server'.

import { callbackNonempty as canonicalUnaryNonempty } from './callback_values.mjs';
import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { plainText, stripSystemEcho, systemPromptText, userRequestText } from './content.mjs';
import { currentDialogId } from './crate/dialog_log.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { issueTitle, reportTurn, titleSettingsFromSeed } from './crate/issue_report_title.mjs';
import { detect } from './crate/language.mjs';
import { replaceAllLiteral, splitWhitespace, trim, utf8Len } from './crate/rust_str.mjs';
import { localizedResponse, responseValuesFor } from './crate/seed.mjs';
import { agentInfoValue } from './crate/seed_agent_info.mjs';
import { mentionsRole, wordsForRole } from './crate/seed_meanings.mjs';
import { agenticMessage } from './messages.mjs';
import { finalAnswer, jsonText, planOne } from './plan.mjs';
import { Progress } from './progress.mjs';
import { ReportScript, shellQuote } from './report_script.mjs';
import { observedPayload, StepOutcome, stepOutcome } from './tool_result.mjs';

const CONTEXT_SESSION_FLAG = ' --session ';
const SOURCE_FLAG = ' --source ';
const OUTPUT_FLAG = ' --output ';
const SEPARATE_CONTEXT_LINKS_FLAG = ' --separate-context-links';
const SURFACE_FLAG = ' --surface ';
const TITLE_FLAG = ' --title ';
const REPOSITORY_FLAG = ' --repo ';
const FORMAL_AI_PROGRAM = 'formal-ai';
const PRINTF_PROGRAM = 'printf';
const PRINTF_LINE_FORMAT = " '%s\\n' ";
const AGENTIC_SURFACE = 'agentic-cli';
const LATEST_SESSION = 'latest';
const LANGUAGES = ['en', 'ru', 'hi', 'zh', 'es'];
const TARGET_QUESTION = 'agentic_report_target_question';
const CONTENTS_QUESTION = 'agentic_report_contents_question';

const ROLE_AGENT_ACTION_REPORT_VERB = 'agent_action_report_verb';
const ROLE_AGENT_ACTION_REPORT_SUBJECT = 'agent_action_report_subject';
const ROLE_FILE_WRITE_ACTION_CUE = 'file_write_action_cue';
const ROLE_FILE_WRITE_TARGET_CUE = 'file_write_target_cue';

/** Mirrors `fn plan_text_report_flow`. */
export function planTextReportFlow(messages) {
  const system = systemPromptText(messages);
  const marker = agentInfoValue('aider-whole-file-format');
  if (!marker || !system.includes(marker)) return planReportFlow(messages, []);
  const notices = (agentInfoValue('aider-report-injected-file-notices') ?? '').split('\n').filter(Boolean);
  const acknowledgment = agentInfoValue('aider-injected-acknowledgment');
  const conversation = [];
  for (let index = 0; index < messages.length; index += 1) {
    const message = messages[index];
    if (!isUser(message)) {
      conversation.push(message);
      continue;
    }
    const text = stripSystemEcho(userRequestText(message.content), system);
    const next = messages[index + 1];
    if (notices.some(prefix => text.startsWith(prefix))
      && acknowledgment && next?.role.toLowerCase() === 'assistant'
      && plainText(next.content).startsWith(acknowledgment)) {
      index += 1;
    } else {
      conversation.push({ ...message, content: text });
    }
  }
  return planReportFlow(conversation, []);
}

const isUser = (message) => message.role.toLowerCase() === 'user';

/**
 * Mirrors `fn plan_report_flow` in rust/src/agentic_coding/report_issue.rs:
 * the next step of an active report flow, or null.
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planReportFlow(messages, toolNames) {
  let reportIndex = -1;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (isUser(messages[index]) && isReportIntent(userRequestText(messages[index].content))) {
      reportIndex = index;
      break;
    }
  }
  if (reportIndex < 0) return null;
  // Responses and Gemini can deliver narration and the associated call as
  // consecutive assistant messages in the same turn.
  const acknowledged = messages.slice(reportIndex + 1).some((message, offset) =>
    message.role.toLowerCase() === 'assistant'
    && !(message.tool_calls || []).length
    && !(messages[reportIndex + offset + 2]?.tool_calls || []).length
    && trim(plainText(message.content)) !== ''
    && !isReportQuestion(plainText(message.content)));
  if (acknowledged) return null;
  const language = detect(userRequestText(messages[reportIndex].content));
  const choices = messages.slice(reportIndex + 1).map((message, offset) => {
    const index = reportIndex + 1 + offset;
    const previous = messages[index - 1];
    const question = previous.role.toLowerCase() === 'assistant' ? plainText(previous.content) : '';
    return { index, message, question };
  }).filter(({ message }) => isUser(message) || message.role.toLowerCase() === 'tool');
  let targets = null;
  let targetIndex = -1;
  for (const choice of choices) {
    const parsed = parseTargets(choice, language);
    if (parsed.length) {
      targets = parsed;
      targetIndex = choice.index;
      break;
    }
  }
  if (targets === null) {
    return askOrRender(toolNames, language, 'report_target', TARGET_QUESTION,
      'agentic_report_target_options', true);
  }
  let explicitContents = null;
  for (const choice of choices) {
    if (choice.index <= targetIndex) continue;
    explicitContents = parseContents(choice, language);
    if (explicitContents !== null) break;
  }
  const selectedHarness = targets.includes('harness_log');
  const selectedServer = targets.includes('server_log');
  let inferred = null;
  if (selectedHarness && selectedServer) inferred = 'both';
  else if (selectedHarness) inferred = 'harness';
  else if (selectedServer) inferred = 'server';
  const contents = explicitContents ?? inferred;
  if (targets.includes('github_issue') && contents === null) {
    return askOrRender(toolNames, language, 'report_contents', CONTENTS_QUESTION,
      'agentic_report_contents_options', false);
  }
  // Injected reminders after the receipt do not start a new user turn.
  const progress = Progress.scan(messages.slice(reportIndex + 1).filter(message =>
    !isUser(message) || userRequestText(message.content)));
  const dialog = dialogId();
  const commands = commandsForTargets(targets, contents ?? 'both', messages, reportIndex, dialog);
  const command = commands[progress.run_outputs.length];
  if (command === undefined) return finalAnswer(reportFinished(targets, progress.run_outputs, language));
  const tool = toolFor(toolNames, Capability.Run) ?? null;
  if (tool !== null) return planOne(tool, jsonText({ command }));
  if (dialog === LATEST_SESSION) return finalAnswer(renderConfig('issue-report-session-missing', []));
  const actions = commands.slice(progress.run_outputs.length).map(command => command.replaceAll('\n', '; ')).join('\n\n');
  return finalAnswer(renderConfig('issue-report-manual', [['commands', actions]]));
}

/** Mirrors `fn ask_or_render`. */
function askOrRender(toolNames, language, id, questionIntent, optionsIntent, multiple) {
  const question = localized(questionIntent, language);
  const options = localizedOptions(optionsIntent, language);
  const tool = toolFor(toolNames, Capability.AskUser) ?? null;
  if (tool !== null) {
    return planOne(tool, jsonText({
      questions: [{
        header: 'Report',
        id,
        question,
        options: options.map(([label, description]) => ({ label, description })),
        // OpenCode's `question` reads `multiple`; Claude Code's AskUserQuestion
        // reads `multiSelect` (issue #1632). Each client ignores the other's field.
        multiple,
        multiSelect: multiple,
      }],
    }));
  }
  // Keep every choice and its description within clients' concise output limits.
  const text = question + '\n' + options.map(([label, description], index) =>
    `${index + 1}. ${label} — ${description}`).join('  ');
  return finalAnswer(text);
}

/** Mirrors `fn localized`. */
function localized(intent, language) {
  return localizedResponse(intent, language) ?? '';
}

/** Mirrors `fn localized_options`: `[label, description]` pairs. */
function localizedOptions(intent, language) {
  const values = responseValuesFor(intent, language) ?? responseValuesFor(intent, 'en') ?? [];
  const out = [];
  for (let index = 0; index + 1 < values.length; index += 2) out.push([values[index], values[index + 1]]);
  return out;
}

/** Mirrors `fn is_report_question`. */
function isReportQuestion(text) {
  return LANGUAGES.some((language) =>
    [TARGET_QUESTION, CONTENTS_QUESTION]
      .some((intent) => text.includes(localized(intent, language))));
}

/** Mirrors `fn parse_targets`. */
function parseTargets(choice, language) {
  const matches = optionMatcher(choice, language, TARGET_QUESTION, 'agentic_report_target_options', true);
  return [
    ['harness_log', 0, 'harness_log'],
    ['server_log', 1, 'server_log'],
    ['github_issue', 2, 'github_issue'],
    ['formal_ai', 3, 'formal_ai'],
  ].filter(([, optionIndex, machineValue]) => matches(optionIndex, machineValue))
    .map(([target]) => target);
}

/** Mirrors `fn parse_contents`. */
function parseContents(choice, language) {
  const matches = optionMatcher(choice, language, CONTENTS_QUESTION, 'agentic_report_contents_options', false);
  if (matches(0, 'both_logs')) return 'both';
  if (matches(1, 'harness_log')) return 'harness';
  return matches(2, 'server_log') ? 'server' : null;
}

/** Mirrors `fn option_matcher`. */
function optionMatcher({ message, question }, language, questionIntent, optionsIntent, multiple) {
  const user = isUser(message);
  const text = user ? userRequestText(message.content) : plainText(message.content);
  const answers = (user && multiple ? text.split(',') : [text]).map(normalizePrompt);
  const numbered = user && question.trimStart().startsWith(localized(questionIntent, language) + '\n1. ');
  return (index, value) => {
    const matches = value => {
      const option = normalizePrompt(value);
      return answers.some(answer => user ? answer === option : answer.includes(option));
    };
    return matches(value) || (numbered && matches(String(index + 1)))
      || LANGUAGES.some(lang => {
        const option = localizedOptions(optionsIntent, lang)[index];
        return option !== undefined && matches(option[0]);
      });
  };
}

/**
 * Mirrors `fn is_report_intent` in rust/src/agentic_coding/report_issue.rs.
 * @param {string} task
 */
export function isReportIntent(task) {
  const normalized = normalizePrompt(task);
  const action = mentionsRole(ROLE_AGENT_ACTION_REPORT_VERB, normalized);
  const bareAction = wordsForRole(ROLE_AGENT_ACTION_REPORT_VERB).some((word) => normalizePrompt(word) === normalized);
  return bareAction || (action && mentionsRole(ROLE_AGENT_ACTION_REPORT_SUBJECT, normalized)
    && reportActionGovernsSubject(normalized));
}

/** An issue form for chat surfaces that do not have an exportable session. */
export function issueFormForPrompt(prompt) {
  if (!isReportIntent(prompt)) return null;
  const instruction = localized('agentic-report-open-issue-form', detect(prompt));
  return `${instruction}\nhttps://github.com/${formalAiRepo()}/issues/new`;
}

/** Mirrors `fn report_action_governs_subject` (distances in UTF-8 bytes). */
function reportActionGovernsSubject(normalized) {
  const padded = ` ${normalized} `;
  const hasWhitespace = /\p{White_Space}/u.test(normalized);
  const matchesFor = (role) => {
    const out = [];
    for (const raw of wordsForRole(role)) {
      const word = normalizePrompt(raw);
      const at = padded.indexOf(` ${word} `);
      if (at >= 0) {
        out.push([utf8Len(padded.slice(0, at)), word]);
        continue;
      }
      if (!hasWhitespace) {
        const inner = normalized.indexOf(word);
        if (inner >= 0) out.push([utf8Len(normalized.slice(0, inner)), word]);
      }
    }
    return out;
  };
  const actions = matchesFor(ROLE_AGENT_ACTION_REPORT_VERB);
  const subjects = matchesFor(ROLE_AGENT_ACTION_REPORT_SUBJECT);
  const ambiguous = [ROLE_FILE_WRITE_ACTION_CUE, ROLE_FILE_WRITE_TARGET_CUE]
    .flatMap((role) => wordsForRole(role))
    .map(normalizePrompt);
  return actions.some(([actionPosition, action]) => subjects.some(([subjectPosition]) => {
    const distance = Math.abs(actionPosition - subjectPosition);
    if (ambiguous.includes(action)) return actionPosition < subjectPosition && distance <= 16;
    return distance <= 32;
  }));
}

/** Mirrors `fn commands_for_targets`. */
function commandsForTargets(targets, contents, messages, reportIndex, dialog) {
  return executionOrder(targets).map((target) => commandFor(target, contents, messages, reportIndex, dialog));
}

/** Mirrors `fn execution_order`: the GitHub issue last (stable). */
function executionOrder(targets) {
  return [...targets.filter((target) => target !== 'github_issue'),
    ...targets.filter((target) => target === 'github_issue')];
}

/** Mirrors `fn command_for`. */
function commandFor(target, contents, messages, reportIndex, dialog) {
  switch (target) {
    case 'github_issue':
      return githubCommand(contents, messages, reportIndex, dialog);
    case 'harness_log':
      return exportCommand(dialog, 'harness', `formal-ai-harness-${dialog}.lino`);
    case 'server_log':
      return exportCommand(dialog, 'server', `formal-ai-server-${dialog}.lino`);
    default:
      return learningCommand(dialog);
  }
}

export function reportCommandIntent(command) {
  const submit = agenticMessage('report-issue-submit-command') + CONTEXT_SESSION_FLAG;
  const learn = agenticMessage('report_issue_context_learn_command') + CONTEXT_SESSION_FLAG;
  const exportContext = agenticMessage('report_issue_context_export_command') + CONTEXT_SESSION_FLAG;
  for (const line of command.split('\n')) {
    if (line.startsWith(submit)) return 'agentic-action-report-github';
    if (line.startsWith(learn)) return 'agentic-action-report-learn';
    if (line.startsWith(exportContext)) {
      if (line.includes(SOURCE_FLAG + 'harness' + OUTPUT_FLAG)) return 'agentic-action-report-harness';
      if (line.includes(SOURCE_FLAG + 'server' + OUTPUT_FLAG)) return 'agentic-action-report-server';
    }
  }
  return null;
}

/** Mirrors `fn export_command`. */
function exportCommand(dialog, contents, output) {
  const script = new ReportScript();
  const outputPath = script.export(output);
  const command = agenticMessage('report_issue_context_export_command') + CONTEXT_SESSION_FLAG + shellQuote(dialog)
    + SOURCE_FLAG + sourceName(contents) + OUTPUT_FLAG + outputPath;
  script.step(FORMAL_AI_PROGRAM, command);
  script.step(PRINTF_PROGRAM, PRINTF_PROGRAM + PRINTF_LINE_FORMAT + outputPath);
  return script.render();
}

/** Mirrors `fn learning_command`. */
function learningCommand(dialog) {
  const command = agenticMessage('report_issue_context_learn_command') + CONTEXT_SESSION_FLAG + shellQuote(dialog);
  const script = new ReportScript();
  script.step(FORMAL_AI_PROGRAM, command);
  return script.render();
}

/** Mirrors `fn github_command`. */
function githubCommand(contents, messages, reportIndex, dialog) {
  let command = agenticMessage('report-issue-submit-command') + CONTEXT_SESSION_FLAG + shellQuote(dialog)
    + SOURCE_FLAG + sourceName(contents) + SURFACE_FLAG + AGENTIC_SURFACE
    + REPOSITORY_FLAG + shellQuote(formalAiRepo()) + TITLE_FLAG + shellQuote(reportTitle(messages, reportIndex));
  if (contents === 'both') command += SEPARATE_CONTEXT_LINKS_FLAG;
  return command;
}

/** Mirrors `const fn target_label`. */
function targetLabel(target) {
  return agenticMessage(`report_issue_target_${target}`);
}

/** Mirrors `const fn source_name`. */
function sourceName(contents) {
  return contents;
}

/** Mirrors `fn dialog_id`. */
function dialogId() {
  const id = currentDialogId();
  const trimmed = id === null ? '' : trim(id);
  return trimmed === '' ? LATEST_SESSION : trimmed;
}

/** Mirrors `fn report_title`. */
function reportTitle(messages, reportIndex) {
  return issueTitle(titleTurns(messages, reportIndex), titleSettingsFromSeed());
}

/** Mirrors `fn title_turns`. */
function titleTurns(messages, reportIndex) {
  const out = [];
  messages.slice(0, reportIndex + 1).forEach((message, index) => {
    if (!isUser(message)) return;
    out.push(reportTurn(message.role, userRequestText(message.content), index === reportIndex));
  });
  return out;
}

/** Mirrors `fn report_finished`. */
function reportFinished(targets, runOutputs, language) {
  const ordered = executionOrder(targets);
  const failures = [];
  let issueLink;
  let githubFailed = false;
  const prefix = 'https://github.com/' + formalAiRepo() + '/issues/';
  ordered.forEach((target, index) => {
    if (index >= runOutputs.length) return;
    const raw = runOutputs[index];
    const payload = observedPayload(raw);
    const failed = !trim(payload ?? '') || stepOutcome(raw) === StepOutcome.Failed
      || stepOutcome(payload) === StepOutcome.Failed;
    if (target === 'github_issue') {
      if (!failed) issueLink = splitWhitespace(payload).find(token => token.startsWith(prefix) && /^[1-9][0-9]*$/u.test(token.slice(prefix.length)));
      githubFailed = issueLink === undefined;
    }
    if (failed || (target === 'github_issue' && githubFailed)) {
      failures.push([target, trim(raw) || config('issue-report-no-receipt')]);
    }
  });
  const created = issueLink === undefined ? undefined : renderConfig('issue_report_created_with_url', [['url', issueLink]]);
  if (failures.length) {
    const detail = failures.map(([target, output]) => `${targetLabel(target)}: ${output}`).join('\n');
    const heading = config(githubFailed ? 'issue_report_failed' : 'issue-report-destinations-failed');
    return [created, `${heading}\n\n\`\`\`text\n${detail}\n\`\`\``].filter(Boolean).join('\n\n');
  }
  if (created !== undefined) return created;
  const trimmed = runOutputs.map(trim).filter(canonicalUnaryNonempty).join('\n');
  const exported = localized('agentic_report_exported', language);
  return trimmed === '' ? exported : `${exported}\n\n\`\`\`text\n${trimmed}\n\`\`\``;
}

/** Mirrors `fn formal_ai_repo`. */
function formalAiRepo() {
  return config('repository');
}

/** Mirrors `fn config`: the agent-info field, or the key itself. */
function config(key) {
  return agentInfoValue(key) ?? key;
}

/** Mirrors `fn render`. */
function renderConfig(key, values) {
  return values.reduce((text, [name, value]) => replaceAllLiteral(text, `{${name}}`, value), config(key));
}
