//! Append-only local audit ledger with a deterministic hash chain.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::hash_map::DefaultHasher,
    fs::{self, OpenOptions},
    hash::{Hash, Hasher},
    io::{self, Write},
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecord {
    pub sequence: u64,
    pub event: String,
    pub detail: String,
    pub previous_hash: String,
    pub hash: String,
}

fn legacy_digest(sequence: u64, event: &str, detail: &str, previous: &str) -> String {
    let mut h = DefaultHasher::new();
    (sequence, event, detail, previous).hash(&mut h);
    format!("{:016x}", h.finish())
}

fn digest(sequence: u64, event: &str, detail: &str, previous: &str) -> String {
    let mut hasher = Sha256::new();
    for value in [
        sequence.to_string(),
        event.into(),
        detail.into(),
        previous.into(),
    ] {
        hasher.update((value.len() as u64).to_be_bytes());
        hasher.update(value.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// Reads every record.
///
/// # Errors
/// Returns an error for unreadable or malformed ledgers.
pub fn read(path: &Path) -> io::Result<Vec<AuditRecord>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let text = fs::read_to_string(path)?;
    text.lines()
        .map(|line| serde_json::from_str(line).map_err(io::Error::other))
        .collect()
}

#[must_use]
pub fn verify(records: &[AuditRecord]) -> bool {
    let mut previous = String::new();
    for (index, record) in records.iter().enumerate() {
        if record.sequence != index as u64
            || record.previous_hash != previous
            || record.hash
                != if record.hash.len() == 16 {
                    legacy_digest(record.sequence, &record.event, &record.detail, &previous)
                } else {
                    digest(record.sequence, &record.event, &record.detail, &previous)
                }
        {
            return false;
        }
        previous.clone_from(&record.hash);
    }
    true
}

/// Appends one verified record.
///
/// # Errors
/// Returns an error for an invalid chain or failed durable write.
pub fn append(path: &Path, event: &str, detail: &str) -> io::Result<AuditRecord> {
    let records = read(path)?;
    if !verify(&records) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "audit chain is invalid",
        ));
    }
    let previous_hash = records.last().map(|r| r.hash.clone()).unwrap_or_default();
    let sequence = records.len() as u64;
    let record = AuditRecord {
        sequence,
        event: event.into(),
        detail: detail.into(),
        hash: digest(sequence, event, detail, &previous_hash),
        previous_hash,
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, &record).map_err(io::Error::other)?;
    writeln!(file)?;
    file.sync_data()?;
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_tampering() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("audit.jsonl");
        append(&p, "start", "safe").unwrap();
        append(&p, "tool", "read").unwrap();
        assert!(verify(&read(&p).unwrap()));
        let mut r = read(&p).unwrap();
        assert_eq!(r[0].hash.len(), 64);
        r[0].detail = "changed".into();
        assert!(!verify(&r));
    }

    #[test]
    fn extends_legacy_chains_with_sha256_records() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("audit.jsonl");
        let legacy = AuditRecord {
            sequence: 0,
            event: "legacy".into(),
            detail: "record".into(),
            previous_hash: String::new(),
            hash: legacy_digest(0, "legacy", "record", ""),
        };
        fs::write(
            &path,
            format!("{}\n", serde_json::to_string(&legacy).unwrap()),
        )
        .unwrap();
        assert!(verify(&read(&path).unwrap()));
        let next = append(&path, "current", "record").unwrap();
        assert_eq!(next.hash.len(), 64);
        assert!(verify(&read(&path).unwrap()));
    }
}
