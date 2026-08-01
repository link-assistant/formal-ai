// Issue #709 statement-fusion bridge. Rust/WASM owns formalization, semantic
// deduplication, contradiction handling, ranking, and provenance rendering.
function fuseBrowserSearchStatements(entries, query, language, texts, evidence, rrfK) {
  const encode = (value) => encodeURIComponent(String(value || ""));
  const sources = entries.flatMap((entry) => [entry, ...(entry.alternateUrls || [])]);
  const rows = [["Q", query, language || "en", texts.readMore, texts.via], ...sources.map((source) => [
    "S", source.url, source.title, source.excerpt, source.sourceTier, source.sourceLanguage,
    (source.providers || []).map((provider) => `${provider.id}#${provider.rank}`).join(", "),
  ])].map((row) => row.map(encode).join("\t"));
  const raw = wasmTextCall("web_search_statement_fusion", rows.join("\n"));
  if (!raw) return { lines: [texts.header(query, 0, rrfK)], statements: [] };
  try {
    const result = JSON.parse(raw);
    evidence.push(...result.evidence);
    result.lines.unshift(texts.header(query, result.statements.length, rrfK), "");
    return result;
  } catch (_error) { return { lines: [texts.header(query, 0, rrfK)], statements: [] }; }
}
