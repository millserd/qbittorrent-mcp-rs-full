use anyhow::Result;
use qbittorrent_mcp_rs::client::QBitClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_rss_operations() -> Result<()> {
    let mock_server = MockServer::start().await;
    let client = QBitClient::new_no_auth(mock_server.uri(), false);

    // Add Feed
    Mock::given(method("POST"))
        .and(path("/api/v2/rss/addFeed"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    // Get Feeds
    Mock::given(method("GET"))
        .and(path("/api/v2/rss/allFeeds"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "test_feed": { "url": "http://example.com/rss", "title": "Test Feed" }
        })))
        .mount(&mock_server)
        .await;

    // Set Rule
    Mock::given(method("POST"))
        .and(path("/api/v2/rss/setRule"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    // Get Rules
    Mock::given(method("GET"))
        .and(path("/api/v2/rss/allRules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "test_rule": {
                "enabled": true,
                "mustContain": "linux",
                "mustNotContain": "windows",
                "useRegex": false,
                "episodeFilter": "",
                "smartFilter": true,
                "assignedCategory": "ISO",
                "savePath": "/downloads",
                "affectedFeeds": ["http://example.com/rss"],
                "ignoreDays": 3
            }
        })))
        .mount(&mock_server)
        .await;

    client
        .add_rss_feed("http://example.com/rss", "test_feed")
        .await?;
    let feeds = client.get_all_rss_feeds().await?;
    assert!(feeds.contains_key("test_feed"));

    client.set_rss_rule("test_rule", "{}").await?;
    let rules = client.get_all_rss_rules().await?;
    assert!(rules.contains_key("test_rule"));
    assert!(rules["test_rule"].smart_episode_filter);
    assert_eq!(rules["test_rule"].must_contain, "linux");
    assert_eq!(rules["test_rule"].assign_category, "ISO");
    assert_eq!(rules["test_rule"].affected_feeds, vec!["http://example.com/rss"]);
    // round trip keeps qBittorrent's key names and unmodelled keys
    let out = serde_json::to_value(&rules["test_rule"])?;
    assert_eq!(out["smartFilter"], true);
    assert_eq!(out["assignedCategory"], "ISO");
    assert_eq!(out["affectedFeeds"][0], "http://example.com/rss");
    assert_eq!(out["ignoreDays"], 3);

    Ok(())
}
