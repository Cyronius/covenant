//! Extract requirements and tests from Covenant AST

use std::collections::HashMap;
use covenant_ast::{Program, Snippet, Section, Priority, ReqStatus};
use crate::{RequirementInfo, TestInfo, RequirementError};

/// Result of extraction phase
pub struct ExtractionResult {
    pub requirements: HashMap<String, RequirementInfo>,
    pub tests: HashMap<String, TestInfo>,
    pub errors: Vec<RequirementError>,
}

/// Extract all requirements and tests from a program
pub fn extract(program: &Program) -> ExtractionResult {
    let mut requirements = HashMap::new();
    let mut tests = HashMap::new();
    let mut errors = Vec::new();

    match program {
        Program::Snippets { snippets, .. } => {
            for snippet in snippets {
                extract_from_snippet(snippet, &mut requirements, &mut tests, &mut errors);
            }
        }
        Program::Legacy { .. } => {
            // Legacy programs don't have requirements/tests sections
        }
    }

    ExtractionResult { requirements, tests, errors }
}

fn extract_from_snippet(
    snippet: &Snippet,
    requirements: &mut HashMap<String, RequirementInfo>,
    tests: &mut HashMap<String, TestInfo>,
    errors: &mut Vec<RequirementError>,
) {
    for section in &snippet.sections {
        match section {
            Section::Requires(req_section) => {
                for req in &req_section.requirements {
                    let info = RequirementInfo {
                        id: req.id.clone(),
                        text: req.text.clone(),
                        priority: req.priority.unwrap_or(Priority::Medium),
                        status: req.status.unwrap_or(ReqStatus::Draft),
                        snippet_id: snippet.id.clone(),
                        covered_by: Vec::new(), // Computed in validation phase
                        span: req.span,
                    };

                    if let Some(existing) = requirements.get(&req.id) {
                        errors.push(RequirementError::DuplicateRequirement {
                            id: req.id.clone(),
                            first: existing.snippet_id.clone(),
                            second: snippet.id.clone(),
                            span: req.span,
                        });
                    } else {
                        requirements.insert(req.id.clone(), info);
                    }
                }
            }
            Section::Tests(test_section) => {
                for test in &test_section.tests {
                    let info = TestInfo {
                        id: test.id.clone(),
                        kind: test.kind,
                        covers: test.covers.clone(),
                        snippet_id: snippet.id.clone(),
                        span: test.span,
                    };

                    if let Some(existing) = tests.get(&test.id) {
                        errors.push(RequirementError::DuplicateTest {
                            id: test.id.clone(),
                            first: existing.snippet_id.clone(),
                            second: snippet.id.clone(),
                            span: test.span,
                        });
                    } else {
                        tests.insert(test.id.clone(), info);
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use covenant_parser::parse;

    #[test]
    fn test_extract_empty_snippets_program() {
        let source = "";
        // Empty source should parse as empty program
        let result = parse(source);
        // Parser may fail on empty, which is fine
        if let Ok(program) = result {
            let extraction = extract(&program);
            assert!(extraction.requirements.is_empty());
            assert!(extraction.tests.is_empty());
        }
    }

    #[test]
    fn test_extract_requirements_only() {
        let source = r#"
snippet id="test.fn" kind="fn"

requires
  req id="R-001"
    text "First requirement"
    priority high
  end
  req id="R-002"
    text "Second requirement"
    priority low
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
        let extraction = extract(&program);

        assert_eq!(extraction.requirements.len(), 2);
        assert!(extraction.requirements.contains_key("R-001"));
        assert!(extraction.requirements.contains_key("R-002"));
        assert!(extraction.tests.is_empty());
        assert!(extraction.errors.is_empty());

        let r1 = extraction.requirements.get("R-001").unwrap();
        assert_eq!(r1.priority, Priority::High);
        assert_eq!(r1.text, Some("First requirement".to_string()));
    }

    #[test]
    fn test_extract_tests_only() {
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
  test id="T-001" kind="unit" covers="R-001"
  end
  test id="T-002" kind="integration"
  end
end

end
"#;
        let program = parse(source).unwrap();
        let extraction = extract(&program);

        assert!(extraction.requirements.is_empty());
        assert_eq!(extraction.tests.len(), 2);
        assert!(extraction.tests.contains_key("T-001"));
        assert!(extraction.tests.contains_key("T-002"));

        let t1 = extraction.tests.get("T-001").unwrap();
        assert_eq!(t1.covers, vec!["R-001".to_string()]);
    }

    #[test]
    fn test_extract_from_multiple_snippets() {
        let source = r#"
snippet id="a.fn" kind="fn"

requires
  req id="R-A-001"
    text "Requirement A"
  end
end

signature
  fn name="fn_a"
    returns type="Unit"
  end
end

body
end

end

snippet id="b.fn" kind="fn"

requires
  req id="R-B-001"
    text "Requirement B"
  end
end

signature
  fn name="fn_b"
    returns type="Unit"
  end
end

body
end

tests
  test id="T-B-001" kind="unit" covers="R-B-001"
  end
end

end
"#;
        let program = parse(source).unwrap();
        let extraction = extract(&program);

        assert_eq!(extraction.requirements.len(), 2);
        assert_eq!(extraction.tests.len(), 1);
        assert!(extraction.requirements.contains_key("R-A-001"));
        assert!(extraction.requirements.contains_key("R-B-001"));
        assert!(extraction.tests.contains_key("T-B-001"));
    }

    #[test]
    fn test_duplicate_requirement_detection() {
        let source = r#"
snippet id="a.fn" kind="fn"

requires
  req id="R-001"
    text "First"
  end
end

signature
  fn name="fn_a"
    returns type="Unit"
  end
end

body
end

end

snippet id="b.fn" kind="fn"

requires
  req id="R-001"
    text "Duplicate"
  end
end

signature
  fn name="fn_b"
    returns type="Unit"
  end
end

body
end

end
"#;
        let program = parse(source).unwrap();
        let extraction = extract(&program);

        // First occurrence should be in requirements
        assert_eq!(extraction.requirements.len(), 1);

        // Should have duplicate error
        assert_eq!(extraction.errors.len(), 1);
        assert!(matches!(&extraction.errors[0],
            RequirementError::DuplicateRequirement { id, first, second, .. }
            if id == "R-001" && first == "a.fn" && second == "b.fn"
        ));
    }

    #[test]
    fn test_duplicate_test_detection() {
        let source = r#"
snippet id="a.fn" kind="fn"

signature
  fn name="fn_a"
    returns type="Unit"
  end
end

body
end

tests
  test id="T-001" kind="unit"
  end
end

end

snippet id="b.fn" kind="fn"

signature
  fn name="fn_b"
    returns type="Unit"
  end
end

body
end

tests
  test id="T-001" kind="integration"
  end
end

end
"#;
        let program = parse(source).unwrap();
        let extraction = extract(&program);

        // First occurrence should be in tests
        assert_eq!(extraction.tests.len(), 1);

        // Should have duplicate error
        assert_eq!(extraction.errors.len(), 1);
        assert!(matches!(&extraction.errors[0],
            RequirementError::DuplicateTest { id, .. }
            if id == "T-001"
        ));
    }
}
