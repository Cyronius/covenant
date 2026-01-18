//! Covenant Lexer - Tokenization using logos
//!
//! Handles Covenant's unusual operators:
//! - `=` is equality (not `==`)
//! - `:=` is assignment
//! - `!=` is inequality

mod token;

pub use token::*;

use logos::Logos;
use covenant_ast::Span;

/// Tokenize a source string into a vector of tokens
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut lexer = TokenKind::lexer(source);

    while let Some(result) = lexer.next() {
        let span = Span::new(lexer.span().start, lexer.span().end);
        let kind = match result {
            Ok(kind) => kind,
            Err(_) => TokenKind::Error,
        };
        tokens.push(Token { kind, span });
    }

    // Add EOF token
    let end = source.len();
    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(end, end),
    });

    tokens
}

/// A token with its span
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.span.start..self.span.end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let tokens = tokenize("let x = 5");
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[2].kind, TokenKind::Eq);
        assert_eq!(tokens[3].kind, TokenKind::Int);
    }

    #[test]
    fn test_assignment_vs_equality() {
        let tokens = tokenize("x := 5");
        assert_eq!(tokens[1].kind, TokenKind::ColonEq);

        let tokens = tokenize("x = 5");
        assert_eq!(tokens[1].kind, TokenKind::Eq);
    }

    #[test]
    fn test_function_no_fn_keyword() {
        let tokens = tokenize("main() { }");
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[1].kind, TokenKind::LParen);
        assert_eq!(tokens[2].kind, TokenKind::RParen);
        assert_eq!(tokens[3].kind, TokenKind::LBrace);
    }

    // === Token Coverage ===

    #[test]
    fn test_snippet_keywords() {
        let tokens = tokenize("snippet id kind signature body end");
        assert_eq!(tokens[0].kind, TokenKind::Snippet);
        assert_eq!(tokens[1].kind, TokenKind::Id);
        assert_eq!(tokens[2].kind, TokenKind::Kind);
        assert_eq!(tokens[3].kind, TokenKind::Signature);
        assert_eq!(tokens[4].kind, TokenKind::Body);
        assert_eq!(tokens[5].kind, TokenKind::End);
    }

    #[test]
    fn test_section_keywords() {
        let tokens = tokenize("effects requires tests relations metadata");
        assert_eq!(tokens[0].kind, TokenKind::Effects);
        assert_eq!(tokens[1].kind, TokenKind::Requires);
        assert_eq!(tokens[2].kind, TokenKind::Tests);
        assert_eq!(tokens[3].kind, TokenKind::Relations);
        assert_eq!(tokens[4].kind, TokenKind::Metadata);
    }

    #[test]
    fn test_step_keywords() {
        let tokens = tokenize("step op input var lit as");
        assert_eq!(tokens[0].kind, TokenKind::Step);
        assert_eq!(tokens[1].kind, TokenKind::Op);
        assert_eq!(tokens[2].kind, TokenKind::Input);
        assert_eq!(tokens[3].kind, TokenKind::Var);
        assert_eq!(tokens[4].kind, TokenKind::Lit);
        assert_eq!(tokens[5].kind, TokenKind::As);
    }

    #[test]
    fn test_operation_keywords() {
        let tokens = tokenize("add sub mul div equals not and or");
        assert_eq!(tokens[0].kind, TokenKind::Add);
        assert_eq!(tokens[1].kind, TokenKind::Sub);
        assert_eq!(tokens[2].kind, TokenKind::Mul);
        assert_eq!(tokens[3].kind, TokenKind::Div);
        assert_eq!(tokens[4].kind, TokenKind::Equals);
        assert_eq!(tokens[5].kind, TokenKind::Not);
        assert_eq!(tokens[6].kind, TokenKind::And);
        assert_eq!(tokens[7].kind, TokenKind::Or);
    }

    #[test]
    fn test_query_keywords() {
        let tokens = tokenize("select from where order by limit");
        assert_eq!(tokens[0].kind, TokenKind::Select);
        assert_eq!(tokens[1].kind, TokenKind::From);
        assert_eq!(tokens[2].kind, TokenKind::Where);
        assert_eq!(tokens[3].kind, TokenKind::Order);
        assert_eq!(tokens[4].kind, TokenKind::By);
        assert_eq!(tokens[5].kind, TokenKind::Limit);
    }

    #[test]
    fn test_all_operators() {
        // Comparison operators
        let tokens = tokenize("= != < <= > >=");
        assert_eq!(tokens[0].kind, TokenKind::Eq);
        assert_eq!(tokens[1].kind, TokenKind::Ne);
        assert_eq!(tokens[2].kind, TokenKind::Lt);
        assert_eq!(tokens[3].kind, TokenKind::Le);
        assert_eq!(tokens[4].kind, TokenKind::Gt);
        assert_eq!(tokens[5].kind, TokenKind::Ge);

        // Arithmetic operators
        let tokens = tokenize("+ - * / %");
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::Percent);

        // Boolean operators
        let tokens = tokenize("&& || !");
        assert_eq!(tokens[0].kind, TokenKind::AndAnd);
        assert_eq!(tokens[1].kind, TokenKind::OrOr);
        assert_eq!(tokens[2].kind, TokenKind::Bang);

        // Special operators
        let tokens = tokenize("-> => :: :=");
        assert_eq!(tokens[0].kind, TokenKind::Arrow);
        assert_eq!(tokens[1].kind, TokenKind::FatArrow);
        assert_eq!(tokens[2].kind, TokenKind::ColonColon);
        assert_eq!(tokens[3].kind, TokenKind::ColonEq);
    }

    #[test]
    fn test_delimiters_and_punctuation() {
        let tokens = tokenize("( ) { } [ ] | , : ; . ?");
        assert_eq!(tokens[0].kind, TokenKind::LParen);
        assert_eq!(tokens[1].kind, TokenKind::RParen);
        assert_eq!(tokens[2].kind, TokenKind::LBrace);
        assert_eq!(tokens[3].kind, TokenKind::RBrace);
        assert_eq!(tokens[4].kind, TokenKind::LBracket);
        assert_eq!(tokens[5].kind, TokenKind::RBracket);
        assert_eq!(tokens[6].kind, TokenKind::Pipe);
        assert_eq!(tokens[7].kind, TokenKind::Comma);
        assert_eq!(tokens[8].kind, TokenKind::Colon);
        assert_eq!(tokens[9].kind, TokenKind::Semicolon);
        assert_eq!(tokens[10].kind, TokenKind::Dot);
        assert_eq!(tokens[11].kind, TokenKind::Question);
    }

    #[test]
    fn test_boolean_literals() {
        let tokens = tokenize("true false none");
        assert_eq!(tokens[0].kind, TokenKind::True);
        assert_eq!(tokens[1].kind, TokenKind::False);
        assert_eq!(tokens[2].kind, TokenKind::None);
    }

    // === String Tests ===

    #[test]
    fn test_simple_string() {
        let source = r#""hello world""#;
        let tokens = tokenize(source);
        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(tokens[0].text(source), r#""hello world""#);
    }

    #[test]
    fn test_string_with_escapes() {
        let source = r#""hello \"world\"""#;
        let tokens = tokenize(source);
        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(tokens[0].text(source), r#""hello \"world\"""#);
    }

    #[test]
    fn test_string_with_newline_escape() {
        let source = r#""line1\nline2""#;
        let tokens = tokenize(source);
        assert_eq!(tokens[0].kind, TokenKind::String);
    }

    #[test]
    fn test_triple_quoted_string() {
        let source = r#""""multi
line
string""""#;
        let tokens = tokenize(source);
        assert_eq!(tokens[0].kind, TokenKind::TripleString);
    }

    // === Number Tests ===

    #[test]
    fn test_integer_formats() {
        let source = "0 42 12345";
        let tokens = tokenize(source);
        assert_eq!(tokens[0].kind, TokenKind::Int);
        assert_eq!(tokens[0].text(source), "0");
        assert_eq!(tokens[1].kind, TokenKind::Int);
        assert_eq!(tokens[1].text(source), "42");
        assert_eq!(tokens[2].kind, TokenKind::Int);
        assert_eq!(tokens[2].text(source), "12345");
    }

    #[test]
    fn test_float_formats() {
        let source = "1.0 0.5 123.456";
        let tokens = tokenize(source);
        assert_eq!(tokens[0].kind, TokenKind::Float);
        assert_eq!(tokens[0].text(source), "1.0");
        assert_eq!(tokens[1].kind, TokenKind::Float);
        assert_eq!(tokens[1].text(source), "0.5");
        assert_eq!(tokens[2].kind, TokenKind::Float);
        assert_eq!(tokens[2].text(source), "123.456");
    }

    #[test]
    fn test_negative_number_is_two_tokens() {
        // Negative numbers are represented as Minus followed by Int
        let tokens = tokenize("-5");
        assert_eq!(tokens[0].kind, TokenKind::Minus);
        assert_eq!(tokens[1].kind, TokenKind::Int);
    }

    // === Error Cases ===

    #[test]
    fn test_invalid_token_produces_error() {
        let tokens = tokenize("let x @ 5");
        // @ is not a valid token
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Error));
    }

    #[test]
    fn test_unterminated_string() {
        let tokens = tokenize(r#""unclosed string"#);
        // Should produce Error token for unterminated string
        assert!(tokens.iter().any(|t| t.kind == TokenKind::Error));
    }

    // === Edge Cases ===

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }

    #[test]
    fn test_whitespace_only() {
        let tokens = tokenize("   \n\t\r\n  ");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
    }

    #[test]
    fn test_line_comments_stripped() {
        let tokens = tokenize("let x // this is a comment\n= 5");
        // Should produce: let, x, =, 5, eof (no comment tokens)
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[2].kind, TokenKind::Eq);
        assert_eq!(tokens[3].kind, TokenKind::Int);
        assert_eq!(tokens[4].kind, TokenKind::Eof);
    }

    #[test]
    fn test_span_accuracy() {
        let source = "let x = 5";
        let tokens = tokenize(source);

        // Verify spans point to correct text
        assert_eq!(tokens[0].text(source), "let");
        assert_eq!(tokens[1].text(source), "x");
        assert_eq!(tokens[2].text(source), "=");
        assert_eq!(tokens[3].text(source), "5");

        // Verify span positions
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, 3);
        assert_eq!(tokens[1].span.start, 4);
        assert_eq!(tokens[1].span.end, 5);
    }

    #[test]
    fn test_identifier_with_underscores() {
        let source = "my_variable _private __dunder__";
        let tokens = tokenize(source);
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].text(source), "my_variable");
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].text(source), "_private");
        assert_eq!(tokens[2].kind, TokenKind::Ident);
        assert_eq!(tokens[2].text(source), "__dunder__");
    }

    #[test]
    fn test_identifier_with_numbers() {
        let source = "var1 x2y z123";
        let tokens = tokenize(source);
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].text(source), "var1");
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].text(source), "x2y");
        assert_eq!(tokens[2].kind, TokenKind::Ident);
        assert_eq!(tokens[2].text(source), "z123");
    }

    #[test]
    fn test_keyword_is_keyword() {
        assert!(TokenKind::Let.is_keyword());
        assert!(TokenKind::Snippet.is_keyword());
        assert!(TokenKind::Add.is_keyword());
        assert!(!TokenKind::Ident.is_keyword());
        assert!(!TokenKind::Int.is_keyword());
        assert!(!TokenKind::Eq.is_keyword());
    }
}
