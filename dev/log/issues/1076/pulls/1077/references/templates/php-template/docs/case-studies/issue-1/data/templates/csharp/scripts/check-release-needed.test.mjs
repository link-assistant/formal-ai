import { describe, expect, test } from 'bun:test';
import {
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import path from 'node:path';

import {
  decide,
  fetchGithubReleaseExists,
  fetchNugetVersions,
  readCsprojInfo,
} from './check-release-needed.mjs';

/**
 * Start a 127.0.0.1 mock that serves the two upstream endpoints
 * `check-release-needed.mjs` consults: NuGet's flat-container index and
 * GitHub's release-by-tag lookup. The script honours `NUGET_INDEX_URL` and
 * `GITHUB_API_URL` overrides so we can point both probes at the mock.
 *
 * @param {{ versions: string[] | null, githubReleaseStatus: number }} options
 * @returns {Promise<{ nugetUrl: string, githubUrl: string, close: () => Promise<void> }>}
 */
function startNugetAndGithubMock({ versions, githubReleaseStatus }) {
  const sockets = new Set();
  const server = createServer((req, res) => {
    res.setHeader('connection', 'close');
    if (req.url?.startsWith('/nuget/')) {
      if (versions === null) {
        res.writeHead(404).end();
      } else {
        res.writeHead(200, { 'content-type': 'application/json' });
        res.end(JSON.stringify({ versions }));
      }
      return;
    }
    if (req.url?.startsWith('/github/')) {
      res.writeHead(githubReleaseStatus).end();
      return;
    }
    res.writeHead(500).end();
  });
  server.on('connection', (socket) => {
    sockets.add(socket);
    socket.on('close', () => sockets.delete(socket));
  });

  return new Promise((resolve) => {
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      resolve({
        nugetUrl: `http://127.0.0.1:${port}/nuget`,
        githubUrl: `http://127.0.0.1:${port}/github`,
        close: () =>
          new Promise((r) => {
            for (const socket of sockets) {
              socket.destroy();
            }
            server.close(() => r());
          }),
      });
    });
  });
}

describe('check-release-needed decide()', () => {
  test('changesets take the normal release path', () => {
    const result = decide({
      hasChangesets: true,
      currentVersion: '2.4.0',
      publishedVersions: ['2.2.2'],
      githubReleaseExists: false,
    });
    expect(result.shouldRelease).toBe(true);
    expect(result.skipBump).toBe(false);
    expect(result.nugetPublished).toBe(false);
    expect(result.reason).toMatch(/changesets present/);
  });

  test('self-heals when csproj version is missing from NuGet', () => {
    const result = decide({
      hasChangesets: false,
      currentVersion: '2.4.0',
      publishedVersions: ['2.2.2', '2.3.0'],
      githubReleaseExists: false,
    });
    expect(result.shouldRelease).toBe(true);
    expect(result.skipBump).toBe(true);
    expect(result.nugetPublished).toBe(false);
    expect(result.reason).toMatch(/not yet published on NuGet/);
  });

  test('self-heals when package id is unknown to NuGet', () => {
    const result = decide({
      hasChangesets: false,
      currentVersion: '0.1.0',
      publishedVersions: null,
      githubReleaseExists: false,
    });
    expect(result.shouldRelease).toBe(true);
    expect(result.skipBump).toBe(true);
    expect(result.nugetPublished).toBe(false);
    expect(result.reason).toMatch(/not yet registered on NuGet/);
  });

  test('self-heals GitHub release when NuGet already has the version', () => {
    const result = decide({
      hasChangesets: false,
      currentVersion: '2.4.0',
      publishedVersions: ['2.2.2', '2.4.0'],
      githubReleaseExists: false,
    });
    expect(result.shouldRelease).toBe(true);
    expect(result.skipBump).toBe(true);
    expect(result.nugetPublished).toBe(true);
    expect(result.reason).toMatch(/no GitHub release/);
  });

  test('no-op when both NuGet and GitHub release exist', () => {
    const result = decide({
      hasChangesets: false,
      currentVersion: '2.4.0',
      publishedVersions: ['2.2.2', '2.4.0'],
      githubReleaseExists: true,
    });
    expect(result.shouldRelease).toBe(false);
    expect(result.skipBump).toBe(false);
    expect(result.nugetPublished).toBe(true);
    expect(result.reason).toMatch(/no release needed/);
  });
});

describe('check-release-needed readCsprojInfo()', () => {
  test('extracts version and package id', () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'check-release-csproj-'));
    try {
      const csprojPath = path.join(dir, 'sample.csproj');
      writeFileSync(
        csprojPath,
        '<Project Sdk="Microsoft.NET.Sdk">\n  <PropertyGroup>\n    <Version>1.2.3</Version>\n    <PackageId>MyPackage</PackageId>\n  </PropertyGroup>\n</Project>\n'
      );

      const info = readCsprojInfo(csprojPath);
      expect(info.version).toBe('1.2.3');
      expect(info.packageId).toBe('MyPackage');
    } finally {
      rmSync(dir, { force: true, recursive: true });
    }
  });

  test('returns empty package id when not set in csproj', () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'check-release-csproj-noid-'));
    try {
      const csprojPath = path.join(dir, 'sample.csproj');
      writeFileSync(
        csprojPath,
        '<Project Sdk="Microsoft.NET.Sdk">\n  <PropertyGroup>\n    <Version>9.0.0</Version>\n  </PropertyGroup>\n</Project>\n'
      );

      const info = readCsprojInfo(csprojPath);
      expect(info.version).toBe('9.0.0');
      expect(info.packageId).toBe('');
    } finally {
      rmSync(dir, { force: true, recursive: true });
    }
  });

  test('throws on missing <Version> tag', () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'check-release-csproj-novr-'));
    try {
      const csprojPath = path.join(dir, 'sample.csproj');
      writeFileSync(
        csprojPath,
        '<Project Sdk="Microsoft.NET.Sdk">\n  <PropertyGroup>\n  </PropertyGroup>\n</Project>\n'
      );

      expect(() => readCsprojInfo(csprojPath)).toThrow(/Could not parse <Version>/);
    } finally {
      rmSync(dir, { force: true, recursive: true });
    }
  });
});

describe('check-release-needed probes', () => {
  test('fetchNugetVersions returns the version list from the flat-container index', async () => {
    const mock = await startNugetAndGithubMock({
      versions: ['1.0.0', '1.1.0', '2.0.0'],
      githubReleaseStatus: 404,
    });
    const previous = process.env.NUGET_INDEX_URL;
    process.env.NUGET_INDEX_URL = mock.nugetUrl;
    try {
      const versions = await fetchNugetVersions('MyPackage');
      expect(versions).toEqual(['1.0.0', '1.1.0', '2.0.0']);
    } finally {
      if (previous === undefined) {
        delete process.env.NUGET_INDEX_URL;
      } else {
        process.env.NUGET_INDEX_URL = previous;
      }
      await mock.close();
    }
  });

  test('fetchNugetVersions returns null when the package id is not registered', async () => {
    const mock = await startNugetAndGithubMock({
      versions: null,
      githubReleaseStatus: 404,
    });
    const previous = process.env.NUGET_INDEX_URL;
    process.env.NUGET_INDEX_URL = mock.nugetUrl;
    try {
      const versions = await fetchNugetVersions('NotRegistered');
      expect(versions).toBeNull();
    } finally {
      if (previous === undefined) {
        delete process.env.NUGET_INDEX_URL;
      } else {
        process.env.NUGET_INDEX_URL = previous;
      }
      await mock.close();
    }
  });

  test('fetchGithubReleaseExists returns true when the GitHub release exists', async () => {
    const mock = await startNugetAndGithubMock({
      versions: ['1.0.0'],
      githubReleaseStatus: 200,
    });
    const previous = process.env.GITHUB_API_URL;
    process.env.GITHUB_API_URL = mock.githubUrl;
    try {
      const exists = await fetchGithubReleaseExists(
        'link-foundation/csharp-ai-driven-development-pipeline-template',
        'csharp_v1.0.0'
      );
      expect(exists).toBe(true);
    } finally {
      if (previous === undefined) {
        delete process.env.GITHUB_API_URL;
      } else {
        process.env.GITHUB_API_URL = previous;
      }
      await mock.close();
    }
  });

  test('fetchGithubReleaseExists returns false on HTTP 404', async () => {
    const mock = await startNugetAndGithubMock({
      versions: ['1.0.0'],
      githubReleaseStatus: 404,
    });
    const previous = process.env.GITHUB_API_URL;
    process.env.GITHUB_API_URL = mock.githubUrl;
    try {
      const exists = await fetchGithubReleaseExists(
        'link-foundation/csharp-ai-driven-development-pipeline-template',
        'csharp_v9.9.9'
      );
      expect(exists).toBe(false);
    } finally {
      if (previous === undefined) {
        delete process.env.GITHUB_API_URL;
      } else {
        process.env.GITHUB_API_URL = previous;
      }
      await mock.close();
    }
  });

  test('fetchGithubReleaseExists returns false when repository is empty', async () => {
    const exists = await fetchGithubReleaseExists('', 'csharp_v1.0.0');
    expect(exists).toBe(false);
  });

  test('end-to-end: missing NuGet version + missing GitHub release triggers self-heal', async () => {
    const mock = await startNugetAndGithubMock({
      versions: ['2.2.0', '2.2.1', '2.2.2'],
      githubReleaseStatus: 404,
    });
    const prevNuget = process.env.NUGET_INDEX_URL;
    const prevGithub = process.env.GITHUB_API_URL;
    process.env.NUGET_INDEX_URL = mock.nugetUrl;
    process.env.GITHUB_API_URL = mock.githubUrl;
    try {
      const versions = await fetchNugetVersions('MyPackage');
      const exists = await fetchGithubReleaseExists(
        'link-foundation/csharp-ai-driven-development-pipeline-template',
        'csharp_v2.4.0'
      );
      const result = decide({
        hasChangesets: false,
        currentVersion: '2.4.0',
        publishedVersions: versions,
        githubReleaseExists: exists,
      });

      expect(result.shouldRelease).toBe(true);
      expect(result.skipBump).toBe(true);
      expect(result.nugetPublished).toBe(false);
      expect(result.githubReleaseExists).toBe(false);
      expect(result.reason).toMatch(/not yet published on NuGet/);
    } finally {
      if (prevNuget === undefined) {
        delete process.env.NUGET_INDEX_URL;
      } else {
        process.env.NUGET_INDEX_URL = prevNuget;
      }
      if (prevGithub === undefined) {
        delete process.env.GITHUB_API_URL;
      } else {
        process.env.GITHUB_API_URL = prevGithub;
      }
      await mock.close();
    }
  });

  test('end-to-end: NuGet has the version and GitHub release exists → no-op', async () => {
    const mock = await startNugetAndGithubMock({
      versions: ['2.3.0', '2.4.0'],
      githubReleaseStatus: 200,
    });
    const prevNuget = process.env.NUGET_INDEX_URL;
    const prevGithub = process.env.GITHUB_API_URL;
    process.env.NUGET_INDEX_URL = mock.nugetUrl;
    process.env.GITHUB_API_URL = mock.githubUrl;
    try {
      const versions = await fetchNugetVersions('MyPackage');
      const exists = await fetchGithubReleaseExists(
        'link-foundation/csharp-ai-driven-development-pipeline-template',
        'csharp_v2.4.0'
      );
      const result = decide({
        hasChangesets: false,
        currentVersion: '2.4.0',
        publishedVersions: versions,
        githubReleaseExists: exists,
      });

      expect(result.shouldRelease).toBe(false);
      expect(result.skipBump).toBe(false);
      expect(result.nugetPublished).toBe(true);
      expect(result.githubReleaseExists).toBe(true);
      expect(result.reason).toMatch(/no release needed/);
    } finally {
      if (prevNuget === undefined) {
        delete process.env.NUGET_INDEX_URL;
      } else {
        process.env.NUGET_INDEX_URL = prevNuget;
      }
      if (prevGithub === undefined) {
        delete process.env.GITHUB_API_URL;
      } else {
        process.env.GITHUB_API_URL = prevGithub;
      }
      await mock.close();
    }
  });
});
