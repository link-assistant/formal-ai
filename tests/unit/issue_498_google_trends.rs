use std::collections::BTreeSet;

use formal_ai::{
    google_trends_catalog, parse_google_trends_rss, GoogleTrendPromptVariant,
    GOOGLE_TRENDS_TOP_LIMIT,
};

const SAMPLE_RSS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<rss xmlns:ht="https://trends.google.com/trending/rss" version="2.0">
  <channel>
    <title>Daily Search Trends</title>
    <link>https://trends.google.com/trending/rss?geo=US</link>
    <item>
      <title>blue jays</title>
      <ht:approx_traffic>2000+</ht:approx_traffic>
      <link>https://trends.google.com/trending/rss?geo=US</link>
      <pubDate>Wed, 8 Jul 2026 12:50:00 -0700</pubDate>
      <ht:news_item>
        <ht:news_item_title>Blue Jays vs. Giants Game Thread</ht:news_item_title>
        <ht:news_item_url>https://sports.yahoo.com/articles/blue-jays-vs-giants-game-193143634.html</ht:news_item_url>
        <ht:news_item_source>Yahoo Sports</ht:news_item_source>
      </ht:news_item>
    </item>
    <item>
      <title>grok 4.5</title>
      <ht:approx_traffic>500+</ht:approx_traffic>
      <link>https://trends.google.com/trending/rss?geo=US</link>
      <pubDate>Wed, 8 Jul 2026 12:50:00 -0700</pubDate>
    </item>
  </channel>
</rss>"#;

#[test]
fn google_trends_rss_is_converted_to_ranked_topics() {
    let snapshot = parse_google_trends_rss(SAMPLE_RSS, "US", "ru")
        .expect("the RSS converter should parse the Google Trends feed shape");

    assert_eq!(snapshot.source, "google_trends_rss");
    assert_eq!(snapshot.geo, "US");
    assert_eq!(snapshot.locale, "ru");
    assert_eq!(snapshot.topics.len(), 2);

    let first = &snapshot.topics[0];
    assert_eq!(first.rank, 1);
    assert_eq!(first.query, "blue jays");
    assert_eq!(first.approx_traffic.as_deref(), Some("2000+"));
    assert_eq!(
        first.pub_date.as_deref(),
        Some("Wed, 8 Jul 2026 12:50:00 -0700")
    );
    assert_eq!(
        first.news_items.first().map(|item| item.source.as_str()),
        Some("Yahoo Sports"),
    );
}

#[test]
fn checked_in_google_trends_catalog_covers_top_ten_in_all_supported_languages() {
    let catalog = google_trends_catalog();

    assert_eq!(
        catalog.topics.len(),
        GOOGLE_TRENDS_TOP_LIMIT,
        "the committed snapshot should keep exactly the top 10 Trends topics",
    );
    assert_eq!(catalog.geo, "US");
    assert_eq!(catalog.locale, "ru");
    assert_eq!(
        catalog.source_url,
        "https://trends.google.com/trending/rss?geo=US"
    );

    let mut seen_ranks = BTreeSet::new();
    for topic in &catalog.topics {
        assert!(
            seen_ranks.insert(topic.rank),
            "rank should be unique: {topic:?}"
        );
        assert!(!topic.query.trim().is_empty());
        assert_eq!(
            topic.prompts.len(),
            8,
            "each topic should have two request variations in each of four supported languages",
        );
        assert_eq!(
            topic.answered.len(),
            topic.prompts.len(),
            "every prompt variation should be answered through the normal Formal AI path",
        );

        let languages: BTreeSet<_> = topic
            .prompts
            .iter()
            .map(|prompt| prompt.language.as_str())
            .collect();
        let language = "en";
        assert!(languages.contains(language), "English prompts are required");
        let language = "ru";
        assert!(languages.contains(language), "Russian prompts are required");
        let language = "hi";
        assert!(languages.contains(language), "Hindi prompts are required");
        let language = "zh";
        assert!(languages.contains(language), "Chinese prompts are required");
        assert_eq!(languages.len(), 4, "only supported languages are expected");

        assert!(
            topic
                .prompts
                .iter()
                .any(GoogleTrendPromptVariant::is_trends_context_request),
            "topic should include a trend-specific request variation: {topic:?}",
        );

        for answered in &topic.answered {
            assert!(answered.prompt.ends_with('?'));
            assert!(!answered.answer.trim().is_empty());
            assert!(
                answered
                    .evidence_links
                    .iter()
                    .any(|link| link.starts_with("trace:")),
                "answers should preserve standard trace evidence: {answered:?}",
            );
        }
    }
}
