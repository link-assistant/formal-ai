import { captureAgentTui } from 'agent-commander';

const required = (name) => {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required`);
  return value;
};

const question = 'What would you like to report? Select one or more.';
const issueUrl = required('ISSUE819_REPORT_URL');
const capture = await captureAgentTui({
  tool: 'opencode',
  workingDirectory: required('ISSUE819_TUI_CWD'),
  executable: required('ISSUE819_TUI_EXECUTABLE'),
  extraArgs: ['.'],
  model: 'formal-ai/formal-ai',
  prompt: 'Report',
  promptAfter: 'Ask anything...',
  extraEnv: {
    PATH: required('ISSUE819_TUI_PATH'),
  },
  interactions: [
    {
      after: question,
      text: '1',
    },
    {
      after: '[✓] Harness log',
      text: '2',
    },
    {
      after: '[✓] Server log',
      text: '3',
    },
    {
      after: '[✓] GitHub issue',
      key: 'TAB',
    },
    {
      after: 'Harness log, Server log, GitHub issue',
      key: 'ENTER',
    },
  ],
  stopMarker: issueUrl,
  stopMarkerGraceMilliseconds: 1_000,
  timeoutMilliseconds: 120_000,
  artifactDirectory: required('ISSUE819_TUI_ARTIFACT_DIR'),
});

const rendered = capture.transcript;
for (const expected of [
  'Report',
  question,
  'select all that apply',
  '[✓] Harness log',
  '[✓] Server log',
  '[✓] GitHub issue',
  'Harness log, Server log, GitHub issue',
  issueUrl,
]) {
  if (!rendered.includes(expected)) {
    throw new Error(
      `report TUI transcript omitted ${JSON.stringify(expected)}\n${rendered}`,
    );
  }
}
if (capture.interactionCount !== 6) {
  throw new Error('OpenCode did not reach the report multi-select interaction');
}
if (!capture.output.includes(issueUrl)) {
  throw new Error('OpenCode TUI ended before displaying the created issue URL');
}

process.stdout.write(`${rendered}\n`);
