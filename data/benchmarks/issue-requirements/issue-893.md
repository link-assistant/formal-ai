Parent: #710

Audit verdict: still-broken.

Current evidence
- The current summarization pipeline supports recursive embedded grammars, exact source captures, checked contexts, and deterministic modes.
- It does not sample two random repository files repeatedly until stable or enforce the original 80 percent quality bar from issue 563.

Acceptance
- Define a reproducible seeded sampling protocol over repository files.
- Validate two files per iteration until the result stabilizes or a reported bound is reached.
- Define and publish the quality metric with an 80 percent minimum ratchet.
- Exercise recursive Markdown embedded grammars through the production summarizer.
