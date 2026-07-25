//! Task decomposition as a first-class working task (issue #847).
//!
//! Splitting a task into sub-tasks and judging whether a task is atomic are two
//! views of one recursion, so they live in one module and share one algorithm:
//!
//! * [`decompose_task`] splits a task recursively until every leaf is atomic or
//!   the depth bound is reached. The bound is *reported* through
//!   [`Decomposition::depth_bound_reached`], never hidden.
//! * [`SubTask::atomic`] is that recursion's base case, so "is this task
//!   atomic?" is answerable standalone: a task is atomic exactly when the
//!   splitter cannot produce two independently checkable children from it.
//!
//! Children are *independently checkable* by construction ([`is_checkable`]): a
//! segment counts only when it names an action whose completion a reader can
//! observe. The issue names "Understand the codebase" as the child a
//! decomposition must never emit; such a fragment is merged into a neighbour
//! instead of being promoted to a sub-task.
//!
//! Every surface this module reasons over comes from
//! `data/seed/meanings-decomposition.lino` through a semantic role (issue
//! #386); no natural-language word appears here. Splitting is therefore the
//! same operation in every supported language, and the whole module is a pure
//! function of (task text, depth bound, seed), which is what makes a
//! decomposition deterministic.

use crate::coding::contains_cjk;
use crate::engine::{normalize_prompt, stable_id};
use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::meta_frame::AtomicityReason;
use crate::seed::{
    self, ROLE_CLAUSE_CONTINUATION_MARKER, ROLE_OBSERVABLE_TASK_ACTION,
    ROLE_UNOBSERVABLE_TASK_ACTION,
};

/// A node of a task decomposition: the task itself at the root, one sub-task at
/// every other position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubTask {
    /// Content-addressed identifier, stable for a given path and text.
    pub id: String,
    /// Dotted position in the tree — `1`, `1.2`, `1.2.1`, … — used both as the
    /// numbering of the rendered list and as the parent link in the trace.
    pub path: String,
    /// The sub-task text, carrying its own observable completion criterion.
    pub text: String,
    /// Recursion depth (0 at the root).
    pub depth: u8,
    /// Whether this node is a recursion leaf.
    pub atomic: bool,
    /// Why the node is (or is not) atomic. Reuses the recursive core's
    /// vocabulary (issue #559) rather than inventing a parallel one.
    pub reason: AtomicityReason,
    /// Sub-tasks produced by splitting this node (empty when atomic).
    pub children: Vec<Self>,
}

impl SubTask {
    /// Collect every leaf of this subtree, in source order.
    pub fn collect_leaves<'a>(&'a self, out: &mut Vec<&'a Self>) {
        if self.children.is_empty() {
            out.push(self);
        } else {
            for child in &self.children {
                child.collect_leaves(out);
            }
        }
    }

    /// Total number of nodes in this subtree.
    #[must_use]
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(Self::node_count).sum::<usize>()
    }

    /// Does this subtree contain a leaf that was cut short by the depth bound?
    ///
    /// A leaf the bound stopped at but which would not have split anyway is not
    /// counted: reporting it would tell the reader the answer is truncated when
    /// it is complete. This is the same test `collect_rows` uses to decide
    /// whether to mark a row, so the note and the markers agree.
    #[must_use]
    pub fn has_depth_bound_leaf(&self) -> bool {
        if self.children.is_empty() {
            return self.reason == AtomicityReason::DepthBound && !self.is_unsplittable();
        }
        self.children.iter().any(Self::has_depth_bound_leaf)
    }

    /// Append this node and its descendants to `out` as `(path, text, bounded)`
    /// rows, skipping the root (which is the task, not a sub-task).
    fn collect_rows(&self, out: &mut Vec<(String, String, bool)>) {
        if self.depth > 0 {
            out.push((
                self.path.clone(),
                self.text.clone(),
                self.reason == AtomicityReason::DepthBound && !self.is_unsplittable(),
            ));
        }
        for child in &self.children {
            child.collect_rows(out);
        }
    }

    /// A depth-bounded leaf that could not have been split anyway is not really
    /// bounded — it is atomic, and marking it would misreport the bound.
    fn is_unsplittable(&self) -> bool {
        split_once_checkable(&self.text).is_empty()
    }

    /// Render this node and its descendants as Links Notation records.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "sub_task".to_owned()),
            ("path", self.path.clone()),
            ("text", self.text.clone()),
            ("depth", self.depth.to_string()),
            ("atomic", self.atomic.to_string()),
            ("atomicity_reason", self.reason.slug().to_owned()),
        ];
        for child in &self.children {
            pairs.push(("child", child.path.clone()));
        }
        let mut out = format_lino_record(&self.id, &pairs);
        for child in &self.children {
            out.push('\n');
            out.push_str(&child.to_links_notation());
        }
        out
    }
}

/// A complete decomposition of one task, bounded by a depth limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decomposition {
    /// The task as given.
    pub task: String,
    /// The depth bound this decomposition ran under.
    pub max_depth: u8,
    /// The root node (the task itself).
    pub root: SubTask,
}

impl Decomposition {
    /// Is the task atomic — that is, did the splitter fail to find two
    /// independently checkable children? This is the recursion's base case, and
    /// answering "is this task atomic?" needs nothing else.
    #[must_use]
    pub const fn is_atomic(&self) -> bool {
        self.root.atomic
    }

    /// Did the recursion stop at the depth bound anywhere, rather than because
    /// every leaf became atomic? The answer reports this instead of hiding it.
    #[must_use]
    pub fn depth_bound_reached(&self) -> bool {
        self.root.has_depth_bound_leaf()
    }

    /// Every leaf of the decomposition, in source order.
    #[must_use]
    pub fn leaves(&self) -> Vec<&SubTask> {
        let mut out = Vec::new();
        self.root.collect_leaves(&mut out);
        out
    }

    /// The sub-tasks as `(path, text, bounded)` rows, in source order and
    /// excluding the root. `bounded` marks a row the depth bound cut short.
    #[must_use]
    pub fn rows(&self) -> Vec<(String, String, bool)> {
        let mut out = Vec::new();
        self.root.collect_rows(&mut out);
        out
    }

    /// Render the decomposition as a hierarchical numbered list. Rows the depth
    /// bound cut short are suffixed with `marker` so the report is visible in
    /// the answer itself.
    #[must_use]
    pub fn numbered_lines(&self, marker: &str) -> Vec<String> {
        self.rows()
            .into_iter()
            .map(|(path, text, bounded)| {
                if bounded {
                    format!("{}. {} {}", path.as_str(), text.as_str(), marker)
                } else {
                    format!("{}. {}", path.as_str(), text.as_str())
                }
            })
            .collect()
    }

    /// Render the whole decomposition as Links Notation records.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        self.root.to_links_notation()
    }
}

/// Decompose `task` recursively until every leaf is atomic or `max_depth` is
/// reached.
///
/// The result is a pure function of `task`, `max_depth`, and the seed lexicon,
/// so the same task and configuration always produce the same decomposition.
#[must_use]
pub fn decompose_task(task: &str, max_depth: u8) -> Decomposition {
    let text = task.trim();
    // The root carries no number of its own: it is the task, and the numbering
    // a reader sees starts at its children.
    let root = build(text, "", 0, max_depth);
    Decomposition {
        task: text.to_owned(),
        max_depth,
        root,
    }
}

fn build(text: &str, path: &str, depth: u8, max_depth: u8) -> SubTask {
    let id = stable_id("sub_task", &format!("{path}:{depth}:{text}"));
    if depth >= max_depth {
        return leaf(id, path, text, depth, AtomicityReason::DepthBound);
    }

    let parts: Vec<String> = split_once_checkable(text)
        .into_iter()
        .filter(|part| part != text)
        .collect();
    if parts.len() < 2 {
        // The base case: nothing splits off that a reader could check on its
        // own, so the task is as small as it usefully gets.
        let reason = if is_checkable(text) {
            AtomicityReason::DirectMethod
        } else {
            AtomicityReason::SingleNeed
        };
        return leaf(id, path, text, depth, reason);
    }

    let children = parts
        .iter()
        .enumerate()
        .map(|(index, part)| build(part, &child_path(path, index), depth + 1, max_depth))
        .collect();
    SubTask {
        id,
        path: path.to_owned(),
        text: text.to_owned(),
        depth,
        atomic: false,
        reason: AtomicityReason::NotAtomic,
        children,
    }
}

/// `1`, `2`, … under the root; `1.1`, `1.2`, … under a sub-task.
fn child_path(parent: &str, index: usize) -> String {
    let number = index + 1;
    if parent.is_empty() {
        number.to_string()
    } else {
        format!("{parent}.{number}")
    }
}

fn leaf(id: String, path: &str, text: &str, depth: u8, reason: AtomicityReason) -> SubTask {
    SubTask {
        id,
        path: path.to_owned(),
        text: text.to_owned(),
        depth,
        atomic: true,
        reason,
        children: Vec::new(),
    }
}

/// Split `task` one level into independently checkable pieces.
///
/// Returns an empty vector when the task cannot be usefully split — which is
/// exactly the atomicity judgement. A split is accepted only when it yields at
/// least two pieces that each satisfy [`is_checkable`], so a decomposition can
/// never hand back a child no one can verify.
#[must_use]
pub fn split_once_checkable(task: &str) -> Vec<String> {
    let segments = segment(task);
    if segments.len() < 2 {
        return Vec::new();
    }
    let distributed = distribute_head_action(&segments);
    let merged = merge_uncheckable(&distributed);
    if merged.len() < 2 {
        return Vec::new();
    }
    merged
}

/// Can a reader tell, by looking, whether `segment` is done?
///
/// A checkable segment names an action with an observable result and does not
/// name a purely mental one. Both sides are seed roles, so the judgement is the
/// same in every supported language.
#[must_use]
pub fn is_checkable(segment: &str) -> bool {
    let normalized = normalize_prompt(segment);
    let lexicon = seed::lexicon();
    lexicon.mentions_role(ROLE_OBSERVABLE_TASK_ACTION, &normalized)
        && !lexicon.mentions_role(ROLE_UNOBSERVABLE_TASK_ACTION, &normalized)
}

/// Split a task into raw segments: sentences when there is more than one,
/// otherwise clauses.
fn segment(task: &str) -> Vec<String> {
    let sentences = split_sentences(task);
    if sentences.len() > 1 {
        return sentences;
    }
    split_clauses(task)
}

/// Split on sentence terminators, keeping the terminator attached. A period
/// flanked by non-whitespace (`3.14`, `release.yml`) stays inside its segment.
fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut current = String::new();
    for (index, &ch) in chars.iter().enumerate() {
        current.push(ch);
        let strong = matches!(ch, '?' | '!' | '。' | '！' | '？');
        let period = ch == '.' && chars.get(index + 1).is_none_or(|next| next.is_whitespace());
        if strong || period {
            push_trimmed(&mut out, &current);
            current.clear();
        }
    }
    push_trimmed(&mut out, &current);
    out
}

/// Split a sentence on list punctuation and on the seed's clause-continuation
/// markers ("and", "и", "और", "并", …), which is the only place a coordinating
/// word is named — and it is named by role, not by spelling.
fn split_clauses(sentence: &str) -> Vec<String> {
    let markers = seed::lexicon().words_for_role(ROLE_CLAUSE_CONTINUATION_MARKER);
    let mut out = Vec::new();
    for chunk in sentence.split([',', ';', '，', '；', '、']) {
        for piece in split_on_markers(chunk, &markers) {
            push_trimmed(&mut out, &piece);
        }
    }
    out
}

/// Split `text` wherever a continuation marker appears as its own word (or, for
/// scripts written without spaces, as a substring).
fn split_on_markers(text: &str, markers: &[String]) -> Vec<String> {
    let mut pieces = vec![text.to_owned()];
    for marker in markers {
        let mut next = Vec::new();
        for piece in pieces {
            next.extend(split_piece_on_marker(&piece, marker));
        }
        pieces = next;
    }
    pieces
}

fn split_piece_on_marker(piece: &str, marker: &str) -> Vec<String> {
    if contains_cjk(marker) {
        return piece
            .split(marker)
            .map(|part| part.trim().to_owned())
            .collect();
    }
    let tokens: Vec<&str> = piece.split_whitespace().collect();
    if !tokens
        .iter()
        .any(|token| token.eq_ignore_ascii_case(marker))
    {
        return vec![piece.to_owned()];
    }
    let mut out = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for token in tokens {
        if token.eq_ignore_ascii_case(marker) {
            out.push(current.join(" "));
            current.clear();
        } else {
            current.push(token);
        }
    }
    out.push(current.join(" "));
    out
}

/// Carry the leading action of the first segment into later segments that do
/// not open with one, so "Create a module and a unit test for it" yields two
/// children that each say what to do instead of leaving a bare noun phrase
/// whose completion criterion only makes sense next to its sibling.
///
/// Only segments long enough to be a task on their own are completed this way;
/// a bare list item ("b.rs" in "add the flag to a.rs, b.rs and c.rs") is left
/// alone so it merges into its neighbour instead of becoming a fake sub-task.
fn distribute_head_action(segments: &[String]) -> Vec<String> {
    let Some(head) = head_action(&segments[0]) else {
        return segments.to_vec();
    };
    segments
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            if index == 0
                || head_action(segment).is_some()
                || contains_cjk(segment)
                || segment.split_whitespace().count() < 3
            {
                return segment.clone();
            }
            format!("{} {}", head.as_str(), segment.as_str())
        })
        .collect()
}

/// The observable action a segment opens with, when it opens with one.
fn head_action(segment: &str) -> Option<String> {
    if contains_cjk(segment) {
        return None;
    }
    let first = segment.split_whitespace().next()?;
    let normalized = normalize_prompt(first);
    seed::lexicon()
        .mentions_role(ROLE_OBSERVABLE_TASK_ACTION, &normalized)
        .then(|| first.to_owned())
}

/// Fold every fragment that is not independently checkable into a neighbour.
///
/// A leading fragment ("In the file scripts/detect-code-changes.rs") belongs to
/// the segment that follows it; a trailing fragment belongs to the one before
/// it. Nothing is dropped, so the children still cover the whole task.
fn merge_uncheckable(segments: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut pending: Option<String> = None;
    for segment in segments {
        let text = pending
            .take()
            .map_or_else(|| segment.clone(), |prefix| join(&prefix, segment));
        if is_checkable(segment) {
            out.push(text);
        } else {
            pending = Some(text);
        }
    }
    if let Some(tail) = pending {
        match out.last_mut() {
            Some(last) => *last = join(last, &tail),
            None => out.push(tail),
        }
    }
    out
}

fn join(left: &str, right: &str) -> String {
    let separator = if contains_cjk(left) || contains_cjk(right) {
        "，"
    } else {
        ", "
    };
    format!("{left}{separator}{right}")
}

fn push_trimmed(out: &mut Vec<String>, candidate: &str) {
    let trimmed = candidate.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_owned());
    }
}

/// Decompose `task`, recording every node as a `sub_impulse` event.
///
/// The existing sub-impulse kind is reused rather than a parallel structure
/// invented (issue #847). The `sub_impulse:*` facets carry the node's position,
/// its atomicity verdict, and the depth-bound report.
pub fn record_task_decomposition(log: &mut EventLog, task: &str, max_depth: u8) -> Decomposition {
    let decomposition = decompose_task(task, max_depth);
    log.append("task_decomposition", decomposition.to_links_notation());
    log.append(
        "task_decomposition:max_depth",
        decomposition.max_depth.to_string(),
    );
    log.append(
        "task_decomposition:atomic",
        decomposition.is_atomic().to_string(),
    );
    emit_sub_impulses(log, &decomposition.root);
    if decomposition.depth_bound_reached() {
        log.append(
            "sub_impulse:depth_bound",
            decomposition.max_depth.to_string(),
        );
    }
    decomposition
}

fn emit_sub_impulses(log: &mut EventLog, node: &SubTask) {
    if node.depth > 0 {
        log.append("sub_impulse", node.text.clone());
        log.append("sub_impulse:path", node.path.clone());
        log.append(
            "sub_impulse:atomicity",
            format!("{} {}", node.path, node.reason.slug()),
        );
    }
    for child in &node.children {
        emit_sub_impulses(log, child);
    }
}
