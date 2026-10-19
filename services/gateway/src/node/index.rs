//! The project feed this node serves (`/index/projects`, ADR 0031): one card per listed project, built
//! by the archivist from the directory listing plus the project clone's live counts.
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
}

#[derive(Clone, Debug, Default)]
pub struct ProjectIndex(Arc<RwLock<Vec<ProjectCard>>>);

impl ProjectIndex {
    pub async fn replace(&self, cards: Vec<ProjectCard>) {
        *self.0.write().await = cards;
    }

    /// One page of the feed, best first.
    pub async fn page(
        &self,
        q: &FeedQuery<'_>,
        offset: usize,
        limit: usize,
    ) -> (Vec<ProjectCard>, usize) {
        let cards = self.0.read().await;
        let stats: Vec<ItemStats> = cards.iter().map(stats_of).collect();
        let pairs: Vec<(&ProjectCard, &ItemStats)> = cards.iter().zip(&stats).collect();
        let ranked = query(
            &pairs,
            q,
            |_| ItemKind::Signalled,
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
}

fn stats_of(card: &ProjectCard) -> ItemStats {
    ItemStats {
        support: card.demand,
        oppose: card.opposition,
        members: card.members,
        created_at: card.created_at,
        last_activity_at: card.last_activity_at,
        ..ItemStats::default()
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
        let (page, total) = index.page(&q, 0, 2).await;
        assert_eq!(total, 3);
        assert_eq!(
            page.iter().map(|c| c.title.as_str()).collect::<Vec<_>>(),
            ["Bikes", "Garden"]
        );
        let q = FeedQuery {
            search: "FOOD garden",
            ..q
        };
        assert_eq!(index.page(&q, 0, 10).await.1, 1);
    }
}
