// Issue #550 de-risk experiment: proves @chakra-ui/react v3 + @emotion/react
// bundle cleanly with the bun bundler (`bun build probe.jsx --target browser
// --format iife --production --minify`) and that `preflight: false` keeps
// Chakra from injecting a global CSS reset. This validated the migration
// approach before converting the real front-end (src/web/app/main.jsx).
import React from "react";
import { createRoot } from "react-dom/client";
import { ChakraProvider, createSystem, defaultConfig, Button, Box } from "@chakra-ui/react";

// Disable preflight so Chakra does NOT inject a global CSS reset — existing
// styles.css stays authoritative and exact-RGB regression tests stay green.
const system = createSystem({ ...defaultConfig, preflight: false });

function Demo() {
  return (
    <ChakraProvider value={system}>
      <Box className="legacy-class" data-testid="probe-box">
        <Button className="legacy-btn" data-testid="probe-btn">Hi</Button>
      </Box>
    </ChakraProvider>
  );
}

const el = document.getElementById("root");
if (el) createRoot(el).render(<Demo />);
export { Demo, system };
