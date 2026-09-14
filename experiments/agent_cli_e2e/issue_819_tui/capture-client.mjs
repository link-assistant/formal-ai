import { captureAgentTui } from 'agent-commander';
import { appendFile, readFile } from 'node:fs/promises';
import { join } from 'node:path';

import {
  promptAfterFor,
  startupInteractionsFor,
} from './client-contract.mjs';

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
const prompt = required('ISSUE819_TUI_PROMPT');
const artifactDirectory = required('ISSUE819_TUI_ARTIFACT_DIR');
const promptAfter = promptAfterFor(tool);
const startupInteractions = startupInteractionsFor(tool);
const capture = await captureAgentTui({
  tool,
  workingDirectory: required('ISSUE819_TUI_CWD'),
  executable: required('ISSUE819_TUI_EXECUTABLE'),
  prefixArgs: jsonArray('ISSUE819_TUI_PREFIX_ARGS'),
  extraArgs: jsonArray('ISSUE819_TUI_EXTRA_ARGS'),
  model: tool === 'opencode' ? 'formal-ai/formal-ai' : undefined,
  prompt,
  promptAfter,
  startupInteractions,
  extraEnv: {
    FORMAL_AI_DESKTOP_DIR: required('ISSUE819_DESKTOP_DIR'),
  },
  interactions: [
    {
      after: expectedResult,
      resize: { cols: 96, rows: 36 },
    },
  ],
  stopMarker: expectedResult,
  stopMarkerGraceMilliseconds: 4_000,
  timeoutMilliseconds: 120_000,
  artifactDirectory,
  artifactOptions: { borderRadius: 0 },
});

const rendered = capture.transcript;
const visibleToolMarker =
  tool === 'claude' ? 'Searching for 1 pattern' : 'find';
for (const expected of [prompt, visibleToolMarker, expectedResult]) {
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
  (!finalFrame || !finalFrame.lines.join('\n').includes(prompt))
) {
  throw new Error('OpenCode TUI omitted the prompt from terminal history');
}

if (
  capture.asciicast.header.width !== 80 ||
  capture.asciicast.header.height !== 30
) {
  throw new Error(
    `${tool} TUI started at ${capture.asciicast.header.width}x${capture.asciicast.header.height}, expected 80x30`,
  );
}

const artifacts = [
  'transcript.txt',
  'frames.json',
  'session.cast',
  'snapshot.svg',
  'recording.svg',
  'recording.gif',
];
for (const artifact of artifacts) {
  if ((await readFile(join(artifactDirectory, artifact))).length === 0) {
    throw new Error(`${tool} TUI wrote an empty ${artifact}`);
  }
}
const recording = await readFile(
  join(artifactDirectory, 'recording.svg'),
  'utf8',
);
const snapshot = await readFile(
  join(artifactDirectory, 'snapshot.svg'),
  'utf8',
);
const hasPaddedTextRun = (svg) =>
  [...svg.matchAll(/<text\b[^>]*>([^<]*)<\/text>/gu)].some(
    ([, text]) => /^\s|\s$/u.test(text),
  );
const rendererFeatures = {
  css_keyframes:
    recording.includes('@keyframes') &&
    recording.includes('steps(1, end)') &&
    !recording.includes('<animate'),
  embedded_font:
    recording.includes('@font-face') &&
    recording.includes('data:font/woff2;base64,'),
  exact_cell_grid:
    recording.includes('textLength=') &&
    recording.includes('lengthAdjust="spacingAndGlyphs"'),
  preserved_whitespace: recording.includes('xml:space="preserve"'),
  square_terminal_frame: recording.includes('rx="0"'),
  visible_text_geometry:
    !hasPaddedTextRun(snapshot) && !hasPaddedTextRun(recording),
};
for (const [feature, present] of Object.entries(rendererFeatures)) {
  if (!present) {
    throw new Error(`${tool} TUI recording omitted ${feature}`);
  }
}
const gifHeader = (
  await readFile(join(artifactDirectory, 'recording.gif'))
)
  .subarray(0, 6)
  .toString();
if (gifHeader !== 'GIF89a') {
  throw new Error(`${tool} TUI wrote invalid GIF header ${gifHeader}`);
}

const observationFile = process.env.ISSUE819_TUI_OBSERVATION_FILE;
if (observationFile) {
  const observation = {
    client_id: tool,
    capability: 'tui_replay',
    task_wording: prompt,
    delivery: 'tool_call',
    advertised_tools: [],
    invoked_tools: [],
    observed_contract: {
      tui_initial_geometry: ['80x30'],
      tui_artifact: artifacts,
      tui_renderer_feature: [
        ...Object.keys(rendererFeatures),
        'gif_fallback',
      ],
    },
    evidence:
      process.env.ISSUE819_TUI_EVIDENCE ??
      join(artifactDirectory, 'recording.svg'),
  };
  await appendFile(observationFile, `${JSON.stringify(observation)}\n`);
}

process.stdout.write(`${rendered}\n`);
