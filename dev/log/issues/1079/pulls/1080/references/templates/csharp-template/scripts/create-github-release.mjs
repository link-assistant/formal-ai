#!/usr/bin/env bun

/**
 * Create GitHub Release from CHANGELOG.md
 * Usage: bun run scripts/create-github-release.mjs --release-version <version> --repository <repository> [--csharp-root <path>] [--tag-prefix <prefix>] [--language <language>] [--package-id <id>] [--assets-glob <glob>]
 */

import { spawnSync } from 'node:child_process';
import {
  existsSync,
  readFileSync,
  readdirSync,
  statSync,
} from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  buildNuGetBadge as buildLayoutNuGetBadge,
  buildReleaseTag as buildLayoutReleaseTag,
  buildReleaseTitle as buildLayoutReleaseTitle,
  detectCsharpLayout,
  findCsharpProjectFile,
  normalizeReleaseVersion,
} from './release-naming.mjs';

export { normalizeReleaseVersion };

const USAGE =
  'Usage: bun run scripts/create-github-release.mjs --release-version <version> --repository <repository> [--csharp-root <path>] [--tag-prefix <prefix>] [--language <language>] [--package-id <id>] [--assets-glob <glob>]';

// Keep comfortably below GitHub's observed 125000-character release body limit.
export const GITHUB_RELEASE_BODY_MAX_BYTES = 120_000;
const textEncoder = new globalThis.TextEncoder();

/**
 * Parse CLI arguments.
 * @param {string[]} argv
 * @param {NodeJS.ProcessEnv} env
 * @returns {{assetsGlob: string, csharpRoot: string, releaseVersion: string, repository: string, tagPrefix: string, language: string, packageId: string}}
 */
export function parseArgs(argv, env = process.env) {
  const config = {
    assetsGlob: env.ASSETS_GLOB ?? '',
    csharpRoot: env.CSHARP_ROOT ?? '',
    language: env.LANGUAGE ?? 'C#',
    packageId: env.PACKAGE_ID ?? '',
    releaseVersion: env.VERSION ?? '',
    repository: env.REPOSITORY ?? '',
    tagPrefix: env.TAG_PREFIX ?? '',
  };

  for (let index = 0; index < argv.length; index++) {
    const arg = argv[index];

    if (arg === '--release-version' || arg === '--version') {
      config.releaseVersion = readOptionValue(argv, index, arg);
      index++;
    } else if (arg.startsWith('--release-version=')) {
      config.releaseVersion = arg.slice('--release-version='.length);
    } else if (arg.startsWith('--version=')) {
      config.releaseVersion = arg.slice('--version='.length);
    } else if (arg === '--repository') {
      config.repository = readOptionValue(argv, index, arg);
      index++;
    } else if (arg.startsWith('--repository=')) {
      config.repository = arg.slice('--repository='.length);
    } else if (arg === '--csharp-root') {
      config.csharpRoot = readOptionValue(argv, index, arg);
      index++;
    } else if (arg.startsWith('--csharp-root=')) {
      config.csharpRoot = arg.slice('--csharp-root='.length);
    } else if (arg === '--tag-prefix') {
      config.tagPrefix = readOptionValue(argv, index, arg);
      index++;
    } else if (arg.startsWith('--tag-prefix=')) {
      config.tagPrefix = arg.slice('--tag-prefix='.length);
    } else if (arg === '--language') {
      config.language = readOptionValue(argv, index, arg);
      index++;
    } else if (arg.startsWith('--language=')) {
      config.language = arg.slice('--language='.length);
    } else if (arg === '--package-id') {
      config.packageId = readOptionValue(argv, index, arg);
      index++;
    } else if (arg.startsWith('--package-id=')) {
      config.packageId = arg.slice('--package-id='.length);
    } else if (arg === '--assets-glob') {
      config.assetsGlob = readOptionValue(argv, index, arg);
      index++;
    } else if (arg.startsWith('--assets-glob=')) {
      config.assetsGlob = arg.slice('--assets-glob='.length);
    }
  }

  return config;
}

/**
 * Read a CLI option value.
 * @param {string[]} argv
 * @param {number} index
 * @param {string} optionName
 * @returns {string}
 */
function readOptionValue(argv, index, optionName) {
  const value = argv[index + 1];

  if (value === undefined || value.startsWith('--')) {
    throw new Error(`Missing value for ${optionName}`);
  }

  return value;
}

/**
 * Escape text for a regular expression.
 * @param {string} value
 * @returns {string}
 */
function escapeRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Build a release tag.
 * @param {string} tagPrefix
 * @param {string} releaseVersion
 * @returns {string}
 */
export function buildReleaseTag(tagPrefix, releaseVersion) {
  return buildLayoutReleaseTag(releaseVersion, { tagPrefix });
}

/**
 * Build a release title.
 * @param {string} language
 * @param {string} releaseVersion
 * @returns {string}
 */
export function buildReleaseTitle(language, releaseVersion) {
  return buildLayoutReleaseTitle(releaseVersion, {
    csharpRoot: 'csharp',
    language,
  });
}

/**
 * Build a NuGet badge markdown link.
 * @param {string} packageId
 * @param {string} releaseVersion
 * @returns {string}
 */
export function buildNuGetBadge(packageId, releaseVersion) {
  return buildLayoutNuGetBadge(packageId, releaseVersion);
}

/**
 * Append a NuGet badge unless release notes already include a shields.io badge.
 * @param {string} releaseNotes
 * @param {string} packageId
 * @param {string} releaseVersion
 * @returns {string}
 */
export function appendNuGetBadgeIfMissing(
  releaseNotes,
  packageId,
  releaseVersion
) {
  if (!packageId || /img\.shields\.io/i.test(releaseNotes)) {
    return releaseNotes;
  }

  return `${releaseNotes}\n\n---\n\n${buildNuGetBadge(
    packageId,
    releaseVersion
  )}`;
}

/**
 * Calculate the UTF-8 byte length for a string.
 * @param {string} value
 * @returns {number}
 */
function getUtf8ByteLength(value) {
  return textEncoder.encode(value).byteLength;
}

/**
 * Truncate a string without splitting UTF-8 characters.
 * @param {string} value
 * @param {number} maxBytes
 * @returns {string}
 */
function truncateToUtf8Bytes(value, maxBytes) {
  const chunks = [];
  let usedBytes = 0;

  for (const character of value) {
    const characterBytes = getUtf8ByteLength(character);

    if (usedBytes + characterBytes > maxBytes) {
      break;
    }

    chunks.push(character);
    usedBytes += characterBytes;
  }

  return chunks.join('');
}

/**
 * Build a link to the changelog at a release tag.
 * @param {string} repository
 * @param {string} tag
 * @returns {string}
 */
function buildTaggedChangelogUrl(repository, tag) {
  return `https://github.com/${repository}/blob/${tag}/CHANGELOG.md`;
}

/**
 * Build the note appended when release notes are shortened.
 * @param {{repository: string, tag: string}} options
 * @returns {string}
 */
function buildTruncatedReleaseNotesNotice({ repository, tag }) {
  const changelogUrl = buildTaggedChangelogUrl(repository, tag);

  return `Release notes were shortened to fit GitHub's release body limit. See the full tagged CHANGELOG.md: ${changelogUrl}`;
}

/**
 * Limit release notes to GitHub's release body budget.
 * @param {{maxBytes?: number, releaseNotes: string, repository: string, tag: string}} options
 * @returns {string}
 */
export function limitReleaseNotesBytes({
  maxBytes = GITHUB_RELEASE_BODY_MAX_BYTES,
  releaseNotes,
  repository,
  tag,
}) {
  if (getUtf8ByteLength(releaseNotes) <= maxBytes) {
    return releaseNotes;
  }

  const suffix = `\n\n...\n\n${buildTruncatedReleaseNotesNotice({
    repository,
    tag,
  })}`;
  const suffixBytes = getUtf8ByteLength(suffix);
  const availableBytes = Math.max(0, maxBytes - suffixBytes);
  const shortenedNotes = truncateToUtf8Bytes(
    releaseNotes,
    availableBytes
  ).trimEnd();
  const limitedNotes = `${shortenedNotes}${suffix}`;

  if (getUtf8ByteLength(limitedNotes) <= maxBytes) {
    return limitedNotes;
  }

  return truncateToUtf8Bytes(limitedNotes, maxBytes);
}

/**
 * Extract changelog content for a specific version
 * @param {string} changelog
 * @param {string} version
 * @returns {string}
 */
export function extractReleaseNotes(changelog, version) {
  const semver = normalizeReleaseVersion(version);

  // Find the section for this version
  const escapedVersion = escapeRegex(semver);
  const pattern = new RegExp(
    `(?:^|\\n)## \\[?${escapedVersion}\\]?[^\\n]*\\n([\\s\\S]*?)(?=\\n## \\[?\\d|$)`
  );
  const match = changelog.match(pattern);

  if (match) {
    const releaseNotes = match[1].trim();
    return releaseNotes || `Release ${semver}`;
  }

  return `Release ${semver}`;
}

/**
 * Find a package id by scanning project files.
 * @param {string} rootDir
 * @returns {string}
 */
export function findPackageId(rootDir = '.') {
  const csprojPath = findCsharpProjectFile(rootDir);
  if (!csprojPath) {
    return '';
  }

  const csproj = readFileSync(csprojPath, 'utf-8');
  const packageIdMatch = csproj.match(/<PackageId>([^<]+)<\/PackageId>/);
  if (packageIdMatch) {
    return packageIdMatch[1].trim();
  }

  const assemblyNameMatch = csproj.match(
    /<AssemblyName>([^<]+)<\/AssemblyName>/
  );
  if (assemblyNameMatch) {
    return assemblyNameMatch[1].trim();
  }

  return path.basename(csprojPath, '.csproj');
}

/**
 * Build the GitHub release API payload.
 * @param {{changelog: string, csharpRoot?: string, language: string, packageId: string, releaseVersion: string, repository?: string, tagPrefix?: string}} options
 * @returns {string}
 */
export function buildReleasePayload({
  changelog,
  csharpRoot = '.',
  language,
  packageId,
  releaseVersion,
  repository = '',
  tagPrefix = '',
}) {
  const semver = normalizeReleaseVersion(releaseVersion);
  const tag = buildLayoutReleaseTag(semver, { csharpRoot, tagPrefix });
  const releaseNotes = appendNuGetBadgeIfMissing(
    extractReleaseNotes(changelog, semver),
    packageId,
    semver
  );

  return JSON.stringify({
    tag_name: tag,
    name: buildLayoutReleaseTitle(semver, {
      csharpRoot,
      language,
      packageName: packageId,
    }),
    body: limitReleaseNotesBytes({ releaseNotes, repository, tag }),
  });
}

/**
 * Create a GitHub release using gh.
 * @param {{payload: string, repository: string, spawn?: typeof spawnSync}} options
 * @returns {{alreadyExists: boolean}}
 */
export function createRelease({ payload, repository, spawn = spawnSync }) {
  const result = spawn(
    'gh',
    ['api', `repos/${repository}/releases`, '-X', 'POST', '--input', '-'],
    {
      encoding: 'utf-8',
      input: payload,
    }
  );

  if (result.error) {
    throw new Error(`gh api failed to start: ${result.error.message}`);
  }

  if (result.status === 0) {
    return { alreadyExists: false };
  }

  const output = [result.stderr, result.stdout]
    .filter((value) => typeof value === 'string' && value.trim())
    .join('\n');

  if (/already_exists|already exists/i.test(output)) {
    return { alreadyExists: true };
  }

  throw new Error(`gh api failed with code ${result.status}: ${output}`);
}

/**
 * Resolve a simple release asset glob.
 *
 * Supports exact file paths or `*` in the file name portion, such as
 * `artifacts/*.nupkg`. Matches are returned in deterministic path order.
 *
 * @param {string} assetsGlob
 * @param {string} cwd
 * @returns {string[]}
 */
export function resolveReleaseAssets(assetsGlob, cwd = '.') {
  const pattern = String(assetsGlob ?? '').trim();
  if (!pattern) {
    return [];
  }

  const absolutePattern = path.isAbsolute(pattern)
    ? pattern
    : path.resolve(cwd, pattern);
  const assetDirectory = path.dirname(absolutePattern);
  const filePattern = path.basename(absolutePattern);

  if (!filePattern.includes('*')) {
    try {
      return existsSync(absolutePattern) && statSync(absolutePattern).isFile()
        ? [absolutePattern]
        : [];
    } catch {
      return [];
    }
  }

  let entries;
  try {
    entries = readdirSync(assetDirectory, { withFileTypes: true });
  } catch {
    return [];
  }

  const filePatternRegex = new RegExp(
    `^${escapeRegex(filePattern).replace(/\\\*/g, '.*')}$`
  );

  return entries
    .filter((entry) => entry.isFile() && filePatternRegex.test(entry.name))
    .map((entry) => path.join(assetDirectory, entry.name))
    .sort();
}

/**
 * Upload release assets using gh.
 * @param {{assetPaths: string[], repository: string, tag: string, spawn?: typeof spawnSync}} options
 * @returns {void}
 */
export function uploadReleaseAssets({
  assetPaths,
  repository,
  tag,
  spawn = spawnSync,
}) {
  if (assetPaths.length === 0) {
    return;
  }

  const result = spawn(
    'gh',
    [
      'release',
      'upload',
      tag,
      ...assetPaths,
      '--clobber',
      '--repo',
      repository,
    ],
    { encoding: 'utf-8' }
  );

  if (result.error) {
    throw new Error(`gh release upload failed to start: ${result.error.message}`);
  }

  if (result.status !== 0) {
    const output = [result.stderr, result.stdout]
      .filter((value) => typeof value === 'string' && value.trim())
      .join('\n');

    throw new Error(
      `gh release upload failed with code ${result.status}: ${output}`
    );
  }
}

/**
 * Run the CLI.
 * @param {{argv?: string[], cwd?: string, env?: NodeJS.ProcessEnv, spawn?: typeof spawnSync, stderr?: typeof console.error, stdout?: typeof console.log}} options
 * @returns {number}
 */
export function main({
  argv = process.argv.slice(2),
  cwd = process.cwd(),
  env = process.env,
  spawn = spawnSync,
  stderr = console.error,
  stdout = console.log,
} = {}) {
  try {
    const {
      assetsGlob,
      csharpRoot,
      language,
      packageId,
      releaseVersion,
      repository,
      tagPrefix,
    } =
      parseArgs(argv, env);

    if (!releaseVersion || !repository) {
      stderr('Error: Missing required arguments');
      stderr(USAGE);
      return 1;
    }

    const layout = detectCsharpLayout({ cwd, csharpRoot });
    const csharpRootPath =
      layout.csharpRoot === '.' ? cwd : path.join(cwd, layout.csharpRoot);
    const changelogPath = path.join(csharpRootPath, 'CHANGELOG.md');
    const changelog = existsSync(changelogPath)
      ? readFileSync(changelogPath, 'utf-8')
      : '';
    const resolvedPackageId = packageId || findPackageId(csharpRootPath);
    const tag = buildLayoutReleaseTag(releaseVersion, {
      csharpRoot: layout.csharpRoot,
      tagPrefix,
    });
    const payload = buildReleasePayload({
      changelog,
      csharpRoot: layout.csharpRoot,
      language,
      packageId: resolvedPackageId,
      releaseVersion,
      repository,
      tagPrefix,
    });

    stdout(`Creating GitHub release for ${tag}...`);

    const result = createRelease({ payload, repository, spawn });

    if (result.alreadyExists) {
      stdout(`GitHub release already exists: ${tag}, reconciling assets`);
    } else {
      stdout(`Created GitHub release: ${tag}`);
    }

    if (assetsGlob) {
      const assetPaths = resolveReleaseAssets(assetsGlob, csharpRootPath);

      if (assetPaths.length === 0) {
        throw new Error(`No release assets matched ${assetsGlob}`);
      }

      stdout(`Uploading ${assetPaths.length} release asset(s) to ${tag}...`);
      uploadReleaseAssets({ assetPaths, repository, spawn, tag });
      stdout(`Uploaded ${assetPaths.length} release asset(s) to ${tag}`);
    }

    return 0;
  } catch (error) {
    stderr(`Error creating release: ${error.message}`);
    return 1;
  }
}

function isCliEntryPoint() {
  return (
    typeof process !== 'undefined' &&
    process.argv?.[1] &&
    fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  );
}

if (isCliEntryPoint()) {
  process.exitCode = main();
}
