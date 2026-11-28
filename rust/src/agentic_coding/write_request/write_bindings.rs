use super::*;

/// Every path a cue binds, in token order.
///
/// A cue stands before its path in the languages that put it there and after it
/// in the languages that do not: Hindi writes `notes/attribution.md फ़ाइल` and
/// `notes/attribution.md में`, so a reader that only ever looks one token back
/// finds no target in a Hindi request at all. A postposition is therefore read
/// too, but only for the target and destination families — an action cue after a
/// path closes the clause rather than naming the file — and only when the path
/// has no cue in front of it.
pub(super) fn write_bindings(toks: &[Token<'_>]) -> Vec<WriteBinding> {
    let target_cues = bare_surfaces(seed::ROLE_FILE_WRITE_TARGET_CUE);
    let dest_cues = bare_surfaces(seed::ROLE_FILE_WRITE_DESTINATION_CUE);
    let action_cues = bare_surfaces(seed::ROLE_FILE_WRITE_ACTION_CUE);
    // The fused reading is for the nouns and particles that a script without
    // spaces writes against their neighbour — `创建文件` is "create" and "file",
    // `文件` is the cue. A verb is not read that way: `输出内容 sample.txt` opens
    // with the write verb `输出` and means "output the contents of", a read, and
    // reading the verb out of the fused token claimed the file the request asked
    // to be shown.
    let families = [
        (CueFamily::Destination, &dest_cues, true),
        (CueFamily::Target, &target_cues, true),
        (CueFamily::Action, &action_cues, false),
    ];
    let quoted =
        crate::normal_markov::quoted_segment_spans(toks.first().map_or("", |token| token.request));
    toks.iter()
        .enumerate()
        .filter_map(|(index, token)| {
            let cleaned = clean_path_token(token.text);
            if !looks_like_file_path(cleaned) || !safe_relative_path(cleaned) {
                return None;
            }
            let path = cleaned.to_owned();
            if let Some(previous) = index.checked_sub(1).map(|before| &toks[before])
                && let Some((family, cue_start, cue_end)) =
                    families.iter().find_map(|(family, cues, fused)| {
                        trailing_cue(previous, cues, *fused).map(|span| (*family, span.0, span.1))
                    })
            {
                return Some(WriteBinding {
                    index,
                    path,
                    cue_start,
                    cue_end,
                    family,
                    cue_precedes: true,
                });
            }
            let next = toks.get(index + 1)?;
            families
                .iter()
                .take(2)
                .find_map(|(family, cues, fused)| {
                    leading_cue(next, cues, *fused).map(|span| (*family, span.0, span.1))
                })
                .map(|(family, cue_start, cue_end)| WriteBinding {
                    index,
                    path,
                    cue_start,
                    cue_end,
                    family,
                    cue_precedes: false,
                })
        })
        .filter(|binding| {
            !quoted
                .iter()
                .any(|segment| binding.cue_start < segment.end && binding.cue_end > segment.start)
        })
        .collect()
}

/// The path a sentence names as the *destination* of a write it states.
///
/// [`stated_write_target`] answers a weaker question -- which path a cue points
/// at -- and a sentence can point a cue at a path it is not delivering to,
/// because the same cues introduce the file the work is done *in*. Both of these
/// name a path after a target cue and state a write action:
///
/// > In the file `src/solver_handlers/document_request.rs`, add "toward " to the
/// > TARGET_MARKERS list.
///
/// > Then create `agent-ladder-effects/node-1.1.2.2.1.lino` recording what you
/// > changed.
///
/// Only the second is somewhere to put an answer. In the first the thing being
/// added goes into a list *inside* the file, and reading it as a delivery
/// destroys the file it was asked to edit -- the planner writes its own status
/// line over the source and the write reports success.
///
/// What separates them is order, the same adjacency principle
/// [`cued_write_target`] already uses to bind a cue to a path. A delivery
/// composes something and then says where to put it, so its path *follows* the
/// action; a sentence that says where the work happens states the place first
/// and the action afterwards.
///
/// Measured over the 1 118 request sentences this repository records that name a
/// file and carry a cue (`experiments/issue_1069_paths_in_prose/cue-order-survey.py`),
/// the path follows the action in 21.56% of them, precedes it in 65.74%, and in
/// the remaining 12.70% there is no write cue at all -- an edit, which is never a
/// delivery. So this reads roughly a fifth of them as destinations, where the
/// weaker "cues a path and states a write" test reads all 1 118.
///
/// The rule that suggests itself first is the cue family: the seed declares a
/// second family for edits, so a sentence mentioning one could be doing work
/// rather than delivering. That fails on this corpus. Thirty of the sentences
/// carry both families, and the ladder's own delivery sentence is one of them --
/// "Then create `agent-ladder-effects/node-1.1.2.2.1.lino` … followed by at
/// least four words that state the **change** you made" -- so the family rule
/// discards the record the node exists to produce. Position separates what
/// mention cannot.
///
/// Position alone still admits a value that merely *looks* like a path, because
/// the operand of an insertion follows the action too: "Edit `scripts/metric.rs`:
/// add \"bun.lock\" to the `IGNORE_FILES` list" names the file before the action
/// and the member after it. So the candidates are taken in order and a
/// double-quoted one is skipped rather than accepted -- see [`quoted_as_value`].
/// The same survey measures what that costs: of the 241 sentences the position
/// rule admits, 2 (0.83%) quote their path, and both are member insertions of
/// exactly this shape. No delivery destination in the corpus is double-quoted.
pub(in super::super) fn delivered_write_target(sentence: &str) -> Option<String> {
    let toks = tokens(sentence);
    let action = first_action_cue_start(&toks)?;
    cued_write_targets(&toks)
        .into_iter()
        .find(|(index, _)| toks[*index].start > action && !quoted_as_value(toks[*index].text))
        .map(|(_, path)| path)
}
