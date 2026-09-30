# @link-assistant/formal-ai-engine

A browser Worker client for the packaged WASM engine, with typed solve and
memory APIs. The package includes the worker module chain, WASM binary, seed
loader and every seed file. Its version comes from `rust/Cargo.toml` at build
preparation time. It does not implement another solver in the bindings.

```html
<script type="module">
import { createEngine } from './node_modules/@link-assistant/formal-ai-engine/src/index.js';
const engine = await createEngine();
const answer = await engine.solve('Hi');
document.body.textContent = answer.content;
// Persist/export your browser's memory explicitly:
const bundle = await engine.memory.exportBundle();
engine.dispose();
</script>
```

Serve assets on the same origin as the page, or pass `assetBase` pointing to
the packaged `assets/` folder on that origin. Worker restrictions apply;
Node.js and SSR are not supported. Memory uses the browser's existing shared
Formal AI IndexedDB store. The API does not imply that solve automatically
appends conversation records; callers use `memory.append` explicitly.
Import merges bundle events; it does not replace the shipped seed or grants.

Before publishing, run the repository web build, which prepares `assets/`.
Publication and npm fixture parity have not been verified in this draft.
