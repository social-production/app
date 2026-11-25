use super::{Change, Roster};

fn invite(by: &'static str, agent: &'static str) -> Change<&'static str> {
    Change::Invite { by, agent }
}

fn grant(by: &'static str, agent: &'static str, on: bool) -> Change<&'static str> {
    Change::Grant { by, agent, on }
}

#[test]
fn the_creator_manages_and_invites() {
    let r = Roster::replay("ann", &[(1, invite("ann", "bob")), (2, invite("bob", "cat"))]);
    assert_eq!(r.members, vec!["ann", "bob"], "only a manager invites");
    assert_eq!(r.manager(), Some(&"ann"));
    assert!(r.managed_at(&"ann", 0));
}

#[test]
fn a_manager_makes_another_and_both_manage() {
    let changes = [
        (1, invite("ann", "bob")),
        (2, invite("ann", "cat")),
        (3, grant("ann", "bob", true)),
        (4, invite("bob", "dan")),
        (5, grant("cat", "dan", true)),
    ];
    let r = Roster::replay("ann", &changes);
    assert_eq!(r.members, vec!["ann", "bob", "cat", "dan"]);
    assert_eq!(r.managers, vec!["ann", "bob"], "cat isn't a manager");
    assert!(!r.managed_at(&"bob", 2) && r.managed_at(&"bob", 3));
}

#[test]
fn managers_stop_by_stepping_down_being_stopped_or_removed_but_one_stays() {
    let changes = [
        (1, invite("ann", "bob")),
        (2, invite("ann", "cat")),
        (3, grant("ann", "bob", true)),
        (4, grant("ann", "cat", true)),
        (5, grant("bob", "cat", false)),
        (6, Change::Remove { by: "ann", agent: "bob" }),
        (7, grant("ann", "ann", false)),
    ];
    let r = Roster::replay("ann", &changes);
    assert_eq!(r.members, vec!["ann", "cat"]);
    assert_eq!(r.managers, vec!["ann"], "the last manager can't step down");
    assert!(r.managed_at(&"cat", 4) && !r.managed_at(&"cat", 5));
    assert!(!r.managed_at(&"bob", 6));
    assert!(r.stopped_once(&"cat") && !r.stopped_once(&"ann"));
}

#[test]
fn a_leaving_manager_hands_over_only_when_they_were_the_last() {
    let changes = [
        (1, invite("ann", "bob")),
        (2, invite("ann", "cat")),
        (3, grant("ann", "bob", true)),
        (4, Change::Leave { agent: "ann", successor: Some("cat") }),
    ];
    let r = Roster::replay("ann", &changes);
    assert_eq!(r.managers, vec!["bob"], "bob still manages, so cat isn't handed it");
    let changes = [
        (1, invite("ann", "bob")),
        (2, invite("ann", "cat")),
        (3, Change::Leave { agent: "ann", successor: Some("cat") }),
        (4, invite("cat", "dan")),
        (5, invite("ann", "eve")),
    ];
    let r = Roster::replay("ann", &changes);
    assert_eq!(r.members, vec!["bob", "cat", "dan"], "ann's invite after leaving is void");
    assert_eq!(r.managers, vec!["cat"]);
}

#[test]
fn without_a_successor_the_longest_standing_member_takes_over() {
    let changes = [
        (1, invite("ann", "bob")),
        (2, Change::Leave { agent: "ann", successor: Some("zed") }),
        (3, Change::Leave { agent: "bob", successor: None }),
    ];
    let r = Roster::replay("ann", &changes[..2]);
    assert_eq!(r.managers, vec!["bob"], "zed isn't in the chat");
    let r = Roster::replay("ann", &changes);
    assert!(r.members.is_empty() && r.managers.is_empty());
}

#[test]
fn removed_people_stop_counting_until_invited_again() {
    let changes = [
        (1, invite("ann", "bob")),
        (5, Change::Remove { by: "ann", agent: "bob" }),
        (9, invite("ann", "bob")),
    ];
    let r = Roster::replay("ann", &changes);
    assert!(r.in_at(&"bob", 4) && !r.in_at(&"bob", 7) && r.in_at(&"bob", 10));
}
