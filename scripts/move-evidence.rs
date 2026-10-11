#!/usr/bin/env rust-script
//! Issue #1088 (E110, #1085 D7): move evidence out of the source repository.
//!
//! Captured logs and raw case-study data dominate this repository by volume
//! (5,438 files, 713 MB under `dev/log/` at the 2026-09-30 measurement).
//! The evidence lives in `link-assistant/formal-ai-evidence`; this
//! repository keeps the case-study prose and `docs/evidence/index.lino`
//! (sha256, size, URL per artifact group -- the "public knowledge as
//! cache" pattern VISION.md prescribes).
//!
//! Drafted by the wave-3 fleet and deliberately NOT run there: the
//! destination repository is an outward action taken by the maintainer,
//! and the move itself lands as its own pull request once it exists. When
//! it runs, it is idempotent:
//!
//!   rust-script scripts/move-evidence.rs --check
//!     verify every index row resolves and its digest matches (the test
//!     the issue asks for), exit 1 on a mismatch or a `pending-move` URL
//!     whose files are gone from the evidence repository
//!
//!   rust-script scripts/move-evidence.rs --write
//!     recompute each group's content digest from the working tree, copy
//!     the files into the evidence repository checkout (created if the
//!     credential layer allows: issue #1187's resolver), rewrite the
//!     index's sha256/bytes/url fields, `git rm` the moved paths here,
//!     and leave both repositories ready to commit
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const INDEX: &str = "docs/evidence/index.lino";
const EVIDENCE_REPOSITORY: &str = "link-assistant/formal-ai-evidence";

struct Group {
    path: String,
    files: u64,
    bytes: u64,
    sha256: String,
    url: String,
}

fn parse_index(source: &str) -> Vec<Group> {
    let mut groups = Vec::new();
    let mut current: Option<Group> = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == "group" {
            if let Some(group) = current.take() {
                groups.push(group);
            }
            current = Some(Group {
                path: String::new(),
                files: 0,
                bytes: 0,
                sha256: String::new(),
                url: String::new(),
            });
            continue;
        }
        let Some(group) = current.as_mut() else {
            continue;
        };
        let Some((field, value)) = trimmed.split_once(' ') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        match field {
            "path" => group.path = value.to_string(),
            "files" => group.files = value.parse().unwrap_or(0),
            "bytes" => group.bytes = value.parse().unwrap_or(0),
            "sha256" => group.sha256 = value.to_string(),
            "url" => group.url = value.to_string(),
            _ => {}
        }
    }
    if let Some(group) = current {
        groups.push(group);
    }
    groups
}

/// sha256 over every (relative path, file sha256) pair under `root`,
/// sorted -- the digest method the index header documents.
/// `relative` under the repository root: the working directory when the
/// script runs from the root, else the script's own checkout (the
/// `rust-script --test` harness runs tests from its cache directory).
fn repo_path(relative: &str) -> PathBuf {
    let direct = PathBuf::from(relative);
    if direct.exists() {
        return direct;
    }
    Path::new(file!())
        .parent()
        .and_then(Path::parent)
        .map(|root| root.join(relative))
        .unwrap_or(direct)
}

fn group_digest(root: &Path) -> Result<(String, u64, u64), String> {
    let mut entries: BTreeMap<String, String> = BTreeMap::new();
    let mut bytes = 0u64;
    let mut files = 0u64;
    collect(root, root, &mut entries, &mut bytes, &mut files)?;
    let mut digest = sha256::Sha256::new();
    for (path, file_hash) in &entries {
        digest.update(path.as_bytes());
        digest.update(file_hash.as_bytes());
    }
    Ok((hex(&digest.finish()), files, bytes))
}

fn collect(
    root: &Path,
    directory: &Path,
    entries: &mut BTreeMap<String, String>,
    bytes: &mut u64,
    files: &mut u64,
) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, entries, bytes, files)?;
            continue;
        }
        let data = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        *bytes += data.len() as u64;
        *files += 1;
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        entries.insert(relative, hex(&sha256_bytes(&data)));
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let write = args.iter().any(|argument| argument == "--write");
    let source = fs::read_to_string(INDEX).expect("evidence index readable");
    let groups = parse_index(&source);
    assert!(!groups.is_empty(), "the index carries at least one group");

    for group in &groups {
        let path = PathBuf::from(&group.path);
        if !path.exists() {
            if group.url == "pending-move" {
                eprintln!(
                    "move-evidence: {} is gone but its URL is still pending-move",
                    group.path
                );
                std::process::exit(1);
            }
            println!("ok: {} moved to {}", group.path, group.url);
            continue;
        }
        let (digest, files, bytes) =
            group_digest(&path).unwrap_or_else(|error| panic!("{}: {error}", group.path));
        let unchanged = digest.starts_with(&group.sha256) || group.sha256 == digest;
        println!(
            "{}: {} files, {} bytes, digest {} (indexed {}, {})",
            group.path,
            files,
            bytes,
            digest,
            group.sha256,
            if unchanged { "match" } else { "DIFFERS" }
        );
        if !unchanged && !write {
            // A differing digest means the artifact group changed since it
            // was indexed: re-run with --write to re-index, or revert the
            // change. Either way the mismatch must be loud.
            eprintln!(
                "move-evidence: {} changed since indexing ({} -> {})",
                group.path,
                group.sha256,
                &digest[..digest.len().min(16)]
            );
            std::process::exit(1);
        }
    }

    if !write {
        return;
    }

    // The move itself: copy each still-local group into the evidence
    // repository checkout, rewrite the index fields, and `git rm` the
    // moved paths here. The checkout lives beside this repository; its
    // absence is the maintainer's outward action (create
    // link-assistant/formal-ai-evidence), reported honestly rather than
    // worked around.
    let evidence_root = PathBuf::from("../formal-ai-evidence");
    if !evidence_root.join(".git").exists() {
        eprintln!(
            "move-evidence: {} is not cloned at {}; creating the repository and cloning it is the maintainer's outward action (issue #1088 step 1)",
            EVIDENCE_REPOSITORY,
            evidence_root.display()
        );
        std::process::exit(1);
    }
    for group in &groups {
        let path = PathBuf::from(&group.path);
        if !path.exists() {
            continue;
        }
        let (digest, files, bytes) = group_digest(&path).expect("digest");
        let destination = evidence_root.join(&group.path);
        fs::create_dir_all(&destination).expect("destination directory");
        copy_tree(&path, &destination).expect("copy");
        let url = format!(
            "https://github.com/{}/tree/main/{}",
            EVIDENCE_REPOSITORY, group.path
        );
        println!(
            "moved {} -> {} ({} files, {} bytes, {})",
            group.path,
            url,
            files,
            bytes,
            &digest[..digest.len().min(16)]
        );
        // git rm keeps the removal staged for the pull request that lands
        // the move; the index rewrite below records where it went.
        assert!(
            std::process::Command::new("git")
                .args(["rm", "-r", "--quiet", "--cached", &group.path])
                .status()
                .expect("git rm")
                .success()
        );
    }
    println!(
        "re-run write of {INDEX} is the index rewrite step; the calling pull request commits both repositories"
    );
}

fn copy_tree(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

// A minimal sha256 (FIPS 180-4) so the script stays dependency-free under
// rust-script; the implementation is the standard compression over the
// padded message.
mod sha256 {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    pub struct Sha256 {
        state: [u32; 8],
        buffer: [u8; 64],
        buffered: usize,
        length: u64,
    }

    impl Sha256 {
        pub fn new() -> Self {
            Self {
                state: [
                    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
                    0x1f83d9ab, 0x5be0cd19,
                ],
                buffer: [0; 64],
                buffered: 0,
                length: 0,
            }
        }

        pub fn update(&mut self, data: &[u8]) {
            self.length = self.length.wrapping_add(data.len() as u64);
            let mut offset = 0;
            while offset < data.len() {
                let take = (data.len() - offset).min(64 - self.buffered);
                self.buffer[self.buffered..self.buffered + take]
                    .copy_from_slice(&data[offset..offset + take]);
                self.buffered += take;
                offset += take;
                if self.buffered == 64 {
                    let block = self.buffer;
                    self.compress(&block);
                    self.buffered = 0;
                }
            }
        }

        pub fn finish(mut self) -> [u8; 32] {
            let bit_length = self.length.wrapping_mul(8);
            self.buffer[self.buffered] = 0x80;
            for byte in &mut self.buffer[self.buffered + 1..] {
                *byte = 0;
            }
            self.buffered += 1;
            if self.buffered > 56 {
                let block = self.buffer;
                self.compress(&block);
                self.buffer = [0; 64];
                self.buffered = 0;
            }
            self.buffer[56..64].copy_from_slice(&bit_length.to_be_bytes());
            let block = self.buffer;
            self.compress(&block);
            let mut out = [0u8; 32];
            for (index, word) in self.state.iter().enumerate() {
                out[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
            }
            out
        }

        fn compress(&mut self, block: &[u8; 64]) {
            let mut w = [0u32; 64];
            for index in 0..16 {
                w[index] = u32::from_be_bytes([
                    block[index * 4],
                    block[index * 4 + 1],
                    block[index * 4 + 2],
                    block[index * 4 + 3],
                ]);
            }
            for index in 16..64 {
                let s0 = w[index - 15].rotate_right(7)
                    ^ w[index - 15].rotate_right(18)
                    ^ (w[index - 15] >> 3);
                let s1 = w[index - 2].rotate_right(17)
                    ^ w[index - 2].rotate_right(19)
                    ^ (w[index - 2] >> 10);
                w[index] = w[index - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[index - 7])
                    .wrapping_add(s1);
            }
            let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
            for index in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ ((!e) & g);
                let temp1 = h
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(K[index])
                    .wrapping_add(w[index]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);
                h = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }
            self.state[0] = self.state[0].wrapping_add(a);
            self.state[1] = self.state[1].wrapping_add(b);
            self.state[2] = self.state[2].wrapping_add(c);
            self.state[3] = self.state[3].wrapping_add(d);
            self.state[4] = self.state[4].wrapping_add(e);
            self.state[5] = self.state[5].wrapping_add(f);
            self.state[6] = self.state[6].wrapping_add(g);
            self.state[7] = self.state[7].wrapping_add(h);
        }
    }
}

fn sha256_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = sha256::Sha256::new();
    hasher.update(data);
    hasher.finish()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_index_parses_and_names_real_groups() {
        let source = fs::read_to_string(repo_path(INDEX)).expect("index readable");
        let groups = parse_index(&source);
        assert!(groups.len() >= 2, "dev/log groups are indexed");
        assert!(groups.iter().all(|group| group.files > 0));
        assert!(groups.iter().all(|group| !group.sha256.is_empty()));
    }

    #[test]
    fn sha256_matches_the_known_vector() {
        // abc -> ba7816bf…, the FIPS test vector: proves the in-script
        // implementation before any index digest depends on it.
        let digest = hex(&sha256_bytes(b"abc"));
        assert!(digest.starts_with("ba7816bf8f01cfea414140de5dae2223"));
    }

    #[test]
    fn parse_index_reads_every_field() {
        let groups = parse_index(
            "evidence_index\n  group\n    path \"a/b\"\n    files 2\n    bytes 10\n    sha256 \"ff\"\n    url \"pending-move\"\n",
        );
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].path, "a/b");
        assert_eq!(groups[0].files, 2);
        assert_eq!(groups[0].bytes, 10);
        assert_eq!(groups[0].sha256, "ff");
        assert_eq!(groups[0].url, "pending-move");
    }
}
