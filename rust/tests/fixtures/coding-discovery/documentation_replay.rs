use std::path::Path;

use formal_ai::seed::parse_lino;
use formal_ai::{CachedSourceClient, FetchError, SourceTransport, sha256_hex};

pub struct DocumentationReplay;

impl SourceTransport for DocumentationReplay {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let manifest = parse_lino(include_str!("documentation-replays.lino"));
        let entry = manifest.children[0]
            .children
            .iter()
            .find(|row| row.name == "capture" && row.find_child_value("url") == url)
            .ok_or_else(|| FetchError::Transport(format!("unrecorded test source {url}")))?;
        let file = entry.find_child_value("fixture");
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository root");
        let bytes = std::fs::read(root.join(file)).expect("replay bytes");
        assert_eq!(
            sha256_hex(&bytes),
            entry.find_child_value("sha256"),
            "{file}"
        );
        Ok(bytes)
    }
}

pub fn client() -> CachedSourceClient<DocumentationReplay> {
    let thread = std::thread::current();
    let name = thread.name().expect("test name").replace("::", "-");
    let cache = std::env::temp_dir()
        .join(format!("formal-ai-doc-replay-{}", std::process::id()))
        .join(name);
    CachedSourceClient::new(cache, DocumentationReplay).with_online(true)
}
