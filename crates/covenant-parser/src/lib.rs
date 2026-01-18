//! Covenant Parser - Recursive descent parser
//!
//! Parses Covenant source code into an AST.
//! Key parsing challenges:
//! - No `fn` keyword: functions are identified by signature shape
//! - `=` is equality, `:=` is assignment
//! - Query expressions with SQL-like syntax

mod error;
mod parser;

pub use error::*;
pub use parser::*;

use covenant_ast::Program;
use covenant_lexer::tokenize;

/// Parse a source string into a Program AST
pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = tokenize(source);
    let mut parser = Parser::new(source, tokens);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hello_world() {
        let source = r#"
            main()
                import { println } from console
            {
                println("Hello, world!")
            }
        "#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_parse_struct() {
        let source = r#"
            struct User {
                id: Int,
                name: String,
            }
        "#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_parse_pure_function() {
        let source = r#"
            double(x: Int) -> Int {
                x * 2
            }
        "#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    // === Snippet Kind Tests ===

    #[test]
    fn test_parse_snippet_fn() {
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
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse fn snippet: {:?}", result.err());
        let program = result.unwrap();
        if let Program::Snippets { snippets, .. } = program {
            assert_eq!(snippets.len(), 1);
            assert_eq!(snippets[0].id, "math.add");
        } else {
            panic!("Expected Snippets program");
        }
    }

    #[test]
    fn test_parse_extern_snippet() {
        let source = r#"
snippet id="io.print" kind="extern"

effects
  effect console
end

signature
  fn name="print"
    param name="msg" type="String"
    returns type="Unit"
  end
end

metadata
  contract="console.log@1"
end

end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse extern snippet: {:?}", result.err());
    }

    #[test]
    fn test_parse_struct_snippet() {
        let source = r#"
snippet id="types.User" kind="struct"

signature
  struct name="User"
    field name="id" type="Int"
    field name="name" type="String"
    field name="email" type="String" optional
  end
end

end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse struct snippet: {:?}", result.err());
    }

    #[test]
    fn test_parse_enum_snippet() {
        let source = r#"
snippet id="types.Result" kind="enum"

signature
  enum name="Result"
    variant name="Ok"
      field name="value" type="Int"
    end
    variant name="Err"
      field name="message" type="String"
    end
  end
end

end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse enum snippet: {:?}", result.err());
    }

    // === Step Kind Tests ===

    #[test]
    fn test_parse_compute_step() {
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
    input lit=1
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end
end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse compute step: {:?}", result.err());
    }

    #[test]
    fn test_parse_call_step() {
        let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="x" type="Int"
    returns type="Int"
  end
end
body
  step id="s1" kind="call"
    fn="math.double"
    arg name="x" from="x"
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end
end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse call step: {:?}", result.err());
    }

    #[test]
    fn test_parse_if_step() {
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
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse if step: {:?}", result.err());
    }

    #[test]
    fn test_parse_match_step() {
        let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="value" type="Result"
    returns type="Int"
  end
end
body
  step id="s1" kind="match"
    on="value"
    case variant type="Result::Ok" bindings=("v")
      step id="s1a" kind="return"
        from="v"
        as="_"
      end
    end
    case wildcard
      step id="s1b" kind="return"
        lit=0
        as="_"
      end
    end
    as="_"
  end
end
end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse match step: {:?}", result.err());
    }

    #[test]
    fn test_parse_for_step() {
        let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="items" type="List<Int>"
    returns type="List<Int>"
  end
end
body
  step id="s1" kind="call"
    fn="new_list"
    as="result"
  end
  step id="s2" kind="for"
    var="item" in="items"
    step id="s2a" kind="call"
      fn="push"
      arg name="list" from="result"
      arg name="item" from="item"
      as="result"
    end
    as="_"
  end
  step id="s3" kind="return"
    from="result"
    as="_"
  end
end
end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse for step: {:?}", result.err());
    }

    // === Query Tests ===

    #[test]
    fn test_parse_query_covenant_dialect() {
        let source = r#"
snippet id="test.fn" kind="fn"
effects
  effect database
end
signature
  fn name="test_fn"
    returns type="List<User>"
  end
end
body
  step id="s1" kind="query"
    target="project"
    select all
    from="users"
    where
      equals field="active" lit=true
    end
    order by="name" dir="asc"
    limit=10
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end
end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse Covenant query: {:?}", result.err());
    }

    #[test]
    fn test_parse_query_sql_dialect() {
        let source = r#"
snippet id="test.fn" kind="fn"
effects
  effect database
end
signature
  fn name="test_fn"
    param name="user_id" type="Int"
    returns type="List<Order>"
  end
end
body
  step id="s1" kind="query"
    dialect="postgres"
    target="app_db"
    body
      SELECT * FROM orders WHERE user_id = :user_id
    end
    params
      param name="user_id" from="user_id"
    end
    returns collection of="Order"
    as="orders"
  end
  step id="s2" kind="return"
    from="orders"
    as="_"
  end
end
end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse SQL dialect query: {:?}", result.err());
    }

    // === Type Syntax Tests ===

    #[test]
    fn test_parse_optional_type() {
        let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="value" type="Json"
    returns type="String" optional
  end
end
body
  step id="s1" kind="return"
    lit=none
    as="_"
  end
end
end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse optional type: {:?}", result.err());
    }

    #[test]
    fn test_parse_list_type() {
        let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    param name="items" type="List<Int>"
    returns type="List<String>"
  end
end
body
  step id="s1" kind="return"
    lit=none
    as="_"
  end
end
end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse List type: {:?}", result.err());
    }

    #[test]
    fn test_parse_union_type() {
        let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test_fn"
    returns union
      type="Int"
      type="String"
      type="Error"
    end
  end
end
body
  step id="s1" kind="return"
    lit=0
    as="_"
  end
end
end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse union type: {:?}", result.err());
    }

    // === Section Tests ===

    #[test]
    fn test_parse_snippet_with_all_sections() {
        let source = r#"
snippet id="test.fn" kind="fn"

effects
  effect database
  effect network
end

requires
  req id="R-001"
    text "Must handle null input"
    priority high
  end
end

signature
  fn name="test_fn"
    param name="x" type="Int"
    returns type="Int"
  end
end

body
  step id="s1" kind="return"
    from="x"
    as="_"
  end
end

tests
  test id="T-001" kind="unit" covers="R-001"
  end
end

metadata
  author="test"
  version="1.0"
end

end
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse snippet with all sections: {:?}", result.err());
    }

    // === Multiple Snippets ===

    #[test]
    fn test_parse_multiple_snippets() {
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

snippet id="math.sub" kind="fn"
signature
  fn name="sub"
    param name="a" type="Int"
    param name="b" type="Int"
    returns type="Int"
  end
end
body
  step id="s1" kind="compute"
    op=sub
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
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse multiple snippets: {:?}", result.err());
        let program = result.unwrap();
        if let Program::Snippets { snippets, .. } = program {
            assert_eq!(snippets.len(), 2);
        } else {
            panic!("Expected Snippets program");
        }
    }

    // === Error Cases ===

    #[test]
    fn test_parse_empty_source() {
        let result = parse("");
        assert!(result.is_ok(), "Empty source should parse successfully");
        let program = result.unwrap();
        match program {
            Program::Snippets { snippets, .. } => assert!(snippets.is_empty()),
            Program::Legacy { declarations, .. } => assert!(declarations.is_empty()),
        }
    }

    #[test]
    fn test_parse_comments_only() {
        let source = r#"
// This is a comment
// Another comment
"#;
        let result = parse(source);
        assert!(result.is_ok(), "Comments-only should parse successfully");
    }

    #[test]
    fn test_parse_unclosed_snippet() {
        let source = r#"
snippet id="test.fn" kind="fn"
signature
  fn name="test"
    returns type="Int"
  end
end
body
end
"#;
        // Missing final "end" for snippet
        let result = parse(source);
        assert!(result.is_err(), "Unclosed snippet should fail to parse");
    }

    #[test]
    fn test_parse_missing_snippet_id() {
        let source = r#"
snippet kind="fn"
signature
  fn name="test"
    returns type="Int"
  end
end
body
end
end
"#;
        let result = parse(source);
        assert!(result.is_err(), "Missing snippet id should fail to parse");
    }
}
