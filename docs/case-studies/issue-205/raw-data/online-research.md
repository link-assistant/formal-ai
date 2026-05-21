# Issue #205 Online Research

Date: 2026-05-20

## Sources

- Tesseract.js repository: https://github.com/naptha/tesseract.js
- Tesseract.js API docs: https://github.com/naptha/tesseract.js/blob/master/docs/api.md
- Tesseract.js local installation docs: https://github.com/naptha/tesseract.js/blob/master/docs/local-installation.md
- Tesseract.js v7.0.0 release: https://github.com/naptha/tesseract.js/releases/tag/v7.0.0
- NPM package page: https://www.npmjs.com/package/tesseract.js
- Project Naptha tessdata index: https://tessdata.projectnaptha.com/

## Findings

Tesseract.js is a browser and Node.js OCR wrapper around a WebAssembly port of
Tesseract. Its public API centers on `createWorker`, then
`worker.recognize(image)`. For repeated OCR jobs, the docs recommend creating a
worker once and reusing it.

The browser docs state that the Tesseract.js API layer opens a web worker. That
worker loads `tesseract.js-core` and language files dynamically. This supports
the requirement for a separate optional app bundle: formal-ai can ship the small
wrapper separately and allow Tesseract.js to download the worker, core, and
traineddata only after explicit opt-in.

The local installation docs warn that `corePath` should be a directory containing
all core variants rather than a single `.js` file. The implementation therefore
does not pin a single core file; it uses Tesseract.js defaults for worker/core
selection and sets only `langPath` to the fast English tessdata path.

The tessdata index exposes `4.0.0_fast/eng.traineddata.gz`, matching the current
implementation's English-only experimental path. Future multilingual OCR can
expand the settings UI and traineddata warning once language selection exists.

## Package Metadata

`npm view tesseract.js@7.0.0 ...` was saved to
`npm-tesseract.json`.

- Version: `7.0.0`
- License: `Apache-2.0`
- Unpacked size: `1,411,341` bytes
- Dependency includes `tesseract.js-core: ^7.0.0`
- Modified: `2025-12-15T03:37:59.984Z`

`npm view tesseract.js-core@7.0.0 ...` was saved to
`npm-tesseract-core.json`.

- Version: `7.0.0`
- License: `Apache-2.0`
- Unpacked size: `45,262,431` bytes
- Modified: `2025-12-15T02:47:17.122Z`

## Size Probes

The warning in the UI is based on direct `curl` size probes saved beside this
file:

| Asset | Bytes | File |
| --- | ---: | --- |
| Tesseract browser wrapper from CDN | `62,961` | `tesseract-js-cdn-size.txt` |
| Tesseract worker script | `111,307` | `tesseract-worker-size.txt` |
| SIMD LSTM core script path | `3,899,472` | `tesseract-core-simd-lstm-size.txt` |
| English fast traineddata | `1,984,273` | `tessdata-eng-fast-size.txt` |

Total measured first-use payload: `6,058,013` bytes, or about `6.06 MB`
decimal (`5.78 MiB`). The UI rounds this to "about 6 MB".
