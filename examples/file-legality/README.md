# File-legality report example

The sidecar is synthetic: `.example` URIs are placeholders, not legal sources
or detector claims. Replace every policy and observation with current evidence
from your own authorized integrations.

```bash
formal-ai file-legality ./candidate.jpg \
  --config examples/file-legality/evidence.json \
  --pretty
```

The report contains three independent assessments per jurisdiction. A risk
signal requests the matching policy action; a negative detector result says
only that its named detector found no signal. Missing evidence stays `unknown`,
and `verdict` remains `not_provided` in every case.

Do not put prohibited sample content or restricted provider databases in this
repository. Supply confirmed child-safety matches only as receipts from an
authorized integration and follow that provider's reporting requirements.
