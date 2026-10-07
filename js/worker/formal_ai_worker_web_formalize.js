// The internet as formal knowledge (issue #1163), browser twin.
//
// JavaScript-first twin of rust/src/web_formalize.rs and
// rust/src/web_formalize_trust.rs: one generic page formalizer that turns a
// fetched resource (HTML, Markdown, plain text, JSON, PDF-extracted text)
// into a page network of heading, paragraph, list item, table row and
// language-tagged code block links; a store keyed by URL and SHA-256; the
// two page queries; the trust score; and the rediscovery record. Every rule
// comes from data/seed/page-formalization-rules.lino and
// data/seed/source-trust-weights.lino, so this file only interprets them.
//
// Offsets are taken on an ASCII-only lowercase copy of the text, which keeps
// every index valid in the original (the native walker lowercases bytes the
// same way). Nothing here fetches: the caller passes the captured text.

const PAGE_RULES_FILE = "page-formalization-rules.lino";
const PAGE_TRUST_FILE = "source-trust-weights.lino";
/** Tags whose content is never page prose: the walker jumps past them. */
const PAGE_OPAQUE_TAGS = ["script", "style", "template", "noscript"];
const PAGE_COMMENT_OPEN = "<!--";
const PAGE_COMMENT_CLOSE = "-->";
const PAGE_HEADING_TAGS = ["h1", "h2", "h3", "h4", "h5", "h6"];
// Openers of block children: a list item holding one is a container the walker
// steps into (kotlinlang's numbered steps hold paragraphs and code blocks).
const PAGE_NESTED_BLOCK_OPENERS = ["<p>", "<p ", "<pre", "<div", "<ul", "<ol", "<table"];
const PAGE_CELL_TAGS = ["td", "th"];
const PAGE_FENCES = ["```", "~~~"];
const PAGE_BULLETS = ["-", "*", "+"];
const PAGE_SNIFF_HEAD = 512;
const PAGE_HEADING_MAX_CHARS = 80;
const PAGE_CLASS_TOKEN_PREFIXES = ["highlight-", "source-"];
const PAGE_ENTITIES = [
  ["&nbsp;", " "],
  ["&amp;", "&"],
  ["&lt;", "<"],
  ["&gt;", ">"],
  ["&quot;", "\""],
  ["&#39;", "'"],
];

let PAGE_RULES_CACHE = null;

/**
 * The records of a seed file: the children of its one top-level wrapper, or
 * the top level itself when the file carries no wrapper.
 * @param {string} text
 * @returns {Array<object>}
 */
function pageSeedRecords(text) {
  const top = parseLinoTree(text || "").children;
  if (top.length === 1 && top[0].children.length > 0) return top[0].children;
  return top;
}

/**
 * The formalization rules, parsed once per seed text.
 * @returns {object}
 */
function pageFormalizationRules() {
  const text = seedRawText(SEED_RAW, PAGE_RULES_FILE);
  if (PAGE_RULES_CACHE && PAGE_RULES_CACHE.text === text) return PAGE_RULES_CACHE.rules;
  const rules = {
    mimeHints: [],
    sniffs: [],
    tagSources: [],
    tagFallback: "unknown",
    classPrefixes: [],
    codeAttributes: [],
    shebangs: [],
    extensions: [],
    commandVerbs: [],
    pageQueries: [],
    documentSources: [],
    suppliedPageSeparator: "",
  };
  for (const record of pageSeedRecords(text)) {
    if (record.name === "mime") {
      const hint = childValue(record, "hint");
      const kind = childValue(record, "kind");
      if (hint && kind) rules.mimeHints.push({ hint: hint, kind: kind });
    } else if (record.name === "sniff") {
      const kind = childValue(record, "kind");
      if (!kind) continue;
      rules.sniffs.push({
        lowerPrefix: childValue(record, "lower_prefix"),
        firstChar: childValue(record, "first_char"),
        lineStart: childValue(record, "line_start"),
        kind: kind,
      });
    } else if (record.name === "language_tag") {
      for (const child of record.children) {
        if (child.name === "tag_source" && child.value) rules.tagSources.push(child.value);
        if (child.name === "html_class_prefix" && child.value) rules.classPrefixes.push(child.value);
        if (child.name === "html_code_attribute" && child.value) rules.codeAttributes.push(child.value);
      }
      const fallback = childValue(record, "fallback");
      if (fallback) rules.tagFallback = fallback;
    } else if (record.name === "shebang" || record.name === "extension") {
      const key = childValue(record, record.name === "shebang" ? "token" : "suffix");
      const language = childValue(record, "language");
      if (key && language) {
        (record.name === "shebang" ? rules.shebangs : rules.extensions).push([key, language]);
      }
    } else if (record.name === "command_verb" && record.value) {
      rules.commandVerbs.push(record.value);
    } else if (record.name === "page_query" && record.value && childValue(record, "template")) {
      rules.pageQueries.push([record.value, childValue(record, "template")]);
    } else if (record.name === "document_source" && record.value && childValue(record, "hint")) {
      const aliases = record.children.filter((child) => child.name === "alias" && child.value).map((child) => child.value);
      rules.documentSources.push({ name: record.value, hint: childValue(record, "hint"), aliases: aliases });
    } else if (record.name === "supplied_page_separator" && record.value) {
      rules.suppliedPageSeparator = record.value;
    }
  }
  PAGE_RULES_CACHE = { text: text, rules: rules };
  return rules;
}

/**
 * ASCII-only lowercase, so offsets stay valid in the original text.
 * @param {string} text
 * @returns {string}
 */
function pageAsciiLower(text) {
  return text.replace(/[A-Z]/g, (letter) => letter.toLowerCase());
}

/**
 * Lines the way the native `str::lines` yields them.
 * @param {string} text
 * @returns {Array<string>}
 */
function pageLines(text) {
  if (text === "") return [];
  const lines = text.split("\n");
  if (lines[lines.length - 1] === "") lines.pop();
  return lines.map((line) => (line.endsWith("\r") ? line.slice(0, -1) : line));
}

/**
 * The seed's kind spelling as a mime kind (`unknown` for anything else).
 * @param {string} kind
 * @returns {string}
 */
function pageKindOf(kind) {
  return ["html", "markdown", "json", "pdf_text", "plain_text"].includes(kind) ? kind : "unknown";
}

/**
 * The seed's sniff rules over the payload's first characters.
 * @param {object} rules
 * @param {string} text
 * @returns {string}
 */
function pageSniff(rules, text) {
  const head = text.slice(0, PAGE_SNIFF_HEAD);
  const lowerHead = pageAsciiLower(head);
  for (const rule of rules.sniffs) {
    const matched =
      (rule.lowerPrefix && lowerHead.startsWith(rule.lowerPrefix)) ||
      (rule.firstChar && head.startsWith(rule.firstChar)) ||
      (rule.lineStart && pageLines(head).some((line) => line.startsWith(rule.lineStart)));
    if (matched) return pageKindOf(rule.kind);
  }
  return "unknown";
}

/**
 * The mime kind from a `Content-Type` style hint, or sniffed from the text.
 * @param {string|null} hint
 * @param {string} text
 * @returns {string}
 */
function pageMimeKind(hint, text) {
  const rules = pageFormalizationRules();
  if (hint) {
    const normalized = pageAsciiLower(String(hint).split(";")[0].trim());
    const found = rules.mimeHints.find((mime) => mime.hint === normalized);
    if (found) return pageKindOf(found.kind);
  }
  return pageSniff(rules, text || "");
}

/**
 * Host of a URL, lowercased, without a port or a leading `www.`.
 * @param {string} url
 * @returns {string}
 */
function pageUrlDomain(url) {
  const marker = url.indexOf("://");
  const rest = marker === -1 ? url : url.slice(marker + 3);
  const host = pageAsciiLower(rest.split("/")[0].split(":")[0].trim());
  return host.startsWith("www.") ? host.slice(4) : host;
}

/**
 * Formalize fetched text into a page network (issue #1163 R1): a page root
 * with one link per block, in document order, each carrying its kind and
 * text (and level, language or command flag where they apply).
 * @param {string} text
 * @param {string|null} mimeHint
 * @param {string|null} urlHint
 * @returns {{mime: string, domain: string, blocks: Array<object>}}
 */
function formalizePage(text, mimeHint, urlHint) {
  const rules = pageFormalizationRules();
  const source = String(text || "");
  const mime = pageMimeKind(mimeHint || null, source);
  let blocks = [];
  if (mime === "html") blocks = pageHtmlBlocks(source, rules);
  else if (mime === "markdown") blocks = pageMarkdownBlocks(source, rules, urlHint || null);
  else if (mime === "json") {
    try {
      pageJsonBlocks(JSON.parse(source.trim()), blocks);
    } catch {
      blocks = [];
    }
  } else blocks = pagePlainBlocks(source, rules);
  return {
    mime: mime,
    domain: urlHint ? pageUrlDomain(urlHint) : "",
    blocks: blocks.map((block, index) => Object.assign({ id: index + 1 }, block)),
  };
}

/**
 * Offset of the close tag that really closes `name` (`</p` never matches
 * `</pre>`), or -1.
 * @param {string} lower
 * @param {number} from
 * @param {string} name
 * @returns {number}
 */
function pageFindCloseTag(lower, from, name) {
  const close = "</" + name;
  let search = from;
  for (;;) {
    const at = lower.indexOf(close, search);
    if (at === -1) return -1;
    const next = lower.charAt(at + close.length);
    if (next === "" || next === ">" || /\s/.test(next)) return at;
    search = at + close.length;
  }
}

/**
 * Walk HTML without an HTML dependency: container tags are stepped into,
 * comments and opaque tags are skipped whole, and each block tag takes the
 * text up to its matching close tag.
 * @param {string} text
 * @param {object} rules
 * @returns {Array<object>}
 */
function pageHtmlBlocks(text, rules) {
  const lower = pageAsciiLower(text);
  const out = [];
  let cursor = 0;
  // The nearest enclosing element whose class names a language (a
  // `language-scala` wrapper around a bare `<pre>`), and where it closes.
  let container = null;
  for (;;) {
    const open = lower.indexOf("<", cursor);
    if (open === -1) break;
    if (lower.startsWith(PAGE_COMMENT_OPEN, open)) {
      const body = open + PAGE_COMMENT_OPEN.length;
      const end = lower.indexOf(PAGE_COMMENT_CLOSE, body);
      cursor = end === -1 ? lower.length : end + PAGE_COMMENT_CLOSE.length;
      continue;
    }
    const tagEnd = lower.indexOf(">", open);
    if (tagEnd === -1) break;
    const innerTag = text.slice(open + 1, tagEnd);
    const closing = innerTag.startsWith("/");
    const name = pageAsciiLower(innerTag.replace(/^\/+/, "").trim().split(/\s+/)[0] || "");
    if (closing || !name) {
      cursor = tagEnd + 1;
      continue;
    }
    const innerEnd = pageFindCloseTag(lower, tagEnd + 1, name);
    const inner = innerEnd === -1 ? "" : text.slice(tagEnd + 1, innerEnd);
    let block = null;
    if (innerEnd !== -1) {
      if (PAGE_HEADING_TAGS.includes(name)) {
        block = { kind: "heading", level: Number(name.charAt(1)), text: pageDecodeEntities(pageStripTags(inner)) };
      } else if (name === "p") {
        block = pageParagraphBlock(pageStripTags(inner), rules);
      } else if (name === "li" || name === "dt") {
        // A list item holding block children is a container instead.
        const nested = PAGE_NESTED_BLOCK_OPENERS.some((opener) => pageAsciiLower(inner).includes(opener));
        if (!nested) block = { kind: "list_item", text: pageDecodeEntities(pageStripTags(inner)) };
      } else if (name === "tr") {
        block = { kind: "table_row", text: pageDecodeEntities(pageCellTexts(inner).join(" | ")) };
      } else if (name === "pre") {
        const inherited = container && open < container.end ? container.classAttr : null;
        block = {
          kind: "code_block",
          language: pageResolveLanguage(rules, null, pagePreClass(rules, innerTag, inner, inherited), inner, null),
          text: pageDecodeEntities(pageStripTagsKeepLines(inner)),
        };
      } else {
        // An element annotated with a seed `html_code_attribute` (kotlinlang's
        // `data-lang`) is a code block in that language.
        const annotated = rules.codeAttributes.map((attr) => pageTagAttr(innerTag, attr)).find((value) => value);
        if (annotated) block = { kind: "code_block", language: annotated, text: pageDecodeEntities(pageStripTagsKeepLines(inner)) };
      }
      const containerClass = block === null ? pageTagAttr(innerTag, "class") : null;
      if (containerClass && rules.classPrefixes.some((prefix) => containerClass.includes(prefix))) {
        container = { classAttr: containerClass, end: innerEnd };
      }
    }
    const consumed = block !== null || PAGE_OPAQUE_TAGS.includes(name);
    if (block !== null) out.push(block);
    cursor = innerEnd !== -1 && consumed ? innerEnd + 2 + name.length : tagEnd + 1;
  }
  return out;
}

/**
 * The class a `<pre>` block's language is read from: the first of its own,
 * its inner `<code>` tag's and the enclosing container's that carries a seed
 * class prefix, else null.
 * @param {object} rules
 * @param {string} innerTag
 * @param {string} raw
 * @param {string|null} inherited
 * @returns {string|null}
 */
function pagePreClass(rules, innerTag, raw, inherited) {
  const codeOpen = raw.indexOf("<code");
  const tagClose = codeOpen === -1 ? -1 : raw.indexOf(">", codeOpen);
  const code = tagClose === -1 ? null : pageTagAttr(raw.slice(codeOpen + 1, tagClose), "class");
  const candidates = [pageTagAttr(innerTag, "class"), code, inherited];
  return candidates.find((value) => value !== null && rules.classPrefixes.some((prefix) => value.includes(prefix))) || null;
}

/**
 * Markdown walking: ATX headings, fenced code, list items, table rows and
 * blank-line-separated paragraphs.
 * @param {string} text
 * @param {object} rules
 * @param {string|null} urlHint
 * @returns {Array<object>}
 */
function pageMarkdownBlocks(text, rules, urlHint) {
  const out = [];
  const state = { paragraph: "" };
  const lines = pageLines(text);
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const trimmed = line.trimStart();
    if (PAGE_FENCES.includes(trimmed.slice(0, 3))) {
      pageFlushParagraph(state, rules, out);
      const fence = trimmed.charAt(0);
      const info = trimmed.replace(new RegExp("^\\" + fence + "+"), "").trim();
      let code = "";
      index += 1;
      for (; index < lines.length; index += 1) {
        const codeTrimmed = lines[index].trimStart();
        if (codeTrimmed.startsWith(fence) && codeTrimmed.split(fence).length - 1 >= 3) break;
        code += lines[index] + "\n";
      }
      out.push({
        kind: "code_block",
        language: pageResolveLanguage(rules, info, null, code, urlHint),
        text: code.trimEnd(),
      });
      continue;
    }
    const level = (trimmed.match(/^#+/) || [""])[0].length;
    if (level > 0 && trimmed.charAt(level) === " ") {
      pageFlushParagraph(state, rules, out);
      out.push({ kind: "heading", level: Math.min(level, 6), text: trimmed.slice(level + 1).trim() });
      continue;
    }
    if (trimmed.length >= 3 && PAGE_BULLETS.includes(trimmed.charAt(0)) && trimmed.charAt(1) === " ") {
      pageFlushParagraph(state, rules, out);
      out.push({ kind: "list_item", text: trimmed.slice(2).trim() });
      continue;
    }
    if (trimmed.startsWith("|") && trimmed.endsWith("|") && trimmed.length > 1) {
      pageFlushParagraph(state, rules, out);
      const cells = trimmed.replace(/^\|+|\|+$/g, "").split("|").map((cell) => cell.trim());
      out.push({ kind: "table_row", text: cells.join(" | ") });
      continue;
    }
    if (line.trim() === "") {
      pageFlushParagraph(state, rules, out);
      continue;
    }
    state.paragraph += line.trim() + " ";
  }
  pageFlushParagraph(state, rules, out);
  return out;
}

/**
 * Every scalar leaf of a JSON payload becomes a statement paragraph; object
 * fields are walked in key order, as the native map does.
 * @param {*} value
 * @param {Array<object>} out
 */
function pageJsonBlocks(value, out) {
  if (value === null || value === undefined) return;
  if (Array.isArray(value)) {
    for (const item of value) pageJsonBlocks(item, out);
  } else if (typeof value === "object") {
    for (const key of Object.keys(value).sort()) pageJsonBlocks(value[key], out);
  } else {
    out.push({ kind: "paragraph", text: String(value), command: false });
  }
}

/**
 * Plain (and PDF-extracted) text: blank-line-separated paragraphs, with a
 * short colon-ended single line as a heading.
 * @param {string} text
 * @param {object} rules
 * @returns {Array<object>}
 */
function pagePlainBlocks(text, rules) {
  const out = [];
  for (const raw of text.split("\n\n")) {
    const chunk = raw.trim();
    if (!chunk) continue;
    const lines = pageLines(chunk);
    if (lines.length === 1 && lines[0].endsWith(":") && lines[0].length < PAGE_HEADING_MAX_CHARS) {
      out.push({ kind: "heading", level: 2, text: lines[0].replace(/:+$/, "").trim() });
      continue;
    }
    out.push(pageParagraphBlock(lines.join(" "), rules));
  }
  return out;
}

/**
 * A paragraph, flagged as a command when its first token is a seed verb.
 * @param {string} text
 * @param {object} rules
 * @returns {object}
 */
function pageParagraphBlock(text, rules) {
  const trimmed = pageDecodeEntities(text).trim();
  const first = trimmed.split(/\s+/)[0] || "";
  return { kind: "paragraph", text: trimmed, command: first !== "" && rules.commandVerbs.includes(first) };
}

/**
 * @param {{paragraph: string}} state
 * @param {object} rules
 * @param {Array<object>} out
 */
function pageFlushParagraph(state, rules, out) {
  if (state.paragraph.trim()) out.push(pageParagraphBlock(state.paragraph, rules));
  state.paragraph = "";
}

/**
 * A code block's language in the seed's order (fence, HTML class prefix,
 * shebang and URL extension), else the seed's fallback (issue #1163 R2).
 * @param {object} rules
 * @param {string|null} fenceInfo
 * @param {string|null} classAttr
 * @param {string} codeText
 * @param {string|null} urlHint
 * @returns {string}
 */
function pageResolveLanguage(rules, fenceInfo, classAttr, codeText, urlHint) {
  for (const source of rules.tagSources) {
    if (source === "fence" && fenceInfo !== null) {
      const token = fenceInfo.split(/\s+/)[0] || "";
      if (token) return token;
    } else if (source === "html_class" && classAttr !== null) {
      for (const prefix of rules.classPrefixes) {
        const at = classAttr.indexOf(prefix);
        if (at === -1) continue;
        let token = classAttr.slice(at + prefix.length).trim().split(/\s+/)[0] || "";
        for (const strip of PAGE_CLASS_TOKEN_PREFIXES) {
          if (token.startsWith(strip)) token = token.slice(strip.length);
        }
        if (token) return token;
      }
    } else if (source === "heuristic") {
      const first = (pageLines(codeText)[0] || "").trim();
      const shebang = rules.shebangs.find((pair) => pair[0] === first);
      if (shebang) return shebang[1];
      if (urlHint) {
        const extension = rules.extensions.find((pair) => urlHint.includes(pair[0]));
        if (extension) return extension[1];
      }
    }
  }
  return rules.tagFallback;
}

/**
 * Value of `name=...` in a tag's attribute text, or null.
 * @param {string} innerTag
 * @param {string} name
 * @returns {string|null}
 */
function pageTagAttr(innerTag, name) {
  const needle = name + "=";
  const at = pageAsciiLower(innerTag).indexOf(needle);
  if (at === -1) return null;
  const rest = innerTag.slice(at + needle.length);
  const quote = rest.charAt(0);
  if (quote === "\"" || quote === "'") {
    const end = rest.indexOf(quote, 1);
    return end === -1 ? null : rest.slice(1, end);
  }
  return rest.trim().split(/\s+/)[0] || "";
}

/**
 * Strip every tag from an HTML fragment and collapse whitespace.
 * @param {string} fragment
 * @returns {string}
 */
function pageStripTags(fragment) {
  let out = "";
  let depth = 0;
  for (const character of fragment) {
    if (character === "<") depth += 1;
    else if (character === ">") depth = Math.max(0, depth - 1);
    else if (depth === 0) out += character;
  }
  return out.split(/\s+/).filter((word) => word.length > 0).join(" ");
}

/**
 * Strip tags line by line, for code blocks.
 * @param {string} fragment
 * @returns {string}
 */
function pageStripTagsKeepLines(fragment) {
  return pageLines(fragment).map(pageStripTags).join("\n");
}

/**
 * Text of each `<td>`/`<th>` cell of a table row.
 * @param {string} row
 * @returns {Array<string>}
 */
function pageCellTexts(row) {
  const lower = pageAsciiLower(row);
  const cells = [];
  let cursor = 0;
  for (;;) {
    const open = lower.indexOf("<t", cursor);
    if (open === -1) break;
    const tagEnd = lower.indexOf(">", open);
    if (tagEnd === -1) break;
    const name = lower.slice(open + 1, tagEnd).replace(/^\/+/, "").trim().split(/\s+/)[0] || "";
    if (PAGE_CELL_TAGS.includes(name)) {
      const close = lower.indexOf("</t", tagEnd + 1);
      if (close !== -1) cells.push(pageStripTags(row.slice(tagEnd + 1, close)));
    }
    cursor = tagEnd + 1;
  }
  return cells;
}

/**
 * Decode the handful of entities real pages use.
 * @param {string} text
 * @returns {string}
 */
function pageDecodeEntities(text) {
  let out = text;
  for (const [entity, character] of PAGE_ENTITIES) out = out.split(entity).join(character);
  return out;
}

/**
 * Every statement a page network carries: its block texts, its domain, and
 * the rediscovery record's values.
 * @param {object} network
 * @returns {Array<string>}
 */
function pageNetworkStatements(network) {
  const out = network.blocks.map((block) => block.text).filter((text) => text !== "");
  if (network.domain) out.push(network.domain);
  if (network.rediscovery) {
    for (const value of Object.values(network.rediscovery)) out.push(String(value));
  }
  return out;
}

/**
 * True when every non-empty statement is carried by the network, exactly
 * or as part of a longer statement (issue #1163 R3, generic covers bespoke).
 * @param {object} network
 * @param {Array<string>} statements
 * @returns {boolean}
 */
function pageCoversStatements(network, statements) {
  const carried = pageNetworkStatements(network);
  return statements
    .filter((statement) => statement.trim() !== "")
    .every((statement) => carried.some((term) => term.includes(statement)));
}

/**
 * The working-memory key of a formalized page (issue #1163 R6).
 * @param {string} url
 * @param {string} sha256
 * @returns {string}
 */
function pageKey(url, sha256) {
  return "page:" + sha256 + ":" + url;
}

/**
 * A captured page formalized, indexed and annotated with its rediscovery
 * procedure (issue #1163 R9). Each paragraph is tied to its command: the
 * paragraph itself when it is a command, else the code block or command
 * paragraph that directly follows it.
 * @param {{url: string, sha256: string, fetchedAt: string, cached: boolean, text: string}} capture
 * @param {string} query
 * @param {number} rank
 * @param {number} trust
 * @param {string|null} mimeHint
 * @returns {object}
 */
function formalizedPageFromCapture(capture, query, rank, trust, mimeHint) {
  const network = formalizePage(capture.text, mimeHint, capture.url);
  network.rediscovery = {
    query: query,
    rank: String(rank),
    url: capture.url,
    sha256: capture.sha256,
    fetched_at: String(capture.fetchedAt || ""),
    cached: String(Boolean(capture.cached)),
    trust: String(trust),
  };
  return pageIndexed(network, { url: capture.url, sha256: capture.sha256, rank: rank, query: query, trust: trust, capture: capture });
}

/**
 * A page record over a formalized network: its code blocks, command
 * paragraphs, and paragraphs tied to the command each introduces.
 * @param {object} network
 * @param {object} fields
 * @returns {object}
 */
function pageIndexed(network, fields) {
  const paragraphs = [];
  network.blocks.forEach((block, index) => {
    if (block.kind !== "paragraph") return;
    const next = network.blocks[index + 1];
    let command = block.command ? block : null;
    if (command === null && next && (next.kind === "code_block" || (next.kind === "paragraph" && next.command))) {
      command = next;
    }
    paragraphs.push({ text: block.text, link: block, command: command });
  });
  return Object.assign({}, fields, {
    network: network,
    codeBlocks: network.blocks.filter((block) => block.kind === "code_block"),
    commands: network.blocks.filter((block) => block.kind === "paragraph" && block.command),
    paragraphs: paragraphs,
  });
}

/**
 * An empty store of formalized pages, keyed by URL and SHA-256.
 * @returns {{pages: Map<string, object>}}
 */
function createFormalizedPageStore() {
  return { pages: new Map() };
}

/**
 * Store a page under its key, replacing an earlier version; returns the key.
 * @param {{pages: Map<string, object>}} store
 * @param {object} page
 * @returns {string}
 */
function formalizedPageStoreInsert(store, page) {
  const key = pageKey(page.url, page.sha256);
  store.pages.set(key, page);
  return key;
}

/**
 * "Code blocks on `<domain>` whose text contains `<term>`" (issue #1163 R8).
 * @param {{pages: Map<string, object>}} store
 * @param {string} domain
 * @param {string} term
 * @returns {Array<object>}
 */
function formalizedPageCodeBlocksOn(store, domain, term) {
  const out = [];
  for (const page of store.pages.values()) {
    if (pageUrlDomain(page.url) !== domain) continue;
    for (const block of page.codeBlocks) if (block.text.includes(term)) out.push(block);
  }
  return out;
}

/**
 * "The command in the paragraph that mentions `<phrase>`" (issue #1163
 * R8): only paragraphs that themselves mention the phrase answer, each with
 * the command tied to it.
 * @param {{pages: Map<string, object>}} store
 * @param {string} phrase
 * @returns {Array<object>}
 */
function formalizedPageCommandMentioning(store, phrase) {
  const out = [];
  for (const page of store.pages.values()) {
    for (const paragraph of page.paragraphs) {
      if (paragraph.command !== null && paragraph.text.includes(phrase)) out.push(paragraph.command);
    }
  }
  return out;
}

/**
 * The trust score in 0..=100 from the seed's weights (issue #1163 R7); it
 * ranks sources and never excludes one.
 * @param {{officialSite: boolean, https: boolean, primacy: string|null, openLicense: boolean, agreementPages: number}} features
 * @returns {number}
 */
function pageTrustScore(features) {
  const weights = {};
  const grades = {};
  let agreementScale = 1;
  for (const record of pageSeedRecords(seedRawText(SEED_RAW, PAGE_TRUST_FILE))) {
    if (record.name !== "feature") continue;
    const name = record.value || childValue(record, "name");
    if (!name) continue;
    const weight = Number.parseFloat(childValue(record, "weight"));
    if (Number.isFinite(weight)) weights[name] = weight;
    const scale = Number.parseFloat(childValue(record, "agreement_scale"));
    if (name === "cross_page_agreement" && Number.isFinite(scale)) agreementScale = Math.max(scale, 1);
    for (const grade of record.children) {
      if (grade.name !== "grade") continue;
      const factor = Number.parseFloat(childValue(grade, "factor"));
      if (childValue(grade, "name") && Number.isFinite(factor)) grades[childValue(grade, "name")] = factor;
    }
  }
  const weightOf = (name) => weights[name] || 0;
  let score = 0;
  if (features.officialSite) score += weightOf("official_site");
  if (features.https) score += weightOf("https");
  if (features.primacy) score += weightOf("primacy") * (grades[features.primacy] || 0);
  if (features.openLicense) score += weightOf("open_license");
  const agreement = Math.min(features.agreementPages || 0, Math.floor(agreementScale)) / agreementScale;
  score += weightOf("cross_page_agreement") * agreement;
  return Math.round(Math.min(Math.max(score, 0), 1) * 100);
}

/**
 * Match a query against a seed page-query template (issue #1163 R8):
 * literal chunks match ASCII case-insensitively, each slot takes the text up
 * to the next literal chunk (the last slot takes the rest) and must be
 * non-empty. Returns the slot values, or null.
 * @param {string} template
 * @param {string} query
 * @returns {object|null}
 */
function pageMatchTemplate(template, query) {
  const pieces = template.split(/(\{[^}]*\})/).filter((piece) => piece !== "");
  const text = query.trim().replace(/[?.!]+$/, "").trimEnd();
  const lower = pageAsciiLower(text);
  const slots = {};
  let position = 0;
  for (let index = 0; index < pieces.length; index += 1) {
    const piece = pieces[index];
    if (!piece.startsWith("{")) {
      const literal = pageAsciiLower(piece);
      if (!lower.startsWith(literal, position)) return null;
      position += literal.length;
      continue;
    }
    const next = pieces[index + 1];
    if (next !== undefined && next.startsWith("{")) return null;
    const end = next === undefined ? text.length : lower.indexOf(pageAsciiLower(next), position);
    if (end === -1) return null;
    const value = text.slice(position, end).trim().replace(/^["'`]+|["'`]+$/g, "").trim();
    if (!value) return null;
    slots[piece.slice(1, -1)] = value;
    position = end;
  }
  return position === text.length ? slots : null;
}

/**
 * Parse and run a page query against a store (issue #1163 R8); null when no
 * seed template matches.
 * @param {{pages: Map<string, object>}} store
 * @param {string} query
 * @returns {Array<object>|null}
 */
function formalizedPageQuery(store, query) {
  for (const [name, template] of pageFormalizationRules().pageQueries) {
    const slots = pageMatchTemplate(template, query);
    if (slots === null) continue;
    if (name === "code_blocks_on") return formalizedPageCodeBlocksOn(store, pageAsciiLower(slots.domain || ""), slots.term || "");
    if (name === "command_mentioning") return formalizedPageCommandMentioning(store, slots.phrase || "");
  }
  return null;
}

/**
 * The capture of a page already in the store from `url`, replayed as a
 * cache hit, so a fetch path consults working memory before it fetches
 * (issue #1163 R6); null when no stored page came from the URL.
 * @param {{pages: Map<string, object>}} store
 * @param {string} url
 * @returns {object|null}
 */
function formalizedPageStoreCapture(store, url) {
  for (const page of store.pages.values()) {
    if (page.url === url && page.capture) return Object.assign({}, page.capture, { cached: true });
  }
  return null;
}

/**
 * The seed's trust-feature detectors: open-license markers and the
 * official-website claim property (issue #1163 R7).
 * @returns {{licenseMarkers: Array<string>, officialSiteProperty: string}}
 */
function pageTrustFeatureRules() {
  const rules = { licenseMarkers: [], officialSiteProperty: "" };
  for (const record of pageSeedRecords(seedRawText(SEED_RAW, PAGE_TRUST_FILE))) {
    if (record.name !== "feature") continue;
    for (const child of record.children) {
      if (child.name === "license_marker" && child.value) rules.licenseMarkers.push(pageAsciiLower(child.value));
      if (child.name === "claim_property" && child.value) rules.officialSiteProperty = child.value;
    }
  }
  return rules;
}

/**
 * The official websites a Wikidata entity payload states through the seed's
 * claim property (P856).
 * @param {string} entityText
 * @returns {Array<string>}
 */
function pageOfficialWebsites(entityText) {
  const property = pageTrustFeatureRules().officialSiteProperty;
  let value = null;
  try {
    value = JSON.parse(String(entityText || ""));
  } catch {
    return [];
  }
  const out = [];
  for (const entity of Object.values((value && value.entities) || {})) {
    const claims = (entity && entity.claims && entity.claims[property]) || [];
    for (const claim of Array.isArray(claims) ? claims : []) {
      const site = claim && claim.mainsnak && claim.mainsnak.datavalue && claim.mainsnak.datavalue.value;
      if (typeof site === "string") out.push(site);
    }
  }
  return out;
}

/**
 * The primacy grade of the registered source whose endpoint shares the
 * domain, or null.
 * @param {string} domain
 * @returns {string|null}
 */
function pageRegistryPrimacy(domain) {
  if (!domain) return null;
  for (const record of pageSeedRecords(seedRawText(SEED_RAW, "sources-registry.lino"))) {
    if (record.name !== "source") continue;
    const api = childValue(record, "api");
    const primacy = childValue(record, "primacy");
    if (api && primacy && pageUrlDomain(api) === domain) return primacy;
  }
  return null;
}

/**
 * Compute a page's trust features (issue #1163 R7): HTTPS from the URL, the
 * registry primacy of the domain, an open license from the seed's markers,
 * agreement from stored pages on other domains, and the official site from
 * the given Wikidata P856 sites.
 * @param {string} url
 * @param {string} text
 * @param {{pages: Map<string, object>}} store
 * @param {Array<string>} officialSites
 * @returns {object}
 */
function pageTrustFeatures(url, text, store, officialSites) {
  const rules = pageTrustFeatureRules();
  const domain = pageUrlDomain(url);
  const lower = pageAsciiLower(String(text || ""));
  const own = new Set(
    formalizePage(text, null, url).blocks
      .filter((block) => (block.kind === "paragraph" || block.kind === "code_block") && block.text.trim() !== "")
      .map((block) => block.text),
  );
  let agreementPages = 0;
  for (const page of store.pages.values()) {
    if (pageUrlDomain(page.url) === domain) continue;
    const texts = page.paragraphs.map((paragraph) => paragraph.text).concat(page.codeBlocks.map((block) => block.text));
    if (texts.some((candidate) => own.has(candidate))) agreementPages += 1;
  }
  return {
    officialSite: domain !== "" && (officialSites || []).some((site) => pageUrlDomain(site) === domain),
    https: url.startsWith("https://"),
    primacy: pageRegistryPrimacy(domain),
    openLicense: rules.licenseMarkers.some((marker) => lower.includes(marker)),
    agreementPages: agreementPages,
  };
}

/**
 * Read a document as a conversion source through the formalizer (issue
 * #1163 R13); the formats and their mime hints are the seed's
 * `document_source` rows. Null for any other format.
 * @param {string} format
 * @param {string} text
 * @returns {object|null}
 */
function formalizeDocumentSource(format, text) {
  const wanted = pageAsciiLower(String(format || "").trim());
  if (!wanted) return null;
  const source = pageFormalizationRules().documentSources.find((candidate) => {
    return pageAsciiLower(candidate.name) === wanted || candidate.aliases.some((alias) => pageAsciiLower(alias) === wanted);
  });
  if (!source) return null;
  const network = formalizePage(text, source.hint, null);
  return {
    sourceFormat: source.name,
    mimeHint: source.hint,
    blocks: network.blocks.map((block) => [block.kind, block.text]),
    codeLanguages: network.blocks.filter((block) => block.kind === "code_block").map((block) => block.language),
  };
}

/**
 * Split a prompt into a page query (its first line, the seed's separator
 * trimmed) and the page it supplies (the rest); null when either is empty.
 * @param {string} prompt
 * @returns {{query: string, page: string}|null}
 */
function pageSplitSuppliedPage(prompt) {
  const text = String(prompt || "").trim();
  const newline = text.indexOf("\n");
  if (newline === -1) return null;
  const separator = pageFormalizationRules().suppliedPageSeparator;
  let query = text.slice(0, newline).trim();
  if (separator) while (query.endsWith(separator)) query = query.slice(0, -separator.length);
  query = query.trimEnd();
  const page = text.slice(newline + 1).trim();
  return query && page ? { query: query, page: page } : null;
}

/**
 * A page query over a page the prompt supplies (issue #1163 R10): the page
 * is formalized, the seed's page-query template runs against it, and the
 * answer is the text of the blocks it links to. Null when the prompt carries
 * no page, no template matches, or no block answers.
 * @param {string} prompt
 * @returns {object|null}
 */
function tryPageQueryText(prompt) {
  const split = pageSplitSuppliedPage(prompt);
  if (split === null) return null;
  const network = formalizePage(split.page, null, null);
  // The worker has no synchronous SHA-256, so a supplied page is keyed by the
  // worker's stable content id, as formalization_support keys a document.
  const digest = conceptStableId("page", split.page).replace("page_", "");
  const page = pageIndexed(network, { url: "", sha256: digest, rank: 0, query: split.query, trust: 0 });
  const store = createFormalizedPageStore();
  const key = formalizedPageStoreInsert(store, page);
  const blocks = formalizedPageQuery(store, split.query);
  if (!blocks || blocks.length === 0) return null;
  const texts = blocks.map((block) => block.text);
  const trace = [
    `page_query:${split.query}`,
    `page_query_page:${key} blocks=${page.codeBlocks.length + page.paragraphs.length}`,
    ...texts.map((text) => `page_query_answer:${text}`),
  ];
  return {
    intent: "page_query",
    content: texts.join("\n"),
    confidence: 0.85,
    evidence: ["handler:page_query_text", ...trace, "response:page_query"],
    trace: trace,
  };
}
