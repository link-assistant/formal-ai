// Execution-ordered file completion events, alongside unchanged LCOV/spec reporters.
export default async function* browserDurationReporter(events) {
  for await (const event of events) {
    const data = event.data;
    if (event.type !== 'test:complete' || data?.nesting !== 0 || data.name !== data.file
        || data.line !== 1 || data.column !== 1) continue;
    yield `${JSON.stringify({ schema: 'browser-file-duration/v1', file: data.file,
      milliseconds: data.details?.duration_ms, passed: data.details?.passed === true,
      skipped: Boolean(data.skip), todo: Boolean(data.todo) })}\n`;
  }
  yield `${JSON.stringify({ schema: 'browser-file-duration-stream/v1', complete: true })}\n`;
}
