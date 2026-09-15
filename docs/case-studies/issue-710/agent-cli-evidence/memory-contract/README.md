# Formal AI retention-contract formalization

External Agent CLI 0.26.0, local `formal-ai/formal-ai` model, session
`ses_f5a131ed8ffe4pk0sDxINN0ZSn`, 2026-09-15. The worktree build includes the
first repository-output composition increment, before reconstruction-metadata
retention. It is not the remote PR baseline binary. No hosted model participated.

The task asked Formal AI to formalize the five retention requirements preserved
verbatim in `knowledge-base.lino`, then verify the file. It completed three
HTTP/model rounds: write, read-back verification, final. Both client and harness
exited zero. The reviewed artifact is copied verbatim; full logs remain private
at `/private/tmp/formal-ai-888-memory-contract-8919`.

Independent inspection: all five requirements have annotations, source spans
and assertions. However, the output contains **zero concepts and procedures**,
and every relation is the generic `pred:states`. This is successful lossless
source recording, not evidence of semantic decomposition, prerequisite
discovery or implementation. Plan 06 keeps those acceptance criteria open.

After the report-coverage fix, external session
`ses_f59f07cb1ffeOToRIT5UmPECXS` repeated this task successfully in three
rounds and reported `2 of 9 protocol primitives`. The generated artifact's
SHA-256 is `0919701afc5c5bdfe6e9357a03adb2d54b1f419f6ef9cf47fbb1ec6e7135e452`;
the replay evidence stays private at
`/private/tmp/formal-ai-888-memory-contract-8921`. Read-back verification still
does not establish semantic understanding of the five requirements.
