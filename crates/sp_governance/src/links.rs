//! Links between projects (web's Links tab, ADR 0042), ported from web-backend
//! `detail_links.py`. Each project votes on its own half of a request under its own rules;
//! a request is approved once both halves pass and rejected once either can't. Each cell only
//! sees its own half, so the halves are paired here from what the two cells report.

/// web's `relationship_label` limit.
pub const RELATIONSHIP_MAX: usize = 120;
/// web's label when none is given.
pub const DEFAULT_RELATIONSHIP: &str = "Linked";
/// web's summary for a sever request when none is given.
pub const SEVER_SUMMARY: &str = "Propose severing this link.";

use serde::{Deserialize, Serialize};

/// Where one project's half of a request stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideStatus {
    Open,
    Approved,
    Rejected,
}

/// One project's half of a link request, as that project's cell reports it.
#[derive(Clone, Debug, PartialEq)]
pub struct Half<Id> {
    pub id: Id,
    /// `None`: proposed in this project. `Some(s)`: the other project's request `s`, mirrored.
    pub source: Option<Id>,
    /// The create half, in the same project, that this sever request ends.
    pub sever: Option<Id>,
    pub relationship: String,
    pub summary: String,
    pub status: SideStatus,
}

/// For each of `mine`, the index of its counterpart in `theirs` (both oldest first). Halves
/// pair when one names the other as its source and their text matches; severs also need
/// their create halves paired. The earliest match wins, so a duplicate mirror is ignored.
#[must_use]
pub fn pair<Id: PartialEq>(mine: &[Half<Id>], theirs: &[Half<Id>]) -> Vec<Option<usize>> {
    let cites = |a: &Half<Id>, b: &Half<Id>| match (&a.source, &b.source) {
        (None, Some(s)) => *s == a.id,
        (Some(s), None) => *s == b.id,
        _ => false,
    };
    let same_text = |a: &Half<Id>, b: &Half<Id>| {
        a.relationship == b.relationship
            && a.summary == b.summary
            && a.sever.is_none() == b.sever.is_none()
    };
    let mut out: Vec<Option<usize>> = mine
        .iter()
        .map(|m| {
            if m.sever.is_some() {
                return None;
            }
            theirs
                .iter()
                .position(|t| t.sever.is_none() && cites(m, t) && same_text(m, t))
        })
        .collect();
    let creates = out.clone();
    let create_pair = |mine_create: &Id, their_create: &Id| {
        mine.iter()
            .zip(&creates)
            .any(|(m, p)| m.id == *mine_create && p.is_some_and(|i| theirs[i].id == *their_create))
    };
    for (m, slot) in mine.iter().zip(out.iter_mut()) {
        let Some(c) = &m.sever else { continue };
        *slot = theirs.iter().position(|t| {
            cites(m, t) && same_text(m, t) && t.sever.as_ref().is_some_and(|tc| create_pair(c, tc))
        });
    }
    out
}

/// A request is approved when both halves pass, rejected once either can't (web's
/// `vote_detail_link_request`), and open otherwise, including while the other project
/// hasn't received its half yet.
#[must_use]
pub fn request_status(mine: SideStatus, theirs: Option<SideStatus>) -> SideStatus {
    match (mine, theirs) {
        (SideStatus::Rejected, _) | (_, Some(SideStatus::Rejected)) => SideStatus::Rejected,
        (SideStatus::Approved, Some(SideStatus::Approved)) => SideStatus::Approved,
        _ => SideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use SideStatus::{Approved, Open, Rejected};

    fn half(id: u8, source: Option<u8>, sever: Option<u8>, status: SideStatus) -> Half<u8> {
        Half {
            id,
            source,
            sever,
            relationship: DEFAULT_RELATIONSHIP.into(),
            summary: "Shared tools".into(),
            status,
        }
    }

    #[test]
    fn halves_pair_by_source_either_way() {
        let a = [half(1, None, None, Open), half(2, Some(20), None, Open)];
        let b = [half(20, None, None, Open), half(10, Some(1), None, Open)];
        assert_eq!(pair(&a, &b), vec![Some(1), Some(0)]);
        assert_eq!(pair(&b, &a), vec![Some(1), Some(0)]);
    }

    #[test]
    fn a_missing_or_altered_mirror_does_not_pair() {
        let a = [half(1, None, None, Open)];
        assert_eq!(pair(&a, &[]), vec![None]);
        let mut altered = half(10, Some(1), None, Open);
        altered.summary = "Something else".into();
        assert_eq!(pair(&a, &[altered]), vec![None]);
    }

    #[test]
    fn the_earliest_mirror_wins() {
        let a = [half(1, None, None, Open)];
        let b = [half(10, Some(1), None, Open), half(11, Some(1), None, Open)];
        assert_eq!(pair(&a, &b), vec![Some(0)]);
    }

    #[test]
    fn severs_pair_only_across_paired_links() {
        let a = [half(1, None, None, Approved), half(2, None, Some(1), Open)];
        let b = [
            half(10, Some(1), None, Approved),
            half(11, None, None, Open),
            half(12, Some(2), Some(10), Open),
            half(13, Some(2), Some(11), Open),
        ];
        assert_eq!(pair(&a, &b), vec![Some(0), Some(2)]);
        // A sever citing an unpaired create doesn't pair.
        assert_eq!(pair(&a, &b[1..2]), vec![None, None]);
    }

    #[test]
    fn both_sides_must_approve() {
        assert_eq!(request_status(Approved, Some(Approved)), Approved);
        assert_eq!(request_status(Approved, Some(Open)), Open);
        assert_eq!(request_status(Approved, None), Open);
        assert_eq!(request_status(Open, Some(Rejected)), Rejected);
        assert_eq!(request_status(Rejected, None), Rejected);
    }
}
