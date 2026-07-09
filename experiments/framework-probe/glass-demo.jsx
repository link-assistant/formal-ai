import React from "react";
import { createRoot } from "react-dom/client";
import LiquidGlass from "liquid-glass-react";

// Minimal utility classes the library relies on (it assumes Tailwind).
const util = `.bg-black{background:#000}.opacity-0{opacity:0}.opacity-20{opacity:.2}
.opacity-100{opacity:1}.pointer-events-none{pointer-events:none}
.mix-blend-overlay{mix-blend-mode:overlay}.transition-all{transition:all .15s}`;

function Demo() {
  const containerRef = React.useRef(null);
  return React.createElement("div", {
    ref: containerRef,
    style: { position: "relative", minHeight: "100vh", padding: "60px", display: "flex",
      flexDirection: "column", alignItems: "center", justifyContent: "flex-end", gap: 40,
      background: "linear-gradient(120deg,#6a8cff,#a06bff 45%,#39d6c0)" }
  },
    React.createElement("style", null, util),
    React.createElement("h1", { style: { color: "#fff", alignSelf: "flex-start" } }, "Liquid Glass probe"),
    // A sized positioned host so the centered glass lands where we want.
    React.createElement("div", { style: { position: "relative", width: 620, height: 76, marginBottom: 40 } },
      React.createElement(LiquidGlass, {
        displacementScale: 70, blurAmount: 0.08, saturation: 140,
        aberrationIntensity: 2, elasticity: 0.15, cornerRadius: 28,
        mouseContainer: containerRef,
        style: { position: "absolute", top: "50%", left: "50%" }
      },
        React.createElement("div", { style: { color: "#fff", fontWeight: 600, fontSize: 16,
          width: 560, height: 44, display: "flex", alignItems: "center", padding: "0 8px" } },
          "Message formal-ai")
      )
    )
  );
}
createRoot(document.getElementById("root")).render(React.createElement(Demo));
