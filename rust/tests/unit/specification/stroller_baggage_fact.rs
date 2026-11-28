//! The reported baggage questions read the current publisher policy (issue #467).

use formal_ai::event_log::EventLog;
use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceTransport};
use formal_ai::try_fact_lookup_with_client;
use formal_ai::web_engine_core::normalize_prompt;

const SOURCE: &str = "https://www.airindia.com/in/en/travel-information/baggage-guidelines/checked-baggage-allowance.html";
const EXCERPT: &str = "Infants are entitled to one collapsible stroller/carrycot/infant car seat.";
const RUSSIAN_ANSWER: &str = "В официальных правилах сказано: «{excerpt}» Источник: {source}";

struct BaggagePage;

impl SourceTransport for BaggagePage {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        assert_eq!(url, SOURCE);
        Ok(
            format!("<h1>Checked Baggage</h1><h2>Domestic flights</h2><ul><li>{EXCERPT}</li></ul>")
                .into_bytes(),
        )
    }
}

#[test]
fn reported_stroller_questions_require_a_publisher_capture_in_each_language() {
    let cache =
        std::env::temp_dir().join(format!("formal-ai-baggage-policy-{}", std::process::id()));
    let ask = |prompt: &str, client: &CachedSourceClient<BaggagePage>| {
        try_fact_lookup_with_client(
            prompt,
            &normalize_prompt(prompt),
            &mut EventLog::new(),
            client,
        )
    };
    let reported = "Даёт airindia бесплатную детскую коляску к багажу?";
    let offline = CachedSourceClient::new(&cache, BaggagePage);
    assert!(ask(reported, &offline).is_none());
    let online = CachedSourceClient::new(&cache, BaggagePage).with_online(true);
    let expected_answer = |wording: &str| {
        wording
            .replace("{excerpt}", EXCERPT)
            .replace("{source}", SOURCE)
    };
    for (prompt, wording) in [
        (reported, RUSSIAN_ANSWER),
        (
            "Можно ли бесплатно сдать детскую коляску на рейсе Air India?",
            RUSSIAN_ANSWER,
        ),
        (
            "Does Air India allow a free stroller for an infant?",
            "The official policy says: “{excerpt}” Source: {source}",
        ),
        (
            "क्या एयर इंडिया शिशु के लिए मुफ्त बेबी स्ट्रॉलर सामान में देती है?",
            "आधिकारिक नीति में लिखा है: “{excerpt}” स्रोत: {source}",
        ),
        (
            "印度航空允许婴儿推车免费作为行李吗?",
            "官方政策写道：“{excerpt}”来源：{source}",
        ),
    ] {
        let answer = ask(prompt, &online).unwrap_or_else(|| panic!("unanswered: {prompt}"));
        assert_eq!(answer.intent, "fact_lookup");
        assert_eq!(answer.answer, expected_answer(wording), "{prompt}");
        assert!(answer.evidence_links.contains(&format!("source:{SOURCE}")));
        assert!(answer.evidence_links.iter().any(|link| {
            link.starts_with(&format!("source:http:{SOURCE}:")) && link.contains("sha256=")
        }));
        assert!(
            answer
                .evidence_links
                .iter()
                .any(|link| link == "wikidata:Q69906")
        );
    }
    let replayed = ask(reported, &offline).expect("reuse the verified publisher capture");
    assert_eq!(replayed.answer, expected_answer(RUSSIAN_ANSWER));
    assert!(
        replayed
            .evidence_links
            .iter()
            .any(|link| link.contains("cached=true"))
    );
    std::fs::remove_dir_all(cache).expect("remove temporary captures");
}
