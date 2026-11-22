# Current hosted JavaScript costs

These are all 916 queued files from Layered CI run 45018199621 at c90f94ca.
The complete run succeeded. Shard 3 used 296 seconds of its unchanged 360-second
budget, crossing the unchanged 270-second warning threshold.

915 rows come from Node file summaries. The assertion-only dialog consumer has
no Node file summary; its separate row retains the actual spec file-pass duration
and source line, explicitly marked `node-spec-file-pass-without-summary`.
Every raw observation, including that distinction, is retained. The replay's
four-worker model and 30-second allowance predict scheduling; they do not prove
execution of a changed plan or independently authored work.

The proposed weights keep every previous floor and apply a 65% scheduling reserve
to this cohort. A finite probe considered 13 reserves from 40% through 100%; 65%
is the first that fits all ten retained cohort checks with their original
allowances. This planning reserve is not a measured runtime or a larger budget.
The observed run used the previous 40% policy. Hosted execution must verify the
changed plan; no bound on future runtime is inferred from the model.

The prior scheduling file is retained verbatim from the observed commit. Its
explicit floors and 0.34-second default are preserved. 226 small new observations
remain in `durations.jsonl` and use that existing default in the scheduling table,
avoiding redundant records that would exceed the source-file limit.
