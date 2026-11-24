# SWE-bench Lite dataset preparation

The latest [scheduled external benchmark run](https://github.com/link-assistant/formal-ai/actions/runs/45197910485)
reached all ten suites. Line 3510 of the workflow log reports that
`pyarrow.parquet` was missing when Formal AI tried to decode the pinned upstream
SWE-bench Lite dataset. The Rust runner checks the Python import before fetching
its Parquet rows. The existing `--allow-install` path applies to the downstream
evaluator, which the missing dataset prevented the run from reaching.

Both hosted paths use Python 3.12 on Linux x86-64. They install the same
[PyArrow 26.0.1 wheel](https://pypi.org/project/pyarrow/26.0.1/#files) with a
required SHA-256 digest and verify the import before invoking the benchmark.
Ordinary suite selections do neither. If preparing SWE-bench Lite fails, the
scheduled job retains the other nine suite results and returns failure. The
sharded path retains the setup error in the shard log and returns failure.

The shell regression in `issue-1569-workflow-failure-isolation.test.mjs`
failed before the setup was added. It now verifies installation and import order
with the actual workflow shell, both SWE-bench invocation paths, an unrelated
suite selection, and installer failure before evaluation. The exact wheel
was installed with hash enforcement under CPython 3.12 and its Parquet API
decoded one written fixture row into the original record.

The latest main results retain separate score failures. HumanEval passed
12 of 20, MBPP passed 10 of 20, and GSM8K passed one of 20, below their
recorded floors. Their expected scores and the SWE-bench evaluator acquisition
are unchanged. Hosted evaluation of the actual dataset and official evaluator
remains pending.
