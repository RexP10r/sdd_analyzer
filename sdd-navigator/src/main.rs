mod cli;

use clap::Parser;

/// @req SCS-CLI-001
/// @req SCS-API-001
/// @req SCS-API-002
#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        Some(cli::Commands::Scan {
            requirements,
            source,
            tests,
            strict,
        }) => {
            run_scan(requirements, source, tests, strict);
        }
        None => {
            let state = sdd_server::state::new_app_state();
            if let Err(e) = sdd_server::server::run(state).await {
                eprintln!("Server error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// @req SCS-CLI-001
/// @req SCS-HOST-001
fn run_scan(
    requirements_path: std::path::PathBuf,
    source_dir: std::path::PathBuf,
    show_tests: bool,
    strict: bool,
) {
    use std::collections::HashSet;

    let reqs = match sdd_engine::parser::parse_requirements(&requirements_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error parsing requirements: {}", e);
            std::process::exit(1);
        }
    };

    let scan_result = match sdd_engine::scanner::scan_directory(&source_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error scanning: {}", e);
            std::process::exit(1);
        }
    };

    let req_ids: HashSet<String> = sdd_core::models::build_req_ids(&reqs);
    let mut violations = 0u32;

    println!("=== SDD Coverage Report ===\n");
    println!("Source: {}", source_dir.display());
    println!("Requirements file: {}", requirements_path.display());
    println!();

    // Per-requirement coverage
    for req in &reqs {
        let status =
            sdd_engine::coverage::compute_requirement_status(&req.id, &scan_result.annotations);
        let impl_count = scan_result
            .annotations
            .iter()
            .filter(|a| {
                a.requirement_id == req.id
                    && a.classification == sdd_core::models::Classification::Impl
            })
            .count();
        let test_count = scan_result
            .annotations
            .iter()
            .filter(|a| {
                a.requirement_id == req.id
                    && a.classification == sdd_core::models::Classification::Test
            })
            .count();

        let marker = match status {
            sdd_core::models::CoverageStatus::Missing => {
                violations += 1;
                "MISS"
            }
            sdd_core::models::CoverageStatus::Partial => {
                violations += 1;
                "PART"
            }
            sdd_core::models::CoverageStatus::Covered => " OK ",
        };

        println!(
            "[{}] {}  (impl: {}, test: {})  {}",
            marker, req.id, impl_count, test_count, req.title
        );
    }

    // Orphan annotations
    let orphan_anns =
        sdd_engine::coverage::find_orphan_annotations(&scan_result.annotations, &req_ids);
    if !orphan_anns.is_empty() {
        violations += orphan_anns.len() as u32;
        println!("\n--- Orphan Annotations ---");
        for a in &orphan_anns {
            println!("  {}:{}  {}", a.file, a.line, a.req_id);
        }
    }

    // Orphan tasks
    let tasks = sdd_engine::parser::parse_tasks(
        &requirements_path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("tasks.yaml"),
    )
    .unwrap_or_default();
    let orphan_tasks = sdd_engine::coverage::find_orphan_tasks(&tasks, &req_ids);
    if !orphan_tasks.is_empty() {
        violations += orphan_tasks.len() as u32;
        println!("\n--- Orphan Tasks ---");
        for t in &orphan_tasks {
            println!("  {}  → {}  [{}]", t.id, t.requirement_id, t.status);
        }
    }

    // Show test annotations if requested
    if show_tests {
        println!("\n--- Test Annotations ---");
        for a in &scan_result.annotations {
            if a.classification == sdd_core::models::Classification::Test {
                println!("  {}:{}  {}", a.file_path, a.line_number, a.requirement_id);
            }
        }
    }

    // Summary
    let stats =
        sdd_engine::coverage::compute_project_stats(&reqs, &scan_result.annotations, &tasks);
    println!();
    println!("=== Summary ===");
    println!(
        "Requirements: {} (covered: {}, partial: {}, missing: {})",
        stats.total_requirements, stats.covered, stats.partial, stats.missing
    );
    println!("Coverage: {:.1}%", stats.coverage_pct);
    println!(
        "Annotations: {} (impl: {}, test: {})",
        stats.total_annotations, stats.impl_count, stats.test_count
    );
    println!(
        "Orphans: {} annotations, {} tasks",
        stats.orphan_annotations, stats.orphan_tasks
    );

    if !scan_result.warnings.is_empty() {
        println!("\n=== Warnings ===");
        for w in &scan_result.warnings {
            println!("  {}", w);
        }
    }

    if strict && violations > 0 {
        eprintln!(
            "\nFAIL: {} traceability violation(s) found (--strict mode)",
            violations
        );
        std::process::exit(1);
    }
}
