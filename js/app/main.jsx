// Issue #550: the front-end is now authored as JSX and bundled by the bun
// bundler into the served `js/app.js` (see package.json `build:web`).
// React and ReactDOM are imported here so they are bundled into the app — a
// single React instance shared with @chakra-ui/react and @emotion/react (the
// vendor bundle no longer needs to expose them as globals for the app). The
// JSX factory stays bound to `h` so the existing `h(tag, props, ...children)`
// render calls — and the static guards that parse them (check-web-tdz,
// check-web-hardcoded-ui-strings) — keep working unchanged during the
// incremental migration to Chakra primitives.
import React from "react";
import { createRoot } from "react-dom/client";
import { ChakraProvider } from "@chakra-ui/react";
// Issue #550: the Chakra system bridges the app's --fa-* CSS design tokens into
// Chakra semantic tokens with the global reset/body styling disabled, so
// styles.css stays authoritative while the UI migrates to Chakra primitives.
import { system as chakraSystem } from "./theme.js";
import { App } from "./app.jsx";

const { createElement: h } = React;

createRoot(document.getElementById("root")).render(
  <ChakraProvider value={chakraSystem}>
    <App />
  </ChakraProvider>,
);
