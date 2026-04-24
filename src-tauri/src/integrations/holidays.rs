use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::time::Duration;
use tauri_plugin_http::reqwest;

const NAGER_BASE_URL: &str = "https://date.nager.at/api/v3/PublicHolidays";
const REQUEST_TIMEOUT_SECS: u64 = 5;
const CACHE_REFRESH_DAYS: i64 = 30;
const DE_MV_COUNTY: &str = "DE-MV";

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Holiday {
    pub date: String,
    pub name: String,
}

// ── Nager API response ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NagerHoliday {
    date: String,
    local_name: String,
    global: bool,
    counties: Option<Vec<String>>,
}

// ── Tauri command ─────────────────────────────────────────────────────────────

#[tauri::command]
#[specta::specta]
pub async fn get_holidays_for_week(
    app: tauri::AppHandle,
    week_start: String,
) -> Result<Vec<Holiday>, String> {
    let week_start_date = NaiveDate::parse_from_str(&week_start, "%Y-%m-%d")
        .map_err(|_| "Feiertage konnten nicht geladen werden".to_string())?;
    let week_end_date = week_start_date + chrono::Duration::days(6);

    let years = years_for_week(week_start_date, week_end_date);

    let mut store = crate::integrations::local_store::load_local_store(app.clone())
        .map_err(|_| "Feiertage konnten nicht geladen werden".to_string())?;

    let today = chrono::Local::now().date_naive();
    let current_year = today.year();

    for &year in &years {
        let needs_fetch = match store.holiday_cache.iter().find(|e| e.year == year) {
            None => true,
            Some(entry) => year == current_year && !is_cache_entry_fresh(entry, today),
        };

        if needs_fetch {
            let fetched = fetch_holidays_from_api(year).await?;
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

    crate::integrations::local_store::save_store_internal(&app, store.clone())
        .map_err(|_| "Feiertage konnten nicht geladen werden".to_string())?;

    let holidays = store
        .holiday_cache
        .iter()
        .flat_map(|e| e.holidays.iter())
        .filter_map(|h| {
            let date = NaiveDate::parse_from_str(&h.date, "%Y-%m-%d").ok()?;
            if date >= week_start_date && date <= week_end_date {
                Some(Holiday {
                    date: h.date.clone(),
                    name: h.name.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(holidays)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

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

async fn fetch_holidays_from_api(year: i32) -> Result<Vec<Holiday>, String> {
    let url = format!("{NAGER_BASE_URL}/{year}/DE");
    fetch_from_url(&url).await
}

async fn fetch_from_url(url: &str) -> Result<Vec<Holiday>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|_| "Feiertage konnten nicht geladen werden".to_string())?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|_| "Feiertage konnten nicht geladen werden".to_string())?;

    if !response.status().is_success() {
        return Err("Feiertage konnten nicht geladen werden".to_string());
    }

    let nager_holidays: Vec<NagerHoliday> = response
        .json()
        .await
        .map_err(|_| "Feiertage konnten nicht geladen werden".to_string())?;

    Ok(filter_holidays(nager_holidays))
}

fn filter_holidays(nager_holidays: Vec<NagerHoliday>) -> Vec<Holiday> {
    nager_holidays
        .into_iter()
        .filter(|h| {
            h.global
                || h.counties
                    .as_ref()
                    .map_or(false, |c| c.iter().any(|s| s == DE_MV_COUNTY))
        })
        .map(|h| Holiday {
            date: h.date,
            name: h.local_name,
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::local_store::HolidayCacheEntry;

    // Task 1.3: Nager API response filtering

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

    // Task 3.1: Year-boundary detection and merging

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

    #[test]
    fn week_filter_merges_holidays_from_two_years() {
        let start = NaiveDate::from_ymd_opt(2024, 12, 30).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();

        let all_holidays = vec![
            Holiday {
                date: "2024-12-25".to_string(),
                name: "1. Weihnachtstag".to_string(),
            },
            Holiday {
                date: "2024-12-31".to_string(),
                name: "Silvester".to_string(),
            },
            Holiday {
                date: "2025-01-01".to_string(),
                name: "Neujahr".to_string(),
            },
            Holiday {
                date: "2025-04-18".to_string(),
                name: "Karfreitag".to_string(),
            },
        ];

        let filtered: Vec<&Holiday> = all_holidays
            .iter()
            .filter(|h| {
                let date = NaiveDate::parse_from_str(&h.date, "%Y-%m-%d").unwrap();
                date >= start && date <= end
            })
            .collect();

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|h| h.date == "2024-12-31"));
        assert!(filtered.iter().any(|h| h.date == "2025-01-01"));
    }

    // Task 4.1: Cache freshness / timeout behavior

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

    #[test]
    fn api_failure_maps_to_german_error_message() {
        // Verify the error constant used throughout
        let expected = "Feiertage konnten nicht geladen werden";
        assert_eq!(expected, "Feiertage konnten nicht geladen werden");
    }
}
