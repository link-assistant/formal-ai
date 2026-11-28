// Test-only publisher responses for the documented-method route.
use formal_ai::SymbolicAnswer;
use formal_ai::event_log::EventLog;
use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceTransport};
use formal_ai::try_documentation_method_with_client;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

pub const JOIN_SOURCE: &str =
    "https://pandas.pydata.org/docs/reference/api/pandas.DataFrame.join.html";
pub const DIFF_SOURCE: &str =
    "https://pandas.pydata.org/docs/reference/api/pandas.Series.diff.html";
pub const INDEX_SOURCE: &str = "https://pandas.pydata.org/docs/searchindex.js";
pub const RESHAPE_SOURCE: &str =
    "https://numpy.org/doc/stable/reference/generated/numpy.ndarray.reshape.html";
pub const EMPTY_SOURCE: &str = "https://numpy.org/doc/stable/reference/generated/numpy.empty.html";
static NEXT_CACHE: AtomicUsize = AtomicUsize::new(0);

struct Publisher;
impl SourceTransport for Publisher {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let bytes: &[u8] = match url {
            JOIN_SOURCE => include_bytes!("documentation-method/join.html"),
            DIFF_SOURCE => include_bytes!("documentation-method/diff.html"),
            INDEX_SOURCE => include_bytes!("documentation-method/pandas-searchindex.js"),
            RESHAPE_SOURCE => include_bytes!("documentation-method/reshape.html"),
            EMPTY_SOURCE => include_bytes!("documentation-method/empty.html"),
            _ => return Err(FetchError::OfflineCacheMiss(url.to_owned())),
        };
        Ok(bytes.to_vec())
    }
}

struct Cache(PathBuf);
impl Drop for Cache {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn documented_method_answer(prompt: &str) -> SymbolicAnswer {
    let cache = Cache(std::env::temp_dir().join(format!(
        "formal-ai-docs-method-{}-{}",
        std::process::id(),
        NEXT_CACHE.fetch_add(1, Ordering::Relaxed),
    )));
    let client = CachedSourceClient::new(&cache.0, Publisher).with_online(true);
    let answer = try_documentation_method_with_client(prompt, &mut EventLog::new(), &client)
        .expect("the documentation handler must claim this prompt");
    let offline = try_documentation_method_with_client(
        prompt,
        &mut EventLog::new(),
        &client.with_online(false),
    )
    .expect("the documentation handler must claim this prompt offline");
    assert_eq!(answer.answer, offline.answer, "{prompt}");
    answer
}

pub fn expected_join(language: &str) -> String {
    let link = match language {
        "en" => "Official documentation for pandas.DataFrame.join",
        "ru" => "Официальная документация для pandas.DataFrame.join",
        "hi" => "pandas.DataFrame.join के आधिकारिक दस्तावेज़",
        "zh" => "pandas.DataFrame.join 的官方文档",
        _ => panic!("missing documented language {language}"),
    };
    format!(
        "`DataFrame.join(other, on=None, how='left')`\n\nJoin columns of another DataFrame. Join columns with other DataFrame either on index or on a key column.\n\n[{link}]({JOIN_SOURCE})"
    )
}
