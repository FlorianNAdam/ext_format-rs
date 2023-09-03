/// Calculates the minimum indentation level of a multiline string.
///
/// This function scans each line in the input string to find the line with the least
/// amount of leading whitespace, ignoring lines that only contain whitespace.
///
fn get_indent_level(source: &str) -> usize {
    let mut min_indent = usize::MAX;
    for line in source.split("\n") {
        if line.trim() == "" {
            continue;
        }
        let indent = line
            .chars()
            .take_while(|ch| ch.is_whitespace() && *ch != '\n')
            .map(|ch| ch.len_utf8())
            .sum();
        if indent < min_indent {
            min_indent = indent;
        }
        if min_indent == 0 {
            break;
        }
    }
    min_indent
}

/// Unindents a multi-line string by removing a uniform level of indentation from each line.
///
pub(crate) fn unindent(source: &str) -> String {
    let indent = get_indent_level(&source);

    let mut res = String::new();
    let split = source.split("\n").collect::<Vec<_>>();
    for (n, line) in split.iter().enumerate() {
        if line.len() > indent {
            res.push_str(&line[indent..]);
        } else {
            res.push_str(line);
        }
        if n < split.len() - 1 {
            res.push_str("\n");
        }
    }
    res
}

/// Converts two hexadecimal characters to a single `char`.
///
/// Takes two `char`s representing hexadecimal digits and returns their combined
/// byte value as a `char`.
///
fn byte_from_hex_chars(first_hex: char, second_hex: char) -> char {
    let ordinal = format!("{}{}", first_hex, second_hex);
    let byte = u8::from_str_radix(&ordinal, 16).unwrap();
    byte as char
}

/// Unescapes a string by converting escape sequences to their character representations.
///
/// Recognizes common escape sequences like `\\`, `\n`, `\r`, and `\t`. Also supports
/// hexadecimal escapes in the form of `\xHH` where `H` is a hexadecimal digit.
///
pub(crate) fn unescape(s: &str) -> String {
    let mut res = String::new();

    let mut chars = s.chars();
    while let Some(char) = chars.next() {
        if '\\' == char {
            if let Some(next_char) = chars.next() {
                match next_char {
                    '\\' => res.push('\\'),
                    'n' => res.push('\n'),
                    'r' => res.push('\r'),
                    't' => res.push('\t'),
                    'x' => match (chars.next(), chars.next()) {
                        (Some(first_hex), Some(second_hex)) => {
                            if first_hex.is_ascii_hexdigit() && second_hex.is_ascii_hexdigit() {
                                res.push(byte_from_hex_chars(first_hex, second_hex));
                            } else {
                                res.push_str(&format!(r"\x{}{}", first_hex, second_hex));
                            }
                        }
                        (Some(first_hex), None) => res.push_str(&format!(r"\x{}", first_hex)),
                        (_, _) => res.push_str(&format!(r"\x")),
                    },
                    c => res.push_str(&format!(r"\{}", c)),
                }
            } else {
                res.push(char);
            }
        } else {
            res.push(char);
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::unescape;
    use super::unindent;

    #[test]
    fn test_unindent_basic() {
        let original = "    Line1\n    Line2";
        let expected = "Line1\nLine2";
        assert_eq!(unindent(original), expected);
    }

    #[test]
    fn test_unindent_mixed_indent() {
        let original = "    Line1\n  Line2";
        let expected = "  Line1\nLine2";
        assert_eq!(unindent(original), expected);
    }

    #[test]
    fn test_unindent_no_indent() {
        let original = "Line1\nLine2";
        let expected = "Line1\nLine2";
        assert_eq!(unindent(original), expected);
    }

    #[test]
    fn test_unindent_empty_string() {
        let original = "";
        let expected = "";
        assert_eq!(unindent(original), expected);
    }

    #[test]
    fn test_unindent_single_line() {
        let original = "    Line1";
        let expected = "Line1";
        assert_eq!(unindent(original), expected);
    }

    #[test]
    fn test_unindent_new_lines_only() {
        let original = "\n\n\n";
        let expected = "\n\n\n";
        assert_eq!(unindent(original), expected);
    }
    #[test]
    fn test_unindent_empty_lines() {
        let original = "    Line1\n  \n  Line2";
        let expected = "  Line1\n  \nLine2";
        assert_eq!(unindent(original), expected);
    }
    #[test]
    fn test_unescape_empty_string() {
        assert_eq!(unescape(""), "");
    }

    #[test]
    fn test_unescape_no_escape_characters() {
        assert_eq!(unescape("hello"), "hello");
    }

    #[test]
    fn test_unescape_newline() {
        assert_eq!(unescape("hello\\nworld"), "hello\nworld");
    }

    #[test]
    fn test_unescape_tab() {
        assert_eq!(unescape("hello\\tworld"), "hello\tworld");
    }

    #[test]
    fn test_unescape_return() {
        assert_eq!(unescape("hello\\rworld"), "hello\rworld");
    }

    #[test]
    fn test_unescape_backslash() {
        assert_eq!(unescape("hello\\\\world"), "hello\\world");
    }

    #[test]
    fn test_unescape_hex() {
        assert_eq!(unescape("hello\\x41world"), "helloAworld");
    }

    #[test]
    fn test_unescape_hex_lowercase() {
        assert_eq!(unescape("hello\\x41world"), "helloAworld");
        assert_eq!(unescape("hello\\x41world"), "helloAworld");
    }

    #[test]
    fn test_invalid_first_hex() {
        assert_eq!(unescape("hello\\xg1world"), "hello\\xg1world");
    }

    #[test]
    fn test_invalid_second_hex() {
        assert_eq!(unescape("hello\\x4gworld"), "hello\\x4gworld");
    }

    #[test]
    fn test_unescape_incomplete_escape() {
        assert_eq!(unescape("hello\\world"), "hello\\world");
        assert_eq!(unescape("hello\\"), "hello\\");
        assert_eq!(unescape("hello\\x"), "hello\\x");
        assert_eq!(unescape("hello\\x4"), "hello\\x4");
    }

    #[test]
    fn test_unescape_mixed() {
        assert_eq!(
            unescape("hello\\nworld\\tfoo\\rbar\\\\\\x41\\x42"),
            "hello\nworld\tfoo\rbar\\AB"
        );
    }

    #[test]
    fn test_unescape_trailing_backslash() {
        assert_eq!(unescape("hello\\"), "hello\\");
    }

    #[test]
    fn test_unescape_trailing_hex() {
        assert_eq!(unescape("hello\\x"), "hello\\x");
    }
}
