use formal_ai::seed::market_price_assets;
fn main() {
    for asset in market_price_assets() {
        println!(
            "asset={} label={} grounded_in={} quote={} aliases={:?}",
            asset.ticker, asset.label, asset.grounded_in, asset.quote_currency, asset.aliases
        );
        for r in &asset.references {
            println!(
                "  {} {} min={} ({}) max={} ({}) url={}",
                r.period,
                r.source_id,
                r.observed_min_price,
                r.observed_min_date,
                r.observed_max_price,
                r.observed_max_date,
                r.source_url
            );
        }
    }
}
