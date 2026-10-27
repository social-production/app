use super::*;

const PRODUCTIVE: Shape<'static> = Shape {
    mode: "productive",
    subtype: "standard",
    include_access: false,
};

fn candidate(id: &'static str, yes: u32, no: u32, ratings: &[&[u8]]) -> Candidate<&'static str> {
    let per: Vec<Ratings> = ratings.iter().map(|r| summarize(r)).collect();
    Candidate {
        id,
        yes,
        no,
        score: Score::of(&per),
    }
}

#[test]
fn kinds_follow_mode_and_phase() {
    assert_eq!(
        PlanKind::for_phase("productive", Phase::P2),
        Some(PlanKind::Production)
    );
    assert_eq!(
        PlanKind::for_phase("productive", Phase::P3),
        Some(PlanKind::Distribution)
    );
    assert_eq!(
        PlanKind::for_phase("collective-service", Phase::P2),
        Some(PlanKind::Organisation)
    );
    assert_eq!(PlanKind::for_phase("personal-service", Phase::P2), None);
    assert_eq!(
        PlanKind::for_phase("event", Phase::P2),
        Some(PlanKind::Event)
    );
    assert_eq!(PlanKind::for_phase("event", Phase::P5), None);
    let ids: Vec<&str> = criteria(PlanKind::Event, "standard")
        .iter()
        .map(|c| c.id)
        .collect();
    assert_eq!(
        ids,
        [
            "rubric:description-clarity",
            "rubric:achievability",
            "rubric:timing-suitable",
            "rubric:location-appropriate"
        ]
    );
    assert_eq!(PlanKind::for_phase("productive", Phase::P1), None);
    assert!(!PlanKind::Access.allowed("productive"));
    assert_eq!(
        PlanKind::parse("organisation"),
        Some(PlanKind::Organisation)
    );
}

#[test]
fn gate_matches_web() {
    assert_eq!(required_plan_kind(PRODUCTIVE, Phase::P1), None);
    assert_eq!(
        required_plan_kind(PRODUCTIVE, Phase::P2),
        Some(PlanKind::Production)
    );
    assert_eq!(
        required_plan_kind(PRODUCTIVE, Phase::P3),
        Some(PlanKind::Distribution)
    );
    assert_eq!(required_plan_kind(PRODUCTIVE, Phase::P5), None);
    let software = Shape {
        subtype: "software",
        ..PRODUCTIVE
    };
    assert_eq!(required_plan_kind(software, Phase::P3), None);
    let collective = Shape {
        mode: "collective-service",
        ..PRODUCTIVE
    };
    assert_eq!(
        required_plan_kind(collective, Phase::P2),
        Some(PlanKind::Organisation)
    );
    assert_eq!(
        required_plan_kind(collective, Phase::P3),
        Some(PlanKind::Access)
    );
}

#[test]
fn rubric_matches_web() {
    let ids = |k, s| criteria(k, s).iter().map(|c| c.id).collect::<Vec<_>>();
    assert_eq!(
        ids(PlanKind::Production, "standard"),
        [
            "rubric:description-clarity",
            "rubric:achievability",
            "rubric:production-approach"
        ]
    );
    assert_eq!(ids(PlanKind::Production, "software").len(), 4);
    assert_eq!(
        ids(PlanKind::Distribution, "standard")[2],
        "rubric:access-approach"
    );
    assert_eq!(ids(PlanKind::Access, "standard").len(), 2);
}

#[test]
fn ratings_summarize_like_web() {
    let r = summarize(&[5, 4, 4, 9, 0]);
    assert_eq!((r.count, r.sum, r.distribution), (3, 13, [0, 0, 0, 2, 1]));
    assert_eq!(r.average_x100(), 433, "13/3 = 4.33");
    assert_eq!(summarize(&[1, 2]).average_x100(), 150);
    assert_eq!(Ratings::default().average_x100(), 0);
    let score = Score::of(&[summarize(&[5]), summarize(&[]), summarize(&[4])]);
    assert_eq!(
        score.average_x100(),
        Some(450),
        "unrated criteria don't count"
    );
}

#[test]
fn leader_is_best_ratio_then_rating() {
    let a = candidate("a", 3, 0, &[&[3]]);
    let b = candidate("b", 2, 1, &[&[5]]);
    assert_eq!(pick_leader(&[a.clone(), b]), Leader::One("a"));
    let c = candidate("c", 6, 0, &[&[4]]);
    assert_eq!(
        pick_leader(&[a.clone(), c]),
        Leader::One("c"),
        "same ratio, better rating"
    );
    let d = candidate("d", 1, 0, &[&[3]]);
    assert_eq!(pick_leader(&[a, d]), Leader::Tied(vec!["a", "d"]));
    assert_eq!(pick_leader::<&str>(&[]), Leader::None);
    let unrated = candidate("u", 1, 0, &[]);
    let rated = candidate("r", 1, 0, &[&[1]]);
    assert_eq!(pick_leader(&[unrated, rated]), Leader::One("r"));
}
