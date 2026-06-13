//! Role constants for program and script authoring, software-project
//! planning, and the measurement, cardinal-number, arithmetic and calendar
//! primitive clusters they build on (issue #386).
//!
//! Re-exported flat through [`super`] so every constant stays reachable as
//! `crate::seed::roles::ROLE_*` and `crate::seed::ROLE_*` (issue #386).

/// Semantic role: a thing a program produces that a later turn can refer back
/// to (a result, an output, the program/script/code itself, an ordering).
pub const ROLE_PROGRAM_ARTIFACT: &str = "program_artifact";
/// Semantic role: an operation a follow-up turn can request against the active
/// program (sort, reverse, cancel, change, …) — additive or subtractive.
pub const ROLE_PROGRAM_MODIFICATION: &str = "program_modification";
/// Semantic role: a kind of program artifact a user can ask to be authored
/// (a program, a script, code, a function). The noun side of "write a <kind>".
pub const ROLE_PROGRAM_KIND: &str = "program_kind";
/// Semantic role: a verb that requests a program artifact be produced (write,
/// create, show, generate, make, build). The verb side of "write a <kind>".
pub const ROLE_PROGRAM_REQUEST: &str = "program_request";
/// Semantic role: the program *genus* itself — the broad "program" noun
/// (program / программа / प्रोग्राम / 程序).
///
/// The script-authoring recognizer defers to the parametric write-program
/// route whenever this appears, so a full "write a program" request keeps its
/// richer formalization rather than collapsing to a bare script.
pub const ROLE_PROGRAM_GENUS: &str = "program_genus";
/// Semantic role: the author verb specifically used to request a script or
/// code be composed (write / напиши / написать / लिखो / 编写).
///
/// The verb side of "write a script" — a strict subset of
/// [`ROLE_PROGRAM_REQUEST`] that omits the show/create/generate verbs, which
/// must not on their own trigger synthesis.
pub const ROLE_SCRIPT_AUTHORING_VERB: &str = "script_authoring_verb";
/// Semantic role: a script-or-code artifact noun (script / code / скрипт /
/// код / स्क्रिप्ट / कोड / 脚本 / 代码).
///
/// The noun side of "write a script" — a strict subset of
/// [`ROLE_PROGRAM_KIND`] that excludes the broad program genus and the
/// function noun.
pub const ROLE_SCRIPT_OR_CODE_ARTIFACT: &str = "script_or_code_artifact";
/// Semantic role: a surface reference to the canonical hello-world archetype
/// — the codebase's minimal first program.
///
/// The script-authoring recognizer defers to the program-synthesis route for
/// these so the hello-world example keeps its dedicated handling.
pub const ROLE_HELLO_WORLD_REFERENCE: &str = "hello_world_reference";
/// Semantic role: a concrete unit of measurement (metre, byte, kilogram, …).
/// Each such meaning is `defined_by` the [`ROLE_PHYSICAL_DIMENSION`] it measures.
pub const ROLE_MEASUREMENT_UNIT: &str = "measurement_unit";
/// Semantic role: a physical dimension (length, mass, time, …). Units that
/// belong to different dimensions cannot be converted into one another.
pub const ROLE_PHYSICAL_DIMENSION: &str = "physical_dimension";
/// Semantic role: a cardinal counting number (zero, one, two … ten).
///
/// Each such meaning is `defined_by` the `cardinal_number` genus and carries
/// spelled forms in every language plus the script-independent numeral surface.
pub const ROLE_CARDINAL_NUMBER_WORD: &str = "cardinal_number_word";
/// Semantic role: a spelled arithmetic operator (plus, minus, times, …).
///
/// Each such meaning is `defined_by` the `arithmetic_operation` genus and
/// carries operator surfaces in every language. `contains_word_operator` reads
/// them to decide whether a prompt names an arithmetic operator in words rather
/// than symbols.
pub const ROLE_ARITHMETIC_OPERATOR_WORD: &str = "arithmetic_operator_word";
/// Semantic role: a named day of the week (Monday … Sunday). The meaning slug
/// is the English weekday name so a handler can resolve a matched lexeme back
/// to a position in the seven-day cycle.
pub const ROLE_CALENDAR_WEEKDAY: &str = "calendar_weekday";
/// Semantic role: the "comes after" relation between weekdays — a +1 step in
/// the seven-day cycle (after, next day, после, के बाद, 之后, …).
pub const ROLE_CALENDAR_DIRECTION_NEXT: &str = "calendar_direction_next";
/// Semantic role: the "comes before" relation between weekdays — a -1 step in
/// the seven-day cycle (before, previous day, перед, से पहले, 之前, …).
pub const ROLE_CALENDAR_DIRECTION_PREVIOUS: &str = "calendar_direction_previous";
/// Semantic role: the present day relative to the system clock (today,
/// сегодня, आज, 今天). Distinguishes a "what day is it now?" query.
pub const ROLE_CALENDAR_TODAY: &str = "calendar_today";
/// Semantic role: a reference to a day, date, or week — the noun a calendar
/// question is about (day, weekday, date, week, день, неделя, 星期, …).
pub const ROLE_CALENDAR_DAY_REFERENCE: &str = "calendar_day_reference";
/// Semantic role: an interrogative or imperative asking which day (what,
/// which, какой, कौन, 什么, …). The question side of a calendar query.
pub const ROLE_CALENDAR_QUESTION: &str = "calendar_question";
/// Semantic role: an imperative or verb phrase requesting creation or booking
/// of a calendar event (забей, поставь, schedule, book on the calendar, …).
/// Surfaces live in data/seed/meanings-calendar.lino under role calendar_schedule_action.
pub const ROLE_CALENDAR_SCHEDULE_ACTION: &str = "calendar_schedule_action";
/// Semantic role: a noun naming the thing being scheduled (встреча, meeting,
/// событие, call, event, …). Surfaces live in meanings-calendar.lino.
pub const ROLE_CALENDAR_EVENT: &str = "calendar_event";
/// Semantic role: clock time / time-of-day expressions that anchor an event
/// (17:00, в 17:00, 5 pm, вечером, …). Actual numeric extraction is in the
/// handler; surfaces are data-driven.
pub const ROLE_CALENDAR_TIME: &str = "calendar_time";
/// Semantic role: alias phrases that indicate a target IANA timezone for a
/// scheduled event (по грузии, по тбилиси, Asia/Tbilisi, …). The handler
/// maps a hit on these surfaces to the concrete IANA string.
pub const ROLE_CALENDAR_TIMEZONE_ALIAS: &str = "calendar_timezone_alias";
/// Semantic role: a relative-date word that anchors a scheduled event to a day
/// offset from "today" (tomorrow, завтра, послезавтра, 后天, …).
///
/// The meaning slug names the offset (`calendar_tomorrow` → +1,
/// `calendar_day_after_tomorrow` → +2); the handler resolves the slug back to a
/// number of days so the words live once, per language, in
/// `data/seed/meanings-calendar.lino`.
pub const ROLE_CALENDAR_RELATIVE_DATE: &str = "calendar_relative_date";
/// Semantic role: a relation in the knowledge base that maps a subject to a
/// value (capital, population, author, …).
///
/// A fact query detects which relation a prompt asks about by walking every
/// meaning carrying this role, in declaration order; each is `defined_by` the
/// `knowledge_relation` concept.
pub const ROLE_FACT_RELATION: &str = "fact_relation";
/// Semantic role: a follow-up that verifies an already-designed software
/// artifact behaves correctly (test it, run the tests, протестируй, 测试, …).
pub const ROLE_SOFTWARE_FOLLOWUP_VERIFICATION: &str = "software_followup_verification";
/// Semantic role: a follow-up that runs or executes the designed artifact
/// (run it, execute it, запусти, 运行, चलाओ, …).
pub const ROLE_SOFTWARE_FOLLOWUP_EXECUTION: &str = "software_followup_execution";
/// Semantic role: a follow-up that demonstrates the artifact's output
/// (show me, demo it, покажи, 显示, दिखाओ, …).
pub const ROLE_SOFTWARE_FOLLOWUP_DEMONSTRATION: &str = "software_followup_demonstration";
/// Semantic role: a request to display a named output ("show me …", "print …",
/// "покажи мне …", "给我看…", …).
///
/// Each surface is a [`crate::seed::Slot::Prefix`] whose text before the
/// ellipsis (U+2026) slot is the show-me/print/display opener; the clause after
/// it — up to the first sentence-ending punctuation, capped at twelve words — is
/// the expected output the user wants surfaced. The opener is matched anywhere
/// in the prompt, not only at the start, so "test it and show me the result"
/// still captures "the result". A meaning carrying this role is `defined_by`
/// the `software_followup` and `action` concepts.
pub const ROLE_OUTPUT_DISPLAY_REQUEST: &str = "output_display_request";
/// Semantic role: a verb that requests a software artifact be authored.
///
/// Surfaces include write, build, create, implement, develop, design, scaffold,
/// … — the verb side of "build me a <artifact>". Distinct from
/// `program_request`, which gates the narrower "write a <program>" synthesis
/// path; the two overlap on the shared verbs, but a software-authoring verb
/// need not trip program synthesis.
pub const ROLE_SOFTWARE_AUTHORING_ACTION: &str = "software_authoring_action";
/// Semantic role: a kind of software artifact an authoring request can ask for.
///
/// Examples are a web app, a CLI tool, a browser extension, a library, …. Each
/// is `defined_by` the `software_artifact` genus; a handler resolves a matched
/// lexeme back to its slug and maps the slug to a canonical English label.
pub const ROLE_SOFTWARE_ARTIFACT_KIND: &str = "software_artifact_kind";
/// Semantic role: a category a software feature requirement falls into.
///
/// Examples are state tracking, data exchange, automation, validation,
/// integration, user interface, and a catch-all project behavior. The union of
/// these meanings' words detects that a clause states a feature requirement;
/// the first category (in declaration order) whose word appears classifies it,
/// so the code knows only the concept "a requirement has a category".
pub const ROLE_SOFTWARE_REQUIREMENT_CATEGORY: &str = "software_requirement_category";
/// Semantic role: the software-feature genus (feature, requirement, …). A
/// prompt that mentions a feature/requirement earns the "requirements"
/// approval gate.
pub const ROLE_SOFTWARE_FEATURE: &str = "software_feature";
/// Semantic role: how the assistant should deliver a software solution.
///
/// The non-default modes — manual instructions, immediate execution, script
/// generation — each carry this role. A handler walks them in declaration
/// order (so the order encodes priority) and selects the first evidenced in
/// the prompt, falling back to code generation when none is.
pub const ROLE_SOFTWARE_DELIVERY_MODE: &str = "software_delivery_mode";
/// Semantic role: the programming language a software implementation targets.
///
/// python, rust, javascript, …. Walked in declaration order; the first
/// evidenced language wins, else the default (typescript) is used.
pub const ROLE_SOFTWARE_IMPLEMENTATION_LANGUAGE: &str = "software_implementation_language";
/// Semantic role: the function word that marks which programming language an
/// implementation should target — English "in", Russian "на" (and the
/// head-final Hindi/Chinese forms, carried for completeness).
///
/// The unknown-language extractor reads the language name that *follows* this
/// marker, so only the head-initial English/Russian surfaces are consulted for
/// extraction. Known languages resolve through the catalog first; this marker
/// is the fallback that names a language absent from the catalog ("in
/// Brainfuck").
pub const ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION: &str = "implementation_language_preposition";
/// Semantic role: the head noun "language" ("language", Russian "языке", …).
///
/// It may sit between [`ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION`] and the
/// language name ("in the language Brainfuck"). When it follows the marker the
/// extractor skips it to read the name after it.
pub const ROLE_IMPLEMENTATION_LANGUAGE_NOUN: &str = "implementation_language_noun";
/// Semantic role: a tabletop/RPG game domain.
///
/// A D&D unit, token, wargame piece, Owlbear scene, …. A request is a
/// game-unit tracker only when it pairs a domain with a mechanic (see
/// [`ROLE_GAME_TRACKER_MECHANIC`]).
pub const ROLE_GAME_TRACKER_DOMAIN: &str = "game_tracker_domain";
/// Semantic role: a combat mechanic a tabletop tracker follows — hit points,
/// damage, protection, resistance, cooldowns. Pairs with the domain above.
pub const ROLE_GAME_TRACKER_MECHANIC: &str = "game_tracker_mechanic";
/// Semantic role: a request to approve the work step by step (each step, step
/// by step, …) — adds the `each_step` approval gate.
pub const ROLE_SOFTWARE_STEP_GRANULARITY: &str = "software_step_granularity";
/// Semantic role: a shell or command-line surface (shell, bash, a command,
/// docker, `WebVM`, …) — adds the `bash_command` approval gate.
pub const ROLE_SOFTWARE_BASH_COMMAND: &str = "software_bash_command";
/// Semantic role: a whole-prompt approval trigger (approve, yes, proceed, …).
///
/// Unlike the other roles this matches the *entire* compacted prompt, not a
/// passing mention: a go-ahead like "approve plan" moves the dialogue from
/// plan to implementation, while "approve the email validation step" does not.
pub const ROLE_SOFTWARE_APPROVAL_TRIGGER: &str = "software_approval_trigger";
/// Semantic role: the subject of a program-synthesis request — the *function*
/// it asks to be written (the noun side of "implement a function …").
pub const ROLE_PROGRAM_SYNTHESIS_SUBJECT: &str = "program_synthesis_subject";
/// Semantic role: a domain signal of a program-synthesis request — the target
/// language (Python) or a data kind it works over (tuple, numbers, vowels).
pub const ROLE_PROGRAM_SYNTHESIS_DOMAIN: &str = "program_synthesis_domain";
/// Semantic role: the request/specification verb of a program-synthesis
/// request (implement, write, return). The verb side of "implement a function".
pub const ROLE_PROGRAM_SYNTHESIS_ACTION: &str = "program_synthesis_action";
/// Semantic role: a surface signal that distinguishes one synthesis task.
///
/// The "distinct numbers"/"differ"/"threshold"/"similar elements"/"count
/// vowels" phrases. A task is `defined_by` the signals that evidence it.
pub const ROLE_PROGRAM_SYNTHESIS_SIGNAL: &str = "program_synthesis_signal";
/// Semantic role: a concrete synthesis task.
///
/// Its slug is the canonical Python function name (`has_close_elements`,
/// `similar_elements`, `count_vowels`). Walked in declaration order; a task is
/// selected when its name is declared or when every `program_synthesis_signal`
/// it is `defined_by` is evidenced in the prompt.
pub const ROLE_PROGRAM_SYNTHESIS_TASK: &str = "program_synthesis_task";
