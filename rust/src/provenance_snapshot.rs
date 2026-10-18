//! Bounded source snapshots; canonical policy is compiled from the shared JavaScript kernel.
//! This authenticates source bytes and inventory, never engine authorship or replay.
use crate::provenance_native::{CpuHashMode, digest};
use crate::provenance_snapshot_generated::{self as policy, SnapshotPair, SnapshotShape};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::io;

pub(crate) mod repository;
/// One observed source file, with its exact repository-relative path.
pub struct SourceFile {
    /// UTF-8 relative path as observed in the Git index.
    pub path: String,
    /// Exact delivered file bytes.
    pub content: Vec<u8>,
}
/// A language-specific root, ordered by canonical UTF-8 language bytes.
#[derive(Debug, Serialize)]
pub struct LanguageRoot {
    /// Maintained language identity.
    pub language: String,
    /// Number of source leaves in this language group.
    pub file_count: usize,
    /// Canonical BLAKE3 root.
    pub root: String,
}
/// A snapshot is an integrity statement, not an execution or authorship claim.
#[derive(Debug, Serialize)]
pub struct SourceSnapshot {
    /// Stable wire-format identity.
    pub schema: &'static str,
    /// Canonically ordered per-language roots.
    pub languages: Vec<LanguageRoot>,
    /// Root over domain-separated language leaves.
    pub total: String,
}
/// An inclusion path authenticates a leaf, whose path/content binding is separate.
#[derive(Debug, Deserialize, Serialize)]
pub struct InclusionProof {
    /// Leaf index within its language group.
    pub index: usize,
    /// Number of leaves before odd-leaf duplication.
    pub count: usize,
    /// Ordered siblings from the leaf to the root.
    pub siblings: Vec<String>,
}
pub(crate) fn failure(reason: &str) -> io::Error {
    io::Error::other(reason)
}
fn scalar(value: usize) -> io::Result<f64> {
    u32::try_from(value)
        .map(f64::from)
        .map_err(|_| failure("snapshot_index_out_of_range"))
}
fn index(value: f64, maximum: usize) -> io::Result<usize> {
    let parsed = value
        .to_string()
        .parse::<usize>()
        .map_err(|_| failure("invalid_snapshot_recipe_index"))?;
    if parsed > maximum {
        return Err(failure("invalid_snapshot_recipe_index"));
    }
    Ok(parsed)
}
fn hexadecimal(value: [u8; 32]) -> String {
    blake3::Hash::from(value).to_hex().to_string()
}
fn byte_order(left: &str, right: &str) -> Ordering {
    let observed = policy::snapshot_byte_order(
        left.bytes().map(f64::from).collect(),
        right.bytes().map(f64::from).collect(),
    );
    observed.partial_cmp(&0.0).unwrap_or(Ordering::Equal)
}
/// Read the actual maintained language table with the production notation parser.
pub fn language_extensions(seed: &str) -> io::Result<Vec<(String, String)>> {
    let document = crate::seed::parse_lino_checked(seed).map_err(failure)?;
    let rules = document
        .children
        .iter()
        .find(|row| row.name == "page_formalization_rules")
        .ok_or_else(|| failure("language_table_required"))?;
    let mut extensions = Vec::new();
    for row in &rules.children {
        if row.name == "extension" {
            let suffix = row.find_child_value("suffix");
            let language = row.find_child_value("language");
            if suffix.is_empty() || language.is_empty() {
                return Err(failure("language_table_required"));
            }
            extensions.push((suffix.to_owned(), language.to_owned()));
        }
    }
    if extensions.is_empty() {
        return Err(failure("language_table_required"));
    }
    extensions.sort_by(|left, right| {
        right
            .0
            .encode_utf16()
            .count()
            .cmp(&left.0.encode_utf16().count())
            .then_with(|| byte_order(&left.0, &right.0))
    });
    Ok(extensions)
}
/// Apply the canonical backend decision; a GPU request never silently becomes CPU.
pub fn cpu_backend(requested: &str) -> io::Result<CpuHashMode> {
    match policy::snapshot_backend(requested.to_owned()).as_str() {
        "cpu" => Ok(CpuHashMode::Auto),
        "gpu-unavailable" => Err(failure("provenance_gpu_backend_unavailable")),
        _ => Err(failure("invalid_snapshot_backend")),
    }
}
fn tree_nodes(leaves: &[[u8; 32]], mode: CpuHashMode) -> io::Result<Vec<[u8; 32]>> {
    let SnapshotShape::Shape(total, _) = policy::snapshot_tree_shape(scalar(leaves.len())?);
    let total = index(total, usize::MAX)?;
    let mut nodes = leaves.to_vec();
    for position in leaves.len()..total {
        let SnapshotPair::Pair(left, right) =
            policy::snapshot_tree_pair(scalar(leaves.len())?, scalar(position)?)
        else {
            return Err(failure("invalid_snapshot_recipe"));
        };
        let left = index(left, position.saturating_sub(1))?;
        let right = index(right, position.saturating_sub(1))?;
        let mut input = b"formal-ai-source-node-v1\0".to_vec();
        input.extend_from_slice(&nodes[left]);
        input.extend_from_slice(&nodes[right]);
        nodes.push(digest(&input, mode));
    }
    Ok(nodes)
}
/// Execute canonical tree instructions with the audited native BLAKE3 implementation.
pub fn merkle_root(leaves: &[[u8; 32]], mode: CpuHashMode) -> io::Result<[u8; 32]> {
    if leaves.is_empty() {
        return Ok(digest(b"formal-ai-source-empty-v1\0", mode));
    }
    tree_nodes(leaves, mode)?
        .last()
        .copied()
        .ok_or_else(|| failure("invalid_snapshot_recipe"))
}
/// Produce siblings using the same canonical recipe consumed by JavaScript.
pub fn merkle_proof(
    leaves: &[[u8; 32]],
    selected: usize,
    mode: CpuHashMode,
) -> io::Result<InclusionProof> {
    if selected >= leaves.len() {
        return Err(failure("invalid_leaf_index"));
    }
    let nodes = tree_nodes(leaves, mode)?;
    let recipe = policy::snapshot_proof_recipe(scalar(leaves.len())?, scalar(selected)?);
    let siblings = recipe
        .into_iter()
        .map(|value| {
            index(value, nodes.len().saturating_sub(1)).map(|position| hexadecimal(nodes[position]))
        })
        .collect::<io::Result<Vec<_>>>()?;
    Ok(InclusionProof {
        index: selected,
        count: leaves.len(),
        siblings,
    })
}
/// Verify exact siblings, odd-leaf duplication and terminal root through the shared policy.
pub fn verify_merkle_proof(
    leaf: [u8; 32],
    proof: &InclusionProof,
    root: [u8; 32],
    mode: CpuHashMode,
) -> io::Result<bool> {
    use crate::provenance_snapshot_generated::SnapshotVerification;
    let mut value = leaf;
    let mut count = proof.count;
    let mut position = proof.index;
    let mut cursor = 0;
    loop {
        let sibling = proof.siblings.get(cursor);
        let observed_equal = sibling.is_some_and(|text| *text == hexadecimal(value));
        let decision = policy::snapshot_verification_step(
            scalar(count)?,
            scalar(position)?,
            scalar(cursor)?,
            scalar(proof.siblings.len())?,
            observed_equal,
            blake3::Hash::from(value) == blake3::Hash::from(root),
        );
        match decision {
            SnapshotVerification::Verified => return Ok(true),
            SnapshotVerification::Invalid => return Ok(false),
            SnapshotVerification::Hash(left_sibling, next_index, next_count, next_cursor) => {
                let Some(sibling) = sibling else {
                    return Ok(false);
                };
                if sibling.len() != 64
                    || !sibling
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
                {
                    return Ok(false);
                }
                let sibling = blake3::Hash::from_hex(sibling.as_bytes())
                    .map_err(|_| failure("invalid_snapshot_hash"))?;
                let mut input = b"formal-ai-source-node-v1\0".to_vec();
                if left_sibling {
                    input.extend_from_slice(sibling.as_bytes());
                    input.extend_from_slice(&value);
                } else {
                    input.extend_from_slice(&value);
                    input.extend_from_slice(sibling.as_bytes());
                }
                value = digest(&input, mode);
                position = index(next_index, proof.count)?;
                count = index(next_count, proof.count)?;
                cursor = index(next_cursor, proof.siblings.len())?;
            }
        }
    }
}
fn observed_language(file: &str, extensions: &[(String, String)]) -> String {
    policy::snapshot_source_language(
        file.to_owned(),
        file.rsplit('/').next().unwrap_or_default().to_owned(),
        extensions.iter().map(|row| row.0.clone()).collect(),
        extensions.iter().map(|row| row.1.clone()).collect(),
    )
}
/// Snapshot exact file observations, preserving the JavaScript domain separators and grouping.
pub fn source_snapshot(
    files: &[SourceFile],
    extensions: &[(String, String)],
    mode: CpuHashMode,
) -> io::Result<SourceSnapshot> {
    let mut observed = Vec::new();
    let mut paths = std::collections::HashSet::new();
    for file in files {
        if !policy::snapshot_path_eligible(
            file.path.clone(),
            file.path.split('/').map(str::to_owned).collect(),
        ) || !paths.insert(file.path.as_str())
        {
            return Err(failure("invalid_source_file"));
        }
        let language = observed_language(&file.path, extensions);
        let mut input = file.path.as_bytes().to_vec();
        input.push(0);
        input.extend_from_slice(&file.content);
        observed.push((language, file.path.as_str(), digest(&input, mode)));
    }
    observed.sort_by(|left, right| {
        byte_order(&left.0, &right.0).then_with(|| byte_order(left.1, right.1))
    });
    let mut languages = Vec::new();
    let mut language_leaves = Vec::new();
    let mut cursor = 0;
    while cursor < observed.len() {
        let language = observed[cursor].0.clone();
        let mut leaves = Vec::new();
        while cursor < observed.len() && observed[cursor].0 == language {
            leaves.push(observed[cursor].2);
            cursor += 1;
        }
        let root = merkle_root(&leaves, mode)?;
        let mut input =
            format!("formal-ai-language-v1\0{language}\0{}\0", leaves.len()).into_bytes();
        input.extend_from_slice(&root);
        language_leaves.push(digest(&input, mode));
        languages.push(LanguageRoot {
            language,
            file_count: leaves.len(),
            root: hexadecimal(root),
        });
    }
    Ok(SourceSnapshot {
        schema: "formal-ai-source-snapshot-v1",
        languages,
        total: hexadecimal(merkle_root(&language_leaves, mode)?),
    })
}
/// Hash a path, Git mode and original index object identity in index order.
pub fn inventory_leaf(path: &str, mode: &str, object: &str, hashing: CpuHashMode) -> [u8; 32] {
    digest(format!("{path}\0{mode}\0{object}").as_bytes(), hashing)
}
/// Snapshot a physical Git repository after complete bounded metadata preflight.
pub use repository::{SnapshotLimits, snapshot_repository};
