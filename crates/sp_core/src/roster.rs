//! Who is in a chat, and who manages a group, over time (ADR 0045). Validation checks each
//! write on its own (a manager's write cites how they became manager); readers replay the
//! chat's changes in time order with this to decide what counts.
//!
//! web-backend's rules (`conversations.py`): the creator manages a group; only the manager
//! renames it, adds and removes people; when the manager leaves, the longest-standing member
//! takes over. Here the leaving manager names that successor in their `Leave`.

/// One change to who is in the chat. `A` is a person (an agent key).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change<A> {
    /// `by` brings `agent` in.
    Invite { by: A, agent: A },
    /// `by` takes `agent` out.
    Remove { by: A, agent: A },
    /// `agent` leaves; a leaving manager names who takes over.
    Leave { agent: A, successor: Option<A> },
}

/// The replayed chat. `T` is a time (a timestamp in microseconds).
#[derive(Clone, Debug)]
pub struct Roster<A, T> {
    /// Who is in now, longest-standing first.
    pub members: Vec<A>,
    /// Who manages the group now; `None` once everyone has left.
    pub manager: Option<A>,
    /// From when each manager managed, oldest first; the creator from the start.
    managers: Vec<(Option<T>, Option<A>)>,
    /// When someone went out: (who, from, back in at).
    gaps: Vec<(A, T, Option<T>)>,
}

impl<A: Clone + PartialEq, T: Copy + Ord> Roster<A, T> {
    /// Replay `changes`, which must be sorted by time. Changes by someone who wasn't the
    /// manager at that moment don't count (their write was valid, but they had handed over).
    #[must_use]
    pub fn replay(creator: A, changes: &[(T, Change<A>)]) -> Self {
        let mut roster = Self {
            members: vec![creator.clone()],
            manager: Some(creator.clone()),
            managers: vec![(None, Some(creator))],
            gaps: Vec::new(),
        };
        for (at, change) in changes {
            roster.apply(*at, change);
        }
        roster
    }

    fn apply(&mut self, at: T, change: &Change<A>) {
        match change {
            Change::Invite { by, agent } if self.manages(by) && !self.members.contains(agent) => {
                self.members.push(agent.clone());
                for gap in self
                    .gaps
                    .iter_mut()
                    .filter(|g| &g.0 == agent && g.2.is_none())
                {
                    gap.2 = Some(at);
                }
            }
            Change::Remove { by, agent } if self.manages(by) && by != agent => {
                self.out(at, agent);
            }
            Change::Leave { agent, successor } if self.members.contains(agent) => {
                self.out(at, agent);
                if self.manages(agent) {
                    let next = successor
                        .clone()
                        .filter(|s| self.members.contains(s))
                        .or_else(|| self.members.first().cloned());
                    self.manager.clone_from(&next);
                    self.managers.push((Some(at), next));
                }
            }
            _ => {}
        }
    }

    fn out(&mut self, at: T, agent: &A) {
        if let Some(i) = self.members.iter().position(|m| m == agent) {
            self.members.remove(i);
            self.gaps.push((agent.clone(), at, None));
        }
    }

    fn manages(&self, agent: &A) -> bool {
        self.manager.as_ref() == Some(agent)
    }

    /// Whether `agent`'s write at `at` counts: not while they were out. Before their first
    /// invitation counts as in, since validation already checked they were invited and their
    /// clock may run behind the inviter's.
    #[must_use]
    pub fn in_at(&self, agent: &A, at: T) -> bool {
        !self
            .gaps
            .iter()
            .any(|(who, from, back)| who == agent && *from <= at && back.is_none_or(|b| at < b))
    }

    /// Who managed the group at `at`.
    #[must_use]
    pub fn manager_at(&self, at: T) -> Option<&A> {
        self.managers
            .iter()
            .rev()
            .find(|(from, _)| from.is_none_or(|f| f <= at))
            .and_then(|(_, who)| who.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::{Change, Roster};

    fn invite(by: &'static str, agent: &'static str) -> Change<&'static str> {
        Change::Invite { by, agent }
    }

    #[test]
    fn the_creator_manages_and_invites() {
        let r = Roster::replay(
            "ann",
            &[(1, invite("ann", "bob")), (2, invite("bob", "cat"))],
        );
        assert_eq!(r.members, vec!["ann", "bob"], "only the manager invites");
        assert_eq!(r.manager, Some("ann"));
        assert_eq!(r.manager_at(0), Some(&"ann"));
    }

    #[test]
    fn removed_people_stop_counting_until_invited_again() {
        let changes = [
            (1, invite("ann", "bob")),
            (
                5,
                Change::Remove {
                    by: "ann",
                    agent: "bob",
                },
            ),
            (9, invite("ann", "bob")),
        ];
        let r = Roster::replay("ann", &changes[..2]);
        assert_eq!(r.members, vec!["ann"]);
        assert!(r.in_at(&"bob", 4));
        assert!(!r.in_at(&"bob", 6));
        assert!(r.in_at(&"bob", 0), "before the invitation counts as in");
        let r = Roster::replay("ann", &changes);
        assert!(r.in_at(&"bob", 10) && !r.in_at(&"bob", 7));
        assert_eq!(r.members, vec!["ann", "bob"]);
    }

    #[test]
    fn nobody_removes_themselves_and_only_the_manager_removes() {
        let changes = [
            (1, invite("ann", "bob")),
            (
                2,
                Change::Remove {
                    by: "ann",
                    agent: "ann",
                },
            ),
            (
                3,
                Change::Remove {
                    by: "bob",
                    agent: "ann",
                },
            ),
        ];
        assert_eq!(Roster::replay("ann", &changes).members, vec!["ann", "bob"]);
    }

    #[test]
    fn a_leaving_manager_hands_over() {
        let changes = [
            (1, invite("ann", "bob")),
            (2, invite("ann", "cat")),
            (
                3,
                Change::Leave {
                    agent: "ann",
                    successor: Some("cat"),
                },
            ),
            (4, invite("cat", "dan")),
            (5, invite("ann", "eve")),
        ];
        let r = Roster::replay("ann", &changes);
        assert_eq!(
            r.members,
            vec!["bob", "cat", "dan"],
            "ann's invite after leaving is void"
        );
        assert_eq!(r.manager, Some("cat"));
        assert_eq!(r.manager_at(2), Some(&"ann"));
        assert_eq!(r.manager_at(3), Some(&"cat"));
    }

    #[test]
    fn without_a_successor_the_longest_standing_member_takes_over() {
        let changes = [
            (1, invite("ann", "bob")),
            (2, invite("ann", "cat")),
            (
                3,
                Change::Leave {
                    agent: "ann",
                    successor: Some("zed"),
                },
            ),
            (
                4,
                Change::Leave {
                    agent: "bob",
                    successor: None,
                },
            ),
            (
                5,
                Change::Leave {
                    agent: "cat",
                    successor: None,
                },
            ),
        ];
        let r = Roster::replay("ann", &changes[..3]);
        assert_eq!(r.manager, Some("bob"), "zed isn't in the chat");
        let r = Roster::replay("ann", &changes);
        assert!(r.members.is_empty());
        assert_eq!(r.manager, None);
    }

    #[test]
    fn a_member_who_leaves_keeps_their_earlier_messages() {
        let changes = [
            (1, invite("ann", "bob")),
            (
                4,
                Change::Leave {
                    agent: "bob",
                    successor: None,
                },
            ),
        ];
        let r = Roster::replay("ann", &changes);
        assert!(r.in_at(&"bob", 3) && !r.in_at(&"bob", 4));
        assert_eq!(r.manager, Some("ann"));
    }
}
