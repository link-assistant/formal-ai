# relative-meta-logic — Rust

Rust implementation of the Relative Meta-Logic (RML) framework.

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2021)

## Building

```bash
cd rust
cargo build
```

## Usage

### Running a knowledge base

```bash
cargo run -- <file.lino>
```

Literate `.lino.md` files are also accepted by evaluator entry points. Only
fenced `lino` code blocks are evaluated; prose and other fenced languages are
ignored. The same extraction applies to files loaded through `(import "...")`.

### Exporting Lean 4

```bash
cargo run -- export lean ../examples/lean-export-basic.lino -o out.lean
```

The supported subset is documented in [`../docs/LEAN_EXPORT.md`](../docs/LEAN_EXPORT.md).

The shared examples live at the repo root in [`/examples/`](../examples/) and
both implementations are required to produce identical output for every file
there. To run one:

```bash
cargo run -- ../examples/classical-logic.lino
```

Or after building:

```bash
./target/release/rml ../examples/classical-logic.lino
```

### Exporting Rocq source

```bash
cargo run -- export rocq ../examples/dependent-types.lino -o dependent_types.v
```

See [`../docs/ROCQ-EXPORT.md`](../docs/ROCQ-EXPORT.md) for the supported
typed subset.

### Example

```lino
(a: a is a)
(!=: not =)
(and: avg)
(or: max)

((a = a) has probability 1)
((a != a) has probability 0)

(? ((a = a) and (a != a)))   # -> 0.5
(? ((a = a) or  (a != a)))   # -> 1
```

## API

```rust
use rml::{
    run, evaluate, format_diagnostic, Diagnostic, EvaluateResult, RunResult, Span,
    tokenize_one, parse_one, Env, EnvOptions, eval_node, quantize, dec_round, subst,
    run_tactics, rewrite, simplify, goal_to_tptp, parse_atp_status, ProofState,
    automatic_sequences_domain_plugin, decide_automatic_sequence_theorem,
    export_lean,
    formalize_selected_interpretation, evaluate_formalization,
    FormalizationRequest, Interpretation,
};

// Run a complete LiNo knowledge base
let results = run(lino_text, None);

// Run with custom range and valence
let results2 = run(lino_text, Some(EnvOptions { lo: -1.0, hi: 1.0, valence: 3 }));

// Structured evaluation: never panics, returns diagnostics for every error.
// See ../docs/DIAGNOSTICS.md for the error-code table.
let evaluation = evaluate(lino_text, Some("kb.lino"), None);
for diag in &evaluation.diagnostics {
    eprintln!("{}", format_diagnostic(diag, Some(lino_text)));
}

// Parse and evaluate individual expressions
let mut env = Env::new(Some(EnvOptions { lo: 0.0, hi: 1.0, valence: 3 }));
let tokens = tokenize_one("(a = a)");
let ast = parse_one(&tokens).unwrap();
let truth_value = eval_node(&ast, &mut env);

// Register a domain plugin, or use the built-in automatic-sequences plugin
// that is already registered on new Env instances.
env.register_domain_plugin("automatic-sequences", automatic_sequences_domain_plugin);
let theorem = decide_automatic_sequence_theorem("thue-morse-cube-free");

// Apply link tactics to a proof state
let tactic = parse_one(&tokenize_one("(by reflexivity)")).unwrap();
let goal = parse_one(&tokenize_one("(a = a)")).unwrap();
let tactic_result = run_tactics(ProofState::from_goals(vec![goal]), &[tactic]);
// -> tactic_result.state.goals is empty, diagnostics is empty

let eq = parse_one(&tokenize_one("(a = b)")).unwrap();
let rewritten = rewrite(&parse_one(&tokenize_one("(a = a)")).unwrap(), &eq).unwrap();
let simplified = simplify(&parse_one(&tokenize_one("((f a) = (f a))")).unwrap(), &[eq]).unwrap();

// Quantize a value to N discrete levels
let q = quantize(0.4, 3, 0.0, 1.0); // -> 0.5 (nearest ternary level)

// Adapter for consumers that already selected an interpretation
let formalization = formalize_selected_interpretation(FormalizationRequest {
    text: "0.1 + 0.2 = 0.3".to_string(),
    interpretation: Interpretation::arithmetic_equality("0.1 + 0.2 = 0.3"),
    formal_system: "rml-arithmetic".to_string(),
    dependencies: vec![],
});
let evaluation = evaluate_formalization(&formalization);
// -> computable truth-value result 1.0
```

The meta-expression adapter deliberately keeps unsupported real-world claims partial. A selected interpretation such as `moon orbits the Sun` is returned as non-computable with explicit unknowns until a consumer supplies a formal shape and reproducible dependencies.

## Testing

```bash
cargo test
```

The test suite covers:
- Tokenization, parsing, and quantization
- Evaluation logic and operator aggregators
- Many-valued logics: unary, binary (Boolean), ternary (Kleene), quaternary, quinary, higher N-valued, and continuous (fuzzy)
- Both `[0, 1]` and `[-1, 1]` ranges
- Liar paradox resolution across logic types
- Decimal-precision arithmetic and numeric equality
- Dependent type system: universes, Pi-types, lambdas, application, definitional equality, capture-avoiding substitution, freshness, type queries
- Link-based tactic engine: reflexivity, symmetry, transitivity, induction, suppose, introduce, by, rewrite, simplify, exact
- Domain plugins: Pecan-style automatic-sequence theorem decisions
- Self-referential types: `(Type: Type Type)`, paradox resolution alongside types

## Implementation Notes

The Rust implementation uses the official [`links-notation`](https://crates.io/crates/links-notation) crate for LiNo parsing. The implementation is a direct port of the JavaScript version and produces identical results for all test cases.

## License

See [LICENSE](../LICENSE) file.
