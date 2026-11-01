//! Shared source/proof vectors; native CI only.
use super::derive_own_key_dictionary_repair;
use serde_json::Value;
#[test]
fn shared_dictionary_binding_cases() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../fixtures/own-key-dictionary-repair-cases.json"
    ))
    .expect("fixture");
    for entry in fixture["cases"].as_array().expect("cases") {
        let owner = entry["owner"].as_str().expect("owner");
        let source = entry["source"].as_str().expect("source");
        let result = derive_own_key_dictionary_repair(owner, source);
        if let Some(reason) = entry["gap"].as_str() {
            assert_eq!(result.expect_err("refusal"), reason, "{}", entry["name"]);
            continue;
        }
        let result = result.expect("source-derived candidate");
        let proof = &result["verification"];
        for (name, expected) in [
            ("writeCount", "writes"),
            ("readCount", "reads"),
            ("spreadCount", "spreads"),
        ] {
            assert_eq!(proof[name], entry[expected]);
        }
        for name in [
            "completeConcreteResidual",
            "initializerTriviaPreserved",
            "independentGrammarReparse",
            "exactInverse",
        ] {
            assert_eq!(proof[name], true);
        }
        let span = result["binding"]["initializerSpan"]
            .as_array()
            .expect("span");
        let start = usize::try_from(span[0].as_u64().expect("start")).expect("finite start");
        let end = usize::try_from(span[1].as_u64().expect("end")).expect("finite end");
        let candidate = result["candidate"]["source"].as_str().expect("candidate");
        assert_eq!(&candidate[..start], &source[..start]);
        assert!(candidate.ends_with(&source[end..]));
        if entry["preserveTrivia"] == true {
            let trivia = |text: &str| {
                text.chars()
                    .filter(|character| crate::meta_reasoner::text::is_js_whitespace(*character))
                    .collect::<String>()
            };
            assert_eq!(trivia(candidate), trivia(source));
        }
        assert_eq!(result["authored"], false);
        assert_eq!(result["verified"], false);
        assert_eq!(result["independentAcceptance"], false);
    }
}
