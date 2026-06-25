use std::path::Path;

use serde::de::DeserializeOwned;

use sdd_core::error::AppError;
use sdd_core::models::{Requirement, RequirementsFile, Task, TasksFile};

/// @req SCS-PARSE-001
/// @req SCS-ERR-001
pub fn parse_requirements(path: &Path) -> Result<Vec<Requirement>, AppError> {
    let content = read_file(path)?;
    let file: RequirementsFile = parse_yaml(path, &content)?;
    if file.requirements.is_empty() {
        return Err(AppError::EmptyList {
            path: path.display().to_string(),
            key: "requirements".to_string(),
        });
    }
    for (i, req) in file.requirements.iter().enumerate() {
        validate_non_empty(path, i, "id", &req.id)?;
        validate_non_empty(path, i, "title", &req.title)?;
        validate_non_empty(path, i, "description", &req.description)?;
    }
    Ok(file.requirements)
}

/// @req SCS-PARSE-002
/// @req SCS-ERR-001
pub fn parse_tasks(path: &Path) -> Result<Vec<Task>, AppError> {
    let content = read_file(path)?;
    let file: TasksFile = parse_yaml(path, &content)?;
    if file.tasks.is_empty() {
        return Err(AppError::EmptyList {
            path: path.display().to_string(),
            key: "tasks".to_string(),
        });
    }
    for (i, task) in file.tasks.iter().enumerate() {
        validate_non_empty(path, i, "id", &task.id)?;
        validate_non_empty(path, i, "requirementId", &task.requirement_id)?;
        validate_non_empty(path, i, "title", &task.title)?;
        validate_non_empty(path, i, "status", &task.status)?;
    }
    Ok(file.tasks)
}

/// @req SCS-PARSE-001
fn read_file(path: &Path) -> Result<String, AppError> {
    std::fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.display().to_string(),
        source,
    })
}

/// @req SCS-PARSE-001
fn parse_yaml<T: DeserializeOwned + 'static>(path: &Path, content: &str) -> Result<T, AppError> {
    serde_yml::from_str::<T>(content).map_err(|e| {
        let line = e.location().map(|loc| loc.line()).unwrap_or(0);
        AppError::YamlParse {
            path: path.display().to_string(),
            line,
            message: e.to_string(),
        }
    })
}

/// @req SCS-PARSE-001
fn validate_non_empty(path: &Path, index: usize, field: &str, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::MissingField {
            path: path.display().to_string(),
            index,
            field: field.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @req SCS-PARSE-001
    /// @req SCS-TEST-001
    #[test]
    fn test_parse_valid_requirements() {
        let yaml = "requirements:\n  - id: REQ-001\n    title: Test\n    description: Desc\n";
        let tmp = write_temp("requirements.yaml", yaml);
        let result = parse_requirements(&tmp).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "REQ-001");
        assert_eq!(result[0].title, "Test");
        assert_eq!(result[0].description, "Desc");
    }

    /// @req SCS-PARSE-001
    /// @req SCS-TEST-001
    #[test]
    fn test_missing_key_in_requirements_fails() {
        let yaml = "wrong_key:\n  - id: REQ-001\n    title: T\n    description: D\n";
        let tmp = write_temp("requirements.yaml", yaml);
        let result = parse_requirements(&tmp);
        assert!(result.is_err());
    }

    /// @req SCS-PARSE-001
    /// @req SCS-TEST-001
    #[test]
    fn test_empty_field_in_requirements_fails() {
        let yaml = "requirements:\n  - id: ''\n    title: T\n    description: D\n";
        let tmp = write_temp("requirements.yaml", yaml);
        let result = parse_requirements(&tmp);
        assert!(result.is_err());
    }

    /// @req SCS-PARSE-001
    /// @req SCS-TEST-001
    #[test]
    fn test_empty_requirements_list_fails() {
        let yaml = "requirements: []\n";
        let tmp = write_temp("requirements.yaml", yaml);
        let result = parse_requirements(&tmp);
        assert!(result.is_err());
    }

    /// @req SCS-PARSE-002
    /// @req SCS-TEST-001
    #[test]
    fn test_parse_valid_tasks() {
        let yaml = "tasks:\n  - id: T-001\n    requirementId: REQ-001\n    title: Task\n    status: open\n";
        let tmp = write_temp("tasks.yaml", yaml);
        let result = parse_tasks(&tmp).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "T-001");
        assert_eq!(result[0].requirement_id, "REQ-001");
        assert_eq!(result[0].title, "Task");
        assert_eq!(result[0].status, "open");
    }

    /// @req SCS-PARSE-002
    /// @req SCS-TEST-001
    #[test]
    fn test_missing_root_key_in_tasks_fails() {
        let yaml =
            "wrong:\n  - id: T-001\n    requirementId: REQ-001\n    title: T\n    status: open\n";
        let tmp = write_temp("tasks.yaml", yaml);
        let result = parse_tasks(&tmp);
        assert!(result.is_err());
    }

    /// @req SCS-PARSE-002
    /// @req SCS-TEST-001
    #[test]
    fn test_empty_tasks_list_fails() {
        let yaml = "tasks: []\n";
        let tmp = write_temp("tasks.yaml", yaml);
        let result = parse_tasks(&tmp);
        assert!(result.is_err());
    }

    /// @req SCS-ERR-001
    /// @req SCS-TEST-001
    #[test]
    fn test_malformed_yaml_does_not_panic() {
        let yaml = "requirements:\n  - :::: bad syntax\n";
        let tmp = write_temp("requirements.yaml", yaml);
        let result = parse_requirements(&tmp);
        assert!(result.is_err());
    }

    /// @req SCS-ERR-001
    /// @req SCS-TEST-001
    #[test]
    fn test_missing_file_returns_error() {
        let result = parse_requirements(Path::new("/nonexistent/path.yaml"));
        assert!(result.is_err());
    }

    fn write_temp(name: &str, content: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join("sdd_test");
        std::fs::create_dir_all(&dir).unwrap();
        let unique_name = format!("{}_{}", id, name);
        let path = dir.join(unique_name);
        std::fs::write(&path, content).unwrap();
        path
    }
}

/// @req SCS-PARSE-001
/// @req SCS-PARSE-002
/// @req SCS-TEST-001
#[test]
fn test_parse_actual_project_files() {
    let reqs = parse_requirements(Path::new("../requirements.yaml")).unwrap();
    assert_eq!(reqs.len(), 13);
    assert_eq!(reqs[0].id, "SCS-PARSE-001");
    let tasks = parse_tasks(Path::new("../tasks.yaml")).unwrap();
    assert_eq!(tasks.len(), 8);
    assert_eq!(tasks[0].id, "TASK-001");
}
