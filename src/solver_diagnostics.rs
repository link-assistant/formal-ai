pub fn append_diagnostic_trace(
    diagnostic_mode: bool,
    answer: String,
    links_notation: &str,
) -> String {
    if !diagnostic_mode {
        return answer;
    }
    format!("{answer}\n\n[diagnostic]\n{}", links_notation.trim_end())
}
