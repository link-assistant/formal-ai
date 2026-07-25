//! Role constants for task decomposition: splitting a task into sub-tasks,
//! judging whether a task is atomic, asking for the first step, and deciding
//! whether a candidate sub-task is independently checkable (issue #847).
//!
//! Re-exported flat through [`super`] so every constant stays reachable as
//! `crate::seed::roles::ROLE_*` and `crate::seed::ROLE_*` (issue #386).

/// Semantic role: a verb asking for a task to be broken into smaller pieces.
///
/// "split", "break down", "decompose", "разбей", "раздели", "विभाजित",
/// "拆分", "分解" — matched as raw substrings so inflected and compound forms
/// are caught. Carried by `task_decomposition_action`; the decomposition
/// handler requires it together with [`ROLE_SUBTASK_UNIT_NOUN`] or
/// [`ROLE_DECOMPOSABLE_TASK_NOUN`].
pub const ROLE_TASK_DECOMPOSITION_ACTION: &str = "task_decomposition_action";
/// Semantic role: the noun naming the pieces a task decomposes into.
///
/// "subtask", "steps", "подзадачи", "шаги", "उपकार्य", "子任务" — matched as
/// raw substrings. Carried by `subtask_unit_noun`; read together with
/// [`ROLE_TASK_DECOMPOSITION_ACTION`] or [`ROLE_SUBTASK_ENUMERATION_CUE`].
pub const ROLE_SUBTASK_UNIT_NOUN: &str = "subtask_unit_noun";
/// Semantic role: the noun naming the thing being decomposed.
///
/// "task", "issue", "problem", "задачу", "कार्य", "任务" — matched as raw
/// substrings. Carried by `decomposable_task_noun`; read by both the
/// decomposition and the atomicity branch of the handler.
pub const ROLE_DECOMPOSABLE_TASK_NOUN: &str = "decomposable_task_noun";
/// Semantic role: a request to enumerate rather than to act.
///
/// "list", "what are", "перечисли", "какие", "सूची", "列出" — matched as raw
/// substrings. Carried by `subtask_enumeration_cue`; lets "what are the steps
/// for X" reach decomposition without naming a splitting verb.
pub const ROLE_SUBTASK_ENUMERATION_CUE: &str = "subtask_enumeration_cue";
/// Semantic role: the property word asking whether a task can still be split.
///
/// "atomic", "indivisible", "атомарн", "неделим", "अविभाज्य", "原子",
/// "不可分" — matched as raw substrings so "atomicity" and the Russian
/// inflections are caught by their stem. Carried by
/// `task_atomicity_predicate`; the atomicity branch requires it together with
/// [`ROLE_DECOMPOSABLE_TASK_NOUN`] or [`ROLE_SUBTASK_UNIT_NOUN`].
pub const ROLE_TASK_ATOMICITY_PREDICATE: &str = "task_atomicity_predicate";
/// Semantic role: a request for the single next action rather than the whole
/// decomposition.
///
/// "first step", "next step", "первый шаг", "पहला कदम", "第一步" — matched as
/// raw substrings. Carried by `task_first_step_cue`; answered with the first
/// leaf of the same decomposition, so the two views can never disagree.
pub const ROLE_TASK_FIRST_STEP_CUE: &str = "task_first_step_cue";
/// Semantic role: a verb whose completion a reader can observe.
///
/// "add", "create", "remove", "run", "добавь", "создай", "जोड़ें", "添加" —
/// matched as raw substrings. Carried by `observable_task_action`; a candidate
/// sub-task counts as independently checkable only when it evidences this role
/// and does not evidence [`ROLE_UNOBSERVABLE_TASK_ACTION`].
pub const ROLE_OBSERVABLE_TASK_ACTION: &str = "observable_task_action";
/// Semantic role: a verb that names a mental state with no observable
/// completion criterion.
///
/// "understand", "explore", "study", "изучи", "разберись", "समझें", "理解" —
/// matched as raw substrings. Carried by `unobservable_task_action`; the issue
/// names "Understand the codebase" as exactly the child a decomposition must
/// never emit, so a segment evidencing this role is merged into a neighbour
/// instead of becoming a sub-task.
pub const ROLE_UNOBSERVABLE_TASK_ACTION: &str = "unobservable_task_action";
