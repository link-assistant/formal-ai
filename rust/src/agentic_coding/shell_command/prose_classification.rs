use super::*;

/// Recover the command from a remainder that *names* it instead of being it.
///
/// Issue #866 and #867 reported *"Execute ls command"* running `/bin/ls
/// command`, which fails with *cannot access 'command'*. The trailing noun
/// names the command; it is not an argument to it. Seed data declares those
/// nouns per language in `data/seed/terminal-commands.lino`, and their presence
/// is the tell that the remainder is a noun phrase *about* a command rather
/// than a command line — so the command is collected out of it, dropping the
/// determiners that lead the phrase and stopping at the prose that ends it,
/// exactly as argument collection does elsewhere.
///
/// Quoting and shell metacharacters switch the recovery off, because inside
/// them a word is data: `git commit -m 'fix command parsing'` keeps its message.
pub(super) fn command_named_in_prose(
    remainder: &str,
    vocab: &TerminalCommandVocabulary,
) -> Option<String> {
    if remainder.contains(|c| SHELL_QUOTING_AND_METACHARACTERS.contains(&c)) {
        return None;
    }
    let words = remainder.split_whitespace().collect::<Vec<_>>();
    let kept = words
        .iter()
        .copied()
        .filter(|word| !names_a_command(word, vocab))
        .collect::<Vec<_>>();
    if kept.len() == words.len() || kept.is_empty() {
        return None;
    }
    let start = kept.iter().position(|word| !is_prose_word(word))?;
    let mut command = vec![kept[start]];
    for word in &kept[start + 1..] {
        if is_prose_word(word) {
            break;
        }
        command.push(word);
    }
    Some(command.join(" "))
}

/// Resolve a semantic *intent* to its concrete command, backed by the seed
/// [`ShellIntentVocabulary`] (issue #680).
///
/// The two strategies above only fire when the prompt *names* the command (`run
/// pwd`) or asks, in prose, to list a directory. This third strategy handles the
/// common case where the user expresses an intent without naming the tool at all —
/// *"Print the current working directory"* → `pwd`, *"How much disk space is
/// free?"* → `df -h`, *"What is my username?"* → `whoami`. Each intent carries
/// multilingual cue phrases; the first intent whose cue is present in the prompt
/// wins (declaration order is most-specific-first), and its argument — if any — is
/// recovered from the prompt. An intent whose cue matches but whose required
/// argument is absent is skipped so the search continues rather than emitting an
/// argument-less command that would hang (`wc -l` on stdin).
/// The prompt split into sentences, each paired with whether it is a question.
///
/// Sentence boundaries are what separate the user's request from a sentence merely
/// *placed next to* it, so intent cues are read one sentence at a time (issue #907).
pub(super) fn sentences_with_mood(lower: &str) -> Vec<(&str, bool)> {
    let mut sentences = Vec::new();
    let mut start = 0;
    for (index, character) in lower.char_indices() {
        if !matches!(
            character,
            '.' | '!' | '?' | ';' | '\n' | '。' | '！' | '？' | '；'
        ) {
            continue;
        }
        // A dot inside a token ends nothing — `main.py` is one word, not two
        // sentences.
        if character == '.'
            && lower[index + character.len_utf8()..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric)
        {
            continue;
        }
        let text = lower[start..index].trim();
        if !text.is_empty() {
            sentences.push((text, matches!(character, '?' | '？')));
        }
        start = index + character.len_utf8();
    }
    let tail = lower[start..].trim();
    if !tail.is_empty() {
        sentences.push((tail, false));
    }
    sentences
}

/// Whether `sentence` states a fact *about* `cue` instead of requesting it.
///
/// The shape is `<cue> <copula> <value>` (English, Russian, Spanish, Chinese) or
/// `<cue> <value> <copula>` (Hindi): the cue opens the sentence as its subject,
/// a seed-declared copula links it, and a value follows. A question is never a
/// statement — neither one punctuated with `?` nor one that merely carries a
/// seed-declared question word ("我的用户名**是什么**") — and a cue that does not
/// open the sentence is not its subject, so
/// *"tell me what the current time is"* and *"what is the date?"* keep routing.
/// Whether `sentence` *declares where something lives*: the cue is followed by a
/// colon and a single absolute path.
///
/// A copula is not the only way to state a fact — agent harnesses prefer the
/// label form, and Hive Mind's *"Your prepared working directory: /tmp/example"*
/// supplies the working directory exactly as a copula would, which made every
/// production run answer with `pwd` (issue #907, follow-up).
///
/// The value is what separates this from a request, and it has to be: earlier
/// attempts keyed on the colon (which swallowed *"delete the file: old.log"*),
/// on request verbs (a four-word list that rescued nothing), and on subject
/// position (which cannot see that *"count"* and *"search"* are verbs). A
/// harness declares an **absolute** path — that is the only kind worth stating —
/// while a request's argument after a colon is a relative path, a search term,
/// or prose. Every row of the table above stays a request under this rule.
pub(super) fn labels_a_value(sentence: &str, cue: &str) -> bool {
    let Some(head_end) = sentence.to_lowercase().find(cue) else {
        return false;
    };
    let Some(value) = sentence[head_end + cue.len()..]
        .trim_start()
        .strip_prefix([':', '：'])
        .map(str::trim)
    else {
        return false;
    };
    let mut words = value.split_whitespace();
    let Some(path) = words.next() else {
        return false;
    };
    words.next().is_none() && (path.starts_with('/') || path.starts_with("~/"))
}
