use std::collections::HashMap;

use serde::Deserialize;

use super::auth_flow::send_authenticated_json;
use super::client::{DayliteApiClient, DayliteHttpMethod, DayliteHttpRequest};
use super::shared::{
    run_daylite_command, with_token_refresh_lock, DayliteApiError, DayliteTokenState,
};

#[derive(Debug, Clone, Deserialize)]
struct DayliteCategoryDto {
    #[serde(default)]
    name: String,
    #[serde(default)]
    hex_colour: Option<String>,
}

/// `/categories` is a plain collection endpoint rather than a `_search`, and the
/// Daylite reference does not pin down whether it wraps the collection in
/// `results`, so both shapes are accepted.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DayliteCategoryListDto {
    Wrapped {
        #[serde(default)]
        results: Vec<DayliteCategoryDto>,
    },
    Bare(Vec<DayliteCategoryDto>),
}

impl DayliteCategoryListDto {
    fn into_categories(self) -> Vec<DayliteCategoryDto> {
        match self {
            Self::Wrapped { results } => results,
            Self::Bare(results) => results,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_project_category_colors(
    app: tauri::AppHandle,
) -> Result<HashMap<String, String>, DayliteApiError> {
    run_daylite_command(app, |client, tokens| async move {
        fetch_project_category_colors_core(&client, tokens).await
    })
    .await
}

pub(crate) async fn fetch_project_category_colors(
    app: tauri::AppHandle,
) -> HashMap<String, String> {
    let Ok(store) = crate::integrations::local_store::load_local_store(app) else {
        return HashMap::new();
    };
    let Ok(client) = DayliteApiClient::new(&store.api_endpoints.daylite_base_url) else {
        return HashMap::new();
    };

    with_token_refresh_lock(|tokens| fetch_project_category_colors_core(&client, tokens))
        .await
        .unwrap_or_default()
}

pub(super) async fn fetch_project_category_colors_core(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
) -> Result<(HashMap<String, String>, DayliteTokenState), DayliteApiError> {
    let (list, token_state) = send_authenticated_json::<DayliteCategoryListDto>(
        client,
        token_state,
        DayliteHttpRequest {
            query: vec![("entity".to_string(), "project".to_string())],
            ..DayliteHttpRequest::new(DayliteHttpMethod::Get, "/categories")
        },
    )
    .await?;

    let colors = list
        .into_categories()
        .into_iter()
        .filter_map(|category| {
            let name = category.name.trim().to_string();
            let color = normalize_hex_colour(category.hex_colour)?;
            if name.is_empty() {
                return None;
            }
            Some((name, color))
        })
        .collect();

    Ok((colors, token_state))
}

fn normalize_hex_colour(value: Option<String>) -> Option<String> {
    let trimmed = value?.trim().to_string();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('#') {
        return Some(trimmed);
    }

    Some(format!("#{trimmed}"))
}

#[cfg(test)]
mod tests {
    use super::fetch_project_category_colors_core;
    use crate::integrations::daylite::client::DayliteHttpMethod;
    use crate::integrations::daylite::test_support::{
        mock_client, mock_response, valid_token_state,
    };

    #[tokio::test]
    async fn requests_categories_filtered_to_projects() {
        let (client, transport) = mock_client(vec![Ok(mock_response(200, r##"{"results":[]}"##))]);

        fetch_project_category_colors_core(&client, valid_token_state())
            .await
            .expect("category fetch should succeed");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].path, "/categories");
        assert_eq!(requests[0].method, DayliteHttpMethod::Get);
        assert_eq!(
            requests[0].query,
            vec![("entity".to_string(), "project".to_string())]
        );
    }

    #[tokio::test]
    async fn maps_category_names_to_their_colors() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r##"{"results":[
                {"name":" Bau ","hex_colour":" #8bc34a ","is_active":true},
                {"name":"Wartung","hex_colour":"#03a9f4","is_active":true}
            ]}"##,
        ))]);

        let (colors, _) = fetch_project_category_colors_core(&client, valid_token_state())
            .await
            .expect("category fetch should succeed");

        assert_eq!(colors.get("Bau"), Some(&"#8bc34a".to_string()));
        assert_eq!(colors.get("Wartung"), Some(&"#03a9f4".to_string()));
    }

    #[tokio::test]
    async fn omits_categories_without_a_color() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r##"{"results":[
                {"name":"Ohne Farbe","hex_colour":null,"is_active":true},
                {"name":"Leer","hex_colour":"  ","is_active":true}
            ]}"##,
        ))]);

        let (colors, _) = fetch_project_category_colors_core(&client, valid_token_state())
            .await
            .expect("category fetch should succeed");

        assert!(colors.is_empty());
    }

    #[tokio::test]
    async fn keeps_inactive_categories() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r##"{"results":[{"name":"Stillgelegt","hex_colour":"#ff9800","is_active":false}]}"##,
        ))]);

        let (colors, _) = fetch_project_category_colors_core(&client, valid_token_state())
            .await
            .expect("category fetch should succeed");

        assert_eq!(colors.get("Stillgelegt"), Some(&"#ff9800".to_string()));
    }

    #[tokio::test]
    async fn accepts_a_bare_array_response() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r##"[{"name":"Bau","hex_colour":"#8bc34a"}]"##,
        ))]);

        let (colors, _) = fetch_project_category_colors_core(&client, valid_token_state())
            .await
            .expect("category fetch should succeed");

        assert_eq!(colors.get("Bau"), Some(&"#8bc34a".to_string()));
    }

    #[tokio::test]
    async fn prefixes_a_missing_hash_on_hex_colours() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r##"{"results":[{"name":"Bau","hex_colour":"8BC34A"}]}"##,
        ))]);

        let (colors, _) = fetch_project_category_colors_core(&client, valid_token_state())
            .await
            .expect("category fetch should succeed");

        assert_eq!(colors.get("Bau"), Some(&"#8BC34A".to_string()));
    }
}
