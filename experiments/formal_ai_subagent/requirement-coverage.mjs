#!/usr/bin/env node
// Renders docs/case-studies/pull-request-1188/requirement-coverage.md: every
// requirement of pull request #1188 mapped to the requirement rows that cover
// it, with each row's computed status and an evidence check.
//
// Inputs:
// - experiments/formal_ai_subagent/requirement-coverage.lino: the owner's
//   requirements, their sources and the rows they map to (judgement), the
//   issues the pull request fixes, and the documents found contradicting the
//   latest vision;
// - docs/requirements/*.md: the rows themselves (issue items are read from the
//   fixed issues' shards: row R<issue>-<n> is the issue's item R<n>);
// - data/meta/requirement-status-ledger/*.lino: the generated verdicts.
//
// The evidence check re-reads every implemented row in scope: each repository
// path the row cites must exist, and each snake_case name it cites (a test
// function, a gate) must occur somewhere in the code.
//
// Usage:
//   node experiments/formal_ai_subagent/requirement-coverage.mjs [--write|--check]

import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const dataPath = join(repositoryRoot, 'experiments/formal_ai_subagent/requirement-coverage.lino');
const outputRelative = 'docs/case-studies/pull-request-1188/requirement-coverage.md';
const shardDirectory = 'docs/requirements';
const ledgerDirectory = 'data/meta/requirement-status-ledger';
const draftedPrefix = 'R1188-U';
const checkMode = process.argv.includes('--check');

const groupTitles = {
  architecture: 'Architecture',
  naming: 'Naming',
  notation: 'Notation',
  'ci-speed': 'CI speed',
  'formal-ai-delegation': 'Formal AI delegation',
  'docs-sync': 'Docs sync',
  'javascript-first-parity': 'JS-first parity',
  translation: 'Translation',
  safety: 'Safety',
  process: 'Process',
};

/** Splits a links-notation line into words, keeping quoted strings whole. */
function words(line) {
  return [...line.matchAll(/"((?:[^"\\]|\\.)*)"|(\S+)/g)].map((match) => (match[1] ?? match[2]));
}

/** Reads the coverage data: top-level records with indented two-space fields. */
function readCoverageData() {
  const data = { fixes: [], sources: new Map(), requirements: [], contradictions: [], reviewed: new Map() };
  let current = null;
  for (const raw of readFileSync(dataPath, 'utf8').split('\n')) {
    if (!raw.trim() || raw.trimStart().startsWith('#')) continue;
    const [head, ...rest] = words(raw);
    if (!raw.startsWith(' ')) {
      if (head === 'source') {
        data.sources.set(rest[0], rest[1]);
        current = null;
      } else if (head === 'requirement') {
        current = { identifier: rest[0], sources: [], rows: [] };
        data.requirements.push(current);
      } else if (head === 'reviewed') {
        data.reviewed.set(rest[0], rest[1]);
        current = null;
      } else if (head === 'contradiction') {
        current = { identifier: rest[0] };
        data.contradictions.push(current);
      } else {
        current = { kind: head };
      }
      continue;
    }
    if (head === 'fixes') data.fixes.push(...rest.map(Number));
    else if (head === 'source') current.sources.push({ source: rest[0], when: rest[1] });
    else if (head === 'rows') current.rows.push(...rest);
    else current[head] = rest.join(' ');
  }
  return data;
}

/** Reads every requirement row of the shards: id -> shard, requirement and status cells. */
function readRows() {
  const rows = new Map();
  const shards = readdirSync(join(repositoryRoot, shardDirectory)).filter((name) => name.endsWith('.md')).sort();
  for (const shard of shards) {
    for (const line of readFileSync(join(repositoryRoot, shardDirectory, shard), 'utf8').split('\n')) {
      const match = /^\| (R[0-9][A-Za-z0-9_-]*) \|/.exec(line);
      if (!match || rows.has(match[1])) continue;
      const cells = line.replace(/^\||\|$/g, '').split('|');
      rows.set(match[1], {
        shard: `${shardDirectory}/${shard}`,
        requirement: (cells[1] ?? '').trim(),
        status: cells.slice(2).join('|').trim(),
      });
    }
  }
  return rows;
}

/** Reads the generated status ledger: id -> verdict and automated test. */
function readLedger() {
  const ledger = new Map();
  const directory = join(repositoryRoot, ledgerDirectory);
  for (const name of readdirSync(directory).filter((file) => file.endsWith('.lino')).sort()) {
    let record = null;
    for (const line of readFileSync(join(directory, name), 'utf8').split('\n')) {
      const field = /^ {4}(\w+) "(.*)"$/.exec(line);
      if (!field) continue;
      if (field[1] === 'id') {
        record = { verdict: '', automatedTest: '' };
        ledger.set(field[2], record);
      } else if (record && field[1] === 'verdict') record.verdict = field[2];
      else if (record && field[1] === 'automated_test') record.automatedTest = field[2];
    }
  }
  return ledger;
}

const indexedDirectories = ['rust/src', 'rust/tests', 'js', 'ts', 'scripts', 'data', '.github', 'tests', 'desktop', 'experiments'];
const indexedExtensions = /\.(rs|mjs|js|ts|tsx|jsx|sh|py|lino|yml|yaml|json|toml|md)$/;
const skippedDirectories = new Set(['node_modules', 'target', 'sandboxes', '.git', 'dist']);

/** Every snake_case name that occurs in the code, gates and data. */
function codeNames() {
  const names = new Set();
  const visit = (directory) => {
    let entries;
    try {
      entries = readdirSync(directory, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) {
        if (!skippedDirectories.has(entry.name)) visit(path);
      } else if (indexedExtensions.test(entry.name) && path !== dataPath && statSync(path).size < 4_000_000) {
        for (const match of readFileSync(path, 'utf8').matchAll(/[A-Za-z0-9_]+/g)) {
          if (match[0].includes('_')) names.add(match[0]);
        }
      }
    }
  };
  for (const directory of indexedDirectories) visit(join(repositoryRoot, directory));
  return names;
}

const pathRoots = /^(?:\.\.\/)*(rust|js|ts|scripts|data|docs|\.github|tests|desktop|experiments)\//;

/** The repository paths and snake_case names a status cell cites. */
function citations(status) {
  const spans = [...status.matchAll(/`([^`]+)`/g)].map((match) => match[1].trim());
  const links = [...status.matchAll(/\]\(([^)\s#]+)/g)].map((match) => match[1]);
  const paths = new Set();
  const names = new Set();
  // A test title is checked only where the row ties it to a test file:
  // "`title` in `file`" or "`file` (*title*, …)".
  const titles = [];
  for (const match of status.matchAll(/`([^`]+)`\s+in\s+`([^`]+(?:\.test\.mjs|\.spec\.js|\.rs))`/g)) {
    titles.push({ title: match[1].trim().replace(/^fn /, ""), file: match[2] });
  }
  for (const match of status.matchAll(/`([^`]+\.test\.mjs)`\s*\(\*([^*]+)\*/g)) {
    titles.push({ title: match[2].trim(), file: match[1] });
  }
  for (const span of [...spans, ...links]) {
    const token = span.split(/\s/)[0].replace(/[:,;.]+$/, '');
    if (/[*<>{}?$]/.test(token)) continue;
    if (pathRoots.test(token) && /\.[a-z]+$/.test(token.replace(/::.*$/, ''))) paths.add(token.replace(/::.*$/, ''));
    else if (/^[a-z][a-z0-9]*(?:_[a-z0-9]+){2,}$/.test(span)) names.add(span);
  }
  return { paths: [...paths], names: [...names], titles };
}

function pathExists(path, shard) {
  const candidates = [path, join('rust', path), normalize(join(dirname(shard), path))];
  return candidates.some((candidate) => !candidate.startsWith('..') && existsSync(join(repositoryRoot, candidate)));
}

function statusOf(rowIdentifier, ledger, rows) {
  if (!rows.has(rowIdentifier)) return 'missing row';
  return ledger.get(rowIdentifier)?.verdict || 'not in ledger';
}

const escapeCell = (text) => text.replace(/\|/g, '\\|').replace(/\n/g, ' ');

function firstSentence(text, limit = 160) {
  const plain = text.replace(/\*\*/g, '').trim();
  const sentence = /^(.+?[.!?])(\s|$)/.exec(plain)?.[1] ?? plain;
  return sentence.length > limit ? `${sentence.slice(0, limit - 1).trimEnd()}…` : sentence;
}

function render() {
  const data = readCoverageData();
  const rows = readRows();
  const ledger = readLedger();
  const linkTo = (path) => `../../../${path}`;

  const issueRequirements = [];
  for (const issue of data.fixes) {
    const pattern = new RegExp(`^R${issue}-([0-9]+)$`);
    const owned = [...rows.keys()].filter((identifier) => pattern.test(identifier))
      .sort((left, right) => Number(pattern.exec(left)[1]) - Number(pattern.exec(right)[1]));
    for (const identifier of owned) {
      issueRequirements.push({ issue, item: `R${pattern.exec(identifier)[1]}`, row: identifier });
    }
  }

  const ownerCovered = data.requirements.filter((requirement) => requirement.rows.some((row) => !row.startsWith(draftedPrefix)));
  const ownerDrafted = data.requirements.filter((requirement) => requirement.rows.some((row) => row.startsWith(draftedPrefix)));
  const ownerOnlyDrafted = data.requirements.filter((requirement) => requirement.rows.every((row) => row.startsWith(draftedPrefix)));
  const draftedRows = [...rows.keys()].filter((identifier) => identifier.startsWith(draftedPrefix));

  const scopedRows = new Set([...data.requirements.flatMap((requirement) => requirement.rows), ...issueRequirements.map((entry) => entry.row)]);
  const verdictCounts = new Map();
  for (const identifier of scopedRows) {
    const verdict = statusOf(identifier, ledger, rows);
    verdictCounts.set(verdict, (verdictCounts.get(verdict) ?? 0) + 1);
  }

  const names = codeNames();
  const evidenceProblems = [];
  const implementedInScope = [...scopedRows].filter((identifier) => ledger.get(identifier)?.verdict === 'implemented').sort();
  for (const identifier of implementedInScope) {
    const row = rows.get(identifier);
    const { paths, names: citedNames, titles } = citations(row.status);
    const missingPaths = paths.filter((path) => !pathExists(path, row.shard));
    const missingNames = citedNames.filter((name) => !names.has(name));
    const missingTitles = titles.filter(({ title, file }) => {
      const found = [file, join('rust', file)].find((candidate) => existsSync(join(repositoryRoot, candidate)));
      return found && !readFileSync(join(repositoryRoot, found), 'utf8').includes(title);
    }).map(({ title, file }) => `${title} (in ${file})`);
    const reviewed = data.reviewed.get(identifier);
    if (missingPaths.length || missingNames.length || missingTitles.length) {
      evidenceProblems.push({ identifier, missingPaths, missingNames, missingTitles, reviewed });
    }
  }

  const lines = [
    '# Requirement coverage for pull request #1188',
    '',
    'Generated by `node experiments/formal_ai_subagent/requirement-coverage.mjs --write`',
    'from `experiments/formal_ai_subagent/requirement-coverage.lino` (the mapping, which is',
    'judgement), the requirement shards and the generated status ledger (the statuses,',
    'which are computed). `--check` regenerates and compares. Do not edit by hand.',
    '',
    'Sources: the owner\'s messages in the Claude Code sessions (as recorded in the',
    'architect notes, the feedback-recovery snapshots and the standing doctrines), the',
    'issues the pull request fixes, the pull request itself (no conversation or review',
    'comments on 2026-10-08) and the latest vision of 2026-10-08.',
    '',
    '## Summary',
    '',
    `- Distinct requirements: ${data.requirements.length + issueRequirements.length} (${data.requirements.length} from the owner's messages and the vision, ${issueRequirements.length} items of the ${data.fixes.length} fixed issues).`,
    `- Covered by rows that existed before this audit: ${ownerCovered.length + issueRequirements.length} (every issue item has its own row).`,
    `- Drafted in \`docs/requirements/issue-1188-user-requirements.md\`: ${draftedRows.length} rows for ${ownerDrafted.length} requirements, ${ownerOnlyDrafted.length} of which no earlier row covered and ${ownerDrafted.length - ownerOnlyDrafted.length} of which earlier rows covered only in part.`,
    `- Row verdicts in scope: ${[...verdictCounts].sort().map(([verdict, count]) => `${verdict} ${count}`).join(', ')}.`,
    `- Evidence check: ${implementedInScope.length} implemented rows re-read; ${evidenceProblems.filter((problem) => !problem.reviewed).length} cite evidence that does not exist, and ${evidenceProblems.filter((problem) => problem.reviewed).length} more cite something the check could not find that was reviewed and is not evidence.`,
    '',
    '## The owner\'s requirements',
  ];

  for (const [group, title] of Object.entries(groupTitles)) {
    const members = data.requirements.filter((requirement) => requirement.group === group);
    if (!members.length) continue;
    lines.push('', `### ${title}`, '', '| Requirement | Source | Rows | Status |', '| --- | --- | --- | --- |');
    for (const requirement of members) {
      const sources = requirement.sources.map(({ source, when }) => `[${source}](${linkTo(data.sources.get(source))}) ${when}`).join('; ');
      const rowCells = requirement.rows.join(', ');
      const statuses = requirement.rows.map((row) => `${row} ${statusOf(row, ledger, rows)}`).join('; ');
      lines.push(`| ${escapeCell(requirement.text)} | ${escapeCell(sources)} | ${rowCells} | ${statuses} |`);
    }
  }

  lines.push('', '## The fixed issues\' requirements', '', 'Each item R<n> of issue #<issue> is row R<issue>-<n> of that issue\'s shard; rows past the issue\'s own items are the shard\'s additions.', '', '| Issue item | Row | Requirement | Status |', '| --- | --- | --- | --- |');
  for (const { issue, item, row } of issueRequirements) {
    lines.push(`| #${issue} ${item} | ${row} | ${escapeCell(firstSentence(rows.get(row).requirement))} | ${statusOf(row, ledger, rows)} |`);
  }

  lines.push('', '## Evidence check of the implemented rows', '');
  if (!evidenceProblems.length) {
    lines.push('Every implemented row in scope cites only paths and names that exist.');
  } else {
    lines.push('A row listed here cites a path, name or test title that the check did not find. "Reviewed" means the citation was read and is not evidence (a removed file named as history, a file in another repository); every other entry is a row whose evidence is broken.', '', '| Row | Not found | Review |', '| --- | --- | --- |');
    for (const { identifier, missingPaths, missingNames, missingTitles, reviewed } of evidenceProblems) {
      const notFound = [...missingPaths, ...missingNames, ...missingTitles].map((item) => `\`${item}\``).join(', ');
      lines.push(`| ${identifier} | ${escapeCell(notFound)} | ${escapeCell(reviewed ? `Reviewed: ${reviewed}` : 'Broken evidence')} |`);
    }
  }

  lines.push('', '## Documents that contradicted the latest vision', '', '| Document | Contradiction | Action |', '| --- | --- | --- |');
  for (const contradiction of data.contradictions) {
    lines.push(`| \`${contradiction.document}\` | ${escapeCell(contradiction.text)} | ${escapeCell(contradiction.action)} |`);
  }

  return `${lines.join('\n')}\n`;
}

const rendered = render();
const outputPath = join(repositoryRoot, outputRelative);
if (checkMode) {
  const current = existsSync(outputPath) ? readFileSync(outputPath, 'utf8') : '';
  if (current !== rendered) {
    console.error(`requirement-coverage: ${outputRelative} is stale; run with --write`);
    process.exit(1);
  }
  console.log('requirement-coverage: up to date');
} else {
  writeFileSync(outputPath, rendered);
  console.log(`requirement-coverage: wrote ${outputRelative}`);
}
