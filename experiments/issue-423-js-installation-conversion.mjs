// Issue #423: verify the browser worker mirror recognizes README/install-guide
// <-> shell/PowerShell conversion requests before generic write_program routing.
//
// Run: `node experiments/issue-423-js-installation-conversion.mjs`

import fs from "node:fs";
import vm from "node:vm";
import { TextDecoder, TextEncoder } from "node:util";

const src = fs.readFileSync(new URL("../src/web/formal_ai_worker.js", import.meta.url), "utf8");

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.WebAssembly = WebAssembly;
sandbox.importScripts = () => {
  throw new Error("no importScripts in node");
};
sandbox.postMessage = () => {};
sandbox.setTimeout = setTimeout;
sandbox.fetch = async () => {
  throw new Error("no fetch");
};
sandbox.location = { search: "", origin: "http://localhost" };
sandbox.TextEncoder = TextEncoder;
sandbox.TextDecoder = TextDecoder;
sandbox.crypto = globalThis.crypto;
sandbox.URL = URL;
vm.createContext(sandbox);
vm.runInContext(src, sandbox, { filename: "formal_ai_worker.js" });

const failures = [];
function check(name, condition, detail = "") {
  console.log(`${condition ? "PASS" : "FAIL"}: ${name}${detail ? ` :: ${detail}` : ""}`);
  if (!condition) failures.push(name);
}

const readmePrompt = `Convert this README.md installation guide for react/react into both sh and PowerShell scripts:

\`\`\`markdown
## Installation

1. Clone the repository.

   \`\`\`sh
   git clone https://github.com/react/react.git
   cd react
   \`\`\`

2. Install and verify.

   \`\`\`sh
   yarn install
   yarn test
   \`\`\`
\`\`\``;

const scriptPrompt = `Convert this shell installation script back to a README.md installation guide:

\`\`\`bash
#!/usr/bin/env bash
set -euo pipefail
git clone https://github.com/ohmyzsh/ohmyzsh.git
cd ohmyzsh
sh tools/install.sh
\`\`\``;

const unwrappedReadmePrompt = `Convert this README.md installation guide for example/widget into a sh script:

## Installation

Clone the repository:

\`\`\`sh
git clone https://github.com/example/widget.git
cd widget
\`\`\`

Install and verify:

\`\`\`sh
npm install
npm test
\`\`\``;

const readmeHit = sandbox.tryInstallationConversion(
  readmePrompt,
  sandbox.normalizePrompt(readmePrompt),
);
check("direct README prompt routes to installation_conversion", readmeHit?.intent === "installation_conversion", readmeHit?.intent);
check("README prompt renders bash", readmeHit?.content.includes("Bash script:"), readmeHit?.content.slice(0, 120));
check("README prompt renders PowerShell", readmeHit?.content.includes("PowerShell script:"), readmeHit?.content.slice(0, 120));
check("README prompt preserves clone command", readmeHit?.content.includes("git clone https://github.com/react/react.git"));
check("README prompt preserves test command", readmeHit?.content.includes("yarn test"));
check(
  "README prompt exposes meta algorithm",
  readmeHit?.content.includes("Meta algorithm for constructing conversion algorithms"),
);
check(
  "README prompt connects coding surfaces",
  readmeHit?.content.includes("program_blueprint") &&
    readmeHit?.content.includes("rule_synthesis") &&
    readmeHit?.content.includes("numeric_list"),
);
check(
  "README evidence records construction stages",
  readmeHit?.evidence?.some((line) => line.includes("algorithm_construction:stage:extract_ir")),
  JSON.stringify(readmeHit?.evidence || []),
);

const scriptHit = sandbox.tryInstallationConversion(
  scriptPrompt,
  sandbox.normalizePrompt(scriptPrompt),
);
check("direct script prompt routes to installation_conversion", scriptHit?.intent === "installation_conversion", scriptHit?.intent);
check("script prompt renders README guide", scriptHit?.content.includes("README.md installation guide:"), scriptHit?.content.slice(0, 120));
check("script prompt marks shell source", scriptHit?.content.includes("source_format shell_script"));
check("script prompt preserves install command", scriptHit?.content.includes("sh tools/install.sh"));

const unwrappedHit = sandbox.tryInstallationConversion(
  unwrappedReadmePrompt,
  sandbox.normalizePrompt(unwrappedReadmePrompt),
);
check("unwrapped README prompt routes to installation_conversion", unwrappedHit?.intent === "installation_conversion", unwrappedHit?.intent);
check("unwrapped README keeps markdown source", unwrappedHit?.content.includes("source_format markdown"));
check("unwrapped README preserves first fenced command", unwrappedHit?.content.includes("git clone https://github.com/example/widget.git"));
check("unwrapped README preserves second fenced command", unwrappedHit?.content.includes("npm test"));

const fullRoute = await sandbox.solve(readmePrompt, [], {});
check("solve routes README prompt before write_program", fullRoute?.intent === "installation_conversion", fullRoute?.intent);
check(
  "solve diagnostics include installation handler",
  fullRoute?.steps?.some((step) => step?.step === "dispatch_handler" && step?.detail === "tryInstallationConversion"),
  JSON.stringify(fullRoute?.steps || []),
);

if (failures.length) {
  console.log(`\n${failures.length} CHECK(S) FAILED`);
  process.exit(1);
}

console.log("\nIssue #423 worker installation-conversion checks passed.");
