// Issue #989: inspect the associative-memory projection before generic routes.
// This mirrors src/web/memory.js so answers describe the links the UI stores.
const MEMORY_INSPECTION_EXPORT_FIELDS = [
  "kind", "role", "intent", "tool", "inputs", "outputs", "content",
  "attachments", "sentAt", "demoLabel", "evidence", "accessCount",
  "writeCount", "conversationId", "conversationTitle",
];

function memoryInspectionStableId(prefix, text) {
  let hash = 2166136261;
  const input = String(text || "");
  for (let index = 0; index < input.length; index += 1) {
    hash ^= input.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return `${prefix}_${(`00000000${(hash >>> 0).toString(16)}`).slice(-8)}`;
}

function memoryInspectionEvidence(value) {
  return Array.isArray(value)
    ? value.filter((item) => typeof item === "string" && item.length > 0).join("|")
    : "";
}

function projectMemoryInspectionRecord(event, index) {
  const safe = event && typeof event === "object" ? event : {};
  const parts = ["id", ...MEMORY_INSPECTION_EXPORT_FIELDS].flatMap((key) => {
    const raw = safe[key];
    if (raw === undefined || raw === null || raw === "") return [];
    const value = key === "evidence" ? memoryInspectionEvidence(raw) : String(raw);
    return value ? [`${key}=${value.length}:${value}`] : [];
  });
  const canonical = parts.sort().join(";");
  const sourceId = safe.id
    ? String(safe.id)
    : memoryInspectionStableId("memory_event", `${index}:${canonical}`);
  const stableId = memoryInspectionStableId(
    "memory_event",
    `${index}:${sourceId}:${canonical}`,
  );
  const populatedFields = MEMORY_INSPECTION_EXPORT_FIELDS.filter((key) => {
    const raw = key === "evidence" ? memoryInspectionEvidence(safe[key]) : safe[key];
    return raw !== undefined && raw !== null && raw !== "";
  }).length;
  return { stableId, sourceId, links: 9 + (2 * populatedFields) };
}

function renderMemoryInspectionCounts(counts, language) {
  const entries = Object.entries(counts).sort(([left], [right]) => left.localeCompare(right));
  return entries.length ? entries.map(([name, count]) => fillMemoryInspectionResponse("memory_inventory_item", language, { name, count })).join(", ") : answerFor("memory_inventory_empty", language);
}

function fillMemoryInspectionResponse(intent, language, values) {
  return Object.entries(values).reduce(
    (text, [name, value]) => text.replace(`{${name}}`, String(value)),
    answerFor(intent, language),
  );
}

function tryMemoryInspection(prompt, normalized, history, memoryEvents, language) {
  const events = Array.isArray(memoryEvents) ? memoryEvents : [];
  const records = events.map(projectMemoryInspectionRecord);
  let intent = null;
  let values = {};
  if (lexiconMentionsRole(ROLE_MEMORY_LINK_COUNT_QUERY, normalized)) {
    intent = "memory_link_count";
    values = {
      records: records.length,
      links: records.reduce((total, record) => total + record.links, 0),
    };
  } else if (lexiconMentionsRole(ROLE_MEMORY_INVENTORY_QUERY, normalized)) {
    const kinds = {};
    const conversations = {};
    events.forEach((event) => {
      const kind = event && event.kind ? String(event.kind) : "memory_event";
      kinds[kind] = (kinds[kind] || 0) + 1;
      if (event && event.conversationId) {
        const conversation = String(event.conversationId);
        conversations[conversation] = (conversations[conversation] || 0) + 1;
      }
    });
    intent = "memory_inventory";
    values = {
      records: events.length,
      kinds: renderMemoryInspectionCounts(kinds, language),
      conversations: renderMemoryInspectionCounts(conversations, language),
    };
  } else {
    const rootQuery = lexiconMentionsRole(ROLE_MEMORY_ROOT_LINKS_QUERY, normalized);
    const priorRootQuery = (Array.isArray(history) ? history : []).some((turn) =>
      turn && String(turn.role || "").toLowerCase() === "user" &&
      lexiconMentionsRole(ROLE_MEMORY_ROOT_LINKS_QUERY, normalizePrompt(turn.content || "")),
    );
    const correction = priorRootQuery &&
      lexiconMentionsRole(ROLE_MEMORY_RETRIEVAL_CORRECTION, normalized);
    if (!rootQuery && !correction) return null;
    intent = "memory_root_links";
    values = {
      listing: records.length
        ? records.map((record) => `- ((${record.stableId}: ${record.stableId} ${record.sourceId}))`).join("\n")
        : answerFor("memory_root_links_empty", language),
    };
  }
  return {
    intent,
    content: fillMemoryInspectionResponse(intent, language, values),
    confidence: 1.0,
    evidence: ["memory:inspect", `response:${intent}`],
  };
}
