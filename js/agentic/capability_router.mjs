// Capability -> advertised tool routing: the JavaScript twin of
// rust/src/agentic_coding/capability_router.rs.

import { Capability, registryId } from './capability.mjs';
import { fetchArguments, finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { requestBlocks } from './stated_request.mjs';
import {
  governsCommandsRatherThanRequestingOne, sentenceSpans, statesACommandPolicy,
} from './shell_command_policy.mjs';
import {
  codeSearchQueryForTask, codeShapedQuery, explicitPassthroughCommand, namesMutatingShellIntent,
  semanticShellCommandForTask, shellCommandForTask,
} from './shell_command.mjs';
import { hasLatestTurnResult, latestTurnAnswer } from './tool_result.mjs';
import { conceptLookupLeavesUnknown, openWebQueryForBlock } from './web_research.mjs';
import { catalogClaims } from './code_artifact.mjs';
import { composeEditRequest, statedWriteTarget, statesWriteAction } from './write_request.mjs';
import { workspaceInspectionSearchForTask } from './workspace_inspection.mjs';
import { listedDirectory } from './directory_listing.mjs';
import { writesWholeFile } from './literal_write_guard.mjs';
import {
  Act, Locus, ObjectType, acts, evidencesRetrieveAct, explicitContent, firstPath, firstUrl, locus, namesOpenWeb,
  isDialogueUtterance, objectType, route, tableRoutingEnabled,
} from './crate/capability_routing.mjs';
import { extractConceptQuery } from './crate/concepts_lookup.mjs';
import { textOutsideQuotedSegments } from './crate/coding_program_contract.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { mentionsRole } from './write_lexicon.mjs';
import { factStoreResolves } from './crate/solver_handlers_benchmark_prompts.mjs';
import { cleanSearchQuery } from './crate/solver_handlers_web_search.mjs';
import { agenticToolCapabilities } from './crate/seed_agentic_tool_capabilities.mjs';
import { isClientWorkspace, scopeOfToolName } from './crate/tool_scope.mjs';
import { eqIgnoreAsciiCase, minByKey, splitWhitespace, toAsciiLowercase, trimMatches } from './crate/rust_str.mjs';

const registryEntry = (capability) => agenticToolCapabilities().find((entry) => entry.id === registryId(capability)) ?? null;

/**
 * Mirrors `fn tool_for` in rust/src/agentic_coding/capability_router.rs.
 * @param {string[]} toolNames
 * @param {string} capability
 * @returns {string|null}
 */
export function toolFor(toolNames, capability) {
  const inScope = toolNames.filter((name) => actsInCapabilityScope(name, capability));
  if (capability === Capability.Search || capability === Capability.Fetch) {
    const classified = inScope.filter((name) => classifyTool(name) === capability);
    if (classified.length) return minByKey(classified, researchToolRank);
  }
  const entry = registryEntry(capability);
  if (entry === null) return null;
  for (const alias of entry.aliases) {
    const found = inScope.find((name) => eqIgnoreAsciiCase(alias, name));
    if (found !== undefined) return found;
  }
  return inScope.find((name) => classifyTool(name) === capability) ?? null;
}

/** Mirrors `fn shell_command_tool` in rust/src/agentic_coding/capability_router.rs. */
export function shellCommandTool(toolNames) {
  const shell = registryEntry(Capability.Run);
  if (shell === null) return null;
  for (const alias of shell.command_aliases) {
    const found = toolNames.find((name) => actsInCapabilityScope(name, Capability.Run)
      && eqIgnoreAsciiCase(toolLeafName(name), alias));
    if (found !== undefined) return found;
  }
  return null;
}

/** Mirrors `fn tool_leaf_name`. */
function toolLeafName(name) {
  const segment = name.split('__').pop();
  return segment.split(/[./:]/).pop();
}

function actsInCapabilityScope(name, capability) {
  if ([Capability.Search, Capability.Fetch, Capability.Todo, Capability.AskUser].includes(capability)) return true;
  return isClientWorkspace(scopeOfToolName(name));
}

function researchToolRank(name) {
  const hosted = isHostedResearchTool(name);
  const namespaced = toAsciiLowercase(name).startsWith('mcp__');
  const clientScoped = isClientWorkspace(scopeOfToolName(name));
  if (hosted) return 3;
  if (!namespaced) return 2;
  return clientScoped ? 0 : 1;
}

const HOSTED_RESEARCH_TOOLS = ['web_search', 'web_fetch', 'web_search_preview', 'file_search', 'computer_use_preview'];

/** Mirrors `fn is_hosted_research_tool` in rust/src/agentic_coding/capability_router.rs. */
export function isHostedResearchTool(name) {
  return HOSTED_RESEARCH_TOOLS.some((hosted) => eqIgnoreAsciiCase(hosted, name));
}

const BROWSER_INTERACTION_MARKERS = ['browser_', 'click', 'hover', 'drag', 'snapshot', 'screenshot', 'press_key',
  'fill_form', 'select_option', 'handle_dialog', 'file_upload'];

const isBrowserInteractionTool = (lower) => BROWSER_INTERACTION_MARKERS.some((marker) => lower.includes(marker));

function toolMatchesCapability(name, capability) {
  const entry = registryEntry(capability);
  return entry !== null && entry.aliases.some((alias) => eqIgnoreAsciiCase(alias, name));
}

const CLASSIFY_ORDER = [
  Capability.Search, Capability.Fetch, Capability.Read, Capability.Write, Capability.Edit, Capability.Run,
  Capability.Grep, Capability.Glob, Capability.ListDir, Capability.Todo, Capability.Subagent, Capability.ReadMany,
  Capability.MultiEdit, Capability.AskUser,
];

/**
 * Mirrors `fn classify_tool` in rust/src/agentic_coding/capability_router.rs.
 * @param {string} name
 * @returns {string|null}
 */
export function classifyTool(name) {
  for (const capability of CLASSIFY_ORDER) if (toolMatchesCapability(name, capability)) return capability;
  const lower = toAsciiLowercase(name);
  const has = (part) => lower.includes(part);
  if (has('todo') || isBrowserInteractionTool(lower)) return null;
  if (lower === 'computer_use' || lower === 'code_interpreter') return Capability.Run;
  if (has('search')) return has('web') && lower !== 'tool_search' ? Capability.Search : null;
  if (lower === 'read' || has('read_file') || has('read_local_file') || has('file_read') || has('open_file')
    || has('view_file')) {
    return Capability.Read;
  }
  if (has('fetch') || has('open') || has('browse') || has('get_url') || has('read_url')) return Capability.Fetch;
  if (lower === 'write' || lower.endsWith('__write') || has('write_file') || has('file_write') || has('create_file')) {
    return Capability.Write;
  }
  if (has('edit') || has('patch') || has('replace')) return Capability.Edit;
  if (has('run') || has('bash') || has('command') || has('exec') || has('shell')) return Capability.Run;
  return null;
}

/** Mirrors `fn workspace_creation_tool` in rust/src/agentic_coding/capability_router.rs. */
export function workspaceCreationTool(toolNames) {
  return toolFor(toolNames, Capability.Write) ?? toolNames.find(isWorkspaceCreationTool) ?? null;
}

/** Mirrors `fn is_workspace_creation_tool` in rust/src/agentic_coding/capability_router.rs. */
export function isWorkspaceCreationTool(name) {
  if (!isClientWorkspace(scopeOfToolName(name))) return false;
  const capability = classifyTool(name);
  if (capability === Capability.Write) return true;
  const leaf = name.split('__').pop();
  return capability === Capability.Edit && toAsciiLowercase(leaf).includes('patch');
}

/** Mirrors `enum RoutingStage` in rust/src/agentic_coding/capability_router.rs. */
export const RoutingStage = Object.freeze({ NamedOrLocal: 'named_or_local', OpenWeb: 'open_web' });

function namesAContainerWithoutAnAct(task) {
  const taskActs = acts(task);
  return objectType(task)[0] === ObjectType.PathScope
    && taskActs.length === 1 && taskActs[0] === Act.Retrieve
    && !evidencesRetrieveAct(task);
}

const NAMED_OBJECTS = new Set([
  ObjectType.Url, ObjectType.Path, ObjectType.PathScope, ObjectType.QuotedContent, ObjectType.Pattern,
  ObjectType.PathSet, ObjectType.TaskList, ObjectType.Delegation,
]);

/** Mirrors `fn stage_of`. */
function stageOf(task) {
  const highest = objectType(task)[0] ?? ObjectType.None;
  return NAMED_OBJECTS.has(highest) || locus(task) === Locus.Workspace ? RoutingStage.NamedOrLocal : RoutingStage.OpenWeb;
}

const ROUTED_CAPABILITIES = [
  ['web_fetch', Capability.Fetch],
  ['web_search', Capability.Search],
  ['read_file', Capability.Read],
  ['write_file', Capability.Write],
  ['list_dir', Capability.ListDir],
  ['grep', Capability.Grep],
  ['shell', Capability.Run],
  ['glob', Capability.Glob],
  ['read_many', Capability.ReadMany],
  ['multi_edit', Capability.MultiEdit],
  ['todo', Capability.Todo],
  ['subagent', Capability.Subagent],
];

const routedCapability = (slug) => ROUTED_CAPABILITIES.find(([name]) => name === slug)?.[1] ?? null;

/**
 * Mirrors `fn plan_routed_capability_step` in rust/src/agentic_coding/capability_router.rs.
 * @returns {object|null} an AgenticPlan
 */
export function planRoutedCapabilityStep(task, messages, toolNames, stage) {
  return planRoutedCapabilityStepIn(task, messages, toolNames, stage, []);
}

/** Mirrors `OBSERVING_SLUGS`: capabilities that only observe the workspace. */
const OBSERVING_SLUGS = ['read_many', 'list_dir', 'glob', 'grep'];
const NAMED_CAPABILITY_SLUGS = ['grep', 'glob', 'list_dir', 'read_many', 'multi_edit', 'todo', 'subagent'];

/** Mirrors `fn plan_named_capability_step` in rust/src/agentic_coding/capability_router.rs. */
export function planNamedCapabilityStep(task, messages, toolNames) {
  return planRoutedCapabilityStepIn(task, messages, toolNames, RoutingStage.NamedOrLocal, NAMED_CAPABILITY_SLUGS);
}

function planRoutedCapabilityStepIn(task, messages, toolNames, stage, only) {
  const firstBlock = requestBlocks(task)[0] ?? task;
  const policyFreeRequest = sentenceSpans(firstBlock).filter((sentence) => !statesACommandPolicy(sentence)).join(' ');
  const routedTask = policyFreeRequest === '' ? firstBlock : policyFreeRequest;
  if (!tableRoutingEnabled() || stageOf(routedTask) !== stage) return null;
  if (explicitPassthroughCommand(routedTask) !== null) return null;
  const engineAnswerableConcept = extractConceptQuery(routedTask) !== null && !conceptLookupLeavesUnknown(routedTask);
  const engineAnswerableFact = factStoreResolves(routedTask);
  const engineAnswerableProgram = catalogClaims(routedTask);
  const engineAnswerableDialogue = isDialogueUtterance(routedTask);
  if (stage === RoutingStage.OpenWeb && !namesOpenWeb(routedTask)
    && (engineAnswerableConcept || engineAnswerableFact || engineAnswerableProgram || engineAnswerableDialogue)) {
    return null;
  }
  const advertised = ROUTED_CAPABILITIES
    .filter(([, capability]) => toolFor(toolNames, capability) !== null)
    .map(([slug]) => slug);
  const outcome = route(routedTask, advertised);
  let slug;
  let loweredFrom = null;
  if (outcome.kind === 'routed') slug = outcome.capability;
  else if (outcome.kind === 'lowered') [slug, loweredFrom] = [outcome.capability, outcome.preferred];
  else return null;
  const decided = loweredFrom ?? slug;
  if (only.length && !only.includes(decided)) return null;
  const capability = routedCapability(slug);
  if (capability === null) return null;
  if (namesAContainerWithoutAnAct(routedTask)) return null;
  // A mutating shell intent (`rename the file a to b`) is the settled shell
  // arm's verified recipe, never a bare run (PR #1188 G24).
  if ((capability === Capability.ReadMany || capability === Capability.Run || OBSERVING_SLUGS.includes(decided))
    && namesMutatingShellIntent(routedTask)) {
    return null;
  }
  if (hasLatestTurnResult(messages)) {
    const answer = latestTurnAnswer(messages, toolNames, task);
    return answer === null ? null : finalAnswer(answer);
  }
  if (capability === Capability.Run && governsCommandsRatherThanRequestingOne(routedTask)) return null;
  if (capability === Capability.Run && loweredFrom === 'list_dir') {
    const listing = shellFallback(Capability.ListDir, routedTask);
    if (listing !== null) {
      const command = semanticShellCommandForTask(routedTask);
      if (command !== null && !command.startsWith(listing)) return null;
    }
  }
  if (capability === Capability.Read && statesWriteAction(routedTask) && statedWriteTarget(routedTask) !== null) {
    return null;
  }
  // Nor is it one of several files to read (PR #1188 G102).
  if (decided === 'read_many' && statesWriteAction(routedTask)
    && fileTokens(routedTask).includes(statedWriteTarget(routedTask))) {
    return null;
  }
  const tool = toolFor(toolNames, capability);
  if (tool === null) return null;
  const args = routedArguments(capability, loweredFrom, routedTask);
  return args === null ? null : planOne(tool, args);
}

function loweredSearchCommand(preferred, task) {
  if (preferred !== Capability.Grep) return null;
  const query = codeShapedQuery(task) ?? codeSearchQueryForTask(task);
  return query === null ? null : `rg -n ${shellQuote(query)}`;
}

function routedArguments(capability, loweredFrom, task) {
  if (capability === Capability.Run) {
    const preferred = (loweredFrom === null ? null : routedCapability(loweredFrom)) ?? Capability.Run;
    const command = shellFallback(preferred, task) ?? shellCommandForTask(task) ?? loweredSearchCommand(preferred, task);
    return command === null ? null : jsonText({ command });
  }
  switch (capability) {
    case Capability.Fetch: {
      const url = firstUrl(task);
      return url === null ? null : fetchArguments(url);
    }
    case Capability.Search: {
      let query = null;
      for (const block of requestBlocks(task)) {
        query = openWebQueryForBlock(block);
        if (query !== null) break;
      }
      return jsonText({ query: query ?? cleanSearchQuery(task) });
    }
    case Capability.Read: {
      const path = firstPath(task);
      return path === null ? null : jsonText({ path, filePath: path, file_path: path });
    }
    case Capability.Write: {
      // A removal never writes new content over the file (PR #1188 T29: a
      // line deletion was routed here and the file became one quoted line),
      // and neither does an edit the edit composer could not read: unless the
      // request states a whole-file write, its edit word asks for a change
      // inside the file (U17: `In notes.txt replace with ','` wrote `','`).
      const outside = normalizePrompt(textOutsideQuotedSegments(task));
      if (mentionsRole('coding_text_remove_action', outside)) return null;
      if (mentionsRole('file_edit_action_cue', outside) && !writesWholeFile(task)) return null;
      const path = firstPath(task);
      if (path === null) return null;
      const content = explicitContent(task);
      return content === null ? null : writeArguments(path, content);
    }
    case Capability.MultiEdit: {
      const edit = composeEditRequest(task);
      if (!edit) return null;
      const [path, old, replacement] = edit;
      return jsonText({
        path,
        paths: fileTokens(task),
        edits: [{ old, new: replacement, old_string: old, new_string: replacement }],
      });
    }
    case Capability.Grep: {
      const search = workspaceInspectionSearchForTask(task);
      if (search === null) return argumentsFor(capability, task);
      const args = { query: search.query, pattern: search.pattern };
      if (search.include !== null && search.include !== undefined) args.include = search.include;
      return jsonText(args);
    }
    default:
      return argumentsFor(capability, task);
  }
}

function argumentsFor(capability, task) {
  switch (capability) {
    case Capability.Grep: {
      const query = codeSearchQueryForTask(task) ?? task;
      return jsonText({ query, pattern: query });
    }
    case Capability.Glob:
      return jsonText({ pattern: wildcardToken(task) ?? '*', path: '.' });
    case Capability.ListDir:
      return jsonText({ path: listedDirectory(task) });
    case Capability.Todo:
      return jsonText({ todos: [{ content: task, status: 'pending' }], plan: [{ step: task, status: 'pending' }] });
    case Capability.Subagent:
      return jsonText({ description: task, prompt: task, input: task, subagent_type: 'general' });
    case Capability.AskUser:
      return '';
    case Capability.ReadMany: {
      const paths = fileTokens(task);
      return jsonText({ paths, file_paths: paths });
    }
    default:
      return jsonText({ prompt: task });
  }
}

const GLOB_TRIM = new Set([',', ';', ':', '`', '"', "'"]);
const FILE_TRIM = new Set([',', ';', ':', '`', '"', "'", '(', ')']);

function wildcardToken(task) {
  return splitWhitespace(task)
    .map((token) => trimMatches(token, (character) => GLOB_TRIM.has(character)))
    .find((token) => token.includes('*') || token.includes('?') || token.includes('[')) ?? null;
}

/**
 * Marks that a path never holds: a token that still holds one after trimming
 * is a fragment of quoted text or code (`fn(«x»`), never a file to read
 * (PR #1188 G102).
 */
const FRAGMENT_MARKS = '«»“”‘’()[]{}<>`"\'';

function fileTokens(task) {
  return splitWhitespace(task)
    .map((token) => trimMatches(token, (character) => FILE_TRIM.has(character)))
    .filter((token) => token.includes('.') && !token.startsWith('.') && !token.endsWith('.') && !token.includes('//'))
    .filter((token) => !Array.from(token).some((character) => FRAGMENT_MARKS.includes(character)));
}

function shellFallback(capability, task) {
  switch (capability) {
    case Capability.Grep:
      return shellCommandForTask(task);
    case Capability.Glob: {
      const pattern = (wildcardToken(task) ?? '*').split("'").join("'\\''");
      return `find . -path '${pattern}'`;
    }
    case Capability.ListDir: {
      const directory = listedDirectory(task);
      return directory === '.' ? 'ls' : `ls ${shellQuote(directory)}`;
    }
    case Capability.ReadMany: {
      const paths = fileTokens(task);
      return paths.length ? `cat ${paths.map(shellQuote).join(' ')}` : null;
    }
    default:
      return null;
  }
}

const shellQuote = (value) => `'${value.split("'").join("'\\''")}'`;
