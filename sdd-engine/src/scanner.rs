use std::path::Path;

use ignore::WalkBuilder;
use regex::Regex;

use sdd_core::error::AppError;
use sdd_core::models::{AnnotatedLocation, Classification};

const SCAN_EXTENSIONS: &[&str] = &["rs", "ts", "js", "py", "dart", "go"];

/// @req SCS-SCAN-001
/// @req SCS-SCAN-002
pub struct ScanResult {
    pub annotations: Vec<AnnotatedLocation>,
    pub warnings: Vec<String>,
}

/// @req SCS-SCAN-001
/// @req SCS-SCAN-002
/// @req SCS-ERR-001
pub fn scan_directory(root: &Path) -> Result<ScanResult, AppError> {
    let re = Regex::new(r"@req\s+([A-Z]+-[A-Z]+-\d+)").map_err(|e| AppError::Regex {
        message: e.to_string(),
    })?;

    let mut annotations = Vec::new();
    let mut warnings = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .filter_entry(|entry| is_scan_target(entry.path()))
        .build();

    for result in walker {
        let entry = match result {
            Ok(e) => e,
            Err(err) => {
                warnings.push(format!("Walk error: {}", err));
                continue;
            }
        };

        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }

        let file_path = entry.path();
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                warnings.push(format!(
                    "Cannot read '{}': {}",
                    file_path.display(),
                    e
                ));
                continue;
            }
        };

        let classification = classify(file_path);

        for (line_idx, line) in content.lines().enumerate() {
            for caps in re.captures_iter(line) {
                let requirement_id = caps.get(1).map(|m| m.as_str().to_string());
                if let Some(req_id) = requirement_id {
                    annotations.push(AnnotatedLocation {
                        file_path: file_path.display().to_string(),
                        line_number: line_idx + 1,
                        requirement_id: req_id,
                        classification,
                        snippet: line.trim().to_string(),
                    });
                }
            }
        }
    }

    Ok(ScanResult {
        annotations,
        warnings,
    })
}

/// @req SCS-SCAN-001
fn is_scan_target(path: &Path) -> bool {
    if path.is_dir() {
        return true;
    }
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| SCAN_EXTENSIONS.contains(&ext))
}

/// @req SCS-SCAN-002
fn classify(path: &Path) -> Classification {
    let path_str = path.to_string_lossy();
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    if file_stem.starts_with("test_")
        || file_stem.ends_with("_test")
        || ext == "test"
        || path_str.contains(".test.")
    {
        return Classification::Test;
    }

    if let Some(parent) = path.parent() {
        if let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) {
            if parent_name == "test" || parent_name == "tests" {
                return Classification::Test;
            }
        }
        let parent_str = parent.to_string_lossy();
        if parent_str.contains("/test/") || parent_str.contains("/tests/") {
            return Classification::Test;
        }
    }

    Classification::Impl
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @req SCS-SCAN-001
    /// @req SCS-TEST-001
    #[test]
    fn test_classify_impl_file() {
        assert_eq!(classify(Path::new("src/parser.rs")), Classification::Impl);
        assert_eq!(classify(Path::new("lib/models.ts")), Classification::Impl);
    }

    /// @req SCS-SCAN-002
    /// @req SCS-TEST-001
    #[test]
    fn test_classify_test_file() {
        assert_eq!(
            classify(Path::new("src/test_parser.rs")),
            Classification::Test
        );
        assert_eq!(
            classify(Path::new("src/parser_test.rs")),
            Classification::Test
        );
        assert_eq!(
            classify(Path::new("src/parser.test.ts")),
            Classification::Test
        );
        assert_eq!(
            classify(Path::new("tests/integration.rs")),
            Classification::Test
        );
    }

    /// @req SCS-SCAN-001
    /// @req SCS-TEST-001
    #[test]
    fn test_scan_finds_annotations_in_own_codebase() {
        let result = scan_directory(Path::new("..")).unwrap();
        assert!(!result.annotations.is_empty(), "Must find @req annotations in own codebase");
        let req_ids: Vec<_> = result
            .annotations
            .iter()
            .filter(|a| a.requirement_id == "SCS-SCAN-001")
            .collect();
        assert!(!req_ids.is_empty(), "Must find at least one @req SCS-SCAN-001");
    }

    /// @req SCS-SCAN-001
    /// @req SCS-TEST-001
    #[test]
    fn test_is_scan_target() {
        assert!(is_scan_target(Path::new("src/main.rs")));
        assert!(is_scan_target(Path::new("app.ts")));
        assert!(is_scan_target(Path::new("lib/helper.js")));
        assert!(is_scan_target(Path::new("script.py")));
        assert!(is_scan_target(Path::new("main.dart")));
        assert!(is_scan_target(Path::new("pkg/handler.go")));
        assert!(is_scan_target(Path::new("src")));
        assert!(!is_scan_target(Path::new("README.md")));
        assert!(!is_scan_target(Path::new("Cargo.toml")));
    }
}
