//! Who holds each role, without a server to refuse the last place twice (ADR 0056).

/// One live assignment as the view reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Taken<K> {
    /// The person's key.
    pub agent: K,
    pub role: usize,
    /// The action's timestamp, then its hash, order the takes.
    pub at: i64,
    pub hash: K,
    /// False for someone who left the project (their roles in upcoming activities lapse).
    pub member: bool,
}

/// The assignments that count, per role: in take order, each person's first one only, and
/// at most each role's `maximum`. Everything else (a second role from another device, a place
/// taken twice at once, someone who left) doesn't count.
#[must_use]
pub fn fill<K: Ord + Clone>(mut taken: Vec<Taken<K>>, maxima: &[Option<u32>]) -> Vec<Vec<K>> {
    taken.sort_by(|a, b| a.at.cmp(&b.at).then_with(|| a.hash.cmp(&b.hash)));
    let mut roles: Vec<Vec<K>> = vec![Vec::new(); maxima.len()];
    let mut seen: Vec<K> = Vec::new();
    for t in taken {
        if !t.member || t.role >= maxima.len() || seen.contains(&t.agent) {
            continue;
        }
        let full = maxima[t.role].is_some_and(|m| roles[t.role].len() >= m as usize);
        if !full {
            seen.push(t.agent.clone());
            roles[t.role].push(t.agent);
        }
    }
    roles
}

#[cfg(test)]
mod tests {
    use super::*;

    fn take(agent: &'static str, role: usize, at: i64) -> Taken<&'static str> {
        Taken {
            agent,
            role,
            at,
            hash: agent,
            member: true,
        }
    }

    #[test]
    fn earliest_takes_win() {
        let taken = vec![take("cat", 0, 3), take("ann", 0, 1), take("bob", 0, 2)];
        assert_eq!(fill(taken, &[Some(2)]), vec![vec!["ann", "bob"]]);
    }

    #[test]
    fn one_role_per_person_and_members_only() {
        let mut gone = take("dan", 1, 0);
        gone.member = false;
        let taken = vec![
            take("ann", 0, 1),
            take("ann", 1, 2),
            gone,
            take("bob", 1, 3),
        ];
        assert_eq!(fill(taken, &[None, None]), vec![vec!["ann"], vec!["bob"]]);
    }

    #[test]
    fn a_full_role_leaves_room_elsewhere() {
        let taken = vec![take("ann", 0, 1), take("bob", 0, 2), take("bob", 1, 3)];
        assert_eq!(
            fill(taken, &[Some(1), None]),
            vec![vec!["ann"], vec!["bob"]]
        );
    }
}
