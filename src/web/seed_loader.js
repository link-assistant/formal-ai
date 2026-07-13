// Shared Links Notation seed loader for the demo.
//
// The browser demo loads its multilingual response phrases, concept summaries,
// project registry, tool registry, language-detection rules, prompt patterns, and agent metadata
// from `src/web/seed/*.lino` instead of hardcoded JavaScript constants. Every
// prompt-handling decision is therefore data-driven: a user who wants to
// retune their own agent can edit a `.lino` file and ship it alongside the
// static site without touching the worker code.
//
// `src/web/seed/` is a deploy artefact synchronised from `data/seed/` (the
// canonical source shared with the Rust solver, CLI, Telegram bot, and HTTP
// server). Run `scripts/sync-seed.sh` to refresh the copy.
//
// The loader is intentionally minimal: indentation-based, untyped, and shared
// between the main thread (`window.FormalAiSeed`) and the worker
// (`self.FormalAiSeed`) without any bundler step.

(function (global) {
  "use strict";

  var DEFAULT_FILES = [
    "seed/agent-info.lino",
    "seed/multilingual-responses.lino",
    "seed/concepts.lino",
    "seed/concept-contexts.lino",
    "seed/facts.lino",
    "seed/projects.lino",
    "seed/brainstorm-seeds.lino",
    "seed/personas.lino",
    "seed/summary-topics.lino",
    "seed/coreference.lino",
    "seed/tools.lino",
    "seed/language-detection.lino",
    "seed/prompt-patterns.lino",
    "seed/intent-routing.lino",
    "seed/operation-vocabulary.lino",
    "seed/numeric-list-operations.lino",
    "seed/coding-idioms.lino",
    "seed/terminal-commands.lino",
    "seed/program-plan-rules.lino",
    "seed/market-price-references.lino",
    "seed/meanings.lino",
    "seed/meanings-behavior-rules.lino",
    "seed/meanings-calculator.lino",
    "seed/meanings-calendar.lino",
    "seed/meanings-coding-catalog.lino",
    "seed/meanings-conversation.lino",
    "seed/meanings-definition-merge.lino",
    "seed/meanings-docs.lino",
    "seed/meanings-facts.lino",
    "seed/meanings-feature-capability.lino",
    "seed/meanings-file-write.lino",
    "seed/meanings-finance.lino",
    "seed/meanings-how.lino",
    "seed/meanings-intent.lino",
    "seed/meanings-lexical-meta.lino",
    "seed/meanings-links-root.lino",
    "seed/meanings-meta.lino",
    "seed/meanings-ontology.lino",
    "seed/meanings-playwright.lino",
    "seed/meanings-policy.lino",
    "seed/meanings-program-synthesis.lino",
    "seed/meanings-proof.lino",
    "seed/meanings-research-table.lino",
    "seed/meanings-semantic-meta.lino",
    "seed/meanings-skill-compiler.lino",
    "seed/meanings-software-project.lino",
    "seed/meanings-summary.lino",
    "seed/meanings-tool-access.lino",
    "seed/meanings-translation.lino",
    "seed/meanings-units.lino",
    "seed/meanings-web-followup.lino",
    "seed/meanings-web-navigation.lino",
    "seed/meanings-web-research.lino",
    "seed/meanings-web-search-query.lino",
    "seed/meanings-web-search.lino",
    "seed/meanings-wikidata.lino",
    "seed/greetings.lino",
    "seed/identity.lino",
    "seed/hello-world-programs.lino",
    "seed/self-improvement-loop.lino",
    "seed/demo-dialogs.lino",
    "seed/environments.lino",
  ];

  function isWorker() {
    return (
      typeof global.WorkerGlobalScope !== "undefined" &&
      global instanceof global.WorkerGlobalScope
    );
  }

  function assetVersion() {
    if (typeof global.FORMAL_AI_ASSET_VERSION === "string") {
      return global.FORMAL_AI_ASSET_VERSION;
    }
    try {
      var search = global.location && global.location.search;
      var match = search && /[?&]v=([^&]+)/.exec(search);
      return match ? decodeURIComponent(match[1].replace(/\+/g, " ")) : "";
    } catch (_error) {
      return "";
    }
  }

  function withAssetVersion(url) {
    var version = assetVersion();
    if (!version) return url;
    if (
      url.indexOf("://") !== -1 ||
      url.indexOf("//") === 0 ||
      url.indexOf("data:") === 0
    ) {
      return url;
    }
    return (
      url +
      (url.indexOf("?") === -1 ? "?" : "&") +
      "v=" +
      encodeURIComponent(version)
    );
  }

  // Single left-to-right pass so escape sequences never re-trigger each other
  // (e.g. `\\n`, an escaped backslash followed by `n`, must stay `\n`, not a
  // newline). Mirrors `src/seed/parser.rs::unescape_value` and serves every
  // quote style emitted by the seed migration (`"`, `'`, and backticks).
  function unescapeQuoted(value) {
    var source = String(value || "");
    var out = "";
    for (var i = 0; i < source.length; i += 1) {
      var ch = source[i];
      if (ch !== "\\") {
        out += ch;
        continue;
      }
      var next = source[i + 1];
      if (next === undefined) {
        out += "\\";
      } else if (next === "n") {
        out += "\n";
        i += 1;
      } else if (next === "r") {
        out += "\r";
        i += 1;
      } else if (next === "t") {
        out += "\t";
        i += 1;
      } else if (next === "\\" || next === '"' || next === "'" || next === "`") {
        out += next;
        i += 1;
      } else if (next === "x" && source[i + 2] === "2" && source[i + 3] === "7") {
        out += "'";
        i += 3;
      } else {
        out += "\\" + next;
        i += 1;
      }
    }
    return out;
  }

  function unescapeValue(value) {
    return unescapeQuoted(value);
  }

  function unescapeSingleValue(value) {
    return unescapeQuoted(value);
  }

  function decodeRawReference(value) {
    var raw = String(value || "");
    if (raw === "unformalized-raw" || raw === "codepoints") return "";
    var unformalizedPrefix = "unformalized-raw ";
    var codepointsPrefix = "codepoints ";
    var prefix = "";
    if (raw.indexOf(unformalizedPrefix) === 0) {
      prefix = unformalizedPrefix;
    } else if (raw.indexOf(codepointsPrefix) === 0) {
      prefix = codepointsPrefix;
    } else {
      return raw;
    }
    return raw
      .slice(prefix.length)
      .trim()
      .split(/\s+/)
      .filter(Boolean)
      .map(function (part) {
        return String.fromCodePoint(parseInt(part, 10) || 0);
      })
      .join("");
  }

  function stripComment(line) {
    var quote = null;
    var escaped = false;
    var previousWasSpace = true;
    for (var i = 0; i < line.length; i += 1) {
      var ch = line[i];
      if (quote !== null) {
        if (escaped) {
          escaped = false;
        } else if ((quote === '"' || quote === "`") && ch === "\\") {
          escaped = true;
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
      if (ch === "#" && previousWasSpace) return line.slice(0, i);
      previousWasSpace = /\s/.test(ch);
    }
    return line;
  }

  // Parse an indented Links Notation document into a nested structure:
  //
  //   root_node
  //     child "name"
  //       key "value"
  //
  // -> { name: "root_node", children: [ { name: "child", id: "name",
  //          children: [ { name: "key", id: "value", children: [] } ] } ] }
  function parseLino(text) {
    var lines = String(text || "").split(/\r?\n/);
    var root = { name: "", id: "", value: "", children: [], indent: -1 };
    var stack = [root];
    for (var i = 0; i < lines.length; i += 1) {
      var line = lines[i];
      if (!line || /^\s*$/.test(stripComment(line))) continue;
      var indentMatch = /^(\s*)/.exec(line);
      var indent = indentMatch ? indentMatch[1].length : 0;
      var content = stripComment(line.slice(indent)).trim();
      // Walk back up to find the parent for this indent.
      while (stack.length > 1 && stack[stack.length - 1].indent >= indent) {
        stack.pop();
      }
      var parent = stack[stack.length - 1];
      var node = parseLinoLine(content, indent);
      parent.children.push(node);
      stack.push(node);
    }
    return root.children.length === 1 ? root.children[0] : root;
  }

  function parseLinoLine(content, indent) {
    var node = {
      name: "",
      id: "",
      value: "",
      children: [],
      indent: indent,
    };
    var colon = content.indexOf(":");
    var firstSpace = content.search(/\s/);
    if (colon !== -1 && (firstSpace === -1 || colon < firstSpace)) {
      node.name = content.slice(0, colon).trim();
      node.id = decodeRawReference(content.slice(colon + 1).trim());
      node.value = node.id;
      return node;
    }
    var parts = /^(\S+)(?:\s+([\s\S]*))?$/.exec(content.trim());
    if (!parts) return node;
    node.name = parts[1] || "";
    var rest = (parts[2] || "").trim();
    var delimiter = rest[0];
    if (delimiter === '"' || delimiter === "'" || delimiter === "`") {
      // The migration backslash-escapes `\` and newlines inside every quote
      // style, so the closing scan honours escapes regardless of delimiter.
      var closing = -1;
      for (var i = 1; i < rest.length; i += 1) {
        if (rest[i] === "\\") {
          i += 1;
          continue;
        }
        if (rest[i] === delimiter) {
          closing = i;
          break;
        }
      }
      // Only treat it as a quoted scalar when the quote spans the whole value.
      if (closing !== -1 && rest.slice(closing + 1).trim().length === 0) {
        node.id = unescapeQuoted(rest.slice(1, closing));
        node.value = node.id;
        return node;
      }
    }
    node.id = decodeRawReference(rest);
    node.value = node.id;
    return node;
  }

  function findChildren(node, name) {
    if (!node || !Array.isArray(node.children)) return [];
    return node.children.filter(function (child) {
      return child.name === name;
    });
  }

  function findChildValue(node, name) {
    var match = findChildren(node, name)[0];
    return match ? match.id : "";
  }

  function findChildValueAlias(node, primary, fallback) {
    var value = findChildValue(node, primary);
    return value || findChildValue(node, fallback);
  }

  function extractMultilingualResponses(node) {
    // The seed stores both a canonical `text` (kept stable so deterministic
    // tests can match it) and zero or more `variant` entries. Issue #27 adds
    // `variant`s to greetings so the demo can randomise its hello messages;
    // issue #160 adds separated courtesy acknowledgement/follow-up fragments.
    var responses = {};
    if (!node) return responses;
    var entries = findChildren(node, "response");
    for (var i = 0; i < entries.length; i += 1) {
      var entry = entries[i];
      var intent = findChildValue(entry, "intent");
      var language = findChildValue(entry, "language");
      var text = findChildValue(entry, "text");
      if (!intent || !language || !text) continue;
      var variantNodes = findChildren(entry, "variant");
      var variants = variantNodes
        .map(function (variant) {
          return variant.id || "";
        })
        .filter(function (value) {
          return value && value.length > 0;
        });
      var acknowledgementNodes = findChildren(entry, "ack_variant");
      var acknowledgements = acknowledgementNodes
        .map(function (variant) {
          return variant.id || "";
        })
        .filter(function (value) {
          return value && value.length > 0;
        });
      var followUpNodes = findChildren(entry, "follow_up_variant");
      var followUps = followUpNodes
        .map(function (variant) {
          return variant.id || "";
        })
        .filter(function (value) {
          return value && value.length > 0;
        });
      if (!responses[intent]) responses[intent] = {};
      responses[intent][language] = {
        text: text,
        variants: variants.length > 0 ? variants : [text],
        acknowledgements: acknowledgements,
        followUps: followUps,
      };
    }
    return responses;
  }

  function extractConcepts(root) {
    if (!root || !Array.isArray(root.children)) return [];
    // Concept files are flat — each top-level child is one record.
    var concepts = [];
    var iterate = function (item) {
      if (!item || !item.name) return;
      if (!item.name.startsWith("concept_")) return;
      var aliases = findChildValue(item, "aliases");
      var contexts = findChildValue(item, "contexts");
      var contextLinks = findChildValue(item, "context_links");
      var localized = findChildren(item, "localized").map(function (loc) {
        var locAliases = findChildValue(loc, "aliases");
        return {
          language: loc.id,
          term: findChildValue(loc, "term"),
          aliases: splitRefList(locAliases),
          summary: findChildValue(loc, "summary"),
          source: findChildValue(loc, "source"),
          sourceKind: findChildValue(loc, "source_kind"),
        };
      });
      concepts.push({
        slug: item.name,
        term: findChildValue(item, "term"),
        category: findChildValue(item, "category") || "concept",
        summary: findChildValue(item, "summary"),
        source: findChildValue(item, "source"),
        sourceKind: findChildValue(item, "source_kind") || "project-docs",
        wikidata: findChildValue(item, "wikidata"),
        aliases: splitRefList(aliases),
        contexts: splitRefList(contexts),
        contextLinks: splitRefList(contextLinks),
        localized: localized,
      });
    };
    if (root.name && root.name.startsWith("concept_")) {
      iterate(root);
    } else if (Array.isArray(root.children)) {
      root.children.forEach(iterate);
    }
    return concepts;
  }

  // Extract disambiguating context records (`concept-contexts.lino`).
  // Each context is anchored by a Wikidata Q-ID and carries per-language
  // localized labels plus a `|`-separated alias list (free-text phrases the
  // user might type in the four supported languages).
  function extractConceptContexts(root) {
    if (!root || !Array.isArray(root.children)) return [];
    var out = [];
    var visit = function (parent) {
      var entries = findChildren(parent, "context");
      for (var i = 0; i < entries.length; i += 1) {
        var entry = entries[i];
        if (!entry.id) continue;
        var aliases = findChildValue(entry, "aliases");
        var labels = findChildren(entry, "label").map(function (label) {
          return { language: label.id, text: findChildValue(label, "text") };
        });
        out.push({
          slug: entry.id,
          wikidata: findChildValue(entry, "wikidata"),
          aliases: splitRefList(aliases),
          labels: labels,
        });
      }
    };
    visit(root);
    for (var i = 0; i < root.children.length; i += 1) {
      visit(root.children[i]);
    }
    return out;
  }

  function extractFacts(root) {
    if (!root || !Array.isArray(root.children)) return [];
    var facts = [];
    var visit = function (item) {
      if (!item || !item.name || item.name.indexOf("fact_") !== 0) return;
      var localized = findChildren(item, "localized").map(function (loc) {
        return {
          language: loc.id,
          subjectLabel: findChildValue(loc, "subject_label"),
          valueLabel: findChildValue(loc, "value_label"),
          summary: findChildValue(loc, "summary"),
          source: findChildValue(loc, "source"),
          sourceKind: findChildValue(loc, "source_kind"),
        };
      });
      facts.push({
        slug: item.name,
        intent: findChildValue(item, "intent") || "fact_lookup",
        category: findChildValue(item, "category"),
        wikidata: splitList(findChildValue(item, "wikidata")),
        relation: findChildValue(item, "relation"),
        subjectQid: findChildValue(item, "subject_qid"),
        valueQid: findChildValue(item, "value_qid"),
        subjectLabel: findChildValue(item, "subject_label"),
        valueLabel: findChildValue(item, "value_label"),
        subjectAliases: splitList(findChildValue(item, "subject_aliases")).map(toLower),
        questionKeywords: splitList(findChildValue(item, "question_keywords")).map(toLower),
        summary: findChildValue(item, "summary"),
        source: findChildValue(item, "source"),
        sourceKind: findChildValue(item, "source_kind"),
        localized: localized,
      });
    };
    if (root.name && root.name.indexOf("fact_") === 0) {
      visit(root);
    } else {
      root.children.forEach(visit);
    }
    return facts;
  }

  function extractProjectStatement(node) {
    if (!node || node.name !== "statement" || !node.id) return null;
    return {
      text: String(node.id || "").trim(),
      kind: findChildValue(node, "kind"),
      weight: parseInt(findChildValue(node, "weight"), 10) || 50,
    };
  }

  function normalizeProjectAlias(value) {
    return String(value || "")
      .toLowerCase()
      .replace(/[-_]+/g, " ")
      .replace(/\s+/g, " ")
      .trim();
  }

  function extractProjects(root) {
    if (!root || !Array.isArray(root.children)) return [];
    var projects = [];
    var visit = function (item) {
      if (!item || !item.name || item.name.indexOf("project_") !== 0) return;
      var localized = findChildren(item, "localized").map(function (loc) {
        return {
          language: loc.id,
          displayName: findChildValue(loc, "display_name"),
          statements: findChildren(loc, "statement")
            .map(extractProjectStatement)
            .filter(Boolean),
        };
      });
      projects.push({
        slug: item.name,
        org: findChildValue(item, "org"),
        name: findChildValue(item, "name"),
        displayName: findChildValue(item, "display_name"),
        url: findChildValue(item, "url"),
        language: findChildValue(item, "language"),
        category: findChildValue(item, "category"),
        aliases: splitList(findChildValue(item, "aliases")).map(normalizeProjectAlias),
        topic: findChildValue(item, "topic"),
        statements: findChildren(item, "statement")
          .map(extractProjectStatement)
          .filter(Boolean),
        localized: localized,
      });
    };
    if (root.name && root.name.indexOf("project_") === 0) {
      visit(root);
    } else {
      root.children.forEach(visit);
    }
    return projects;
  }

  function extractBrainstormSeeds(root) {
    var seeds = { triggers: [], categories: [] };
    if (!root || !Array.isArray(root.children)) return seeds;
    var section = root.name === "brainstorm_seeds" ? root : findChildren(root, "brainstorm_seeds")[0];
    if (!section) return seeds;
    seeds.triggers = splitList(findChildValue(section, "trigger")).map(toLower);
    var categories = findChildren(section, "category");
    for (var i = 0; i < categories.length; i += 1) {
      var category = categories[i];
      var items = findChildren(category, "item")
        .map(function (item) {
          return item.id || "";
        })
        .filter(Boolean);
      if (!items.length) continue;
      seeds.categories.push({
        slug: category.id,
        intent: findChildValue(category, "intent"),
        detectionKeywords: splitList(findChildValue(category, "detection_keywords")).map(toLower),
        items: items,
      });
    }
    return seeds;
  }

  function extractPersonas(root) {
    var seeds = {
      triggers: [],
      defaultPersona: "",
      bodyTemplate: "",
      fallbackBody: "",
      personas: [],
      topics: [],
    };
    if (!root || !Array.isArray(root.children)) return seeds;
    var section = root.name === "personas" ? root : findChildren(root, "personas")[0];
    if (!section) return seeds;
    seeds.triggers = splitList(findChildValue(section, "trigger")).map(toLower);
    seeds.defaultPersona = findChildValue(section, "default_persona");
    seeds.bodyTemplate = findChildValue(section, "body_template");
    seeds.fallbackBody = findChildValue(section, "fallback_body");
    var personaEntries = findChildren(section, "persona");
    for (var i = 0; i < personaEntries.length; i += 1) {
      var persona = personaEntries[i];
      seeds.personas.push({
        displayName: persona.id,
        aliases: splitList(findChildValue(persona, "aliases")).map(toLower),
        wikidata: findChildValue(persona, "wikidata"),
      });
    }
    var topicEntries = findChildren(section, "topic");
    for (var j = 0; j < topicEntries.length; j += 1) {
      var topic = topicEntries[j];
      seeds.topics.push({
        slug: topic.id,
        detectionKeywords: splitList(findChildValue(topic, "detection_keywords")).map(toLower),
        body: findChildValue(topic, "body"),
      });
    }
    return seeds;
  }

  function extractCoreferenceSeeds(root) {
    var seeds = { pronouns: [], antecedents: [] };
    if (!root || !Array.isArray(root.children)) return seeds;
    var section = root.name === "coreference" ? root : findChildren(root, "coreference")[0];
    if (!section) return seeds;

    var pronouns = findChildren(section, "pronoun");
    for (var i = 0; i < pronouns.length; i += 1) {
      var pronoun = pronouns[i];
      if (!pronoun || !pronoun.id) continue;
      seeds.pronouns.push({
        token: toLower(pronoun.id),
        contexts: findChildren(pronoun, "context").map(function (context) {
          return toLower(context.id);
        }).filter(Boolean),
        startsWith: findChildren(pronoun, "starts_with").map(function (prefix) {
          return toLower(prefix.id);
        }).filter(Boolean),
      });
    }

    var antecedents = findChildren(section, "antecedent");
    for (var j = 0; j < antecedents.length; j += 1) {
      var antecedent = antecedents[j];
      if (!antecedent || !antecedent.id) continue;
      seeds.antecedents.push({
        displayName: antecedent.id,
        aliases: findChildren(antecedent, "alias").map(function (alias) {
          return toLower(alias.id);
        }).filter(Boolean),
        wikidata: findChildValue(antecedent, "wikidata"),
        intent: findChildValue(antecedent, "intent"),
        body: findChildValue(antecedent, "body"),
      });
    }

    return seeds;
  }

  function extractTools(node) {
    var tools = [];
    if (!node) return tools;
    var entries = findChildren(node, "tool");
    for (var i = 0; i < entries.length; i += 1) {
      var entry = entries[i];
      var localized = findChildren(entry, "localized").map(function (loc) {
        return {
          language: loc.id,
          name: findChildValue(loc, "name"),
          description: findChildValueAlias(loc, "note", "description"),
        };
      });
      tools.push({
        id: entry.id,
        name: findChildValue(entry, "name"),
        description: findChildValueAlias(entry, "note", "description"),
        mode: findChildValue(entry, "mode") || "thinking",
        inputs: splitList(findChildValue(entry, "inputs")),
        outputs: splitList(findChildValue(entry, "outputs")),
        isolation: findChildValue(entry, "isolation"),
        sources: splitList(findChildValue(entry, "sources")),
        localized: localized,
      });
    }
    return tools;
  }

  function extractAgentInfo(node) {
    var info = {};
    if (!node) return info;
    var entries = findChildren(node, "field");
    for (var i = 0; i < entries.length; i += 1) {
      var entry = entries[i];
      var key = entry.id;
      var value = findChildValue(entry, "value");
      if (key) info[key] = value;
    }
    return info;
  }

  function extractLanguageRules(node) {
    var rules = [];
    if (!node) return rules;
    var entries = findChildren(node, "rule");
    for (var i = 0; i < entries.length; i += 1) {
      var entry = entries[i];
      rules.push({
        id: entry.id,
        language: findChildValue(entry, "language"),
        label: findChildValue(entry, "label"),
        start: parseCodepoint(findChildValue(entry, "start")),
        end: parseCodepoint(findChildValue(entry, "end")),
        note: findChildValue(entry, "note"),
      });
    }
    return rules;
  }

  function parseCodepoint(value) {
    if (!value) return 0;
    var str = String(value).trim();
    if (str.indexOf("0x") === 0 || str.indexOf("0X") === 0) {
      return parseInt(str.slice(2), 16) || 0;
    }
    return parseInt(str, 10) || 0;
  }

  // Extract the environment directory (`environments.lino`) so the demo can
  // show every supported interface and how to migrate memory between them.
  // Mirrors `src/seed.rs::environment_directory` so the two surfaces always
  // agree on the schema.
  function extractEnvironmentDirectory(root) {
    var directory = {
      environments: [],
      migrationDescription: "",
      flows: [],
    };
    if (!root || !Array.isArray(root.children)) return directory;
    for (var i = 0; i < root.children.length; i += 1) {
      var section = root.children[i];
      if (!section || !section.name) continue;
      if (section.name === "environments") {
        var envEntries = findChildren(section, "environment");
        for (var j = 0; j < envEntries.length; j += 1) {
          var entry = envEntries[j];
          directory.environments.push({
            id: entry.id,
            label: findChildValue(entry, "label"),
            runtime: findChildValue(entry, "runtime"),
            seedPath: findChildValue(entry, "seed_path"),
            memoryStore: findChildValue(entry, "memory_store"),
            memoryExport: findChildValue(entry, "memory_export_command"),
            bundleExport: findChildValue(entry, "bundle_export_command"),
            bundleImport: findChildValue(entry, "bundle_import_command"),
            tools: splitList(findChildValue(entry, "tools")),
          });
        }
      } else if (section.name === "migration") {
        directory.migrationDescription = findChildValueAlias(section, "note", "description");
        var flowEntries = findChildren(section, "flow");
        for (var k = 0; k < flowEntries.length; k += 1) {
          var flow = flowEntries[k];
          directory.flows.push({
            id: flow.id,
            description: findChildValueAlias(flow, "note", "description"),
            fileFormat: findChildValue(flow, "file_format"),
          });
        }
      }
    }
    return directory;
  }

  function extractPromptPatterns(node) {
    var patterns = [];
    if (!node) return patterns;
    var entries = findChildren(node, "pattern");
    for (var i = 0; i < entries.length; i += 1) {
      var entry = entries[i];
      patterns.push({
        id: entry.id,
        intent: findChildValue(entry, "intent"),
        language: findChildValue(entry, "language") || "en",
        kind: findChildValue(entry, "kind"),
        text: findChildValue(entry, "text"),
      });
    }
    return patterns;
  }

  // Extract the intent routing table (`intent-routing.lino`) used by the
  // worker to decide between greeting, identity, hello-world and unknown
  // intents in a fully data-driven way. The schema is mirrored from
  // `src/seed.rs::IntentRoute`.
  function extractIntentRouting(node) {
    var routing = {
      intents: [],
      articlePrefixes: [],
      tracePrefixes: [],
    };
    if (!node || !Array.isArray(node.children)) return routing;
    for (var i = 0; i < node.children.length; i += 1) {
      var child = node.children[i];
      if (!child || !child.name) continue;
      if (child.name === "intent") {
        var route = {
          id: child.id,
          slug: findChildValue(child, "slug"),
          responseLink: findChildValue(child, "response_link"),
          keywords: [],
          phrases: [],
          tokens: [],
          combos: [],
        };
        for (var j = 0; j < child.children.length; j += 1) {
          var entry = child.children[j];
          if (entry.name === "keyword") route.keywords.push(entry.id);
          else if (entry.name === "phrase") route.phrases.push(entry.id);
          else if (entry.name === "token") route.tokens.push(entry.id);
          else if (entry.name === "combo") {
            route.combos.push(
              String(entry.id || "")
                .split("+")
                .map(trim)
                .filter(Boolean),
            );
          }
        }
        routing.intents.push(route);
      } else if (child.name === "article") {
        routing.articlePrefixes.push(child.id);
      } else if (child.name === "trace_prefix") {
        routing.tracePrefixes.push(child.id);
      }
    }
    return routing;
  }

  // Split a canonical `(a "b c" d)` reference list into its items. Each item is
  // a quoted scalar (which may contain spaces) or a bare whitespace-delimited
  // token. Falls back to the legacy `a|b|c` pipe packing for any value that is
  // not wrapped in parentheses, so old and migrated seeds both load.
  function splitRefList(value) {
    var raw = trim(value);
    if (!raw) return [];
    if (raw.charAt(0) === "(" && raw.charAt(raw.length - 1) === ")") {
      return tokenizeRefList(raw.slice(1, -1));
    }
    return raw.split("|").map(trim).filter(Boolean);
  }

  function tokenizeRefList(body) {
    var tokens = [];
    var i = 0;
    while (i < body.length) {
      var ch = body.charAt(i);
      if (/\s/.test(ch)) {
        i += 1;
        continue;
      }
      if (ch === '"' || ch === "'" || ch === "`") {
        var quote = ch;
        i += 1;
        var value = "";
        while (i < body.length) {
          var c = body.charAt(i);
          if ((quote === '"' || quote === "`") && c === "\\") {
            value += body.charAt(i + 1) || "";
            i += 2;
            continue;
          }
          if (c === quote) {
            i += 1;
            break;
          }
          value += c;
          i += 1;
        }
        tokens.push(value);
      } else {
        var bare = "";
        while (i < body.length && !/\s/.test(body.charAt(i))) {
          bare += body.charAt(i);
          i += 1;
        }
        if (bare) tokens.push(bare);
      }
    }
    return tokens;
  }

  // Backwards-compatible alias retained for existing call sites.
  function splitList(value) {
    return splitRefList(value);
  }

  function trim(value) {
    return String(value || "").trim();
  }

  function toLower(value) {
    return trim(value).toLowerCase();
  }

  function fetchText(url) {
    if (typeof global.fetch !== "function") {
      return Promise.resolve("");
    }
    return global.fetch(url).then(function (response) {
      if (!response || !response.ok) return "";
      return response.text();
    }, function () {
      return "";
    });
  }

  function loadAll(files) {
    var target = Array.isArray(files) && files.length ? files : DEFAULT_FILES;
    return Promise.all(
      target.map(function (file) {
        return fetchText(withAssetVersion(file)).then(function (text) {
          return { file: file, text: text };
        });
      }),
    ).then(buildSeed);
  }

  function mergeResponses(a, b) {
    var out = {};
    var keys = Object.keys(a).concat(Object.keys(b || {}));
    for (var i = 0; i < keys.length; i += 1) {
      var key = keys[i];
      out[key] = Object.assign({}, a[key] || {}, (b || {})[key] || {});
    }
    return out;
  }

  // Parse a single merged bundle (the `formal_ai_seed_bundle` document
  // produced by `seed::merged_bundle()` on the Rust side) back into a list
  // of `{ file, text }` pairs. Indentation contract:
  //   formal_ai_seed_bundle             (top-level, indent 0)
  //     file "data/seed/X.lino"         (indent 2)
  //       <line of X.lino>              (indent 4)
  //       <line of X.lino>
  //     file "data/seed/Y.lino"
  //       ...
  function parseBundle(text) {
    var lines = String(text || "").split(/\r?\n/);
    var sections = [];
    var current = null;
    for (var i = 0; i < lines.length; i += 1) {
      var line = lines[i];
      if (line === "") {
        if (current) current.text += "\n";
        continue;
      }
      var indentMatch = /^(\s*)/.exec(line);
      var indent = indentMatch ? indentMatch[1].length : 0;
      var trimmed = line.slice(indent);
      if (indent === 0) {
        // New top-level header — flush any open section.
        if (current) sections.push(current);
        current = null;
        continue;
      }
      if (indent === 2 && trimmed.indexOf("file ") === 0) {
        if (current) sections.push(current);
        current = null;
        var rest = trimmed.slice("file ".length).trim();
        if (rest[0] === '"') {
          var inner = rest.slice(1);
          // Reuse the same closing-quote scan as `parseLinoLine`.
          for (var j = 0; j < inner.length; j += 1) {
            if (inner[j] === "\\") {
              j += 1;
              continue;
            }
            if (inner[j] === '"') {
              current = { file: unescapeValue(inner.slice(0, j)), text: "" };
              break;
            }
          }
        }
        continue;
      }
      if (current) {
        var body = line.indexOf("    ") === 0 ? line.slice(4) : trimmed;
        current.text += body + "\n";
      }
    }
    if (current) sections.push(current);
    return sections;
  }

  // Build a seed object from a pre-fetched merged bundle. Mirrors `loadAll`
  // but skips the network step, so a worker booted offline (or one given a
  // user-uploaded bundle) can hydrate without touching `fetch`.
  function loadFromBundle(text) {
    var sections = parseBundle(text);
    return buildSeed(sections);
  }

  function buildSeed(results) {
    var seed = {
      responses: {},
      concepts: [],
      conceptContexts: [],
      facts: [],
      projects: [],
      brainstormSeeds: { triggers: [], categories: [] },
      personas: {
        triggers: [],
        defaultPersona: "",
        bodyTemplate: "",
        fallbackBody: "",
        personas: [],
        topics: [],
      },
      coreferenceSeeds: { pronouns: [], antecedents: [] },
      tools: [],
      agentInfo: {},
      languageRules: [],
      promptPatterns: [],
      intentRouting: { intents: [], articlePrefixes: [], tracePrefixes: [] },
      environments: { environments: [], migrationDescription: "", flows: [] },
      raw: {},
    };
    for (var i = 0; i < results.length; i += 1) {
      var item = results[i];
      seed.raw[item.file] = item.text;
      if (!item.text) continue;
      var root = parseLino(item.text);
      if (item.file.indexOf("multilingual") !== -1) {
        seed.responses = mergeResponses(
          seed.responses,
          extractMultilingualResponses(root),
        );
      } else if (item.file.indexOf("facts") !== -1) {
        seed.facts = seed.facts.concat(extractFacts(root));
      } else if (item.file.indexOf("projects") !== -1) {
        seed.projects = seed.projects.concat(extractProjects(root));
      } else if (item.file.indexOf("brainstorm-seeds") !== -1) {
        seed.brainstormSeeds = extractBrainstormSeeds(root);
      } else if (item.file.indexOf("personas") !== -1) {
        seed.personas = extractPersonas(root);
      } else if (item.file.indexOf("coreference") !== -1) {
        seed.coreferenceSeeds = extractCoreferenceSeeds(root);
      } else if (item.file.indexOf("concept-contexts") !== -1) {
        seed.conceptContexts = seed.conceptContexts.concat(
          extractConceptContexts(root),
        );
      } else if (item.file.indexOf("concepts") !== -1) {
        seed.concepts = seed.concepts.concat(extractConcepts(root));
      } else if (item.file.indexOf("tools") !== -1) {
        seed.tools = seed.tools.concat(extractTools(root));
      } else if (item.file.indexOf("agent-info") !== -1) {
        Object.assign(seed.agentInfo, extractAgentInfo(root));
      } else if (item.file.indexOf("language-detection") !== -1) {
        seed.languageRules = seed.languageRules.concat(extractLanguageRules(root));
      } else if (item.file.indexOf("prompt-patterns") !== -1) {
        seed.promptPatterns = seed.promptPatterns.concat(extractPromptPatterns(root));
      } else if (item.file.indexOf("intent-routing") !== -1) {
        seed.intentRouting = extractIntentRouting(root);
      } else if (item.file.indexOf("environments") !== -1) {
        seed.environments = extractEnvironmentDirectory(root);
      }
    }
    return seed;
  }

  global.FormalAiSeed = {
    parse: parseLino,
    loadAll: loadAll,
    loadFromBundle: loadFromBundle,
    parseBundle: parseBundle,
    extractMultilingualResponses: extractMultilingualResponses,
    extractAgentInfo: extractAgentInfo,
    extractLanguageRules: extractLanguageRules,
    extractPromptPatterns: extractPromptPatterns,
    extractConcepts: extractConcepts,
    extractConceptContexts: extractConceptContexts,
    extractFacts: extractFacts,
    extractProjects: extractProjects,
    extractBrainstormSeeds: extractBrainstormSeeds,
    extractPersonas: extractPersonas,
    extractCoreferenceSeeds: extractCoreferenceSeeds,
    extractTools: extractTools,
    extractIntentRouting: extractIntentRouting,
    extractEnvironmentDirectory: extractEnvironmentDirectory,
    DEFAULT_FILES: DEFAULT_FILES,
    isWorker: isWorker(),
  };
})(typeof self !== "undefined" ? self : globalThis);
