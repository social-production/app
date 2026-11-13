use crate::{replay, week_start, Act, Kind, Standing, GRACE_US, WEEK_US};

const ME: u8 = 0;
const DAY: i64 = 24 * 60 * 60 * 1_000_000;

fn volunteer(at: i64, on: bool) -> Act<u8> {
    Act {
        who: ME,
        at,
        kind: Kind::Volunteer(on),
    }
}

fn vote(who: u8, at: i64, v: Option<bool>) -> Act<u8> {
    Act {
        who,
        at,
        kind: Kind::Vote(v),
    }
}

fn yes_votes(from: u8, n: u8, at: i64) -> Vec<Act<u8>> {
    (from..from + n).map(|w| vote(w, at, Some(true))).collect()
}

#[test]
fn a_candidate_with_the_quorum_and_66_percent_yes_is_seated() {
    let mut acts = vec![volunteer(1, true)];
    acts.extend(yes_votes(1, 3, 2));
    let s = replay(&ME, &acts, 10, |_| 3);
    assert_eq!(s.standing, Standing::Moderator);
    assert_eq!(s.seated_at, Some(2));
    assert_eq!((s.stats.yes, s.stats.no), (3, 0));
}

#[test]
fn short_of_the_quorum_or_the_share_stays_a_candidate() {
    let mut acts = vec![volunteer(1, true)];
    acts.extend(yes_votes(1, 2, 3));
    assert_eq!(replay(&ME, &acts, 10, |_| 3).standing, Standing::Candidate);
    // (Votes count one at a time: the no votes come first, or 2 yes + 1 no would seat.)
    acts.extend([vote(5, 2, Some(false)), vote(6, 2, Some(false))]);
    // 2 yes of 4: enough votes, 50% yes.
    assert_eq!(replay(&ME, &acts, 10, |_| 3).standing, Standing::Candidate);
}

#[test]
fn votes_on_someone_not_volunteering_seat_nobody() {
    let acts = yes_votes(1, 5, 2);
    assert_eq!(replay(&ME, &acts, 10, |_| 3).standing, Standing::None);
}

#[test]
fn approval_below_66_percent_removes_a_moderator_at_once() {
    let mut acts = vec![volunteer(1, true)];
    acts.extend(yes_votes(1, 3, 2));
    acts.extend([vote(7, 5, Some(false)), vote(8, 5, Some(false))]);
    let s = replay(&ME, &acts, 10, |_| 3);
    assert_eq!(s.standing, Standing::None);
    assert_eq!(s.removed_at, Some(5));
}

#[test]
fn a_moderator_short_of_the_quorum_gets_a_week_then_is_removed() {
    let mut acts = vec![volunteer(1, true)];
    acts.extend(yes_votes(1, 3, 2));
    // The network grows: from next week three votes are no longer enough.
    let next = week_start(1) + WEEK_US;
    let quorum = move |at: i64| if at >= next { 5 } else { 3 };
    assert_eq!(
        replay(&ME, &acts, next - 1, quorum).standing,
        Standing::Moderator
    );
    // Every reader sees the same grace period, whenever it reads.
    for now in [next, next + DAY, next + GRACE_US] {
        let s = replay(&ME, &acts, now, quorum);
        assert_eq!(
            s.standing,
            Standing::Grace {
                ends: next + GRACE_US
            }
        );
    }
    let s = replay(&ME, &acts, next + GRACE_US + 1, quorum);
    assert_eq!(s.standing, Standing::None);
    assert_eq!(s.removed_at, Some(next + GRACE_US));
}

#[test]
fn reaching_the_quorum_again_ends_the_grace_period() {
    let mut acts = vec![volunteer(1, true)];
    acts.extend(yes_votes(1, 3, 2));
    let next = week_start(1) + WEEK_US;
    let quorum = move |at: i64| if at >= next { 5 } else { 3 };
    assert!(matches!(
        replay(&ME, &acts, next + DAY, quorum).standing,
        Standing::Grace { .. }
    ));
    acts.extend(yes_votes(10, 2, next + 2 * DAY));
    let s = replay(&ME, &acts, next + 3 * DAY, quorum);
    assert_eq!(s.standing, Standing::Moderator);
    assert_eq!(s.stats.votes(), 5);
}

#[test]
fn withdrawing_ends_the_role_and_the_votes_stay_for_next_time() {
    let mut acts = vec![volunteer(1, true)];
    acts.extend(yes_votes(1, 3, 2));
    acts.push(volunteer(5, false));
    let s = replay(&ME, &acts, 10, |_| 3);
    assert_eq!((s.standing, s.removed_at), (Standing::None, Some(5)));
    acts.push(volunteer(8, true));
    // web keeps standing votes after removal: volunteering again seats them at once.
    assert_eq!(replay(&ME, &acts, 10, |_| 3).standing, Standing::Moderator);
}

#[test]
fn own_votes_are_ignored_and_neutral_withdraws_a_vote() {
    let mut acts = vec![volunteer(1, true), vote(ME, 2, Some(true))];
    acts.extend(yes_votes(1, 2, 2));
    acts.push(vote(3, 3, Some(true)));
    acts.push(vote(3, 4, None));
    let s = replay(&ME, &acts, 10, |_| 4);
    assert_eq!(s.standing, Standing::Candidate);
    assert_eq!(s.stats.votes(), 2);
}

#[test]
fn with_no_quorum_a_volunteer_is_seated_at_once() {
    // web: `required_votes(0) == 0`, and zero votes qualify.
    let s = replay(&ME, &[volunteer(1, true)], 10, |_| 0);
    assert_eq!((s.standing, s.seated_at), (Standing::Moderator, Some(1)));
}

#[test]
fn acts_after_now_and_out_of_order_are_handled() {
    let mut acts = yes_votes(1, 3, 2);
    acts.insert(0, vote(9, 50, Some(false)));
    acts.push(volunteer(1, true));
    let s = replay(&ME, &acts, 10, |_| 3);
    assert_eq!(s.standing, Standing::Moderator);
    assert_eq!(s.stats.votes(), 3);
}
