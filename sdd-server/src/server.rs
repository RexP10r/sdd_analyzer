use std::collections::HashSet;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use sdd_core::models::{Annotation, Classification};
use sdd_core::models::Task;
use sdd_engine::coverage::{compute_project_stats, compute_requirement_status};

use crate::state::{AppState, ScanStatus};

// ── Query params ─────────────────────────────────────────────

/// @req SCS-API-001
#[derive(Deserialize, Default)]
pub struct RequirementsQuery {
    #[serde(rename = "type")]
    pub filter_type: Option<String>,
    pub status: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

/// @req SCS-API-001
#[derive(Deserialize, Default)]
pub struct AnnotationsQuery {
    #[serde(rename = "type")]
    pub filter_type: Option<String>,
    pub orphans: Option<bool>,
}

/// @req SCS-API-001
#[derive(Deserialize, Default)]
pub struct TasksQuery {
    pub status: Option<String>,
    pub orphans: Option<bool>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

// ── Response types ───────────────────────────────────────────

/// @req SCS-API-002
#[derive(Serialize)]
struct HealthcheckResponse {
    status: String,
    version: String,
    timestamp: String,
}

/// @req SCS-API-002
#[derive(Serialize)]
struct StatsResponse {
    requirements: RequirementStats,
    annotations: AnnotationStats,
    tasks: TaskStats,
    coverage: f64,
    #[serde(rename = "lastScanAt")]
    last_scan_at: Option<String>,
}

/// @req SCS-API-002
#[derive(Serialize)]
struct RequirementStats {
    total: usize,
    #[serde(rename = "byType")]
    by_type: std::collections::HashMap<String, usize>,
    #[serde(rename = "byStatus")]
    by_status: std::collections::HashMap<String, usize>,
}

/// @req SCS-API-002
#[derive(Serialize)]
struct AnnotationStats {
    total: usize,
    #[serde(rename = "impl")]
    impl_count: usize,
    test: usize,
    orphans: usize,
}

/// @req SCS-API-002
#[derive(Serialize)]
struct TaskStats {
    total: usize,
    #[serde(rename = "byStatus")]
    by_status: std::collections::HashMap<String, usize>,
    orphans: usize,
}

/// @req SCS-API-002
#[derive(Serialize)]
struct RequirementResponse {
    id: String,
    #[serde(rename = "type")]
    req_type: String,
    title: String,
    description: String,
    status: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

/// @req SCS-API-002
#[derive(Serialize)]
struct RequirementDetailResponse {
    id: String,
    #[serde(rename = "type")]
    req_type: String,
    title: String,
    description: String,
    status: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    annotations: Vec<Annotation>,
    tasks: Vec<Task>,
}

/// @req SCS-API-002
#[derive(Serialize)]
struct ScanStatusResponse {
    status: String,
    #[serde(rename = "startedAt")]
    started_at: Option<String>,
    #[serde(rename = "completedAt")]
    completed_at: Option<String>,
    duration: Option<u64>,
}

/// @req SCS-API-002
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

// ── Router ───────────────────────────────────────────────────

/// @req SCS-API-001
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/stats", get(get_stats))
        .route("/requirements", get(get_requirements))
        .route("/requirements/{id}", get(get_requirement_detail))
        .route("/annotations", get(get_annotations))
        .route("/tasks", get(get_tasks))
        .route("/scan", get(get_scan_status).post(post_scan))
        .with_state(state)
}

// ── Handlers ─────────────────────────────────────────────────

/// @req SCS-API-002
async fn healthcheck() -> Json<HealthcheckResponse> {
    Json(HealthcheckResponse {
        status: "healthy".to_string(),
        version: "0.1.0".to_string(),
        timestamp: chrono_now(),
    })
}

/// @req SCS-API-002
/// @req SCS-COV-003
async fn get_stats(State(state): State<AppState>) -> Json<StatsResponse> {
    let inner = state.read().await;
    let stats = compute_project_stats(&inner.requirements, &inner.annotations, &inner.tasks);

    let mut by_status = std::collections::HashMap::new();
    by_status.insert("covered".to_string(), stats.covered);
    by_status.insert("partial".to_string(), stats.partial);
    by_status.insert("missing".to_string(), stats.missing);

    let mut task_by_status = std::collections::HashMap::new();
    task_by_status.insert("open".to_string(), stats.tasks_open);
    task_by_status.insert("in_progress".to_string(), stats.tasks_in_progress);
    task_by_status.insert("done".to_string(), stats.tasks_done);

    Json(StatsResponse {
        requirements: RequirementStats {
            total: stats.total_requirements,
            by_type: stats.by_type,
            by_status,
        },
        annotations: AnnotationStats {
            total: stats.total_annotations,
            impl_count: stats.impl_count,
            test: stats.test_count,
            orphans: stats.orphan_annotations,
        },
        tasks: TaskStats {
            total: stats.total_tasks,
            by_status: task_by_status,
            orphans: stats.orphan_tasks,
        },
        coverage: (stats.coverage_pct * 10.0).round() / 10.0,
        last_scan_at: None,
    })
}

/// @req SCS-API-002
async fn get_requirements(
    State(state): State<AppState>,
    Query(query): Query<RequirementsQuery>,
) -> Json<Vec<RequirementResponse>> {
    let inner = state.read().await;
    let mut results: Vec<RequirementResponse> = inner
        .requirements
        .iter()
        .filter(|r| {
            if let Some(ref t) = query.filter_type {
                if r.req_type() != *t {
                    return false;
                }
            }
            if let Some(ref s) = query.status {
                let status = compute_requirement_status(&r.id, &inner.annotations);
                if status.as_str() != *s {
                    return false;
                }
            }
            true
        })
        .map(|r| {
            let status = compute_requirement_status(&r.id, &inner.annotations);
            RequirementResponse {
                id: r.id.clone(),
                req_type: r.req_type().to_string(),
                title: r.title.clone(),
                description: r.description.clone(),
                status: status.as_str().to_string(),
                created_at: String::new(),
                updated_at: String::new(),
            }
        })
        .collect();

    if query.sort.as_deref() == Some("id") {
        results.sort_by(|a, b| a.id.cmp(&b.id));
    }
    if query.order.as_deref() == Some("desc") {
        results.reverse();
    }

    Json(results)
}

/// @req SCS-API-002
async fn get_requirement_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RequirementDetailResponse>, (StatusCode, Json<ErrorResponse>)> {
    let inner = state.read().await;
    let req = inner
        .requirements
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "not_found".to_string(),
                    message: format!("Requirement '{}' not found", id),
                }),
            )
        })?;

    let status = compute_requirement_status(&req.id, &inner.annotations);
    let annotations: Vec<Annotation> = inner
        .annotations
        .iter()
        .filter(|a| a.requirement_id == id)
        .map(Annotation::from)
        .collect();
    let tasks: Vec<Task> = inner
        .tasks
        .iter()
        .filter(|t| t.requirement_id == id)
        .cloned()
        .collect();

    Ok(Json(RequirementDetailResponse {
        id: req.id.clone(),
        req_type: req.req_type().to_string(),
        title: req.title.clone(),
        description: req.description.clone(),
        status: status.as_str().to_string(),
        created_at: String::new(),
        updated_at: String::new(),
        annotations,
        tasks,
    }))
}

/// @req SCS-API-002
async fn get_annotations(
    State(state): State<AppState>,
    Query(query): Query<AnnotationsQuery>,
) -> Json<Vec<Annotation>> {
    let inner = state.read().await;
    let req_ids: HashSet<String> = inner.requirements.iter().map(|r| r.id.clone()).collect();

    let results: Vec<Annotation> = inner
        .annotations
        .iter()
        .filter(|a| {
            if let Some(ref t) = query.filter_type {
                let expected = match t.as_str() {
                    "impl" => Classification::Impl,
                    "test" => Classification::Test,
                    _ => return true,
                };
                if a.classification != expected {
                    return false;
                }
            }
            if query.orphans.unwrap_or(false)
                && req_ids.contains(&a.requirement_id)
            {
                return false;
            }
            true
        })
        .map(Annotation::from)
        .collect();

    Json(results)
}

/// @req SCS-API-002
async fn get_tasks(
    State(state): State<AppState>,
    Query(query): Query<TasksQuery>,
) -> Json<Vec<Task>> {
    let inner = state.read().await;
    let req_ids: HashSet<String> = inner.requirements.iter().map(|r| r.id.clone()).collect();

    let mut results: Vec<Task> = inner
        .tasks
        .iter()
        .filter(|t| {
            if let Some(ref s) = query.status {
                if t.status != *s {
                    return false;
                }
            }
            if query.orphans.unwrap_or(false)
                && req_ids.contains(&t.requirement_id)
            {
                return false;
            }
            true
        })
        .cloned()
        .collect();

    if query.sort.as_deref() == Some("id") {
        results.sort_by(|a, b| a.id.cmp(&b.id));
    }
    if query.order.as_deref() == Some("desc") {
        results.reverse();
    }

    Json(results)
}

/// @req SCS-API-002
async fn post_scan(State(state): State<AppState>) -> (StatusCode, Json<ScanStatusResponse>) {
    let mut inner = state.write().await;
    if inner.scan_status == ScanStatus::Scanning {
        return (
            StatusCode::CONFLICT,
            Json(ScanStatusResponse {
                status: "scanning".to_string(),
                started_at: None,
                completed_at: None,
                duration: None,
            }),
        );
    }

    inner.scan_status = ScanStatus::Scanning;
    drop(inner);

    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        run_scan(state_clone).await;
    });

    (
        StatusCode::ACCEPTED,
        Json(ScanStatusResponse {
            status: "scanning".to_string(),
            started_at: Some(chrono_now()),
            completed_at: None,
            duration: None,
        }),
    )
}

/// @req SCS-API-002
async fn get_scan_status(State(state): State<AppState>) -> Json<ScanStatusResponse> {
    let inner = state.read().await;
    match &inner.scan_status {
        ScanStatus::Idle => Json(ScanStatusResponse {
            status: "idle".to_string(),
            started_at: None,
            completed_at: None,
            duration: None,
        }),
        ScanStatus::Scanning => Json(ScanStatusResponse {
            status: "scanning".to_string(),
            started_at: None,
            completed_at: None,
            duration: None,
        }),
        ScanStatus::Completed => Json(ScanStatusResponse {
            status: "completed".to_string(),
            started_at: None,
            completed_at: None,
            duration: None,
        }),
        ScanStatus::Failed(ref msg) => Json(ScanStatusResponse {
            status: format!("failed: {}", msg),
            started_at: None,
            completed_at: None,
            duration: None,
        }),
    }
}

// ── Background scan ──────────────────────────────────────────

/// @req SCS-API-002
/// @req SCS-ERR-001
async fn run_scan(state: AppState) {
    let requirements = sdd_engine::parser::parse_requirements(std::path::Path::new("../requirements.yaml"))
        .unwrap_or_default();
    let tasks = sdd_engine::parser::parse_tasks(std::path::Path::new("../tasks.yaml"))
        .unwrap_or_default();
    let scan_result = sdd_engine::scanner::scan_directory(std::path::Path::new(".."))
        .unwrap_or_else(|e| {
            tracing::warn!("Scan error: {}", e);
            sdd_engine::scanner::ScanResult {
                annotations: Vec::new(),
                warnings: vec![e.to_string()],
            }
        });

    let mut inner = state.write().await;
    inner.requirements = requirements;
    inner.tasks = tasks;
    inner.annotations = scan_result.annotations;
    inner.scan_status = ScanStatus::Completed;
}

// ── Helpers ──────────────────────────────────────────────────

fn chrono_now() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let nsecs = dur.subsec_nanos();

    let (y, m, d, h, min, s) = unix_to_utc(secs as i64);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        y, m, d, h, min, s, nsecs / 1_000_000
    )
}

fn unix_to_utc(ts: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = ts / 86400;
    let time = ts % 86400;
    let h = (time / 3600) as u32;
    let min = ((time % 3600) / 60) as u32;
    let s = (time % 60) as u32;

    let mut y = 1970i64;
    let mut remaining = days;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let months = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 1u32;
    for &mdays in months.iter() {
        if remaining < mdays as i64 {
            break;
        }
        remaining -= mdays as i64;
        m += 1;
    }

    (y, m, (remaining + 1) as u32, h, min, s)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

// ── Server start ─────────────────────────────────────────────

/// @req SCS-API-001
/// @req SCS-ERR-001
pub async fn run(state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
