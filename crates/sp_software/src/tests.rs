use crate::{
    check_merge, check_pull_request, check_repository, replay, takes_software, Event, Merger,
    Source, Stage,
};

type E = Event<u32, char>;

fn lead(author: char, repository: &str) -> E {
    Event::Lead {
        author,
        repository: repository.into(),
    }
}

fn merge(proposal: u32, pull_request: u32, author: char) -> E {
    Event::Merge {
        proposal,
        pull_request,
        author,
        id: "abc123".into(),
        url: "https://example.org/commit/abc123".into(),
    }
}

fn decided(proposal: u32, passed: bool) -> E {
    Event::PullRequestDecided { proposal, passed }
}

#[test]
fn fields_are_required_and_bounded() {
    assert!(check_pull_request("Fix", "Fixes the build.", "#12", "https://x.org/pr/12").is_ok());
    assert!(check_pull_request(" ", "s", "#1", "u").is_err());
    assert!(check_pull_request("t", "s", &"9".repeat(121), "u").is_err());
    assert!(check_merge("abc", "").is_err());
    assert!(check_repository("https://x.org/new", "The old host closed.").is_ok());
    assert!(check_repository("https://x.org/new", "").is_err());
    assert!(takes_software("productive") && takes_software("collective-service"));
    assert!(!takes_software("event") && !takes_software("personal-service"));
}

#[test]
fn a_pull_request_is_voted_merged_and_confirmed() {
    let out = replay(vec![
        lead('a', "https://x.org/repo"),
        Event::PullRequest { proposal: 1 },
        decided(1, true),
        merge(2, 1, 'a'),
    ]);
    assert_eq!(out.repository, "https://x.org/repo");
    let pr = out.pull_request(&1).expect("filed");
    assert_eq!(pr.stage, Stage::Confirmation);
    assert_eq!(pr.merge.as_ref().map(|m| m.by), Some('a'));

    let mut events = vec![
        lead('a', ""),
        Event::PullRequest { proposal: 1 },
        decided(1, true),
        merge(2, 1, 'a'),
    ];
    events.push(Event::MergeDecided {
        proposal: 2,
        passed: true,
    });
    assert_eq!(replay(events).pull_requests[0].stage, Stage::Confirmed);
}

#[test]
fn a_rejected_confirmation_waits_for_another_merge() {
    let out = replay(vec![
        lead('a', ""),
        Event::PullRequest { proposal: 1 },
        decided(1, true),
        merge(2, 1, 'a'),
        Event::MergeDecided {
            proposal: 2,
            passed: false,
        },
        merge(3, 1, 'a'),
    ]);
    let pr = &out.pull_requests[0];
    assert_eq!(pr.stage, Stage::Confirmation);
    assert_eq!(pr.merge.as_ref().map(|m| m.proposal), Some(3));
}

#[test]
fn only_a_merger_records_a_merge_and_only_once_approved() {
    let out = replay(vec![
        lead('a', ""),
        Event::PullRequest { proposal: 1 },
        merge(2, 1, 'a'),
        decided(1, true),
        merge(3, 1, 'b'),
    ]);
    assert_eq!(out.pull_requests[0].stage, Stage::AwaitingMerge);
    assert_eq!(out.ignored, vec![2, 3]);
    let rejected = replay(vec![Event::PullRequest { proposal: 1 }, decided(1, false)]);
    assert_eq!(rejected.pull_requests[0].stage, Stage::Rejected);
}

#[test]
fn grants_revokes_and_a_new_lead() {
    let out = replay(vec![
        lead('a', ""),
        Event::Capability {
            target: 'b',
            grant: true,
        },
        Event::Capability {
            target: 'a',
            grant: false,
        },
    ]);
    assert_eq!(
        out.mergers,
        vec![Merger {
            agent: 'b',
            source: Source::ApprovedRequest
        }]
    );
    // A new leading plan replaces the old plan's author and relabels one who could merge.
    let out = replay(vec![
        lead('a', "one"),
        Event::Capability {
            target: 'b',
            grant: true,
        },
        lead('b', "two"),
    ]);
    assert_eq!(out.repository, "two");
    assert_eq!(
        out.mergers,
        vec![Merger {
            agent: 'b',
            source: Source::PlanCreator
        }]
    );
}

#[test]
fn a_replacement_moves_the_repository_and_who_may_merge() {
    let out = replay(vec![
        lead('a', "https://x.org/old"),
        Event::Capability {
            target: 'c',
            grant: true,
        },
        Event::PullRequest { proposal: 1 },
        decided(1, true),
        Event::Repository {
            proposal: 2,
            pull_request: 1,
            author: 'b',
            url: "https://x.org/new".into(),
        },
    ]);
    assert_eq!(out.repository, "https://x.org/new");
    assert_eq!(out.replaced, vec![(2, "https://x.org/old".to_string())]);
    assert_eq!(out.pull_requests[0].stage, Stage::Replaced);
    let agents: Vec<char> = out.mergers.iter().map(|m| m.agent).collect();
    assert_eq!(agents, vec!['c', 'b']);
    assert_eq!(Stage::Replaced.label(), "Replaced");
    assert!(Stage::Confirmation.in_progress() && !Stage::Confirmed.in_progress());
}
