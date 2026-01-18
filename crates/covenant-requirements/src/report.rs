//! Generate coverage reports in various formats

use covenant_ast::Priority;
use crate::{CoverageReport, RequirementError, Severity};

/// Output format for reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReportFormat {
    #[default]
    Text,
    Json,
    Markdown,
}

impl std::str::FromStr for ReportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" | "txt" => Ok(ReportFormat::Text),
            "json" => Ok(ReportFormat::Json),
            "markdown" | "md" => Ok(ReportFormat::Markdown),
            _ => Err(format!("Unknown format: {}. Expected: text, json, or markdown", s)),
        }
    }
}

/// Format a coverage report
pub fn format_report(report: &CoverageReport, format: ReportFormat) -> String {
    match format {
        ReportFormat::Text => format_text(report),
        ReportFormat::Json => format_json(report),
        ReportFormat::Markdown => format_markdown(report),
    }
}

fn format_text(report: &CoverageReport) -> String {
    let mut output = String::new();

    // Summary
    output.push_str("=== Requirement Coverage Report ===\n\n");
    output.push_str(&format!("Total Requirements: {}\n", report.summary.total_requirements));
    output.push_str(&format!("Covered: {}\n", report.summary.covered_requirements));
    output.push_str(&format!("Uncovered: {}\n", report.summary.uncovered_requirements));
    output.push_str(&format!("Coverage: {:.1}%\n\n", report.summary.coverage_percent));

    // Priority breakdown
    output.push_str("By Priority:\n");
    for priority in ["Critical", "High", "Medium", "Low"] {
        if let Some(stats) = report.summary.by_priority.get(priority) {
            if stats.total > 0 {
                output.push_str(&format!(
                    "  {}: {}/{} covered\n",
                    priority, stats.covered, stats.total
                ));
            }
        }
    }
    output.push_str("\n");

    // Covered requirements
    let covered: Vec<_> = report.requirements.values()
        .filter(|r| !r.covered_by.is_empty())
        .collect();

    if !covered.is_empty() {
        output.push_str("Covered Requirements:\n");
        for req in &covered {
            output.push_str(&format!(
                "  [+] {} - {} (by: {})\n",
                req.id,
                req.text.as_deref().unwrap_or("(no description)"),
                req.covered_by.join(", ")
            ));
        }
        output.push_str("\n");
    }

    // Uncovered requirements
    let mut uncovered: Vec<_> = report.requirements.values()
        .filter(|r| r.covered_by.is_empty())
        .collect();

    // Sort by priority (Critical first)
    uncovered.sort_by_key(|r| match r.priority {
        Priority::Critical => 0,
        Priority::High => 1,
        Priority::Medium => 2,
        Priority::Low => 3,
    });

    if !uncovered.is_empty() {
        output.push_str("Uncovered Requirements:\n");
        for req in uncovered {
            let priority_marker = match req.priority {
                Priority::Critical => "[CRITICAL]",
                Priority::High => "[HIGH]",
                Priority::Medium => "[MEDIUM]",
                Priority::Low => "[LOW]",
            };
            output.push_str(&format!(
                "  [-] {} {} - {} (in {})\n",
                priority_marker,
                req.id,
                req.text.as_deref().unwrap_or("(no description)"),
                req.snippet_id
            ));
        }
        output.push_str("\n");
    }

    // Errors (non-coverage errors only)
    let other_errors: Vec<_> = report.errors.iter()
        .filter(|e| !matches!(e, RequirementError::UncoveredRequirement { .. }))
        .collect();

    if !other_errors.is_empty() {
        output.push_str("Validation Errors:\n");
        for error in other_errors {
            let severity_marker = match error.severity() {
                Severity::Error => "ERROR",
                Severity::Warning => "WARN",
                Severity::Info => "INFO",
            };
            output.push_str(&format!("  [{}] {}: {}\n", severity_marker, error.code(), error));
        }
    }

    output
}

fn format_json(report: &CoverageReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

fn format_markdown(report: &CoverageReport) -> String {
    let mut output = String::new();

    output.push_str("# Requirement Coverage Report\n\n");

    // Summary table
    output.push_str("## Summary\n\n");
    output.push_str("| Metric | Value |\n");
    output.push_str("|--------|-------|\n");
    output.push_str(&format!("| Total Requirements | {} |\n", report.summary.total_requirements));
    output.push_str(&format!("| Covered | {} |\n", report.summary.covered_requirements));
    output.push_str(&format!("| Uncovered | {} |\n", report.summary.uncovered_requirements));
    output.push_str(&format!("| Coverage | {:.1}% |\n\n", report.summary.coverage_percent));

    // Priority breakdown table
    output.push_str("## By Priority\n\n");
    output.push_str("| Priority | Covered | Total | % |\n");
    output.push_str("|----------|---------|-------|---|\n");
    for priority in ["Critical", "High", "Medium", "Low"] {
        if let Some(stats) = report.summary.by_priority.get(priority) {
            let pct = if stats.total > 0 {
                (stats.covered as f64 / stats.total as f64) * 100.0
            } else {
                100.0
            };
            output.push_str(&format!(
                "| {} | {} | {} | {:.1}% |\n",
                priority, stats.covered, stats.total, pct
            ));
        }
    }
    output.push_str("\n");

    // Covered requirements list
    let covered: Vec<_> = report.requirements.values()
        .filter(|r| !r.covered_by.is_empty())
        .collect();

    if !covered.is_empty() {
        output.push_str("## Covered Requirements\n\n");
        for req in covered {
            output.push_str(&format!(
                "- **{}** ({:?}): {}\n  - Covered by: `{}`\n  - Snippet: `{}`\n",
                req.id,
                req.priority,
                req.text.as_deref().unwrap_or("(no description)"),
                req.covered_by.join("`, `"),
                req.snippet_id
            ));
        }
        output.push_str("\n");
    }

    // Uncovered requirements list
    let mut uncovered: Vec<_> = report.requirements.values()
        .filter(|r| r.covered_by.is_empty())
        .collect();

    uncovered.sort_by_key(|r| match r.priority {
        Priority::Critical => 0,
        Priority::High => 1,
        Priority::Medium => 2,
        Priority::Low => 3,
    });

    if !uncovered.is_empty() {
        output.push_str("## Uncovered Requirements\n\n");
        for req in uncovered {
            let priority_badge = match req.priority {
                Priority::Critical => "**CRITICAL**",
                Priority::High => "HIGH",
                Priority::Medium => "Medium",
                Priority::Low => "Low",
            };
            output.push_str(&format!(
                "- **{}** [{}]: {}\n  - Snippet: `{}`\n",
                req.id,
                priority_badge,
                req.text.as_deref().unwrap_or("(no description)"),
                req.snippet_id
            ));
        }
        output.push_str("\n");
    }

    // Errors
    let other_errors: Vec<_> = report.errors.iter()
        .filter(|e| !matches!(e, RequirementError::UncoveredRequirement { .. }))
        .collect();

    if !other_errors.is_empty() {
        output.push_str("## Validation Errors\n\n");
        for error in other_errors {
            let emoji = match error.severity() {
                Severity::Error => "x",
                Severity::Warning => "!",
                Severity::Info => "i",
            };
            output.push_str(&format!("- [{}] `{}`: {}\n", emoji, error.code(), error));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use covenant_ast::{Span, TestKind, ReqStatus};
    use crate::{RequirementInfo, TestInfo, CoverageSummary, PrioritySummary};

    fn make_test_report() -> CoverageReport {
        let mut requirements = HashMap::new();

        // Covered requirement
        requirements.insert("R-001".to_string(), RequirementInfo {
            id: "R-001".to_string(),
            text: Some("Covered requirement".to_string()),
            priority: Priority::High,
            status: ReqStatus::Draft,
            snippet_id: "test.fn".to_string(),
            covered_by: vec!["T-001".to_string()],
            span: Span { start: 0, end: 0 },
        });

        // Uncovered requirement
        requirements.insert("R-002".to_string(), RequirementInfo {
            id: "R-002".to_string(),
            text: Some("Uncovered requirement".to_string()),
            priority: Priority::Critical,
            status: ReqStatus::Draft,
            snippet_id: "test.fn".to_string(),
            covered_by: Vec::new(),
            span: Span { start: 0, end: 0 },
        });

        let mut tests = HashMap::new();
        tests.insert("T-001".to_string(), TestInfo {
            id: "T-001".to_string(),
            kind: TestKind::Unit,
            covers: vec!["R-001".to_string()],
            snippet_id: "test.fn".to_string(),
            span: Span { start: 0, end: 0 },
        });

        let mut by_priority = HashMap::new();
        by_priority.insert("Critical".to_string(), PrioritySummary { total: 1, covered: 0, uncovered: 1 });
        by_priority.insert("High".to_string(), PrioritySummary { total: 1, covered: 1, uncovered: 0 });
        by_priority.insert("Medium".to_string(), PrioritySummary { total: 0, covered: 0, uncovered: 0 });
        by_priority.insert("Low".to_string(), PrioritySummary { total: 0, covered: 0, uncovered: 0 });

        CoverageReport {
            requirements,
            tests,
            summary: CoverageSummary {
                total_requirements: 2,
                covered_requirements: 1,
                uncovered_requirements: 1,
                coverage_percent: 50.0,
                by_priority,
            },
            errors: vec![
                RequirementError::UncoveredRequirement {
                    id: "R-002".to_string(),
                    priority: Priority::Critical,
                    snippet_id: "test.fn".to_string(),
                    span: Span { start: 0, end: 0 },
                },
            ],
        }
    }

    #[test]
    fn test_format_text() {
        let report = make_test_report();
        let output = format_text(&report);

        assert!(output.contains("=== Requirement Coverage Report ==="));
        assert!(output.contains("Total Requirements: 2"));
        assert!(output.contains("Covered: 1"));
        assert!(output.contains("Uncovered: 1"));
        assert!(output.contains("Coverage: 50.0%"));
        assert!(output.contains("[CRITICAL] R-002"));
        assert!(output.contains("R-001"));
    }

    #[test]
    fn test_format_json() {
        let report = make_test_report();
        let output = format_json(&report);

        assert!(output.contains("\"total_requirements\": 2"));
        assert!(output.contains("\"coverage_percent\": 50.0"));
        assert!(output.contains("\"R-001\""));
        assert!(output.contains("\"R-002\""));
    }

    #[test]
    fn test_format_markdown() {
        let report = make_test_report();
        let output = format_markdown(&report);

        assert!(output.contains("# Requirement Coverage Report"));
        assert!(output.contains("| Total Requirements | 2 |"));
        assert!(output.contains("| Coverage | 50.0% |"));
        assert!(output.contains("## Uncovered Requirements"));
        assert!(output.contains("**R-002**"));
    }

    #[test]
    fn test_format_parsing() {
        assert_eq!("text".parse::<ReportFormat>().unwrap(), ReportFormat::Text);
        assert_eq!("txt".parse::<ReportFormat>().unwrap(), ReportFormat::Text);
        assert_eq!("json".parse::<ReportFormat>().unwrap(), ReportFormat::Json);
        assert_eq!("markdown".parse::<ReportFormat>().unwrap(), ReportFormat::Markdown);
        assert_eq!("md".parse::<ReportFormat>().unwrap(), ReportFormat::Markdown);
        assert!("invalid".parse::<ReportFormat>().is_err());
    }
}
