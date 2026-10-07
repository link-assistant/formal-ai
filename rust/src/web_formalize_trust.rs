/// Compute a page's trust score in 0..=100 from the seed's weights. The
/// score ranks sources; it never excludes one.
#[must_use]
pub fn trust_score(features: &TrustFeatures) -> u8 {
    let tree = parse_lino(TRUST_TEXT);
    let mut weights: BTreeMap<String, f64> = BTreeMap::new();
    let mut grades: BTreeMap<String, f64> = BTreeMap::new();
    let mut agreement_scale = 1.0f64;
    for record in seed_records(&tree) {
        if record.name != "feature" {
            continue;
        }
        let name = if record.id.is_empty() {
            record.find_child_value("name").to_owned()
        } else {
            record.id.clone()
        };
        if name.is_empty() {
            continue;
        }
        if let Ok(weight) = record.find_child_value("weight").parse::<f64>() {
            weights.insert(name.clone(), weight);
        }
        if name == "cross_page_agreement"
            && let Ok(scale) = record.find_child_value("agreement_scale").parse::<f64>()
        {
            agreement_scale = scale.max(1.0);
        }
        for grade in &record.children {
            if grade.name != "grade" {
                continue;
            }
            let grade_name = grade.find_child_value("name").to_owned();
            if let Ok(factor) = grade.find_child_value("factor").parse::<f64>()
                && !grade_name.is_empty()
            {
                grades.insert(grade_name, factor);
            }
        }
    }
    let mut score = 0.0f64;
    if features.official_site {
        score += weights.get("official_site").copied().unwrap_or(0.0);
    }
    if features.https {
        score += weights.get("https").copied().unwrap_or(0.0);
    }
    if let Some(primacy) = &features.primacy {
        let factor = grades.get(primacy).copied().unwrap_or(0.0);
        score = f64::mul_add(weights.get("primacy").copied().unwrap_or(0.0), factor, score);
    }
    if features.open_license {
        score += weights.get("open_license").copied().unwrap_or(0.0);
    }
    // `agreement_scale` is at least 1.0, so its floor is the whole-page cap.
    let agreement = f64::from(features.agreement_pages).min(agreement_scale.floor()) / agreement_scale;
    score = f64::mul_add(
        weights.get("cross_page_agreement").copied().unwrap_or(0.0),
        agreement,
        score,
    );
    // The rounded percentage is a whole number in 0..=100: pick the matching
    // `u8` without a lossy float-to-int cast (NaN falls through to 0).
    let percent = (score.clamp(0.0, 1.0) * 100.0).round();
    (0..=100u8)
        .rev()
        .find(|candidate| f64::from(*candidate) <= percent)
        .unwrap_or(0)
}

/// Attach a trust score and the rediscovery procedure to a just-formalized
/// network (issue #1163 R9).
///
/// It adds a `rediscovery` node carrying the query text,
/// the rank at fetch time, the URL, the SHA-256, the timestamp, and whether
/// the bytes came from the cache, so the record can be dropped under
/// storage pressure and re-fetched deterministically.
pub fn annotate_capture(network: &mut LinkNetwork, capture: &SourceCapture, query: &str, rank: u32, trust: u8) {
    let rediscovery = network.insert_object("rediscovery");
    for (label, value) in [
        ("query", query.to_owned()),
        ("rank", rank.to_string()),
        ("url", capture.source_url().to_owned()),
        ("sha256", capture.sha256().to_owned()),
        ("fetched_at", capture.fetched_at().to_owned()),
        ("cached", capture.cached().to_string()),
        ("trust", trust.to_string()),
    ] {
        let value_id = network.insert_point(&value);
        network.insert_field(rediscovery, label, value_id);
    }
}

/// Every statement text a network carries: the terms of its links.
#[must_use]
pub fn network_statements(network: &LinkNetwork) -> Vec<String> {
    let mut out = Vec::new();
    for link in network.links() {
        if let Some(term) = link.metadata().term()
            && !term.is_empty()
        {
            out.push(term.to_owned());
        }
    }
    out
}

/// True when every `statement` appears among the network's statements, so
/// the generic formalizer yields everything a bespoke extractor yields on
/// the same bytes (issue #1163 R3: generic ⊇ bespoke).
#[must_use]
pub fn covers_statements(network: &LinkNetwork, statements: &[String]) -> bool {
    let carried = network_statements(network);
    statements
        .iter()
        .filter(|statement| !statement.trim().is_empty())
        .all(|statement| {
            carried
                .iter()
                .any(|term| term == statement || term.contains(statement.as_str()))
        })
}

/// Statements of a freshly fetched payload, for the `generic_page_v1`
/// extractor branch in the source dispatcher.
#[must_use]
pub fn generic_page_statements(bytes: &[u8], mime_hint: Option<&str>) -> Vec<String> {
    network_statements(&formalize_page(bytes, mime_hint))
}

/// One indexed code block of a formalized page.
#[derive(Debug, Clone)]
pub struct CodeBlockIndex {
    /// The language tag the block carries.
    pub language: String,
    /// The block's text.
    pub text: String,
    /// The block's `code_block` link in the page network.
    pub link: Link,
}

/// One indexed command paragraph of a formalized page.
#[derive(Debug, Clone)]
pub struct CommandIndex {
    /// The paragraph's text.
    pub text: String,
    /// The paragraph's `paragraph` link in the page network.
    pub link: Link,
}

/// One indexed paragraph of a formalized page.
#[derive(Debug, Clone)]
pub struct ParagraphIndex {
    /// The paragraph's text.
    pub text: String,
    /// The paragraph's `paragraph` link in the page network.
    pub link: Link,
    /// The command tied to the paragraph: the paragraph itself when it is a
    /// command paragraph, else the code block or command paragraph that
    /// directly follows it (the command the paragraph introduces).
    pub command: Option<Link>,
}

/// A fetched, formalized, and annotated page held in working memory.
#[derive(Debug, Clone)]
pub struct FormalizedPage {
    /// Where the bytes came from.
    pub url: String,
    /// The SHA-256 of the fetched bytes.
    pub sha256: String,
    /// The result's rank at fetch time.
    pub rank: u32,
    /// The query that found the page.
    pub query: String,
    /// The page's trust score in 0..=100.
    pub trust: u8,
    /// The formalized page network.
    pub network: LinkNetwork,
    code_blocks: Vec<CodeBlockIndex>,
    commands: Vec<CommandIndex>,
    paragraphs: Vec<ParagraphIndex>,
}

/// The working-memory key under which a formalized page is stored, encoding
/// the URL and the SHA-256 so a second solve for the same URL reuses the
/// network without a re-fetch (issue #1163 R6).
#[must_use]
pub fn page_key(url: &str, sha256: &str) -> String {
    format!("page:{sha256}:{url}")
}

impl FormalizedPage {
    /// Build a page record from fetched bytes: formalize, index the code
    /// blocks, command paragraphs, and plain paragraphs, and annotate the
    /// network with the rediscovery procedure.
    #[must_use]
    pub fn from_capture(capture: &SourceCapture, query: &str, rank: u32, trust: u8, mime_hint: Option<&str>) -> Self {
        let (mut network, nodes) =
            formalize_page_with_context(capture.bytes(), mime_hint, Some(capture.source_url()));
        annotate_capture(&mut network, capture, query, rank, trust);
        let mut code_blocks = Vec::new();
        let mut commands = Vec::new();
        let mut paragraphs = Vec::new();
        let linked: Vec<(Link, PageBlock)> = nodes
            .into_iter()
            .filter_map(|(node, block)| network.link(node).map(|link| (link.clone(), block)))
            .collect();
        for (index, (link, block)) in linked.iter().enumerate() {
            match block {
                PageBlock::CodeBlock { language, text } => code_blocks.push(CodeBlockIndex {
                    language: language.clone(),
                    text: text.clone(),
                    link: link.clone(),
                }),
                PageBlock::Paragraph { text, command } => {
                    if *command {
                        commands.push(CommandIndex {
                            text: text.clone(),
                            link: link.clone(),
                        });
                    }
                    let tied = if *command {
                        Some(link.clone())
                    } else {
                        linked.get(index + 1).and_then(|(next_link, next)| match next {
                            PageBlock::CodeBlock { .. } | PageBlock::Paragraph { command: true, .. } => {
                                Some(next_link.clone())
                            }
                            _ => None,
                        })
                    };
                    paragraphs.push(ParagraphIndex {
                        text: text.clone(),
                        link: link.clone(),
                        command: tied,
                    });
                }
                _ => {}
            }
        }
        Self {
            url: capture.source_url().to_owned(),
            sha256: capture.sha256().to_owned(),
            rank,
            query: query.to_owned(),
            trust,
            network,
            code_blocks,
            commands,
            paragraphs,
        }
    }

    /// The store key for this page.
    #[must_use]
    pub fn key(&self) -> String {
        page_key(&self.url, &self.sha256)
    }

    /// The page's indexed code blocks.
    #[must_use]
    pub fn code_blocks(&self) -> &[CodeBlockIndex] {
        &self.code_blocks
    }

    /// The page's indexed command paragraphs.
    #[must_use]
    pub fn commands(&self) -> &[CommandIndex] {
        &self.commands
    }

    /// The page's indexed paragraphs, each with the command tied to it.
    #[must_use]
    pub fn paragraphs(&self) -> &[ParagraphIndex] {
        &self.paragraphs
    }

    /// Whether any statement of the page mentions the phrase.
    #[must_use]
    pub fn mentions(&self, phrase: &str) -> bool {
        network_statements(&self.network)
            .iter()
            .any(|statement| statement.contains(phrase))
    }
}

/// Formalized pages held in the solver's working memory, keyed by URL and
/// SHA-256 (issue #1163 R6).
#[derive(Debug, Default)]
pub struct FormalizedPageStore {
    pages: BTreeMap<String, FormalizedPage>,
}

impl FormalizedPageStore {
    /// An empty store.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
        }
    }

    /// Store a page under its key, replacing any earlier version.
    pub fn insert(&mut self, page: FormalizedPage) -> String {
        let key = page.key();
        self.pages.insert(key.clone(), page);
        key
    }

    /// The page stored under a key, if any.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&FormalizedPage> {
        self.pages.get(key)
    }

    /// Whether any stored page was fetched from the URL.
    #[must_use]
    pub fn contains_url(&self, url: &str) -> bool {
        self.pages.values().any(|page| page.url == url)
    }

    /// A stored page fetched from the URL, so a later solve can reuse the
    /// formalized network before deciding to fetch again.
    #[must_use]
    pub fn page_for_url(&self, url: &str) -> Option<&FormalizedPage> {
        self.pages.values().find(|page| page.url == url)
    }

    /// How many pages the store holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// Whether the store holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// "Code blocks on `<domain>` whose text contains `<term>`": every code
    /// block link of every page fetched from the domain whose text carries
    /// the term (issue #1163 R8).
    #[must_use]
    pub fn code_blocks_on(&self, domain: &str, term: &str) -> Vec<Link> {
        self.pages
            .values()
            .filter(|page| url_domain(&page.url) == domain)
            .flat_map(|page| page.code_blocks.iter())
            .filter(|block| block.text.contains(term))
            .map(|block| block.link.clone())
            .collect()
    }

    /// "The command in the paragraph that mentions `<phrase>`": for every
    /// paragraph that itself mentions the phrase, the command tied to it --
    /// the paragraph when it is a command, else the code block or command
    /// paragraph it introduces (issue #1163 R8). A command elsewhere on the
    /// same page never answers.
    #[must_use]
    pub fn command_mentioning(&self, phrase: &str) -> Vec<Link> {
        self.pages
            .values()
            .flat_map(|page| page.paragraphs.iter())
            .filter(|paragraph| paragraph.text.contains(phrase))
            .filter_map(|paragraph| paragraph.command.clone())
            .collect()
    }
}

/// The scheme whose pages earn the trust seed's `https` feature.
const HTTPS_SCHEME: &str = "https://";

thread_local! {
    /// The solver's working memory of formalized pages (issue #1163 R6).
    static WORKING_MEMORY: std::cell::RefCell<FormalizedPageStore> =
        const { std::cell::RefCell::new(FormalizedPageStore::new()) };
}

/// Run `action` against the solver's working memory of formalized pages
/// (issue #1163 R6): the store source research writes every captured page
/// into, keyed by URL and SHA-256, and the page queries read.
#[must_use]
pub fn with_working_memory<R>(action: impl FnOnce(&mut FormalizedPageStore) -> R) -> R {
    WORKING_MEMORY.with(|store| action(&mut store.borrow_mut()))
}

/// Formalize a captured page into working memory unless the same URL and
/// SHA-256 is already there. Returns the key and whether the stored network
/// was reused instead of formalizing the bytes again (issue #1163 R6).
#[must_use]
pub fn remember_capture(capture: &SourceCapture, query: &str, rank: u32) -> (String, bool) {
    let key = page_key(capture.source_url(), capture.sha256());
    with_working_memory(|store| {
        if store.get(&key).is_some() {
            return (key, true);
        }
        let trust = trust_score(&TrustFeatures {
            https: capture.source_url().starts_with(HTTPS_SCHEME),
            ..TrustFeatures::default()
        });
        let page = FormalizedPage::from_capture(capture, query, rank, trust, None);
        (store.insert(page), false)
    })
}
