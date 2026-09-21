//! Persistence of link-cli's string-to-address map beside a database.
//!
//! Issue #1106: appending to an existing projection means resolving a string
//! the projection already created to the address link-cli gave it. Recomputing
//! those addresses costs a full rebuild, so they are saved here instead. The
//! format is one `address<TAB>name` line per node; names are arbitrary content,
//! so newlines are escaped -- and the escape character with them, without which
//! a name containing a literal backslash-n would read back as a newline.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Path of the saved string-to-address map beside a link-cli database.
#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
pub(super) fn node_address_path(database: &Path) -> PathBuf {
    let mut name = database.as_os_str().to_os_string();
    name.push(".nodes");
    PathBuf::from(name)
}

/// Read a saved string-to-address map, or `None` when it is absent or malformed.
#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
pub(super) fn read_node_addresses(path: &Path) -> Option<BTreeMap<String, usize>> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut nodes = BTreeMap::new();
    for line in text.lines() {
        let (address, node) = line.split_once('\t')?;
        nodes.insert(decode_node_name(node)?, address.parse::<usize>().ok()?);
    }
    Some(nodes)
}

/// Decode one escaped node name, rejecting an escape the encoder cannot emit.
///
/// Scanning left to right is what makes this the exact inverse of the encoder:
/// a `\\` consumes its own escape before the following character is examined,
/// so the two characters `\n` inside a node name never decode as a newline.
#[cfg(all(not(target_arch = "wasm32"), feature = "doublets-native"))]
fn decode_node_name(encoded: &str) -> Option<String> {
    let mut decoded = String::with_capacity(encoded.len());
    let mut characters = encoded.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            decoded.push(character);
            continue;
        }
        match characters.next()? {
            'n' => decoded.push('\n'),
            '\\' => decoded.push('\\'),
            _ => return None,
        }
    }
    Some(decoded)
}
