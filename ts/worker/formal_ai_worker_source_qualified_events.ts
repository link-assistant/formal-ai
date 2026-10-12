// Observed native events of formalization/source_qualified_definition.rs
// and SourceCapture::record in source_fetch.rs, before finalize_simple.
function sourceQualifiedEventProjection(outcome) {
  const events = [{ kind: "source-qualified-definition:term", payload: outcome.request.term }];
  if (outcome.status === "captured") {
    const provenance = outcome.provenance;
    events.push({ kind: "source:http", payload: provenance.sourceUrl + " fetched_at=" + provenance.fetchedAt
      + " sha256=" + provenance.sha256 + " cached=" + String(provenance.cached) });
    if (provenance.cached) events.push({ kind: "cache_hit", payload: provenance.sourceUrl });
    events.push({ kind: "source-qualified-definition:source", payload: outcome.sourceId },
      { kind: "source", payload: provenance.sourceUrl },
      { kind: "source-qualified-definition:kind", payload: outcome.projection.kind });
  }
  events.push({ kind: "source-qualified-definition:status", payload: outcome.status });
  return { solverEvents: events, responseLink: "response:source-qualified-definition" };
}
