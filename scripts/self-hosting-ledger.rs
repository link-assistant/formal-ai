//! Bounded, lossless storage for the append-only release ledger.
//! Parts contain complete release records, in order; no evidence is discarded.

use std::fmt::Write as FmtWrite;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub const PARTS: &str = "data/meta/self-hosting-ledger-parts";
const LIMIT: usize = 1_500;
const MAX_PARTS: usize = 128;
const PREFIX: &str = "  archive \"self-hosting-ledger-parts/";

fn read_text(path: &Path) -> Result<String, String> {
    let meta = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !meta.is_file() || meta.len() > 1_048_576 {
        return Err(format!("invalid ledger file {}", path.display()));
    }
    fs::read_to_string(path).map_err(|error| error.to_string())
}

fn parts(text: &str) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    for line in text.lines().filter(|line| line.starts_with("  archive")) {
        let expected = format!("{PREFIX}{:04}.lino\"", names.len() + 1);
        if line != expected || names.len() >= MAX_PARTS {
            return Err("noncanonical or excessive ledger archives".to_owned());
        }
        names.push(format!("{:04}.lino", names.len() + 1));
    }
    Ok(names)
}

pub fn read(ledger: &Path) -> Result<String, String> {
    let text = read_text(ledger)?;
    let directory = ledger.parent().unwrap_or_else(|| Path::new("."));
    let mut result = String::new();
    let names = parts(&text)?;
    let directory = directory.join("self-hosting-ledger-parts");
    if !names.is_empty()
        && !fs::symlink_metadata(&directory)
            .map_err(|error| error.to_string())?
            .is_dir()
    {
        return Err("ledger archive directory must not be a symlink".to_owned());
    }
    for name in names {
        let part = read_text(&directory.join(name))?;
        if !part.starts_with("self-hosting-ledger-archive\n")
            || !parts(&part)?.is_empty()
            || part.lines().count() > LIMIT
        {
            return Err("invalid or nested ledger archive".to_owned());
        }
        result.push_str(&part);
    }
    result.push_str(&text);
    Ok(result)
}

pub fn append(ledger: &Path, record: &str) -> Result<(), String> {
    // Validate all referenced history before changing anything.
    read(ledger)?;
    let text = read_text(ledger)?;
    let mut pending = text.clone();
    pending.push_str(record);
    if pending.lines().count() <= LIMIT {
        return OpenOptions::new()
            .append(true)
            .open(ledger)
            .and_then(|mut file| file.write_all(record.as_bytes()))
            .map_err(|error| error.to_string());
    }
    let offset = text
        .find("  release\n")
        .ok_or("ledger has no release record")?;
    let (header, records) = text.split_at(offset);
    let mut names = parts(header)?;
    let mut chunks = Vec::new();
    let mut chunk = String::from("self-hosting-ledger-archive\n");
    let mut rows: Vec<_> = records.split("  release\n").skip(1).collect();
    let latest = rows.pop().ok_or("ledger has no release record")?;
    for row in rows {
        let row = format!("  release\n{row}");
        if row.lines().count() + 1 > LIMIT {
            return Err("single release record exceeds ledger limit".to_owned());
        }
        if chunk.lines().count() + row.lines().count() > LIMIT {
            chunks.push(chunk);
            chunk = String::from("self-hosting-ledger-archive\n");
        }
        chunk.push_str(&row);
    }
    if chunk != "self-hosting-ledger-archive\n" {
        chunks.push(chunk);
    }
    if names.len() + chunks.len() > MAX_PARTS {
        return Err("ledger archive bound exhausted".to_owned());
    }
    let mut next = header.to_owned();
    for _ in &chunks {
        let _ = writeln!(next, "{PREFIX}{:04}.lino\"", names.len() + 1);
        names.push(String::new());
    }
    next.push_str("  release\n");
    next.push_str(latest);
    next.push_str(record);
    if next.lines().count() > LIMIT {
        return Err("release or archive index exceeds ledger limit".to_owned());
    }
    let directory = ledger.parent().unwrap_or_else(|| Path::new("."));
    let directory = directory.join("self-hosting-ledger-parts");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    if !fs::symlink_metadata(&directory)
        .map_err(|error| error.to_string())?
        .is_dir()
    {
        return Err("ledger archive directory must not be a symlink".to_owned());
    }
    let first = names.len() - chunks.len() + 1;
    for (index, chunk) in chunks.iter().enumerate() {
        // Never overwrite history, including a part left by an interrupted write.
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(format!("{:04}.lino", first + index)))
            .and_then(|mut file| file.write_all(chunk.as_bytes()))
            .map_err(|error| error.to_string())?;
    }
    let pending = ledger.with_extension("lino.pending");
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .and_then(|mut file| {
            file.write_all(next.as_bytes())
                .and_then(|()| file.sync_all())
        })
        .map_err(|error| error.to_string())?;
    fs::rename(pending, ledger).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("formal-ai-ledger-{}-{nonce}", std::process::id()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn ledger(&self) -> std::path::PathBuf {
            self.0.join("self-hosting-ledger.lino")
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn row(index: usize) -> String {
        let mut result = format!("  release\n    tag \"v{index}\"\n");
        for field in 0..12 {
            let _ = writeln!(result, "    field_{field} \"{index}\"");
        }
        result
    }
    fn records(text: &str) -> Vec<&str> {
        text.lines()
            .filter(|line| *line == "  release" || line.starts_with("    "))
            .collect()
    }
    #[test]
    fn repeated_rotation_preserves_all_records_and_latest_root_status() {
        let fixture = Fixture::new();
        let ledger = fixture.ledger();
        fs::write(&ledger, format!("self_hosting_ledger\n{}", row(0))).unwrap();
        let mut expected = row(0);
        for index in 1..=330 {
            let record = row(index);
            append(&ledger, &record).unwrap();
            expected.push_str(&record);
        }
        let actual = read(&ledger).unwrap();
        assert_eq!(records(&actual), records(&expected));
        assert!(
            fs::read_to_string(&ledger)
                .unwrap()
                .contains("tag \"v330\"")
        );
        assert!(fs::read_to_string(&ledger).unwrap().lines().count() <= LIMIT);
        let entries = fs::read_dir(fixture.0.join("self-hosting-ledger-parts")).unwrap();
        let mut count = 0;
        for entry in entries {
            assert!(read_text(&entry.unwrap().path()).unwrap().lines().count() <= LIMIT);
            count += 1;
        }
        assert!(count >= 3);
    }
    #[test]
    fn missing_traversal_duplicate_and_nested_archives_refuse_before_writes() {
        for name in [
            "../outside",
            "0002.lino",
            "0001.lino",
            "0001.lino\"\n  archive \"self-hosting-ledger-parts/0001.lino",
        ] {
            let fixture = Fixture::new();
            let text = format!("self_hosting_ledger\n{PREFIX}{name}\"\n{}", row(1));
            fs::write(fixture.ledger(), &text).unwrap();
            assert!(append(&fixture.ledger(), &row(2)).is_err());
            assert_eq!(fs::read_to_string(fixture.ledger()).unwrap(), text);
        }
        let fixture = Fixture::new();
        fs::write(
            fixture.ledger(),
            format!("self_hosting_ledger\n{PREFIX}0001.lino\"\n{}", row(1)),
        )
        .unwrap();
        let directory = fixture.0.join("self-hosting-ledger-parts");
        fs::create_dir(&directory).unwrap();
        fs::write(
            directory.join("0001.lino"),
            format!(
                "self-hosting-ledger-archive\n{PREFIX}0001.lino\"\n{}",
                row(0)
            ),
        )
        .unwrap();
        assert!(read(&fixture.ledger()).is_err());
    }
    #[test]
    fn interrupted_write_never_overwrites_existing_archive() {
        let fixture = Fixture::new();
        let text = format!(
            "self_hosting_ledger\n{}",
            (0..108).map(row).collect::<String>()
        );
        fs::write(fixture.ledger(), &text).unwrap();
        let directory = fixture.0.join("self-hosting-ledger-parts");
        fs::create_dir(&directory).unwrap();
        let part = directory.join("0001.lino");
        fs::write(&part, "interrupted write\n").unwrap();
        assert!(append(&fixture.ledger(), "").is_err());
        assert_eq!(fs::read_to_string(fixture.ledger()).unwrap(), text);
        assert_eq!(fs::read_to_string(part).unwrap(), "interrupted write\n");
    }
}
