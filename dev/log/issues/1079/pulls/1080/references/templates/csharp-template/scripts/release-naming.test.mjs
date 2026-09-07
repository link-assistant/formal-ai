import { describe, expect, test } from 'bun:test';
import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import {
  buildNuGetBadge,
  buildReleaseTag,
  buildReleaseTitle,
  detectCsharpLayout,
  getTagPrefix,
  isMultiLanguage,
  normalizeReleaseVersion,
} from './release-naming.mjs';

function makeRepo(files) {
  const root = mkdtempSync(path.join(tmpdir(), 'csharp-layout-'));

  for (const [relativePath, content] of Object.entries(files)) {
    const filePath = path.join(root, relativePath);
    mkdirSync(path.dirname(filePath), { recursive: true });
    writeFileSync(filePath, content);
  }

  return root;
}

describe('release naming layout detection', () => {
  test('treats the template root layout as single-language', () => {
    const root = makeRepo({
      'MyPackage.sln': '',
      'src/MyPackage/MyPackage.csproj': '<Project />',
    });
    try {
      expect(detectCsharpLayout({ cwd: root })).toEqual({
        csharpRoot: '.',
        isMultiLanguage: false,
      });
      expect(isMultiLanguage({ cwd: root })).toBe(false);
    } finally {
      rmSync(root, { force: true, recursive: true });
    }
  });

  test('treats a csharp subdirectory layout as multi-language', () => {
    const root = makeRepo({
      'js/package.json': '{}',
      'csharp/src/MyPackage/MyPackage.csproj': '<Project />',
    });
    try {
      expect(detectCsharpLayout({ cwd: root })).toEqual({
        csharpRoot: 'csharp',
        isMultiLanguage: true,
      });
      expect(isMultiLanguage({ cwd: root })).toBe(true);
    } finally {
      rmSync(root, { force: true, recursive: true });
    }
  });
});

describe('release naming tags and titles', () => {
  test('uses plain v tags and package titles for single-language releases', () => {
    const single = { csharpRoot: '.' };

    expect(getTagPrefix(single)).toBe('v');
    expect(buildReleaseTag('1.2.3', single)).toBe('v1.2.3');
    expect(buildReleaseTitle('1.2.3', {
      ...single,
      packageName: 'MyPackage',
    })).toBe('MyPackage 1.2.3');
  });

  test('uses cs_v tags and C# title prefixes for multi-language releases', () => {
    const multi = { csharpRoot: 'csharp' };

    expect(getTagPrefix(multi)).toBe('cs_v');
    expect(buildReleaseTag('1.2.3', multi)).toBe('cs_v1.2.3');
    expect(buildReleaseTitle('1.2.3', {
      ...multi,
      packageName: 'MyPackage',
    })).toBe('[C#] 1.2.3');
  });

  test('normalizes already-prefixed versions idempotently', () => {
    expect(normalizeReleaseVersion('cs_v1.2.3')).toBe('1.2.3');
    expect(normalizeReleaseVersion('cs-v1.2.3')).toBe('1.2.3');
    expect(normalizeReleaseVersion('csharp_v1.2.3')).toBe('1.2.3');
    expect(normalizeReleaseVersion('v1.2.3')).toBe('1.2.3');
    expect(buildReleaseTag('cs_v1.2.3', { csharpRoot: 'csharp' })).toBe(
      'cs_v1.2.3'
    );
    expect(buildReleaseTag('v1.2.3', { csharpRoot: '.' })).toBe('v1.2.3');
  });
});

describe('release naming NuGet badge', () => {
  test('links to the exact NuGet version page using the bare version', () => {
    const badge = buildNuGetBadge('MyPackage', 'cs_v1.2.3');

    expect(badge).toContain('img.shields.io');
    expect(badge).toContain('https://www.nuget.org/packages/MyPackage/1.2.3');
    expect(badge).not.toContain('cs_v');
  });
});
