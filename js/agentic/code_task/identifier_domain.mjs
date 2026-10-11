// Rust Reference lex.keywords and ident, Rust 2024; bounded ASCII non-keyword identifiers.
const NON_IDENTIFIERS = new Set(["_", "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "gen", "macro", "override", "priv", "try", "typeof", "unsized", "virtual", "yield"]);
/** Mirrors rust_identifier_is_valid; unicode/raw identifier profiles remain unsupported. */
export function rustIdentifierIsValid(identifier) {
  return typeof identifier === 'string' && /^[A-Za-z_][A-Za-z_0-9]*$/u.test(identifier) && !NON_IDENTIFIERS.has(identifier);
}
