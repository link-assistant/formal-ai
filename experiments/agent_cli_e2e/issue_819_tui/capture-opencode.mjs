import { captureTuiTranscript } from './tui-transcript.mjs';

const required = (name) => {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required`);
  return value;
};

const transcript = await captureTuiTranscript({
  command: required('ISSUE819_TUI_COMMAND'),
  cwd: required('ISSUE819_TUI_CWD'),
  environment: {
    FORMAL_AI_DESKTOP_DIR: required('ISSUE819_DESKTOP_DIR'),
  },
  stopMarker: required('ISSUE819_EXPECT_PATH'),
  stopMarkerOccurrences: 2,
  outputPath: required('ISSUE819_TUI_OUTPUT'),
});

const rendered = transcript.sequence.join('\n');
for (const expected of [
  'Find hive-mind-control center folder on my desktop',
  'find',
  required('ISSUE819_EXPECT_PATH'),
]) {
  if (!rendered.includes(expected)) {
    throw new Error(`TUI transcript omitted ${JSON.stringify(expected)}\n${rendered}`);
  }
}
if (!transcript.stop_marker_seen) {
  throw new Error('OpenCode TUI ended before rendering the expected local path');
}
