"""Describe the measured final-review correction without claiming native execution."""
from pathlib import Path
p = Path('docs/case-studies/issue-1582/README.md')
s = p.read_text().replace('focused JavaScript tests in total.', 'focused JavaScript tests at that stage. Final transport review adds two deadline/cancellation regressions: the final issue suite has 12 passing tests, plus the 16 logger tests (28 focused core tests).')
s += '''
## Final transport deadline review

The second repair head, `ca57748ca7daf0882618977e23382a13e07b4d0d`,
passes all 22 hosted workflows. Its logs and both attached artifacts are retained.
Final source review then found worker source-capture calls that pass no abort
signal, plus a search helper that clears its timer when headers arrive. The new
Node network transport therefore needed its own overall deadline, including
response-body consumption. Two bounded regressions fail before this correction:
the combined deadline signal is absent and a deliberately unfinished body only
stops when the independent two-second probe guard closes the socket.

The Node transport now supplies a 30-second AbortSignal deadline and combines
it with caller cancellation without changing the caller's options. The maintained
TypeScript generator produces the matching twin. The whole issue suite passes
12 tests afterward, including actual stalled-body cancellation in about 275 ms
with a 250 ms test deadline. The guard, test timeout and server cleanup bound
both failing and passing probes. Native curl transport already has its own
finite IO bound and is unchanged. This correction changes no shared registry
data; the complete eighth local gate round remains the shared-data verification,
and the affected generated/source gates are checked again for this change.
'''
p.write_text(s)
p = Path('experiments/issue_1582/pull-request.md')
s = p.read_text().replace('final issue suite passes ten regressions', 'final issue suite passes twelve regressions').replace('**26 focused JavaScript tests pass**', '**28 focused JavaScript tests pass**')
s = s.replace('The next repair head still requires hosted CI verification.', 'The second repair head completes all 22 workflows successfully. Final review then adds a transport-wide 30-second deadline that remains active while reading the response body and preserves caller cancellation. Both new regressions fail beforehand; all 12 issue tests pass afterward. The affected source/generated gates are checked again. The final source head still requires hosted CI verification.')
p.write_text(s)
