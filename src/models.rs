use serde::{Deserialize, Serialize};

// ── Parsed from YAML ────────────────────────────────────────

/// @req SCS-PARSE-001
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub description: String,
}

impl Requirement {
    /// Extract type from ID: "SCS-PARSE-001" → "SCS"
    pub fn req_type(&self) -> &str {
        self.id.split('-').next().unwrap_or("UNKNOWN")
    }
}

/// @req SCS-PARSE-001
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RequirementsFile {
    pub requirements: Vec<Requirement>,
}

/// @req SCS-PARSE-002
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Task {
    pub id: String,
    #[serde(rename = "requirementId")]
    pub requirement_id: String,
    pub title: String,
    pub status: String,
    #[serde(default)]
    pub assignee: Option<String>,
}

/// @req SCS-PARSE-002
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TasksFile {
    pub tasks: Vec<Task>,
}

// ── Scanner types ────────────────────────────────────────────

/// @req SCS-SCAN-002
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
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
    pub snippet: String,
}

/// @req SCS-SCAN-001
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Annotation {
    pub file: String,
    pub line: usize,
    #[serde(rename = "reqId")]
    pub req_id: String,
    #[serde(rename = "type")]
    pub annotation_type: Classification,
    pub snippet: String,
}

impl From<&AnnotatedLocation> for Annotation {
    fn from(loc: &AnnotatedLocation) -> Self {
        Annotation {
            file: loc.file_path.clone(),
            line: loc.line_number,
            req_id: loc.requirement_id.clone(),
            annotation_type: loc.classification,
            snippet: loc.snippet.clone(),
        }
    }
}

// ── Coverage types ───────────────────────────────────────────

/// @req SCS-COV-001
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CoverageStatus {
    Covered,
    Partial,
    Missing,
}

impl CoverageStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoverageStatus::Covered => "covered",
            CoverageStatus::Partial => "partial",
            CoverageStatus::Missing => "missing",
        }
    }
}
