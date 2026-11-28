// --- Substitution engine (mirror of src/substitution.rs) -------------------

function unescapeLinoValue(value) {
  let out = "";
  for (let index = 0; index < value.length; index += 1) {
    const ch = value[index];
    // Links Notation doubles a delimiter to escape it; the seed corpus now
    // writes that canonical form, so both dialects are read.
    if (ch === '"' && value[index + 1] === '"') {
      out += '"';
      index += 1;
      continue;
    }
    if (ch === "\\" && index + 1 < value.length) {
      const next = value[index + 1];
      if (next === "n" || next === "t" || next === "r") {
        out += { n: "\n", t: "\t", r: "\r" }[next];
        index += 1;
        continue;
      }
      if (next === '"' || next === "\\") {
        out += next;
        index += 1;
        continue;
      }
    }
    out += ch;
  }
  return out;
}

function unescapeSingleLinoValue(value) {
  let out = "";
  for (let index = 0; index < value.length; index += 1) {
    const ch = value[index];
    if (ch === "\\" && index + 1 < value.length) {
      const next = value[index + 1];
      if (next === "n") {
        out += "\n";
        index += 1;
        continue;
      }
      if (next === "\\") {
        out += "\\";
        index += 1;
        continue;
      }
      if (next === "x" && value.slice(index + 2, index + 4) === "27") {
        out += "'";
        index += 3;
        continue;
      }
    }
    out += ch;
  }
  return out;
}

function decodeRawReference(raw) {
  const value = String(raw || "").trim();
  if (value === "unformalized-raw" || value === "codepoints") return "";
  if (value.startsWith("unformalized-raw ")) {
    return decodeCodepoints(value.slice("unformalized-raw ".length));
  }
  if (value.startsWith("codepoints ")) {
    return decodeCodepoints(value.slice("codepoints ".length));
  }
  return value;
}

function stripLinoComment(line) {
  // Mirrors rust/src/seed/parser.rs strip_comment: any of the three quote
  // characters opens a string the comment rule must not see into, a doubled
  // single quote is an escape, and a backslash escapes inside double quotes
  // and backticks.
  let quote = null;
  let escaped = false;
  let previousWasSpace = true;
  for (let index = 0; index < line.length; index += 1) {
    const ch = line[index];
    if (quote) {
      if (escaped) {
        escaped = false;
      } else if ((quote === '"' || quote === "`") && ch === "\\") {
        escaped = true;
      } else if (quote === "'" && ch === "'" && line[index + 1] === "'") {
        index += 1;
      } else if (ch === quote) {
        quote = null;
      }
      continue;
    }
    if (ch === '"' || ch === "'" || ch === "`") {
      quote = ch;
      previousWasSpace = false;
      continue;
    }
    if (ch === "#" && previousWasSpace) return line.slice(0, index);
    previousWasSpace = /\s/.test(ch);
  }
  return line;
}

function parseLinoValue(raw) {
  const trimmed = raw.trim(); // a backtick string unescapes as a double-quoted one (decode_quoted_reference)
  if (trimmed.length >= 2 && ['"', "`"].some((quote) => trimmed.startsWith(quote) && trimmed.endsWith(quote))) {
    return unescapeLinoValue(trimmed.slice(1, -1));
  }
  if (trimmed.length >= 2 && trimmed.startsWith("'") && trimmed.endsWith("'")) {
    return unescapeSingleLinoValue(trimmed.slice(1, -1));
  }
  return decodeRawReference(trimmed);
}

function parseLinoTree(text) {
  const root = { name: "", value: "", depth: -1, children: [] };
  const stack = [root];
  for (const line of (globalThis.FormalAiSeed?.expandConciseLexemes(text) ?? text).split("\n")) {
    const stripped = stripLinoComment(line);
    if (!stripped.trim()) continue;
    const indent = stripped.length - stripped.trimStart().length;
    const depth = indent / 2;
    const rest = stripped.trim();
    const colonIndex = rest.indexOf(":");
    const whitespaceIndex = rest.search(/\s/);
    let name = "";
    let value = "";
    if (colonIndex !== -1 && (whitespaceIndex === -1 || colonIndex < whitespaceIndex)) {
      name = rest.slice(0, colonIndex).trim();
      value = parseLinoValue(rest.slice(colonIndex + 1));
    } else if (whitespaceIndex === -1) {
      name = rest;
    } else {
      name = rest.slice(0, whitespaceIndex);
      value = parseLinoValue(rest.slice(whitespaceIndex + 1));
    }
    const node = { name, value, depth, children: [] };
    while (stack.length && stack[stack.length - 1].depth >= depth) stack.pop();
    stack[stack.length - 1].children.push(node);
    stack.push(node);
  }
  return root;
}

function parsePatternNode(text) {
  if (!text) throw new Error("pattern node is empty");
  if (text.startsWith("$")) return { kind: "variable", variable: text.slice(1) };
  const dollar = text.indexOf("$");
  if (dollar !== -1) {
    return { kind: "prefix", prefix: text.slice(0, dollar), variable: text.slice(dollar + 1) };
  }
  return { kind: "literal", value: text };
}

function parseLinkPattern(text) {
  const index = text.indexOf("->");
  if (index === -1) throw new Error(`expected \`from -> to\`, got \`${text}\``);
  return {
    from: parsePatternNode(text.slice(0, index).trim()),
    to: parsePatternNode(text.slice(index + 2).trim()),
  };
}

function parseCrudEvent(value) {
  const map = {
    manual: "manual",
    apply: "manual",
    create: "create",
    created: "create",
    read: "read",
    select: "read",
    query: "read",
    update: "update",
    updated: "update",
    delete: "delete",
    deleted: "delete",
  };
  const key = String(value).trim().toLowerCase();
  if (!map[key]) throw new Error(`invalid CRUD event: ${value}`);
  return map[key];
}

function parseSubstitutionRule(node) {
  const rule = { id: node.value, order: 0, events: [], conditions: [], actions: [] };
  for (const child of node.children) {
    switch (child.name) {
      case "order": {
        const parsed = parseInt(child.value, 10);
        rule.order = Number.isNaN(parsed) ? 0 : parsed;
        break;
      }
      case "event":
        rule.events.push(parseCrudEvent(child.value));
        break;
      case "when":
        rule.conditions.push(parseLinkPattern(child.value));
        break;
      case "replace": {
        const add = child.children
          .filter((grandchild) => grandchild.name === "with")
          .map((grandchild) => parseLinkPattern(grandchild.value));
        rule.actions.push({ remove: parseLinkPattern(child.value), add });
        break;
      }
      default:
        break;
    }
  }
  return rule;
}

function parseSubstitutionRules(text) {
  const tree = parseLinoTree(text.trim());
  const root = tree.children[0];
  if (!root || root.name !== "substitution_rules") {
    throw new Error("not a substitution_rules document");
  }
  const idNode = root.children.find((child) => child.name === "id");
  const id = idNode ? idNode.value : "";
  const rules = root.children
    .filter((child) => child.name === "rule")
    .map(parseSubstitutionRule);
  rules.sort((left, right) =>
    left.order - right.order ||
    (left.id < right.id ? -1 : left.id > right.id ? 1 : 0),
  );
  return { id, rules };
}

const LINK_KEY_SEPARATOR = "\u0000";
const linkKey = (link) => `${link.from}${LINK_KEY_SEPARATOR}${link.to}`;
const linkFromKey = (key) => {
  const [from, to] = key.split(LINK_KEY_SEPARATOR);
  return { from, to };
};

function sortedLinksFromSet(linkSet) {
  return Array.from(linkSet, linkFromKey).sort((left, right) =>
    left.from < right.from
      ? -1
      : left.from > right.from
        ? 1
        : left.to < right.to
          ? -1
          : left.to > right.to
            ? 1
            : 0,
  );
}

function bindVariable(bindings, variable, value) {
  if (Object.prototype.hasOwnProperty.call(bindings, variable)) {
    return bindings[variable] === value;
  }
  bindings[variable] = value;
  return true;
}

function nodeMatches(pattern, value, bindings) {
  if (pattern.kind === "literal") return pattern.value === value;
  if (pattern.kind === "variable") return bindVariable(bindings, pattern.variable, value);
  if (!value.startsWith(pattern.prefix)) return false;
  return bindVariable(bindings, pattern.variable, value.slice(pattern.prefix.length));
}

function patternMatchesLink(pattern, link, bindings) {
  return (
    nodeMatches(pattern.from, link.from, bindings) &&
    nodeMatches(pattern.to, link.to, bindings)
  );
}

function instantiateNode(node, bindings) {
  if (node.kind === "literal") return node.value;
  if (node.kind === "variable") {
    return Object.prototype.hasOwnProperty.call(bindings, node.variable)
      ? bindings[node.variable]
      : null;
  }
  const value = bindings[node.variable];
  return value === undefined ? null : node.prefix + value;
}

function instantiatePattern(pattern, bindings) {
  const from = instantiateNode(pattern.from, bindings);
  const to = instantiateNode(pattern.to, bindings);
  if (from === null || to === null) return null;
  return { from, to };
}

function findBindings(links, patterns, index, bindings) {
  if (index >= patterns.length) return bindings;
  const pattern = patterns[index];
  for (const link of links) {
    const candidate = Object.assign({}, bindings);
    if (patternMatchesLink(pattern, link, candidate)) {
      const found = findBindings(links, patterns, index + 1, candidate);
      if (found) return found;
    }
  }
  return null;
}
