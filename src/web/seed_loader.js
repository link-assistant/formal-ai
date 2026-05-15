// Shared Links Notation seed loader for the demo.
//
// The browser demo loads its multilingual response phrases, concept summaries,
// tool registry, language-detection rules, prompt patterns, and agent metadata
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
    "seed/tools.lino",
    "seed/language-detection.lino",
    "seed/prompt-patterns.lino",
    "seed/intent-routing.lino",
    "seed/greetings.lino",
    "seed/identity.lino",
    "seed/hello-world-programs.lino",
    "seed/demo-dialogs.lino",
    "seed/environments.lino",
  ];

  function isWorker() {
    return (
      typeof global.WorkerGlobalScope !== "undefined" &&
      global instanceof global.WorkerGlobalScope
    );
  }

  function unescapeValue(value) {
    return String(value || "")
      .replace(/\\n/g, "\n")
      .replace(/\\"/g, '"')
      .replace(/\\\\/g, "\\");
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
      if (!line || /^\s*$/.test(line)) continue;
      var indentMatch = /^(\s*)/.exec(line);
      var indent = indentMatch ? indentMatch[1].length : 0;
      var content = line.slice(indent);
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
    var first = content.indexOf(' "');
    if (first === -1) {
      node.name = content.trim();
      return node;
    }
    node.name = content.slice(0, first).trim();
    var rest = content.slice(first + 1).trim();
    if (rest.length === 0 || rest[0] !== '"') return node;
    var closing = -1;
    for (var i = 1; i < rest.length; i += 1) {
      if (rest[i] === "\\") {
        i += 1;
        continue;
      }
      if (rest[i] === '"') {
        closing = i;
        break;
      }
    }
    if (closing === -1) return node;
    node.id = unescapeValue(rest.slice(1, closing));
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

  function extractMultilingualResponses(node) {
    var responses = {};
    if (!node) return responses;
    var entries = findChildren(node, "response");
    for (var i = 0; i < entries.length; i += 1) {
      var entry = entries[i];
      var intent = findChildValue(entry, "intent");
      var language = findChildValue(entry, "language");
      var text = findChildValue(entry, "text");
      if (!intent || !language || !text) continue;
      if (!responses[intent]) responses[intent] = {};
      responses[intent][language] = text;
    }
    return responses;
  }

  function extractConcepts(root) {
    if (!root || !Array.isArray(root.children)) return [];
    var nodes = root.name === "" || !root.name ? root.children : [root].concat(root.children || []);
    // Concept files are flat — each top-level child is one record.
    var concepts = [];
    var iterate = function (item) {
      if (!item || !item.name) return;
      if (!item.name.startsWith("concept_")) return;
      var aliases = findChildValue(item, "aliases");
      concepts.push({
        slug: item.name,
        term: findChildValue(item, "term"),
        category: findChildValue(item, "category") || "concept",
        summary: findChildValue(item, "summary"),
        source: findChildValue(item, "source"),
        sourceKind: findChildValue(item, "source_kind") || "project-docs",
        aliases: aliases ? aliases.split("|").map(trim).filter(Boolean) : [],
      });
    };
    if (root.name && root.name.startsWith("concept_")) {
      iterate(root);
    } else if (Array.isArray(root.children)) {
      root.children.forEach(iterate);
    }
    return concepts;
  }

  function extractTools(node) {
    var tools = [];
    if (!node) return tools;
    var entries = findChildren(node, "tool");
    for (var i = 0; i < entries.length; i += 1) {
      var entry = entries[i];
      tools.push({
        id: entry.id,
        name: findChildValue(entry, "name"),
        description: findChildValue(entry, "description"),
        mode: findChildValue(entry, "mode") || "thinking",
        inputs: splitList(findChildValue(entry, "inputs")),
        outputs: splitList(findChildValue(entry, "outputs")),
        isolation: findChildValue(entry, "isolation"),
        sources: splitList(findChildValue(entry, "sources")),
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
        directory.migrationDescription = findChildValue(section, "description");
        var flowEntries = findChildren(section, "flow");
        for (var k = 0; k < flowEntries.length; k += 1) {
          var flow = flowEntries[k];
          directory.flows.push({
            id: flow.id,
            description: findChildValue(flow, "description"),
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

  function splitList(value) {
    if (!value) return [];
    return String(value)
      .split("|")
      .map(trim)
      .filter(Boolean);
  }

  function trim(value) {
    return String(value || "").trim();
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
        return fetchText(file).then(function (text) {
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
    extractTools: extractTools,
    extractIntentRouting: extractIntentRouting,
    extractEnvironmentDirectory: extractEnvironmentDirectory,
    DEFAULT_FILES: DEFAULT_FILES,
    isWorker: isWorker(),
  };
})(typeof self !== "undefined" ? self : globalThis);
