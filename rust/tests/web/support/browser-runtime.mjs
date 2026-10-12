// A minimal browser runtime for loading the site's production JavaScript under
// `node --test` (issue #895).
//
// The scripts under `js/` that the pages load as plain `<script>` tags —
// `preferences.js`, `i18n.js`, `syntax-highlight.js`, `memory.js`,
// `seed_loader.js`, `site-chrome.js`, the per-page configs — and the worker
// mirror under `js/worker/*.js` are the *production* browser sources, not
// build output. Running them through `node:vm` with a `filename` of the real
// file makes V8 attribute coverage to that path, so
// `node --test --experimental-test-coverage` measures the browser denominator
// against the same files the browser downloads.
//
// The stubs (now in js/server/worker-host.mjs, shared with the JavaScript
// server) are deliberately small: enough to let a module install its
// `window.FormalAi*` namespace and run its pure logic, and no more. Anything a
// test needs beyond that it passes in explicitly, so a stub can never quietly
// stand in for behaviour the assertions claim to check.

export {
  REPO_ROOT,
  WorkerHost,
  createBrowserContext,
  createStorage,
  createWorkerContext,
  evaluate,
  loadBrowserScript,
  loadWorkerMirror,
  plain,
  workerMirrorFiles,
} from "../../../../js/server/worker-host.mjs";
