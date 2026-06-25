use std::collections::{HashMap, HashSet};

use sdd_core::models::{
    build_req_ids, AnnotatedLocation, Annotation, Classification, CoverageStatus, Requirement,
    Task, TASK_STATUS_DONE, TASK_STATUS_IN_PROGRESS, TASK_STATUS_OPEN,
};

/// @req SCS-COV-001
pub fn compute_requirement_status(
    requirement_id: &str,
    annotations: &[AnnotatedLocation],
) -> CoverageStatus {
    let relevant: Vec<_> = annotations
        .iter()
        .filter(|a| a.requirement_id == requirement_id)
        .collect();

    let has_impl = relevant
        .iter()
        .any(|a| a.classification == Classification::Impl);
    let has_test = relevant
        .iter()
        .any(|a| a.classification == Classification::Test);

    match (has_impl, has_test) {
        (true, true) => CoverageStatus::Covered,
        (true, false) | (false, true) => CoverageStatus::Partial,
        (false, false) => CoverageStatus::Missing,
    }
}

/// @req SCS-COV-002
pub fn find_orphan_annotations(
    annotations: &[AnnotatedLocation],
    requirement_ids: &HashSet<String>,
) -> Vec<Annotation> {
    annotations
        .iter()
        .filter(|a| !requirement_ids.contains(&a.requirement_id))
        .map(Annotation::from)
        .collect()
}

/// @req SCS-COV-002
pub fn find_orphan_tasks(tasks: &[Task], requirement_ids: &HashSet<String>) -> Vec<Task> {
    tasks
        .iter()
        .filter(|t| !requirement_ids.contains(&t.requirement_id))
        .cloned()
        .collect()
}

/// @req SCS-COV-003
#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub total_requirements: usize,
    pub covered: usize,
    pub partial: usize,
    pub missing: usize,
    pub coverage_pct: f64,
    pub total_annotations: usize,
    pub impl_count: usize,
    pub test_count: usize,
    pub orphan_annotations: usize,
    pub total_tasks: usize,
    pub tasks_open: usize,
    pub tasks_in_progress: usize,
    pub tasks_done: usize,
    pub orphan_tasks: usize,
    pub by_type: HashMap<String, usize>,
}

/// @req SCS-COV-003
pub fn compute_project_stats(
    requirements: &[Requirement],
    annotations: &[AnnotatedLocation],
    tasks: &[Task],
) -> ProjectStats {
    let req_ids = build_req_ids(requirements);

    let mut covered = 0usize;
    let mut partial = 0usize;
    let mut missing = 0usize;
    let mut by_type: HashMap<String, usize> = HashMap::new();

    for req in requirements {
        let status = compute_requirement_status(&req.id, annotations);
        match status {
            CoverageStatus::Covered => covered += 1,
            CoverageStatus::Partial => partial += 1,
            CoverageStatus::Missing => missing += 1,
        }
        *by_type.entry(req.req_type().to_string()).or_insert(0) += 1;
    }

    let total = requirements.len();
    let coverage_pct = if total > 0 {
        (covered as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let impl_count = annotations
        .iter()
        .filter(|a| a.classification == Classification::Impl)
        .count();
    let test_count = annotations
        .iter()
        .filter(|a| a.classification == Classification::Test)
        .count();

    let orphan_annotations = find_orphan_annotations(annotations, &req_ids).len();
    let orphan_tasks_list = find_orphan_tasks(tasks, &req_ids);
    let orphan_tasks = orphan_tasks_list.len();

    let tasks_open = tasks
        .iter()
        .filter(|t| t.status == TASK_STATUS_OPEN)
        .count();
    let tasks_in_progress = tasks
        .iter()
        .filter(|t| t.status == TASK_STATUS_IN_PROGRESS)
        .count();
    let tasks_done = tasks
        .iter()
        .filter(|t| t.status == TASK_STATUS_DONE)
        .count();

    ProjectStats {
        total_requirements: total,
        covered,
        partial,
        missing,
        coverage_pct,
        total_annotations: annotations.len(),
        impl_count,
        test_count,
        orphan_annotations,
        total_tasks: tasks.len(),
        tasks_open,
        tasks_in_progress,
        tasks_done,
        orphan_tasks,
        by_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @req SCS-COV-001
    /// @req SCS-TEST-001
    #[test]
    fn test_covered_has_both_impl_and_test() {
        let annotations = vec![
            AnnotatedLocation {
                file_path: "src/lib.rs".into(),
                line_number: 1,
                requirement_id: "REQ-001".into(),
                classification: Classification::Impl,
                snippet: String::new(),
            },
            AnnotatedLocation {
                file_path: "tests/lib_test.rs".into(),
                line_number: 2,
                requirement_id: "REQ-001".into(),
                classification: Classification::Test,
                snippet: String::new(),
            },
        ];
        assert_eq!(
            compute_requirement_status("REQ-001", &annotations),
            CoverageStatus::Covered
        );
    }

    /// @req SCS-COV-001
    /// @req SCS-TEST-001
    #[test]
    fn test_partial_has_only_impl() {
        let annotations = vec![AnnotatedLocation {
            file_path: "src/lib.rs".into(),
            line_number: 1,
            requirement_id: "REQ-001".into(),
            classification: Classification::Impl,
            snippet: String::new(),
        }];
        assert_eq!(
            compute_requirement_status("REQ-001", &annotations),
            CoverageStatus::Partial
        );
    }

    /// @req SCS-COV-001
    /// @req SCS-TEST-001
    #[test]
    fn test_missing_has_no_annotations() {
        let annotations: Vec<AnnotatedLocation> = vec![];
        assert_eq!(
            compute_requirement_status("REQ-001", &annotations),
            CoverageStatus::Missing
        );
    }

    /// @req SCS-COV-002
    /// @req SCS-TEST-001
    #[test]
    fn test_orphan_detection() {
        let mut req_ids = HashSet::new();
        req_ids.insert("REQ-001".to_string());

        let annotations = vec![
            AnnotatedLocation {
                file_path: "src/lib.rs".into(),
                line_number: 1,
                requirement_id: "REQ-001".into(),
                classification: Classification::Impl,
                snippet: String::new(),
            },
            AnnotatedLocation {
                file_path: "src/old.rs".into(),
                line_number: 5,
                requirement_id: "REQ-999".into(),
                classification: Classification::Impl,
                snippet: String::new(),
            },
        ];

        let orphans = find_orphan_annotations(&annotations, &req_ids);
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].req_id, "REQ-999");
    }

    /// @req SCS-COV-003
    /// @req SCS-TEST-001
    #[test]
    fn test_project_stats() {
        let requirements = vec![Requirement {
            id: "REQ-001".into(),
            title: "Test".into(),
            description: "Desc".into(),
        }];
        let annotations = vec![AnnotatedLocation {
            file_path: "src/lib.rs".into(),
            line_number: 1,
            requirement_id: "REQ-001".into(),
            classification: Classification::Impl,
            snippet: String::new(),
        }];
        let tasks = vec![];

        let stats = compute_project_stats(&requirements, &annotations, &tasks);
        assert_eq!(stats.total_requirements, 1);
        assert_eq!(stats.covered, 0);
        assert_eq!(stats.partial, 1);
        assert_eq!(stats.missing, 0);
        assert_eq!(stats.coverage_pct, 0.0);
        assert_eq!(stats.total_annotations, 1);
        assert_eq!(stats.impl_count, 1);
        assert_eq!(stats.test_count, 0);
    }
}
