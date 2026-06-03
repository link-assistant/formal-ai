// Issue #386 — one-shot: the behavior-rules seed file was authored with
// backslash-escaped inner double-quotes (\"…\"), a JSON/Rust convention the
// Links Notation multi-quote parser does NOT understand (it escapes an inner
// double-quote by DOUBLING it, never with a backslash). Every other seed file
// avoids inner double-quotes and uses apostrophes inside the double-quoted
// description/gloss strings, which parse cleanly. This rewrites the inner
// \"…\" spans to '…' to match that house style and fix the parse error caught
// by tests/unit/data_files.rs. Re-run is a no-op once the file is clean.

import fs from "node:fs";

const path = new URL("../data/seed/meanings-behavior-rules.lino", import.meta.url);
let text = fs.readFileSync(path, "utf8");

const before = (text.match(/\\"/g) || []).length;
// backslash + double-quote  ->  apostrophe
text = text.split('\\"').join("'");
const after = (text.match(/\\"/g) || []).length;

fs.writeFileSync(path, text);
console.log(`replaced backslash-quote sequences: ${before} -> remaining ${after}`);
