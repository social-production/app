//! The project feed this node serves (`/index/projects`, ADR 0031): one card per listed project, built
//! by the archivist from the directory listing plus the space's live counts in the commons.
//! Non-authoritative (ADR 0012): ranking comes from `sp_index`, the data from the DHT.

use std::sync::Arc;

use serde::Serialize;
use sp_index::{query, FeedQuery, ItemKind, ItemStats};
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCard {
    /// Base64 action hash of the original listing.
    pub id: String,
    pub network_seed: String,
    pub creator: String,
    /// The creator's username (empty if they have no profile yet).
    pub creator_name: String,
    pub title: String,
    pub summary: String,
    pub mode: String,
    pub subtype: String,
    pub location_label: String,
    pub phase: String,
    pub tags: Vec<String>,
    pub members: u32,
    pub demand: u32,
    pub opposition: u32,
    pub created_at: i64,
    pub last_activity_at: i64,
    /// The newest applied update's text (web's `latestDescription`).
    pub latest_update: Option<String>,
    /// Messages in the project chat, up to its page size (web's `commentCount`).
    pub comment_count: u32,
    /// A help request's time and sign-ups (ADR 0067); absent on other cards.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<super::cards::HelpCard>,
    /// An event's start, from its leading plan (web's `scheduledAt`); absent on other cards.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<i64>,
    /// web's `moderationState` when a report is open: `under_review` or `hidden` (ADR 0071).
    /// Removed items leave the feed; absent while nothing is reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moderation: Option<String>,
}

/// The moderation state of an item a report took down.
pub const REMOVED: &str = "removed";

/// Whose items a feed page holds.
pub enum Who<'a> {
    /// The public feed: everyone's items but posts, which live on personal feeds (ADR 0070);
    /// Search asks for posts too (`with_posts`).
    Anyone { with_posts: bool },
    /// These people's items, posts included: a profile's feed.
    Creators(&'a [&'a str]),
    /// web-backend's `get_personal_feed`: everything from the people `me` follows, `me`'s own
    /// posts, and with "Following + popular" (`popular`) everyone else's posts and threads.
    Personal {
        following: &'a [&'a str],
        me: &'a str,
        popular: bool,
    },
}

impl Who<'_> {
    fn keeps(&self, c: &ProjectCard) -> bool {
        let post = c.mode == sp_discussion::POST;
        match self {
            Who::Anyone { with_posts } => !post || *with_posts,
            Who::Creators(creators) => creators.contains(&c.creator.as_str()),
            Who::Personal {
                following,
                me,
                popular,
            } => {
                let followed = following.contains(&c.creator.as_str());
                let mine = c.creator == *me;
                // Not followed past the first test: discovery is everyone else's.
                followed
                    || (post && mine)
                    || (*popular && !mine && sp_discussion::is_discussion(&c.mode))
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ProjectIndex(Arc<RwLock<Vec<ProjectCard>>>);

impl ProjectIndex {
    pub async fn replace(&self, cards: Vec<ProjectCard>) {
        *self.0.write().await = cards;
    }

    /// The cards held now, by listing id (kept for spaces nobody wrote to since).
    pub async fn by_id(&self) -> std::collections::BTreeMap<String, ProjectCard> {
        self.0
            .read()
            .await
            .iter()
            .map(|c| (c.id.clone(), c.clone()))
            .collect()
    }

    /// One page of the feed, best first, of the items `who` keeps; non-empty `tags` keeps only
    /// the items tagged with any of those channels or communities (`#food`, ADR 0068; several
    /// for web's Home scope). Channels and communities themselves aren't feed items.
    pub async fn page(
        &self,
        q: &FeedQuery<'_>,
        who: &Who<'_>,
        tags: &[&str],
        offset: usize,
        limit: usize,
    ) -> (Vec<ProjectCard>, usize) {
        let all = self.0.read().await;
        let cards: Vec<&ProjectCard> = all
            .iter()
            .filter(|c| !sp_scope::is_scope(&c.mode))
            // Removed after a report (ADR 0071): web unlists them everywhere.
            .filter(|c| c.moderation.as_deref() != Some(REMOVED))
            .filter(|c| who.keeps(c))
            .filter(|c| tags.is_empty() || c.tags.iter().any(|have| tags.contains(&have.as_str())))
            .collect();
        let stats: Vec<ItemStats> = cards.iter().map(|c| stats_of(c)).collect();
        let pairs: Vec<(&ProjectCard, &ItemStats)> = cards.into_iter().zip(&stats).collect();
        let ranked = query(
            &pairs,
            q,
            // Help requests, threads and posts rank by votes (their Support/Oppose signals,
            // ADR 0067, 0070), as on web.
            |(c, _)| {
                if c.help.is_some() || sp_discussion::is_discussion(&c.mode) {
                    ItemKind::Voted
                } else {
                    ItemKind::Signalled
                }
            },
            |(_, s)| s,
            |(c, _)| {
                let mut fields = vec![
                    c.title.as_str(),
                    c.summary.as_str(),
                    c.location_label.as_str(),
                ];
                fields.extend(c.tags.iter().map(String::as_str));
                fields
            },
        );
        let total = ranked.len();
        let page = ranked
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|(c, _)| (*c).clone())
            .collect();
        (page, total)
    }

    /// Channels and communities (ADR 0068) whose handle or summary matches `q`, the earliest
    /// listing of each handle only (first come, first served), by handle.
    pub async fn scopes(&self, q: &str, limit: usize) -> Vec<ProjectCard> {
        let needle = q.trim().to_lowercase();
        let all = self.0.read().await;
        let mut first = std::collections::BTreeMap::<String, &ProjectCard>::new();
        for c in all.iter().filter(|c| sp_scope::is_scope(&c.mode)) {
            let key = format!("{}:{}", c.mode, sp_scope::slug(&c.title));
            let earlier = first
                .get(&key)
                .is_some_and(|f| (f.created_at, &f.id) <= (c.created_at, &c.id));
            if !earlier {
                first.insert(key, c);
            }
        }
        first
            .into_values()
            .filter(|c| {
                needle.is_empty()
                    || c.title.to_lowercase().contains(&needle)
                    || c.summary.to_lowercase().contains(&needle)
            })
            .take(limit)
            .cloned()
            .collect()
    }
}

fn stats_of(card: &ProjectCard) -> ItemStats {
    ItemStats {
        support: card.demand,
        oppose: card.opposition,
        members: card.members,
        created_at: card.created_at,
        last_activity_at: card.last_activity_at,
        votes: i32::try_from(card.demand).unwrap_or(i32::MAX)
            - i32::try_from(card.opposition).unwrap_or(i32::MAX),
        comments: card.comment_count,
    }
}

#[cfg(test)]
#[path = "index_tests.rs"]
mod tests;
