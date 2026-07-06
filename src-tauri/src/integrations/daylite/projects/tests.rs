use super::{
    list_projects_core, map_daylite_project_summary, map_project_status,
    query_overdue_projects_core, search_projects_core, DayliteProjectSummaryDto,
    PlanningProjectStatus,
};
use crate::integrations::daylite::client::DayliteApiClient;
use crate::integrations::daylite::client::DayliteHttpMethod;
use crate::integrations::daylite::shared::{
    DayliteApiError, DayliteApiErrorCode, DayliteSearchInput, DayliteSearchSort, DayliteTokenState,
};
use crate::integrations::daylite::test_support::{
    mock_response, token_state, valid_token_state, MockTransport,
};

#[test]
fn maps_project_summary_to_planning_project_record() {
    let project = DayliteProjectSummaryDto {
        reference: " /v1/projects/7000 ".to_string(),
        name: " Projekt Nord ".to_string(),
        status: Some(" NEW ".to_string()),
        category: Some(" Überfällig ".to_string()),
        keywords: vec![
            " Aufträge ".to_string(),
            "".to_string(),
            "Vorbereitung".to_string(),
        ],
        due: Some("2026-02-15".to_string()),
        started: None,
        completed: None,
        create_date: Some("not-a-date".to_string()),
        modify_date: Some("2026-02-15T12:45:00+01:00".to_string()),
    };

    let mapped = map_daylite_project_summary(project);

    assert_eq!(mapped.reference, "/v1/projects/7000");
    assert_eq!(mapped.name, "Projekt Nord");
    assert_eq!(mapped.status, PlanningProjectStatus::NewStatus);
    assert_eq!(mapped.category, Some("Überfällig".to_string()));
    assert_eq!(
        mapped.keywords,
        vec!["Aufträge".to_string(), "Vorbereitung".to_string()]
    );
    assert_eq!(mapped.due, Some("2026-02-15T00:00:00.000Z".to_string()));
    assert_eq!(mapped.create_date, None);
    assert_eq!(
        mapped.modify_date,
        Some("2026-02-15T11:45:00.000Z".to_string())
    );
}

#[test]
fn defaults_unknown_project_status_to_new_status() {
    let mapped_status = map_project_status(Some("unknown-status".to_string()));
    assert_eq!(mapped_status, PlanningProjectStatus::NewStatus);
}

#[test]
fn list_projects_sends_search_request_and_maps_results() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[{"self":"/v1/projects/1","name":"Projekt A","status":"in_progress"},{"self":"/v1/projects/2","name":"Projekt B"}],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport.clone()));

        let (projects, token_state) = list_projects_core(&client, valid_token_state())
            .await
            .expect("list should succeed");

        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].name, "Projekt A");
        assert_eq!(projects[0].status, PlanningProjectStatus::InProgress);
        assert_eq!(projects[1].name, "Projekt B");
        assert_eq!(projects[1].status, PlanningProjectStatus::NewStatus);
        assert_eq!(token_state.access_token, "at");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].path, "/projects/_search");
        assert_eq!(requests[0].method, DayliteHttpMethod::Post);
        assert_eq!(
            requests[0].query,
            vec![("full-records".to_string(), "true".to_string())]
        );
        assert!(requests[0].body.is_some());
    });
}

#[test]
fn search_projects_sends_correct_body_and_query() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[{"self":" /v1/projects/10 ","name":" Projekt Nord ","category":" Bau ","keywords":[" Aufträge ",""],"due":"2026-02-15"}],"next":" /v1/projects/_search?offset=5 "}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport.clone()));

        let (result, _) = search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].reference, "/v1/projects/10");
        assert_eq!(result.results[0].name, "Projekt Nord");
        assert_eq!(result.results[0].category, Some("Bau".to_string()));
        assert_eq!(result.results[0].keywords, vec!["Aufträge".to_string()]);
        assert_eq!(
            result.results[0].due,
            Some("2026-02-15T00:00:00.000Z".to_string())
        );
        assert_eq!(
            result.next,
            Some("/v1/projects/_search?offset=5".to_string())
        );

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].query,
            vec![("limit".to_string(), "5".to_string())]
        );
        let body = requests[0].body.as_ref().expect("should have body");
        assert_eq!(body["name"]["contains"], "Nord");
    });
}

#[test]
fn search_results_are_sorted_by_numeric_id_ascending() {
    tauri::async_runtime::block_on(async {
        // A string sort would order these 100 < 20 < 3, so the fixture can tell the sorts apart.
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[
                {"self":"/v1/projects/100","name":"Hundert"},
                {"self":"/v1/projects/20","name":"Zwanzig"},
                {"self":"/v1/projects/3","name":"Drei"}
            ],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport));

        let (result, _) =
            search_projects_core(&client, valid_token_state(), &DayliteSearchInput::default())
                .await
                .expect("search should succeed");

        assert_eq!(result.results[0].reference, "/v1/projects/3");
        assert_eq!(result.results[1].reference, "/v1/projects/20");
        assert_eq!(result.results[2].reference, "/v1/projects/100");
    });
}

#[test]
fn search_treats_empty_object_response_as_no_results() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(200, r#"{}"#))]);
        let client = DayliteApiClient::with_transport(Box::new(transport));

        let (result, _) = search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                ..Default::default()
            },
        )
        .await
        .expect("empty object response should be treated as no results");

        assert!(result.results.is_empty());
        assert_eq!(result.next, None);
    });
}

#[test]
fn search_sorts_by_name_when_sort_is_name() {
    tauri::async_runtime::block_on(async {
        // IDs ascend but names do not, so an ID sort and a name sort diverge.
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[
                {"self":"/v1/projects/1","name":"Zeta"},
                {"self":"/v1/projects/2","name":"Alpha"},
                {"self":"/v1/projects/3","name":"Mitte"}
            ],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport));

        let (result, _) = search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                sort: Some(DayliteSearchSort::Name),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        assert_eq!(result.results[0].name, "Alpha");
        assert_eq!(result.results[1].name, "Mitte");
        assert_eq!(result.results[2].name, "Zeta");
    });
}

#[test]
fn search_defaults_to_numeric_id_sort_when_sort_is_none() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[
                {"self":"/v1/projects/3","name":"Alpha"},
                {"self":"/v1/projects/1","name":"Zeta"}
            ],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport));

        let (result, _) =
            search_projects_core(&client, valid_token_state(), &DayliteSearchInput::default())
                .await
                .expect("search should succeed");

        assert_eq!(result.results[0].reference, "/v1/projects/1");
        assert_eq!(result.results[1].reference, "/v1/projects/3");
    });
}

#[test]
fn search_limit_is_applied_after_sort() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[
                {"self":"/v1/projects/100","name":"Hundert"},
                {"self":"/v1/projects/20","name":"Zwanzig"},
                {"self":"/v1/projects/3","name":"Drei"}
            ],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport));

        let (result, _) = search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                limit: Some(2),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        assert_eq!(result.results.len(), 2);
        assert_eq!(result.results[0].reference, "/v1/projects/3");
        assert_eq!(result.results[1].reference, "/v1/projects/20");
    });
}

#[test]
fn overdue_query_sends_category_and_status_filter_in_a_single_call() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport.clone()));

        query_overdue_projects_core(&client, valid_token_state())
            .await
            .expect("overdue query should succeed");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1, "overdue query must be a single call");
        assert_eq!(requests[0].path, "/projects/_search");
        assert_eq!(requests[0].method, DayliteHttpMethod::Post);
        let body = requests[0].body.as_ref().expect("body should be present");
        assert_eq!(
            *body,
            serde_json::json!([
                {
                    "category": { "equal": "Überfällig" },
                    "status": { "equal": "new_status" }
                },
                {
                    "category": { "equal": "Überfällig" },
                    "status": { "equal": "in_progress" }
                }
            ]),
            "body must pair the category filter with each allowed status as OR clauses"
        );
    });
}

#[test]
fn overdue_results_are_sorted_by_numeric_id_and_limited_to_five() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[
                {"self":"/v1/projects/100","name":"Hundert"},
                {"self":"/v1/projects/20","name":"Zwanzig"},
                {"self":"/v1/projects/3","name":"Drei"},
                {"self":"/v1/projects/50","name":"Fünfzig"},
                {"self":"/v1/projects/7","name":"Sieben"},
                {"self":"/v1/projects/9","name":"Neun"}
            ],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport));

        let (results, _) = query_overdue_projects_core(&client, valid_token_state())
            .await
            .expect("overdue query should succeed");

        assert_eq!(results.len(), 5);
        let references: Vec<&str> = results
            .iter()
            .map(|project| project.reference.as_str())
            .collect();
        assert_eq!(
            references,
            vec![
                "/v1/projects/3",
                "/v1/projects/7",
                "/v1/projects/9",
                "/v1/projects/20",
                "/v1/projects/50"
            ]
        );
    });
}

#[test]
fn overdue_query_treats_empty_object_response_as_no_results() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(200, r#"{}"#))]);
        let client = DayliteApiClient::with_transport(Box::new(transport));

        let (results, _) = query_overdue_projects_core(&client, valid_token_state())
            .await
            .expect("empty object response should be treated as no results");

        assert!(results.is_empty());
    });
}

#[test]
fn query_overdue_projects_replays_vcr_cassette() {
    // The cassette is produced by the live recording harness
    // (`record_daylite_cassettes_from_live_api`), which needs real Daylite
    // credentials. Skip instead of failing until it has been recorded.
    let cassette_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/cassettes/daylite-overdue-projects.json");
    if !cassette_path.exists() {
        eprintln!(
            "skipping query_overdue_projects_replays_vcr_cassette: cassette not recorded yet"
        );
        return;
    }

    tauri::async_runtime::block_on(async {
        let client = DayliteApiClient::with_replay_cassette("daylite-overdue-projects.json")
            .expect("replay client should be created");

        let (results, token_state) = query_overdue_projects_core(
            &client,
            token_state("replay-access-token", "replay-refresh-token"),
        )
        .await
        .expect("overdue query should replay from cassette");

        assert!(results.len() <= 5);
        assert!(results.iter().all(|project| {
            project.reference.starts_with("/v1/projects/")
                && !project.name.is_empty()
                && project.name == project.name.trim()
        }));
        assert_eq!(token_state.access_token, "replay-access-token");
    });
}

#[test]
fn extract_numeric_id_handles_standard_reference() {
    assert_eq!(super::extract_numeric_id("/v1/projects/3001"), 3001);
    assert_eq!(super::extract_numeric_id("/v1/projects/100"), 100);
    assert_eq!(super::extract_numeric_id("/v1/projects/20"), 20);
}

#[test]
fn extract_numeric_id_returns_max_for_non_numeric() {
    assert_eq!(super::extract_numeric_id("/v1/projects/abc"), u64::MAX);
    assert_eq!(super::extract_numeric_id(""), u64::MAX);
}

#[test]
fn list_projects_returns_updated_token_state_after_refresh() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![
            Ok(mock_response(
                200,
                r#"{"access_token":"new-at","refresh_token":"new-rt","expires_in":3600}"#,
            )),
            Ok(mock_response(200, r#"{"results":[],"next":null}"#)),
        ]);
        let client = DayliteApiClient::with_transport(Box::new(transport));

        let (projects, token_state) = list_projects_core(
            &client,
            DayliteTokenState {
                access_token: String::new(),
                refresh_token: "old-rt".to_string(),
                access_token_expires_at_ms: None,
            },
        )
        .await
        .expect("list after refresh should succeed");

        assert!(projects.is_empty());
        assert_eq!(token_state.access_token, "new-at");
        assert_eq!(token_state.refresh_token, "new-rt");
        assert!(token_state.access_token_expires_at_ms.is_some());
    });
}

#[test]
fn list_projects_replays_vcr_cassette() {
    tauri::async_runtime::block_on(async {
        let client = DayliteApiClient::with_replay_cassette("daylite-list-projects.json")
            .expect("replay client should be created");

        let (projects, token_state) = list_projects_core(
            &client,
            token_state("replay-access-token", "replay-refresh-token"),
        )
        .await
        .expect("list should replay from cassette");

        assert!(!projects.is_empty());
        assert!(projects
            .iter()
            .all(|project| project.reference.starts_with("/v1/projects/")));
        assert!(projects
            .iter()
            .all(|project| !project.name.is_empty() && project.name == project.name.trim()));
        assert_eq!(token_state.access_token, "replay-access-token");
    });
}

#[test]
fn search_projects_replays_vcr_cassette() {
    tauri::async_runtime::block_on(async {
        let client = DayliteApiClient::with_replay_cassette("daylite-search-projects.json")
            .expect("replay client should be created");

        let (search_result, token_state) = search_projects_core(
            &client,
            token_state("replay-access-token", "replay-refresh-token"),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                ..Default::default()
            },
        )
        .await
        .expect("search should replay from cassette");

        assert!(!search_result.results.is_empty());
        assert!(search_result.results.len() <= 5);
        assert!(search_result.results.iter().all(|project| {
            project.reference.starts_with("/v1/projects/")
                && !project.name.is_empty()
                && project.name == project.name.trim()
                && project.name.to_lowercase().contains("nord")
        }));
        assert!(search_result
            .next
            .as_deref()
            .map(|next| next.starts_with("/v1/projects/_search"))
            .unwrap_or(true));
        assert_eq!(token_state.access_token, "replay-access-token");
    });
}

#[test]
fn search_projects_with_status_filter_replays_vcr_cassette() {
    tauri::async_runtime::block_on(async {
        let client = DayliteApiClient::with_replay_cassette("daylite-search-projects.json")
            .expect("status-filter cassette client should be created");

        let (search_result, token_state) = search_projects_core(
            &client,
            token_state("test-token", "test-refresh"),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                full_records: Some(true),
                statuses: Some(vec!["new_status".to_string(), "in_progress".to_string()]),
                ..Default::default()
            },
        )
        .await
        .expect("search with status filter should replay from cassette");

        assert!(
            !search_result.results.is_empty(),
            "cassette should contain results"
        );
        assert_eq!(token_state.access_token, "test-token");

        for project in &search_result.results {
            assert!(
                project.status.as_deref() == Some("new")
                    || project.status.as_deref() == Some("in_progress"),
                "project {:?} has unexpected status",
                project.reference
            );
        }
    });
}

#[test]
fn search_projects_no_match_replays_vcr_cassette() {
    tauri::async_runtime::block_on(async {
        let client = DayliteApiClient::with_replay_cassette("daylite-search-projects.json")
            .expect("no-match cassette client should be created");

        let (search_result, token_state) = search_projects_core(
            &client,
            token_state("test-token", "test-refresh"),
            &DayliteSearchInput {
                search_term: "XXXXX".to_string(),
                limit: Some(50),
                statuses: Some(vec!["new_status".to_string(), "in_progress".to_string()]),
                sort: Some(DayliteSearchSort::Name),
                ..Default::default()
            },
        )
        .await
        .expect("no-match search should replay from cassette");

        assert!(search_result.results.is_empty());
        assert_eq!(token_state.access_token, "test-token");
    });
}

#[test]
fn search_with_statuses_sends_array_body_with_or_clauses() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport.clone()));

        search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                statuses: Some(vec!["new_status".to_string(), "in_progress".to_string()]),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body = requests[0].body.as_ref().expect("body should be present");
        assert!(body.is_array(), "body should be an array for OR conditions");
        let items = body.as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["name"]["contains"], "Nord");
        assert_eq!(items[0]["status"]["equal"], "new_status");
        assert_eq!(items[1]["name"]["contains"], "Nord");
        assert_eq!(items[1]["status"]["equal"], "in_progress");
    });
}

#[test]
fn search_without_statuses_sends_plain_object_body() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport.clone()));

        search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body = requests[0].body.as_ref().expect("body should be present");
        assert!(
            body.is_object(),
            "body should be a plain object when no statuses"
        );
        assert_eq!(body["name"]["contains"], "Nord");
        assert!(
            body.get("status").is_none(),
            "no status key when statuses is None"
        );
    });
}

#[test]
fn search_with_full_records_sends_query_param() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport.clone()));

        search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                full_records: Some(true),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        let requests = transport.requests();
        assert!(
            requests[0]
                .query
                .contains(&("full-records".to_string(), "true".to_string())),
            "query should include full-records=true, got {:?}",
            requests[0].query
        );
    });
}

#[test]
fn search_without_full_records_omits_query_param() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport.clone()));

        search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        let requests = transport.requests();
        assert!(
            !requests[0].query.iter().any(|(k, _)| k == "full-records"),
            "query should not include full-records when None, got {:?}",
            requests[0].query
        );
    });
}

#[test]
fn search_with_start_sends_query_param() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);
        let client = DayliteApiClient::with_transport(Box::new(transport.clone()));

        search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                start: Some("3001".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        let requests = transport.requests();
        assert!(
            requests[0]
                .query
                .contains(&("start".to_string(), "3001".to_string())),
            "query should include start=3001, got {:?}",
            requests[0].query
        );
    });
}

#[test]
fn malformed_response_returns_invalid_response_with_german_message() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Ok(mock_response(200, "not valid json {{{"))]);
        let client = DayliteApiClient::with_transport(Box::new(transport));

        let result = search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                ..Default::default()
            },
        )
        .await;

        let err = result.expect_err("malformed response should return error");
        assert_eq!(err.code, DayliteApiErrorCode::InvalidResponse);
        assert!(
            err.user_message.contains("Daylite"),
            "error message should mention Daylite: {}",
            err.user_message
        );
    });
}

#[test]
fn timeout_error_propagates_from_transport() {
    tauri::async_runtime::block_on(async {
        let transport = MockTransport::new(vec![Err(DayliteApiError {
            code: DayliteApiErrorCode::Timeout,
            http_status: None,
            user_message: "Zeitüberschreitung bei der Daylite-Anfrage.".to_string(),
            technical_message: "request timed out".to_string(),
        })]);
        let client = DayliteApiClient::with_transport(Box::new(transport));

        let result = search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                ..Default::default()
            },
        )
        .await;

        let err = result.expect_err("timeout from transport should propagate as error");
        assert_eq!(err.code, DayliteApiErrorCode::Timeout);
        assert_eq!(
            err.user_message,
            "Zeitüberschreitung bei der Daylite-Anfrage."
        );
    });
}
