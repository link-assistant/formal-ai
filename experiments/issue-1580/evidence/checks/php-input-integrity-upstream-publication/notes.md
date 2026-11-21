Release-note truncation can emit invalid UTF-8 at its default byte limit.

Reproduced in [ReleaseNotes::capBytes](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/blob/b61e334703d93a15e08bf1a278e0df0a9d6e98a8/scripts/src/ReleaseNotes.php)
at b61e334703d93a15e08bf1a278e0df0a9d6e98a8; the relevant blob is identical to
the pinned 3a034b27a56067971ca74dd4c74c7707056b215d source.

After substr cuts the body to its byte budget, the loop removes trailing UTF-8
continuation bytes. It leaves the leading byte. This corrupts a complete final
multibyte character as well as a partially cut one. The retained test places é
exactly at the full body budget under the default 120000-byte cap; the original
output fails strict UTF-8 decoding. Separate two-, three- and four-byte cases
cover every partial cut and a complete-character boundary. ASCII and short
untruncated Unicode remain positive controls.

There is also a smaller-cap case: when max is below the notice length, the
original returns the entire notice and exceeds the requested cap. Controls use
caps 0, 1, 7 and notice-length minus one.

The [15-case maintained original/candidate file](https://github.com/link-assistant/formal-ai/blob/ac9cd6a55e3ea7d8738e6122f623d7676666630d/experiments/issue-1580/template-php-input-integrity.test.mjs) executes the pinned PHP
class directly without Composer installation:

```sh
node --test --test-concurrency=1 experiments/issue-1580/template-php-input-integrity.test.mjs
```

Run from Formal AI commit ac9cd6a55e3ea7d8738e6122f623d7676666630d. Each PHP child
has a five-second boundary, 32 MiB memory limit and bounded input/output. Original
invalid output is checked with a fatal UTF-8 decoder. A disposable candidate
trims to an actual valid boundary and caps an oversized ASCII fixture notice;
it preserves the existing ASCII/short Unicode controls. That candidate is not an
upstream patch and does not claim arbitrary Unicode notice/URL qualification.

Please preserve a valid UTF-8 boundary and the requested byte limit for both
body and notice. Existing ASCII-only ReleaseNotesTest coverage misses this case.
Found during https://github.com/link-assistant/formal-ai/pull/1581.
