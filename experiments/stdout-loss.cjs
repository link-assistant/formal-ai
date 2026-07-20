// Issue #808, hypothesis test (REJECTED).
//
// Hypothesis: the `[adhoc-sign-mac]` diagnostics never reached the CI log
// because process.stdout.write() to a pipe is buffered asynchronously and the
// pending buffer is discarded when electron-builder aborts the process.
//
// Result on Linux/macOS: NOT reproduced -- Node writes synchronously to pipes
// on POSIX, so all 200 stdout lines survive process.exit(1):
//
//   $ node experiments/stdout-loss.cjs > out.txt 2> err.txt
//   stdout lines kept: 200 / 200
//   stderr lines kept: 200 / 200
//
// The missing output therefore has a different cause (the hook's exclusion was
// not in effect at all -- see dev/log/issues/808/pulls/809/analysis.md §4).
// Diagnostics were still moved to stderr, because that is where
// electron-builder and electron-osx-sign write and interleaving keeps the sign
// trace readable.
const big = 'x'.repeat(200);
for (let i = 0; i < 200; i++) process.stdout.write(`STDOUT-${i} ${big}\n`);
for (let i = 0; i < 200; i++) process.stderr.write(`STDERR-${i} ${big}\n`);
process.exit(1);
