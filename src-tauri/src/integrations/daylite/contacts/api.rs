use super::ical_urls::merge_contact_ical_urls;
use super::mapping::{
    filter_planning_contacts, is_planning_contact, map_cached_contact, map_daylite_contact_summary,
    map_planning_contact_to_cache_entry, sort_contacts,
};
use super::types::{
    DayliteContactSummary, DayliteUpdateContactIcalUrlsInput, PlanningContactRecord,
};
use crate::integrations::daylite::auth_flow::{
    send_authenticated_json, send_authenticated_request,
};
use crate::integrations::daylite::client::{DayliteApiClient, DayliteHttpMethod};
use crate::integrations::daylite::shared::{
    with_token_refresh_lock, DayliteApiError, DayliteApiErrorCode, DayliteSearchResult,
    DayliteTokenState,
};
use crate::integrations::local_store::LocalStore;
use chrono::{SecondsFormat, Utc};
use serde_json::json;

pub async fn sync_contact_ical_urls(
    store: &mut LocalStore,
    input: DayliteUpdateContactIcalUrlsInput,
) -> Result<(), DayliteApiError> {
    let daylite_base_url = store.api_endpoints.daylite_base_url.clone();
    let client = DayliteApiClient::new(&daylite_base_url)?;

    with_token_refresh_lock(|tokens| update_contact_ical_urls_core(&client, tokens, store, &input))
        .await?;

    store.daylite_cache.last_synced_at = Some(current_timestamp_iso8601());
    // Caller is responsible for saving the store.
    Ok(())
}

pub(in crate::integrations::daylite) async fn update_contact_ical_urls_core(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
    store: &mut LocalStore,
    input: &DayliteUpdateContactIcalUrlsInput,
) -> Result<(PlanningContactRecord, DayliteTokenState), DayliteApiError> {
    let contact_id = parse_contact_id(&input.contact_reference)?;
    let contact_path = format!("/contacts/{contact_id}");
    let (current_contact, token_state) = send_authenticated_json::<DayliteContactSummary>(
        client,
        token_state,
        DayliteHttpMethod::Get,
        &contact_path,
        Vec::new(),
        None,
    )
    .await?;
    let merged_urls = merge_contact_ical_urls(
        current_contact.urls.clone(),
        &input.primary_ical_url,
        &input.absence_ical_url,
    );
    // PATCH the contact URLs. Daylite may return 204 No Content (empty body),
    // so we only verify the status and construct the result from the GET data + merged URLs.
    let token_state = send_authenticated_request(
        client,
        token_state,
        DayliteHttpMethod::Patch,
        &contact_path,
        Vec::new(),
        Some(json!({
            "urls": merged_urls,
        })),
    )
    .await?;
    let updated_contact = map_daylite_contact_summary(DayliteContactSummary {
        urls: merged_urls,
        ..current_contact
    });

    let mut cached_contacts: Vec<PlanningContactRecord> = store
        .daylite_cache
        .contacts
        .clone()
        .into_iter()
        .map(map_cached_contact)
        .collect();

    cached_contacts.retain(|contact| contact.reference != updated_contact.reference);
    if is_planning_contact(&updated_contact) {
        cached_contacts.push(updated_contact.clone());
    }

    store.daylite_cache.contacts = sort_contacts(filter_planning_contacts(cached_contacts))
        .into_iter()
        .map(map_planning_contact_to_cache_entry)
        .collect();

    Ok((updated_contact, token_state))
}

pub(in crate::integrations::daylite) async fn list_contacts_core(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
) -> Result<(Vec<PlanningContactRecord>, DayliteTokenState), DayliteApiError> {
    let (search_result, token_state) =
        send_authenticated_json::<DayliteSearchResult<DayliteContactSummary>>(
            client,
            token_state,
            DayliteHttpMethod::Post,
            "/contacts/_search",
            vec![("full-records".to_string(), "true".to_string())],
            // A top-level array of clauses is matched with OR semantics, so this
            // fetches both planning categories: "Monteur" and "Test". The "Test"
            // employees are filtered out in the view unless the user disables the
            // "hide non-plannable employees" toggle.
            Some(json!([
                { "category": { "equal": "Monteur" } },
                { "category": { "equal": "Test" } },
            ])),
        )
        .await?;
    let contacts = sort_contacts(filter_planning_contacts(
        search_result
            .results
            .into_iter()
            .map(map_daylite_contact_summary)
            .collect(),
    ));

    Ok((contacts, token_state))
}

pub(super) fn current_timestamp_iso8601() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn parse_contact_id(contact_reference: &str) -> Result<u64, DayliteApiError> {
    let trimmed_reference = contact_reference.trim();
    let contact_id_raw = trimmed_reference.rsplit('/').next().unwrap_or_default();

    contact_id_raw.parse::<u64>().map_err(|error| {
        DayliteApiError::new(
            DayliteApiErrorCode::InvalidResponse,
            None,
            "Die Daylite-Kontaktreferenz ist ungültig.",
            format!("Ungültige Kontaktreferenz `{trimmed_reference}`: {error}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::super::mapping::contact_display_name;
    use super::*;
    use crate::integrations::daylite::test_support::{mock_response, token_state, MockTransport};
    use crate::integrations::local_store::DayliteContactCacheEntry;

    #[test]
    fn list_contacts_searches_both_monteur_and_test_categories() {
        tauri::async_runtime::block_on(async {
            let search_response = mock_response(
                200,
                r#"{"results":[{"self":"/v1/contacts/900","first_name":"Max","last_name":"M","category":"Monteur","urls":[]},{"self":"/v1/contacts/901","first_name":"Bea","last_name":"T","category":"Test","urls":[]}]}"#,
            );
            let transport = MockTransport::new(vec![Ok(search_response)]);
            let client = DayliteApiClient::with_transport(Box::new(transport.clone()));

            let (contacts, _) = list_contacts_core(&client, token_state("token", "refresh"))
                .await
                .expect("list should succeed");

            assert_eq!(contacts.len(), 2);

            let requests = transport.requests();
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].path, "/contacts/_search");
            let body = requests[0].body.as_ref().expect("search should have body");
            let clauses = body.as_array().expect("body should be an OR clause array");
            assert_eq!(clauses.len(), 2);
            assert_eq!(clauses[0]["category"]["equal"], "Monteur");
            assert_eq!(clauses[1]["category"]["equal"], "Test");
        });
    }

    #[test]
    fn parse_contact_id_rejects_invalid_reference() {
        let error = parse_contact_id("/v1/contacts/not-a-number")
            .expect_err("invalid contact reference should fail");

        assert_eq!(
            error.user_message,
            "Die Daylite-Kontaktreferenz ist ungültig."
        );
    }

    #[test]
    fn update_ical_urls_fetches_merges_and_patches() {
        tauri::async_runtime::block_on(async {
            let get_response = mock_response(
                200,
                r#"{"self":"/v1/contacts/500","first_name":"Max","last_name":"M","urls":[{"label":"Website","url":"https://example.com"}]}"#,
            );
            let patch_response = mock_response(
                200,
                r#"{"self":"/v1/contacts/500","first_name":"Max","last_name":"M","category":"Monteur","urls":[{"label":"Website","url":"https://example.com"},{"label":"Einsatz iCal","url":"https://example.com/primary.ics"},{"label":"Abwesenheit iCal","url":"https://example.com/absence.ics"}]}"#,
            );
            let transport = MockTransport::new(vec![Ok(get_response), Ok(patch_response)]);
            let client = DayliteApiClient::with_transport(Box::new(transport.clone()));
            let mut store = LocalStore::default();

            let (contact, token_state) = update_contact_ical_urls_core(
                &client,
                token_state("token", "refresh"),
                &mut store,
                &DayliteUpdateContactIcalUrlsInput {
                    contact_reference: "/v1/contacts/500".to_string(),
                    primary_ical_url: "https://example.com/primary.ics".to_string(),
                    absence_ical_url: "https://example.com/absence.ics".to_string(),
                },
            )
            .await
            .expect("update should succeed");

            assert_eq!(contact.reference, "/v1/contacts/500");
            assert_eq!(token_state.access_token, "token");

            let requests = transport.requests();
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[0].method, DayliteHttpMethod::Get);
            assert_eq!(requests[0].path, "/contacts/500");
            assert_eq!(requests[1].method, DayliteHttpMethod::Patch);
            assert_eq!(requests[1].path, "/contacts/500");

            let patch_body = requests[1].body.as_ref().expect("PATCH should have body");
            let urls = patch_body["urls"].as_array().expect("urls should be array");
            assert_eq!(urls.len(), 3);
        });
    }

    #[test]
    fn update_ical_urls_handles_204_no_content_from_patch() {
        tauri::async_runtime::block_on(async {
            let get_response = mock_response(
                200,
                r#"{"self":"/v1/contacts/800","first_name":"Karl","last_name":"G","category":"Monteur","urls":[]}"#,
            );
            let patch_response = mock_response(204, "");
            let transport = MockTransport::new(vec![Ok(get_response), Ok(patch_response)]);
            let client = DayliteApiClient::with_transport(Box::new(transport));
            let mut store = LocalStore::default();

            let (contact, _) = update_contact_ical_urls_core(
                &client,
                token_state("token", "refresh"),
                &mut store,
                &DayliteUpdateContactIcalUrlsInput {
                    contact_reference: "/v1/contacts/800".to_string(),
                    primary_ical_url: "https://example.com/primary.ics".to_string(),
                    absence_ical_url: "".to_string(),
                },
            )
            .await
            .expect("update should succeed even when PATCH returns 204 No Content");

            assert_eq!(contact.reference, "/v1/contacts/800");
            assert_eq!(contact.category, Some("Monteur".to_string()));
            assert_eq!(contact.urls.len(), 1);
            assert_eq!(
                contact.urls[0].url,
                Some("https://example.com/primary.ics".to_string())
            );
        });
    }

    #[test]
    fn update_ical_urls_updates_cache_for_monteur() {
        tauri::async_runtime::block_on(async {
            let get_response = mock_response(
                200,
                r#"{"self":"/v1/contacts/600","first_name":"Anna","last_name":"B","category":"Monteur","urls":[]}"#,
            );
            let patch_response = mock_response(
                200,
                r#"{"self":"/v1/contacts/600","first_name":"Anna","last_name":"B","category":"Monteur","urls":[{"label":"Einsatz iCal","url":"https://example.com/anna.ics"}]}"#,
            );
            let transport = MockTransport::new(vec![Ok(get_response), Ok(patch_response)]);
            let client = DayliteApiClient::with_transport(Box::new(transport));
            let mut store = LocalStore::default();
            store.daylite_cache.contacts = vec![DayliteContactCacheEntry {
                reference: "/v1/contacts/600".to_string(),
                full_name: Some("Anna B".to_string()),
                nickname: None,
                category: Some("Monteur".to_string()),
                urls: vec![],
            }];

            let (contact, _) = update_contact_ical_urls_core(
                &client,
                token_state("token", "refresh"),
                &mut store,
                &DayliteUpdateContactIcalUrlsInput {
                    contact_reference: "/v1/contacts/600".to_string(),
                    primary_ical_url: "https://example.com/anna.ics".to_string(),
                    absence_ical_url: "".to_string(),
                },
            )
            .await
            .expect("update should succeed");

            assert_eq!(contact.category, Some("Monteur".to_string()));
            assert_eq!(store.daylite_cache.contacts.len(), 1);
            assert_eq!(
                store.daylite_cache.contacts[0].reference,
                "/v1/contacts/600"
            );
            assert_eq!(store.daylite_cache.contacts[0].urls.len(), 1);
        });
    }

    #[test]
    fn update_ical_urls_removes_non_monteur_from_cache() {
        tauri::async_runtime::block_on(async {
            let get_response = mock_response(
                200,
                r#"{"self":"/v1/contacts/700","first_name":"Kai","last_name":"V","category":"Vertrieb","urls":[]}"#,
            );
            let patch_response = mock_response(
                200,
                r#"{"self":"/v1/contacts/700","first_name":"Kai","last_name":"V","category":"Vertrieb","urls":[]}"#,
            );
            let transport = MockTransport::new(vec![Ok(get_response), Ok(patch_response)]);
            let client = DayliteApiClient::with_transport(Box::new(transport));
            let mut store = LocalStore::default();
            store.daylite_cache.contacts = vec![DayliteContactCacheEntry {
                reference: "/v1/contacts/700".to_string(),
                full_name: Some("Kai V".to_string()),
                nickname: None,
                category: Some("Monteur".to_string()),
                urls: vec![],
            }];

            let (contact, _) = update_contact_ical_urls_core(
                &client,
                token_state("token", "refresh"),
                &mut store,
                &DayliteUpdateContactIcalUrlsInput {
                    contact_reference: "/v1/contacts/700".to_string(),
                    primary_ical_url: "".to_string(),
                    absence_ical_url: "".to_string(),
                },
            )
            .await
            .expect("update should succeed");

            assert_eq!(contact.category, Some("Vertrieb".to_string()));
            assert!(store.daylite_cache.contacts.is_empty());
        });
    }

    #[test]
    fn list_contacts_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let client = DayliteApiClient::with_replay_cassette("daylite-list-contacts.json")
                .expect("replay client should be created");

            let (contacts, token_state) = list_contacts_core(
                &client,
                token_state("replay-access-token", "replay-refresh-token"),
            )
            .await
            .expect("list should replay from cassette");

            assert!(!contacts.is_empty());
            assert!(contacts
                .iter()
                .all(|contact| contact.reference.starts_with("/v1/contacts/")));
            assert!(contacts.iter().all(|contact| {
                matches!(contact.category.as_deref(), Some("Monteur") | Some("Test"))
            }));
            assert!(contacts.iter().all(|contact| {
                contact
                    .full_name
                    .as_deref()
                    .map(|name| name == name.trim())
                    .unwrap_or(true)
                    && contact
                        .nickname
                        .as_deref()
                        .map(|nickname| nickname == nickname.trim())
                        .unwrap_or(true)
            }));
            assert!(contacts.windows(2).all(|pair| {
                contact_display_name(&pair[0]).to_lowercase()
                    <= contact_display_name(&pair[1]).to_lowercase()
            }));
            assert_eq!(token_state.access_token, "replay-access-token");
        });
    }

    #[test]
    fn update_ical_urls_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let client =
                DayliteApiClient::with_replay_cassette("daylite-update-contact-ical-urls.json")
                    .expect("replay client should be created");
            let mut store = LocalStore::default();

            let (contact, token_state) = update_contact_ical_urls_core(
                &client,
                token_state("replay-access-token", "replay-refresh-token"),
                &mut store,
                &DayliteUpdateContactIcalUrlsInput {
                    contact_reference: "/v1/contacts/1029".to_string(),
                    primary_ical_url: "https://example.com/primary.ics".to_string(),
                    absence_ical_url: "https://example.com/absence.ics".to_string(),
                },
            )
            .await
            .expect("update should replay from cassette");

            assert_eq!(contact.reference, "/v1/contacts/1029");
            assert_eq!(contact.category, Some("Monteur".to_string()));
            assert_eq!(contact.urls.len(), 2);
            assert_eq!(token_state.access_token, "replay-access-token");
            assert_eq!(store.daylite_cache.contacts.len(), 1);
            assert_eq!(
                store.daylite_cache.contacts[0].reference,
                "/v1/contacts/1029"
            );
        });
    }
}
