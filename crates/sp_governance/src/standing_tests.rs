use super::*;

const NOW: i64 = 1_800_000_000_000_000;

fn vouch(voucher: &'static str, vouchee: &'static str, at: i64) -> VouchFact<&'static str> {
    VouchFact {
        voucher,
        vouchee,
        at,
    }
}

#[test]
fn only_full_votes_and_vouches() {
    assert!(StandingLevel::Full.can_vote());
    assert!(StandingLevel::Full.can_vouch());
    assert!(!StandingLevel::New.can_vote());
    assert!(!StandingLevel::Paused.can_vouch());
}

#[test]
fn two_distinct_vouchers_grant_full() {
    let v = [vouch("ann", "eve", NOW - 10), vouch("bob", "eve", NOW - 5)];
    assert_eq!(check_vouch_evidence(&"eve", &v, NOW), Ok(()));
}

#[test]
fn same_voucher_twice_is_not_enough() {
    let v = [vouch("ann", "eve", NOW - 10), vouch("ann", "eve", NOW - 5)];
    assert_eq!(
        check_vouch_evidence(&"eve", &v, NOW),
        Err(StandingError::NotEnoughVouchers { distinct: 1 })
    );
}

#[test]
fn rejects_borrowed_self_and_future_vouches() {
    let borrowed = [vouch("ann", "zed", NOW - 1), vouch("bob", "eve", NOW - 1)];
    assert_eq!(
        check_vouch_evidence(&"eve", &borrowed, NOW),
        Err(StandingError::WrongVouchee)
    );
    let selfie = [vouch("eve", "eve", NOW - 1), vouch("bob", "eve", NOW - 1)];
    assert_eq!(
        check_vouch_evidence(&"eve", &selfie, NOW),
        Err(StandingError::SelfVouch)
    );
    let future = [vouch("ann", "eve", NOW), vouch("bob", "eve", NOW - 1)];
    assert_eq!(
        check_vouch_evidence(&"eve", &future, NOW),
        Err(StandingError::VouchAfterGrant)
    );
}

#[test]
fn vouch_cap_is_five_per_thirty_days() {
    let four: Vec<i64> = (1..=4).map(|d| NOW - d * DAY_US).collect();
    assert_eq!(check_vouch_cap(&four, NOW), Ok(()));
    let five: Vec<i64> = (1..=5).map(|d| NOW - d * DAY_US).collect();
    assert_eq!(
        check_vouch_cap(&five, NOW),
        Err(StandingError::VouchCapReached)
    );
    let old: Vec<i64> = (31..=40).map(|d| NOW - d * DAY_US).collect();
    assert_eq!(
        check_vouch_cap(&old, NOW),
        Ok(()),
        "vouches older than 30 days don't count"
    );
}

#[test]
fn steady_activity_needs_age_and_ten_days() {
    let start = NOW - 40 * DAY_US;
    let ten_days: Vec<i64> = (0..10).map(|d| start + d * DAY_US).collect();
    assert_eq!(check_steady_activity(start, &ten_days, NOW), Ok(()));

    let same_day: Vec<i64> = (0..50).map(|i| start + i).collect();
    assert_eq!(
        check_steady_activity(start, &same_day, NOW),
        Err(StandingError::TooFewActiveDays { days: 1 })
    );

    let young = NOW - 29 * DAY_US;
    assert_eq!(
        check_steady_activity(young, &ten_days, NOW),
        Err(StandingError::ChainTooYoung)
    );
}
