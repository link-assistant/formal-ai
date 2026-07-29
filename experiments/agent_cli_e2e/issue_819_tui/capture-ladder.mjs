import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join, resolve } from 'node:path';

import { captureAgentTui } from 'agent-commander';

const required = (name) => {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required`);
  return value;
};

const tasksPath = required('ISSUE840_TASKS');
const outPath = required('ISSUE840_OUT');
const sandbox = required('ISSUE840_SANDBOX');
const port = required('ISSUE840_PORT');
const only = process.env.ISSUE840_ONLY ?? '';
const artifactRoot =
  process.env.ISSUE840_ARTIFACT_DIR ?? `${outPath}.artifacts`;
const executable = process.env.OPENCODE ?? 'opencode';
const requireAllPass = process.env.REQUIRE_ALL_PASS === '1';
const data = JSON.parse(await readFile(tasksPath, 'utf8'));
const nodes = data.tasks.filter((task) => !only || task.id.includes(only));

await mkdir(artifactRoot, { recursive: true });
const configPath = join(sandbox, 'opencode.json');
await writeFile(
  configPath,
  `${JSON.stringify(
    {
      $schema: 'https://opencode.ai/config.json',
      provider: {
        'formal-ai': {
          npm: '@ai-sdk/openai-compatible',
          name: 'Formal AI',
          options: {
            baseURL: `http://127.0.0.1:${port}/v1`,
            apiKey: 'local',
          },
          models: {
            'formal-ai': { name: 'Formal AI Symbolic Production' },
          },
        },
      },
    },
    null,
    2,
  )}\n`,
);

const results = [];
for (const node of nodes) {
  const artifactDirectory = join(artifactRoot, node.id);
  let transcript = '';
  let error = null;
  try {
    const capture = await captureAgentTui({
      tool: 'opencode',
      workingDirectory: sandbox,
      executable,
      extraArgs: ['.'],
      model: 'formal-ai/formal-ai',
      prompt: node.prompt,
      promptAfter: 'Ask anything...',
      extraEnv: {
        FORMAL_AI_DESKTOP_DIR: join(sandbox, 'Desktop'),
        HOME: sandbox,
        OPENCODE_CONFIG: configPath,
      },
      cols: 120,
      rows: 14,
      // OpenCode inserts ANSI styling between "▣" and "Build". The trailing
      // completion footer is emitted as one stable byte sequence only after
      // the final assistant response, so it is safe for every ladder node.
      stopMarker: ' · Formal AI Symbolic Production · ',
      stopMarkerGraceMilliseconds: 1_000,
      timeoutMilliseconds: 120_000,
      artifactDirectory,
    });
    transcript = capture.transcript;
  } catch (captureError) {
    error = `${captureError.name}: ${captureError.message}`;
  }

  // The user's own prompt is visible in a TUI but is not part of the answer
  // being scored. Remove only that exact echo; all assistant and tool states
  // remain on the assertion surface.
  const answer = transcript.split(node.prompt).join('');
  const lower = answer.toLocaleLowerCase();
  const missing = (node.expect ?? []).filter(
    (expected) => !lower.includes(expected.toLocaleLowerCase()),
  );
  const leaked = (node.forbid ?? []).filter((forbidden) =>
    lower.includes(forbidden.toLocaleLowerCase()),
  );
  const pass = error === null && missing.length === 0 && leaked.length === 0;
  results.push({
    id: node.id,
    level: node.level,
    seed: node.seed,
    lang: node.lang,
    prompt: node.prompt,
    note: node.note ?? '',
    answer,
    error,
    missing_expect: missing,
    leaked_forbid: leaked,
    artifact_directory: resolve(artifactDirectory),
    pass,
  });

  const reason = error
    ? ` error=${error}`
    : missing.length
      ? ` missing=${JSON.stringify(missing)}`
      : leaked.length
        ? ` leaked=${JSON.stringify(leaked)}`
        : '';
  process.stdout.write(
    `${pass ? 'PASS' : 'FAIL'}  ${node.id.padEnd(12)} L${node.level}  ${JSON.stringify(node.prompt.slice(0, 58))}${reason}\n`,
  );
}

const passed = results.filter((result) => result.pass).length;
const summary = {
  transport: 'opencode-tui',
  total: results.length,
  passed,
  failed: results.length - passed,
  by_level: {},
  by_seed: {},
};
for (const result of results) {
  const level = (summary.by_level[`L${result.level}`] ??= {
    passed: 0,
    total: 0,
  });
  level.total += 1;
  level.passed += result.pass ? 1 : 0;
  const seed = (summary.by_seed[result.seed] ??= { passed: 0, total: 0 });
  seed.total += 1;
  seed.passed += result.pass ? 1 : 0;
}

await writeFile(
  outPath,
  `${JSON.stringify({ summary, results }, null, 2)}\n`,
);
process.stdout.write(
  `\nTOTAL ${passed}/${results.length} passed through the real OpenCode TUI\nwrote ${outPath}\nartifacts ${artifactRoot}\n`,
);
if (requireAllPass && passed !== results.length) process.exitCode = 1;
