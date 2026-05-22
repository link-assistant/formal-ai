// CI guard for PRs that update one supported language but forget Hindi or Chinese.
//
// Existing catalog checks verify key parity in the final tree. This diff-aware
// check catches stale translations when a PR changes wording for English,
// Russian, or another supported language without changing both hi and zh in
// the same language resource.

import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '../../..');
const requiredCompanionLocales = ['hi', 'zh'];

const watchedFiles = [
  'src/web/i18n-catalog.lino',
  'data/seed/prompt-patterns.lino',
  'data/seed/multilingual-responses.lino',
  'data/seed/concepts.lino',
  'data/seed/tools.lino',
  'data/seed/concept-contexts.lino',
];

function runGit(args) {
  return spawnSync('git', args, {
    cwd: repoRoot,
    encoding: 'utf8',
    maxBuffer: 10 * 1024 * 1024,
  });
}

function gitRefExists(ref) {
  return runGit(['rev-parse', '--verify', `${ref}^{commit}`]).status === 0;
}

function resolveBaseRef() {
  const configured = process.env.LANGUAGE_PARITY_BASE_REF;
  const githubBase = process.env.GITHUB_BASE_REF;
  const candidates = [];

  if (configured) candidates.push(configured);
  if (githubBase) {
    candidates.push(`origin/${githubBase}`, githubBase);
  }
  candidates.push('origin/main', 'origin/master', 'main', 'master', 'HEAD^');

  return candidates.find((candidate) => gitRefExists(candidate)) || '';
}

function readCurrentFile(relativePath) {
  const fullPath = path.join(repoRoot, relativePath);
  return fs.existsSync(fullPath) ? fs.readFileSync(fullPath, 'utf8') : '';
}

function readFileAtRef(ref, relativePath) {
  if (!ref) return '';
  const result = runGit(['show', `${ref}:${relativePath}`]);
  return result.status === 0 ? result.stdout : '';
}

function parseSupportedLanguages() {
  const text = readCurrentFile('data/seed/agent-info.lino');
  const match = text.match(/field "supported_languages"\s*\n\s+value "([^"]+)"/);
  if (!match) {
    throw new Error('data/seed/agent-info.lino is missing supported_languages');
  }
  return match[1].split('|').filter(Boolean);
}

function appendSignature(map, language, lines) {
  if (!language || lines.length === 0) return;
  if (!map.has(language)) map.set(language, []);
  map.get(language).push(lines.join('\n').trim());
}

function leadingSpaces(line) {
  return line.length - line.trimStart().length;
}

function collectTopLevelLocaleBlocks(text, supportedLanguages) {
  const supported = new Set(supportedLanguages);
  const signatures = new Map();
  let currentLanguage = '';
  let currentLines = [];

  for (const line of text.split(/\r?\n/)) {
    const locale = line.match(/^([a-z]{2})$/);
    if (locale && supported.has(locale[1])) {
      appendSignature(signatures, currentLanguage, currentLines);
      currentLanguage = locale[1];
      currentLines = [line];
      continue;
    }
    if (currentLanguage) currentLines.push(line);
  }

  appendSignature(signatures, currentLanguage, currentLines);
  return signatures;
}

function collectRecordsByLanguage(text, recordStart, languageField) {
  const signatures = new Map();
  let currentLanguage = '';
  let currentLines = [];

  function commit() {
    appendSignature(signatures, currentLanguage, currentLines);
    currentLanguage = '';
    currentLines = [];
  }

  for (const line of text.split(/\r?\n/)) {
    if (recordStart.test(line)) {
      commit();
      currentLines = [line];
      continue;
    }

    if (currentLines.length === 0) continue;

    currentLines.push(line);
    const language = line.match(languageField);
    if (language) currentLanguage = language[1];
  }

  commit();
  return signatures;
}

function collectIndentedLanguageBlocks(text, headerPattern) {
  const signatures = new Map();
  let currentLanguage = '';
  let currentIndent = 0;
  let currentLines = [];

  function commit() {
    appendSignature(signatures, currentLanguage, currentLines);
    currentLanguage = '';
    currentIndent = 0;
    currentLines = [];
  }

  for (const line of text.split(/\r?\n/)) {
    if (
      currentLanguage &&
      line.trim().length > 0 &&
      leadingSpaces(line) <= currentIndent
    ) {
      commit();
    }

    const header = line.match(headerPattern);
    if (header) {
      commit();
      currentLanguage = header[1];
      currentIndent = leadingSpaces(line);
      currentLines = [line];
      continue;
    }

    if (currentLanguage) currentLines.push(line);
  }

  commit();
  return signatures;
}

function collectToolSignatures(text) {
  const signatures = collectIndentedLanguageBlocks(text, /^\s+localized "([^"]+)"/);
  const englishLines = [];

  for (const line of text.split(/\r?\n/)) {
    if (/^    (name|description) "/.test(line)) {
      englishLines.push(line.trim());
    }
  }

  appendSignature(signatures, 'en', englishLines);
  return signatures;
}

function signaturesForFile(relativePath, text, supportedLanguages) {
  switch (relativePath) {
    case 'src/web/i18n-catalog.lino':
      return collectTopLevelLocaleBlocks(text, supportedLanguages);
    case 'data/seed/prompt-patterns.lino':
      return collectRecordsByLanguage(
        text,
        /^  pattern "/,
        /^    language "([^"]+)"/,
      );
    case 'data/seed/multilingual-responses.lino':
      return collectRecordsByLanguage(
        text,
        /^  response "/,
        /^    language "([^"]+)"/,
      );
    case 'data/seed/concepts.lino':
      return collectIndentedLanguageBlocks(text, /^  localized "([^"]+)"/);
    case 'data/seed/tools.lino':
      return collectToolSignatures(text);
    case 'data/seed/concept-contexts.lino':
      return collectIndentedLanguageBlocks(text, /^    label "([^"]+)"/);
    default:
      return new Map();
  }
}

function signatureValue(signatures, language) {
  return JSON.stringify(signatures.get(language) || []);
}

function changedLanguages(oldSignatures, newSignatures, supportedLanguages) {
  return supportedLanguages.filter(
    (language) =>
      signatureValue(oldSignatures, language) !==
      signatureValue(newSignatures, language),
  );
}

const supportedLanguages = parseSupportedLanguages();
const companionLocales = requiredCompanionLocales.filter((language) =>
  supportedLanguages.includes(language),
);
const baseRef = resolveBaseRef();
const errors = [];

if (!baseRef) {
  console.warn('No git base ref found; skipping language change parity check.');
  process.exit(0);
}

for (const relativePath of watchedFiles) {
  const oldText = readFileAtRef(baseRef, relativePath);
  const newText = readCurrentFile(relativePath);
  const oldSignatures = signaturesForFile(relativePath, oldText, supportedLanguages);
  const newSignatures = signaturesForFile(relativePath, newText, supportedLanguages);
  const changed = changedLanguages(oldSignatures, newSignatures, supportedLanguages);

  if (changed.length === 0) continue;

  const missingCompanions = companionLocales.filter(
    (language) => !changed.includes(language),
  );
  if (missingCompanions.length > 0) {
    errors.push(
      `${relativePath} changed ${changed.join(', ')} language content without updating ${missingCompanions.join(', ')}`,
    );
  }
}

if (errors.length > 0) {
  console.error('Language change parity check failed:');
  for (const error of errors) {
    console.error(`- ${error}`);
  }
  console.error(
    `When a supported-language resource changes, update both ${companionLocales.join(' and ')} in the same PR.`,
  );
  process.exit(1);
}

console.log(
  `Language change parity OK against ${baseRef} for companion locales ${companionLocales.join(', ')}.`,
);
