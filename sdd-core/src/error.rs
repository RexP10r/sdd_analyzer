use miette::Diagnostic;
use thiserror::Error;

/// @req SCS-PARSE-001
/// @req SCS-PARSE-002
/// @req SCS-ERR-001
#[derive(Error, Debug, Diagnostic)]
pub enum AppError {
    #[error("Failed to read file '{path}': {source}")]
    #[diagnostic(code(sdd::io_error), help("Verify the file exists and is readable"))]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse YAML in '{path}' at line {line}: {message}")]
    #[diagnostic(code(sdd::yaml_parse), help("Check YAML syntax at the indicated line"))]
    YamlParse {
        path: String,
        line: usize,
        message: String,
    },

    #[error("Missing required field '{field}' in entry at index {index} in '{path}'")]
    #[diagnostic(code(sdd::missing_field), help("Add the missing field to the YAML entry"))]
    MissingField {
        path: String,
        index: usize,
        field: String,
    },

    #[error("Missing root key '{key}' in '{path}'. Expected a top-level '{key}' list")]
    #[diagnostic(code(sdd::missing_root_key), help("Add a top-level '{key}' key containing a list"))]
    MissingRootKey {
        path: String,
        key: String,
    },

    #[error("Empty '{key}' list in '{path}'")]
    #[diagnostic(code(sdd::empty_list), help("Add at least one entry to the '{key}' list"))]
    EmptyList {
        path: String,
        key: String,
    },

    #[error("Directory traversal error at '{path}': {message}")]
    #[diagnostic(code(sdd::scanner), help("Verify the directory exists and is accessible"))]
    Scan {
        path: String,
        message: String,
    },

    #[error("Regex compilation error: {message}")]
    #[diagnostic(code(sdd::regex), help("This is an internal error — verify the regex pattern"))]
    Regex {
        message: String,
    },
}
