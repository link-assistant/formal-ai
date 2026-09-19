//! The unified fragment store: a deletable bootstrap seed plus rediscovered
//! fragments, content-addressed (issue #1138, plan 02 L4, L16–L18).
//!
//! [`FragmentCatalog::bootstrap`] reads `data/seed/` at **runtime**, so "delete
//! the seed" is expressible: a missing file yields an empty catalog, never a
//! panic. Forgetting a fragment and rediscovering it from the same captures must
//! reproduce the same [`FragmentCatalog::content_id`].

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::coding::program_ir::{IrType, ReuseMode, parse_type_slug, type_slug};
use crate::links_format::push_lino_node;
use crate::seed::parser::{LinoNode, parse_lino, split_pipe_list};

/// Where a fragment came from, and whether it may be deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FragmentOrigin {
    /// Shipped in `data/seed/`, marked `bootstrap true`, deletable.
    Bootstrap,
    /// Rediscovered from a trusted source into the ignored cache.
    Rediscovered,
}

/// Bootstrap vocabulary that introduced an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FragmentKind {
    Meaning,
    Runtime,
}

/// One reusable operation, language-neutral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub id: String,
    pub signature: Vec<IrType>,
    pub result: IrType,
    pub origin: FragmentOrigin,
    pub kind: FragmentKind,
    pub reuse: ReuseMode,
    pub grounding: String,
    pub license: String,
    /// Whether this fragment's grounding belongs in a derived artifact's
    /// source list. Idioms that are pure language syntax (a spelling the
    /// language reference defines, not knowledge the artifact relies on)
    /// declare `cites false`: they appear in derivations but not citations.
    pub cites: bool,
    pub sha256: String,
    pub fetched_at: String,
    /// The natural-language query that rediscovers this fragment when it is
    /// forgotten — never a URL and never a task name.
    pub rediscovery_query: String,
    /// Requested meanings this smaller operation can help realize. These are
    /// relevance edges, not an authored program: typed search still has to
    /// discover an arrangement which type-checks and passes the examples.
    pub supports: Vec<String>,
    /// Target-language surface forms discovered with this operation. The IR
    /// names only `id`; lowering binds these placeholders to child nodes.
    pub realizations: BTreeMap<String, String>,
}

impl Fragment {
    /// Placeholder order for a target-language realization. Keeping names in
    /// the catalog lets search introduce binder references without teaching it
    /// Python templates or individual algorithms.
    #[must_use]
    pub fn argument_names(&self, language: &str) -> Vec<String> {
        self.realizations
            .get(language)
            .map_or_else(Vec::new, |surface| placeholders(surface))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FragmentCatalog {
    fragments: Vec<Fragment>,
}

impl FragmentCatalog {
    /// Bootstrap fragments from `data/seed/`, or an empty catalog when the seed
    /// files are absent. Never panics on a missing id.
    #[must_use]
    pub fn bootstrap() -> Self {
        Self::bootstrap_from(bootstrap_seed_directory())
    }

    /// Bootstrap from an explicit seed directory, so the deletability gate can
    /// point the catalog at a tree with the seed moved aside.
    #[must_use]
    pub fn bootstrap_from(seed_directory: impl AsRef<Path>) -> Self {
        let seed_directory = seed_directory.as_ref();
        let seed_directory = if seed_directory.join("data/seed").is_dir() {
            seed_directory.join("data/seed")
        } else {
            seed_directory.to_path_buf()
        };
        let mut fragments = Vec::new();
        // The coding-structure meanings exceed the reviewability line cap of a
        // single document (issue #960 R222-1), so the family is split across
        // two files that contribute to one catalog.
        for name in [
            "meanings-coding-structure.lino",
            "meanings-coding-structure-2.lino",
        ] {
            if let Ok(text) = fs::read_to_string(seed_directory.join(name)) {
                fragments.extend(fragments_from_meanings(&text));
            }
        }
        let composition_fragments = seed_directory.join("coding-composition-fragments.lino");
        if let Ok(text) = fs::read_to_string(composition_fragments) {
            fragments.extend(fragments_from_meanings(&text));
        }
        let runtime = seed_directory.join("coding-discovery-runtime.lino");
        if let Ok(text) = fs::read_to_string(runtime) {
            fragments.extend(fragments_from_runtime(&text));
        }
        normalize_catalog(fragments)
    }

    /// The bootstrap seed files missing from `seed_directory`. The runtime logs
    /// these when it boots, so a deleted bootstrap shows up as an explicit
    /// event instead of silently accepted absence.
    #[must_use]
    pub fn absent_seed_files(seed_directory: impl AsRef<Path>) -> Vec<String> {
        let seed_directory = seed_directory.as_ref();
        let seed_directory = if seed_directory.join("data/seed").is_dir() {
            seed_directory.join("data/seed")
        } else {
            seed_directory.to_path_buf()
        };
        [
            "meanings-coding-structure.lino",
            "meanings-coding-structure-2.lino",
            "coding-composition-fragments.lino",
            "coding-discovery-runtime.lino",
        ]
        .into_iter()
        .filter(|name| !seed_directory.join(name).is_file())
        .map(str::to_owned)
        .collect()
    }

    /// Merge rediscovered fragments from the ignored cache ledger.
    #[must_use]
    pub fn with_rediscovered(self, ledger: &FragmentLedger) -> Self {
        let mut by_id = self
            .fragments
            .into_iter()
            .map(|fragment| (fragment.id.clone(), fragment))
            .collect::<BTreeMap<_, _>>();
        let Ok(entries) = fs::read_dir(&ledger.path) else {
            return Self {
                fragments: by_id.into_values().collect(),
            };
        };
        let mut paths = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "lino")
            })
            .collect::<Vec<_>>();
        paths.sort();
        for path in paths {
            if let Ok(text) = fs::read_to_string(path)
                && let Some(fragment) = parse_fragment(&text)
                && fragment.origin == FragmentOrigin::Rediscovered
                && valid_rediscovered(&fragment)
            {
                by_id.insert(fragment.id.clone(), fragment);
            }
        }
        Self {
            fragments: by_id.into_values().collect(),
        }
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Fragment> {
        self.fragments.iter().find(|fragment| fragment.id == id)
    }

    /// Fragments whose result type unifies with `ty`, cheapest first.
    #[must_use]
    pub fn producing(&self, ty: &IrType) -> Vec<&Fragment> {
        let mut found = self
            .fragments
            .iter()
            .filter(|fragment| types_may_unify(&fragment.result, ty))
            .collect::<Vec<_>>();
        found.sort_by_key(|fragment| (fragment.signature.len(), fragment.id.as_str()));
        found
    }

    /// Every fragment in the catalog, in catalog order.
    #[must_use]
    pub fn fragments(&self) -> &[Fragment] {
        &self.fragments
    }

    /// Render one fragment in `language`, or return `None` when the source did
    /// not provide a reusable surface for that backend.
    #[must_use]
    pub fn render(&self, id: &str, language: &str, arguments: &[String]) -> Option<String> {
        let fragment = self.get(id)?;
        let mut rendered = fragment.realizations.get(language)?.clone();
        let slots = placeholders(&rendered);
        if slots.len() != arguments.len() {
            return None;
        }
        for (slot, value) in slots.iter().zip(arguments) {
            rendered = rendered.replace(&format!("{{{slot}}}"), value);
        }
        Some(rendered)
    }

    /// Render by placeholder name. Composition adapters use this while they
    /// are migrated to typed IR, so a missing id, backend, or required slot is
    /// explicit rather than becoming an empty source string.
    #[must_use]
    pub fn render_named(
        &self,
        id: &str,
        language: &str,
        values: &[(&str, &str)],
    ) -> Option<String> {
        let fragment = self.get(id)?;
        let mut rendered = fragment.realizations.get(language)?.clone();
        for slot in placeholders(&rendered) {
            let value = values
                .iter()
                .find_map(|(name, value)| (*name == slot).then_some(*value))?;
            rendered = rendered.replace(&format!("{{{slot}}}"), value);
        }
        Some(rendered)
    }

    /// Content hash over every fragment's canonical projection, origin
    /// excluded. This is the value the forget / rediscover test compares.
    #[must_use]
    pub fn content_id(&self) -> String {
        let canonical = self
            .fragments
            .iter()
            .map(canonical_fragment)
            .collect::<Vec<_>>()
            .join("\n");
        crate::source_fetch::sha256_hex(canonical.as_bytes())
    }
}

pub(crate) fn bootstrap_seed_directory() -> PathBuf {
    std::env::var_os("FORMAL_AI_DATA_DIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("data/seed"),
        PathBuf::from,
    )
}

pub(crate) fn bootstrap_seed_text(name: &str) -> Option<String> {
    let directory = bootstrap_seed_directory();
    let directory = if directory.join("data/seed").is_dir() {
        directory.join("data/seed")
    } else {
        directory
    };
    fs::read_to_string(directory.join(name)).ok()
}

/// Content-addressed store of rediscovered fragments under
/// `FORMAL_AI_CACHE_DIR`.
#[derive(Debug, Clone)]
pub struct FragmentLedger {
    path: PathBuf,
}

impl FragmentLedger {
    #[must_use]
    pub fn new(cache_directory: impl AsRef<Path>) -> Self {
        Self {
            path: cache_directory.as_ref().join("coding-fragments"),
        }
    }

    pub fn recall(&self, id: &str) -> io::Result<Option<Fragment>> {
        let path = self.fragment_path(id);
        match fs::read_to_string(path) {
            Ok(text) => Ok(parse_fragment(&text).filter(|fragment| fragment.id == id)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Forget only rediscovered cache entries. Shipped seed and conversation
    /// memory are outside this directory and cannot be affected.
    pub fn forget_all(&self) -> io::Result<usize> {
        let Ok(entries) = fs::read_dir(&self.path) else {
            return Ok(0);
        };
        let mut removed = 0;
        for entry in entries {
            let path = entry?.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "lino")
            {
                fs::remove_file(path)?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    pub fn remember(&self, fragment: &Fragment) -> io::Result<()> {
        if fragment.origin != FragmentOrigin::Rediscovered || !valid_rediscovered(fragment) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "a rediscovered fragment requires grounding, license, sha256, and a query",
            ));
        }
        fs::create_dir_all(&self.path)?;
        let path = self.fragment_path(&fragment.id);
        let temporary = path.with_extension(format!("lino.tmp-{}", std::process::id()));
        fs::write(&temporary, fragment_projection(fragment))?;
        fs::rename(temporary, path)
    }

    fn fragment_path(&self, id: &str) -> PathBuf {
        self.path.join(format!(
            "{}.lino",
            crate::source_fetch::sha256_hex(id.as_bytes())
        ))
    }
}

fn normalize_catalog(fragments: Vec<Fragment>) -> FragmentCatalog {
    let by_id = fragments
        .into_iter()
        .map(|fragment| (fragment.id.clone(), fragment))
        .collect::<BTreeMap<_, _>>();
    FragmentCatalog {
        fragments: by_id.into_values().collect(),
    }
}

fn fragments_from_meanings(text: &str) -> Vec<Fragment> {
    let tree = parse_lino(text);
    tree.children
        .iter()
        .filter(|node| node.name == "meanings")
        .flat_map(|node| node.children.iter())
        .filter_map(|node| {
            let idiom = node.find_child_value("idiom");
            if idiom.is_empty() {
                return None;
            }
            let id = if node.name == "meaning" {
                node.id.as_str()
            } else {
                node.name.as_str()
            };
            fragment_from_seed_node(
                id,
                idiom,
                node,
                "bootstrap-reference-only",
                FragmentKind::Meaning,
            )
        })
        .collect()
}

fn fragments_from_runtime(text: &str) -> Vec<Fragment> {
    let tree = parse_lino(text);
    tree.children
        .iter()
        .filter(|node| node.name == "coding_discovery_runtime")
        .flat_map(|node| node.children.iter())
        .filter(|node| node.name == "template")
        // Renderer scaffolds share this file with reusable operations. Only a
        // record with an explicit typed signature is a searchable fragment;
        // treating `python_return` or a test-entrypoint template as an
        // operation makes data layout leak into the generated programs.
        .filter(|node| {
            !node.find_child_value("fragment_signature").is_empty()
                && !node.find_child_value("fragment_result").is_empty()
        })
        .filter_map(|node| {
            fragment_from_seed_node(
                &node.id,
                node.find_child_value("text"),
                node,
                "repository-bootstrap",
                FragmentKind::Runtime,
            )
        })
        .collect()
}

fn fragment_from_seed_node(
    id: &str,
    surface: &str,
    node: &LinoNode,
    default_license: &str,
    kind: FragmentKind,
) -> Option<Fragment> {
    if id.is_empty() || surface.is_empty() {
        return None;
    }
    let signature = if node.find_child_value("fragment_signature").is_empty() {
        placeholders(surface)
            .into_iter()
            .enumerate()
            .map(|(index, _)| IrType::Unknown(index))
            .collect()
    } else {
        split_pipe_list(node.find_child_value("fragment_signature"))
            .iter()
            .map(String::as_str)
            .map(parse_type_slug)
            .collect::<Option<Vec<_>>>()?
    };
    let result = node
        .find_child_value("fragment_result")
        .is_empty()
        .then(|| inferred_result(id, node.find_child_value("defined-by")))
        .or_else(|| parse_type_slug(node.find_child_value("fragment_result")))?;
    let grounding = match node.find_child_value("grounding") {
        "" => format!("data/seed:fragment:{id}"),
        grounding => grounding.to_owned(),
    };
    let license = match node.find_child_value("license") {
        "" => default_license.to_owned(),
        license => license.to_owned(),
    };
    // Citations default on; a seed row opts out only for pure language
    // syntax (`cites false`), whose grounding documents a spelling rather
    // than knowledge the derived artifact relies on.
    let cites = node.find_child_value("cites") != "false";
    let rediscovery_query = match node.find_child_value("rediscovery_query") {
        "" => crate::coding::python_render::runtime_template(
            "fragment_definition_query",
            &[("fragment", &id.replace('_', " "))],
        )
        .unwrap_or_else(|| id.replace('_', " ")),
        query => query.to_owned(),
    };
    let normalized_license = license.to_ascii_lowercase();
    let reuse = if normalized_license.contains("by-sa")
        || normalized_license.contains("gfdl")
        || normalized_license.contains("reference-only")
    {
        ReuseMode::ShapeOnly
    } else {
        ReuseMode::Verbatim
    };
    let mut realizations = BTreeMap::from([("python".to_owned(), surface.to_owned())]);
    // A seed fragment may carry additional per-language realizations of the
    // same operation (`realization` → language → surface). They widen which
    // target the search can name arguments and lower for; the python idiom
    // stays the canonical surface the other languages translate.
    for child in node
        .children
        .iter()
        .filter(|child| child.name == "realization")
    {
        for language_node in &child.children {
            if !language_node.id.is_empty() {
                realizations
                    .entry(language_node.name.clone())
                    .or_insert_with(|| language_node.id.clone());
            }
        }
    }
    Some(Fragment {
        id: id.to_owned(),
        signature,
        result,
        origin: FragmentOrigin::Bootstrap,
        kind,
        reuse,
        grounding,
        license,
        cites,
        sha256: crate::source_fetch::sha256_hex(surface.as_bytes()),
        fetched_at: "bootstrap".to_owned(),
        rediscovery_query,
        supports: node
            .children
            .iter()
            .filter(|child| child.name == "supports")
            .map(|child| child.id.clone())
            .collect(),
        realizations,
    })
}

fn placeholders(template: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        let tail = &rest[start + 1..];
        let Some(end) = tail.find('}') else {
            break;
        };
        // A replacement may sit inside target-language braces. For example,
        // the Python regex quantifier `{{minimum},}` means “retain the outer
        // braces and substitute `minimum`”. Treat the extra opening brace as
        // target syntax rather than part of the placeholder name.
        let name = tail[..end].trim().trim_start_matches('{').trim();
        if !name.is_empty() && !found.iter().any(|seen| seen == name) {
            found.push(name.to_owned());
        }
        rest = &tail[end + 1..];
    }
    found
}

fn inferred_result(id: &str, defined_by: &str) -> IrType {
    let evidence = [id, defined_by].join(" ").to_ascii_lowercase();
    if [
        "predicate",
        "quantifier",
        "comparison",
        "contains",
        "membership",
    ]
    .iter()
    .any(|cue| evidence.contains(cue))
    {
        IrType::Boolean
    } else if ["count", "length", "index", "sum", "integer"]
        .iter()
        .any(|cue| evidence.contains(cue))
    {
        IrType::Integer
    } else if ["text", "string", "join", "normalization"]
        .iter()
        .any(|cue| evidence.contains(cue))
    {
        IrType::Text
    } else if ["map", "filter", "sequence", "ordering", "split"]
        .iter()
        .any(|cue| evidence.contains(cue))
    {
        IrType::Sequence(Box::new(IrType::Unknown(0)))
    } else {
        IrType::Unknown(0)
    }
}

fn types_may_unify(left: &IrType, right: &IrType) -> bool {
    match (left, right) {
        (IrType::Unknown(_), _) | (_, IrType::Unknown(_)) => true,
        (IrType::Sequence(left), IrType::Sequence(right)) => types_may_unify(left, right),
        (IrType::OrderedSequence(left), IrType::OrderedSequence(right)) => {
            types_may_unify(left, right)
        }
        // An ordered sequence is a sequence, so order-indifferent consumers
        // accept it. The reverse never unifies: a producer that fixes no
        // element order cannot feed a consumer that depends on one.
        (IrType::OrderedSequence(ordered), IrType::Sequence(unordered)) => {
            types_may_unify(ordered, unordered)
        }
        (IrType::Sequence(_), IrType::OrderedSequence(_)) => false,
        (IrType::Sequence(element), IrType::Text) | (IrType::Text, IrType::Sequence(element)) => {
            types_may_unify(element, &IrType::Text)
        }
        (IrType::Pair(left_a, left_b), IrType::Pair(right_a, right_b))
        | (IrType::Mapping(left_a, left_b), IrType::Mapping(right_a, right_b)) => {
            types_may_unify(left_a, right_a) && types_may_unify(left_b, right_b)
        }
        _ => left == right,
    }
}

const fn valid_rediscovered(fragment: &Fragment) -> bool {
    !fragment.id.is_empty()
        && !fragment.grounding.is_empty()
        && !fragment.license.is_empty()
        && !fragment.sha256.is_empty()
        && !fragment.rediscovery_query.is_empty()
}

fn canonical_fragment(fragment: &Fragment) -> String {
    let signature = fragment
        .signature
        .iter()
        .map(type_slug)
        .collect::<Vec<_>>()
        .join("|");
    let result = type_slug(&fragment.result);
    let realizations = fragment
        .realizations
        .iter()
        .map(|(language, surface)| format!("{language}\u{1e}{surface}"))
        .collect::<Vec<_>>()
        .join("\u{1d}");
    let supports = fragment.supports.join("|");
    [
        fragment.id.as_str(),
        signature.as_str(),
        result.as_str(),
        kind_slug(fragment.kind),
        reuse_slug(fragment.reuse),
        fragment.grounding.as_str(),
        fragment.license.as_str(),
        fragment.sha256.as_str(),
        fragment.rediscovery_query.as_str(),
        supports.as_str(),
        realizations.as_str(),
    ]
    .join("\u{1f}")
}

fn fragment_projection(fragment: &Fragment) -> String {
    let mut output = String::new();
    push_lino_node(&mut output, 0, "fragment", Some(&fragment.id));
    for argument in &fragment.signature {
        push_lino_node(&mut output, 2, "argument_type", Some(&type_slug(argument)));
    }
    push_lino_node(
        &mut output,
        2,
        "result_type",
        Some(&type_slug(&fragment.result)),
    );
    push_lino_node(&mut output, 2, "origin", Some(origin_slug(fragment.origin)));
    push_lino_node(&mut output, 2, "kind", Some(kind_slug(fragment.kind)));
    push_lino_node(&mut output, 2, "reuse", Some(reuse_slug(fragment.reuse)));
    push_lino_node(&mut output, 2, "grounding", Some(&fragment.grounding));
    push_lino_node(&mut output, 2, "license", Some(&fragment.license));
    push_lino_node(&mut output, 2, "sha256", Some(&fragment.sha256));
    push_lino_node(&mut output, 2, "fetched_at", Some(&fragment.fetched_at));
    push_lino_node(
        &mut output,
        2,
        "rediscovery_query",
        Some(&fragment.rediscovery_query),
    );
    for supported in &fragment.supports {
        push_lino_node(&mut output, 2, "supports", Some(supported));
    }
    for (language, surface) in &fragment.realizations {
        push_lino_node(
            &mut output,
            2,
            "realization",
            Some(&format!("{language}|{surface}")),
        );
    }
    output.trim_end().to_owned()
}

fn parse_fragment(text: &str) -> Option<Fragment> {
    let tree = parse_lino(text);
    let node = tree.children.iter().find(|node| node.name == "fragment")?;
    Some(Fragment {
        id: node.id.clone(),
        signature: node
            .children
            .iter()
            .filter(|child| child.name == "argument_type")
            .map(|child| parse_type_slug(&child.id))
            .collect::<Option<Vec<_>>>()?,
        result: parse_type_slug(node.find_child_value("result_type"))?,
        origin: match node.find_child_value("origin") {
            "bootstrap" => FragmentOrigin::Bootstrap,
            "rediscovered" => FragmentOrigin::Rediscovered,
            _ => return None,
        },
        kind: match node.find_child_value("kind") {
            "meaning" => FragmentKind::Meaning,
            "runtime" => FragmentKind::Runtime,
            _ => return None,
        },
        reuse: match node.find_child_value("reuse") {
            "verbatim" => ReuseMode::Verbatim,
            "shape_only" => ReuseMode::ShapeOnly,
            _ => return None,
        },
        grounding: node.find_child_value("grounding").to_owned(),
        license: node.find_child_value("license").to_owned(),
        cites: node.find_child_value("cites") != "false",
        sha256: node.find_child_value("sha256").to_owned(),
        fetched_at: node.find_child_value("fetched_at").to_owned(),
        rediscovery_query: node.find_child_value("rediscovery_query").to_owned(),
        supports: node
            .children
            .iter()
            .filter(|child| child.name == "supports")
            .map(|child| child.id.clone())
            .collect(),
        realizations: node
            .children
            .iter()
            .filter(|child| child.name == "realization")
            .filter_map(|child| child.id.split_once('|'))
            .map(|(language, surface)| (language.to_owned(), surface.to_owned()))
            .collect(),
    })
}

const fn origin_slug(origin: FragmentOrigin) -> &'static str {
    match origin {
        FragmentOrigin::Bootstrap => "bootstrap",
        FragmentOrigin::Rediscovered => "rediscovered",
    }
}

const fn kind_slug(kind: FragmentKind) -> &'static str {
    match kind {
        FragmentKind::Meaning => "meaning",
        FragmentKind::Runtime => "runtime",
    }
}

const fn reuse_slug(reuse: ReuseMode) -> &'static str {
    match reuse {
        ReuseMode::Verbatim => "verbatim",
        ReuseMode::ShapeOnly => "shape_only",
    }
}
