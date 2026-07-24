import { captureTuiTranscript } from './tui-transcript.mjs';

const required = (name) => {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required`);
  return value;
};

const question = 'What would you like to report? Select one or more.';
const issueUrl = required('ISSUE819_REPORT_URL');
const transcript = await captureTuiTranscript({
  command: required('ISSUE819_TUI_COMMAND'),
  cwd: required('ISSUE819_TUI_CWD'),
  environment: {
    PATH: required('ISSUE819_TUI_PATH'),
  },
  interactions: [
    {
      after: question,
      inputs: ['1', '2', '3', '\t', '\r'],
      delayMs: 150,
    },
  ],
  stopMarker: issueUrl,
  outputPath: required('ISSUE819_TUI_OUTPUT'),
  timeoutMs: 120_000,
});

const rendered = transcript.sequence.join('\n');
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
if (transcript.interaction_count !== 1) {
  throw new Error('OpenCode did not reach the report multi-select interaction');
}
if (!transcript.stop_marker_seen) {
  throw new Error('OpenCode TUI ended before displaying the created issue URL');
}
