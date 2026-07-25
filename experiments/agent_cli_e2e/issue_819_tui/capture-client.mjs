import { captureAgentTui } from 'agent-commander';

const required = (name) => {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required`);
  return value;
};

const jsonArray = (name) => {
  const value = process.env[name];
  if (!value) return [];
  const parsed = JSON.parse(value);
  if (!Array.isArray(parsed)) throw new Error(`${name} must be a JSON array`);
  return parsed;
};

const tool = required('ISSUE819_TUI_CLIENT');
const expectedResult = required('ISSUE819_EXPECT_RESULT');
const promptAfter = {
  opencode: 'Ask anything...',
  claude: 'Tips for getting started',
  codex: 'formal-ai default',
}[tool];
const startupInteractions =
  {
    claude: [
      { after: 'Enter y/n:', text: 'y', key: 'ENTER' },
      { after: 'y. Yes, I accept', text: 'y', key: 'ENTER' },
    ],
    codex: [{ after: 'Press enter to continue', key: 'ENTER' }],
  }[tool] ?? [];
const capture = await captureAgentTui({
  tool,
  workingDirectory: required('ISSUE819_TUI_CWD'),
  executable: required('ISSUE819_TUI_EXECUTABLE'),
  prefixArgs: jsonArray('ISSUE819_TUI_PREFIX_ARGS'),
  extraArgs: jsonArray('ISSUE819_TUI_EXTRA_ARGS'),
  model: tool === 'opencode' ? 'formal-ai/formal-ai' : undefined,
  prompt: required('ISSUE819_TUI_PROMPT'),
  promptAfter,
  startupInteractions,
  extraEnv: {
    FORMAL_AI_DESKTOP_DIR: required('ISSUE819_DESKTOP_DIR'),
  },
  cols: 120,
  rows: 12,
  interactions: [
    {
      after: expectedResult,
      resize: { cols: 132, rows: 14 },
    },
  ],
  stopMarker: expectedResult,
  stopMarkerGraceMilliseconds: 4_000,
  timeoutMilliseconds: 120_000,
  artifactDirectory: required('ISSUE819_TUI_ARTIFACT_DIR'),
});

const rendered = capture.transcript;
for (const expected of [
  'Find hive-mind-control center folder on my desktop',
  'find',
  expectedResult,
]) {
  if (!rendered.includes(expected)) {
    throw new Error(
      `TUI transcript omitted ${JSON.stringify(expected)}\n${rendered}`,
    );
  }
}
if (!capture.output.includes(expectedResult)) {
  throw new Error(`${tool} TUI ended before rendering the expected result`);
}
const expectedInteractions = startupInteractions.length + 2;
if (capture.interactionCount !== expectedInteractions) {
  throw new Error(
    `${tool} TUI applied ${capture.interactionCount}/${expectedInteractions} interactions`,
  );
}
const finalFrame = capture.frames.at(-1);
if (
  tool === 'opencode' &&
  (!finalFrame ||
    finalFrame.lines.length <= finalFrame.screen.length ||
    finalFrame.screen.join('\n').includes(required('ISSUE819_TUI_PROMPT')))
) {
  throw new Error(
    'OpenCode TUI did not preserve scrollback beyond its final viewport',
  );
}

process.stdout.write(`${rendered}\n`);
