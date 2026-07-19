# Browser demo setup

Use the deployed GitHub Pages demo or serve `src/web/` over HTTP so its
WebAssembly worker can load. Do not open `index.html` directly as a `file:` URL.

```bash
python3 -m http.server 8000 --directory src/web
```

The browser demo is fully in-process: JavaScript owns UI/browser integration
and the WebAssembly worker owns parity-sensitive symbolic primitives. It cannot
read `~/.formal-ai/`, spawn a native server, or run host bash. Web search and
fetch remain subject to browser CORS; the diagnostics page at `/tests/` checks
connectivity and frame policy.

Memory lives in the browser profile's IndexedDB. Click **Export memory** to
download `formal-ai-memory.lino`, then **Import memory** in Desktop, VS Code, or
another browser to share memory. Import accepts a full `formal_ai_bundle` and
legacy `demo_memory` projection.

Verify the demo with `Hi`, a calculator prompt, and an export/import round trip.
The status must identify the in-process browser environment.
