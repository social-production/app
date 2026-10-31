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

    /// One page of the feed, best first; non-empty `creators` keeps only their projects (a
    /// profile's feed, or the people someone follows for web's Home feed), and non-empty
    /// `tags` only the items tagged with any of those channels or communities (`#food`, ADR
    /// 0068; several for web's Home scope). Channels and communities themselves aren't feed
    /// items.
    pub async fn page(
        &self,
        q: &FeedQuery<'_>,
        creators: &[&str],
        tags: &[&str],
        offset: usize,
        limit: usize,
    ) -> (Vec<ProjectCard>, usize) {
        let all = self.0.read().await;
        let cards: Vec<&ProjectCard> = all
            .iter()
            .filter(|c| !sp_scope::is_scope(&c.mode))
            .filter(|c| creators.is_empty() || creators.contains(&c.creator.as_str()))
            .filter(|c| tags.is_empty() || c.tags.iter().any(|have| tags.contains(&have.as_str())))
            .collect();
        let stats: Vec<ItemStats> = cards.iter().map(|c| stats_of(c)).collect();
        let pairs: Vec<(&ProjectCard, &ItemStats)> = cards.into_iter().zip(&stats).collect();
        let ranked = query(
            &pairs,
            q,
            // Help requests rank by votes (their Support/Oppose signals, ADR 0067), as on web.
            |(c, _)| {
                if c.help.is_some() {
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
mod tests {
    use super::*;
    use sp_index::{Sort, Window};

    fn card(title: &str, demand: u32, at: i64) -> ProjectCard {
        ProjectCard {
            id: title.into(),
            network_seed: String::new(),
            creator: String::new(),
            creator_name: String::new(),
            title: title.into(),
            summary: String::new(),
            mode: "productive".into(),
            subtype: "standard".into(),
            location_label: String::new(),
            phase: String::new(),
            tags: vec!["food".into()],
            members: 1,
            demand,
            opposition: 0,
            created_at: at,
            last_activity_at: at,
            latest_update: None,
            comment_count: 0,
            help: None,
        }
    }

    #[tokio::test]
    async fn pages_ranked_cards() {
        let index = ProjectIndex::default();
        index
            .replace(vec![
                card("Garden", 1, 10),
                card("Bikes", 5, 5),
                card("Tools", 0, 20),
            ])
            .await;
        let q = FeedQuery {
            sort: Sort::Top,
            window: Window::All,
            search: "",
            now: 30,
            utc_offset_minutes: 0,
        };
        let (page, total) = index.page(&q, &[], &[], 0, 2).await;
        assert_eq!(total, 3);
        assert_eq!(
            page.iter().map(|c| c.title.as_str()).collect::<Vec<_>>(),
            ["Bikes", "Garden"]
        );
        let q = FeedQuery {
            search: "FOOD garden",
            ..q
        };
        assert_eq!(index.page(&q, &[], &[], 0, 10).await.1, 1);
    }

    #[tokio::test]
    async fn keeps_the_given_creators() {
        let index = ProjectIndex::default();
        let mut mine = card("Garden", 1, 10);
        mine.creator = "ann".into();
        let mut bobs = card("Tools", 2, 20);
        bobs.creator = "bob".into();
        index.replace(vec![mine, card("Bikes", 5, 5), bobs]).await;
        let q = FeedQuery {
            sort: Sort::Top,
            window: Window::All,
            search: "",
            now: 30,
            utc_offset_minutes: 0,
        };
        let (page, total) = index.page(&q, &["ann"], &[], 0, 10).await;
        assert_eq!((page[0].title.as_str(), total), ("Garden", 1));
        let (page, total) = index.page(&q, &["ann", "bob"], &[], 0, 10).await;
        assert_eq!(
            (
                page.iter().map(|c| c.title.as_str()).collect::<Vec<_>>(),
                total
            ),
            (vec!["Tools", "Garden"], 2)
        );
    }

    #[tokio::test]
    async fn scopes_stay_out_of_the_feed_and_the_first_handle_wins() {
        let index = ProjectIndex::default();
        let scope = |id: &str, title: &str, at: i64| ProjectCard {
            id: id.into(),
            mode: "channel".into(),
            ..card(title, 0, at)
        };
        let mut tagged = card("Seed swap", 0, 3);
        tagged.tags = vec!["#food".into()];
        index
            .replace(vec![
                scope("late", "Food", 5),
                scope("first", "food", 1),
                card("Bikes", 0, 2),
                tagged,
            ])
            .await;
        let q = FeedQuery {
            sort: Sort::Recent,
            window: Window::All,
            search: "",
            now: 10,
            utc_offset_minutes: 0,
        };
        assert_eq!(index.page(&q, &[], &[], 0, 10).await.1, 2, "no scopes");
        let (page, _) = index.page(&q, &[], &["#food"], 0, 10).await;
        assert_eq!(
            page.iter().map(|c| c.title.as_str()).collect::<Vec<_>>(),
            ["Seed swap"]
        );
        let (page, _) = index.page(&q, &[], &["@elsewhere", "#food"], 0, 10).await;
        assert_eq!(page.len(), 1, "any of several tags (web's Home scope)");
        let scopes = index.scopes("", 10).await;
        assert_eq!(
            scopes.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
            ["first"]
        );
    }
}
