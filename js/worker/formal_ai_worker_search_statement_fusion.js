// Issue #709 bridge: Rust/WASM owns formalization, deduplication, ranking, conflicts, and rendering.
function fuseBrowserSearchStatements(entries, query, language, texts, evidence, rrfK) {
  const encode = (value) => encodeURIComponent(String(value || ""));
  const rankedSources = [...entries.map((source, index) => ({ source, rank: index + 1, alternate: false })),
    ...entries.flatMap((entry, index) => (entry.alternateUrls || []).map((source) => ({ source, rank: index + 1, alternate: true })))];
  const sources = rankedSources.slice(0, 24); if (sources.length < rankedSources.length) evidence.push(`search_fusion:source_limit:${sources.length}:${rankedSources.length}`);
  const rows = [["Q", query, language || "en", texts.readMore, texts.via, texts.otherSources].map(encode).join("\t")];
  const capacity = wasm && typeof wasm.input_capacity === "function" ? wasm.input_capacity() : 65536; let used = rows[0].length;
  for (const { source, rank, alternate } of sources) {
    const row = ["S", source.url, source.title, extractQuoteAroundQuery(source.excerpt, query, 500), source.sourceTier, source.sourceLanguage,
      (source.providers || []).map((provider) => `${provider.id}#${provider.rank}`).join(", "), rank, alternate ? "alternate" : "primary"].map(encode).join("\t");
    if (used + row.length + 1 > capacity) evidence.push(`search_fusion:input_limit:${source.url}`); else { rows.push(row); used += row.length + 1; }
  }
  const raw = wasmTextCall("web_search_statement_fusion", rows.join("\n")); if (!raw) return { lines: [texts.header(query, 0, rrfK)], statements: [] };
  try {
    const result = JSON.parse(raw); evidence.push(...result.evidence); result.lines.unshift(texts.header(query, result.statements.length, rrfK), ""); return result;
  } catch (_error) { return { lines: [texts.header(query, 0, rrfK)], statements: [] }; }
}
