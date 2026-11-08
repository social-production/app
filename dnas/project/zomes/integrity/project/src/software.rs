//! Software projects (ADR 0086): pull requests, recorded merges, merge capability and the
//! repository are proposals voted like any other. Integrity checks their fields and that a
//! merge or replacement cites a pull request of the same space; who may record a merge, and
//! whether the pull request was waiting for one, depend on earlier decisions, so the
//! governance coordinator's replay (`sp_software::replay`) decides those.

use hdi::prelude::*;

use crate::{typed_entry, EntryTypes, ProposalKind};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SoftwareRequest {
    /// web's "Submit pull request": an external pull request members approve or reject.
    PullRequest {
        title: String,
        summary: String,
        pull_request_id: String,
        pull_request_url: String,
    },
    /// web's "Record merge": its vote is the merge confirmation.
    Merge {
        pull_request: ActionHash,
        merge_id: String,
        merge_url: String,
    },
    /// Grant or revoke who may record merges.
    MergeCapability { agent: AgentPubKey, grant: bool },
    /// Move the project to another repository, triggered by a pull request.
    Repository {
        repository_url: String,
        reason: String,
        pull_request: ActionHash,
    },
}

pub fn check(request: &SoftwareRequest) -> Result<(), String> {
    match request {
        SoftwareRequest::PullRequest {
            title,
            summary,
            pull_request_id,
            pull_request_url,
        } => sp_software::check_pull_request(title, summary, pull_request_id, pull_request_url),
        SoftwareRequest::Merge {
            merge_id,
            merge_url,
            ..
        } => sp_software::check_merge(merge_id, merge_url),
        SoftwareRequest::MergeCapability { .. } => Ok(()),
        SoftwareRequest::Repository {
            repository_url,
            reason,
            ..
        } => sp_software::check_repository(repository_url, reason),
    }
}

/// A merge or a replacement cites a pull request proposed in the same space.
pub fn check_cited(
    request: &SoftwareRequest,
    space: &ActionHash,
) -> ExternResult<Result<(), String>> {
    let (SoftwareRequest::Merge {
        pull_request: cited,
        ..
    }
    | SoftwareRequest::Repository {
        pull_request: cited,
        ..
    }) = request
    else {
        return Ok(Ok(()));
    };
    let ok = match typed_entry(&must_get_valid_record(cited.clone())?)? {
        Some(EntryTypes::Proposal(p)) => {
            &p.space == space
                && matches!(
                    p.kind,
                    ProposalKind::Software(SoftwareRequest::PullRequest { .. })
                )
        }
        _ => false,
    };
    Ok(if ok {
        Ok(())
    } else {
        Err("a merge or replacement cites a pull request of this project".into())
    })
}
