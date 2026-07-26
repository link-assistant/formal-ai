import { captureTuiTranscript } from './tui-transcript.mjs';

const required = (name) => {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required`);
  return value;
};

const expectedResult = required('ISSUE819_EXPECT_RESULT');
const transcript = await captureTuiTranscript({
  command: required('ISSUE819_TUI_COMMAND'),
  cwd: required('ISSUE819_TUI_CWD'),
  environment: {
    FORMAL_AI_DESKTOP_DIR: required('ISSUE819_DESKTOP_DIR'),
  },
  stopMarker: expectedResult,
  stopMarkerOccurrences: expectedResult.startsWith('No matching') ? 1 : 2,
  outputPath: required('ISSUE819_TUI_OUTPUT'),
});

const rendered = transcript.sequence.join('\n');
for (const expected of [
  'Find hive-mind-control center folder on my desktop',
  'find',
  expectedResult,
]) {
  if (!rendered.includes(expected)) {
    throw new Error(`TUI transcript omitted ${JSON.stringify(expected)}\n${rendered}`);
  }
}
if (!transcript.stop_marker_seen) {
  throw new Error('OpenCode TUI ended before rendering the expected result');
}
