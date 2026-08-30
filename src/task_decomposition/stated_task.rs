//! Recovering, from a prompt, the task it is asking about (issues #847, #1066).
//!
//! A prompt is not a task. It states one, and around it the caller writes
//! whatever else the request needs -- a quotation, an introducing colon, a
//! deadline, the conditions the work runs under. Every reply this crate makes
//! about a decomposition is composed by putting the recovered task inside a
//! statement about it, so reading the wrong span does not degrade the answer,
//! it replaces the subject of the answer.
//!
//! The reading lives here rather than beside the handler because it is the same
//! judgement [`super::decompose_task`] is asked to make, and the two may not
//! drift: a handler that recovered one span while the recursion described
//! another is how a task with four checkable children came back reported as an
//! irreducible single need (issue #1066).

/// The punctuation that ends a sentence in the supported writing systems.
///
/// Devanagari ends a sentence with a danda rather than a full stop, and the CJK
/// forms are their own code points, so trimming ASCII alone would leave the
/// question mark on exactly the languages that need it removed most.
const SENTENCE_END: &[char] = &['.', '!', '?', '。', '！', '？', '।', '॥', '…'];

/// The quote pairs used across the supported languages. Left and right differ
/// for every pair except the straight quotes, which close with themselves.
const QUOTE_PAIRS: &[(char, char)] = &[
    ('«', '»'),
    ('“', '”'),
    ('‘', '’'),
    ('「', '」'),
    ('『', '』'),
    ('"', '"'),
    ('\'', '\''),
];

/// The colon forms that introduce what follows them.
const INTRODUCING_COLON: &[char] = &[':', '：'];

/// Recover the task `prompt` is asking about.
///
/// The reading narrows in three steps, each scoped by the one before it: the
/// block that asks, then the quotation or introducing colon inside it, then the
/// punctuation that ended the sentence it came from.
///
/// A prompt that states a task and then addresses the solver separates the two
/// with a blank line -- "… identify where a node stores its children.\n\nThis
/// is recursive binary-tree node 1.1.1.1.1 at depth 5. Solve only this node's
/// task in this fresh temporary repository. …". The second block says how to
/// work and how to report; it is not work of its own, and decomposing it
/// alongside the task produced a sub-task made entirely of the framing
/// (issue #1066). So the blocks that ask are the task, and the rest is
/// addressed to the solver. `asks` decides which is which, and it is the same
/// recogniser that routed the prompt here, never a copy of it.
///
/// Inside that block, the prompts in issue #847 quote the task ("… nothing
/// else: 'Add a paths-ignore filter …'"), so a quoted span wins. Failing that,
/// the text a colon introduces is the task -- but only a colon in the sentence
/// that asks the question. Failing both, the block itself is the task, which is
/// what makes "Split this into steps" work on a bare task.
///
/// The task is then stripped of the punctuation that ended the sentence it was
/// recovered from, because a task is work to do and not an utterance. A task
/// that still carries its asker's question mark turns the statement built
/// around it back into a question: "Is refactoring the payment module an atomic
/// task?" produced the sub-task "Record independently checkable requirements
/// for Is refactoring the payment module an atomic task?", which
/// [`crate::question_necessity`] then read as this answer asking something it
/// had not earned and deleted, leaving a numbered list whose every entry had
/// lost its text. Issue #1066 calls that hollow, and it is: the reply announced
/// sub-tasks and showed none of them.
#[must_use]
pub fn stated_task(prompt: &str, asks: &dyn Fn(&str) -> bool) -> String {
    let stated = asking_blocks(prompt, asks);
    without_sentence_end(
        quoted_span(&stated)
            .or_else(|| after_introducing_colon(&stated, asks))
            .unwrap_or_else(|| stated.clone())
            .trim(),
    )
    .to_owned()
}

/// The blocks of `prompt` that ask, joined; the whole prompt when that reading
/// would lose the question.
///
/// A block that does not ask is kept only when no block asks on its own -- a
/// task can be stated across a blank line, and dropping half of it would be a
/// worse reading than keeping the framing. Nothing is dropped when the prompt
/// is a single block, which is the ordinary case.
fn asking_blocks(prompt: &str, asks: &dyn Fn(&str) -> bool) -> String {
    let blocks: Vec<&str> = prompt
        .split("\n\n")
        .map(str::trim)
        .filter(|block| !block.is_empty())
        .collect();
    if blocks.len() < 2 {
        return prompt.trim().to_owned();
    }
    let asking: Vec<&str> = blocks.iter().copied().filter(|block| asks(block)).collect();
    if asking.is_empty() {
        return prompt.trim().to_owned();
    }
    asking.join("\n\n")
}

/// Drop the sentence-ending punctuation, and any space it left behind.
///
/// Repeated because a sentence can end with more than one mark ("Is it atomic?!")
/// and because trimming one can expose a space in front of the next.
#[must_use]
pub fn without_sentence_end(task: &str) -> &str {
    task.trim_end_matches(|character: char| {
        SENTENCE_END.contains(&character) || character.is_whitespace()
    })
}

fn quoted_span(prompt: &str) -> Option<String> {
    let characters: Vec<char> = prompt.chars().collect();
    let (open_index, closing) = characters
        .iter()
        .enumerate()
        .find_map(|(index, character)| {
            QUOTE_PAIRS
                .iter()
                .find(|(open, _)| open == character)
                .map(|(_, close)| (index, *close))
        })?;
    let close_index = characters
        .iter()
        .rposition(|character| *character == closing)?;
    if close_index <= open_index + 1 {
        return None;
    }
    let span: String = characters[open_index + 1..close_index].iter().collect();
    let trimmed = span.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// The span a colon introduces, when the colon belongs to the sentence that
/// asks the question.
///
/// The scoping is the difference between an introduction and a coincidence. A
/// request states its task once and then keeps writing: "Break the warehouse
/// restocking rewrite into sub-tasks. Deadline: the end of the quarter." Taking
/// the last colon in the prompt made "the end of the quarter" the task, and a
/// deadline is an irreducible single need, so the reply refused to enumerate a
/// rewrite that splits four ways. The ladder of issue #1028 states the same
/// shape in its own words -- every node ends with "Its completion criterion is:
/// <criterion>" -- and every interior node came back reported as unsplittable.
///
/// Scoping a two-part reading to one sentence is the rule
/// [`crate::agentic_coding::shell_command`] already applies to a command that
/// is named rather than ordered (issue #907), and
/// [`crate::agentic_coding::evidence_record`] to a path that is written rather
/// than read.
fn after_introducing_colon(prompt: &str, asks: &dyn Fn(&str) -> bool) -> Option<String> {
    let asking = sentence_spans(prompt)
        .into_iter()
        .find(|span| asks(prompt[span.clone()].trim()))?;
    let colon = prompt[asking.clone()].rfind(INTRODUCING_COLON)? + asking.start;
    let tail = prompt[colon..]
        .char_indices()
        .nth(1)
        .map(|(offset, _)| &prompt[colon + offset..])?;
    let trimmed = tail.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// The byte ranges of `prompt`'s sentences, in source order.
///
/// A dot inside a token ends nothing, so the ladder's "node 1.2.1" and a
/// version number survive as one sentence. A semicolon ends one, because a
/// caller may ask the question in the first half of a compound sentence and
/// state its conditions in the second. The splitter reads punctuation only, so
/// it holds in every supported language and needs no per-language surface list
/// (R386); it is the same split [`crate::agentic_coding::shell_command`] makes
/// to scope a named command to its clause.
fn sentence_spans(prompt: &str) -> Vec<std::ops::Range<usize>> {
    let mut spans = Vec::new();
    let mut start = 0;
    for (index, character) in prompt.char_indices() {
        if !SENTENCE_END.contains(&character) && !matches!(character, '\n' | ';' | '\u{ff1b}') {
            continue;
        }
        if character == '.'
            && prompt[index + character.len_utf8()..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric)
        {
            continue;
        }
        let end = index + character.len_utf8();
        if !prompt[start..index].trim().is_empty() {
            spans.push(start..end);
        }
        start = end;
    }
    if !prompt[start..].trim().is_empty() {
        spans.push(start..prompt.len());
    }
    spans
}
