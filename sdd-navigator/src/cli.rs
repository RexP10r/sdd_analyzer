use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// @req SCS-CLI-001
#[derive(Parser)]
#[command(name = "sdd-coverage", about = "SDD Navigator — traceability scanner")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// @req SCS-CLI-001
#[derive(Subcommand)]
pub enum Commands {
    /// Scan the project for @req annotations and compute coverage
    Scan {
        /// Path to requirements.yaml
        #[arg(long, default_value = "requirements.yaml")]
        requirements: PathBuf,

        /// Source directory to scan
        #[arg(long, default_value = ".")]
        source: PathBuf,

        /// Show test annotations in output
        #[arg(long)]
        tests: bool,

        /// Exit with code 1 on any traceability violation
        #[arg(long)]
        strict: bool,
    },
}
