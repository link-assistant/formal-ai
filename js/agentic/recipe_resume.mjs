import { recipeArtifactEvidenceRoot, recipeArtifactInventoryCommand, recipeArtifactInventoryResult } from './recipe_artifact_inventory.mjs';
import { repositoryWorkReference } from './general_planner.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { fill } from './work_item_steps.mjs';
// Client-owned source probes; mirrors rust/src/agentic_coding/recipe_resume.rs.
// A successful probe proves bytes at its observed epoch, never issue completion.
import { cached, childrenNamed, childValue, parseLino, readText } from './host.mjs';
import { shellQuote } from './git_commit.mjs';
import { observedPayload, stepOutcome, StepOutcome, commandArgument, reportedExitCode, incompleteReceipt } from './tool_result.mjs';
import { Capability } from './capability.mjs';
import { qualifiedToolAttempt, qualifiedCommandAttempt, Progress } from './progress.mjs';
import { toolFor } from './capability_router.mjs';
import { preparedRecipeHygiene, preparedWorkCommitCommand, preparedWorkPaths } from './restart_feedback.mjs';
import { planOne, jsonText, resolvedFinalAnswer, FinalDisposition } from './plan.mjs';
function policy() {
  return cached('recipe-resume-policy', () => {
    const root = parseLino(readText('data/meta/recipe-resume.lino'));
    const library = parseLino(readText('data/meta/workspace-source-io.lino'));
    return {
      command: childValue(root, 'command'),
      deliveryPatterns: childValue(root, 'prepared-pull-request-patterns'),
      statusCommand: childValue(root, 'status-command'),
      ciCommand: childValue(root, 'ci-command'),
      verified: childValue(root, 'ci-verified-report'),
      verifiedLocal: childValue(root, 'ci-verified-local-report'),
      stagedPrefix: childValue(root, 'staged-workflow-prefix'),
      stagedLocal: childValue(root, 'staged-local-report'),
      stagedUnverified: childValue(root, 'staged-unverified-report'),
      ciSource: [...childrenNamed(parseLino(readText('data/meta/bounded-command-capture-source.lino')), 'line'), ...childrenNamed(root, 'ci-source-line')].map(node => node.value).join('\n') + '\n',
      pending: childValue(root, 'ci-pending-report'),
      refusal: childValue(root, 'scope-refusal'),
      updateSource: [...childrenNamed(library, 'source-line'), ...childrenNamed(root, 'update-source-line')].map(node => node.value).join('\n') + '\n',
      source: [...childrenNamed(library, 'source-line'), ...childrenNamed(root, 'source-line')].map(node => node.value).join('\n') + '\n'
    };
  });
}

/** Mirrors `fn recipe_source_command`; constructor-owned relative path only. */
export function recipeSourceCommand(path) {
  const installed = policy();
  return installed.command.replace(/\{(?:source|plan)\}/gu, slot => shellQuote(slot === '{source}' ? installed.source : jsonText({
    path
  })));
}

/** Mirrors `fn recipe_source_result`; caller must first bind the exact command. */
export function recipeSourceResult(output, path) {
  const payload = observedPayload(output);
  if (payload === null) return null;
  let value;
  try {
    value = JSON.parse(payload);
  } catch {
    return null;
  }
  if (value === null || typeof value !== 'object' || Array.isArray(value) || value.schema !== 'workspace-source-v1' || value.path !== path || value.success !== true || value.complete !== true) return null;
  if (value.absent === true && value.error_code === 'ENOENT' && !('source' in value)) {
    return {
      absent: true,
      source: null
    };
  }
  if (value.absent === false && typeof value.source === 'string' && !('error_code' in value)) {
    return {
      absent: false,
      source: value.source,
      sha256: value.sha256 ?? null,
      identity: value.identity ?? null
    };
  }
  return null;
}

/** Source constructors decode semantic receipts; missing process status stays unknown. */
export function recipeSourceReceipt(progress, command) {
  const frame = qualifiedCommandAttempt(progress, command);
  const exit = frame === null ? null : reportedExitCode(frame.detail);
  if (frame?.binding !== 'exact' || !frame.succeeded || incompleteReceipt(frame.detail)
    || exit !== null && exit !== 0) return null;
  return frame.detail;
}

/** Mirrors `fn recipe_source_matches`; later writes invalidate an earlier probe. */
export function recipeSourceMatches(progress, path, source) {
  const command = recipeSourceCommand(path);
  const receipt = recipeSourceReceipt(progress, command);
  if (receipt === null) return false;
  const result = recipeSourceResult(receipt, path);
  if (result?.absent !== false || result.source !== source) return false;
  const probe = progress.attempts.findLastIndex(attempt => attempt.capability === Capability.Run && attempt.arguments !== null && commandArgument(attempt.arguments) === command);
  const write = progress.latestSourceMutationIndex(path);
  return probe >= 0 && (write === null || write < probe);
}

/** Mirrors `fn recipe_ci_command`; source-projected bounded kernel, no copied poller. */
export function recipeCiCommand(reference, workflow, ready) {
  return ciInvocation({reference, workflow, ready});
}
function ciInvocation(plan) {
  const installed = policy();
  return installed.ciCommand.replace(/\{(?:source|plan)\}/gu, slot => shellQuote(slot === '{source}' ? installed.ciSource : jsonText(plan)));
}
/** Mirrors `fn staged_ready_command`; staged readiness grants no hosted-CI credit. */
export function recipeStagedReadyCommand(reference, workflow, file, ready) {
  return ciInvocation({reference,workflow,ready,operation:'staged-ready',path:file.path,source:file.source});
}
function receiptIdentity(value,reference,workflow,ready) {
  return value?.success===true&&value.complete===true&&value.reference===reference&&value.workflow===workflow
    &&value.ready===ready&&typeof value.isDraft==='boolean'&&typeof value.head==='string'&&/^[a-f0-9]{40}$/u.test(value.head)
    &&typeof value.branch==='string'&&value.branch!=='';
}
/** Mirrors `fn staged_ready_result`; never substitutes for positive hosted checks. */
export function recipeStagedReadyResult(output,reference,workflow,path,ready) {
  let value;try{value=JSON.parse(observedPayload(output));}catch{return null;}
  return receiptIdentity(value,reference,workflow,ready)&&value.schema==='recipe-staged-ready-v1'
    &&value.path===path&&value.checks===0&&value.ciVerified===false?value:null;
}

/** Mirrors `fn recipe_ci_result`; exact command qualification belongs to the caller. */
export function recipeCiResult(output, reference, workflow, ready) {
  const payload = observedPayload(output);
  let value;
  try {
    value = JSON.parse(payload);
  } catch {
    return null;
  }
  if (!receiptIdentity(value,reference,workflow,ready)||value.schema!=='recipe-ci-v1'||!Number.isSafeInteger(value.checks)||value.checks<=0) return null;
  return value;
}

/** Mirrors `fn plan_recipe_ci_fallback`; only the admitted recipe invokes this completion path. */
export function planRecipeCiFallback(messages, tools, recipe, reference, workflow, followup, localVerified = false, commitReference = reference) {
  const installed = policy();
  const gap = answer => resolvedFinalAnswer(answer, FinalDisposition.Gap, 'recipe-local-unverified');
  const run = toolFor(tools, Capability.Run);
  if (run === null) return gap(installed.refusal);
  const progress = Progress.scan(messages);
  const observed = command => {
    const frame = qualifiedCommandAttempt(progress, command);
    if (frame === null) return {
      kind: 'missing'
    };
    return frame.binding === 'exact' && frame.succeeded && !incompleteReceipt(frame.detail) && stepOutcome(frame.detail) !== StepOutcome.Failed ? {
      kind: 'success',
      output: frame.detail
    } : {
      kind: 'failed'
    };
  };
  const call = command => planOne(run, jsonText({
    command
  }));
  const status = observed(installed.statusCommand);
  if (status.kind === 'missing') return call(installed.statusCommand);
  if (status.kind !== 'success') return gap(installed.refusal);
  const paths = preparedWorkPaths(status.output);
  if (paths === null) return gap(installed.refusal);
  const sourcePaths = [recipe.path, ...recipe.supporting_files.map(file => file.path)];
  const evidenceRoot = recipeArtifactEvidenceRoot(commitReference);
  const inventoryCommand = evidenceRoot === null ? null : recipeArtifactInventoryCommand(sourcePaths, evidenceRoot, commitReference);
  if (inventoryCommand === null) return gap(installed.refusal);
  const inventoryObserved = observed(inventoryCommand);
  if (inventoryObserved.kind === 'missing') return call(inventoryCommand);
  const inventory = inventoryObserved.kind === 'success' ? recipeArtifactInventoryResult(inventoryObserved.output, sourcePaths, evidenceRoot, commitReference) : null;
  if (inventory === null) return gap(installed.refusal);
  if (paths.length !== 0 || inventory.ownedPaths.length !== 0) {
    const hygiene = preparedRecipeHygiene(sourcePaths, paths, inventory, commitReference);
    if (hygiene === null) return gap(installed.refusal);
    const commands = [hygiene.command, preparedWorkCommitCommand(hygiene, commitReference), hygiene.cleanCommand];
    for (const command of commands) {
      const result = observed(command);
      if (result.kind === 'failed') return gap(installed.refusal);
      if (result.kind === 'missing') return call(command);
    }
  }
  const staged = recipe.supporting_files.find(file=>installed.stagedPrefix!==''&&file.path.startsWith(installed.stagedPrefix)
    && /\.ya?ml$/u.test(file.path) && /^name: ([^\r\n]+)$/mu.exec(file.source)?.[1]===workflow);
  if(staged){
    if(followup[0]!==undefined){
      const review=observed(followup[0]);
      if(review.kind==='missing')return call(followup[0]);if(review.kind==='failed')return gap(installed.refusal);
    }
    const command=recipeStagedReadyCommand(reference,workflow,staged,false),view=observed(command);
    if(view.kind==='missing')return call(command);
    const before=view.kind==='success'?recipeStagedReadyResult(view.output,reference,workflow,staged.path,false):null;
    if(before===null)return gap(installed.refusal);
    if(before.isDraft&&followup[1]!==undefined){
      const edit=observed(followup[1]);
      if(edit.kind==='missing')return call(followup[1]);if(edit.kind==='failed')return gap(installed.refusal);
    }
    const readyCommand=recipeStagedReadyCommand(reference,workflow,staged,true),ready=observed(readyCommand);
    if(ready.kind==='missing')return call(readyCommand);
    const after=ready.kind==='success'?recipeStagedReadyResult(ready.output,reference,workflow,staged.path,true):null;
    if(after===null||after.isDraft||after.head!==before.head)return gap(installed.refusal);
    return resolvedFinalAnswer((localVerified?installed.stagedLocal:installed.stagedUnverified).replace('{head}',()=>after.head),
      FinalDisposition.Gap,'recipe-staged-workflow-pending');
  }
  const ciCommand = recipeCiCommand(reference, workflow, false);
  const ci = observed(ciCommand);
  if (ci.kind === 'missing') return call(ciCommand);
  if (ci.kind !== 'success') return gap(installed.pending);
  const evidence = recipeCiResult(ci.output, reference, workflow, false);
  if (evidence === null) return gap(installed.refusal);
  const comments = followup[0];
  if (comments !== undefined) {
    const result = observed(comments);
    if (result.kind === 'missing') return call(comments);
    if (result.kind === 'failed') return gap(installed.refusal);
  }
  if (evidence.isDraft) {
    if (followup[1] !== undefined) {
      const result = observed(followup[1]);
      if (result.kind === 'missing') return call(followup[1]);
      if (result.kind === 'failed') return gap(installed.refusal);
    }
  }
  const readyCommand = recipeCiCommand(reference, workflow, true);
  const ready = observed(readyCommand);
  if (ready.kind === 'missing') return call(readyCommand);
  const confirmed = ready.kind === 'success' ? recipeCiResult(ready.output, reference, workflow, true) : null;
  if (confirmed === null || confirmed.isDraft || confirmed.head !== evidence.head) return gap(installed.pending);
  const report = (localVerified ? installed.verifiedLocal : installed.verified).replace('{head}', () => evidence.head);
  return resolvedFinalAnswer(report, FinalDisposition.Finding, 'recipe-head-ci-verified');
}

/** Mirrors `fn recipe_delivery_reference`; only an explicit latest-user handoff may retarget delivery. */
export function recipeDeliveryReference(request, requirementsReference) {
  const installed = policy();
  const patterns = JSON.parse(installed.deliveryPatterns).map(pattern => new RegExp(pattern, 'iu'));
  const fences = JSON.parse(fill('owned-clause-fence-prefixes', []));
  const quoted = quotedSegmentSpans(request);
  const references = new Set();
  let fenced = false,
    offset = 0;
  for (const raw of request.split('\n')) {
    const start = offset;
    offset += raw.length + 1;
    const line = raw.trim();
    if (fences.some(prefix => line.startsWith(prefix))) {
      fenced = !fenced;
      continue;
    }
    const ownedAt = start + raw.indexOf(line);
    if (fenced || quoted.some(segment => segment.start <= ownedAt && ownedAt < segment.end)) continue;
    for (const pattern of patterns) {
      const matched = pattern.exec(line);
      if (matched === null) continue;
      const reference = repositoryWorkReference(matched[1]);
      const segments = reference?.split('/') ?? [];
      const requirements = requirementsReference?.split('/') ?? [];
      if (reference !== matched[1] || segments[5] !== 'pull' || !/^[0-9]+$/u.test(segments[6] ?? '') || !/[1-9]/u.test(segments[6]) || segments[3] !== requirements[3]
    || segments[4] !== requirements[4]) return {
        kind: 'refused',
        reason: installed.refusal
      };
      references.add(reference);
    }
  }
  if (references.size > 1) return {
    kind: 'refused',
    reason: installed.refusal
  };
  return references.size === 0 ? {
    kind: 'absent'
  } : {
    kind: 'bound',
    reference: [...references][0]
  };
}

/** Mirrors fn recipe_source_refusal: stale source bytes cannot certify a later recipe step. */
export function recipeSourceRefusal() {
  return resolvedFinalAnswer(policy().refusal, FinalDisposition.Gap, 'recipe-source-epoch-invalidated');
}

/** Mirrors fn recipe_source_update_command; caller owns full recipe and exact probe. */
export function recipeSourceUpdateCommand(path, source, before) {
  if (before?.absent !== false || typeof source !== 'string' || !/^[a-f0-9]{64}$/u.test(before.sha256 ?? '') || !Array.isArray(before.identity) || before.identity.length !== 5
    || !before.identity.every(value => typeof value === 'string' && /^[0-9]+$/u.test(value))) return null;
  const installed = policy();
  return installed.command.replace(/\{(?:source|plan)\}/gu, slot => shellQuote(slot === '{source}' ? installed.updateSource : jsonText({
    path,
    source,
    before: {
      sha256: before.sha256,
      identity: before.identity
    }
  })));
}

/** Mirrors fn recipe_source_update_result; exact command binding remains required. */
export function recipeSourceUpdateResult(output, path) {
  let value;
  try {
    value = JSON.parse(observedPayload(output));
  } catch {
    return false;
  }
  return value?.schema === 'workspace-source-update-v1' && value.path === path && value.success === true && value.complete === true;
}
