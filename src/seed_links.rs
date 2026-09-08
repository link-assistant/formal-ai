//! The seed and meta documents as one links network (issue #1085 D1.2).
//!
//! Every bundled `data/seed` document and the routing documents under
//! `data/meta` are projected at startup into doublets: a node link
//! `(parent -> name)` per Links Notation node and a value link
//! `(node -> value)` for a node that carries one. Handler precedence, cue
//! lookup and intent routing read from this network through link queries
//! instead of each owning a parser over its own text, and a native build
//! mirrors the same links into a link-cli store beside the memory store when
//! the server starts.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::link_store::DoubletLink;
use crate::links_substitution_query::{LinkPattern, LinkRewriteProgram, LinkRewriteRule, Slot};
use crate::seed::parser::{LinoNode, parse_lino};

/// The `data/meta` documents routing reads; they join the network beside the
/// seed bundle.
#[must_use]
pub fn meta_documents() -> Vec<(&'static str, &'static str)> {
    vec![(
        crate::cue_lexicon::CUE_LEXICON_PATH,
        crate::cue_lexicon::CUE_LEXICON_LINO,
    )]
}

/// The projected network.
#[derive(Debug)]
pub struct SeedLinkNetwork {
    links: Vec<DoubletLink>,
    by_source: BTreeMap<String, Vec<usize>>,
    documents: Vec<(String, String)>,
}

/// The network over every bundled seed document and the meta documents,
/// projected once per process.
#[must_use]
pub fn network() -> &'static SeedLinkNetwork {
    static CACHE: OnceLock<SeedLinkNetwork> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut documents = crate::seed::seed_files();
        documents.extend(meta_documents());
        SeedLinkNetwork::from_documents(&documents)
    })
}

impl SeedLinkNetwork {
    /// Project `(path, text)` documents into links.
    #[must_use]
    pub fn from_documents(documents: &[(&str, &str)]) -> Self {
        let mut network = Self {
            links: Vec::new(),
            by_source: BTreeMap::new(),
            documents: Vec::new(),
        };
        for (position, (path, text)) in documents.iter().enumerate() {
            let document_index = format!("d{position}");
            network.push(
                document_index.clone(),
                String::from("seed"),
                (*path).to_owned(),
            );
            network
                .documents
                .push(((*path).to_owned(), document_index.clone()));
            let tree = parse_lino(text);
            let mut counter = 0;
            network.project(
                &document_index,
                &document_index,
                &tree.children,
                &mut counter,
            );
        }
        network
    }

    fn project(&mut self, document: &str, parent: &str, nodes: &[LinoNode], counter: &mut usize) {
        for node in nodes {
            *counter += 1;
            let index = format!("{document}:{counter}");
            self.push(index.clone(), parent.to_owned(), node.name.clone());
            if !node.id.is_empty() {
                self.push(format!("{index}="), index.clone(), node.id.clone());
            }
            self.project(document, &index, &node.children, counter);
        }
    }

    fn push(&mut self, index: String, from: String, to: String) {
        self.by_source
            .entry(from.clone())
            .or_default()
            .push(self.links.len());
        self.links.push(DoubletLink { index, from, to });
    }

    /// Every projected link, in projection order.
    #[must_use]
    pub fn links(&self) -> &[DoubletLink] {
        &self.links
    }

    /// Number of projected documents.
    #[must_use]
    pub const fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// The document node index for `path`.
    #[must_use]
    pub fn document(&self, path: &str) -> Option<&str> {
        self.documents
            .iter()
            .find(|(candidate, _)| candidate == path)
            .map(|(_, index)| index.as_str())
    }

    /// Every link whose source is `parent`, in projection order.
    #[must_use]
    pub fn children(&self, parent: &str) -> Vec<&DoubletLink> {
        self.by_source
            .get(parent)
            .map_or_else(Vec::new, |positions| {
                positions
                    .iter()
                    .map(|position| &self.links[*position])
                    .collect()
            })
    }

    /// The node links under `parent`: children without the value link.
    #[must_use]
    pub fn nodes_under(&self, parent: &str) -> Vec<&DoubletLink> {
        self.children(parent)
            .into_iter()
            .filter(|link| !link.index.ends_with('='))
            .collect()
    }

    /// The top-level nodes of the document at `path`.
    #[must_use]
    pub fn top_level(&self, path: &str) -> Vec<&DoubletLink> {
        self.document(path)
            .map_or_else(Vec::new, |document| self.nodes_under(document))
    }

    /// The value a node carries, if any.
    #[must_use]
    pub fn value_of(&self, node: &str) -> Option<&str> {
        let index = format!("{node}=");
        self.children(node)
            .into_iter()
            .find(|link| link.index == index)
            .map(|link| link.to.as_str())
    }

    /// The value of the first child of `parent` named `name`.
    #[must_use]
    pub fn field(&self, parent: &str, name: &str) -> Option<&str> {
        self.nodes_under(parent)
            .into_iter()
            .find(|link| link.to == name)
            .and_then(|link| self.value_of(&link.index))
    }

    /// The values of every child of `parent` named `name`, in order.
    #[must_use]
    pub fn field_values(&self, parent: &str, name: &str) -> Vec<String> {
        self.nodes_under(parent)
            .into_iter()
            .filter(|link| link.to == name)
            .filter_map(|link| self.value_of(&link.index))
            .map(str::to_owned)
            .collect()
    }

    /// Run a link pattern over the whole network.
    #[must_use]
    pub fn query(&self, pattern: &LinkPattern) -> Vec<DoubletLink> {
        let program = LinkRewriteProgram::new(
            vec![LinkRewriteRule {
                pattern: Some(pattern.clone()),
                replacement: None,
            }],
            1,
        );
        program.matched_links(&self.links)
    }

    /// The pattern `(parent $child)`: every link leaving `parent`.
    #[must_use]
    pub fn children_pattern(parent: &str) -> LinkPattern {
        LinkPattern {
            index: None,
            source: Slot::Value(parent.to_owned()),
            target: Slot::Variable(String::from("child")),
        }
    }
}

/// Whether `formal-ai serve` mirrors the network into the native store.
///
/// Opt-in through `FORMAL_AI_SEED_LINKS_MIRROR=1` (or `true`, `on`). The
/// mirror writes tens of thousands of doublets through the transaction log,
/// and a server started per scenario pays that once per server: the held-out
/// generalization end-to-end run starts 24 of them and went from 265 s to over
/// its 540 s budget with the mirror on by default. Routing does not read the
/// mirror -- it reads the in-process network, which is built once either way --
/// so what the mirror buys is the on-disk projection, and that is worth asking
/// for where it is wanted (issue #1085 D1.2).
#[must_use]
pub fn native_mirror_enabled() -> bool {
    matches!(
        std::env::var("FORMAL_AI_SEED_LINKS_MIRROR").as_deref(),
        Ok("1" | "true" | "on")
    )
}

/// Where a native build keeps the seed network beside the memory store.
#[must_use]
pub fn seed_link_database_path(memory_path: &Path) -> PathBuf {
    memory_path.with_extension("seed.links")
}

/// Mirror the network into a link-cli store beside the configured memory
/// file and keep it open for the process.
///
/// Returns `Ok(None)` when no memory path is configured or the build has no
/// native store.
///
/// # Errors
///
/// Returns the store error as text when the native store cannot be rebuilt.
pub fn mirror_native_store() -> Result<Option<(PathBuf, usize)>, String> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
    {
        native::mirror()
    }
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "doublets-native")))]
    {
        Ok(None)
    }
}

/// Number of links in the native mirror this process opened, if any.
#[must_use]
pub fn native_mirror_link_count() -> Option<usize> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
    {
        native::link_count()
    }
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "doublets-native")))]
    {
        None
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
mod native {
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    use crate::link_store::LinkCliLinkStore;

    static MIRROR: OnceLock<Mutex<Option<LinkCliLinkStore>>> = OnceLock::new();

    pub(super) fn mirror() -> Result<Option<(PathBuf, usize)>, String> {
        let Some(memory_path) = crate::memory_sync::configured_memory_path() else {
            return Ok(None);
        };
        let database = super::seed_link_database_path(&memory_path);
        let store = LinkCliLinkStore::rebuild_with_doublets(&database, super::network().links())
            .map_err(|error| error.to_string())?;
        let count = store.native_link_count();
        let slot = MIRROR.get_or_init(|| Mutex::new(None));
        if let Ok(mut guard) = slot.lock() {
            *guard = Some(store);
        }
        Ok(Some((database, count)))
    }

    pub(super) fn link_count() -> Option<usize> {
        let guard = MIRROR.get()?.lock().ok()?;
        guard.as_ref().map(LinkCliLinkStore::native_link_count)
    }
}
