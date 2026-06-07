fn main() {
    let status = std::process::Command::new("cargo")
        .arg("run")
        .arg("--example")
        .arg("wikidata_json_to_lino")
        .arg("--")
        .args(std::env::args().skip(1))
        .status()
        .expect("failed to run cargo example wikidata_json_to_lino");
    std::process::exit(status.code().unwrap_or(1));
}
