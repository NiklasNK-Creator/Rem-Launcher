//! Live Modrinth smoke test. Ignored by default (needs network).
//! Run: cargo test -p launcher-core --test live_modrinth -- --ignored --nocapture
use launcher_core::models::SearchQuery;
use launcher_core::providers::modrinth::ModrinthProvider;
use launcher_core::providers::ContentProvider;

#[tokio::test]
#[ignore]
async fn live_search_sodium() {
    let p = ModrinthProvider::new();
    let q = SearchQuery {
        text: "sodium".into(),
        limit: 3,
        ..Default::default()
    };
    let hits = p.search(&q).await.expect("search works");
    assert!(!hits.is_empty(), "expected hits for sodium");
    assert!(hits.iter().any(|h| h.slug.as_deref() == Some("sodium")));
    println!("first hit: {} ({})", hits[0].title, hits[0].id);
    let vers = p.versions(&hits[0].id, &[], &[]).await.expect("versions work");
    assert!(!vers.is_empty());
    assert!(!vers[0].files.is_empty());
}

#[tokio::test]
#[ignore]
async fn live_search_filtered_loader() {
    let p = ModrinthProvider::new();
    let q = SearchQuery {
        text: "sodium".into(),
        game_versions: vec!["1.20.1".into()],
        loaders: vec!["fabric".into()],
        limit: 5,
        ..Default::default()
    };
    let vers = p
        .versions("AANobbMI", &q.game_versions, &q.loaders)
        .await
        .expect("filtered versions work");
    assert!(!vers.is_empty(), "sodium should have 1.20.1 fabric versions");
    assert!(vers.iter().all(|v| v.game_versions.contains(&"1.20.1".to_string())));
}
