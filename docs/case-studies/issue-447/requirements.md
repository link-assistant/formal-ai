# Issue 447 requirements

| ID | Requirement | Evidence / acceptance criterion |
|---|---|---|
| R1 | Make every control in the left panel reachable at the reported 1280×565 Windows/Firefox viewport. | The panel sections keep their independent vertical `overflow-y: auto`; the browser test runs at 1280×565 and programmatically exercises a section's scroll container. |
| R2 | Ensure the splitter does not interfere with mouse scrolling inside the left panel. | The resize target remains a separate 10 px grid column; it never overlays the panel's scrollable bodies, and the E2E test creates real overflow and verifies that a sidebar body changes `scrollTop`. |
| R3 | Reduce the chance that the splitter is mistaken for a scrollbar. | Replace the filled track and short thumb with a transparent hit target and one full-height 1 px boundary line. |
| R4 | Give clear resize feedback similar to VS Code. | Use `ew-resize`; on hover, keyboard focus, or active drag only the boundary becomes a 2 px blue sash. |
| R5 | Preserve resizing, persistence, mobile hiding, themes, and accessibility. | Existing resize behavior remains unchanged; focused E2E tests cover pointer resizing, persisted width, and ARIA/keyboard controls, while existing issue #136/#153 coverage protects mobile hiding and CSS includes explicit light/dark/auto-theme states. |
| R6 | Compile issue evidence, external research, alternatives, and a plan under `docs/case-studies/issue-447/`. | This case-study directory, its raw data, checksums, screenshots, and README satisfy the requirement. |
| R7 | Provide before/after visual evidence and automated regression coverage. | `before.png`, `after.png`, and `tests/e2e/tests/issue-447.spec.js`. |
