//! Replaying a software project's decisions, in the order they happened, into what web's
//! tables hold: each pull request's stage and recorded merge, who may merge, and the official
//! repository (web-backend's `pull_requests.py`, `merge_capability.py`, `repository.py`).

use serde::{Deserialize, Serialize};

use crate::Stage;

/// Why someone may merge (web's `source_label`).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    /// The author of the leading production or operations plan.
    PlanCreator,
    /// A passed grant vote.
    ApprovedRequest,
    /// The author of a passed repository replacement.
    RepoReplacement,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Merger<A> {
    pub agent: A,
    pub source: Source,
}

/// A recorded merge: the proposal members confirm it with.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Merge<Id, A> {
    pub proposal: Id,
    pub id: String,
    pub url: String,
    pub by: A,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PullRequest<Id, A> {
    pub proposal: Id,
    pub stage: Stage,
    /// The merge waiting for (or given) confirmation; cleared when members reject it.
    pub merge: Option<Merge<Id, A>>,
}

/// One thing that happened, in time order. Votes appear only as their decisions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event<Id, A> {
    /// A production or operations plan took the lead: its author may merge and its
    /// repository is the official one (web's `sync_merge_capability_for_leading_plan`).
    Lead {
        author: A,
        repository: String,
    },
    PullRequest {
        proposal: Id,
    },
    PullRequestDecided {
        proposal: Id,
        passed: bool,
    },
    /// A merge recorded for a pull request; it counts only from a merge-capable member while
    /// the pull request waits for one (web's `record_pull_request_merge`).
    Merge {
        proposal: Id,
        pull_request: Id,
        author: A,
        id: String,
        url: String,
    },
    MergeDecided {
        proposal: Id,
        passed: bool,
    },
    /// A passed grant or revoke vote.
    Capability {
        target: A,
        grant: bool,
    },
    /// A passed repository replacement, triggered by `pull_request`.
    Repository {
        proposal: Id,
        pull_request: Id,
        author: A,
        url: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Software<Id, A> {
    pub repository: String,
    pub mergers: Vec<Merger<A>>,
    /// In the order they were filed.
    pub pull_requests: Vec<PullRequest<Id, A>>,
    /// Merges that didn't count (not from a merger, or the pull request wasn't waiting).
    pub ignored: Vec<Id>,
    /// Passed replacements with the repository each replaced, oldest first.
    pub replaced: Vec<(Id, String)>,
}

impl<Id, A> Default for Software<Id, A> {
    fn default() -> Self {
        Self {
            repository: String::new(),
            mergers: Vec::new(),
            pull_requests: Vec::new(),
            ignored: Vec::new(),
            replaced: Vec::new(),
        }
    }
}

impl<Id: PartialEq, A: PartialEq> Software<Id, A> {
    pub fn can_merge(&self, agent: &A) -> bool {
        self.mergers.iter().any(|m| &m.agent == agent)
    }

    pub fn pull_request(&self, proposal: &Id) -> Option<&PullRequest<Id, A>> {
        self.pull_requests.iter().find(|p| &p.proposal == proposal)
    }

    fn pull_request_mut(&mut self, proposal: &Id) -> Option<&mut PullRequest<Id, A>> {
        self.pull_requests
            .iter_mut()
            .find(|p| &p.proposal == proposal)
    }

    /// web's insert … `on_conflict_do_nothing`.
    fn add(&mut self, agent: A, source: Source) {
        if !self.can_merge(&agent) {
            self.mergers.push(Merger { agent, source });
        }
    }
}

/// Apply `events` (in time order) to an empty project.
#[must_use]
pub fn replay<Id, A>(events: Vec<Event<Id, A>>) -> Software<Id, A>
where
    Id: Clone + PartialEq,
    A: Clone + PartialEq,
{
    let mut out = Software::default();
    for event in events {
        apply(&mut out, event);
    }
    out
}

fn apply<Id, A>(out: &mut Software<Id, A>, event: Event<Id, A>)
where
    Id: Clone + PartialEq,
    A: Clone + PartialEq,
{
    match event {
        Event::Lead { author, repository } => {
            out.repository = repository;
            // web's upsert relabels a member who could already merge.
            out.mergers
                .retain(|m| m.source != Source::PlanCreator && m.agent != author);
            out.mergers.push(Merger {
                agent: author,
                source: Source::PlanCreator,
            });
        }
        Event::PullRequest { proposal } => out.pull_requests.push(PullRequest {
            proposal,
            stage: Stage::Approval,
            merge: None,
        }),
        Event::PullRequestDecided { proposal, passed } => {
            if let Some(pr) = out.pull_request_mut(&proposal) {
                if pr.stage == Stage::Approval {
                    pr.stage = if passed {
                        Stage::AwaitingMerge
                    } else {
                        Stage::Rejected
                    };
                }
            }
        }
        Event::Merge {
            proposal,
            pull_request,
            author,
            id,
            url,
        } => out.merge(
            pull_request,
            Merge {
                proposal,
                id,
                url,
                by: author,
            },
        ),
        Event::MergeDecided { proposal, passed } => out.merge_decided(&proposal, passed),
        Event::Capability { target, grant } => {
            if grant {
                out.add(target, Source::ApprovedRequest);
            } else {
                out.mergers.retain(|m| m.agent != target);
            }
        }
        Event::Repository {
            proposal,
            pull_request,
            author,
            url,
        } => out.replace(proposal, &pull_request, author, url),
    }
}

impl<Id: Clone + PartialEq, A: Clone + PartialEq> Software<Id, A> {
    fn merge(&mut self, pull_request: Id, merge: Merge<Id, A>) {
        let merger = self.can_merge(&merge.by);
        match self.pull_request_mut(&pull_request) {
            Some(pr) if merger && pr.stage == Stage::AwaitingMerge => {
                pr.stage = Stage::Confirmation;
                pr.merge = Some(merge);
            }
            _ => self.ignored.push(merge.proposal),
        }
    }

    fn merge_decided(&mut self, proposal: &Id, passed: bool) {
        let confirming = self.pull_requests.iter_mut().find(|p| {
            p.stage == Stage::Confirmation
                && p.merge.as_ref().is_some_and(|m| &m.proposal == proposal)
        });
        if let Some(pr) = confirming {
            if passed {
                pr.stage = Stage::Confirmed;
            } else {
                pr.stage = Stage::AwaitingMerge;
                pr.merge = None;
            }
        }
    }

    fn replace(&mut self, proposal: Id, pull_request: &Id, author: A, url: String) {
        let previous = std::mem::replace(&mut self.repository, url);
        self.replaced.push((proposal, previous));
        if let Some(pr) = self.pull_request_mut(pull_request) {
            pr.stage = Stage::Replaced;
        }
        self.mergers
            .retain(|m| !matches!(m.source, Source::PlanCreator | Source::RepoReplacement));
        self.add(author, Source::RepoReplacement);
    }
}
