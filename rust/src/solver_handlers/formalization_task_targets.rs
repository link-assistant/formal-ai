// ---------------------------------------------------------------------------
// Target-text parsing (deformalization from Lean 4, Rocq, Links Notation)
// ---------------------------------------------------------------------------
//
// Issue #1186 R5 asks for the round trip from every rendered target, not only
// from first-order logic. Each target's own seed templates are run in reverse:
// a binder-style target (Lean 4, Rocq) is matched against the clause template
// its quantifier would have selected, with `{variable}`, `{antecedent}`,
// `{consequent}` and `{joined}` as capture slots, and each captured atom is
// matched against the target's atom templates. A line-shaped target (Links
// Notation) is read field by field, the field names coming from the same
// templates. No surface is spelled here — every literal is the seed's.
//
// Mirrored by `js/worker/formal_ai_worker_formalization_targets.js`.

/// One piece of a template: literal text, or a `{slot}` to capture.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TemplatePiece {
    Literal(String),
    Slot(String),
}

/// Collapse every whitespace run to one space, so a pasted rendering with
/// different line breaks still matches its template.
fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Split a template into literal pieces and `{slot}` pieces.
fn template_pieces(template: &str) -> Vec<TemplatePiece> {
    let mut pieces = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        let Some(close) = rest[open..].find('}').map(|offset| open + offset) else {
            break;
        };
        if open > 0 {
            pieces.push(TemplatePiece::Literal(rest[..open].to_owned()));
        }
        pieces.push(TemplatePiece::Slot(rest[open + 1..close].to_owned()));
        rest = &rest[close + 1..];
    }
    if !rest.is_empty() {
        pieces.push(TemplatePiece::Literal(rest.to_owned()));
    }
    pieces
}

/// Match `text` against `template`, returning the captured slots.
///
/// The first literal may sit anywhere (leading prompt prose is skipped); a
/// literal after a slot is found at its first occurrence; a literal after a
/// literal must follow directly. With `anchored`, nothing may trail the last
/// piece. Every captured slot must be non-empty.
fn match_template(template: &str, text: &str, anchored: bool) -> Option<BTreeMap<String, String>> {
    let template = collapse_whitespace(template);
    let text = collapse_whitespace(text);
    let mut captures = BTreeMap::new();
    let mut cursor = 0usize;
    let mut pending: Option<String> = None;
    let mut first = true;
    for piece in template_pieces(&template) {
        match piece {
            TemplatePiece::Slot(name) => pending = Some(name),
            TemplatePiece::Literal(literal) => {
                let at = if pending.is_some() || first {
                    cursor + text[cursor..].find(literal.as_str())?
                } else if text[cursor..].starts_with(literal.as_str()) {
                    cursor
                } else {
                    return None;
                };
                if let Some(name) = pending.take() {
                    let value = text[cursor..at].trim();
                    if value.is_empty() {
                        return None;
                    }
                    captures.insert(name, value.to_owned());
                }
                cursor = at + literal.len();
            }
        }
        first = false;
    }
    let rest = text[cursor..].trim();
    match pending {
        Some(_) if rest.is_empty() => return None,
        Some(name) => {
            captures.insert(name, rest.to_owned());
        }
        None if anchored && !rest.is_empty() => return None,
        None => {}
    }
    Some(captures)
}

/// Read one atom back through the target's atom templates (the object
/// shape first, so `Passes x exam` keeps its object). Guillemet quoting
/// (or whatever `identifier_quoted` the language declares) is stripped
/// from the captured predicate name and object before they are returned.
fn parse_target_atom(
    language: &FormalLanguage,
    text: &str,
    variable: &str,
) -> Option<AppliedPredicate> {
    let bound = |template: &str| fill(template, &[("variable", variable)]);
    if let Some(captures) = match_template(&bound(&language.atom_with_object), text, true) {
        let name = captures.get("predicate")?;
        let object = captures.get("object")?;
        return Some(AppliedPredicate {
            name: capitalize(&strip_identifier_quoting(language, name)),
            object: Some(strip_identifier_quoting(language, object)),
        });
    }
    let captures = match_template(&bound(&language.atom), text, true)?;
    Some(AppliedPredicate {
        name: capitalize(&strip_identifier_quoting(
            language,
            captures.get("predicate")?,
        )),
        object: None,
    })
}

/// Split a captured body on the target's `and` join and read every atom.
fn parse_target_atoms(
    language: &FormalLanguage,
    body: &str,
    variable: &str,
) -> Option<Vec<AppliedPredicate>> {
    let join = language
        .joins
        .get("and")
        .map(|text| text.trim().to_owned())?;
    if join.is_empty() {
        return None;
    }
    body.split(join.as_str())
        .map(|atom| parse_target_atom(language, atom, variable))
        .collect()
}

/// The clause template `render_clause` picks for `kind` in `language`.
fn clause_template_for<'a>(language: &'a FormalLanguage, kind: &str) -> &'a str {
    match kind {
        "forall" => &language.clause_conditional,
        "no" if !language.clause_negative.is_empty() => &language.clause_negative,
        _ => &language.clause_conjunctive,
    }
}

/// Reverse a binder-style rendering (Lean 4, Rocq) into a clause.
fn parse_binder_clause(language: &FormalLanguage, text: &str) -> Option<QuantifiedClause> {
    // `no` before `exists`: the negative surface embeds the existential one.
    for kind in ["forall", "no", "exists"] {
        let symbol = language.quantifiers.get(kind).cloned().unwrap_or_default();
        let template = fill(
            clause_template_for(language, kind),
            &[("quantifier", &symbol), ("quantifier_word", kind)],
        );
        let Some(captures) = match_template(&template, text, false) else {
            continue;
        };
        let Some(variable) = captures.get("variable") else {
            continue;
        };
        let clause = if let (Some(antecedent), Some(consequent)) =
            (captures.get("antecedent"), captures.get("consequent"))
        {
            QuantifiedClause {
                quantifier: kind.to_owned(),
                variable: variable.clone(),
                antecedent: parse_target_atoms(language, antecedent, variable)?,
                consequent: parse_target_atom(language, consequent, variable)?,
            }
        } else {
            let mut atoms = parse_target_atoms(language, captures.get("joined")?, variable)?;
            let consequent = atoms.pop()?;
            QuantifiedClause {
                quantifier: kind.to_owned(),
                variable: variable.clone(),
                antecedent: atoms,
                consequent,
            }
        };
        if clause.antecedent.is_empty() {
            return None;
        }
        return Some(clause);
    }
    None
}

/// The first word of the template line carrying `slot` (`variable {variable}`
/// → `variable`), or of the line just before it when the slot stands alone
/// on its line (`antecedent` above `{antecedent}`).
fn line_key(template: &str, slot: &str, line_before: bool) -> Option<String> {
    let marker = ["{", slot, "}"].concat();
    let lines: Vec<&str> = template.lines().collect();
    let index = lines
        .iter()
        .position(|line| line.contains(marker.as_str()))?;
    let line = if line_before {
        lines.get(index.checked_sub(1)?)?
    } else {
        lines.get(index)?
    };
    line.split_whitespace().next().map(str::to_owned)
}

/// Reverse a line-shaped rendering (Links Notation) into a clause, reading
/// the field names from the target's own templates.
fn parse_line_clause(language: &FormalLanguage, text: &str) -> Option<QuantifiedClause> {
    let head = language
        .clause_conditional
        .lines()
        .next()?
        .trim()
        .to_owned();
    let quantifier_key = line_key(&language.clause_conjunctive, "quantifier_word", false)?;
    let variable_key = line_key(&language.clause_conditional, "variable", false)?;
    let antecedent_key = line_key(&language.clause_conditional, "antecedent", true)?;
    let consequent_key = line_key(&language.clause_conditional, "consequent", true)?;
    let conjuncts_key = line_key(&language.clause_conjunctive, "joined", true)?;
    let predicate_key = line_key(&language.atom, "predicate", false)?;
    let object_key = line_key(&language.atom_with_object, "object", false)?;

    let mut lines = text.lines().map(str::trim).skip_while(|line| *line != head);
    lines.next()?;
    let mut kind = String::new();
    let mut variable = DEFAULT_VARIABLE.to_owned();
    let mut sections: BTreeMap<String, Vec<AppliedPredicate>> = BTreeMap::new();
    let mut section = String::new();
    for line in lines {
        let (key, value) = line.split_once(' ').unwrap_or((line, ""));
        let value = value.trim().trim_matches('"');
        if key == quantifier_key {
            kind = language
                .quantifiers
                .iter()
                .find(|(_, surface)| surface.as_str() == value)
                .map(|(name, _)| name.clone())?;
        } else if key == variable_key && !value.is_empty() {
            value.clone_into(&mut variable);
        } else if key == antecedent_key || key == consequent_key || key == conjuncts_key {
            key.clone_into(&mut section);
        } else if key == predicate_key && !value.is_empty() {
            sections
                .entry(section.clone())
                .or_default()
                .push(AppliedPredicate {
                    name: capitalize(value),
                    object: None,
                });
        } else if key == object_key && !value.is_empty() {
            let last = sections
                .get_mut(&section)
                .and_then(|items| items.last_mut())?;
            last.object = Some(value.to_owned());
        }
    }
    if kind.is_empty() {
        return None;
    }
    let mut consequent_items = sections.remove(&consequent_key).unwrap_or_default();
    let (antecedent, consequent) = if consequent_items.is_empty() {
        let mut conjuncts = sections.remove(&conjuncts_key)?;
        let last = conjuncts.pop()?;
        (conjuncts, last)
    } else {
        let last = consequent_items.pop()?;
        (sections.remove(&antecedent_key).unwrap_or_default(), last)
    };
    if antecedent.is_empty() {
        return None;
    }
    Some(QuantifiedClause {
        quantifier: kind,
        variable,
        antecedent,
        consequent,
    })
}

/// Parse the rendering of any non-FOL target the seed declares, naming the
/// target it matched. Line-shaped targets are the ones whose atom spans
/// lines; the others are binder-style.
fn parse_target_clause(text: &str) -> Option<(String, QuantifiedClause)> {
    grammar()
        .formal
        .iter()
        .filter(|language| language.slug != "fol")
        .find_map(|language| {
            let clause = if language.atom.trim_end().lines().count() > 1 {
                parse_line_clause(language, text)
            } else {
                parse_binder_clause(language, text)
            }?;
            Some((language.slug.clone(), clause))
        })
}

/// Parse a formal rendering in any declared target: first-order logic
/// first (its span starts at the first quantifier symbol), then the others.
fn parse_any_formal_clause(prompt: &str) -> Option<(String, QuantifiedClause)> {
    parse_fol_clause(formal_span(prompt))
        .map(|clause| ("fol".to_owned(), clause))
        .or_else(|| parse_target_clause(prompt))
}

/// True when rendering `clause` back into `slug` and parsing that text
/// again reproduces the clause structure (the target leg of the round trip).
fn target_round_trip_holds(clause: &QuantifiedClause, slug: &str) -> bool {
    render_clause(clause, slug)
        .and_then(|rendered| parse_any_formal_clause(&rendered))
        .is_some_and(|(_, again)| structure_key(&again) == structure_key(clause))
}

// ---------------------------------------------------------------------------
// Derivation record (issue #1186 R6)
// ---------------------------------------------------------------------------

/// Append the parsed clause to the derivation stage `formal-ai explain`
/// prints, as one `formalize:fragment` event.
fn record_clause_fragment(log: &mut EventLog, stage: &str, clause: &QuantifiedClause) {
    let antecedents = clause
        .antecedent
        .iter()
        .map(|predicate| predicate.name.as_str())
        .collect::<Vec<_>>()
        .join("|");
    let object = clause.consequent.object.as_deref().unwrap_or_default();
    log.append_fields(
        crate::derivation::FORMALIZE_FRAGMENT_KIND,
        &[
            ("stage", stage),
            ("quantifier", &clause.quantifier),
            ("variable", &clause.variable),
            ("antecedents", &antecedents),
            ("consequent", &clause.consequent.name),
            ("object", object),
        ],
    );
}

/// Append one rendered target (slug, template shape, text) to the
/// derivation record.
fn record_render_fragment(
    log: &mut EventLog,
    clause: &QuantifiedClause,
    slug: &str,
    rendered: &str,
) {
    let template = match clause.quantifier.as_str() {
        "forall" => "clause_conditional",
        "no" if grammar()
            .formal
            .iter()
            .any(|language| language.slug == slug && !language.clause_negative.is_empty()) =>
        {
            "clause_negative"
        }
        _ => "clause_conjunctive",
    };
    log.append_fields(
        crate::derivation::FORMALIZE_FRAGMENT_KIND,
        &[
            ("stage", "render"),
            ("target", slug),
            ("template", template),
            ("text", rendered),
        ],
    );
}
