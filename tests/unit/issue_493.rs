use std::{fs, path::Path};

use formal_ai::{assess_market_price_claims, extract_market_price_claims, FormalAiEngine};

const ISSUE_493_OCR_TEXT: &str = "$ETH
ETH in 2021: $1,700
ETH in 2022: $1,700
ETH in 2023: $1,700
ETH in 2024: $1,700
ETH in 2025: $1,700
ETH in 2026: $1,700
ETH before BitMine buying: $1,700
ETH after BitMine buying: $1,700
ETH before ETF approval: $1,700
ETH after ETF approval: $1,700
ETH during anti-crypto President: $1,700
ETH during pro-crypto President: $1,700
ETH before US-Iran war: $1,700
ETH after US-Iran war: $1,700

Performance of $ETH is an absolute joke.";

#[test]
fn market_price_claim_parser_extracts_repeated_eth_year_claims_from_ocr_text() {
    let claims = extract_market_price_claims(ISSUE_493_OCR_TEXT);

    assert!(
        claims.iter().any(|claim| claim.asset == "ETH"
            && claim.period == "2024"
            && (claim.claimed_price - 1700.0).abs() < f64::EPSILON),
        "the screenshot OCR text should expose the ETH 2024 $1,700 claim: {claims:?}",
    );
}

#[test]
fn market_price_claim_parser_handles_asset_symbol_before_price_symbol() {
    let claims = extract_market_price_claims("Buy $ETH in 2024 at $1,700.");

    assert!(
        claims.iter().any(|claim| claim.asset == "ETH"
            && claim.period == "2024"
            && (claim.claimed_price - 1700.0).abs() < f64::EPSILON),
        "the parser should skip the ticker dollar marker and keep scanning for the price: {claims:?}",
    );
}

#[test]
fn market_price_claim_parser_accepts_supported_language_eth_aliases() {
    struct Case {
        language: &'static str,
        sample: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            sample: "Ethereum in 2024: $1,700",
        },
        Case {
            language: "ru",
            sample: "эфириум в 2024: $1,700",
        },
        Case {
            language: "hi",
            sample: "एथेरियम 2024: $1,700",
        },
        Case {
            language: "zh",
            sample: "以太坊 2024: $1,700",
        },
    ];

    for case in cases {
        let claims = extract_market_price_claims(case.sample);

        assert!(
            claims.iter().any(|claim| claim.asset == "ETH"
                && claim.period == "2024"
                && (claim.claimed_price - 1700.0).abs() < f64::EPSILON),
            "{} should extract the localized ETH 2024 price claim: {:?}",
            case.language,
            claims,
        );
    }
}

#[test]
fn market_price_assessment_contradicts_eth_2024_1700_claim_with_market_data() {
    let claims = extract_market_price_claims(ISSUE_493_OCR_TEXT);
    let assessments = assess_market_price_claims(&claims);
    let eth_2024 = assessments
        .iter()
        .find(|assessment| assessment.claim.asset == "ETH" && assessment.claim.period == "2024")
        .expect("ETH 2024 claim should have a market-data assessment");

    assert_eq!(eth_2024.status, "contradicted");
    assert_eq!(eth_2024.source_label, "Binance ETHUSDT daily klines");
    assert!(
        (eth_2024.observed_min_price - 2100.0).abs() < f64::EPSILON,
        "expected 2024 ETH minimum of 2100.0, got {}",
        eth_2024.observed_min_price,
    );
    assert_eq!(eth_2024.observed_min_date, "2024-01-03");
    assert!(
        eth_2024.statement_plan.assessment.posterior.get() < 0.5,
        "contradicting market data should lower the statement below probable: {:?}",
        eth_2024.statement_plan.assessment,
    );
}

#[test]
fn issue_493_multiline_ocr_fact_check_catches_false_eth_2024_price_claim() {
    let prompt = format!(
        "Verify factual accuracy of this attached image\n\n\
Attached files:\n\
1. eth-claim.jpg (image/jpeg, 51.0 KB)\n\
OCR text: {ISSUE_493_OCR_TEXT}"
    );

    let response = FormalAiEngine.answer(&prompt);

    assert_eq!(response.intent, "document_originality_check");
    assert!(
        response.evidence_links.iter().any(|link| {
            link == "market_price_claim:assessment:asset=ETH period=2024 claimed=1700.00 status=contradicted source=binance_ethusdt_1d_2024 min=2100.00 min_date=2024-01-03 max=4107.80 max_date=2024-12-16 posterior=0.030000"
        }),
        "the OCR verification path should log the contradiction for ETH in 2024: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .answer
            .contains("ETH in 2024: $1,700 is contradicted"),
        "the answer should summarize the caught false claim, got: {}",
        response.answer,
    );
}

#[test]
fn issue_493_tesseract_ocr_result_contains_the_key_price_claim() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/case-studies/issue-493/raw-data/tesseract-issue-screenshot-ocr.json");
    let ocr = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));

    assert!(
        ocr.contains("ETH in 2024: $1,700"),
        "the preserved Tesseract result should transcribe the key ETH 2024 claim: {ocr}",
    );
    assert!(
        ocr.contains("\"containsEth2024Claim\": true"),
        "the OCR experiment should record a passing structured check: {ocr}",
    );
}
