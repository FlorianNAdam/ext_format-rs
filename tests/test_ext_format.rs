#[cfg(test)]
mod tests {
    use ext_format::ext_format;
    use ext_format::ext_format_unindented;

    #[test]
    fn test_basic_interpolation() {
        let name = "Alice";
        let output = ext_format!("Hello, $name!");
        assert_eq!(output, "Hello, Alice!");
    }

    #[test]
    fn test_binding_new_variable_names() {
        let number = 42;
        let output = ext_format!("Number: ${number:n} $n $n");
        assert_eq!(output, "Number: 42 42 42");
    }

    #[test]
    fn test_basic_repetition() {
        let numbers = vec![1, 2, 3];
        let output = ext_format!("Numbers: $($numbers),*");
        assert_eq!(output, "Numbers: 1,2,3");
    }

    #[test]
    fn test_newline_as_separator() {
        let items = vec!["apple", "banana", "cherry"];
        let output = ext_format!("Items:\n$($items)(\n)*");
        assert_eq!(output, "Items:\napple\nbanana\ncherry");
    }

    #[test]
    fn test_repetition_with_hidden_variables() {
        let items = vec!["apple", "banana", "cherry"];
        let counter = vec![1, 2];
        let output = ext_format!("Items:\n$(@counter $items)\n*");
        assert_eq!(output, "Items:\n apple\n banana");
    }

    #[test]
    fn test_repetition_with_named_iteration_variables() {
        let numbers = vec![1, 2, 3];
        let output = ext_format!("Numbers: $(@{numbers:number} $number),*");
        assert_eq!(output, "Numbers:  1, 2, 3");
    }

    #[test]
    fn test_nested_repetitions() {
        let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let output = ext_format!("Matrix:\n$(@{matrix:row}$($row) *)(\n)*");
        assert_eq!(output, "Matrix:\n1 2 3\n4 5 6\n7 8 9");
    }

    #[test]
    fn test_zipped_variables() {
        let names = vec!["Alice", "Bob"];
        let ages = vec![30, 40];
        let output = ext_format!("Profiles:\n$($names $ages)\n*");
        assert_eq!(output, "Profiles:\nAlice 30\nBob 40");
    }

    #[test]
    fn test_unindented_multiline_strings() {
        let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let output = ext_format_unindented!(
            r#"
            void func3() {
                $(@{matrix:inner_matrix}printf("$($inner_matrix) *");)(\n    )*
            }
        "#
        );
        assert_eq!(output, "\nvoid func3() {\n    printf(\"1 2 3\");\n    printf(\"4 5 6\");\n    printf(\"7 8 9\");\n}\n        ");
    }
}
