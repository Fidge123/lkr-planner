use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::{observe, record_failure, RequestFailure};

#[tauri::command]
#[specta::specta]
pub async fn get_holidays_for_week(
    app: tauri::AppHandle,
    week_start: String,
) -> Result<Vec<Holiday>, String> {
    let handle = app.clone();
    observe(
        &handle,
        Operation::GetHolidaysForWeek,
        Integration::Holidays,
        get_holidays_for_week_inner(app, week_start),
    )
    .await
}

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::time::Duration;
use tauri_plugin_http::reqwest;

const NAGER_BASE_URL: &str = "https://date.nager.at/api/v3/PublicHolidays";
const REQUEST_TIMEOUT_SECS: u64 = 5;
const CACHE_REFRESH_DAYS: i64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Holiday {
    pub date: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NagerHoliday {
    date: String,
    local_name: String,
    global: bool,
    counties: Option<Vec<String>>,
}

async fn get_holidays_for_week_inner(
    app: tauri::AppHandle,
    week_start: String,
) -> Result<Vec<Holiday>, String> {
    let week_start_date = NaiveDate::parse_from_str(&week_start, "%Y-%m-%d").map_err(|e| {
        eprintln!("holidays: invalid week_start '{week_start}': {e}");
        "Feiertage konnten nicht geladen werden".to_string()
    })?;
    let week_end_date = week_start_date + chrono::Duration::days(6);

    let years = years_for_week(week_start_date, week_end_date);

    let mut store =
        crate::integrations::local_store::load_local_store(app.clone()).map_err(|e| {
            eprintln!("holidays: store load failed: {}", e.technical_message);
            "Feiertage konnten nicht geladen werden".to_string()
        })?;

    let today = chrono::Utc::now().date_naive();
    let current_year = today.year();

    let mut fetched_any = false;
    for &year in &years {
        let needs_fetch = match store.holiday_cache.iter().find(|e| e.year == year) {
            None => true,
            Some(entry) => year == current_year && !is_cache_entry_fresh(entry, today),
        };

        if needs_fetch {
            fetched_any = true;
            let fetched = fetch_holidays_from_api(&app, year).await?;
            let cached = fetched
                .iter()
                .map(|h| crate::integrations::local_store::CachedHoliday {
                    date: h.date.clone(),
                    name: h.name.clone(),
                })
                .collect();
            let today_str = today.format("%Y-%m-%d").to_string();
            store.holiday_cache.retain(|e| e.year != year);
            store
                .holiday_cache
                .push(crate::integrations::local_store::HolidayCacheEntry {
                    year,
                    fetched_at: today_str,
                    holidays: cached,
                });
        }
    }

    let holidays = holidays_in_range(&store.holiday_cache, week_start_date, week_end_date);

    if fetched_any {
        crate::integrations::local_store::save_store_internal(&app, store).map_err(|e| {
            eprintln!("holidays: store save failed: {}", e.technical_message);
            "Feiertage konnten nicht geladen werden".to_string()
        })?;
    }

    Ok(holidays)
}

/// Both bounds are inclusive, so a holiday on the last displayed day still counts.
fn holidays_in_range(
    cache: &[crate::integrations::local_store::HolidayCacheEntry],
    start: NaiveDate,
    end: NaiveDate,
) -> Vec<Holiday> {
    cache
        .iter()
        .flat_map(|entry| entry.holidays.iter())
        .filter_map(|holiday| {
            let date = NaiveDate::parse_from_str(&holiday.date, "%Y-%m-%d").ok()?;
            if date < start || date > end {
                return None;
            }
            Some(Holiday {
                date: holiday.date.clone(),
                name: holiday.name.clone(),
            })
        })
        .collect()
}

fn years_for_week(start: NaiveDate, end: NaiveDate) -> Vec<i32> {
    if start.year() == end.year() {
        vec![start.year()]
    } else {
        vec![start.year(), end.year()]
    }
}

fn is_cache_entry_fresh(
    entry: &crate::integrations::local_store::HolidayCacheEntry,
    today: NaiveDate,
) -> bool {
    let fetched = NaiveDate::parse_from_str(&entry.fetched_at, "%Y-%m-%d")
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
    (today - fetched).num_days() <= CACHE_REFRESH_DAYS
}

async fn fetch_holidays_from_api(app: &tauri::AppHandle, year: i32) -> Result<Vec<Holiday>, String> {
    let url = format!("{NAGER_BASE_URL}/{year}/DE");
    fetch_from_url(Some(app), &url).await
}

async fn fetch_from_url(app: Option<&tauri::AppHandle>, url: &str) -> Result<Vec<Holiday>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|_| "Feiertage konnten nicht geladen werden".to_string())?;

    let response = client.get(url).send().await.map_err(|e| {
        record_request_failure(app, RequestFailure::new("REQUEST_FAILED", e.to_string()));
        "Feiertage konnten nicht geladen werden".to_string()
    })?;

    if !response.status().is_success() {
        record_request_failure(
            app,
            RequestFailure::with_status("UNEXPECTED_STATUS", response.status().as_u16()),
        );
        return Err("Feiertage konnten nicht geladen werden".to_string());
    }

    let body = response.text().await.map_err(|e| {
        record_request_failure(app, RequestFailure::new("BODY_READ_FAILED", e.to_string()));
        "Feiertage konnten nicht geladen werden".to_string()
    })?;

    let nager_holidays: Vec<NagerHoliday> = serde_json::from_str(&body).map_err(|e| {
        record_request_failure(app, RequestFailure::new("INVALID_RESPONSE", e.to_string()));
        "Feiertage konnten nicht geladen werden".to_string()
    })?;

    Ok(filter_holidays(nager_holidays))
}

fn record_request_failure(app: Option<&tauri::AppHandle>, failure: RequestFailure) {
    if let Some(app) = app {
        record_failure(
            app,
            Operation::HolidayApiRequest,
            Integration::Holidays,
            &failure,
        );
    }
}

fn filter_holidays(nager_holidays: Vec<NagerHoliday>) -> Vec<Holiday> {
    nager_holidays
        .into_iter()
        .filter(|h| {
            h.global
                || h.counties
                    .as_ref()
                    .is_some_and(|c| c.iter().any(|s| s == "DE-MV"))
        })
        .map(|h| Holiday {
            date: h.date,
            name: h.local_name,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::local_store::{CachedHoliday, HolidayCacheEntry};

    #[test]
    fn includes_global_holidays() {
        let nager = vec![NagerHoliday {
            date: "2024-01-01".to_string(),
            local_name: "Neujahr".to_string(),
            global: true,
            counties: None,
        }];
        let result = filter_holidays(nager);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Neujahr");
        assert_eq!(result[0].date, "2024-01-01");
    }

    #[test]
    fn includes_de_mv_specific_holidays() {
        let nager = vec![NagerHoliday {
            date: "2024-10-31".to_string(),
            local_name: "Reformationstag".to_string(),
            global: false,
            counties: Some(vec!["DE-MV".to_string(), "DE-HH".to_string()]),
        }];
        let result = filter_holidays(nager);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Reformationstag");
    }

    #[test]
    fn excludes_other_state_holidays() {
        let nager = vec![NagerHoliday {
            date: "2024-11-01".to_string(),
            local_name: "Allerheiligen".to_string(),
            global: false,
            counties: Some(vec!["DE-BY".to_string(), "DE-BW".to_string()]),
        }];
        let result = filter_holidays(nager);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn excludes_non_global_holidays_with_no_counties() {
        let nager = vec![NagerHoliday {
            date: "2024-06-01".to_string(),
            local_name: "Lokaler Feiertag".to_string(),
            global: false,
            counties: None,
        }];
        let result = filter_holidays(nager);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn uses_local_name_as_holiday_name() {
        let nager = vec![NagerHoliday {
            date: "2024-12-25".to_string(),
            local_name: "1. Weihnachtstag".to_string(),
            global: true,
            counties: None,
        }];
        let result = filter_holidays(nager);
        assert_eq!(result[0].name, "1. Weihnachtstag");
    }

    #[test]
    fn single_year_week_returns_one_year() {
        let start = NaiveDate::from_ymd_opt(2024, 6, 3).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 6, 9).unwrap();
        assert_eq!(years_for_week(start, end), vec![2024]);
    }

    #[test]
    fn year_boundary_week_returns_two_years() {
        let start = NaiveDate::from_ymd_opt(2024, 12, 30).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        assert_eq!(years_for_week(start, end), vec![2024, 2025]);
    }

    fn cache_entry(year: i32, holidays: &[(&str, &str)]) -> HolidayCacheEntry {
        HolidayCacheEntry {
            year,
            fetched_at: "2026-01-01".to_string(),
            holidays: holidays
                .iter()
                .map(|(date, name)| CachedHoliday {
                    date: date.to_string(),
                    name: name.to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn week_filter_merges_holidays_from_two_years() {
        let cache = [
            cache_entry(
                2024,
                &[
                    ("2024-12-25", "1. Weihnachtstag"),
                    ("2024-12-31", "Silvester"),
                ],
            ),
            cache_entry(
                2025,
                &[("2025-01-01", "Neujahr"), ("2025-04-18", "Karfreitag")],
            ),
        ];

        let filtered = holidays_in_range(
            &cache,
            NaiveDate::from_ymd_opt(2024, 12, 30).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(),
        );

        let dates: Vec<&str> = filtered.iter().map(|h| h.date.as_str()).collect();
        assert_eq!(dates, vec!["2024-12-31", "2025-01-01"]);
    }

    #[test]
    fn week_filter_includes_holidays_on_weekend_days() {
        let start = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        let cache = [cache_entry(
            2026,
            &[
                ("2026-01-31", "Samstagsfeiertag"),
                ("2026-02-01", "Sonntagsfeiertag"),
            ],
        )];

        let filtered = holidays_in_range(&cache, start, start + chrono::Duration::days(6));

        let dates: Vec<&str> = filtered.iter().map(|h| h.date.as_str()).collect();
        assert_eq!(dates, vec!["2026-01-31", "2026-02-01"]);
    }

    #[test]
    fn week_filter_excludes_holidays_outside_the_week() {
        let cache = [cache_entry(
            2026,
            &[("2026-01-25", "Davor"), ("2026-02-02", "Danach")],
        )];

        let filtered = holidays_in_range(
            &cache,
            NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        );

        assert!(filtered.is_empty());
    }

    #[test]
    fn cache_entry_is_fresh_within_30_days() {
        let today = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let entry = HolidayCacheEntry {
            year: 2024,
            fetched_at: "2024-06-10".to_string(),
            holidays: vec![],
        };
        assert!(is_cache_entry_fresh(&entry, today));
    }

    #[test]
    fn cache_entry_is_stale_after_30_days() {
        let today = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let entry = HolidayCacheEntry {
            year: 2024,
            fetched_at: "2024-05-29".to_string(),
            holidays: vec![],
        };
        assert!(!is_cache_entry_fresh(&entry, today));
    }

    #[test]
    fn cache_entry_is_fresh_exactly_at_30_days() {
        let today = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        let entry = HolidayCacheEntry {
            year: 2024,
            fetched_at: "2024-05-31".to_string(),
            holidays: vec![],
        };
        assert!(is_cache_entry_fresh(&entry, today));
    }
}
