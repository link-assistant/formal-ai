use super::*;

pub fn try_translation(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let target = detect_target_language(normalized);
    let unquoted_surface = extract_unquoted_translation_surface(prompt);
    // Issue #386: recognise a translation command by *meaning*, not by hardcoded
    // verbs. The translation-action stems live once in
    // data/seed/meanings-translation.lino; this code knows the concept and the
    // head-initial/head-final typology the `translate` gloss documents.
    // Clause-initial English/Russian commands are matched as a prefix; head-final
    // Hindi/Chinese place the verb later, so they are matched anywhere but gated
    // by a target marker (as before) to avoid firing on an incidental verb noun.
    let lexicon = crate::seed::lexicon();
    let head_initial_command = lexicon
        .words_for_role_in_languages(crate::seed::ROLE_TRANSLATION_ACTION, &["en", "ru"])
        .iter()
        .any(|stem| normalized.starts_with(stem.as_str()));
    let head_final_command = target.is_some()
        && lexicon
            .words_for_role_in_languages(crate::seed::ROLE_TRANSLATION_ACTION, &["hi", "zh"])
            .iter()
            .any(|stem| normalized.contains(stem.as_str()));
    let source_first_command = crate::translation::prompt::is_source_first_translation_request(
        normalized,
        target.is_some(),
        unquoted_surface.is_some(),
    );
    // Issue #386: the define-in-Links-Notation request is recognised by *meaning*
    // too, not by literal verbs and format strings. The imperative verb lives once
    // as the `definition_command` meaning and the target-format phrases as
    // `links_notation_format`, both in data/seed/meanings-translation.lino. Exactly
    // as the original recogniser, only the English verb is scanned (a clause-initial
    // prefix with a trailing space, so `defined`/`definition` never trigger it) and
    // the English and Russian format markers are scanned as space-prefixed
    // substrings; the Hindi and Chinese surfaces are carried for coverage only.
    let is_define_in_links = || {
        lexicon
            .words_for_role_in_languages(crate::seed::ROLE_DEFINITION_COMMAND, &["en"])
            .iter()
            .any(|verb| normalized.starts_with(format!("{verb} ").as_str()))
            && (extract_quoted_phrase(prompt).is_some() || extract_backticked(prompt).is_some())
            && lexicon
                .words_for_role_in_languages(crate::seed::ROLE_LINKS_NOTATION_FORMAT, &["en", "ru"])
                .iter()
                .any(|marker| normalized.contains(format!(" {marker}").as_str()))
    };
    let define_in_links = is_define_in_links();
    let is_translation_request =
        head_initial_command || head_final_command || source_first_command || define_in_links;
    if !is_translation_request {
        return None;
    }

    let mut source = detect_source_language(normalized);
    if source.is_none() {
        source = Some(infer_source_from_prompt(prompt));
    }

    let backticked = extract_backticked(prompt);

    if let Some(code) = &backticked {
        let detected = detect_program_languages(normalized)
            .or_else(|| infer_program_languages_from_code(code, normalized));
        if let Some((source_lang, target_lang)) = detected {
            let translated = translate_program(code, source_lang, target_lang);
            let body = format!(
                "Translated `{code}` from {source_lang} to {target_lang}:\n\n```{target_lang}\n{translated}\n```"
            );
            log.append("language_from", source_lang.to_owned());
            log.append("language_to", target_lang.to_owned());
            let meaning_id = stable_id("meaning", &normalize_code_meaning(code));
            log.append("meaning", meaning_id);
            let intent = format!("translate_{source_lang}_to_{target_lang}");
            return Some(finalize_simple(
                prompt,
                log,
                &intent,
                "response:translate_code",
                &body,
                1.0,
            ));
        }
    }

    // Prefer an explicitly quoted fragment (`Translate "apple" to Russian`).
    // When the user omits the quotes (`translate apple to russian`),
    // fall back to a structural extraction of the substring between the
    // verb and the target preposition so the Wiktionary pipeline still
    // receives a non-empty surface. See issue #216.
    let surface = extract_quoted_phrase(prompt)
        .or(unquoted_surface)
        .unwrap_or_default();
    let source_slug = source.unwrap_or("en");
    let target_slug = target.unwrap_or("en");

    log.append("language_from", source_slug.to_owned());
    log.append("language_to", target_slug.to_owned());

    // Run the real Wiktionary + Wikidata translation pipeline. The pipeline
    // returns a `MeaningId` that we publish into the trace verbatim, so two
    // surfaces that resolve to the same Wikidata Q-item end up with the
    // same `meaning:...` id regardless of source language.
    let pipeline_result =
        crate::solver_helpers::translate_surface_detailed(&surface, source_slug, target_slug);

    let (target_surface, meaning_id, translation_gap) = if let Ok(translation) = pipeline_result {
        let target_surface = translation.primary_surface().map(str::to_owned);
        let gap = target_surface.is_none();
        let mut meaning_id = translation.meaning.slug();
        if define_in_links && !translation.meaning.is_wikidata_backed() {
            if let Some(seed_meaning) =
                crate::translation::seed_meaning_for_surface(&surface, source_slug)
            {
                meaning_id = seed_meaning.slug();
            }
        }
        (target_surface, meaning_id, gap)
    } else {
        // Fallback: hash the surface fragment so the trace still has a
        // stable id. The pipeline error itself is not propagated to the
        // user; the response below reports the gap without manufacturing
        // a fake target-language surface.
        let surface_meaning = if surface.is_empty() {
            prompt
        } else {
            surface.as_str()
        };
        let id = stable_id("meaning", &normalize_meaning(surface_meaning));
        (None, id, true)
    };
    log.append("meaning", meaning_id);
    if translation_gap && !surface.is_empty() {
        log.append("translation_gap", surface.clone());
    }

    let body = target_surface.map_or_else(
        || render_translation_gap(&surface, source_slug, target_slug),
        |raw_target| {
            let translated_surface =
                crate::translation::match_source_formatting(&raw_target, &surface);
            log.append("surface", translated_surface.clone());
            if surface.is_empty() {
                translated_surface
            } else {
                format!("\"{translated_surface}\"")
            }
        },
    );
    let intent = format!("translate_{source_slug}_to_{target_slug}");
    Some(finalize_simple(
        prompt,
        log,
        &intent,
        "response:translate",
        &body,
        1.0,
    ))
}
