use super::*;

/// Search again for the part of the question the evidence has not covered.
///
/// The refinement is the uncovered aspects *alone*, not the original question
/// repeated. Re-issuing the whole question returns the whole first result set
/// again; dropping the aspects already grounded is what makes the second search
/// reach documents the first could not. Returns `None` when the question is
/// fully covered or the round budget is spent — both mean it is time to answer.
pub(super) fn plan_deeper_round(
    tool_names: &[&str],
    progress: &Progress,
    query: &str,
) -> Option<AgenticPlan> {
    if progress.count(Capability::Search) >= MAX_RESEARCH_ROUNDS {
        return None;
    }
    let open = uncovered_aspects(query, progress);
    // Another round is only worth spending on a gap we can actually name, and
    // this rule is deliberately strict about that: exactly one aspect of a
    // several-aspect question is unsupported by everything read so far.
    //
    // The strictness is not tuning, it is a response to how weak the underlying
    // signal is. Token coverage is a poor proxy for "answered": a page that
    // genuinely answers "when are elections in usa" does not repeat every word
    // of the question, and how many words it echoes varies with the language it
    // is written in — the same page shape leaves two aspects open in Hindi and
    // none in English. Anything looser turns that variation into wasted rounds
    // on questions that were already answered.
    //
    // A single open aspect is different in kind. It is the shape of a real
    // follow-up — the specifications were found and only the warranty is
    // missing — and it yields a refinement worth issuing, because the refined
    // query is that aspect alone rather than a restatement of the question.
    //
    // The cost is that genuinely partial answers with two or more gaps are
    // returned as-is instead of researched further. That is the intended
    // trade: answering with what was actually found beats spending a round on
    // a guess about what is missing.
    //
    // This also makes the loop terminate on its own, independently of the round
    // budget: uncovered aspects only ever shrink as evidence accumulates.
    if open.len() != 1 || aspects_of(query).len() < 3 {
        return None;
    }
    let tool = tool_for(tool_names, Capability::Search)?;
    Some(plan_one(
        tool,
        json!({ "query": open.join(" ") }).to_string(),
    ))
}

/// Reduce fetched evidence to the sentences that actually bear on `query`.
///
/// A fetch tool returns the whole page; answering with it verbatim is what made
/// issue #771's session unreadable. Sentences are scored by symbolic token
/// overlap with the query — the same non-neural similarity the ranker uses — and
/// the best few are returned in document order so the extract still reads as
/// prose. Scoring is deterministic and carries no natural-language vocabulary,
/// so it works in every supported language — see [`relevance`] for how the
/// space-less scripts are handled.
pub(super) fn extract_answer(query: &str, evidence: &str) -> String {
    let evidence = structurally_complete_prose(evidence);
    let sentences = crate::summarization::formalize(&evidence);
    let mut scored: Vec<(usize, f32, &str)> = sentences
        .iter()
        .enumerate()
        .map(|(position, statement)| {
            (
                position,
                relevance(query, &statement.text),
                statement.text.as_str(),
            )
        })
        .filter(|(_, score, _)| *score > 0.0)
        .collect();
    if scored.is_empty() {
        // Nothing overlaps the query: fall back to the head of the document
        // rather than the whole of it, so the answer stays bounded either way.
        return truncate_chars(&evidence, MAX_EXTRACT_CHARS);
    }
    // Rank by relevance, keep the best few, then restore document order.
    scored.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(left.0.cmp(&right.0))
    });
    scored.truncate(EXTRACT_SENTENCES);
    scored.sort_by_key(|(position, _, _)| *position);
    scored
        .iter()
        .map(|(_, _, text)| *text)
        .collect::<Vec<_>>()
        .join(" ")
}
