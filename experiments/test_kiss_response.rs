// Quick experiment to see the KISS response content
// Run with: cargo test kiss_response_debug -- --nocapture
#[cfg(test)]
mod tests {
    use formal_ai::FormalAiEngine;
    
    #[test]
    fn kiss_response_debug() {
        let response = FormalAiEngine.answer("что такое Kiss в рамках програмирования");
        eprintln!("Intent: {}", response.intent);
        eprintln!("Answer:\n{}", response.answer);
        eprintln!("Evidence: {:?}", response.evidence_links);
    }
}
