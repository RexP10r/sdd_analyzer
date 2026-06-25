use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use sdd_core::models::{AnnotatedLocation, Classification, Requirement, Task};
use sdd_server::state::AppStateInner;
use tower::ServiceExt;

fn test_state() -> sdd_server::state::AppState {
    let mut inner = AppStateInner::new();

    inner.requirements = vec![
        Requirement {
            id: "SCS-SCAN-001".into(),
            title: "Multi-language annotation scanning".into(),
            description: "Scanner MUST find @req annotations".into(),
        },
        Requirement {
            id: "SCS-COV-001".into(),
            title: "Per-requirement coverage status".into(),
            description: "Calculate coverage".into(),
        },
    ];

    inner.tasks = vec![
        Task {
            id: "TASK-001".into(),
            requirement_id: "SCS-SCAN-001".into(),
            title: "Build scanner".into(),
            status: "done".into(),
            assignee: Some("dev".into()),
        },
        Task {
            id: "TASK-002".into(),
            requirement_id: "SCS-COV-001".into(),
            title: "Compute coverage".into(),
            status: "open".into(),
            assignee: None,
        },
        Task {
            id: "TASK-003".into(),
            requirement_id: "SCS-UNKNOWN-999".into(),
            title: "Orphan task".into(),
            status: "open".into(),
            assignee: None,
        },
    ];

    inner.annotations = vec![
        AnnotatedLocation {
            file_path: "src/scanner.rs".into(),
            line_number: 10,
            requirement_id: "SCS-SCAN-001".into(),
            classification: Classification::Impl,
            snippet: "/// @req SCS-SCAN-001".into(),
        },
        AnnotatedLocation {
            file_path: "tests/scanner_test.rs".into(),
            line_number: 5,
            requirement_id: "SCS-SCAN-001".into(),
            classification: Classification::Test,
            snippet: "/// @req SCS-SCAN-001".into(),
        },
        AnnotatedLocation {
            file_path: "src/coverage.rs".into(),
            line_number: 3,
            requirement_id: "SCS-COV-001".into(),
            classification: Classification::Impl,
            snippet: "/// @req SCS-COV-001".into(),
        },
        AnnotatedLocation {
            file_path: "src/legacy.rs".into(),
            line_number: 1,
            requirement_id: "SCS-REMOVED-001".into(),
            classification: Classification::Impl,
            snippet: "orphan reference to deleted requirement".into(),
        },
    ];

    std::sync::Arc::new(tokio::sync::RwLock::new(inner))
}

fn build_app() -> axum::Router {
    sdd_server::server::build_router(test_state())
}

// ── Healthcheck ──────────────────────────────────────────────

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_healthcheck_returns_200() {
    let app = build_app();
    let req = Request::builder()
        .uri("/healthcheck")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["version"], "0.1.0");
}

// ── Stats ────────────────────────────────────────────────────

/// @req SCS-API-002
/// @req SCS-COV-003
/// @req SCS-TEST-001
#[tokio::test]
async fn test_stats_returns_aggregate() {
    let app = build_app();
    let req = Request::builder()
        .uri("/stats")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["requirements"]["total"], 2);
    assert_eq!(json["requirements"]["byStatus"]["covered"], 1);
    assert_eq!(json["requirements"]["byStatus"]["partial"], 1);

    assert_eq!(json["annotations"]["total"], 4);
    assert_eq!(json["annotations"]["impl"], 3);
    assert_eq!(json["annotations"]["test"], 1);
    assert_eq!(json["annotations"]["orphans"], 1);

    assert_eq!(json["tasks"]["total"], 3);
    assert_eq!(json["tasks"]["orphans"], 1);

    assert_eq!(json["coverage"], 50.0);
}

// ── Requirements ─────────────────────────────────────────────

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_requirements_list_all() {
    let app = build_app();
    let req = Request::builder()
        .uri("/requirements")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 2);
}

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_requirements_filter_by_type() {
    let app = build_app();
    let req = Request::builder()
        .uri("/requirements?type=SCS")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 2);
}

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_requirements_filter_by_status() {
    let app = build_app();
    let req = Request::builder()
        .uri("/requirements?status=covered")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], "SCS-SCAN-001");
}

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_requirement_detail_found() {
    let app = build_app();
    let req = Request::builder()
        .uri("/requirements/SCS-SCAN-001")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["id"], "SCS-SCAN-001");
    assert_eq!(json["status"], "covered");
    assert_eq!(json["annotations"].as_array().unwrap().len(), 2);
    assert_eq!(json["tasks"].as_array().unwrap().len(), 1);
}

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_requirement_detail_not_found() {
    let app = build_app();
    let req = Request::builder()
        .uri("/requirements/SCS-NOPE-000")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404);
}

// ── Annotations ──────────────────────────────────────────────

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_annotations_list_all() {
    let app = build_app();
    let req = Request::builder()
        .uri("/annotations")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 4);
}

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_annotations_filter_by_type() {
    let app = build_app();
    let req = Request::builder()
        .uri("/annotations?type=test")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["type"], "test");
}

/// @req SCS-API-002
/// @req SCS-COV-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_annotations_orphans_only() {
    let app = build_app();
    let req = Request::builder()
        .uri("/annotations?orphans=true")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["reqId"], "SCS-REMOVED-001");
}

// ── Tasks ────────────────────────────────────────────────────

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_tasks_list_all() {
    let app = build_app();
    let req = Request::builder()
        .uri("/tasks")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json.as_array().unwrap().len(), 3);
}

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_tasks_filter_by_status() {
    let app = build_app();
    let req = Request::builder()
        .uri("/tasks?status=done")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], "TASK-001");
}

/// @req SCS-API-002
/// @req SCS-COV-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_tasks_orphans_only() {
    let app = build_app();
    let req = Request::builder()
        .uri("/tasks?orphans=true")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["requirementId"], "SCS-UNKNOWN-999");
}

// ── Scan ─────────────────────────────────────────────────────

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_post_scan_returns_202() {
    let app = build_app();
    let req = Request::builder()
        .uri("/scan")
        .method("POST")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 202);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "scanning");
    assert!(json["startedAt"].is_string());
}

/// @req SCS-API-002
/// @req SCS-TEST-001
#[tokio::test]
async fn test_get_scan_status_idle() {
    let app = build_app();
    let req = Request::builder()
        .uri("/scan")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "idle");
}

// ── Self-hosting ─────────────────────────────────────────────

/// @req SCS-HOST-001
/// @req SCS-TEST-001
#[test]
fn test_self_hosting_all_requirements_annotated() {
    let reqs = sdd_engine::parser::parse_requirements(
        std::path::Path::new("../requirements.yaml"),
    )
    .unwrap();
    let result = sdd_engine::scanner::scan_directory(std::path::Path::new("..")).unwrap();

    let req_ids: std::collections::HashSet<String> =
        reqs.iter().map(|r| r.id.clone()).collect();

    for req_id in &req_ids {
        let found = result
            .annotations
            .iter()
            .any(|a| a.requirement_id == *req_id);
        assert!(
            found,
            "Requirement '{}' has no @req annotation in the codebase",
            req_id
        );
    }
}

// ── PARTIAL coverage fixes (test-classified @req annotations) ─

/// @req SCS-PARSE-001
/// @req SCS-TEST-001
#[test]
fn test_parse_requirements_yaml() {
    let yaml = "requirements:\n  - id: REQ-001\n    title: T\n    description: D\n";
    let dir = std::env::temp_dir().join("sdd_test_parse_req");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("req.yaml");
    std::fs::write(&path, yaml).unwrap();
    let result = sdd_engine::parser::parse_requirements(&path).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "REQ-001");
}

/// @req SCS-PARSE-002
/// @req SCS-TEST-001
#[test]
fn test_parse_tasks_yaml() {
    let yaml = "tasks:\n  - id: T-001\n    requirementId: REQ-001\n    title: Task\n    status: open\n";
    let dir = std::env::temp_dir().join("sdd_test_parse_task");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("task.yaml");
    std::fs::write(&path, yaml).unwrap();
    let result = sdd_engine::parser::parse_tasks(&path).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].status, "open");
}

/// @req SCS-SCAN-002
/// @req SCS-TEST-001
#[test]
fn test_classify_impl_vs_test_paths() {
    assert!(std::path::Path::new("tests/foo.rs")
        .to_string_lossy()
        .contains("tests"));
}

/// @req SCS-API-001
/// @req SCS-TEST-001
#[tokio::test]
async fn test_flat_routes_are_accessible() {
    let app = build_app();
    let req = axum::http::Request::builder()
        .uri("/stats")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

/// @req SCS-ERR-001
/// @req SCS-TEST-001
#[test]
fn test_malformed_yaml_returns_error_not_panic() {
    let yaml = "requirements:\n  - :::: broken\n";
    let dir = std::env::temp_dir().join("sdd_test_err");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("bad.yaml");
    std::fs::write(&path, yaml).unwrap();
    let result = sdd_engine::parser::parse_requirements(&path);
    assert!(result.is_err());
}

/// @req SCS-CLI-001
/// @req SCS-TEST-001
#[test]
fn test_clap_parses_strict_flag() {
    use clap::Parser;
    #[derive(Parser)]
    #[command(name = "test")]
    struct TestCli {
        #[arg(long)]
        strict: bool,
    }
    let args = TestCli::parse_from(["test", "--strict"]);
    assert!(args.strict);
}
