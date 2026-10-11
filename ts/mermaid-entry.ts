// The Mermaid renderer the debugger's recipe pane loads on demand (issue #667,
// R383): `bun run build:web` bundles this entry into js/mermaid.bundle.js,
// which js/debugger-client.js injects only when a diagram is shown, so the
// chat never pays for it. Rendering runs at Mermaid's `strict` security level
// (labels are text, never HTML or script) and with no start-on-load scan.
import mermaid from "mermaid";

// Labels render as SVG text rather than HTML in a foreignObject, so the
// sanitizer's SVG profile keeps them.
mermaid.initialize({
  startOnLoad: false, securityLevel: "strict", theme: "neutral", htmlLabels: false, flowchart: { htmlLabels: false },
});

let renders = 0;

async function render(source) {
  renders += 1;
  const { svg } = await mermaid.render(`formal-ai-mermaid-${renders}`, source);
  return svg;
}

window.FormalAiMermaid = { render };
