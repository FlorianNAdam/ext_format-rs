use core::iter::Peekable;
use core::str::Chars;

#[derive(Debug, PartialEq)]
pub(crate) enum QuoteToken {
    Literal(String),
    Variable(String, Option<String>),
    HiddenVariable(String, Option<String>),
    Group(Vec<QuoteToken>, Option<String>),
}

/// A simple recursive descent parser
/// It is quite fast but definitely needs a bit of a refactoring before release
/// I will probably end up writing a library to do this eventually
pub(crate) fn parse(source: &str) -> Vec<QuoteToken> {
    parse_toplevel(&mut source.chars().peekable())
}

macro_rules! flush_literal {
    ($tokens:ident, $literal:ident) => {
        if !$literal.is_empty() {
            $tokens.push(QuoteToken::Literal($literal));
            $literal = String::new();
        }
    };
}

macro_rules! final_flush_literal {
    ($tokens:ident, $literal:ident) => {
        if !$literal.is_empty() {
            $tokens.push(QuoteToken::Literal($literal));
        }
    };
}

fn parse_toplevel(source: &mut Peekable<Chars>) -> Vec<QuoteToken> {
    let mut res = vec![];

    let mut current_literal = String::new();
    while let Some(current_char) = source.next() {
        match current_char {
            '@' => {
                flush_literal!(res, current_literal);

                let token = parse_hidden_variable(source);
                res.push(token);
            }
            '$' => {
                flush_literal!(res, current_literal);

                let token = parse_binding(source);
                res.push(token);
            }
            '\\' => {
                let next_char = source.next().unwrap();
                current_literal.push(next_char);
            }
            char => {
                current_literal.push(char);
            }
        }
    }
    final_flush_literal!(res, current_literal);

    res
}

fn parse_group(source: &mut Peekable<Chars>) -> QuoteToken {
    if source.next() != Some('(') {
        panic!("expected (")
    }

    let mut res = vec![];

    let mut depth = 0;

    let mut current_literal = String::new();
    while let Some(current_char) = source.next() {
        match current_char {
            '@' => {
                flush_literal!(res, current_literal);

                let token = parse_hidden_variable(source);
                res.push(token);
            }
            '$' => {
                flush_literal!(res, current_literal);

                let token = parse_binding(source);
                res.push(token);
            }
            '\\' => {
                let next_char = source.next().unwrap();
                current_literal.push(next_char);
            }
            '(' => {
                depth += 1;
                current_literal.push('(');
            }
            ')' => {
                if depth == 0 {
                    final_flush_literal!(res, current_literal);

                    let separator = parse_group_separator(source);

                    return QuoteToken::Group(res, separator);
                } else {
                    depth -= 1;
                    current_literal.push(')');
                }
            }
            char => {
                current_literal.push(char);
            }
        }
    }
    panic!("unexpected end of variable group")
}

fn parse_group_separator(source: &mut Peekable<Chars>) -> Option<String> {
    let next_char = source.next().expect("expected separator");
    if next_char == '*' {
        None
    } else if next_char == '(' {
        let mut separator = String::new();
        while let Some(next_char) = source.next() {
            if next_char == ')' {
                break;
            }
            separator.push(next_char);
        }
        if source.next().unwrap() != '*' {
            panic!("expected * after variable group");
        }
        Some(separator)
    } else {
        if source.next().unwrap() != '*' {
            panic!("expected * after variable group");
        }
        Some(next_char.to_string())
    }
}

fn parse_binding(source: &mut Peekable<Chars>) -> QuoteToken {
    let next_char = *source.peek().unwrap();
    let token = match next_char {
        '(' => parse_group(source),
        _ => parse_variable(source),
    };
    token
}

fn parse_variable(source: &mut Peekable<Chars>) -> QuoteToken {
    let (ident, inner_ident) = parse_variable_idents(source);
    QuoteToken::Variable(ident, inner_ident)
}

fn parse_hidden_variable(source: &mut Peekable<Chars>) -> QuoteToken {
    let (ident, inner_ident) = parse_variable_idents(source);
    QuoteToken::HiddenVariable(ident, inner_ident)
}

fn parse_variable_idents(source: &mut Peekable<Chars>) -> (String, Option<String>) {
    let next_char = *source.peek().unwrap();
    match next_char {
        '{' => parse_bound_ident(source),
        _ => (parse_ident(source), None),
    }
}

fn parse_ident(source: &mut Peekable<Chars>) -> String {
    let mut ident = String::new();
    let var_start = source.next().unwrap();
    if !(var_start.is_alphabetic() || var_start == '_') {
        panic!("expected identifier")
    }
    ident.push(var_start);
    while let Some(current_char) = source.peek() {
        if !(current_char.is_alphanumeric() || current_char == &'_') {
            break;
        }
        let current_char = source.next().unwrap();
        ident.push(current_char);
    }
    ident
}

fn parse_bound_ident(source: &mut Peekable<Chars>) -> (String, Option<String>) {
    if source.next() != Some('{') {
        panic!("expected {{")
    }
    let ident = parse_ident(source);
    let next_char = source.next().unwrap();
    match next_char {
        ':' => {
            let inner_ident = parse_ident(source);
            if source.next() != Some('}') {
                panic!("expected }}")
            }
            (ident, Some(inner_ident))
        }
        '}' => (ident, None),
        _ => panic!("expected : or }}"),
    }
}

#[cfg(test)]
mod tests {
    use super::QuoteToken::*;
    use super::*;
    use crate::util::{unescape, unindent};
    use std::iter::Peekable;
    use std::str::Chars;
    macro_rules! expect_match {
        ($value:expr => $pattern:pat in $unpacked:expr) => {
            if let $pattern = $value {
                $unpacked
            } else {
                panic!("unexpected enum Variant: {:?}", $value);
            }
        };
    }
    #[test]
    fn test_parse_toplevel() {
        let source = unescape(&unindent(
            r#"
        void $name($($types $names)(, )*) {
            $func("hallo", $num);
            $(@lines printf("$($lines)( --> )* %d, %d", $nums, $nums2))(;\n    )*;
        }
            "#,
        ));
        let mut source: Peekable<Chars> = source.trim().chars().peekable();
        let tokens = parse_toplevel(&mut source);

        assert_eq!(
            tokens,
            vec![
                Literal("void ".to_string()),
                Variable("name".to_string(), None),
                Literal("(".to_string()),
                Group(
                    vec![
                        Variable("types".to_string(), None),
                        Literal(" ".to_string()),
                        Variable("names".to_string(), None)
                    ],
                    Some(", ".to_string())
                ),
                Literal(") {\n    ".to_string()),
                Variable("func".to_string(), None),
                Literal("(\"hallo\", ".to_string()),
                Variable("num".to_string(), None),
                Literal(");\n    ".to_string()),
                Group(
                    vec![
                        HiddenVariable("lines".to_string(), None),
                        Literal(" printf(\"".to_string()),
                        Group(
                            vec![Variable("lines".to_string(), None)],
                            Some(" --> ".to_string())
                        ),
                        Literal(" %d, %d\", ".to_string()),
                        Variable("nums".to_string(), None),
                        Literal(", ".to_string()),
                        Variable("nums2".to_string(), None),
                        Literal(")".to_string())
                    ],
                    Some(";\n    ".to_string())
                ),
                Literal(";\n}".to_string())
            ]
        );
    }

    #[test]
    fn test_parse_toplevel_matrix() {
        let source = unescape(&unindent(
            r#"
        void func() {
            $(@{matrix:inner_matrix}printf("$($inner_matrix) *");)(\n    )*
            printf("\\(");
        }
            "#,
        ));
        let mut source: Peekable<Chars> = source.trim().chars().peekable();
        let tokens = parse_toplevel(&mut source);

        assert_eq!(
            tokens,
            vec![
                Literal("void func() {\n    ".to_string()),
                Group(
                    vec![
                        HiddenVariable("matrix".to_string(), Some("inner_matrix".to_string())),
                        Literal("printf(\"".to_string()),
                        Group(
                            vec![Variable("inner_matrix".to_string(), None)],
                            Some(" ".to_string())
                        ),
                        Literal("\");".to_string())
                    ],
                    Some("\n    ".to_string())
                ),
                Literal("\n    printf(\"(\");\n}".to_string())
            ]
        );
    }

    #[test]
    fn test_parse_group_basic() {
        let mut source: Peekable<Chars> = "(literal)*".chars().peekable();
        let token = parse_group(&mut source);

        expect_match!(token => QuoteToken::Group(tokens, separator) in {
            assert_eq!(1, tokens.len());
            expect_match!(&tokens[0] => QuoteToken::Literal(literal) in assert_eq!(literal, "literal"));
            assert_eq!(separator, None);
        });
    }

    #[test]
    fn test_parse_group_with_char_separator() {
        let mut source: Peekable<Chars> = "(literal);*".chars().peekable();
        let token = parse_group(&mut source);

        expect_match!(token => QuoteToken::Group(tokens, separator) in {
            assert_eq!(1, tokens.len());
            expect_match!(&tokens[0] => QuoteToken::Literal(literal) in assert_eq!(literal, "literal"));
            assert_eq!(separator, Some(";".to_string()));
        });
    }

    #[test]
    fn test_parse_group_with_string_separator() {
        let mut source: Peekable<Chars> = "(literal)(=>)*".chars().peekable();
        let token = parse_group(&mut source);

        expect_match!(token => QuoteToken::Group(tokens, separator) in {
            assert_eq!(1, tokens.len());
            expect_match!(&tokens[0] => QuoteToken::Literal(literal) in assert_eq!(literal, "literal"));
            assert_eq!(separator, Some("=>".to_string()));
        });
    }

    #[test]
    fn test_parse_group_with_escaped_separator() {
        let mut source: Peekable<Chars> = "(literal)(\n)*".chars().peekable();
        let token = parse_group(&mut source);

        expect_match!(token => QuoteToken::Group(tokens, separator) in {
            assert_eq!(1, tokens.len());
            expect_match!(&tokens[0] => QuoteToken::Literal(literal) in assert_eq!(literal, "literal"));
            assert_eq!(separator, Some("\n".to_string()));
        });
    }

    #[test]
    fn test_parse_group_with_escaped_escaped_separator() {
        let mut source: Peekable<Chars> = "(literal)(\\n)*".chars().peekable();
        let token = parse_group(&mut source);

        expect_match!(token => QuoteToken::Group(tokens, separator) in {
            assert_eq!(1, tokens.len());
            expect_match!(&tokens[0] => QuoteToken::Literal(literal) in assert_eq!(literal, "literal"));
            assert_eq!(separator, Some("\\n".to_string()));
        });
    }

    #[test]
    fn test_parse_group_with_variable() {
        let mut source: Peekable<Chars> = "(literal $var)*".chars().peekable();
        let token = parse_group(&mut source);

        expect_match!(token => QuoteToken::Group(tokens, _) in {
            expect_match!(&tokens[0] => QuoteToken::Literal(literal) in assert_eq!(literal, "literal "));
            expect_match!(
                &tokens[1] => QuoteToken::Variable(ident, inner_ident) in {
                    assert_eq!(ident, "var");
                    assert_eq!(inner_ident, &None);
                }
            );
        });
    }

    #[test]
    fn test_parse_group_with_variable_and_trailing_literal() {
        let mut source: Peekable<Chars> = "(literal1 $variable literal2)*".chars().peekable();
        let token = parse_group(&mut source);

        expect_match!(token => QuoteToken::Group(tokens, _) in {
            expect_match!(&tokens[0] => QuoteToken::Literal(literal) in assert_eq!(literal, "literal1 "));
            expect_match!(
                &tokens[1] => QuoteToken::Variable(ident, inner_ident) in {
                    assert_eq!(ident, "variable");
                    assert_eq!(inner_ident, &None);
                }
            );
            expect_match!(&tokens[2] => QuoteToken::Literal(literal) in assert_eq!(literal, " literal2"));
        });
    }
    #[test]
    fn test_parse_group_with_hidden_variable() {
        let mut source: Peekable<Chars> = "(literal @var)*".chars().peekable();
        let token = parse_group(&mut source);

        expect_match!(token => QuoteToken::Group(tokens, _) in {
            expect_match!(&tokens[0] => QuoteToken::Literal(literal) in assert_eq!(literal, "literal "));
            expect_match!(
                &tokens[1] => QuoteToken::HiddenVariable(ident, inner_ident) in {
                    assert_eq!(ident, "var");
                    assert_eq!(inner_ident, &None);
                }
            );
        });
    }

    #[test]
    fn test_parse_group_with_hidden_variable_and_trailing_literal() {
        let mut source: Peekable<Chars> = "(literal1 @variable literal2)**".chars().peekable();
        let token = parse_group(&mut source);

        expect_match!(token => QuoteToken::Group(tokens, _) in {
            expect_match!(&tokens[0] => QuoteToken::Literal(literal) in assert_eq!(literal, "literal1 "));
            expect_match!(
                &tokens[1] => QuoteToken::HiddenVariable(ident, inner_ident) in {
                    assert_eq!(ident, "variable");
                    assert_eq!(inner_ident, &None);
                }
            );
            expect_match!(&tokens[2] => QuoteToken::Literal(literal) in assert_eq!(literal, " literal2"));
        });
    }
    #[test]
    #[should_panic]
    fn test_parse_group_unexpected_end() {
        let mut source: Peekable<Chars> = "(".chars().peekable();
        parse_group(&mut source);
    }

    #[test]
    fn test_parse_group_with_balanced_parenthesis() {
        let expected_literal = "literal () ((literal), ((), ()))";

        let mut source: Peekable<Chars> = "(literal () ((literal), ((), ())))*".chars().peekable();
        let token = parse_group(&mut source);

        expect_match!(token => QuoteToken::Group(tokens, _) in {
            expect_match!(&tokens[0] => QuoteToken::Literal(literal) in assert_eq!(literal, expected_literal));
        });
    }

    #[test]
    fn test_parse_group_with_unbalanced_parenthesis() {
        let expected_literal = "literal ( () ((literal, ((, ()))))";

        let mut source: Peekable<Chars> = ("(literal \\( () (\\(literal, (\\(, ()))\\)\\))*")
            .chars()
            .peekable();
        let token = parse_group(&mut source);

        expect_match!(token => QuoteToken::Group(tokens, _) in {
            expect_match!(&tokens[0] => QuoteToken::Literal(literal) in assert_eq!(literal, expected_literal));
        });
    }
    #[test]
    fn test_parse_binding_with_variable() {
        let mut source: Peekable<Chars> = "variable".chars().peekable();
        let token = parse_binding(&mut source);

        expect_match!(
            token => QuoteToken::Variable(ident, inner_ident) in {
                assert_eq!(ident, "variable");
                assert_eq!(inner_ident, None);
            }
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_binding_invalid_start() {
        let mut source: Peekable<Chars> = "1invalid".chars().peekable();
        parse_binding(&mut source);
    }

    #[test]
    fn test_parse_variable_idents_with_braces() {
        let mut source: Peekable<Chars> = "{foo:bar}".chars().peekable();
        let (ident, inner_ident) = parse_variable_idents(&mut source);

        assert_eq!(ident, "foo");
        assert_eq!(inner_ident, Some("bar".to_string()));
    }

    #[test]
    fn test_parse_variable_idents_with_braces_single_ident() {
        let mut source: Peekable<Chars> = "{foo}".chars().peekable();
        let (ident, inner_ident) = parse_variable_idents(&mut source);

        assert_eq!(ident, "foo");
        assert_eq!(inner_ident, None);
    }

    #[test]
    fn test_parse_variable_idents_without_braces() {
        let mut source: Peekable<Chars> = "foo".chars().peekable();
        let (ident, inner_ident) = parse_variable_idents(&mut source);

        assert_eq!(ident, "foo");
        assert_eq!(inner_ident, None);
    }

    #[test]
    #[should_panic]
    fn test_parse_variable_idents_invalid_start_with_braces() {
        let mut source: Peekable<Chars> = "{1foo:bar}".chars().peekable();
        parse_variable_idents(&mut source);
    }

    #[test]
    #[should_panic]
    fn test_parse_variable_idents_invalid_start_without_braces() {
        let mut source: Peekable<Chars> = "1foo".chars().peekable();
        parse_variable_idents(&mut source);
    }

    #[test]
    fn test_parse_ident_valid() {
        let mut source: Peekable<Chars> = "foo123_".chars().peekable();
        let ident = parse_ident(&mut source);

        assert_eq!(ident, "foo123_");
    }

    #[test]
    fn test_parse_ident_start_with_underscore() {
        let mut source: Peekable<Chars> = "_foo".chars().peekable();
        let ident = parse_ident(&mut source);

        assert_eq!(ident, "_foo");
    }

    #[test]
    #[should_panic(expected = "expected identifier")]
    fn test_parse_ident_start_with_number() {
        let mut source: Peekable<Chars> = "1foo".chars().peekable();
        parse_ident(&mut source);
    }

    #[test]
    #[should_panic(expected = "expected identifier")]
    fn test_parse_ident_start_with_special_char() {
        let mut source: Peekable<Chars> = "@foo".chars().peekable();
        parse_ident(&mut source);
    }

    #[test]
    fn test_parse_ident_stops_at_special_char() {
        let mut source: Peekable<Chars> = "foo@".chars().peekable();
        let ident = parse_ident(&mut source);

        assert_eq!(ident, "foo");
    }

    #[test]
    fn test_parse_bound_ident_only_ident() {
        let mut source: Peekable<Chars> = "{foo}".chars().peekable();
        let (ident, inner_ident) = parse_bound_ident(&mut source);

        assert_eq!(ident, "foo");
        assert_eq!(inner_ident, None);
    }

    #[test]
    fn test_parse_bound_ident_with_inner_ident() {
        let mut source: Peekable<Chars> = "{foo:bar}".chars().peekable();
        let (ident, inner_ident) = parse_bound_ident(&mut source);

        assert_eq!(ident, "foo");
        assert_eq!(inner_ident, Some("bar".to_string()));
    }

    #[test]
    #[should_panic(expected = "expected : or }")]
    fn test_parse_bound_ident_with_invalid_char() {
        let mut source: Peekable<Chars> = "{foo|".chars().peekable();
        parse_bound_ident(&mut source);
    }

    #[test]
    #[should_panic(expected = "expected }")]
    fn test_parse_bound_ident_missing_closing_brace() {
        let mut source: Peekable<Chars> = "{foo:bar".chars().peekable();
        parse_bound_ident(&mut source);
    }

    #[test]
    #[should_panic(expected = "expected {")]
    fn test_parse_bound_ident_missing_opening_brace() {
        let mut source: Peekable<Chars> = "foo".chars().peekable();
        parse_bound_ident(&mut source);
    }
}
