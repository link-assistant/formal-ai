// ---------------------------------------------------------------------------
// Formal rendering
// ---------------------------------------------------------------------------

/// Render one atom (`Name(x)` / `Name(x, object)`) in a target language.
fn render_atom(language: &FormalLanguage, predicate: &AppliedPredicate, variable: &str) -> String {
    let pred_name = apply_identifier_rule(language, &predicate.name);
    let obj_name = predicate
        .object
        .as_deref()
        .map(|o| apply_identifier_rule(language, o))
        .unwrap_or_default();
    let template = if predicate.object.is_some() {
        &language.atom_with_object
    } else {
        &language.atom
    };
    fill(
        template,
        &[
            ("predicate", &pred_name),
            ("variable", variable),
            ("object", &obj_name),
        ],
    )
}

/// Render the whole clause in a target formal language.
pub fn render_clause(clause: &QuantifiedClause, slug: &str) -> Option<String> {
    let language = grammar().formal.iter().find(|item| item.slug == slug)?;
    let and = language.joins.get("and").cloned().unwrap_or_default();
    let variable = &clause.variable;
    let antecedent = clause
        .antecedent
        .iter()
        .map(|predicate| render_atom(language, predicate, variable))
        .collect::<Vec<_>>()
        .join(&and);
    let consequent = render_atom(language, &clause.consequent, variable);
    let template = match clause.quantifier.as_str() {
        "forall" => &language.clause_conditional,
        "no" if !language.clause_negative.is_empty() => &language.clause_negative,
        _ => &language.clause_conjunctive,
    };
    let quantifier_symbol = language
        .quantifiers
        .get(&clause.quantifier)
        .cloned()
        .unwrap_or_default();
    let joined = [antecedent.clone(), consequent.clone()].join(&and);
    let rendered = fill(
        template,
        &[
            ("quantifier", &quantifier_symbol),
            ("variable", variable),
            ("antecedent", &antecedent),
            ("consequent", &consequent),
            ("joined", &joined),
            ("quantifier_word", &clause.quantifier),
        ],
    );
    Some(rendered.trim_end().to_owned())
}

// ---------------------------------------------------------------------------
// Formal parsing (deformalization input side)
// ---------------------------------------------------------------------------

/// Split `text` on `separator` at paren depth zero.
fn split_top_level(text: &str, separator: char) -> Vec<String> {
    let mut depth = 0usize;
    let mut parts = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        match character {
            '(' => {
                depth += 1;
                current.push(character);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(character);
            }
            c if c == separator && depth == 0 => {
                parts.push(current.trim().to_owned());
                current.clear();
            }
            c => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_owned());
    }
    parts
}

/// Parse one FOL atom: `Name(x)` or `Name(x, object)`.
fn parse_atom(text: &str, variable: &str) -> Option<AppliedPredicate> {
    let text = text.trim();
    let open = text.find('(')?;
    let close = text.rfind(')')?;
    let name = capitalize(text[..open].trim());
    let args = text[open + 1..close]
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>();
    let object = args
        .iter()
        .find(|argument| **argument != variable)
        .map(|argument| argument.trim_matches('"').to_owned());
    Some(AppliedPredicate { name, object })
}

/// Parse FOL text (any quantifier surface the seed declares) back into a
/// clause. `∀x (A(x) ∧ B(x) → C(x))` and `¬∃x (A(x) ∧ B(x) ∧ C(x))` are
/// both accepted; `forall`/`exists`/`~ exists` (Rocq surfaces) too.
pub fn parse_fol_clause(text: &str) -> Option<QuantifiedClause> {
    let text = text.trim();
    let fol = grammar().formal.iter().find(|item| item.slug == "fol")?;
    let mut quantifier = String::new();
    let mut rest = text;
    // Rocq-style ASCII word surfaces first (they are prefixes of nothing
    // the symbol table carries), then the FOL symbol table.
    for (kind, surface) in [
        ("no", "~ exists"),
        ("forall", "forall"),
        ("exists", "exists"),
    ] {
        if let Some(stripped) = text.strip_prefix(surface) {
            kind.clone_into(&mut quantifier);
            rest = stripped;
            break;
        }
    }
    if quantifier.is_empty() {
        for (kind, symbol) in &fol.quantifiers {
            if text.starts_with(symbol.as_str()) {
                quantifier.clone_from(kind);
                rest = &text[symbol.len()..];
                break;
            }
        }
    }
    if quantifier.is_empty() {
        return None;
    }
    let rest = rest.trim();
    let variable: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if variable.is_empty() {
        return None;
    }
    let open = rest.find('(')?;
    let close = rest.rfind(')')?;
    // Rocq's ASCII arrow normalizes to the FOL arrow before splitting.
    let body = rest[open + 1..close].trim().replace("->", "→");
    let conditional_parts = split_top_level(&body, '→');
    let atoms: Vec<String> = if conditional_parts.len() >= 2 {
        let mut all = split_top_level(&conditional_parts[0], '∧');
        all.extend(split_top_level(&conditional_parts[1], '∧'));
        all
    } else {
        split_top_level(&body, '∧')
    };
    let predicates: Vec<AppliedPredicate> = atoms
        .iter()
        .filter_map(|atom| parse_atom(atom, &variable))
        .collect();
    if predicates.len() < 2 {
        return None;
    }
    let (antecedent, consequent) = if conditional_parts.len() >= 2 {
        let antecedent_atoms: Vec<AppliedPredicate> = split_top_level(&conditional_parts[0], '∧')
            .iter()
            .filter_map(|atom| parse_atom(atom, &variable))
            .collect();
        let consequent = parse_atom(&conditional_parts[1], &variable)?;
        (antecedent_atoms, consequent)
    } else {
        let consequent = predicates[predicates.len() - 1].clone();
        (predicates[..predicates.len() - 1].to_vec(), consequent)
    };
    Some(QuantifiedClause {
        quantifier,
        variable,
        antecedent,
        consequent,
    })
}

// ---------------------------------------------------------------------------
// Natural rendering (deformalization output side)
// ---------------------------------------------------------------------------

/// Render the clause as a natural sentence in `language`.
pub fn render_clause_natural(clause: &QuantifiedClause, language: &str) -> Option<String> {
    let natural = grammar()
        .natural
        .iter()
        .find(|item| item.language == language)?;
    let quantifier_word = natural
        .quantifiers
        .iter()
        .find(|(_, kind)| *kind == clause.quantifier)
        .map(|(word, _)| word.clone())?;
    let join_word = natural
        .joins
        .iter()
        .find(|(_, kind)| *kind == "and")
        .map(|(word, _)| word.clone())
        .unwrap_or_default();
    let rel_marker = natural.rel_markers.first().cloned().unwrap_or_default();
    let conditional = clause.quantifier == "forall";
    let template = if conditional {
        &natural.clause_conditional
    } else {
        &natural.clause_conjunctive
    };
    let words_of = |predicate: &AppliedPredicate| -> String {
        let head = decapitalize(&predicate.name);
        // The predicate's words in order: head, the seed's object
        // introducer when the language declares one, then the object.
        match (&predicate.object, natural.object_introducers.first()) {
            (Some(object), _) if natural.verb_final => [object.as_str(), head.as_str()].join(" "),
            (Some(object), Some(introducer)) => {
                [head.as_str(), introducer.as_str(), object.as_str()].join(" ")
            }
            (Some(object), None) => [head.as_str(), object.as_str()].join(" "),
            (None, _) => head,
        }
    };
    let head = decapitalize(&clause.antecedent.first()?.name);
    let main = words_of(&clause.consequent);
    let relatives = clause.antecedent[1..]
        .iter()
        .map(|predicate| decapitalize(&predicate.name))
        .collect::<Vec<_>>()
        .join(&[" ", join_word.as_str(), " "].concat());
    Some(fill(
        template,
        &[
            ("quantifier", &quantifier_word),
            ("head", &head),
            ("rel_marker", &rel_marker),
            ("relatives", &relatives),
            ("main", &main),
        ],
    ))
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// True when a formal quantifier symbol from any target grammar occurs in
/// the prompt (the deformalization trigger for text already in FOL). Only
/// the unambiguous symbols count: the ASCII words `forall`/`exists` are
/// ordinary English words, so they never trigger on their own.
fn carries_formal_surface(prompt: &str) -> bool {
    ["∀", "∃", "¬∃"]
        .iter()
        .any(|symbol| prompt.contains(symbol))
}

/// The language of a cue role suffix (`command_ru` → `ru`).
fn role_language(role: &str) -> &str {
    role.rsplit('_').next().unwrap_or("en")
}

/// Which cue role families matched the prompt, as (family, language)
/// pairs — `command` for formalize, `direction` for deformalize.
fn matched_roles(prompt: &str, normalized: &str) -> Vec<(String, String)> {
    let lower = prompt.to_lowercase();
    let mut out = Vec::new();
    let tree = parse_lino(TARGETS);
    for record in target_records(&tree).filter(|child| child.name == "cues") {
        let Some(intent_node) = named_child(record, "intent") else {
            continue;
        };
        for role_node in intent_node
            .children
            .iter()
            .filter(|child| child.name == "role")
        {
            let role = role_node.id.clone();
            let hit = child_values(role_node, "phrase").iter().any(|phrase| {
                normalized.contains(phrase.as_str()) || lower.contains(phrase.as_str())
            });
            if hit {
                let family = role.split('_').next().unwrap_or("command").to_owned();
                out.push((family, role_language(&role).to_owned()));
            }
        }
    }
    out
}

/// The formal target the prompt names, if any (an alias surface).
fn named_target(prompt: &str, normalized: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    grammar()
        .formal
        .iter()
        .find(|language| {
            language.aliases.iter().any(|alias| {
                normalized.contains(alias.as_str()) || lower.contains(&alias.to_lowercase())
            })
        })
        .map(|language| language.slug.clone())
}

/// The sentence to formalize: the text after a colon separator when
/// present, else the prompt with the cue phrase removed.
fn sentence_under_discussion(prompt: &str) -> String {
    if let Some((_, after)) = prompt.split_once(':') {
        let trimmed = after.trim();
        if !trimmed.is_empty() {
            return trimmed.trim_end_matches(['.', '。', '।']).to_owned();
        }
    }
    prompt.trim().to_owned()
}

/// The statement a formalization request carries beyond its cue, or the
/// formal clause it deformalizes (issue #1175 R3).
///
/// The statement is [`sentence_under_discussion`]; it is an operand when a
/// formal quantifier symbol occurs, or when, with every cue phrase and target
/// alias removed, a word outside the frame roles remains ("Formalize this in
/// Lean" carries none). Twin of `claimOperandFormalizationStatement` in
/// `js/worker/formal_ai_worker_claim_operands.js`.
#[must_use]
pub fn formalization_statement(prompt: &str) -> Option<String> {
    let sentence = sentence_under_discussion(prompt);
    if carries_formal_surface(prompt) {
        return Some(sentence);
    }
    let tree = parse_lino(TARGETS);
    let phrases: Vec<String> = target_records(&tree)
        .filter(|record| record.name == "cues")
        .filter_map(|record| named_child(record, "intent"))
        .flat_map(|intent| intent.children.iter().filter(|child| child.name == "role"))
        .flat_map(|role| child_values(role, "phrase"))
        .chain(
            grammar()
                .formal
                .iter()
                .flat_map(|language| language.aliases.iter().cloned()),
        )
        .collect();
    let rest = crate::capability_routing::without_surfaces(&sentence, &phrases);
    (!crate::capability_routing::only_frame_words(&rest)).then_some(sentence)
}

/// Recognize and answer a formalization or deformalization request.
///
/// Returns `None` when the prompt is neither, so the dispatch chain
/// continues (issue requirement R1: the canned search paragraph must
/// never answer a formalization prompt again).
pub fn handle_formalization_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let roles = matched_roles(prompt, normalized);
    // Every deformalize verb contains its formalize verb ("deformalize" /
    // "деформализуй"), so a matched direction cue decides the direction —
    // mirroring `tryFormalizationRequest` in the JS twin.
    let direction = roles.iter().any(|(family, _)| family == "direction");
    let formalize = !direction && roles.iter().any(|(family, _)| family == "command");
    let deformalize = direction || (!formalize && carries_formal_surface(prompt));
    if !formalize && !deformalize {
        return None;
    }
    // The language of the cue that decided the direction: a Lean statement
    // named `formalized` carries the English formalize verb, so a Russian
    // deformalize request must not answer in English.
    let wanted = if direction { "direction" } else { "command" };
    let request_language = roles
        .iter()
        .find(|(family, _)| family == wanted)
        .or_else(|| roles.first())
        .map_or_else(|| "en".to_owned(), |(_, language)| language.clone());
    log.append(
        "formalization:direction",
        if formalize {
            "formalize"
        } else {
            "deformalize"
        }
        .to_owned(),
    );
    log.append("formalization:language", request_language.clone());
    // Issue #1175 R3: a cue with no statement beyond it (and no formal
    // clause) is refused by name, the answer its claim row's refusal lane keeps.
    if formalization_statement(prompt).is_none() {
        log.append("formalization_request:refusal", "no_statement".to_owned());
        return Some(finalize_simple(
            prompt,
            log,
            INTENT,
            "response:formalization",
            &response("formalization_no_statement", &request_language),
            0.4,
        ));
    }

    let body = if formalize {
        formalize_answer(prompt, &request_language, log)
    } else {
        deformalize_answer(prompt, &request_language, log)
    };
    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:formalization",
        &body.0,
        body.1,
    ))
}

/// Compose the formalize-direction answer body.
fn formalize_answer(prompt: &str, language: &str, log: &mut EventLog) -> (String, f32) {
    let sentence = sentence_under_discussion(prompt);
    let natural = grammar()
        .natural
        .iter()
        .find(|item| item.language == language)
        .cloned()
        .unwrap_or_else(|| {
            grammar()
                .natural
                .iter()
                .find(|item| item.language == "en")
                .cloned()
                .unwrap_or_default()
        });
    if let Some(clause) = parse_quantified_clause(&sentence, &natural) {
        log.append_fields(
            "formalization:clause",
            &[
                ("quantifier", &clause.quantifier),
                ("variable", &clause.variable),
                ("antecedents", &clause.antecedent.len().to_string()),
                ("consequent", &clause.consequent.name),
            ],
        );
        record_clause_fragment(log, "parse", &clause);
        let slugs: Vec<String> = {
            let mut all: Vec<String> = grammar()
                .formal
                .iter()
                .map(|item| item.slug.clone())
                .collect();
            // A named target renders first; the others follow, because
            // every declared target is a legitimate rendering of the
            // same parsed clause (issue requirement R4).
            if let Some(one) = named_target(prompt, &sentence.to_lowercase()) {
                all.sort_by_key(|slug| slug != &one);
            }
            all
        };
        let mut blocks = Vec::new();
        for slug in &slugs {
            if let Some(rendered) = ClauseExporter::active().export(&clause, slug) {
                record_render_fragment(log, &clause, slug, &rendered);
                blocks.push(format!("```{slug}\n{rendered}\n```"));
            }
        }
        let prover_slots = record_prover_fragment(log, &clause, language);
        let rml_check = record_rml_fragment(log, &clause);
        let statement_block = blocks.join("\n\n");
        let antecedents = clause
            .antecedent
            .iter()
            .map(|predicate| predicate.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let derivation = crate::seed::report_text(
            "formalization_derivation",
            &[
                ("sentence", &sentence),
                ("quantifier", &clause.quantifier),
                ("variable", &clause.variable),
                ("antecedents", &antecedents),
                ("consequent", &clause.consequent.name),
                ("templates", &slugs.join(", ")),
            ],
        );
        let honesty = honesty_block(&prover_slots, &rml_check, language);
        (
            fill(
                &response("formalization_result", language),
                &[
                    ("statement_block", &statement_block),
                    ("honesty", &honesty),
                    ("derivation", &derivation),
                ],
            ),
            0.7,
        )
    } else {
        log.append(
            "formalization:clause",
            "no quantified clause recognized".to_owned(),
        );
        (
            fill(
                &response("formalization_unparsed", language),
                &[(
                    "reason",
                    "no quantifier word or symbol matched the seed grammar",
                )],
            ),
            0.4,
        )
    }
}

/// The formal span of a prompt: from the first quantifier symbol to the
/// end, so leading cue words ("Deformalize ∀x (…)") do not block parsing.
fn formal_span(prompt: &str) -> &str {
    let start = ["∀", "∃", "¬∃"]
        .iter()
        .filter_map(|symbol| prompt.find(symbol))
        .min()
        .unwrap_or(0);
    prompt[start..].trim()
}

/// Compose the deformalize-direction answer body, including the
/// structural round-trip check (requirement R5).
fn deformalize_answer(prompt: &str, language: &str, log: &mut EventLog) -> (String, f32) {
    let Some((source, clause)) = parse_any_formal_clause(prompt) else {
        log.append(
            "formalization:clause",
            "no formal clause recognized".to_owned(),
        );
        return (
            fill(
                &response("formalization_unparsed", language),
                &[(
                    "reason",
                    "no formal quantifier surface matched the seed grammar",
                )],
            ),
            0.4,
        );
    };
    log.append_fields(
        "formalization:clause",
        &[
            ("quantifier", &clause.quantifier),
            ("variable", &clause.variable),
            ("source", "formal_text"),
            ("target", &source),
        ],
    );
    record_clause_fragment(log, "deformalize", &clause);
    let natural = grammar()
        .natural
        .iter()
        .find(|item| item.language == language)
        .cloned()
        .unwrap_or_default();
    let sentence = render_clause_natural(&clause, language).unwrap_or_default();
    // Both legs must hold: the natural reading re-parses to the clause, and
    // the clause re-rendered into its source target parses back to it.
    let target_holds = target_round_trip_holds(&clause, &source);
    let round_trip = match parse_quantified_clause(&sentence, &natural) {
        Some(reparsed) if target_holds && structure_key(&reparsed) == structure_key(&clause) => {
            "structure preserved (re-parse matches)"
        }
        Some(_) => "structure drifted (re-parse differs; stated honestly)",
        None => "re-parse failed (stated honestly)",
    };
    log.append("formalization:round_trip", round_trip.to_owned());
    let prover_slots = record_prover_fragment(log, &clause, language);
    let rml_check = record_rml_fragment(log, &clause);
    log.append_fields(
        crate::derivation::FORMALIZE_FRAGMENT_KIND,
        &[
            ("stage", "round_trip"),
            ("target", &source),
            ("verdict", round_trip),
        ],
    );
    let honesty = honesty_block(&prover_slots, &rml_check, language);
    (
        fill(
            &response("formalization_deformalized", language),
            &[
                ("sentence", &sentence),
                ("round_trip", round_trip),
                ("honesty", &honesty),
            ],
        ),
        0.7,
    )
}
