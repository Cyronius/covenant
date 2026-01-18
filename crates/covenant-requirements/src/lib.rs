//! Covenant Requirement Validator
//!
//! Validates that all requirements have test coverage and generates coverage reports.

mod extractor;
mod validator;
mod report;

pub use extractor::extract;
pub use validator::{validate, ValidatorConfig, filter_uncovered, priority_ord};
pub use report::{format_report, ReportFormat};

use std::collections::HashMap;
use covenant_ast::{Priority, ReqStatus, TestKind, Span};
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Information about a requirement extracted from the AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementInfo {
    /// Requirement ID (e.g., "R-001", "R-AUTH-001")
    pub id: String,
    /// Human-readable description
    pub text: Option<String>,
    /// Priority level (Critical, High, Medium, Low)
    pub priority: Priority,
    /// Current status (Draft, Approved, Implemented, Tested)
    pub status: ReqStatus,
    /// Parent snippet ID where this requirement is defined
    pub snippet_id: String,
    /// Test IDs that cover this requirement (computed bidirectionally)
    pub covered_by: Vec<String>,
    /// Source span for error reporting
    pub span: Span,
}

/// Information about a test extracted from the AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInfo {
    /// Test ID (e.g., "T-001", "T-AUTH-001")
    pub id: String,
    /// Test kind (Unit, Integration, Golden, Property)
    pub kind: TestKind,
    /// Requirement IDs this test claims to cover
    pub covers: Vec<String>,
    /// Parent snippet ID where this test is defined
    pub snippet_id: String,
    /// Source span for error reporting
    pub span: Span,
}

/// Summary statistics for coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub total_requirements: usize,
    pub covered_requirements: usize,
    pub uncovered_requirements: usize,
    pub coverage_percent: f64,
    /// Breakdown by priority
    pub by_priority: HashMap<String, PrioritySummary>,
}

/// Per-priority summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritySummary {
    pub total: usize,
    pub covered: usize,
    pub uncovered: usize,
}

/// Full coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    /// All requirements indexed by ID
    pub requirements: HashMap<String, RequirementInfo>,
    /// All tests indexed by ID
    pub tests: HashMap<String, TestInfo>,
    /// Computed summary statistics
    pub summary: CoverageSummary,
    /// Validation errors encountered
    pub errors: Vec<RequirementError>,
}

/// Requirement validation errors
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum RequirementError {
    /// E-REQ-001: Requirement has no test coverage
    #[error("uncovered requirement '{id}' (priority: {priority:?}) in snippet '{snippet_id}'")]
    UncoveredRequirement {
        id: String,
        priority: Priority,
        snippet_id: String,
        span: Span,
    },

    /// E-REQ-002: Test references a nonexistent requirement
    #[error("test '{test_id}' references nonexistent requirement '{req_id}' in snippet '{snippet_id}'")]
    NonexistentRequirement {
        test_id: String,
        req_id: String,
        snippet_id: String,
        span: Span,
    },

    /// E-REQ-003: Duplicate requirement ID
    #[error("duplicate requirement ID '{id}' found in snippets '{first}' and '{second}'")]
    DuplicateRequirement {
        id: String,
        first: String,
        second: String,
        span: Span,
    },

    /// E-REQ-004: Duplicate test ID
    #[error("duplicate test ID '{id}' found in snippets '{first}' and '{second}'")]
    DuplicateTest {
        id: String,
        first: String,
        second: String,
        span: Span,
    },
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl RequirementError {
    /// Get the error code for display
    pub fn code(&self) -> &'static str {
        match self {
            RequirementError::UncoveredRequirement { .. } => "E-REQ-001",
            RequirementError::NonexistentRequirement { .. } => "E-REQ-002",
            RequirementError::DuplicateRequirement { .. } => "E-REQ-003",
            RequirementError::DuplicateTest { .. } => "E-REQ-004",
        }
    }

    /// Get severity based on error type and priority
    pub fn severity(&self) -> Severity {
        match self {
            RequirementError::UncoveredRequirement { priority, .. } => {
                match priority {
                    Priority::Critical => Severity::Error,
                    Priority::High => Severity::Warning,
                    _ => Severity::Info,
                }
            }
            RequirementError::NonexistentRequirement { .. } => Severity::Error,
            RequirementError::DuplicateRequirement { .. } => Severity::Error,
            RequirementError::DuplicateTest { .. } => Severity::Error,
        }
    }

    /// Get the source span for this error
    pub fn span(&self) -> Span {
        match self {
            RequirementError::UncoveredRequirement { span, .. } => *span,
            RequirementError::NonexistentRequirement { span, .. } => *span,
            RequirementError::DuplicateRequirement { span, .. } => *span,
            RequirementError::DuplicateTest { span, .. } => *span,
        }
    }

    /// Get severity based on error type, priority, and config thresholds
    pub fn severity_with_config(&self, config: &ValidatorConfig) -> Severity {
        match self {
            RequirementError::UncoveredRequirement { priority, .. } => {
                let prio = priority_ord(*priority);
                let error_threshold = priority_ord(config.error_min_priority);
                let warning_threshold = priority_ord(config.warning_min_priority);

                if prio <= error_threshold {
                    Severity::Error
                } else if prio <= warning_threshold {
                    Severity::Warning
                } else {
                    Severity::Info
                }
            }
            // Other errors are always Error severity
            RequirementError::NonexistentRequirement { .. } => Severity::Error,
            RequirementError::DuplicateRequirement { .. } => Severity::Error,
            RequirementError::DuplicateTest { .. } => Severity::Error,
        }
    }
}

/// Validate requirements coverage for a program
///
/// This is the main entry point for the requirement validator.
///
/// # Example
/// ```ignore
/// use covenant_requirements::{validate_program, ReportFormat, format_report};
/// use covenant_parser::parse;
///
/// let source = "...";
/// let program = parse(source).unwrap();
/// let report = validate_program(&program, None);
/// println!("{}", format_report(&report, ReportFormat::Text));
/// ```
pub fn validate_program(program: &covenant_ast::Program, config: Option<ValidatorConfig>) -> CoverageReport {
    let config = config.unwrap_or_else(ValidatorConfig::default_config);
    let extraction = extract(program);
    validate(extraction, &config)
}

/// Check if a program has any coverage errors (for CI integration)
pub fn has_coverage_errors(report: &CoverageReport) -> bool {
    report.errors.iter().any(|e| e.severity() == Severity::Error)
}

/// Get only the errors that are actual failures (Error severity)
pub fn get_failures(report: &CoverageReport) -> Vec<&RequirementError> {
    report.errors.iter().filter(|e| e.severity() == Severity::Error).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use covenant_parser::parse;

    #[test]
    fn test_extract_single_requirement() {
        let source = r#"
snippet id="test.fn" kind="fn"

requires
  req id="R-001"
    text "Test requirement"
    priority high
  end
end

signature
  fn name="test_fn"
    returns type="Unit"
  end
end

body
end

end
"#;
        let program = parse(source).unwrap();
        let report = validate_program(&program, None);
        assert_eq!(report.requirements.len(), 1);
        assert!(report.requirements.contains_key("R-001"));
    }

    #[test]
    fn test_covered_requirement() {
        let source = r#"
snippet id="test.fn" kind="fn"

requires
  req id="R-001"
    text "Test requirement"
  end
end

signature
  fn name="test_fn"
    returns type="Unit"
  end
end

body
end

tests
  test id="T-001" kind="unit" covers="R-001"
  end
end

end
"#;
        let program = parse(source).unwrap();
        let report = validate_program(&program, None);
        assert_eq!(report.summary.covered_requirements, 1);
        assert_eq!(report.summary.uncovered_requirements, 0);
        assert!(report.requirements.get("R-001").unwrap().covered_by.contains(&"T-001".to_string()));
    }

    #[test]
    fn test_uncovered_critical_requirement() {
        let source = r#"
snippet id="test.fn" kind="fn"

requires
  req id="R-001"
    text "Critical requirement"
    priority critical
  end
end

signature
  fn name="test_fn"
    returns type="Unit"
  end
end

body
end

end
"#;
        let program = parse(source).unwrap();
        let report = validate_program(&program, None);
        assert_eq!(report.summary.uncovered_requirements, 1);
        assert!(has_coverage_errors(&report));
    }

    #[test]
    fn test_nonexistent_requirement_reference() {
        let source = r#"
snippet id="test.fn" kind="fn"

signature
  fn name="test_fn"
    returns type="Unit"
  end
end

body
end

tests
  test id="T-001" kind="unit" covers="R-NONEXISTENT"
  end
end

end
"#;
        let program = parse(source).unwrap();
        let report = validate_program(&program, None);
        assert!(report.errors.iter().any(|e| matches!(e,
            RequirementError::NonexistentRequirement { req_id, .. }
            if req_id == "R-NONEXISTENT"
        )));
    }

    #[test]
    fn test_json_output() {
        let source = r#"
snippet id="test.fn" kind="fn"

requires
  req id="R-001"
    text "Test"
    priority high
  end
end

signature
  fn name="test_fn"
    returns type="Unit"
  end
end

body
end

end
"#;
        let program = parse(source).unwrap();
        let report = validate_program(&program, None);
        let json = format_report(&report, ReportFormat::Json);
        assert!(json.contains("R-001"));
        assert!(json.contains("coverage_percent"));
    }
}
