//! Static template asset embedding and data substitution, per
//! `specs/001-remote-movie-catalog/research.md` §6.

use crate::catalog::model::CatalogSnapshot;

/// Marker in `assets/index.html.tmpl` replaced with the serialized catalog snapshot.
pub const CATALOG_DATA_MARKER: &str = "__CATALOG_DATA_JSON__";

const TEMPLATE: &str = include_str!("assets/index.html.tmpl");

/// Renders the static HTML template with `snapshot` embedded as inline JSON data.
/// Escapes `</` sequences in the serialized JSON so a movie title containing
/// `</script>` cannot break out of the embedding `<script>` tag.
pub fn render(snapshot: &CatalogSnapshot) -> Result<String, serde_json::Error> {
    let json = serde_json::to_string(snapshot)?.replace("</", "<\\/");
    Ok(TEMPLATE.replace(CATALOG_DATA_MARKER, &json))
}
