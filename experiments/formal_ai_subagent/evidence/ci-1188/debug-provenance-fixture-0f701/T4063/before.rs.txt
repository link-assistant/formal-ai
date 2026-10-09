//! Recover statement-bound literal writes without executing source prose.
use super::literal_payload;
use crate::agentic_coding::shell_command_policy::prose_sentences;
use crate::agentic_coding::write_request::{
    CueFamily, Token, WriteBinding, action_cue_start_after, bare_surfaces, clean_content,
    clean_cue_token, first_action_cue_end, first_action_cue_start, first_prefix_lead_end,
    first_raw_content_lead_end, payload_continues_past_its_first_line, ranked_bindings,
    raw_content_lead_close, tokens,
};
use crate::seed::{self, Slot};

/// A write cue cannot authorize a target in another statement or a preceding read request.
fn binding_has_write_instruction(
    request: &str,
    toks: &[Token<'_>],
    binding: &WriteBinding,
) -> bool {
    let clause_start = if binding.cue_precedes {
        binding.cue_start
    } else {
        toks[binding.index].start
    };
    let scoped = statement_scope(request);
    let Some(sentence) = prose_sentences(&scoped)
        .into_iter()
        .find(|sentence| sentence.span.contains(&clause_start))
    else {
        return false;
    };
    let target = &toks[binding.index];
    if target.start < sentence.span.start
        || target.end > sentence.span.end
        || binding.cue_start < sentence.span.start
        || binding.cue_end > sentence.span.end
    {
        return false;
    }
    let Some(statement) = scoped.get(sentence.span.clone()) else {
        return false;
    };
    let action_start =
        first_action_cue_start(&tokens(statement)).map(|start| sentence.span.start + start);
    let Some(before) =
        request.get(sentence.span.start..action_start.unwrap_or(toks[binding.index].start))
    else {
        return false;
    };
    !seed::lexicon().mentions_role(
        "file_read_action_cue",
        &crate::engine::normalize_prompt(before),
    )
}

/// Recover the `(target, content)` of a write request from its wording.
///
/// The recogniser is entirely seed-driven (issue #680). It locates the target
/// file by a `file_write_target_cue`/`file_write_destination_cue` that directly
/// precedes a file-looking, safe relative path, then recovers the content two
/// ways:
///
/// * **Marker-led** — a `file_write_content_lead` phrase ("containing", "with
///   the following", …) introduces the payload. The content is the span after
///   the marker (when the file precedes it) or the span between the marker and
///   the file clause (when the content precedes the file).
/// * **Destination-led** — the "write CONTENT to FILE" shape, where a
///   `file_write_action_cue` opens the request and a *destination* cue (not a
///   positional target cue) routes the preceding span into the file.
/// * **Action-led** — the "write FILE saying CONTENT" shape, where a
///   `file_write_action_cue` ("write"/"create"/"save"/…) directly names the file
///   and a content-lead marker introduces the payload after it. An action cue
///   licenses the file just like a target cue, but only marker-led content is
///   accepted for it, so a bare "create app.rs" (no content) still falls through
///   to the ordinary solver rather than fabricating an empty file.
///
/// Seeded matching uses lowercase text; all slicing boundaries map back to
/// original UTF-8 so expanding and shrinking case mappings retain payload bytes.
pub(super) fn parse_write_request(request: &str) -> Option<(String, String)> {
    let contract = parse_write_contract(request)?;
    Some((contract.target, contract.content))
}

pub(super) fn parse_write_contract(request: &str) -> Option<LiteralWriteContract> {
    let toks = tokens(request);
    if let Some(literal) = literal_payload(request) {
        // A closed whole-file payload supplies the content. Only target cues
        // before it bind the file; its internal prose is authored data.
        if let Some(binding) = ranked_bindings(&toks).into_iter().find(|candidate| {
            toks[candidate.index].end <= literal.start
                && candidate.cue_end <= literal.start
                && binding_has_write_instruction(request, &toks, candidate)
        }) {
            return write_contract(
                request,
                &toks,
                &binding,
                clean_content(&request[literal.start..literal.end]).unwrap_or(literal.text),
                literal.start..literal.end,
            );
        }
    }
    // A cue between two file-shaped tokens can belong to either of them, and the
    // ranking only says which reading to try first. The reading that recovers a
    // payload is the one the sentence supports, so the candidates are taken in
    // order and the first that parses is the answer. A sentence that offers one
    // candidate reaches exactly the branch it always did.
    ranked_bindings(&toks)
        .into_iter()
        .find_map(|binding| parse_write_request_bound(request, &toks, &binding))
}

/// Read `request` as a write delivered through one particular binding.
fn parse_write_request_bound(
    request: &str,
    toks: &[Token<'_>],
    binding: &WriteBinding,
) -> Option<LiteralWriteContract> {
    if !binding_has_write_instruction(request, toks, binding) {
        return None;
    }
    let lowered = request.to_lowercase();
    let dest_cues = bare_surfaces(seed::ROLE_FILE_WRITE_DESTINATION_CUE);
    let file_index = binding.index;
    // The clause is the cue and its path together, so it begins at whichever of
    // them the language puts first.
    let clause_start = if binding.cue_precedes {
        binding.cue_start
    } else {
        toks[file_index].start
    };
    let cue_is_destination = binding.family == CueFamily::Destination;
    // Marker-led content. The payload sits after the marker, bounded by the file
    // clause when the marker comes first ("write the following: hello to x.txt")
    // and running to the end when the clause comes first ("store file x.txt
    // containing hello").
    //
    // A marker that *precedes* the clause additionally needs a write verb, which
    // is the same rule the destination-led and assignment-shaped branches below
    // already apply — without it a read request whose object happens to be a
    // content-lead surface claims a write. The issue-#671 matrix caught
    // `show me the contents of the file beta.md` planning
    // `write(beta.md, "of the")`, destroying the fixture it was asked to read.
    // A marker inside a sentence that specifies a document to compose introduces
    // that document's structure, not its bytes; see
    // [`crate::agentic_coding::note_composition::composed_document_specification_span`]. Reading
    // it as a literal payload is what wrote "the selected tree level, node
    // outcomes, test results, and session id." into the ladder's final proof
    // file (issue #1066), and it claimed the request too, so the note the caller
    // asked for was never composed.
    let specification =
        crate::agentic_coding::note_composition::composed_document_specification_span(request);
    if let Some((marker_start, marker_end)) = first_raw_content_lead_end(request)
        && !specification.is_some_and(|span| span.contains(&marker_end))
        && positions_share_statement(request, marker_end, clause_start)
    {
        let marker_leads = marker_end <= clause_start;
        let statement_end = if marker_leads {
            end_of_statement(request, marker_end, clause_start)
        } else {
            end_of_statement(request, marker_end, request.len())
        };
        // A circumfix marker states where its payload ends as well as where it
        // begins, and the closing literal is grammar rather than bytes: the
        // Hindi "जिसमें Gemfile.lock हो" would otherwise write the verb into the
        // file with the content.
        let payload_end = raw_content_lead_close(request, marker_end)
            .map_or(statement_end, |close| close.min(statement_end));
        let marker_span = request.get(marker_end..payload_end);
        if (!marker_leads || first_action_cue_end(toks).is_some())
            && let Some(content) = marker_span.and_then(clean_content).filter(|content| {
                is_literal_content(
                    content,
                    marker_span,
                    seed::lexicon().mentions_role(
                        seed::ROLE_FILE_WRITE_CONTENT_QUALIFIER,
                        &crate::engine::normalize_prompt(&request[marker_start..marker_end]),
                    ),
                ) && (!names_deferred_work_product(content)
                    || first_prefix_lead_end(
                        &lowered,
                        seed::ROLE_FILE_WRITE_AUTHORITATIVE_CONTENT_LEAD,
                    )
                    .is_some()
                    || literal_payload(request).is_some())
            })
        {
            return write_contract(request, toks, binding, content, marker_end..payload_end);
        }
    }
    let payload;
    let content_span = if cue_is_destination && binding.cue_precedes {
        let action_end = first_action_cue_end(toks)?;
        payload = action_end..clause_start;
        (action_end <= clause_start && positions_share_statement(request, action_end, clause_start))
            .then(|| request.get(action_end..clause_start))?
    } else if cue_is_destination {
        // The same shape read from the other side. A language that marks its
        // destination with a postposition and closes the clause with the verb —
        // "notes/attribution.md में Gemfile.lock लिखो" — states the payload
        // between the two, exactly as the prepositional wording states it
        // between the verb and the preposition.
        let action_start = action_cue_start_after(toks, binding.cue_end)?;
        payload = binding.cue_end..action_start;
        (binding.cue_end <= action_start
            && positions_share_statement(request, binding.cue_end, action_start))
        .then(|| request.get(binding.cue_end..action_start))?
    } else if let Some(value_lead) = toks
        .iter()
        .skip(file_index + 1)
        .find(|token| dest_cues.contains(&clean_cue_token(token.text)))
    {
        // Assignment shape: "set the contents of FILE to VALUE". The target
        // cue identifies the file object and a following destination cue
        // introduces its literal value. Requiring a write action before the
        // file keeps an unrelated "contents of FILE" read request out.
        let action_end = first_action_cue_end(toks)?;
        payload = value_lead.end..request.len();
        (action_end <= clause_start
            && positions_share_statement(request, action_end, clause_start)
            && positions_share_statement(request, clause_start, value_lead.start))
        .then(|| request.get(value_lead.end..))?
    } else {
        return None;
    };
    let content = clean_content(content_span?)?;
    // A recovered payload that is *only* a non-referential subject ("save it to
    // FILE", "write this to FILE") names no literal content — the pronoun points
    // back at content the request expects the recipe to still compose. Treating
    // it as a literal write both fabricates the wrong file (the string "it") and
    // steals the request from the keyword recipe that would author the real
    // artifact, so fall through instead (issue #663).
    //
    // The same is true of a payload that names the *work product* rather than
    // supplying it: "save the answer to FILE" states where an answer goes, not
    // what it says (issue #1066).
    if is_non_referential_content(&content)
        || names_deferred_work_product(&content)
        || !is_literal_content(&content, content_span, false)
    {
        return None;
    }
    write_contract(request, toks, binding, content, payload)
}
/// Mask literal punctuation without changing UTF-8 byte positions.
fn statement_scope(request: &str) -> String {
    let mut scoped = request.to_owned();
    for segment in crate::normal_markov::quoted_segment_spans(request) {
        scoped.replace_range(
            segment.start..segment.end,
            &" ".repeat(segment.end - segment.start),
        );
    }
    scoped
}
/// Where the statement that begins at `from` ends, never past `limit`.
///
/// A literal payload is something the request *states*, and a statement ends
/// where its sentence does. Bounding the span by the file clause alone reads
/// across every sentence in between: "Draft a handover memo containing the
/// migration status, the outstanding blockers, and the on-call owner. Leave the
/// memo in `handover/2026-q3.md`" put the marker in the first sentence and the
/// clause in the second, so the recovered payload ended with the words *Leave
/// the memo* and the caller's memo opened by instructing them to leave it
/// (issue #1066).
///
/// The clause bound still applies inside the sentence, because "write the
/// following: hello to `x.txt`" states marker, payload and clause in one
/// breath. This only refuses to look further than the sentence the marker is in.
///
/// A marker that says nothing more on its own line is the exception, because
/// there the payload is a *block* rather than a phrase: "Create file
/// `rules.lino` containing\n<three lines of lino>" leaves the marker with an
/// empty tail, and a newline ends a sentence, so the sentence bound would
/// recover nothing at all. When the marker's own line has no word left on it,
/// the statement is the block that follows and runs to `limit`, which is what
/// this route always did for block payloads.
///
/// The sentence is the one *prose* reads, so a semicolon inside the payload
/// joins its two halves instead of ending it. Reading the payload at the scope
/// shell routing reads at cut issue #918's minimal-core invariant in half at
/// its semicolon.
fn end_of_statement(request: &str, from: usize, limit: usize) -> usize {
    let scoped = statement_scope(request);
    let Some(sentence) = prose_sentences(&scoped)
        .into_iter()
        .find(|sentence| sentence.span.contains(&from))
    else {
        return limit;
    };
    let says_more = request
        .get(from..sentence.span.end)
        .is_some_and(|tail| tail.chars().any(char::is_alphanumeric));
    if says_more && !payload_continues_past_its_first_line(request, from, sentence.span.end) {
        literal_statement_end(request, sentence.span.end).min(limit)
    } else {
        limit
    }
}
/// Whether two write-request cues belong to one prose statement.
///
/// Literal-write roles are structural only inside the statement that relates
/// them. Without this check, a content lead in one issue-description paragraph
/// can pair with an incidental file-shaped token in a later paragraph and turn
/// repository policy prose into an executable write (issue #1069).
///
/// [`end_of_statement`] deliberately lets a marker-only line introduce the
/// block below it. Reusing that boundary here preserves that supported block
/// shape while rejecting ordinary completed sentences between the two cues.
fn literal_statement_end(request: &str, from: usize) -> usize {
    let mut end = from;
    for character in request.get(from..).unwrap_or_default().chars() {
        if !matches!(character, '.' | '!' | '?' | '。' | '！' | '？' | '।') {
            break;
        }
        end += character.len_utf8();
    }
    end
}
fn positions_share_statement(request: &str, left: usize, right: usize) -> bool {
    let (from, limit) = if left <= right {
        (left, right)
    } else {
        (right, left)
    };
    from == limit || end_of_statement(request, from, limit) == limit
}
/// Punctuation needs a closed literal operand or a seeded explicit qualifier.
fn is_literal_content(content: &str, raw: Option<&str>, explicitly_qualified: bool) -> bool {
    if content.chars().any(char::is_alphanumeric) {
        return true;
    }
    let Some(raw) = raw.filter(|_| !content.is_empty()) else {
        return false;
    };
    let quoted = crate::normal_markov::quoted_segment_spans(raw);
    if let [only] = quoted.as_slice()
        && only.text == content
        && raw[..only.start]
            .chars()
            .all(|character| character.is_whitespace() || ":—–-".contains(character))
        && raw[only.end..]
            .chars()
            .all(|character| character.is_whitespace() || ".!?。！？।".contains(character))
    {
        return true;
    }
    explicitly_qualified
        && !content
            .chars()
            .any(|character| "`\"'«»“”‘’„‚「」『』".contains(character))
}
/// Whether a recovered write payload is nothing but a non-referential subject —
/// a bare pronoun/function word ("it", "this", "that", …) that refers back to
/// context rather than naming literal content. The surfaces carry the
/// [`seed::ROLE_NON_REFERENTIAL_SUBJECT`] role; only whole-word
/// ([`Slot::Bare`]) forms are rejected, so legitimate content that merely
/// *begins* with such a word ("to be or not to be") is still accepted.
fn is_non_referential_content(content: &str) -> bool {
    let lower = content.to_lowercase();
    seed::lexicon()
        .role_word_forms(seed::ROLE_NON_REFERENTIAL_SUBJECT)
        .iter()
        .any(|form| form.slot() == Slot::Bare && lower == form.text)
}
/// Whether a recovered write payload names the result of work the same request
/// asks for, instead of supplying bytes (issue #1066).
///
/// "Save the answer to `out/e.md`" and "leave observable evidence to
/// `out/e.md`" have the destination-led shape of a literal write -- a write
/// verb, a span, a destination cue, a path -- and supply no literal. Taking the
/// span at face value wrote the words *the answer* into the file and, worse,
/// claimed the request as finished, so the investigation that would have
/// produced the real bytes never ran.
///
/// The surfaces carry [`seed::ROLE_FILE_WRITE_DEFERRED_CONTENT_REFERENCE`] as
/// [`Slot::Suffix`] forms, because the head noun is what defers and the
/// modifiers in front of it ("observable", "final") are the caller's, not the
/// lexicon's.
///
/// Applied to marker-led content too: “with the observed result” uses a content
/// marker syntactically, but still names work that has not happened. Explicit
/// bytes remain expressible through the authoritative-content marker, whose
/// distinct role is routed before derived-delivery planning.
fn names_deferred_work_product(content: &str) -> bool {
    let lower = content
        .trim()
        .trim_end_matches(['.', '!', '?', '。', '！', '？'])
        .trim_end()
        .to_lowercase();
    seed::lexicon()
        .role_word_forms(seed::ROLE_FILE_WRITE_DEFERRED_CONTENT_REFERENCE)
        .iter()
        .any(|form| match form.slot() {
            Slot::Bare => lower == form.text,
            Slot::Suffix => ends_with_head_noun(&lower, form.after_slot().trim_start()),
            Slot::Prefix | Slot::Circumfix => false,
        })
}
/// Whether `content` ends with `noun` standing as its own word.
///
/// English and Russian separate a head noun from its modifiers with a space, so
/// the character before the match settles it. Chinese writes the same phrase
/// with no separator at all, which is why the boundary is stated as "not an
/// ASCII alphanumeric" rather than "whitespace": demanding a space would never
/// match 结论, and demanding nothing would match the tail of an unrelated
/// English word.
fn ends_with_head_noun(content: &str, noun: &str) -> bool {
    !noun.is_empty()
        && content.strip_suffix(noun).is_some_and(|before| {
            before
                .chars()
                .next_back()
                .is_none_or(|character| !character.is_ascii_alphanumeric())
        })
}

/// Owned payload and target spans use original UTF-8 byte boundaries.
#[derive(Debug, Clone)]
pub(super) struct LiteralWriteContract {
    pub(super) target: String,
    pub(super) content: String,
    pub(super) payload: std::ops::Range<usize>,
    pub(super) target_span: std::ops::Range<usize>,
}
fn write_contract(
    request: &str,
    toks: &[Token<'_>],
    binding: &WriteBinding,
    content: String,
    range: std::ops::Range<usize>,
) -> Option<LiteralWriteContract> {
    let raw = request.get(range.clone())?;
    let literal = crate::normal_markov::quoted_segment_spans(raw)
        .into_iter()
        .find(|span| {
            clean_content(&raw[span.start..span.end]).as_deref() == Some(content.as_str())
                && raw[..span.start]
                    .chars()
                    .all(|character| character.is_whitespace() || character == ':')
                && raw[span.end..].chars().all(|character| {
                    character.is_whitespace() || ".!?。！？।;；".contains(character)
                })
        });
    let payload = literal.map_or(range.clone(), |span| {
        range.start + span.start..range.start + span.end
    });
    let target = &toks[binding.index];
    Some(LiteralWriteContract {
        target: binding.path.clone(),
        content,
        payload,
        target_span: target.start..target.end,
    })
}
pub(super) fn instruction_view(request: &str, contract: &LiteralWriteContract) -> Option<String> {
    request.get(contract.payload.clone())?;
    let mut view = request.to_owned();
    view.replace_range(
        contract.payload.clone(),
        &" ".repeat(contract.payload.len()),
    );
    Some(view)
}

/// An instruction operand cannot be owned literal payload.
pub(super) fn owns_instruction_span(
    contract: &LiteralWriteContract,
    span: &std::ops::Range<usize>,
) -> bool {
    span.start <= span.end
        && (span.end <= contract.payload.start || span.start >= contract.payload.end)
}
