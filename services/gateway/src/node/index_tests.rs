//! Tests for the feed index (`index.rs`).

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
async fn posts_stay_on_personal_feeds() {
    let index = ProjectIndex::default();
    let mut post = card("Hello", 0, 2);
    (post.mode, post.creator) = ("post".into(), "ann".into());
    index.replace(vec![post, card("Garden", 0, 1)]).await;
    let q = FeedQuery {
        sort: Sort::Recent,
        window: Window::All,
        search: "",
        now: 30,
        utc_offset_minutes: 0,
    };
    let titles = |page: Vec<ProjectCard>| page.into_iter().map(|c| c.title).collect::<Vec<_>>();
    assert_eq!(titles(index.page(&q, &[], &[], 0, 10).await.0), ["Garden"]);
    assert_eq!(
        titles(index.page(&q, &["ann"], &[], 0, 10).await.0),
        ["Hello"]
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
