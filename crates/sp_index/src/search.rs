//! Text search: every query word must appear in one of the item's fields (case-insensitive).

#[must_use]
pub fn matches(query: &str, fields: &[&str]) -> bool {
    let haystack: Vec<String> = fields.iter().map(|f| f.to_lowercase()).collect();
    query
        .split_whitespace()
        .map(str::to_lowercase)
        .all(|word| haystack.iter().any(|field| field.contains(&word)))
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn all_words_must_match_somewhere() {
        let fields = ["Community Garden", "Growing food together", "food,garden"];
        assert!(matches("garden FOOD", &fields));
        assert!(!matches("garden bikes", &fields));
        assert!(matches("   ", &fields));
    }
}
