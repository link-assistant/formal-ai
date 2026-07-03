//! Market-price reference registry loaded from seed data.
//!
//! Issue #493 asks the verifier to catch false market-price claims for the whole
//! class of assets, periods, and languages — not one hardcoded example. Every
//! natural-language surface (asset aliases) and every observed price range lives
//! in `data/seed/market-price-references.lino` and is loaded here, so the
//! recogniser stays meaning-grounded and data-driven (issue #386 convention: no
//! per-language phrase list ever lives in Rust). Each asset is grounded in a
//! Wikidata entity and carries one reference per covered period with the daily
//! candle min/max captured from an original first-party source.

use std::sync::OnceLock;

use super::parser::{parse_lino, LinoNode};
use super::MARKET_PRICE_REFERENCES_LINO;

/// One market asset with its aliases, grounding, and per-period references.
#[derive(Debug, Clone, PartialEq)]
pub struct MarketPriceAsset {
    /// Canonical ticker, e.g. `ETH`.
    pub ticker: String,
    /// Human label for the asset, e.g. `Ethereum`.
    pub label: String,
    /// Wikidata entity id grounding the asset, e.g. `Q16783523`.
    pub grounded_in: String,
    /// Quote currency used by the reference source, e.g. `USDT`.
    pub quote_currency: String,
    /// Natural-language and ticker aliases accepted for this asset, across every
    /// supported language, in declaration order.
    pub aliases: Vec<String>,
    /// One reference per covered period, in declaration order.
    pub references: Vec<MarketPricePeriod>,
}

/// A versioned observed price range for one asset over one period.
#[derive(Debug, Clone, PartialEq)]
pub struct MarketPricePeriod {
    /// Covered period, e.g. `2024`.
    pub period: String,
    /// Stable source slug for trace logs.
    pub source_id: String,
    /// Human-readable source label.
    pub source_label: String,
    /// Source URL used for the captured raw data.
    pub source_url: String,
    /// Minimum observed price in the covered period.
    pub observed_min_price: f64,
    /// Date of the minimum observed price.
    pub observed_min_date: String,
    /// Maximum observed price in the covered period.
    pub observed_max_price: f64,
    /// Date of the maximum observed price.
    pub observed_max_date: String,
}

/// The market-price reference registry, parsed once from seed data.
#[must_use]
pub fn market_price_assets() -> &'static [MarketPriceAsset] {
    static REGISTRY: OnceLock<Vec<MarketPriceAsset>> = OnceLock::new();
    REGISTRY.get_or_init(parse_market_price_assets).as_slice()
}

fn parse_market_price_assets() -> Vec<MarketPriceAsset> {
    let tree = parse_lino(MARKET_PRICE_REFERENCES_LINO);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "market_price_references")
        .expect("data/seed/market-price-references.lino must declare market_price_references");
    root.children
        .iter()
        .filter(|node| node.name == "asset")
        .map(parse_asset)
        .collect()
}

fn parse_asset(node: &LinoNode) -> MarketPriceAsset {
    MarketPriceAsset {
        ticker: node.id.clone(),
        label: node.find_child_value("label").to_owned(),
        grounded_in: node.find_child_value("grounded-in").to_owned(),
        quote_currency: node.find_child_value("quote-currency").to_owned(),
        aliases: parse_asset_aliases(node),
        references: node
            .children
            .iter()
            .filter(|child| child.name == "reference")
            .map(parse_reference)
            .collect(),
    }
}

/// Collect every alias surface across all `lexeme <lang>` blocks in declaration
/// order. The verifier matches on meaning, so aliases from every language share
/// one flat list; the per-language grouping in the seed file keeps the
/// multilingual coverage auditable.
fn parse_asset_aliases(node: &LinoNode) -> Vec<String> {
    let mut aliases = Vec::new();
    for lexeme in node.children.iter().filter(|child| child.name == "lexeme") {
        for surface in lexeme
            .children
            .iter()
            .filter(|child| child.name == "surface")
        {
            let text = surface.find_child_value("text");
            if !text.is_empty() && !aliases.iter().any(|existing| existing == text) {
                aliases.push(text.to_owned());
            }
        }
    }
    aliases
}

fn parse_reference(node: &LinoNode) -> MarketPricePeriod {
    MarketPricePeriod {
        period: node.id.clone(),
        source_id: node.find_child_value("source-id").to_owned(),
        source_label: node.find_child_value("source-label").to_owned(),
        source_url: node.find_child_value("source-url").to_owned(),
        observed_min_price: parse_price(node.find_child_value("observed-min-price")),
        observed_min_date: node.find_child_value("observed-min-date").to_owned(),
        observed_max_price: parse_price(node.find_child_value("observed-max-price")),
        observed_max_date: node.find_child_value("observed-max-date").to_owned(),
    }
}

fn parse_price(raw: &str) -> f64 {
    raw.trim().parse::<f64>().unwrap_or_default()
}
