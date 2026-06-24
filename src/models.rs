use serde::Deserialize;

/// @req SCS-PARSE-001
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub description: String,
}

/// @req SCS-PARSE-001
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RequirementsFile {
    pub requirements: Vec<Requirement>,
}

/// @req SCS-PARSE-002
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Task {
    pub id: String,
    #[serde(rename = "requirementId")]
    pub requirement_id: String,
    pub title: String,
    pub status: String,
}

/// @req SCS-PARSE-002
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TasksFile {
    pub tasks: Vec<Task>,
}

/// @req SCS-SCAN-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Classification {
    Impl,
    Test,
}

/// @req SCS-SCAN-001
/// @req SCS-SCAN-002
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedLocation {
    pub file_path: String,
    pub line_number: usize,
    pub requirement_id: String,
    pub classification: Classification,
}
