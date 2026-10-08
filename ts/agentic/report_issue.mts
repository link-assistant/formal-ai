// Confirm and execute agentic report requests with complete context (#822):
// rust/src/agentic_coding/report_issue.rs.
//
// Targets are 'harness_log' | 'server_log' | 'github_issue' | 'formal_ai';
// report contents are 'both' | 'harness' | 'server'.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { plainText, userRequestText } from './content.mjs';
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
import { StepOutcome, stepOutcome } from './tool_result.mjs';

const CONTEXT_SESSION_FLAG = ' --session ';
const SOURCE_FLAG = ' --source ';
const OUTPUT_FLAG = ' --output ';
const CONTEXT_OUTPUT_FLAG = ' --context-output ';
const SEPARATE_CONTEXT_LINKS_FLAG = ' --separate-context-links';
const SURFACE_FLAG = ' --surface ';
const TITLE_FLAG = ' --title ';
const BODY_FILE_FLAG = ' --body-file ';
const FORMAL_AI_PROGRAM = 'formal-ai';
const GH_PROGRAM = 'gh';
const PRINTF_PROGRAM = 'printf';
const PRINTF_LINE_FORMAT = " '%s\\n' ";
const AGENTIC_SURFACE = 'agentic-cli';
const BODY_FILE = 'body.md';
const CONTEXT_FILE = 'context.lino';
const LATEST_SESSION = 'latest';
const LANGUAGES = ['en', 'ru', 'hi', 'zh'];

const ROLE_AGENT_ACTION_REPORT_VERB = 'agent_action_report_verb';
const ROLE_AGENT_ACTION_REPORT_SUBJECT = 'agent_action_report_subject';
const ROLE_FILE_WRITE_ACTION_CUE = 'file_write_action_cue';
const ROLE_FILE_WRITE_TARGET_CUE = 'file_write_target_cue';

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
  const acknowledged = messages.slice(reportIndex + 1).some((message) =>
    message.role.toLowerCase() === 'assistant'
    && !(message.tool_calls || []).length
    && trim(plainText(message.content)) !== ''
    && !isReportQuestion(plainText(message.content)));
  if (acknowledged) return null;
  const language = detect(userRequestText(messages[reportIndex].content));
  const choices = answerTexts(messages, reportIndex + 1);
  let targets = null;
  let targetIndex = -1;
  for (const [index, text] of choices) {
    const parsed = parseTargets(text);
    if (parsed.length) {
      targets = parsed;
      targetIndex = index;
      break;
    }
  }
  if (targets === null) {
    return askOrRender(toolNames, language, 'report_target', 'agentic_report_target_question',
      'agentic_report_target_options', true);
  }
  let explicitContents = null;
  for (const [index, text] of choices) {
    if (index <= targetIndex) continue;
    explicitContents = parseContents(text);
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
    return askOrRender(toolNames, language, 'report_contents', 'agentic_report_contents_question',
      'agentic_report_contents_options', false);
  }
  const progress = Progress.scan(messages.slice(reportIndex + 1));
  const dialog = dialogId();
  const commands = commandsForTargets(targets, contents ?? 'both', messages, reportIndex, dialog);
  const command = commands[progress.run_outputs.length];
  if (command === undefined) return finalAnswer(reportFinished(targets, progress.run_outputs, language));
  const tool = toolFor(toolNames, Capability.Run) ?? null;
  if (tool !== null) return planOne(tool, jsonText({ command }));
  return finalAnswer(renderConfig('issue_report_tool_missing', [['repository', formalAiRepo()]]));
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
        multiple,
      }],
    }));
  }
  let text = question;
  options.forEach(([label, description], index) => {
    text += `\n${index + 1}. ${label} — ${description}`;
  });
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
    ['agentic_report_target_question', 'agentic_report_contents_question']
      .some((intent) => text.includes(localized(intent, language))));
}

/** Mirrors `fn answer_texts`: `[index, text]` of each later user/tool message. */
function answerTexts(messages, start) {
  const out = [];
  for (let index = start; index < messages.length; index += 1) {
    const role = messages[index].role.toLowerCase();
    if (role === 'user' || role === 'tool') out.push([index, plainText(messages[index].content)]);
  }
  return out;
}

/** Mirrors `fn parse_targets`. */
function parseTargets(text) {
  return [
    ['harness_log', 0, 'harness_log'],
    ['server_log', 1, 'server_log'],
    ['github_issue', 2, 'github_issue'],
    ['formal_ai', 3, 'formal_ai'],
  ].filter(([, optionIndex, machineValue]) =>
    matchesOption(text, 'agentic_report_target_options', optionIndex, machineValue))
    .map(([target]) => target);
}

/** Mirrors `fn parse_contents`. */
function parseContents(text) {
  if (matchesOption(text, 'agentic_report_contents_options', 0, 'both_logs')) return 'both';
  if (matchesOption(text, 'agentic_report_contents_options', 1, 'harness_log')) return 'harness';
  return matchesOption(text, 'agentic_report_contents_options', 2, 'server_log') ? 'server' : null;
}

/** Mirrors `fn matches_option`. */
function matchesOption(text, intent, optionIndex, machineValue) {
  const normalized = normalizePrompt(text);
  return normalized.includes(normalizePrompt(machineValue))
    || LANGUAGES.some((language) => {
      const option = localizedOptions(intent, language)[optionIndex];
      return option !== undefined && normalized.includes(normalizePrompt(option[0]));
    });
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
  const script = new ReportScript();
  const bodyFile = script.scratch(BODY_FILE);
  const contextFile = script.scratch(CONTEXT_FILE);
  let body = agenticMessage('report_issue_report_body_command') + CONTEXT_SESSION_FLAG + shellQuote(dialog)
    + SOURCE_FLAG + sourceName(contents) + SURFACE_FLAG + AGENTIC_SURFACE + OUTPUT_FLAG + bodyFile
    + CONTEXT_OUTPUT_FLAG + contextFile;
  if (contents === 'both') body += SEPARATE_CONTEXT_LINKS_FLAG;
  script.step(FORMAL_AI_PROGRAM, body);
  const create = agenticMessage('report_issue_issue_create_command') + formalAiRepo() + TITLE_FLAG
    + shellQuote(reportTitle(messages, reportIndex)) + BODY_FILE_FLAG + bodyFile;
  script.step(GH_PROGRAM, create);
  return script.render();
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
  ordered.forEach((target, index) => {
    if (index >= runOutputs.length) return;
    if (stepOutcome(runOutputs[index]) === StepOutcome.Failed) failures.push([target, trim(runOutputs[index])]);
  });
  if (failures.length) {
    const detail = failures.map(([target, output]) => `${targetLabel(target)}: ${output}`).join('\n');
    return `${config('issue_report_failed')}\n\n\`\`\`text\n${detail}\n\`\`\``;
  }
  const trimmed = runOutputs.map(trim).filter((output) => output !== '').join('\n');
  if (targets.includes('github_issue')) {
    const url = splitWhitespace(trimmed).find((token) => token.startsWith('https://') && token.includes('/issues/'));
    if (url !== undefined) return renderConfig('issue_report_created_with_url', [['url', url]]);
    const failed = config('issue_report_failed');
    return trimmed === '' ? failed : `${failed}\n\n\`\`\`text\n${trimmed}\n\`\`\``;
  }
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
