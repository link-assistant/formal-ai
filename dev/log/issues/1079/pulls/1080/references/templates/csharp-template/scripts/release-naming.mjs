import {
  existsSync,
  readFileSync,
  readdirSync,
  statSync,
} from 'node:fs';
import path from 'node:path';

const DEFAULT_CSHARP_SUBDIR = 'csharp';
const IGNORED_DIRECTORIES = new Set([
  '.git',
  'artifacts',
  'bin',
  'node_modules',
  'obj',
]);

/**
 * Normalize a C# root to a stable repository-relative value.
 * @param {string} csharpRoot
 * @returns {string}
 */
function normalizeCsharpRoot(csharpRoot) {
  const normalized = String(csharpRoot ?? '')
    .trim()
    .replaceAll('\\', '/')
    .replace(/\/+$/u, '')
    .replace(/^\.\//u, '');

  return normalized === '' || normalized === '.' ? '.' : normalized;
}

/**
 * Resolve a C# root against a working directory.
 * @param {string} cwd
 * @param {string} csharpRoot
 * @returns {string}
 */
function resolveRootPath(cwd, csharpRoot) {
  return path.resolve(cwd, csharpRoot === '.' ? '' : csharpRoot);
}

/**
 * Return true when a directory exists.
 * @param {string} directory
 * @returns {boolean}
 */
function isDirectory(directory) {
  try {
    return statSync(directory).isDirectory();
  } catch {
    return false;
  }
}

/**
 * Find C# project files under a directory.
 * @param {string} rootDir
 * @param {number} maxDepth
 * @returns {string[]}
 */
export function findCsharpProjectFiles(rootDir = '.', maxDepth = 6) {
  const projects = [];
  const root = path.resolve(rootDir);

  function walk(directory, depth) {
    if (depth > maxDepth) {
      return;
    }

    let entries;
    try {
      entries = readdirSync(directory, { withFileTypes: true });
    } catch {
      return;
    }

    for (const entry of entries) {
      if (IGNORED_DIRECTORIES.has(entry.name)) {
        continue;
      }

      const fullPath = path.join(directory, entry.name);

      if (entry.isDirectory()) {
        walk(fullPath, depth + 1);
      } else if (entry.isFile() && entry.name.endsWith('.csproj')) {
        projects.push(fullPath);
      }
    }
  }

  walk(root, 0);
  return projects.sort();
}

/**
 * Check whether a directory is a C# package root for this template.
 * @param {string} rootDir
 * @returns {boolean}
 */
function hasCsharpManifest(rootDir) {
  if (!isDirectory(rootDir)) {
    return false;
  }

  let entries;
  try {
    entries = readdirSync(rootDir, { withFileTypes: true });
  } catch {
    return false;
  }

  if (
    entries.some(
      (entry) =>
        entry.isFile() &&
        (entry.name.endsWith('.csproj') || entry.name.endsWith('.sln'))
    )
  ) {
    return true;
  }

  const srcPath = path.join(rootDir, 'src');
  return findCsharpProjectFiles(srcPath, 4).length > 0;
}

/**
 * Detect whether the C# project lives at repo root or in a csharp/ subdirectory.
 * @param {object} [options]
 * @param {string} [options.cwd]
 * @param {string} [options.csharpRoot]
 * @param {string} [options.csharpSubdir]
 * @returns {{csharpRoot: string, isMultiLanguage: boolean}}
 */
export function detectCsharpLayout({
  cwd = process.cwd(),
  csharpRoot = process.env.CSHARP_ROOT ?? '',
  csharpSubdir = DEFAULT_CSHARP_SUBDIR,
} = {}) {
  const explicitRoot = normalizeCsharpRoot(csharpRoot);
  if (explicitRoot !== '.') {
    return {
      csharpRoot: explicitRoot,
      isMultiLanguage: true,
    };
  }

  if (String(csharpRoot ?? '').trim() === '.') {
    return {
      csharpRoot: '.',
      isMultiLanguage: false,
    };
  }

  const repoRoot = path.resolve(cwd);
  if (hasCsharpManifest(repoRoot)) {
    return {
      csharpRoot: '.',
      isMultiLanguage: false,
    };
  }

  const subdir = normalizeCsharpRoot(csharpSubdir);
  if (hasCsharpManifest(resolveRootPath(repoRoot, subdir))) {
    return {
      csharpRoot: subdir,
      isMultiLanguage: true,
    };
  }

  return {
    csharpRoot: '.',
    isMultiLanguage: false,
  };
}

/**
 * Whether the detected layout is a multi-language repository.
 * @param {object} [options]
 * @returns {boolean}
 */
export function isMultiLanguage(options = {}) {
  return detectCsharpLayout(options).isMultiLanguage;
}

/**
 * Get the release tag prefix for this layout.
 * @param {object} [options]
 * @param {string} [options.tagPrefix]
 * @returns {string}
 */
export function getTagPrefix(options = {}) {
  if (options.tagPrefix) {
    return options.tagPrefix;
  }

  return isMultiLanguage(options) ? 'cs_v' : 'v';
}

/**
 * Normalize release versions to bare semver.
 * @param {string} releaseVersion
 * @returns {string}
 */
export function normalizeReleaseVersion(releaseVersion) {
  const trimmedVersion = String(releaseVersion ?? '').trim();
  const semverTagMatch = trimmedVersion.match(
    /(?:^|[-_])v?(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?)$/iu
  );

  if (semverTagMatch) {
    return semverTagMatch[1];
  }

  return trimmedVersion
    .replace(/^[A-Za-z][A-Za-z0-9]*[-_]/u, '')
    .replace(/^v/iu, '');
}

/**
 * Build a release tag for this layout.
 * @param {string} releaseVersion
 * @param {object} [options]
 * @returns {string}
 */
export function buildReleaseTag(releaseVersion, options = {}) {
  return `${getTagPrefix(options)}${normalizeReleaseVersion(releaseVersion)}`;
}

/**
 * Build a release title for this layout.
 * @param {string} releaseVersion
 * @param {object} [options]
 * @param {string} [options.language]
 * @param {string} [options.packageName]
 * @returns {string}
 */
export function buildReleaseTitle(releaseVersion, options = {}) {
  const semver = normalizeReleaseVersion(releaseVersion);
  const language = String(options.language ?? 'C#').trim() || 'C#';

  if (isMultiLanguage(options)) {
    return `[${language}] ${semver}`;
  }

  const packageName = String(options.packageName ?? '').trim();
  return `${packageName || language} ${semver}`;
}

/**
 * Build the exact NuGet package version URL.
 * @param {string} packageId
 * @param {string} releaseVersion
 * @returns {string}
 */
export function buildNuGetPackageVersionUrl(packageId, releaseVersion) {
  const encodedPackageId = encodeURIComponent(packageId);
  const encodedVersion = encodeURIComponent(
    normalizeReleaseVersion(releaseVersion)
  );
  return `https://www.nuget.org/packages/${encodedPackageId}/${encodedVersion}`;
}

/**
 * Build a NuGet shields.io badge markdown link.
 * @param {string} packageId
 * @param {string} releaseVersion
 * @returns {string}
 */
export function buildNuGetBadge(packageId, releaseVersion) {
  const version = normalizeReleaseVersion(releaseVersion);
  const badgeVersion = encodeURIComponent(version);

  return `[![NuGet](https://img.shields.io/badge/NuGet-${badgeVersion}-004880?logo=nuget)](${buildNuGetPackageVersionUrl(
    packageId,
    version
  )})`;
}

/**
 * Extract PackageId or AssemblyName from a project file.
 * @param {string} csprojPath
 * @returns {string}
 */
function readProjectName(csprojPath) {
  let csproj;
  try {
    csproj = readFileSync(csprojPath, 'utf-8');
  } catch {
    return path.basename(csprojPath, '.csproj');
  }

  const packageIdMatch = csproj.match(/<PackageId>([^<]+)<\/PackageId>/u);
  if (packageIdMatch) {
    return packageIdMatch[1].trim();
  }

  const assemblyNameMatch = csproj.match(/<AssemblyName>([^<]+)<\/AssemblyName>/u);
  if (assemblyNameMatch) {
    return assemblyNameMatch[1].trim();
  }

  return path.basename(csprojPath, '.csproj');
}

/**
 * Find the package project file, preferring src/ over tests/examples.
 * @param {string} rootDir
 * @param {string} packageName
 * @returns {string}
 */
export function findCsharpProjectFile(rootDir = '.', packageName = 'MyPackage') {
  const root = path.resolve(rootDir);
  const normalizedPackageName = packageName.toLowerCase();
  const candidates = findCsharpProjectFiles(root);

  if (candidates.length === 0) {
    return '';
  }

  const scored = candidates.map((candidate) => {
    const relativePath = path.relative(root, candidate).replaceAll('\\', '/');
    const projectName = readProjectName(candidate);
    let score = 0;

    if (projectName.toLowerCase() === normalizedPackageName) {
      score += 100;
    }
    if (path.basename(candidate, '.csproj').toLowerCase() === normalizedPackageName) {
      score += 60;
    }
    if (relativePath.startsWith('src/')) {
      score += 40;
    }
    if (relativePath.startsWith('tests/') || relativePath.startsWith('examples/')) {
      score -= 40;
    }

    return { candidate, relativePath, score };
  });

  scored.sort((left, right) => {
    if (right.score !== left.score) {
      return right.score - left.score;
    }
    return left.relativePath.localeCompare(right.relativePath);
  });

  return scored[0].candidate;
}

/**
 * Resolve a repository-relative path under the detected C# root.
 * @param {string} csharpRoot
 * @param {string} relativePath
 * @returns {string}
 */
export function resolvePathInCsharpRoot(csharpRoot, relativePath) {
  const root = normalizeCsharpRoot(csharpRoot);
  return root === '.' ? relativePath : path.join(root, relativePath);
}
