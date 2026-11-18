//! Whole-prompt controlled dialogue (issue #1584) and its solver hook.
//! JavaScript root: js/agentic/crate/cnl_output.mjs (`controlledDialogue`);
//! the hook mirrors js/worker/formal_ai_worker_solve.js ("Issue #1584").
use super::grammar::{
    Graph, Slots, answerable_questions, canonical_prompt, graph_holds, js_space, js_trim,
    prompt_analysis,
};
use super::projection::project_graph;
use super::{
    OutputConfig, OutputDocument, OutputPart, OutputPolicy, OutputTurn, RenderedOutput,
    render_output,
};
use crate::event_log::EventLog;
use crate::{ConversationRole, ConversationTurn, SymbolicAnswer};

/// The outcome of a recognized whole prompt.
#[derive(Debug, Clone)]
pub enum ControlledDialogue {
    /// A task paraphrase: solve its canonical prompt.
    Task { language: String, prompt: String },
    /// An answer (`cnl_answer`) or a clarification request (`cnl_clarification`).
    Rendered {
        intent: String,
        language: String,
        content: String,
        output_projection: Box<RenderedOutput>,
    },
}

enum Item {
    Graph(Graph),
    Text(OutputPart),
}

fn graph_document(
    language: &str,
    items: Vec<Item>,
    configuration: OutputConfig,
    policy: &OutputPolicy,
) -> OutputDocument {
    let parts: Vec<OutputPart> = items
        .into_iter()
        .map(|item| match item {
            Item::Graph(graph) => {
                let surface = project_graph(&graph, None, language, configuration, policy);
                OutputPart::Graph {
                    graph,
                    surface: Some(surface),
                }
            }
            Item::Text(part) => part,
        })
        .collect();
    let source = parts
        .iter()
        .map(|part| match part {
            OutputPart::Graph { surface, .. } => surface.clone().unwrap_or_default(),
            OutputPart::Meaning { surface, .. } => surface.clone(),
            OutputPart::Opaque { text } | OutputPart::Space { text } => text.clone(),
        })
        .collect();
    OutputDocument {
        language: language.to_owned(),
        source,
        parts,
    }
}

fn polarity(positive: bool) -> &'static str {
    if positive { "positive" } else { "negative" }
}

fn answer_document(
    question: &Graph,
    language: &str,
    configuration: OutputConfig,
    policy: &OutputPolicy,
) -> OutputDocument {
    let holds = graph_holds(question, &policy.shared);
    let yes = holds == (question.polarity == "positive");
    let space = policy.grammar(language).map_or_else(
        || " ".to_owned(),
        |grammar| grammar.data.sentence_space.clone(),
    );
    let statement = Graph {
        polarity: polarity(holds).to_owned(),
        mood: "statement".to_owned(),
        ..question.clone()
    };
    let items = vec![
        Item::Graph(Graph::new(
            "reply",
            Slots::default(),
            polarity(yes),
            "statement",
        )),
        Item::Text(OutputPart::Space { text: space }),
        Item::Graph(statement),
    ];
    graph_document(language, items, configuration, policy)
}

fn clarification_document(
    questions: &[Graph],
    language: &str,
    configuration: OutputConfig,
    policy: &OutputPolicy,
) -> OutputDocument {
    let count = questions.len().to_string();
    let mut items = vec![Item::Graph(Graph::new(
        "clarify",
        Slots::from([("count", count.as_str())]),
        "positive",
        "statement",
    ))];
    for (index, graph) in questions.iter().enumerate() {
        items.push(Item::Text(OutputPart::Space {
            text: "\n".to_owned(),
        }));
        items.push(Item::Text(OutputPart::Opaque {
            text: format!("{}) ", index + 1),
        }));
        items.push(Item::Graph(graph.clone()));
    }
    items.push(Item::Text(OutputPart::Space {
        text: "\n".to_owned(),
    }));
    items.push(Item::Graph(Graph::new(
        "choose",
        Slots::default(),
        "positive",
        "request",
    )));
    graph_document(language, items, configuration, policy)
}

/// The number of `^\s*([0-9]+)\s*[.)]?\s*$`, as text.
fn numbered_choice(prompt: &str) -> Option<&str> {
    let trimmed = js_trim(prompt);
    let digits = trimmed
        .strip_suffix(['.', ')'])
        .unwrap_or(trimmed)
        .trim_end_matches(js_space);
    (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())).then_some(digits)
}

/// Whole-prompt controlled dialogue: a numbered reply chooses a reading of the
/// previous ambiguous question; an ambiguous question asks for clarification;
/// an unambiguous fact question is answered from declared facts; a recognized
/// task paraphrase becomes its canonical solver prompt. Anything else: `None`.
#[must_use]
pub fn controlled_dialogue(
    prompt: &str,
    history: &[ConversationTurn],
    configuration: OutputConfig,
    policy: &OutputPolicy,
) -> Option<ControlledDialogue> {
    let grammars = policy.ordered_grammars();
    let rendered = |intent: &str, language: &str, document: &OutputDocument| {
        let output = render_output(document, configuration, policy, &());
        ControlledDialogue::Rendered {
            intent: intent.to_owned(),
            language: language.to_owned(),
            content: output.content.clone(),
            output_projection: Box::new(output),
        }
    };
    if let Some(number) = numbered_choice(prompt) {
        let previous = history
            .last()
            .filter(|turn| turn.role == ConversationRole::Assistant)
            .and_then(|_| {
                history
                    .iter()
                    .rev()
                    .find(|turn| turn.role == ConversationRole::User)
            })?;
        let earlier = prompt_analysis(&previous.content, &grammars)?;
        if earlier.graphs.len() < 2 {
            return None;
        }
        let options = answerable_questions(&earlier.graphs, &policy.shared);
        let index = number.parse::<usize>().ok()?.checked_sub(1)?;
        if options.len() != earlier.graphs.len() {
            return None;
        }
        let question = options.get(index)?;
        let document = answer_document(question, &earlier.language, configuration, policy);
        return Some(rendered("cnl_answer", &earlier.language, &document));
    }
    let analysis = prompt_analysis(prompt, &grammars)?;
    let language = analysis.language.as_str();
    let graphs = &analysis.graphs;
    if graphs.len() == 1 {
        if let Some(task) = canonical_prompt(&graphs[0], &policy.shared) {
            return Some(ControlledDialogue::Task {
                language: language.to_owned(),
                prompt: task,
            });
        }
    }
    let questions = answerable_questions(graphs, &policy.shared);
    if questions.is_empty() || questions.len() != graphs.len() {
        return None;
    }
    if questions.len() == 1 {
        let document = answer_document(&questions[0], language, configuration, policy);
        return Some(rendered("cnl_answer", language, &document));
    }
    let document = clarification_document(&questions, language, configuration, policy);
    Some(rendered("cnl_clarification", language, &document))
}

/// What the solver does with one turn after the controlled dialogue ran.
pub enum ControlledSolve {
    /// A declared fact answer or a clarification: return it as is.
    Direct(Box<SymbolicAnswer>),
    /// The solved answer and the prompt the rest of the pipeline uses.
    Solved {
        prompt: String,
        answer: SymbolicAnswer,
    },
}

/// Issue #1584 solver hook. A whole-prompt CNL reading answers declared fact
/// questions and asks which reading an ambiguous one means. A task paraphrase
/// is also solved as its canonical prompt (with a fresh log): the user's wording
/// is kept when both reach the same intent, otherwise the controlled reading of
/// the whole prompt wins and its log replaces the original one.
pub fn controlled_solve(
    turn: &OutputTurn,
    history: &[ConversationTurn],
    policy: &OutputPolicy,
    log: &mut EventLog,
    solve: &mut dyn FnMut(&str, &mut EventLog) -> SymbolicAnswer,
) -> ControlledSolve {
    let prompt = turn.prompt.as_str();
    let dialogue = if turn.rewrite.is_none() {
        controlled_dialogue(prompt, history, turn.config, policy)
    } else {
        None
    };
    let task = match dialogue {
        Some(ControlledDialogue::Rendered {
            intent,
            language,
            content,
            output_projection,
        }) => {
            return ControlledSolve::Direct(Box::new(SymbolicAnswer {
                intent,
                answer: content,
                confidence: 1.0,
                evidence_links: vec![format!("cnl:{language}")],
                thinking_steps: Vec::new(),
                links_notation: output_projection.links_notation,
                execution_recipe: None,
            }));
        }
        Some(ControlledDialogue::Task { language, prompt }) => Some((language, prompt)),
        None => None,
    };
    let answer = solve(prompt, log);
    if let Some((language, canonical_prompt)) = task.filter(|(_, task)| task != prompt) {
        let mut canonical_log = EventLog::new();
        let mut canonical = solve(&canonical_prompt, &mut canonical_log);
        if canonical.intent != "unknown" && canonical.intent != answer.intent {
            canonical
                .evidence_links
                .push(format!("cnl:task:{language}"));
            *log = canonical_log;
            return ControlledSolve::Solved {
                prompt: canonical_prompt,
                answer: canonical,
            };
        }
    }
    ControlledSolve::Solved {
        prompt: prompt.to_owned(),
        answer,
    }
}
