#!/usr/bin/env node

/**
 * Install a published NuGet package into a throwaway console project and run a
 * small consumer smoke test against the template's public library API.
 *
 * Usage:
 *   node scripts/smoke-test-nuget-package.mjs \
 *     --package-id <id> --release-version <version>
 *
 * Optional arguments:
 *   --source <url-or-path>           Defaults to NuGet.org.
 *   --framework <tfm>                Defaults to net8.0.
 *   --library-namespace <namespace>  Defaults to MyPackage.
 *   --max-attempts <count>           Defaults to 5.
 *   --sleep-seconds <count>          Defaults to 60.
 */

import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const DEFAULT_SOURCE = 'https://api.nuget.org/v3/index.json';
export const DEFAULT_FRAMEWORK = 'net8.0';
export const DEFAULT_LIBRARY_NAMESPACE = 'MyPackage';
export const DEFAULT_MAX_ATTEMPTS = 5;
export const DEFAULT_SLEEP_SECONDS = 60;

function readCliOptions(argv) {
  const options = {};

  for (let index = 0; index < argv.length; index++) {
    const arg = argv[index];
    if (!arg.startsWith('--')) {
      continue;
    }

    const inlineValueIndex = arg.indexOf('=');
    if (inlineValueIndex !== -1) {
      options[arg.slice(2, inlineValueIndex)] = arg.slice(inlineValueIndex + 1);
      continue;
    }

    const value = argv[index + 1];
    if (value === undefined || value.startsWith('--')) {
      throw new Error(`Missing value for ${arg}`);
    }

    options[arg.slice(2)] = value;
    index++;
  }

  return options;
}

function parsePositiveInteger(value, optionName) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 1) {
    throw new Error(`${optionName} must be a positive integer`);
  }
  return parsed;
}

export function parseArgs(argv, env = process.env) {
  const cliOptions = readCliOptions(argv);

  return {
    framework:
      cliOptions.framework ||
      env.NUGET_SMOKE_FRAMEWORK ||
      DEFAULT_FRAMEWORK,
    libraryNamespace:
      cliOptions['library-namespace'] ||
      env.NUGET_SMOKE_LIBRARY_NAMESPACE ||
      env.PACKAGE_NAMESPACE ||
      DEFAULT_LIBRARY_NAMESPACE,
    maxAttempts: parsePositiveInteger(
      cliOptions['max-attempts'] ||
        env.NUGET_SMOKE_MAX_ATTEMPTS ||
        env.MAX_ATTEMPTS ||
        String(DEFAULT_MAX_ATTEMPTS),
      '--max-attempts'
    ),
    packageId: cliOptions['package-id'] || env.PACKAGE_ID || '',
    releaseVersion:
      cliOptions['release-version'] ||
      cliOptions.version ||
      env.RELEASE_VERSION ||
      env.VERSION ||
      '',
    sleepSeconds: parsePositiveInteger(
      cliOptions['sleep-seconds'] ||
        env.NUGET_SMOKE_SLEEP_SECONDS ||
        env.SLEEP_SECONDS ||
        String(DEFAULT_SLEEP_SECONDS),
      '--sleep-seconds'
    ),
    source:
      cliOptions.source ||
      env.NUGET_SMOKE_SOURCE ||
      env.NUGET_SOURCE ||
      DEFAULT_SOURCE,
  };
}

function validateNamespace(namespace) {
  if (!/^[_A-Za-z][_A-Za-z0-9]*(\.[_A-Za-z][_A-Za-z0-9]*)*$/.test(namespace)) {
    throw new Error(`Invalid C# namespace for smoke test: ${namespace}`);
  }
}

function toCSharpStringLiteral(value) {
  return JSON.stringify(String(value));
}

function getExpectedPackageInfoVersion(releaseVersion) {
  return String(releaseVersion).split('+')[0];
}

export function buildSmokeTestProgram({ expectedVersion, libraryNamespace }) {
  validateNamespace(libraryNamespace);

  return `using ${libraryNamespace};

var sum = Calculator.Add(2, 3);
if (sum != 5)
{
    throw new InvalidOperationException($"Calculator.Add returned {sum}, expected 5.");
}

var product = Calculator.Multiply(2, 3);
if (product != 6)
{
    throw new InvalidOperationException($"Calculator.Multiply returned {product}, expected 6.");
}

var actualVersion = PackageInfo.Version;
var expectedVersion = ${toCSharpStringLiteral(expectedVersion)};
if (actualVersion != expectedVersion)
{
    throw new InvalidOperationException($"PackageInfo.Version returned {actualVersion}, expected {expectedVersion}.");
}

Console.WriteLine($"Smoke test loaded ${libraryNamespace} {actualVersion}.");
`;
}

function sleep(seconds) {
  return new Promise((resolve) => {
    globalThis.setTimeout(resolve, seconds * 1000);
  });
}

function defaultRunCommand(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd,
    encoding: 'utf-8',
    env: options.env ? { ...process.env, ...options.env } : process.env,
    maxBuffer: 20 * 1024 * 1024,
  });

  if (result.error) {
    return {
      status: 1,
      stdout: result.stdout ?? '',
      stderr: result.error.message,
    };
  }

  return {
    status: result.status ?? 1,
    stdout: result.stdout ?? '',
    stderr: result.stderr ?? '',
  };
}

function formatCommand(command, args) {
  return [command, ...args].join(' ');
}

function previewCapturedOutput(output, maxLines = 20) {
  const trimmed = String(output ?? '').trim();
  if (!trimmed) {
    return '';
  }

  const lines = trimmed.split(/\r?\n/);
  const preview = lines.slice(0, maxLines).join('\n');
  if (lines.length <= maxLines) {
    return preview;
  }

  return `${preview}\n... (${lines.length - maxLines} more captured line(s))`;
}

function logCapturedOutput(label, result, stdout) {
  const stdoutPreview = previewCapturedOutput(result.stdout);
  if (stdoutPreview) {
    stdout(`${label} stdout:\n${stdoutPreview}`);
  }

  const stderrPreview = previewCapturedOutput(result.stderr);
  if (stderrPreview) {
    stdout(`${label} stderr:\n${stderrPreview}`);
  }
}

function formatCommandFailure(command, args, result) {
  const output = [
    `Command failed with exit code ${result.status}: ${formatCommand(command, args)}`,
  ];
  const stdoutPreview = previewCapturedOutput(result.stdout);
  const stderrPreview = previewCapturedOutput(result.stderr);

  if (stdoutPreview) {
    output.push(`stdout:\n${stdoutPreview}`);
  }
  if (stderrPreview) {
    output.push(`stderr:\n${stderrPreview}`);
  }

  return output.join('\n');
}

function runCheckedCommand({ args, command = 'dotnet', cwd, runCommand, stdout }) {
  stdout(`Running: ${formatCommand(command, args)}`);
  const result = runCommand(command, args, { cwd });
  logCapturedOutput(formatCommand(command, args), result, stdout);

  if (result.status !== 0) {
    throw new Error(formatCommandFailure(command, args, result));
  }

  return result;
}

async function installPackageWithRetries({
  config,
  cwd,
  runCommand,
  sleepFn,
  stdout,
}) {
  const args = [
    'add',
    'package',
    config.packageId,
    '--version',
    config.releaseVersion,
    '--source',
    config.source,
  ];
  let lastResult = null;

  for (let attempt = 1; attempt <= config.maxAttempts; attempt++) {
    stdout(
      `Installing ${config.packageId}@${config.releaseVersion} ` +
        `(attempt ${attempt}/${config.maxAttempts})`
    );

    const result = runCommand('dotnet', args, { cwd });
    logCapturedOutput('dotnet add package', result, stdout);

    if (result.status === 0) {
      return;
    }

    lastResult = result;
    if (attempt < config.maxAttempts) {
      stdout(`Waiting ${config.sleepSeconds}s before retrying package install`);
      await sleepFn(config.sleepSeconds);
    }
  }

  throw new Error(
    `Unable to install ${config.packageId}@${config.releaseVersion} ` +
      `after ${config.maxAttempts} attempt(s).\n` +
      formatCommandFailure('dotnet', args, lastResult)
  );
}

export async function smokeTestNugetPackage({
  config,
  makeTempDir = () => mkdtempSync(path.join(tmpdir(), 'nuget-consumer-smoke-')),
  runCommand = defaultRunCommand,
  sleepFn = sleep,
  stdout = console.log,
} = {}) {
  const tempDir = makeTempDir();
  stdout(`Created throwaway NuGet consumer project at ${tempDir}`);

  try {
    runCheckedCommand({
      args: ['new', 'console', '--framework', config.framework, '--output', '.'],
      cwd: tempDir,
      runCommand,
      stdout,
    });

    await installPackageWithRetries({
      config,
      cwd: tempDir,
      runCommand,
      sleepFn,
      stdout,
    });

    writeFileSync(
      path.join(tempDir, 'Program.cs'),
      buildSmokeTestProgram({
        expectedVersion: getExpectedPackageInfoVersion(config.releaseVersion),
        libraryNamespace: config.libraryNamespace,
      })
    );

    runCheckedCommand({
      args: ['build', '--configuration', 'Release', '--no-restore'],
      cwd: tempDir,
      runCommand,
      stdout,
    });

    runCheckedCommand({
      args: ['run', '--configuration', 'Release', '--no-build'],
      cwd: tempDir,
      runCommand,
      stdout,
    });
  } finally {
    rmSync(tempDir, { force: true, recursive: true });
  }
}

export async function main({
  argv = process.argv.slice(2),
  env = process.env,
  stderr = console.error,
  stdout = console.log,
} = {}) {
  try {
    const config = parseArgs(argv, env);
    if (!config.packageId || !config.releaseVersion) {
      stderr('Error: --package-id and --release-version are required');
      return 1;
    }

    await smokeTestNugetPackage({ config, stdout });
    stdout(
      `NuGet consumer smoke test passed for ` +
        `${config.packageId}@${config.releaseVersion}`
    );
    return 0;
  } catch (error) {
    stderr(`Error: ${error.message}`);
    return 1;
  }
}

const invokedDirectly = process.argv[1]
  && fileURLToPath(import.meta.url) === path.resolve(process.argv[1]);

if (invokedDirectly) {
  process.exitCode = await main();
}
