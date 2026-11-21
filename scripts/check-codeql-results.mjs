#!/usr/bin/env node
// Analyze/upload success establishes delivery, not the absence of findings.
import { readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

// Keep every untrusted value on one log line, including paths and rule IDs.
const quote = (value) => JSON.stringify(
  // JSON objects can shadow toString/valueOf with non-callable properties.
  // Serialize them directly so malformed fields cannot abort aggregation.
  value !== null && typeof value === 'object' ? JSON.stringify(value) : String(value ?? '?')
);

function reportsIn(directory, problems) {
  let entries;
  try { entries = readdirSync(directory, { withFileTypes: true }); }
  catch (error) {
    problems.push(`Unreadable SARIF directory: ${quote(directory)}: ${quote(error.message)}`);
    return [];
  }
  return entries.flatMap((entry) => {
    const file = path.join(directory, entry.name);
    if (entry.isSymbolicLink()) {
      problems.push(`Unexpected SARIF symlink: ${quote(file)}`);
      return [];
    }
    if (entry.isDirectory()) return reportsIn(file, problems);
    return entry.isFile() && entry.name.endsWith('.sarif') ? [file] : [];
  });
}

function describe(item) {
  const location = item.locations?.[0]?.physicalLocation;
  return `${quote(location?.artifactLocation?.uri)}:${quote(location?.region?.startLine)}: ` +
    quote(item.message.text ?? item.message.markdown);
}

function hasMessage(item) {
  return typeof item?.message?.text === 'string' || typeof item?.message?.markdown === 'string';
}

// SARIF defaults an omitted notification level to warning. Even an otherwise
// successful invocation can contain extraction/configuration diagnostics.
function invocationDiagnostics(invocation, context, problems) {
  for (const field of ['toolExecutionNotifications', 'toolConfigurationNotifications']) {
    if (!Object.hasOwn(invocation, field)) continue;
    if (!Array.isArray(invocation[field])) {
      problems.push(`Malformed SARIF notification array: ${context}, ${field}`);
      continue;
    }
    for (const [index, notification] of invocation[field].entries()) {
      if (!hasMessage(notification) ||
          (Object.hasOwn(notification, 'level') && !['error', 'warning', 'note', 'none'].includes(notification.level))) {
        problems.push(`Malformed SARIF notification: ${context}, ${field}, item ${index + 1}`);
        continue;
      }
      const level = notification.level ?? 'warning';
      if (level === 'note' || level === 'none') continue;
      problems.push(`Incomplete CodeQL analysis diagnostic: ${context}, ${field}: ` +
        `${quote(notification.descriptor?.id ?? notification.descriptor?.index)} (${quote(level)}) ${describe(notification)}`);
    }
  }
}

export function checkCodeqlResults(directory) {
  const problems = [];
  const reports = reportsIn(directory, problems).sort();
  if (!reports.length) problems.push(`No SARIF reports in ${quote(directory)}`);
  for (const file of reports) {
    let data;
    try { data = JSON.parse(readFileSync(file, 'utf8')); }
    catch (error) {
      problems.push(`Invalid SARIF report: ${quote(file)}: ${quote(error.message)}`);
      continue;
    }
    if (data?.version !== '2.1.0' || !Array.isArray(data.runs) || !data.runs.length) {
      problems.push(`Invalid SARIF report: ${quote(file)}`);
      continue;
    }
    for (const [index, run] of data.runs.entries()) {
      const context = `${quote(file)}, run ${index + 1}`;
      if (run?.tool?.driver?.name !== 'CodeQL' || !Array.isArray(run.results) ||
          !Array.isArray(run.invocations) || !run.invocations.length) {
        problems.push(`Missing or unsuccessful CodeQL analysis: ${context}`);
      }
      for (const [invocationIndex, invocation] of (Array.isArray(run?.invocations) ? run.invocations : []).entries()) {
        const at = `${context}, invocation ${invocationIndex + 1}`;
        if (invocation?.executionSuccessful !== true) problems.push(`Missing or unsuccessful CodeQL analysis: ${at}`);
        if (invocation && typeof invocation === 'object') invocationDiagnostics(invocation, at, problems);
      }
      for (const [resultIndex, result] of (Array.isArray(run?.results) ? run.results : []).entries()) {
        if (typeof result?.ruleId !== 'string' || !hasMessage(result)) {
          problems.push(`Malformed SARIF result: ${context}, item ${resultIndex + 1}`);
          continue;
        }
        if (result.baselineState === 'absent') continue;
        problems.push(`Unresolved CodeQL finding: ${context}: ${quote(result.ruleId)} ` +
          `(${quote(result.level ?? 'warning')}) ${describe(result)}`);
      }
    }
  }
  if (problems.length) throw new Error(`${problems.length} CodeQL gate problem(s):\n${problems.join('\n')}`);
  return reports.length;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  try {
    console.log(`CodeQL gate: ${checkCodeqlResults(process.argv[2] ?? 'codeql-results')} report(s), no unresolved findings`);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
