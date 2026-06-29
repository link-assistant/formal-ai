// Diff-aware CI guard for supported-language test coverage.
//
// When a PR changes language-facing code, it must also add or update tests
// that cover every supported language from data/seed/agent-info.lino. This
// keeps fixes from landing with only one-language regressions, such as a
// Russian-only translation test that leaves English, Hindi, and Chinese
// behavior unpinned.

import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { parseSupportedLanguagesFromAgentInfo } from './lino-seed-parser.mjs';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '../../..');

const languageNames = {
  en: ['english'],
  ru: ['russian', 'русский', 'русского'],
  hi: ['hindi', 'хинди', 'हिंदी', 'हिन्दी'],
  zh: ['chinese', 'китайский', 'китайского', '中文', '汉语', '漢語'],
};

const scriptMarkers = {
  ru: /\p{Script=Cyrillic}/u,
  hi: /\p{Script=Devanagari}/u,
  zh: /\p{Script=Han}/u,
};

const languageFacingPrefixes = [
  'data/seed/',
  'src/solver_handlers/',
  'src/translation/',
  'src/web/worker/',
];

const languageFacingFiles = new Set([
  'src/language.rs',
  'src/solver.rs',
  'src/solver_helpers.rs',
  'src/web/app/main.jsx',
  'src/web/formal_ai_worker.js',
  'src/web/i18n-catalog.lino',
  'src/web/i18n-catalog-permissions.lino',
  'src/web/i18n.js',
]);

function readRepoFile(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function parseSupportedLanguages() {
  const agentInfo = readRepoFile('data/seed/agent-info.lino');
  return parseSupportedLanguagesFromAgentInfo(agentInfo);
}

function runGit(args) {
  return spawnSync('git', args, {
    cwd: repoRoot,
    encoding: 'utf8',
    maxBuffer: 20 * 1024 * 1024,
  });
}

function gitRefExists(ref) {
  return runGit(['rev-parse', '--verify', `${ref}^{commit}`]).status === 0;
}

function resolveBaseRef() {
  const configured = process.env.LANGUAGE_TEST_COVERAGE_BASE_REF;
  const githubBase = process.env.GITHUB_BASE_REF;
  const candidates = [];

  if (configured) candidates.push(configured);
  if (githubBase) {
    candidates.push(`origin/${githubBase}`, githubBase);
  }
  candidates.push('origin/main', 'origin/master', 'main', 'master', 'HEAD^');

  return candidates.find((candidate) => gitRefExists(candidate)) || '';
}

function gitDiffFromBase(baseRef, args, paths = []) {
  const command = ['diff', ...args, baseRef];
  if (paths.length > 0) command.push('--', ...paths);

  const result = runGit(command);
  if (result.status === 0) return result.stdout;

  throw new Error(`git diff failed against ${baseRef}: ${result.stderr}`);
}

function changedFiles(baseRef) {
  return gitDiffFromBase(baseRef, ['--name-only'])
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function isLanguageFacingChange(relativePath) {
  if (languageFacingFiles.has(relativePath)) return true;
  return languageFacingPrefixes.some((prefix) => relativePath.startsWith(prefix));
}

function isChangedTestFile(relativePath) {
  const isTestRoot =
    relativePath.startsWith('tests/unit/') ||
    relativePath.startsWith('tests/integration/') ||
    relativePath.startsWith('tests/e2e/tests/');
  return isTestRoot && /\.(rs|js|mjs|ts|tsx)$/.test(relativePath);
}

function addedTestLines(baseRef, testFiles) {
  if (testFiles.length === 0) return [];
  return gitDiffFromBase(baseRef, ['--unified=0', '--no-ext-diff'], testFiles)
    .split(/\r?\n/)
    .filter((line) => line.startsWith('+') && !line.startsWith('+++'))
    .map((line) => line.slice(1));
}

function lineCoversLanguage(line, language) {
  const lower = line.toLowerCase();
  const quotedCode = String.raw`['"]${language}['"]`;
  const structuredPatterns = [
    new RegExp(String.raw`\b(?:language|locale|source|target)\s*[:=]\s*${quotedCode}`, 'i'),
    new RegExp(String.raw`\blanguage_(?:from|to):${language}\b`, 'i'),
    new RegExp(String.raw`\btranslate_${language}_to_[a-z]{2}\b`, 'i'),
    new RegExp(String.raw`\btranslate_[a-z]{2}_to_${language}\b`, 'i'),
  ];

  if (structuredPatterns.some((pattern) => pattern.test(line))) return true;
  if ((languageNames[language] || []).some((name) => lower.includes(name))) return true;
  return Boolean(scriptMarkers[language]?.test(line));
}

const supportedLanguages = parseSupportedLanguages();
const baseRef = resolveBaseRef();

if (!baseRef) {
  console.warn('No git base ref found; skipping language test coverage check.');
  process.exit(0);
}

const files = changedFiles(baseRef);
const languageFacingChanges = files.filter(isLanguageFacingChange);

if (languageFacingChanges.length === 0) {
  console.log(`Language test coverage OK: no language-facing changes against ${baseRef}.`);
  process.exit(0);
}

const testFiles = files.filter(isChangedTestFile);
const addedLines = addedTestLines(baseRef, testFiles);
const coveredLanguages = new Set();

for (const line of addedLines) {
  for (const language of supportedLanguages) {
    if (lineCoversLanguage(line, language)) coveredLanguages.add(language);
  }
}

const missingLanguages = supportedLanguages.filter(
  (language) => !coveredLanguages.has(language),
);

if (missingLanguages.length > 0) {
  console.error('Language test coverage check failed.');
  console.error(
    `Language-facing changes were detected in: ${languageFacingChanges.join(', ')}`,
  );
  if (testFiles.length === 0) {
    console.error('No changed test files were found in this PR diff.');
  } else {
    console.error(`Changed test files: ${testFiles.join(', ')}`);
    console.error(`Covered languages in added test lines: ${[...coveredLanguages].join(', ') || '(none)'}`);
  }
  console.error(
    `Add or update tests for every supported language: ${supportedLanguages.join(', ')}. Missing: ${missingLanguages.join(', ')}.`,
  );
  process.exit(1);
}

console.log(
  `Language test coverage OK against ${baseRef}: ${supportedLanguages.join(', ')}.`,
);
