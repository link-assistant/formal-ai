function sourceQualifiedLiteralPattern(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/gu, '\\$&').replace(/\s+/gu, '\\s+');
}
function sourceQualifiedUnquote(text) {
  const pairs = new Map([['"', '"'], ["'", "'"], ['`', '`'], ['«', '»'], ['“', '”'], ['‘', '’']]);
  const value = text.trim();
  return pairs.get(value[0]) === value.at(-1) ? value.slice(1, -1).trim() : value;
}
function sourceQualifiedRegistryMatches(source, record) {
  return [record.id, record.name, ...(record.aliases || [])]
    .some(value => String(value).toLowerCase() === source.toLowerCase());
}
function sourceQualifiedRequest(prompt, frames, sources = []) {
  const input = String(prompt).trim().replace(/[.?!。！？]+$/gu, '').trim();
  for (const frame of [...frames].sort((left, right) => right.text.length - left.text.length)) {
    const chunks = frame.text.split(/(\{term\}|\{source\})/u);
    const slots = chunks.filter(chunk => chunk === '{term}' || chunk === '{source}');
    if (slots.length !== 2 || new Set(slots).size !== 2 || chunks.length !== 5 || !chunks[2].trim() || chunks.some(chunk => !slots.includes(chunk) && /[{}]/u.test(chunk))) continue;
    const outer = new RegExp('^' + sourceQualifiedLiteralPattern(chunks[0]) + '([\\s\\S]+)' + sourceQualifiedLiteralPattern(chunks[4]) + '$', 'iu').exec(input);
    if (!outer) continue;
    const separator = new RegExp(sourceQualifiedLiteralPattern(chunks[2]), 'giu');
    const candidates = [];
    for (const boundary of outer[1].matchAll(separator)) {
      const values = [outer[1].slice(0, boundary.index), outer[1].slice(boundary.index + boundary[0].length)];
      const request = Object.fromEntries(slots.map((slot, index) => [slot.slice(1, -1), sourceQualifiedUnquote(values[index])]));
      if (!request.term || !request.source || /[\r\n]/u.test(request.term + request.source)) continue;
      candidates.push({ ...request, language: frame.language });
    }
    const bound = candidates.filter(candidate => sources.some(record => sourceQualifiedRegistryMatches(candidate.source, record)));
    if (bound.length) return bound.sort((left, right) => right.source.length - left.source.length)[0];
    if (candidates.length) return slots[0] === '{term}' ? candidates.at(-1) : candidates[0];
  }
  return null;
}
async function observeSourceQualifiedDefinition(prompt, frames, sources, operations) {
  const request = sourceQualifiedRequest(prompt, frames, sources);
  if (!request) return null;
  const matches = sources.filter(record => sourceQualifiedRegistryMatches(request.source, record));
  const source = matches[0];
  const outcome = { request, sourceId: source ? source.id : null };
  if (!source) return { ...outcome, status: 'missing-source' };
  if (matches.length > 1) return { ...outcome, sourceId: null, status: 'ambiguous-source' };
  const allowed = operations.allowed ? operations.allowed(source) : source.defaultEnabled !== false;
  if (!allowed) return { ...outcome, status: 'disabled-source' };
  const url = operations.entryUrl(source, request.term, request.language);
  if (!url) return { ...outcome, status: 'unbound-endpoint' };
  const capture = await operations.capture(url);
  if (!capture || capture.ok === false) return { ...outcome, url, status: 'no-capture' };
  const actualUrl = capture.source_url ?? capture.url;
  const fetchedAt = String(capture.fetched_at ?? capture.fetchedAt ?? '');
  const bytes = capture.bytes ?? new TextEncoder().encode(capture.text ?? '');
  const digest = await operations.sha256(bytes);
  if (actualUrl !== url || capture.sha256 !== digest || !/^[0-9a-f]{64}$/u.test(digest) || !/^\d+$/u.test(fetchedAt) || BigInt(fetchedAt) <= 0n || BigInt(fetchedAt) > 18446744073709551615n) {
    return { ...outcome, url, status: 'invalid-capture' };
  }
  const projection = await operations.read(source, capture, request.term);
  if (!projection || !Array.isArray(projection.items) || !projection.items.length) return { ...outcome, url, status: 'no-items' };
  return { ...outcome, url, status: 'captured', projection,
    provenance: { sourceUrl: actualUrl, sha256: digest, fetchedAt, cached: Boolean(capture.cached),
      licenseName: source.licenseName, licenseUrl: source.licenseUrl } };
}

function sourceQualifiedFrames() {
  return (typeof PROMPT_PATTERNS === "undefined" ? [] : PROMPT_PATTERNS)
    .filter(row => row.intent === "source-qualified-definition" && row.kind === "frame" && row.text);
}
function sourceQualifiedProjection(record, capture, term) {
  let value;
  try { value = JSON.parse(capture.text); } catch { return null; }
  if (record.extractor === "mediawiki_summary_v1" && value.type === "disambiguation") {
    const items = Array.from(String(value.extract_html || "").matchAll(/<li\b[^>]*>([\s\S]*?)<\/li>/giu),
      match => stripHtmlToText(match[1])).filter(Boolean);
    return { kind: "disambiguation", title: value.title || term, items };
  }
  return { kind: "definition", title: value.title || term,
    items: conceptReadGlosses(record.extractor, capture.text, term).map(row => row.gloss) };
}
async function trySourceQualifiedDefinition(prompt, preferences = {}) {
  const outcome = await observeSourceQualifiedDefinition(prompt, sourceQualifiedFrames(), sourceWalkRegistry(), {
    allowed: record => sourceWalkServiceAllowed(preferences, record),
    entryUrl: sourceWalkEntryUrl, capture: sourceWalkFetchCapture,
    sha256: bytes => sourceWalkSha256Hex(new TextDecoder().decode(bytes)), read: sourceQualifiedProjection,
  });
  if (!outcome) return null;
  const { request } = outcome;
  const evidence = ["source-qualified-definition:term:" + request.term,
    "source-qualified-definition:status:" + outcome.status];
  if (outcome.status !== "captured") {
    return { intent: "concept_lookup", confidence: 0.85, evidence, ...sourceQualifiedEventProjection(outcome),
      content: wordDefinitionRender("source-qualified-definition-unresolved", request.language,
        { term: request.term, source: request.source, status: outcome.status }) };
  }
  const provenance = outcome.provenance;
  const senses = outcome.projection.items.map((gloss, index) => wordDefinitionRender("word_definition_sense",
    request.language, { n: String(index + 1), pos: "", gloss })).join("");
  evidence.push("source-qualified-definition:source:" + outcome.sourceId, "source:" + provenance.sourceUrl,
    "source:http:" + provenance.sourceUrl + " fetched_at=" + provenance.fetchedAt
      + " sha256=" + provenance.sha256 + " cached=" + String(provenance.cached),
    "source-qualified-definition:kind:" + outcome.projection.kind);
  if (provenance.cached) evidence.push("cache_hit:" + provenance.sourceUrl);
  return { intent: "concept_lookup", confidence: 0.85, evidence, ...sourceQualifiedEventProjection(outcome),
    content: wordDefinitionRender("source-qualified-definition-answer", request.language, {
      term: request.term, source: request.source, kind: outcome.projection.kind, senses,
      url: provenance.sourceUrl, sha256: provenance.sha256, "captured-at": provenance.fetchedAt,
      cached: String(provenance.cached), license: provenance.licenseName, "license-url": provenance.licenseUrl,
    }) };
}
