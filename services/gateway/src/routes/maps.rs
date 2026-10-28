//! Where the resident memory sits, for `/readyz` (ADR 0066), from `/proc/self/smaps`: the brk
//! heap (glibc's main arena), executable anonymous maps (wasmer's compiled zomes), other
//! anonymous maps (jemalloc's and glibc's other arenas, wasm memories, thread stacks) and
//! file-backed maps (the binary and libraries). Subtract `heap.residentMb` from `anonMb` to
//! see what lives outside jemalloc.

use std::collections::BTreeMap;

pub fn resident_by_kind() -> Option<serde_json::Value> {
    let smaps = std::fs::read_to_string("/proc/self/smaps").ok()?;
    let mut kb: BTreeMap<&str, u64> = BTreeMap::new();
    let mut kind = "anonMb";
    for line in smaps.lines() {
        if let Some(rss) = line.strip_prefix("Rss:") {
            let value = rss.split_whitespace().next()?.parse::<u64>().ok()?;
            *kb.entry(kind).or_default() += value;
        } else if is_mapping(line) {
            kind = classify(line);
        }
    }
    Some(serde_json::json!(kb
        .into_iter()
        .map(|(k, v)| (k, v / 1024))
        .collect::<BTreeMap<_, _>>()))
}

/// A mapping header: `start-end perms offset dev inode [path]`.
fn is_mapping(line: &str) -> bool {
    line.split_whitespace().next().is_some_and(|range| {
        range.contains('-') && range.bytes().all(|b| b == b'-' || b.is_ascii_hexdigit())
    })
}

fn classify(header: &str) -> &'static str {
    let fields: Vec<&str> = header.split_whitespace().collect();
    let perms = fields.get(1).copied().unwrap_or("");
    match fields.get(5).copied() {
        Some("[heap]") => "brkHeapMb",
        Some(p) if p.starts_with("[stack") => "stackMb",
        Some(p) if p.starts_with('/') => "fileMb",
        _ if perms.contains('x') => "anonExecMb",
        _ => "anonMb",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_mappings() {
        assert_eq!(
            classify("55d0-55d1 rw-p 00000000 00:00 0 [heap]"),
            "brkHeapMb"
        );
        assert_eq!(
            classify("7f00-7f01 r-xp 00000000 08:01 12 /usr/bin/gateway"),
            "fileMb"
        );
        assert_eq!(classify("7f00-7f01 r-xp 00000000 00:00 0"), "anonExecMb");
        assert_eq!(classify("7f00-7f01 rw-p 00000000 00:00 0"), "anonMb");
        assert!(is_mapping("7f00a-7f01b rw-p 00000000 00:00 0"));
        assert!(!is_mapping("Rss:                 4 kB"));
        assert!(!is_mapping("VmFlags: rd wr mr mw me ac sd"));
    }

    #[test]
    fn reads_this_process() {
        let maps = resident_by_kind().expect("smaps on Linux");
        assert!(maps["fileMb"].as_u64().is_some());
    }
}
