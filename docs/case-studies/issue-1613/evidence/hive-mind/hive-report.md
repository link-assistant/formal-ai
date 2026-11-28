The issue-created draft workflow can select a Hive Mind version from main before the corresponding Docker manifest is published. The selected tag is correct, but the first `manifest unknown` is treated as permanent. No draft container or serving identity can be started.

On 2026-11-21, both [Hive Mind run 44922819938](https://github.com/link-assistant/hive-mind/actions/runs/44922819938) at 16:09:08 UTC and [Formal AI run 44923287323](https://github.com/link-assistant/formal-ai/actions/runs/44923287323) at 16:16:53 UTC failed with `DRAFT_IMAGE_UNAVAILABLE` after one attempt to pull `konard/hive-mind:2.45.1`. Formal AI bound that image to Hive commit `3e9dcff2e1dac101b9d7825e79ebccfccea21dba`. GitHub records the [2.45.1 release](https://github.com/link-assistant/hive-mind/releases/tag/v2.45.1) as published at 16:24:21 UTC. A later `docker manifest inspect konard/hive-mind:2.45.1` succeeded. The exact registry publication time is unknown.

## Reproduce

Open an automation-enabled issue after the version bump reaches main and before the version's Docker manifest is published. Watch the issue-created workflow select the new version, fail its pull once and stop. To exercise the same branch without timing a release, check out `3e9dcff2e1dac101b9d7825e79ebccfccea21dba` and run the following with Node.

```sh
node --input-type=module <<'JS'
import assert from 'node:assert/strict';
import { pullDraftImage } from './scripts/formal-ai-draft-image.lib.mjs';

let pulls = 0;
await assert.rejects(pullDraftImage({
  version: '2.45.1',
  run: async () => ++pulls === 1
    ? { code: 1, output: 'manifest unknown: manifest unknown' }
    : { code: 0, output: '' },
  sleep: async () => {},
  delays: [0],
}), { code: 'DRAFT_IMAGE_UNAVAILABLE', attempts: 1 });
assert.equal(pulls, 1);
JS
```

## Workaround and proposed change

Wait for `docker manifest inspect konard/hive-mind:<selected-version>` to succeed, then retry the admitted attempt with the same recorded source and verify the image and serving identity. An attempt that selects a new source should get a new identity.

Use bounded waits for the exact selected image when a fresh release is still publishing, or resume the attempt after a verified publication signal. Keep access-denied failures distinct from a not-yet-published manifest. Preserve the immutable image binding and fail with a specific deferred outcome after the wait budget. Add a fail-then-succeed test for the same version, plus missing-through-budget and access-denied cases.

Related to [Formal AI #1613](https://github.com/link-assistant/formal-ai/issues/1613) and the current-source contract in [Hive Mind #3909](https://github.com/link-assistant/hive-mind/issues/3909).
