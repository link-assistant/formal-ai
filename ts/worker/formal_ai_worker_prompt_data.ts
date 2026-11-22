// Prompt-local classification, record queries, extraction and language identification.
// Mirrors rust/src/solver_handlers/prompt_data.rs. The request binds its operands
// before generic routing can treat their contents as a command or a source query.

function promptDataRules() {
  return textTransformSeedRecords("prompt-data.lino");
}

function promptDataSurfaces(rules, identifier) {
  const rule = rules.find((item) => item.name === "cue" && item.value === identifier);
  return rule ? textTransformRecordValues(rule, "surface") : [];
}

function promptDataHas(text, surface) {
  return crateModule("crate/seed_meanings.mjs").surfacePresent(text, surface);
}

function promptDataStarts(head, rules, identifier) {
  return promptDataSurfaces(rules, identifier).some((surface) => head === surface || head.startsWith(surface + " "));
}

function promptDataField(rules, locale, key) {
  const get = (name) => rules.find((item) => item.name === "locale" && item.value === name)
    ?.children.find((item) => item.name === key)?.value;
  return get(locale) || get("en") || "";
}

function promptDataReply(operation, answer, spans, rules, locale, links = []) {
  const evidence = ["prompt_data:network:none", ...links];
  const template = promptDataField(rules, locale, "evidence");
  const quotes = [...new Set(spans)].map((span) => {
    evidence.push("prompt_data:span:" + span);
    return template.replace("{span}", () => span);
  });
  return {
    intent: "prompt_data_" + operation,
    content: [answer, ...quotes].join("\n"),
    confidence: spans.length ? 0.9 : 0.5,
    evidence,
  };
}

function promptDataFacet(rules, label) {
  return rules.find((item) => item.name === "facet" && textTransformRecordValues(item, "label").includes(label));
}

function promptDataChoices(text, rules, splitConjunctions = false) {
  const joins = promptDataSurfaces(rules, "option_join");
  const parts = text.split(/[,;|]/u).flatMap((part) => {
    const words = part.trim().split(/\s+/u);
    if (joins.includes(words[0].toLowerCase())) words.shift();
    if (!splitConjunctions) return [words.join(" ")];
    return words.map((word) => joins.includes(word.toLowerCase()) ? "|" : word).join(" ").split("|");
  });
  const labels = parts.map((part) => part.trim().replace(/^["'«“]|["'»”]$/gu, "")).filter(Boolean);
  return [...new Map(labels.map((label) => [normalizePrompt(label), label])).values()];
}

function promptDataClassify(head, payload, rules, locale) {
  const start = head.search(/[[(]/u);
  const end = start < 0 ? -1 : head.indexOf(head[start] === "[" ? "]" : ")", start + 1);
  let source = head.slice(start + 1, end);
  if (end < 0) {
    const starts = promptDataSurfaces(rules, "label-start");
    const words = head.split(/\s+/u);
    const opening = words.findIndex((word) => starts.includes(word.toLowerCase()));
    source = opening < 0 ? "" : words.slice(opening + 1).join(" ");
    const closing = promptDataSurfaces(rules, "label-end").find((cue) => source.toLowerCase().endsWith(" " + cue));
    if (closing) source = source.slice(0, -closing.length);
  }
  const labels = promptDataChoices(source, rules, end < 0 || head[start] === "(");
  const unknown = promptDataField(rules, locale, "unknown");
  if (labels.length < 2 || labels.length > 64) {
    return promptDataReply("classification", unknown, [], rules, locale);
  }
  const spans = payload.split(/(?<=[.!?])\s+|,/u).map((span) => span.trim()).filter(Boolean);
  const clauses = spans.map((span) => [span, normalizePrompt(span)]);
  const sentiment = promptDataSurfaces(rules, "sentiment").some((cue) => promptDataHas(normalizePrompt(head), cue));
  const candidates = [];
  let fallback;
  let neutral;
  for (const label of labels) {
    const key = normalizePrompt(label);
    const facet = promptDataFacet(rules, key);
    if (facet?.value === "fallback") {
      fallback ??= label;
      continue;
    }
    if (sentiment && facet?.value === "neutral") neutral = label;
    const terms = facet ? textTransformRecordValues(facet, "term") : [];
    const explicit = clauses.find(([, part]) => promptDataHas(part, key));
    const hits = terms.filter((term) => clauses.some(([, part]) => promptDataHas(part, term)));
    if (!explicit && !hits.length) continue;
    const support = explicit?.[0] ?? clauses.find(([, part]) => hits.some((term) => promptDataHas(part, term)))?.[0];
    if (support) candidates.push({ label, support, score: hits.length + (explicit ? 2 : 0) });
  }
  candidates.sort((left, right) => right.score - left.score);
  if (candidates[0] && candidates[0].score > (candidates[1]?.score ?? 0)) {
    const { label, support } = candidates[0];
    return promptDataReply("classification", label, [support], rules, locale);
  }
  if (neutral && candidates.length === 0) return promptDataReply("classification", neutral, [payload], rules, locale);
  return promptDataReply("classification", fallback ?? unknown, [], rules, locale);
}

function promptDataTokenLinks(lines) {
  return lines.flatMap((line) => {
    const at = line.search(/[:：]/u);
    const field = at < 0 ? "" : line.slice(0, at).trim();
    const value = line.slice(at + 1).trim();
    return field && value ? [{ field, value, span: line.trim() }] : [];
  });
}

function promptDataStripListPrefix(line) {
  return line.trim().replace(/^(?:\d+[.)]|[-*])\s+/u, "");
}

function promptDataRow(subject, fields, span, header) {
  return fields.map(({ field, value }) => ({ subject, field, value, span, header }));
}

function promptDataTableCells(line) {
  return line.trim().replace(/^\||\|$/gu, "").split("|").map((cell) => cell.trim());
}

function promptDataRecords(text, rules) {
  const lines = text.split("\n");
  const links = [];
  let fields = [];
  let rowLines = [];
  const flush = () => {
    const identities = promptDataSurfaces(rules, "identity_field");
    const identity = fields.find(({ field }) => identities.includes(normalizePrompt(field)))?.value;
    links.push(...promptDataRow(identity || "", fields, rowLines.join("\n")));
    fields = [];
    rowLines = [];
  };

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index].trim();
    const next = lines[index + 1]?.trim();
    if (line.includes("|") && next?.includes("|") &&
        promptDataTableCells(next).every((cell) => /^:?-{3,}:?$/u.test(cell))) {
      flush();
      const headers = promptDataTableCells(line);
      index += 1;
      while (lines[index + 1]?.includes("|")) {
        const row = lines[++index].trim();
        const cells = promptDataTableCells(row);
        if (cells.length !== headers.length || !cells[0]) continue;
        const properties = headers.slice(1).flatMap((field, column) => cells[column + 1]
          ? [{ field, value: cells[column + 1] }] : []);
        links.push(...promptDataRow(cells[0], properties, row, line));
      }
      continue;
    }
    if (!line) {
      flush();
      continue;
    }
    const content = promptDataStripListPrefix(line);
    const parts = content.split(/ [-–—] |; /u);
    if (parts.length > 1 && !parts[0].includes(":")) {
      flush();
      links.push(...promptDataRow(parts[0], promptDataTokenLinks(parts.slice(1)), line));
      continue;
    }
    const values = promptDataTokenLinks(parts);
    if (values.some(({ field }) => promptDataSurfaces(rules, "identity_field").includes(normalizePrompt(field)))) flush();
    fields.push(...values);
    rowLines.push(lines[index]);
  }
  flush();
  return links;
}

function promptDataRecordAnswer(payload, rules, locale) {
  const lines = payload.split("\n");
  let index = lines.findLastIndex((line) => {
    const at = line.search(/[:：]/u);
    return at >= 0 && promptDataSurfaces(rules, "question").includes(normalizePrompt(line.slice(0, at)));
  });
  let question = index >= 0 ? lines.slice(index).join(" ").replace(/^[^:：]*[:：]/u, "") : "";
  if (index < 0 && lines.at(-1)?.trimEnd().endsWith("?")) {
    index = lines.length - 1;
    question = lines[index];
  }
  const query = normalizePrompt(question);
  const links = promptDataRecords(lines.slice(0, index < 0 ? lines.length : index).join("\n"), rules);
  const gap = () => promptDataReply("record_qa", promptDataField(rules, locale, "gap"), [], rules, locale);
  const fields = [...new Set(links.map(({ field }) => normalizePrompt(field)).filter((key) => promptDataHas(query, key)))];
  if (fields.length !== 1) return gap();
  const matches = links.filter(({ field }) => normalizePrompt(field) === fields[0]);
  const named = matches.filter(({ subject }) => promptDataHas(query, normalizePrompt(subject)));
  let selected = matches.every(({ subject }) => !subject) ? matches : named;
  let output = "value";
  if (!named.length && matches.some(({ subject }) => subject) && promptDataStarts(query, rules, "subject_question")) {
    output = "subject";
    const explicit = matches.filter(({ subject, value }) => subject && promptDataHas(query, normalizePrompt(value)));
    const negated = promptDataSurfaces(rules, "negation").some((cue) => promptDataHas(query, cue));
    const polarity = promptDataSurfaces(rules, negated ? "negative" : "affirmative");
    selected = explicit.length ? explicit : matches.filter(({ subject, value }) => subject && polarity.includes(normalizePrompt(value)));
  }
  const values = [...new Set(selected.map((link) => link[output]))];
  if (values.length !== 1) return gap();
  const evidence = selected.map(({ subject, field, value }) => `prompt_data:link:${subject}→${field}→${value}`);
  return promptDataReply("record_qa", values[0], selected.flatMap(({ span, header }) => header ? [span, header] : [span]), rules, locale, evidence);
}

function promptDataExtract(head, payload, rules, locale) {
  const cue = promptDataSurfaces(rules, "extraction").find((surface) => promptDataHas(head, surface));
  const requested = promptDataChoices(head.slice(cue.length).trim().replace(/^the\s+/u, ""), rules, true);
  const fields = payload.split("\n").flatMap((line) =>
    promptDataTokenLinks([promptDataStripListPrefix(line)]).map((link) => ({ ...link, span: line.trim() })));
  if (!fields.length && !payload.includes("\n")) return null;
  const answers = [];
  const spans = [];
  for (const key of requested) {
    const matches = fields.filter(({ field }) => normalizePrompt(field) === normalizePrompt(key));
    if (new Set(matches.map(({ value }) => value)).size === 1) {
      const { field, value } = matches[0];
      answers.push(`${field}: ${value}`);
      spans.push(...matches.map(({ span }) => span));
    } else {
      answers.push(`${key}: ${promptDataField(rules, locale, "missing")}`);
    }
  }
  return promptDataReply("extraction", answers.join("\n") || promptDataField(rules, locale, "gap"), spans, rules, locale);
}

function promptDataLanguage(payload, rules, locale) {
  const language = crateModule("crate/language.mjs");
  const slug = language.detect(payload);
  const words = new Set(normalizePrompt(payload).split(" ").filter(Boolean));
  const dictionary = new Set();
  if (language.surfaceMatchesLanguage(payload, slug)) {
    for (const meaning of crateModule("crate/seed_meanings.mjs").lexicon()) {
      for (const lexeme of meaning.lexemes) {
        if (lexeme.language !== slug) continue;
        for (const word of lexeme.words) {
          if (words.has(word.text)) dictionary.add(word.text);
        }
      }
    }
  }
  if (!dictionary.size) {
    return promptDataReply("language", promptDataField(rules, locale, "unknown"), [], rules, locale);
  }
  // Keep dictionary evidence in the order it occurs in the caller's text.
  const matches = [...words].filter((word) => dictionary.has(word)).join(", ");
  const name = promptDataField(rules, locale, slug) || language.languageName(slug);
  const body = `${name}\n${promptDataField(rules, locale, "dictionary")}: ${matches}`;
  return promptDataReply("language", body, [payload], rules, locale, [`prompt_data:dictionary:${slug}:${matches}`]);
}

function tryPromptData(prompt, forcedLocale) {
  const colon = prompt.search(/[:：]/u);
  if (colon < 0) return null;
  const head = prompt.slice(0, colon).trim();
  const key = normalizePrompt(head);
  const payload = prompt.slice(colon + 1).trim();
  if (!payload) return null;
  const rules = promptDataRules();
  const locale = forcedLocale || crateModule("crate/language.mjs").detect(head);
  if (promptDataStarts(key, rules, "classification")) return promptDataClassify(head, payload, rules, locale);
  if (promptDataStarts(key, rules, "language")) return promptDataLanguage(payload, rules, locale);
  if (promptDataSurfaces(rules, "records").includes(key)) return promptDataRecordAnswer(payload, rules, locale);
  if (promptDataSurfaces(rules, "identity_field").includes(key) &&
      payload.split("\n").some((line) => promptDataSurfaces(rules, "question").includes(normalizePrompt(line.split(/[:：]/u)[0])))) {
    return promptDataRecordAnswer(prompt, rules, locale);
  }
  if (promptDataStarts(key, rules, "extraction")) return promptDataExtract(key, payload, rules, locale);
  if (promptDataStarts(key, rules, "rewrite") &&
      promptDataSurfaces(rules, "omit").some((surface) => promptDataHas(key, surface))) {
    return promptDataReply("rewrite", promptDataField(rules, locale, "gap"), [], rules, locale);
  }
  return null;
}
