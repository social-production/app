//! Who is in a chat, and who manages a group, over time (ADRs 0045, 0122). Validation checks
//! each write on its own (a manager's write cites how they became one); readers replay the
//! chat's changes in time order with this to decide what counts.
//!
//! web-backend's rules (`conversations.py`), plus co-managers (owner, 2026-11-25): the creator
//! manages a group; any manager renames it, changes its photo, adds and removes people, and
//! makes another member a manager or stops them being one. A manager who leaves or is removed
//! stops managing; when the last one goes, the successor they named (or the longest-standing
//! member) takes over. Pure: no HDK.

#[cfg(test)]
mod tests;

/// One change to who is in the chat or who manages it. `A` is a person (an agent key).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change<A> {
    /// `by` brings `agent` in.
    Invite { by: A, agent: A },
    /// `by` takes `agent` out.
    Remove { by: A, agent: A },
    /// `agent` leaves; a leaving manager names who takes over.
    Leave { agent: A, successor: Option<A> },
    /// `by` makes `agent` a manager (`on`) or stops them being one.
    Grant { by: A, agent: A, on: bool },
}

/// The replayed chat. `T` is a time (a timestamp in microseconds).
#[derive(Clone, Debug)]
pub struct Roster<A, T> {
    /// Who is in now, longest-standing first.
    pub members: Vec<A>,
    /// Who manages the group now, longest-managing first; empty once everyone has left.
    pub managers: Vec<A>,
    /// Who managed from when, oldest first; the creator from the start.
    history: Vec<(Option<T>, Vec<A>)>,
    /// When someone went out: (who, from, back in at).
    gaps: Vec<(A, T, Option<T>)>,
}

impl<A: Clone + PartialEq, T: Copy + Ord> Roster<A, T> {
    /// Replay `changes`, which must be sorted by time. Changes by someone who wasn't a
    /// manager at that moment don't count (their write was valid, but they had stopped).
    #[must_use]
    pub fn replay(creator: A, changes: &[(T, Change<A>)]) -> Self {
        let mut roster = Self {
            members: vec![creator.clone()],
            managers: vec![creator.clone()],
            history: vec![(None, vec![creator])],
            gaps: Vec::new(),
        };
        for (at, change) in changes {
            roster.apply(*at, change);
        }
        roster
    }

    /// The longest-managing manager now (web's single `created_by`).
    #[must_use]
    pub fn manager(&self) -> Option<&A> {
        self.managers.first()
    }

    fn apply(&mut self, at: T, change: &Change<A>) {
        let before = self.managers.clone();
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
                self.managers.retain(|m| m != agent);
            }
            Change::Leave { agent, successor } if self.members.contains(agent) => {
                self.out(at, agent);
                if self.manages(agent) {
                    self.managers.retain(|m| m != agent);
                    if self.managers.is_empty() {
                        let next = successor
                            .clone()
                            .filter(|s| self.members.contains(s))
                            .or_else(|| self.members.first().cloned());
                        self.managers.extend(next);
                    }
                }
            }
            Change::Grant { by, agent, on: true }
                if self.manages(by) && self.members.contains(agent) && !self.manages(agent) =>
            {
                self.managers.push(agent.clone());
            }
            // Stepping down or stopping someone, as long as someone still manages.
            Change::Grant { by, agent, on: false }
                if self.manages(by) && self.manages(agent) && self.managers.len() > 1 =>
            {
                self.managers.retain(|m| m != agent);
            }
            _ => {}
        }
        if self.managers != before {
            self.history.push((Some(at), self.managers.clone()));
        }
    }

    fn out(&mut self, at: T, agent: &A) {
        if let Some(i) = self.members.iter().position(|m| m == agent) {
            self.members.remove(i);
            self.gaps.push((agent.clone(), at, None));
        }
    }

    /// Whether `agent` manages the group now.
    pub fn manages(&self, agent: &A) -> bool {
        self.managers.contains(agent)
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

    /// Whether `agent` ever stopped managing (so a write of theirs cites how they came back).
    #[must_use]
    pub fn stopped_once(&self, agent: &A) -> bool {
        let mut was = false;
        for (_, who) in &self.history {
            let is = who.contains(agent);
            if was && !is {
                return true;
            }
            was = is;
        }
        false
    }

    /// Whether `agent` managed the group at `at`.
    #[must_use]
    pub fn managed_at(&self, agent: &A, at: T) -> bool {
        self.history
            .iter()
            .rev()
            .find(|(from, _)| from.is_none_or(|f| f <= at))
            .is_some_and(|(_, who)| who.contains(agent))
    }
}
