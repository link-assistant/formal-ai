// Migration boundary: callers must supply a genuine compiled-core transport.
// No JavaScript renderer or solver is silently substituted on this surface.
export const RUST_UI_OPERATIONS = Object.freeze({
  issueReport: "ui.render_issue_report",
  fitIssueUrl: "ui.fit_issue_url",
  normalizeDesktopStatus: "ui.normalize_desktop_status",
  exportMemoryBundle: "ui.export_memory_bundle",
  evidenceSlug: "ui.evidence_slug",
});
export function createRustUiBridge(transport) {
  if (!transport || typeof transport.call !== "function") throw new TypeError("Compiled Rust UI transport required");
  return Object.freeze({
    async call(operation, facts) {
      if (!Object.values(RUST_UI_OPERATIONS).includes(operation)) throw new Error("Unknown UI core operation");
      if (typeof transport.supports !== "function" || !transport.supports(operation)) {
        throw new Error(`Compiled core does not expose ${operation}`);
      }
      return transport.call(operation, facts);
    },
  });
}
