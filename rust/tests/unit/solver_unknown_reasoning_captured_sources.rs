use super::*;
use crate::relative_meta_logic::SourceTier;

fn captured_sense(surface: &str, language: &str) -> ConceptSense {
    let rows: serde_json::Value = serde_json::from_str(include_str!(
        "../fixtures/issue-1138-b1/expected-senses.json"
    ))
    .expect("the committed concept capture expectations are valid JSON");
    let row = rows
        .as_array()
        .expect("the capture expectations are a list")
        .iter()
        .find(|row| row["surface"] == surface && row["language"] == language)
        .expect("the held-out captured sense exists");
    let field = |name: &str| {
        row[name]
            .as_str()
            .expect("a capture string field")
            .to_owned()
    };
    ConceptSense {
        surface: field("surface"),
        lemma: field("lemma"),
        language: field("language"),
        gloss: field("gloss"),
        part_of_speech: field("partOfSpeech"),
        synonyms: Vec::new(),
        source_id: field("sourceId"),
        source_url: field("sourceUrl"),
        sha256: field("sha256"),
        fetched_at: String::new(),
        cached: true,
        tier: SourceTier::IndependentCorroboration,
        license_name: field("licenseName"),
        license_url: field("licenseUrl"),
        depth: 0,
    }
}

fn offline_config() -> UnknownReasoningConfig {
    UnknownReasoningConfig {
        questioning_rigor: 0.4,
        offline: true,
        agent_mode: false,
    }
}

#[test]
fn unparsed_spanish_question_keeps_its_full_body_despite_a_retrieved_word() {
    let prompt = "¿Cómo funciona la fotosíntesis submarina de xyzzy?";
    let mut log = EventLog::new();
    let answer = answer_unknown_prompt(
        prompt,
        Language::Spanish,
        &mut log,
        offline_config(),
        &[captured_sense("isograma", "es")],
    );
    assert_eq!(answer.intent, "unknown");
    assert_eq!(
        answer.answer,
        "No pude determinar `¿Cómo funciona la fotosíntesis submarina de xyzzy` a partir de la \
         memoria local de Links Notation, el conocimiento público en caché ni la caché de fuentes, \
         y no puedo inferir una respuesta verificada. Registré en la traza los intentos fallidos \
         de recopilación.\n\n\
         Si el razonamiento sigue sin resolverlo y hace falta un hecho semilla compartido o una \
         regla de enlaces en Links Notation, usa Report issue con la traza. Para conservar una \
         regla local del diálogo, exporta la memoria o enséñala con `When I say ... answer ...`; \
         revisa las rutas con `List behavior rules` y `Show behavior rule unknown`.\n\n\
         Detecté un fallo mientras trabajaba en esta solicitud. ¿Quieres que prepare un informe \
         del problema con el contexto de diagnóstico? Responde `Report issue`."
    );
    assert!(log.events().iter().any(|event| {
        event.kind == "reasoning:gather_result"
            && event.payload == "external_source:no_verified_answer"
    }));
    assert!(!log.events().iter().any(|event| {
        event.kind == "reasoning:gather_result" && event.payload.starts_with("external_source:hit:")
    }));
}

#[test]
fn a_word_in_an_unparsed_question_is_still_only_a_word_definition() {
    let prompt = "¿Cómo funciona un isograma?";
    let mut log = EventLog::new();
    let answer = answer_unknown_prompt(
        prompt,
        Language::Spanish,
        &mut log,
        offline_config(),
        &[captured_sense("isograma", "es")],
    );
    assert_eq!(answer.intent, "unknown");
    assert_eq!(
        answer.answer,
        "No pude determinar `¿Cómo funciona un isograma` a partir de la \
         memoria local de Links Notation, el conocimiento público en caché ni la caché de fuentes, \
         y no puedo inferir una respuesta verificada. Registré en la traza los intentos fallidos \
         de recopilación.\n\n\
         Si el razonamiento sigue sin resolverlo y hace falta un hecho semilla compartido o una \
         regla de enlaces en Links Notation, usa Report issue con la traza. Para conservar una \
         regla local del diálogo, exporta la memoria o enséñala con `When I say ... answer ...`; \
         revisa las rutas con `List behavior rules` y `Show behavior rule unknown`.\n\n\
         Detecté un fallo mientras trabajaba en esta solicitud. ¿Quieres que prepare un informe \
         del problema con el contexto de diagnóstico? Responde `Report issue`."
    );
    assert!(answer.answer.contains("`¿Cómo funciona un isograma`"));
}

#[test]
fn supported_definition_keeps_its_attributed_retrieved_answer() {
    let sense = captured_sense("isogram", "en");
    let mut log = EventLog::new();
    let answer = answer_unknown_prompt(
        "What is an isogram?",
        Language::English,
        &mut log,
        offline_config(),
        std::slice::from_ref(&sense),
    );
    assert_eq!(answer.intent, "concept_lookup");
    assert_eq!(
        answer.answer,
        "isogram (isogram): a line drawn on a map connecting points having the same numerical value of some variable\n\nSource: https://en-word.net/api/lemma/isogram (sha256 ae8c98f774fd255a; CC BY 4.0)"
    );
    assert!(answer.answer.contains(&sense.gloss));
    assert!(answer.answer.contains(&sense.source_url));
    assert!(answer.answer.contains(&sense.sha256[..16]));
}

#[test]
fn unsupported_bare_term_keeps_its_attributed_retrieved_answer() {
    let sense = captured_sense("isograma", "es");
    let mut log = EventLog::new();
    let answer = answer_unknown_prompt(
        "isograma",
        Language::Spanish,
        &mut log,
        offline_config(),
        std::slice::from_ref(&sense),
    );
    assert_eq!(answer.intent, "concept_lookup");
    assert_eq!(
        answer.answer,
        "isograma (Heterograma): Un heterograma es una palabra o frase que no contiene ninguna letra repetida. Esta propiedad fue descrita por Dmitri Borgmann en su libro Language on Vacation: An Olio of Orthographical Oddities, donde intentó encontrar la palabra más larga que cumpliera esta condición.\n\nFuente: https://es.wikipedia.org/api/rest_v1/page/summary/Isograma (sha256 7fbc0bf57dce4de1; CC BY-SA 4.0)"
    );
    assert!(answer.answer.contains(&sense.gloss));
    assert!(answer.answer.contains(&sense.source_url));
}
