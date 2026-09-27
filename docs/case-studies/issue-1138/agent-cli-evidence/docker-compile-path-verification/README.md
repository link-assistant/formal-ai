# Formal AI docker-compile-path verification sessions (issue #1151)

External Agent CLI, local `formal-ai/formal-ai` model at
`formal-ai/0.351.0`, 2026-09-27. The server binary is
`/tmp/formal-ai-target-incr/debug/formal-ai`, the same build verified for
PR #1144, #1146 and #1148; the Dockerfile fix changes no Rust source, so
the served model behavior is unchanged. No hosted model participated.

## Context

Main run 36278582429 (merge of PR #1148, `ce89b4b41`, cutting v0.352.0)
published the crate to crates.io, waited for availability, and passed the
published-crate smoke test — then failed the GHCR image push at the final
`COPY --from=selected-binary /app/target/release/formal-ai`: the compile
stage builds with `--manifest-path rust/Cargo.toml`, so after the Plan 16
L1 move cargo emits the binary to `rust/target/release`, and only the
prebuilt stage had been taught to stage it at the contract path (CI
recovery batch H). The fix makes the compile stage copy its binary to
`/app/target/release/formal-ai` too and states the shared contract in the
Dockerfile. The sessions below had the local Formal AI model read the
fixed Dockerfile.

## Prompt-shape note (routing observation)

The Dockerfile was staged in the session workspace as `image-recipe.txt`.
Under its own name it is unreachable for read-intent prompts: three
batches (ports 8802/8803/8806) of "Read the file Dockerfile in this
workspace and quote ..." prompts — six phrasings, including the exact
grammar that produces genuine reads for `coverage.yml` — all routed to a
single `list` call whose output the model then narrated as the answer; the
same grammar against the same bytes named `image-recipe.txt` reads the
file in round one ("Let me open image-recipe.txt and read what it says."),
reproducing immediately on re-run while a same-minute `coverage.yml`
control still read fine. The `Dockerfile` token itself steers the router
to list. That is a real capability-routing gap in the 0.351.0 model,
recorded here for the routing work; the sessions below are the renamed
batch and the failed batches are not counted.

## Sessions

All three prompts are of the form "Read the file image-recipe.txt in this
workspace and quote ... in it." Every session ends `hasError: false`,
HTTP 200 on all requests, with the file's contents in the final assistant
turn.

1. `ses_f1fbe376fffedFNoqjRfluI5u3` — asked to quote the value of the
   `manifest-path` argument. The returned contents carry
   `--manifest-path rust/Cargo.toml` on both `cargo build --release`
   invocations — the lines that put cargo's output under
   `rust/target/release` and motivated the fix.
2. `ses_f1fbe466cffeUjQCTg1cH52TYZ` — asked to quote the `RUN cp` line.
   The returned contents include
   `cp rust/target/release/formal-ai target/release/formal-ai` inside the
   `compile-binary` stage — the fix itself.
3. `ses_f1fbe6309ffeeFB0osGjg01UBH` — asked to quote the destination of
   the final `COPY` instruction. The returned contents include
   `COPY --from=selected-binary /app/target/release/formal-ai
   /usr/local/bin/formal-ai` — the step run 36278582429 failed on.

## Mechanical confirmation

The human-side verification recorded in issue #1151 and the PR: a local
`docker build` of the full compile path (default `BINARY_SOURCE=compile`,
`rust:1.98-slim` builder, engine v29.5.3) completed exit 0 — the new
`compile-binary` layer ran, the previously failing
`COPY --from=selected-binary /app/target/release/formal-ai` resolved, and
the image's final layer executed `formal-ai --version` →
`formal-ai 0.352.0`. The PR-side `docker-build` check (prebuilt leg)
re-runs as well: the Dockerfile is in its green-ledger path set.
