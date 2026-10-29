//! Where the resident memory sits, for `/readyz` (ADR 0066), from `/proc/self/smaps`: the brk
//! heap (glibc's main arena), executable anonymous maps (wasmer's compiled zomes), other
//! anonymous maps (jemalloc's and glibc's other arenas, wasm memories, thread stacks) and
//! file-backed maps (the binary and libraries). Subtract `heap.residentMb` from `anonMb` to
//! see what lives outside jemalloc. `anonBySize` splits `anonMb` by each mapping's size:
//! thread stacks are 2–8 MB, glibc's arena heaps exactly 64 MB, jemalloc's extents vary, and
//! wasm memories reserve gigabytes. `anonHugeMb` is what transparent huge pages back.

use std::collections::BTreeMap;

pub fn resident_by_kind() -> Option<serde_json::Value> {
    let smaps = std::fs::read_to_string("/proc/self/smaps").ok()?;
    let mut kb: BTreeMap<&str, u64> = BTreeMap::new();
    let mut by_size: BTreeMap<&str, (u64, u32)> = BTreeMap::new();
    let mut kind = "anonMb";
    let mut size = "";
    let mut huge_kb = 0;
    for line in smaps.lines() {
        if let Some(huge) = line.strip_prefix("AnonHugePages:") {
            huge_kb += huge.split_whitespace().next()?.parse::<u64>().ok()?;
        } else if let Some(rss) = line.strip_prefix("Rss:") {
            let value = rss.split_whitespace().next()?.parse::<u64>().ok()?;
            *kb.entry(kind).or_default() += value;
            if kind == "anonMb" {
                let entry = by_size.entry(size).or_default();
                entry.0 += value;
                entry.1 += 1;
            }
        } else if is_mapping(line) {
            kind = classify(line);
            size = size_bucket(line);
        }
    }
    let mut out: BTreeMap<&str, serde_json::Value> = kb
        .into_iter()
        .map(|(k, v)| (k, (v / 1024).into()))
        .collect();
    let sizes: BTreeMap<&str, serde_json::Value> = by_size
        .into_iter()
        .map(|(k, (rss, n))| (k, serde_json::json!({ "rssMb": rss / 1024, "maps": n })))
        .collect();
    out.insert("anonBySize", serde_json::json!(sizes));
    // Transparent huge pages (ADR 0066): with THP "always", every thread stack's first touch
    // can take a whole 2 MB page.
    out.insert("anonHugeMb", (huge_kb / 1024).into());
    Some(serde_json::json!(out))
}

/// A mapping's virtual size, bucketed.
fn size_bucket(header: &str) -> &'static str {
    let range = header.split_whitespace().next().unwrap_or("");
    let (start, end) = range.split_once('-').unwrap_or(("0", "0"));
    let parse = |h: &str| u64::from_str_radix(h, 16).unwrap_or(0);
    let mb = parse(end).saturating_sub(parse(start)) / (1024 * 1024);
    match mb {
        0..=1 => "under2Mb",
        2..=8 => "2to8Mb",
        9..=63 => "9to63Mb",
        64 => "64Mb",
        65..=1023 => "65to1023Mb",
        _ => "1GbAndUp",
    }
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
        assert_eq!(
            size_bucket("7f0000000000-7f0004000000 rw-p 0 00:00 0"),
            "64Mb"
        );
        assert_eq!(
            size_bucket("7f0000000000-7f0000001000 rw-p 0 00:00 0"),
            "under2Mb"
        );
    }

    #[test]
    fn reads_this_process() {
        let maps = resident_by_kind().expect("smaps on Linux");
        assert!(maps["fileMb"].as_u64().is_some());
    }
}
