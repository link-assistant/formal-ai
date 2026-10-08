### Changed

- PR #1188 round 15, Formal AI as a subagent working on its own requirements:
  - **No local Rust builds.** The gate runner `experiments/formal_ai_subagent/local-gates.mjs` now runs a script's JavaScript twin (`scripts/<name>.mjs`) in place of `rust-script scripts/<name>.rs`, and leaves scripts without a twin to CI. Twins with CI parity gates now cover:
    - the requirement pipeline: assemble, generate status, render, check;
    - file size, the debt ratchet, hardcoded language, the minimal-core boundary, worker line budgets and the changelog fragment check.
  - **The response-language debt is measured over every `data/seed/multilingual-responses*.lino`,** not one file. Moving an intent between files can no longer hide it. The honest ceiling is 436, and `scripts/generate-response-parity-debt.mjs` writes the ledger. Code templates (`language rust`) owe no translation.
  - **Edit composer (TEACH-F), fixed in JS and the Rust twin:**
    - unpaired quotes are declined before any route reads the payload (G71, unsafe);
    - contractions are expanded in both roots (G70);
    - an unquoted addition asks a question instead of writing (G69);
    - a whole-line anchor is widened until it is unique (G73), and a repeated anchor is reported with its count (G72);
    - "followed by an empty line" writes that line (G74);
    - a listing is detected by its command word (G75);
    - a Replace whose new text contains the old text is applied in one pass (G78);
    - a scoped Replace edits only the named part, or declines (G79);
    - a list of lines is replaced as one block (G80);
    - unquoted anchors (G51), assertions in the file's own style (G13) and stated-expectation tests (G25) work;
    - a backticked command followed by prose runs only the command (G76, unsafe);
    - the timeout report is honest (G77);
    - line moves work (G81);
    - multi-sentence copy-and-replace requests are planned sentence by sentence (G82);
    - words inside file names are not request words (G83).

    The planner arms are back to 58.
  - **Routing (ROUTE3):**
    - "a program that prints <text>" is answered by the documented hello_world procedure with the operand. No program text is stored.
    - Native recurrence requests read the Wikifunctions recurrence cache offline.
    - GitHub traffic is asked before promotion.
    - Both misroute ceilings are 0.
  - **Handlers (MIGRATE4):** `diagnostic`, `coreference`, `verifiable_task`, `nl_tool`, `installation_conversion` and `translation` are migrated to seed data. 57 are migrated and 14 pending. The browser gains honest twins in place of "unknown".
  - **Discovery (DISCOVER):** check and run commands come from captured documentation pages (R1165-6 implemented). No language row restates a command that a page states.
  - **CI fixes (CIFIX2):**
    - file-creation requests write without reading first;
    - an explicit grep runs as written;
    - Spanish articles are never read as directories;
    - `solver.rs` is split below its warning band;
    - the debug-session test helper is fixed.
  - **Upstream:** link-foundation/meta-language PR #216 (issues #202, #203) translates sibling items and relative imports. Measured on our tree, it raises the JS -> Rust translated items from 254 to 657.
  - Formal AI did the ledger rows, gap marks and most single-line edits. Its new failures are logged as gaps G73–G86.
