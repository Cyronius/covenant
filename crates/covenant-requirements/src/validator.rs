//! Validate requirement coverage

use std::collections::HashMap;
use covenant_ast::Priority;
use crate::{RequirementInfo, TestInfo, RequirementError, CoverageSummary, PrioritySummary, CoverageReport};
use crate::extractor::ExtractionResult;

/// Convert priority to ordinal for comparison (lower = higher priority)
pub fn priority_ord(p: Priority) -> u8 {
    match p {
        Priority::Critical => 0,
        Priority::High => 1,
        Priority::Medium => 2,
        Priority::Low => 3,
    }
}

/// Configuration for validation
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Minimum priority level that causes an error (Critical, High, Medium, Low)
    /// Uncovered requirements at this level or higher become errors
    pub error_min_priority: Priority,
    /// Minimum priority level that causes a warning
    /// Uncovered requirements at this level (below error threshold) become warnings
    pub warning_min_priority: Priority,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

impl ValidatorConfig {
    /// Default configuration: Critical=Error, High=Warning, others=Info
    pub fn default_config() -> Self {
        Self {
            error_min_priority: Priority::Critical,
            warning_min_priority: Priority::High,
        }
    }

    /// Strict configuration: All uncovered requirements are errors
    pub fn strict() -> Self {
        Self {
            error_min_priority: Priority::Low,
            warning_min_priority: Priority::Low,
        }
    }
}

/// Validate coverage and build the full report
pub fn validate(extraction: ExtractionResult, config: &ValidatorConfig) -> CoverageReport {
    let mut requirements = extraction.requirements;
    let tests = extraction.tests;
    let mut errors = extraction.errors;

    // Step 1: Build bidirectional coverage links
    build_coverage_links(&mut requirements, &tests, &mut errors);

    // Step 2: Check for uncovered requirements (respecting config thresholds)
    check_uncovered_requirements(&requirements, &mut errors, config);

    // Step 3: Compute summary statistics
    let summary = compute_summary(&requirements);

    CoverageReport {
        requirements,
        tests,
        summary,
        errors,
    }
}

/// Build bidirectional links between requirements and tests
fn build_coverage_links(
    requirements: &mut HashMap<String, RequirementInfo>,
    tests: &HashMap<String, TestInfo>,
    errors: &mut Vec<RequirementError>,
) {
    for (test_id, test) in tests {
        for req_id in &test.covers {
            if let Some(req) = requirements.get_mut(req_id) {
                // Add test to requirement's covered_by list
                if !req.covered_by.contains(test_id) {
                    req.covered_by.push(test_id.clone());
                }
            } else {
                // Test references nonexistent requirement
                errors.push(RequirementError::NonexistentRequirement {
                    test_id: test_id.clone(),
                    req_id: req_id.clone(),
                    snippet_id: test.snippet_id.clone(),
                    span: test.span,
                });
            }
        }
    }
}

/// Check for requirements without test coverage
///
/// Only reports uncovered requirements at or above the warning_min_priority threshold.
/// Requirements below the threshold are silently ignored.
fn check_uncovered_requirements(
    requirements: &HashMap<String, RequirementInfo>,
    errors: &mut Vec<RequirementError>,
    config: &ValidatorConfig,
) {
    let warning_threshold = priority_ord(config.warning_min_priority);

    for req in requirements.values() {
        if req.covered_by.is_empty() {
            // Only report if priority is at or above the warning threshold
            // (lower ordinal = higher priority)
            if priority_ord(req.priority) <= warning_threshold {
                errors.push(RequirementError::UncoveredRequirement {
                    id: req.id.clone(),
                    priority: req.priority,
                    snippet_id: req.snippet_id.clone(),
                    span: req.span,
                });
            }
        }
    }
}

/// Compute coverage summary statistics
fn compute_summary(requirements: &HashMap<String, RequirementInfo>) -> CoverageSummary {
    let total = requirements.len();
    let covered = requirements.values().filter(|r| !r.covered_by.is_empty()).count();
    let uncovered = total - covered;
    let coverage_percent = if total > 0 {
        (covered as f64 / total as f64) * 100.0
    } else {
        100.0 // No requirements = 100% coverage
    };

    // Compute per-priority statistics
    let mut by_priority: HashMap<String, PrioritySummary> = HashMap::new();

    for priority in [Priority::Critical, Priority::High, Priority::Medium, Priority::Low] {
        let priority_name = format!("{:?}", priority);
        let priority_reqs: Vec<_> = requirements.values()
            .filter(|r| r.priority == priority)
            .collect();
        let priority_total = priority_reqs.len();
        let priority_covered = priority_reqs.iter().filter(|r| !r.covered_by.is_empty()).count();

        by_priority.insert(priority_name, PrioritySummary {
            total: priority_total,
            covered: priority_covered,
            uncovered: priority_total - priority_covered,
        });
    }

    CoverageSummary {
        total_requirements: total,
        covered_requirements: covered,
        uncovered_requirements: uncovered,
        coverage_percent,
        by_priority,
    }
}

/// Filter report to only show uncovered requirements
pub fn filter_uncovered(report: &CoverageReport) -> CoverageReport {
    let uncovered_reqs: HashMap<_, _> = report.requirements.iter()
        .filter(|(_, r)| r.covered_by.is_empty())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    CoverageReport {
        requirements: uncovered_reqs,
        tests: HashMap::new(), // No tests for uncovered-only view
        summary: report.summary.clone(),
        errors: report.errors.iter()
            .filter(|e| matches!(e, RequirementError::UncoveredRequirement { .. }))
            .cloned()
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use covenant_ast::{Span, TestKind, ReqStatus};

    fn make_req(id: &str, priority: Priority) -> RequirementInfo {
        RequirementInfo {
            id: id.to_string(),
            text: Some(format!("Requirement {}", id)),
            priority,
            status: ReqStatus::Draft,
            snippet_id: "test.fn".to_string(),
            covered_by: Vec::new(),
            span: Span { start: 0, end: 0 },
        }
    }

    fn make_test(id: &str, covers: Vec<&str>) -> TestInfo {
        TestInfo {
            id: id.to_string(),
            kind: TestKind::Unit,
            covers: covers.into_iter().map(String::from).collect(),
            snippet_id: "test.fn".to_string(),
            span: Span { start: 0, end: 0 },
        }
    }

    #[test]
    fn test_coverage_links_single() {
        let mut requirements = HashMap::new();
        requirements.insert("R-001".to_string(), make_req("R-001", Priority::High));

        let mut tests = HashMap::new();
        tests.insert("T-001".to_string(), make_test("T-001", vec!["R-001"]));

        let mut errors = Vec::new();
        build_coverage_links(&mut requirements, &tests, &mut errors);

        assert!(errors.is_empty());
        assert_eq!(requirements.get("R-001").unwrap().covered_by, vec!["T-001"]);
    }

    #[test]
    fn test_coverage_links_multiple_tests() {
        let mut requirements = HashMap::new();
        requirements.insert("R-001".to_string(), make_req("R-001", Priority::High));

        let mut tests = HashMap::new();
        tests.insert("T-001".to_string(), make_test("T-001", vec!["R-001"]));
        tests.insert("T-002".to_string(), make_test("T-002", vec!["R-001"]));

        let mut errors = Vec::new();
        build_coverage_links(&mut requirements, &tests, &mut errors);

        assert!(errors.is_empty());
        let covered = &requirements.get("R-001").unwrap().covered_by;
        assert_eq!(covered.len(), 2);
        assert!(covered.contains(&"T-001".to_string()));
        assert!(covered.contains(&"T-002".to_string()));
    }

    #[test]
    fn test_nonexistent_requirement_reference() {
        let mut requirements = HashMap::new();
        // No R-001 requirement

        let mut tests = HashMap::new();
        tests.insert("T-001".to_string(), make_test("T-001", vec!["R-001"]));

        let mut errors = Vec::new();
        build_coverage_links(&mut requirements, &tests, &mut errors);

        assert_eq!(errors.len(), 1);
        assert!(matches!(&errors[0],
            RequirementError::NonexistentRequirement { test_id, req_id, .. }
            if test_id == "T-001" && req_id == "R-001"
        ));
    }

    #[test]
    fn test_summary_all_covered() {
        let mut requirements = HashMap::new();
        let mut req = make_req("R-001", Priority::High);
        req.covered_by = vec!["T-001".to_string()];
        requirements.insert("R-001".to_string(), req);

        let summary = compute_summary(&requirements);

        assert_eq!(summary.total_requirements, 1);
        assert_eq!(summary.covered_requirements, 1);
        assert_eq!(summary.uncovered_requirements, 0);
        assert_eq!(summary.coverage_percent, 100.0);
    }

    #[test]
    fn test_summary_none_covered() {
        let mut requirements = HashMap::new();
        requirements.insert("R-001".to_string(), make_req("R-001", Priority::High));
        requirements.insert("R-002".to_string(), make_req("R-002", Priority::Low));

        let summary = compute_summary(&requirements);

        assert_eq!(summary.total_requirements, 2);
        assert_eq!(summary.covered_requirements, 0);
        assert_eq!(summary.uncovered_requirements, 2);
        assert_eq!(summary.coverage_percent, 0.0);
    }

    #[test]
    fn test_summary_partial_coverage() {
        let mut requirements = HashMap::new();
        let mut req1 = make_req("R-001", Priority::High);
        req1.covered_by = vec!["T-001".to_string()];
        requirements.insert("R-001".to_string(), req1);
        requirements.insert("R-002".to_string(), make_req("R-002", Priority::Low));

        let summary = compute_summary(&requirements);

        assert_eq!(summary.total_requirements, 2);
        assert_eq!(summary.covered_requirements, 1);
        assert_eq!(summary.uncovered_requirements, 1);
        assert_eq!(summary.coverage_percent, 50.0);
    }

    #[test]
    fn test_summary_by_priority() {
        let mut requirements = HashMap::new();
        requirements.insert("R-001".to_string(), make_req("R-001", Priority::Critical));
        requirements.insert("R-002".to_string(), make_req("R-002", Priority::High));
        requirements.insert("R-003".to_string(), make_req("R-003", Priority::High));

        let summary = compute_summary(&requirements);

        assert_eq!(summary.by_priority.get("Critical").unwrap().total, 1);
        assert_eq!(summary.by_priority.get("High").unwrap().total, 2);
        assert_eq!(summary.by_priority.get("Medium").unwrap().total, 0);
        assert_eq!(summary.by_priority.get("Low").unwrap().total, 0);
    }

    #[test]
    fn test_summary_empty() {
        let requirements = HashMap::new();
        let summary = compute_summary(&requirements);

        assert_eq!(summary.total_requirements, 0);
        assert_eq!(summary.coverage_percent, 100.0); // No requirements = 100% covered
    }

    #[test]
    fn test_filter_uncovered() {
        let mut requirements = HashMap::new();
        let mut req1 = make_req("R-001", Priority::High);
        req1.covered_by = vec!["T-001".to_string()];
        requirements.insert("R-001".to_string(), req1);
        requirements.insert("R-002".to_string(), make_req("R-002", Priority::Low));

        let report = CoverageReport {
            requirements,
            tests: HashMap::new(),
            summary: CoverageSummary {
                total_requirements: 2,
                covered_requirements: 1,
                uncovered_requirements: 1,
                coverage_percent: 50.0,
                by_priority: HashMap::new(),
            },
            errors: vec![
                RequirementError::UncoveredRequirement {
                    id: "R-002".to_string(),
                    priority: Priority::Low,
                    snippet_id: "test.fn".to_string(),
                    span: Span { start: 0, end: 0 },
                },
            ],
        };

        let filtered = filter_uncovered(&report);

        assert_eq!(filtered.requirements.len(), 1);
        assert!(filtered.requirements.contains_key("R-002"));
        assert!(!filtered.requirements.contains_key("R-001"));
    }
}
