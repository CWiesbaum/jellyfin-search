//! Contract test for parsing Jellyfin `/Items` responses, per research.md §2.

use jellyfin_catalog_export::jellyfin::models::ItemsResponse;

const FIXTURE: &str = r#"{
  "Items": [
    {
      "Id": "1",
      "Name": "Example Movie",
      "ProductionYear": 2019,
      "Genres": ["Drama", "Thriller"],
      "ImageTags": { "Primary": "abc123" }
    },
    {
      "Id": "2",
      "Name": "No Metadata Movie"
    }
  ],
  "TotalRecordCount": 2
}"#;

#[test]
fn parses_items_response_fixture() {
    let parsed: ItemsResponse = serde_json::from_str(FIXTURE).unwrap();
    assert_eq!(parsed.total_record_count, 2);
    assert_eq!(parsed.items.len(), 2);

    let first = &parsed.items[0];
    assert_eq!(first.id, "1");
    assert_eq!(first.name.as_deref(), Some("Example Movie"));
    assert_eq!(first.production_year, Some(2019));
    assert_eq!(
        first.genres.as_deref(),
        Some(&["Drama".to_string(), "Thriller".to_string()][..])
    );
    assert_eq!(
        first.image_tags.as_ref().unwrap().primary.as_deref(),
        Some("abc123")
    );

    let second = &parsed.items[1];
    assert_eq!(second.name.as_deref(), Some("No Metadata Movie"));
    assert_eq!(second.production_year, None);
    assert!(second.genres.is_none());
    assert!(second.image_tags.is_none());
}
