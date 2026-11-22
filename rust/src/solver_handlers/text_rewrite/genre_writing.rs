use super::*;

/// The genre styleguide's required slots that a request left unfilled.
fn unsatisfied_requirements(requires: &[String], unsatisfied: &[bool]) -> Vec<String> {
    requires
        .iter()
        .zip(unsatisfied.iter().chain(std::iter::repeat(&false)))
        .filter(|&(_, unsatisfied)| *unsatisfied)
        .map(|(requirement, _)| requirement.clone())
        .collect()
}

/// Finalize the honest genre-incomplete response: name the genre and its
/// missing slots instead of fabricating them.
fn finalize_genre_incomplete(
    prompt: &str,
    log: &mut EventLog,
    intent: &str,
    response_link: &str,
    language: &str,
    genre: &str,
    missing: &[String],
) -> SymbolicAnswer {
    let body = render_line(
        "text_transform_genre_incomplete",
        language,
        &[("genre", genre), ("missing", &missing.join(", "))],
    );
    finalize_simple(prompt, log, intent, response_link, &body, 0.5)
}

/// Compose one piece of genre writing from a described change or intent
/// (issue #1174): a Conventional Commits line for the `commit_message` genre
/// and a business email for the `email` genre, each from the slot grammar its
/// seed styleguide declares. A missing required slot is named through the
/// genre-incomplete template rather than fabricated.
pub fn handle_genre_writing(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let head = command_head(prompt);
    let head_normalized = crate::web_engine_core::normalize_prompt(head);
    let (genre, _) = genre_cues(GENRE_PATH)
        .into_iter()
        .find(|(_, cue)| head_cue_matches(&head_normalized, cue))?;
    let payload = free_text_payload(prompt)?;
    let records = root_records(GENRE_PATH);
    let guide = records
        .iter()
        .find(|record| record.name == "genre" && record.find_child_value("name") == genre)?;
    let language = crate::language::detect(&payload).slug();
    let intent = format!("{INTENT_GENRE_PREFIX}_{genre}");
    let response_link = format!("response:{INTENT_GENRE_PREFIX}_{genre}");
    let styleguide = guide.find_child_value("styleguide").to_owned();
    if !styleguide.is_empty() {
        log.append("genre_styleguide", styleguide);
    }
    let requires: Vec<String> = record_values(guide, "requires");

    match genre.as_str() {
        "commit_message" => {
            let tokens: Vec<&str> = payload.split_whitespace().collect();
            let slots: Vec<(String, String)> = guide
                .children
                .iter()
                .filter(|child| child.name == "slot")
                .filter_map(|slot| {
                    let cue = slot.find_child_value("cue");
                    let value = slot.find_child_value("value");
                    (!cue.is_empty() && !value.is_empty())
                        .then(|| (cue.to_lowercase(), value.to_owned()))
                })
                .collect();
            let articles: Vec<String> = record_values(guide, "article")
                .into_iter()
                .map(|article| article.to_lowercase())
                .collect();
            let preposition = guide
                .find_child_value("component_preposition")
                .to_lowercase();
            let verb_core = tokens
                .first()
                .map(|token| token_core(token).1.to_lowercase());
            let commit_type = verb_core.as_deref().and_then(|verb| {
                slots
                    .iter()
                    .find(|(cue, _)| *cue == verb)
                    .map(|(_, value)| value.clone())
            });
            let preposition_index = if preposition.is_empty() {
                None
            } else {
                tokens
                    .iter()
                    .rposition(|token| token_core(token).1.to_lowercase() == preposition)
            };
            let effect_range: Vec<&str> = preposition_index.map_or_else(
                || tokens[1..].to_vec(),
                |preposition_index| {
                    tokens
                        .get(1..preposition_index)
                        .map_or_else(Vec::new, <[&str]>::to_vec)
                },
            );
            let effect_tokens: Vec<&str> = effect_range
                .into_iter()
                .skip_while(|token| {
                    articles
                        .iter()
                        .any(|article| *article == token_core(token).1.to_lowercase())
                })
                .collect();
            let component: Option<String> = preposition_index.and_then(|preposition_index| {
                tokens[preposition_index + 1..]
                    .iter()
                    .find(|token| {
                        let core = token_core(token).1;
                        !core.is_empty()
                            && !articles
                                .iter()
                                .any(|article| *article == core.to_lowercase())
                    })
                    .map(|token| token_core(token).1.to_owned())
            });
            let missing_slots = unsatisfied_requirements(
                &requires,
                &[commit_type.is_none(), effect_tokens.is_empty()],
            );
            if !missing_slots.is_empty() {
                return Some(finalize_genre_incomplete(
                    prompt,
                    log,
                    &intent,
                    &response_link,
                    language,
                    &genre,
                    &missing_slots,
                ));
            }
            let imperative = verb_core.unwrap_or_default();
            let effect = effect_tokens.join(" ");
            let body = component.map_or_else(
                || {
                    fill_template(
                        guide.find_child_value("template_no_scope"),
                        &[
                            ("type", commit_type.as_deref().unwrap_or_default()),
                            ("imperative", &imperative),
                            ("effect", &effect),
                        ],
                    )
                },
                |component| {
                    fill_template(
                        guide.find_child_value("template"),
                        &[
                            ("type", commit_type.as_deref().unwrap_or_default()),
                            ("component", &component),
                            ("imperative", &imperative),
                            ("effect", &effect),
                        ],
                    )
                },
            );
            log.append("genre_composition", body.clone());
            Some(finalize_simple(
                prompt,
                log,
                &intent,
                &response_link,
                &body,
                0.8,
            ))
        }
        "email" => {
            let tokens: Vec<&str> = payload.split_whitespace().collect();
            let preposition = guide
                .find_child_value("recipient_preposition")
                .to_lowercase();
            let possessives: Vec<String> = record_values(guide, "possessive")
                .into_iter()
                .map(|possessive| possessive.to_lowercase())
                .collect();
            let head_tokens: Vec<&str> = head.split_whitespace().collect();
            let preposition_index = if preposition.is_empty() {
                None
            } else {
                head_tokens
                    .iter()
                    .position(|token| token_core(token).1.to_lowercase() == preposition)
            };
            let recipient: Option<String> = preposition_index.and_then(|preposition_index| {
                head_tokens[preposition_index + 1..]
                    .iter()
                    .find(|token| {
                        let core = token_core(token).1;
                        !core.is_empty()
                            && !possessives
                                .iter()
                                .any(|possessive| *possessive == core.to_lowercase())
                    })
                    .map(|token| token_core(token).1.to_owned())
            });
            let reason_cue = guide.find_child_value("reason_cue").to_lowercase();
            let reason_index = if reason_cue.is_empty() {
                None
            } else {
                tokens
                    .iter()
                    .position(|token| token_core(token).1.to_lowercase() == reason_cue)
            };
            let intent_range: Vec<&str> = reason_index.map_or_else(
                || tokens.clone(),
                |reason_index| tokens[..reason_index].to_vec(),
            );
            let intent_text = intent_range.join(" ");
            if intent_text.is_empty() {
                let missing_slots = unsatisfied_requirements(&requires, &[true]);
                return Some(finalize_genre_incomplete(
                    prompt,
                    log,
                    &intent,
                    &response_link,
                    language,
                    &genre,
                    &missing_slots,
                ));
            }
            let reason: Option<String> =
                reason_index.map(|reason_index| tokens[reason_index + 1..].join(" "));
            let weekday_role = guide.find_child_value("weekday_role").to_owned();
            let weekday: Option<String> = if weekday_role.is_empty() {
                None
            } else {
                crate::seed::lexicon()
                    .words_for_role_in_languages(&weekday_role, &[language])
                    .iter()
                    .find_map(|weekday| {
                        tokens
                            .iter()
                            .find(|token| token_core(token).1.to_lowercase() == *weekday)
                            .map(|token| capitalize_first(token_core(token).1))
                    })
            };
            let mut parts: Vec<String> = Vec::new();
            for part in record_values(guide, "part") {
                match part.as_str() {
                    "greeting" => {
                        let greeting = recipient.as_ref().map_or_else(
                            || guide.find_child_value("greeting_default").to_owned(),
                            |recipient| {
                                fill_template(
                                    guide.find_child_value("greeting"),
                                    &[("recipient", recipient)],
                                )
                            },
                        );
                        parts.push(greeting);
                    }
                    "intent_frame" => {
                        parts.push(fill_template(
                            guide.find_child_value("intent_frame"),
                            &[("intent", &intent_text)],
                        ));
                    }
                    "reason_frame" => {
                        if let Some(reason) = &reason {
                            parts.push(fill_template(
                                guide.find_child_value("reason_frame"),
                                &[("reason", reason)],
                            ));
                        }
                    }
                    "date_frame" => {
                        if let Some(weekday) = &weekday {
                            parts.push(fill_template(
                                guide.find_child_value("date_frame"),
                                &[("date", weekday)],
                            ));
                        }
                    }
                    "closing" => {
                        parts.push(guide.find_child_value("closing").to_owned());
                    }
                    _ => {}
                }
            }
            let body = parts.join("\n\n");
            log.append("genre_composition", body.clone());
            Some(finalize_simple(
                prompt,
                log,
                &intent,
                &response_link,
                &body,
                0.8,
            ))
        }
        _ => None,
    }
}
