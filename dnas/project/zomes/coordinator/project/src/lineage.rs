//! Convert (ADR 0106): a converted project and the proposal continuing its work name each
//! other with a `Lineage` link on their own space; the tag says which way.

use hdk::prelude::*;
use project_integrity::LinkTypes;
use space_scope::{enter, In};

const CONTINUES_FROM: u8 = 0;
const CONTINUED_BY: u8 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct LineageView {
    /// Closed by a passed Convert proposal.
    pub converted: bool,
    /// The project this one continues.
    pub continues: Option<String>,
    /// The proposal that continues this one's work.
    pub continued_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LineageInput {
    /// The other project's id.
    pub other: String,
    /// True on the converted project (it is continued by `other`); false on the new one.
    pub continued_by: bool,
}

pub fn read(converted: bool) -> ExternResult<LineageView> {
    let mut view = LineageView {
        converted,
        ..LineageView::default()
    };
    let mut found = crate::links(LinkTypes::Lineage)?;
    found.sort_by_key(|l| l.timestamp);
    for link in found {
        let Some((&way, id)) = link.tag.0.split_first() else {
            continue;
        };
        let id = String::from_utf8_lossy(id).into_owned();
        match way {
            CONTINUES_FROM => view.continues = Some(id),
            CONTINUED_BY => view.continued_by = Some(id),
            _ => {}
        }
    }
    Ok(view)
}

/// Name the other project of a conversion on this space.
pub fn link(call: In<LineageInput>) -> ExternResult<ActionHash> {
    let input = enter(call)?;
    let base = space_scope::base()?;
    let mut tag = vec![if input.continued_by {
        CONTINUED_BY
    } else {
        CONTINUES_FROM
    }];
    tag.extend(input.other.as_bytes());
    create_link(base.clone(), base, LinkTypes::Lineage, LinkTag::new(tag))
}
