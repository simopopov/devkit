use std::collections::{BTreeMap, BTreeSet};

/// Status of a key across environments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyStatus {
    /// Present in all files with the same value.
    Same,
    /// Present in all files but with different values.
    Different,
    /// Missing from at least one file.
    Missing,
}

/// A single row in the diff result.
#[derive(Debug, Clone)]
pub struct DiffRow {
    pub key: String,
    pub status: KeyStatus,
    /// One value per file (None if missing from that file).
    pub values: Vec<Option<String>>,
}

/// Compare multiple env maps and produce a sorted diff table.
pub fn diff_envs(envs: &[BTreeMap<String, String>]) -> Vec<DiffRow> {
    // Collect all keys across all files
    let all_keys: BTreeSet<&String> = envs.iter().flat_map(|m| m.keys()).collect();

    let mut rows = Vec::with_capacity(all_keys.len());

    for key in all_keys {
        let values: Vec<Option<String>> = envs
            .iter()
            .map(|m| m.get(key).cloned())
            .collect();

        let present_count = values.iter().filter(|v| v.is_some()).count();
        let status = if present_count < envs.len() {
            KeyStatus::Missing
        } else {
            // All present -- check if all equal
            let first = values[0].as_ref().unwrap();
            if values.iter().all(|v| v.as_ref() == Some(first)) {
                KeyStatus::Same
            } else {
                KeyStatus::Different
            }
        };

        rows.push(DiffRow {
            key: key.clone(),
            status,
            values,
        });
    }

    // Sort: Missing first, then Different, then Same -- within each group alphabetical by key
    rows.sort_by(|a, b| {
        let ord_a = status_order(&a.status);
        let ord_b = status_order(&b.status);
        ord_a.cmp(&ord_b).then_with(|| a.key.cmp(&b.key))
    });

    rows
}

fn status_order(s: &KeyStatus) -> u8 {
    match s {
        KeyStatus::Missing => 0,
        KeyStatus::Different => 1,
        KeyStatus::Same => 2,
    }
}

/// Returns true if there are any differences (missing or different values).
pub fn has_differences(rows: &[DiffRow]) -> bool {
    rows.iter().any(|r| r.status != KeyStatus::Same)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_diff() {
        let mut a = BTreeMap::new();
        a.insert("KEY1".into(), "val1".into());
        a.insert("KEY2".into(), "val2".into());

        let mut b = BTreeMap::new();
        b.insert("KEY1".into(), "val1".into());
        b.insert("KEY3".into(), "val3".into());

        let rows = diff_envs(&[a, b]);
        // KEY2 missing from b, KEY3 missing from a => Missing
        // KEY1 same in both => Same
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].status, KeyStatus::Missing); // KEY2 or KEY3
    }

    #[test]
    fn test_different_values() {
        let mut a = BTreeMap::new();
        a.insert("DB".into(), "localhost".into());

        let mut b = BTreeMap::new();
        b.insert("DB".into(), "prod-db.example.com".into());

        let rows = diff_envs(&[a, b]);
        assert_eq!(rows[0].status, KeyStatus::Different);
    }

    #[test]
    fn test_three_files() {
        let mut a = BTreeMap::new();
        a.insert("A".into(), "1".into());

        let mut b = BTreeMap::new();
        b.insert("A".into(), "1".into());
        b.insert("B".into(), "2".into());

        let mut c = BTreeMap::new();
        c.insert("A".into(), "1".into());

        let rows = diff_envs(&[a, b, c]);
        assert_eq!(rows.len(), 2);
    }
}
