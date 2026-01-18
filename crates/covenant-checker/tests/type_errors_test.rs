//! Integration tests for type error detection (Phase 4)

use covenant_checker::check;
use covenant_parser::parse;

/// Helper to check that parsing and type checking produces errors
fn check_source_has_errors(source: &str) -> Vec<covenant_checker::CheckError> {
    let program = parse(source).expect("parse failed");
    match check(&program) {
        Ok(_) => vec![],
        Err(errors) => errors,
    }
}

/// Helper to check that parsing and type checking succeeds
fn check_source_ok(source: &str) {
    let program = parse(source).expect("parse failed");
    let result = check(&program);
    assert!(result.is_ok(), "Expected check to succeed, got errors: {:?}", result.err());
}

// === Type Mismatch Tests ===

#[test]
fn test_return_type_mismatch() {
    let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    returns type="Int"
  end
end
body
  step id="s1" kind="return"
    lit="not an int"
    as="_"
  end
end
end
"#;
    let errors = check_source_has_errors(source);
    // Should have type mismatch error: returning String when Int expected
    assert!(!errors.is_empty(), "Expected type mismatch error");
}

#[test]
fn test_correct_return_type() {
    let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    returns type="Int"
  end
end
body
  step id="s1" kind="return"
    lit=42
    as="_"
  end
end
end
"#;
    check_source_ok(source);
}

// === Undefined Variable Tests ===

#[test]
fn test_undefined_variable_in_return() {
    let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    returns type="Int"
  end
end
body
  step id="s1" kind="return"
    from="undefined_var"
    as="_"
  end
end
end
"#;
    let errors = check_source_has_errors(source);
    assert!(!errors.is_empty(), "Expected undefined variable error");
}

#[test]
fn test_undefined_variable_in_compute() {
    let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="x" type="Int"
    returns type="Int"
  end
end
body
  step id="s1" kind="compute"
    op=add
    input var="x"
    input var="y"
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end
end
"#;
    let errors = check_source_has_errors(source);
    assert!(!errors.is_empty(), "Expected undefined variable 'y' error");
}

// === Compute Operation Type Tests ===

#[test]
fn test_compute_add_int_int() {
    let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="a" type="Int"
    param name="b" type="Int"
    returns type="Int"
  end
end
body
  step id="s1" kind="compute"
    op=add
    input var="a"
    input var="b"
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end
end
"#;
    check_source_ok(source);
}

#[test]
fn test_compute_equals_returns_bool() {
    let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="a" type="Int"
    param name="b" type="Int"
    returns type="Bool"
  end
end
body
  step id="s1" kind="compute"
    op=equals
    input var="a"
    input var="b"
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end
end
"#;
    check_source_ok(source);
}

// === If Statement Tests ===

#[test]
fn test_if_condition_bool() {
    let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="x" type="Int"
    returns type="Int"
  end
end
body
  step id="s1" kind="compute"
    op=less
    input var="x"
    input lit=0
    as="is_negative"
  end
  step id="s2" kind="if"
    condition="is_negative"
    then
      step id="s2a" kind="return"
        lit=0
        as="_"
      end
    end
    else
      step id="s2b" kind="return"
        from="x"
        as="_"
      end
    end
    as="_"
  end
end
end
"#;
    check_source_ok(source);
}

// === Call Argument Type Tests ===

#[test]
fn test_call_correct_arg_type() {
    let source = r#"
snippet id="math.double" kind="fn"
signature
  fn name="double"
    param name="x" type="Int"
    returns type="Int"
  end
end
body
  step id="s1" kind="compute"
    op=mul
    input var="x"
    input lit=2
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end
end

snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="n" type="Int"
    returns type="Int"
  end
end
body
  step id="s1" kind="call"
    fn="math.double"
    arg name="x" from="n"
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end
end
"#;
    check_source_ok(source);
}

// === Multiple Snippets Interaction ===

#[test]
fn test_cross_snippet_call() {
    let source = r#"
snippet id="math.add" kind="fn"
signature
  fn name="add"
    param name="a" type="Int"
    param name="b" type="Int"
    returns type="Int"
  end
end
body
  step id="s1" kind="compute"
    op=add
    input var="a"
    input var="b"
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end
end

snippet id="app.main" kind="fn"
signature
  fn name="main"
    returns type="Int"
  end
end
body
  step id="s1" kind="call"
    fn="math.add"
    arg name="a" lit=1
    arg name="b" lit=2
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end
end
"#;
    check_source_ok(source);
}

// === Union Type Tests ===

#[test]
fn test_union_return_type() {
    let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="x" type="Int"
    returns union
      type="Int"
      type="String"
    end
  end
end
body
  step id="s1" kind="return"
    from="x"
    as="_"
  end
end
end
"#;
    check_source_ok(source);
}

// === Bind Step Tests ===

#[test]
fn test_bind_literal() {
    let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    returns type="Int"
  end
end
body
  step id="s1" kind="bind"
    lit=42
    as="x"
  end
  step id="s2" kind="return"
    from="x"
    as="_"
  end
end
end
"#;
    check_source_ok(source);
}

#[test]
fn test_bind_from_param() {
    let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="input" type="Int"
    returns type="Int"
  end
end
body
  step id="s1" kind="bind"
    from="input"
    as="x"
  end
  step id="s2" kind="return"
    from="x"
    as="_"
  end
end
end
"#;
    check_source_ok(source);
}
