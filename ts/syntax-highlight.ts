// Dependency-free syntax highlighter for the formal-ai chat UI (issue #330).
//
// The chat renders assistant answers through marked + DOMPurify and injects the
// resulting HTML into a `.markdown-body` container. Code fences become
// `<pre><code class="language-xxx">…</code></pre>`. This module turns that
// escaped plain text into token spans so the browser demo shows real syntax
// highlighting without pulling a heavyweight dependency into the committed
// `vendor.bundle.js` artifact.
//
// Design notes:
//   * It is a plain `<script>` (like preferences.js / memory.js / i18n.js), so
//     it never touches the bun-built vendor bundle and stays trivially testable.
//   * Token class names use the `hljs-*` convention so the CSS theme keeps
//     working if the project ever swaps in highlight.js proper.
//   * The tokenizer is the sole producer of markup here, and every literal slice
//     of source is HTML-escaped before it reaches the DOM, so there is no XSS
//     surface even when the source contains `<script>` text.
(function attachSyntaxHighlighter(global) {
  "use strict";

  function escapeHtml(value) {
    return String(value)
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#039;");
  }

  // Shared keyword vocabularies. Kept compact but covering every language the
  // `write_program` seed emits (rust, python, javascript, typescript, go, c,
  // cpp, java, csharp, ruby) plus a few common companions (bash, json, css).
  const C_FAMILY_TYPES = [
    "int",
    "long",
    "short",
    "char",
    "float",
    "double",
    "void",
    "bool",
    "unsigned",
    "signed",
    "size_t",
    "auto",
    "const",
  ];

  const LANGUAGES = {
    rust: {
      lineComment: ["//"],
      blockComment: ["/*", "*/"],
      keywords: [
        "as", "async", "await", "break", "const", "continue", "crate", "dyn",
        "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
        "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
        "self", "Self", "static", "struct", "super", "trait", "true", "type",
        "unsafe", "use", "where", "while",
      ],
      types: [
        "Vec", "String", "str", "Option", "Result", "Box", "i8", "i16", "i32",
        "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "bool", "char", "Ok", "Err", "Some", "None",
      ],
    },
    python: {
      lineComment: ["#"],
      tripleStrings: ['"""', "'''"],
      keywords: [
        "and", "as", "assert", "async", "await", "break", "class", "continue",
        "def", "del", "elif", "else", "except", "finally", "for", "from",
        "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or",
        "pass", "raise", "return", "try", "while", "with", "yield",
      ],
      literals: ["True", "False", "None"],
      types: [
        "print", "len", "range", "str", "int", "float", "list", "dict", "set",
        "tuple", "bool", "sorted", "open", "enumerate", "zip", "map", "filter",
      ],
    },
    javascript: {
      lineComment: ["//"],
      blockComment: ["/*", "*/"],
      template: true,
      keywords: [
        "async", "await", "break", "case", "catch", "class", "const",
        "continue", "debugger", "default", "delete", "do", "else", "export",
        "extends", "finally", "for", "function", "if", "import", "in",
        "instanceof", "let", "new", "of", "return", "super", "switch", "this",
        "throw", "try", "typeof", "var", "void", "while", "with", "yield",
      ],
      literals: ["true", "false", "null", "undefined", "NaN", "Infinity"],
      types: [
        "console", "require", "module", "process", "Math", "JSON", "Object",
        "Array", "Promise", "Map", "Set", "Number", "String", "Boolean",
      ],
    },
    go: {
      lineComment: ["//"],
      blockComment: ["/*", "*/"],
      template: true,
      keywords: [
        "break", "case", "chan", "const", "continue", "default", "defer",
        "else", "fallthrough", "for", "func", "go", "goto", "if", "import",
        "interface", "map", "package", "range", "return", "select", "struct",
        "switch", "type", "var",
      ],
      literals: ["true", "false", "nil", "iota"],
      types: [
        "string", "int", "int32", "int64", "float64", "bool", "byte", "rune",
        "error", "fmt", "os", "sort", "len", "append", "make", "panic",
      ],
    },
    c: {
      lineComment: ["//"],
      blockComment: ["/*", "*/"],
      keywords: [
        "break", "case", "continue", "default", "do", "else", "enum", "extern",
        "for", "goto", "if", "return", "sizeof", "static", "struct", "switch",
        "typedef", "union", "volatile", "while", "include", "define",
      ],
      types: C_FAMILY_TYPES.concat(["FILE", "DIR", "struct"]),
      literals: ["NULL"],
    },
    cpp: {
      lineComment: ["//"],
      blockComment: ["/*", "*/"],
      keywords: [
        "break", "case", "catch", "class", "const", "continue", "default",
        "delete", "do", "else", "enum", "explicit", "export", "extern", "for",
        "friend", "goto", "if", "inline", "namespace", "new", "operator",
        "private", "protected", "public", "return", "sizeof", "static",
        "struct", "switch", "template", "this", "throw", "try", "typedef",
        "typename", "union", "using", "virtual", "volatile", "while", "include",
      ],
      types: C_FAMILY_TYPES.concat([
        "std", "string", "vector", "namespace", "fs",
      ]),
      literals: ["nullptr", "true", "false", "NULL"],
    },
    java: {
      lineComment: ["//"],
      blockComment: ["/*", "*/"],
      keywords: [
        "abstract", "assert", "break", "case", "catch", "class", "const",
        "continue", "default", "do", "else", "enum", "extends", "final",
        "finally", "for", "goto", "if", "implements", "import", "instanceof",
        "interface", "native", "new", "package", "private", "protected",
        "public", "return", "static", "strictfp", "super", "switch",
        "synchronized", "this", "throw", "throws", "transient", "try", "void",
        "volatile", "while",
      ],
      types: [
        "int", "long", "short", "byte", "char", "float", "double", "boolean",
        "String", "Object", "System", "Integer", "File", "Arrays", "List",
      ],
      literals: ["true", "false", "null"],
    },
    csharp: {
      lineComment: ["//"],
      blockComment: ["/*", "*/"],
      template: true,
      keywords: [
        "abstract", "as", "base", "break", "case", "catch", "class", "const",
        "continue", "default", "delegate", "do", "else", "enum", "event",
        "explicit", "extern", "finally", "fixed", "for", "foreach", "goto",
        "if", "implicit", "in", "interface", "internal", "is", "lock",
        "namespace", "new", "operator", "out", "override", "params", "private",
        "protected", "public", "readonly", "ref", "return", "sealed", "sizeof",
        "static", "struct", "switch", "this", "throw", "try", "typeof", "using",
        "var", "virtual", "void", "while",
      ],
      types: [
        "int", "long", "short", "byte", "char", "float", "double", "bool",
        "string", "object", "Console", "System", "Directory", "Path",
      ],
      literals: ["true", "false", "null"],
    },
    ruby: {
      lineComment: ["#"],
      keywords: [
        "alias", "and", "begin", "break", "case", "class", "def", "defined?",
        "do", "else", "elsif", "end", "ensure", "for", "if", "in", "module",
        "next", "not", "or", "redo", "rescue", "retry", "return", "self",
        "super", "then", "undef", "unless", "until", "when", "while", "yield",
      ],
      literals: ["true", "false", "nil"],
      types: ["puts", "print", "require", "Dir", "File", "Array", "Hash"],
    },
    bash: {
      lineComment: ["#"],
      keywords: [
        "if", "then", "else", "elif", "fi", "for", "while", "do", "done",
        "case", "esac", "function", "in", "return", "export", "local",
      ],
      types: ["echo", "cd", "cargo", "rustc", "node", "python3", "go", "npm"],
    },
    json: {
      keywords: [],
      literals: ["true", "false", "null"],
    },
  };

  // Aliases that map common fence labels onto a defined grammar.
  const ALIASES = {
    rs: "rust",
    py: "python",
    js: "javascript",
    jsx: "javascript",
    ts: "javascript",
    tsx: "javascript",
    typescript: "javascript",
    golang: "go",
    "c++": "cpp",
    cc: "cpp",
    h: "c",
    hpp: "cpp",
    cs: "csharp",
    rb: "ruby",
    sh: "bash",
    shell: "bash",
    zsh: "bash",
    console: "bash",
  };

  function resolveLanguage(name) {
    const key = String(name || "").trim().toLowerCase();
    if (!key) return null;
    if (LANGUAGES[key]) return key;
    if (ALIASES[key] && LANGUAGES[ALIASES[key]]) return ALIASES[key];
    return null;
  }

  function isIdentStart(ch) {
    return /[A-Za-z_$]/.test(ch);
  }

  function isIdentPart(ch) {
    return /[A-Za-z0-9_$]/.test(ch);
  }

  function span(cls, text) {
    return `<span class="hljs-${cls}">${escapeHtml(text)}</span>`;
  }

  // Tokenize `source` against the `grammar` config and return safe HTML.
  function tokenize(source, grammar) {
    const text = String(source);
    const length = text.length;
    const keywords = new Set(grammar.keywords || []);
    const literals = new Set(grammar.literals || []);
    const types = new Set(grammar.types || []);
    const lineComments = grammar.lineComment || [];
    const blockComment = grammar.blockComment || null;
    const tripleStrings = grammar.tripleStrings || [];
    const allowTemplate = Boolean(grammar.template);

    let out = "";
    let i = 0;

    const startsWith = (token, at) => text.startsWith(token, at);

    while (i < length) {
      const ch = text[i];

      // 1. Whitespace passes through verbatim (already-safe characters).
      if (ch === " " || ch === "\t" || ch === "\n" || ch === "\r") {
        out += ch;
        i += 1;
        continue;
      }

      // 2. Line comments.
      let matchedLineComment = false;
      for (const marker of lineComments) {
        if (startsWith(marker, i)) {
          let end = text.indexOf("\n", i);
          if (end === -1) end = length;
          out += span("comment", text.slice(i, end));
          i = end;
          matchedLineComment = true;
          break;
        }
      }
      if (matchedLineComment) continue;

      // 3. Block comments.
      if (blockComment && startsWith(blockComment[0], i)) {
        let end = text.indexOf(blockComment[1], i + blockComment[0].length);
        end = end === -1 ? length : end + blockComment[1].length;
        out += span("comment", text.slice(i, end));
        i = end;
        continue;
      }

      // 4. Triple-quoted strings (Python docstrings).
      let matchedTriple = false;
      for (const quote of tripleStrings) {
        if (startsWith(quote, i)) {
          let end = text.indexOf(quote, i + quote.length);
          end = end === -1 ? length : end + quote.length;
          out += span("string", text.slice(i, end));
          i = end;
          matchedTriple = true;
          break;
        }
      }
      if (matchedTriple) continue;

      // 5. Strings: double, single, and (when enabled) template literals.
      if (ch === '"' || ch === "'" || (allowTemplate && ch === "`")) {
        const quote = ch;
        let j = i + 1;
        while (j < length) {
          if (text[j] === "\\") {
            j += 2;
            continue;
          }
          if (text[j] === quote) {
            j += 1;
            break;
          }
          // Unterminated single-quote spans (e.g. Rust lifetimes `'a`) stop at
          // whitespace so we never swallow the rest of the line.
          if (quote === "'" && (text[j] === "\n")) {
            break;
          }
          j += 1;
        }
        out += span("string", text.slice(i, j));
        i = j;
        continue;
      }

      // 6. Numbers (decimal, hex, float, with common suffixes).
      if (/[0-9]/.test(ch) || (ch === "." && /[0-9]/.test(text[i + 1] || ""))) {
        let j = i;
        while (j < length && /[0-9a-fA-FxXoObB._]/.test(text[j])) {
          j += 1;
        }
        out += span("number", text.slice(i, j));
        i = j;
        continue;
      }

      // 7. Identifiers / keywords / types / functions.
      if (isIdentStart(ch)) {
        let j = i;
        while (j < length && isIdentPart(text[j])) {
          j += 1;
        }
        const word = text.slice(i, j);
        // Determine whether the identifier is immediately followed by `(`.
        let k = j;
        while (k < length && (text[k] === " " || text[k] === "\t")) k += 1;
        const isCall = text[k] === "(";

        if (keywords.has(word)) {
          out += span("keyword", word);
        } else if (literals.has(word)) {
          out += span("literal", word);
        } else if (types.has(word)) {
          out += span("type", word);
        } else if (isCall) {
          out += span("title", word);
        } else {
          out += escapeHtml(word);
        }
        i = j;
        continue;
      }

      // 8. Everything else (operators, punctuation) — escaped, unstyled.
      out += escapeHtml(ch);
      i += 1;
    }

    return out;
  }

  // Public entry point. Returns `{ value, language }` where `value` is safe HTML
  // and `language` is the resolved grammar key (or null when unsupported, in
  // which case `value` is the escaped source with no token spans).
  function highlight(source, requestedLanguage) {
    const language = resolveLanguage(requestedLanguage);
    if (!language) {
      return { value: escapeHtml(String(source ?? "")), language: null };
    }
    return { value: tokenize(String(source ?? ""), LANGUAGES[language]), language };
  }

  function listLanguages() {
    return Object.keys(LANGUAGES).sort();
  }

  global.FormalAiHighlight = {
    highlight,
    resolveLanguage,
    listLanguages,
    escapeHtml,
  };
})(typeof window !== "undefined" ? window : globalThis);
