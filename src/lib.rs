//! # ext_format
//!
//! A small, yet powerful, Rust crate for string interpolation. Inspired by Rust's macro rules, it provides two main macros: `ext_format!` and `ext_format_unindent!` \
//! The `ext_format_unindent!` macro works exactly like `ext_format!`, but first trims leading whitespace, to make working with multiline strings less painful.
//!
//! ## Installation
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! ext_format = "0.1.0"
//! ```
//!
//! ## Usage
//!
//! ### Basic Interpolation
//!
//! Use `$` for basic interpolation:
//!
//! ```rust
//! # use ext_format::ext_format;
//! let name = "Alice";
//! let output = ext_format!("Hello, $name!");
//! ```
//!
//! ### Binding new variable names
//! Use `{name:new_name}` to bind a new name to a variable.
//!
//! ```rust
//! # use ext_format::ext_format;
//! let number = 42;
//! let output = ext_format!("Number: ${number:n} $n $n");
//! // Output: "Number: 42 42 42"
//! ```
//!
//! This syntax can also be used to avoid unnecessary spaces in the output:
//! ```rust
//! # use ext_format::ext_format;
//! let number = 42;
//! let output = ext_format!("Number: ${number}${number}${number}");
//! // Output: "Number: 424242"
//! ```
//!
//! ### Basic Repetition
//!
//! - `$($var)*`: No separators
//! - `$($var),*`: Character as a separator
//! - `$($var)(...)*`: String as a separator (supports escaped characters)
//!
//! ```rust
//! # use ext_format::ext_format;
//! let numbers = vec![1, 2, 3];
//! let output = ext_format!("Numbers: $($numbers),*");
//! // Output: "Numbers: 1, 2, 3"
//! ```
//!
//! For using newlines as separators:
//!
//! ```rust
//! # use ext_format::ext_format;
//! let items = vec!["apple", "banana", "cherry"];
//! let output = ext_format!("Items:\n$($items)(\n)*");
//! // Output:
//! // Items:
//! // apple
//! // banana
//! // cherry
//! ```
//!
//! ### Repetition with Hidden Variables
//!
//! Use `@` to include variables that control the loop but aren't included in the output.
//!
//! ```rust
//! # use ext_format::ext_format;
//! let counter = vec![1, 2];
//! let output = ext_format!("Lines:\n$(@counter line)\n*");
//! // Output:
//! // Lines:
//! //  line
//! //  line
//! ```
//!
//! ### Repetition with named Iteration Variables
//!
//! Use `{name:new_name}` to bind a Name to a Variable.
//!
//! ```rust
//! # use ext_format::ext_format;
//! let numbers = vec![1, 2, 3];
//! let output = ext_format!("Numbers: $(@{numbers:number} $number),*");
//! // Output: "Numbers: 1, 2, 3"
//! ```
//!
//! ### Nested Repetitions
//!
//! Repetitions can contain other repetitions, acting like nested for-loops:
//!
//! ```rust
//! # use ext_format::ext_format;
//! let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
//! let output = ext_format!("Matrix:\n$(@{matrix:row}$($row) *)(\n)*");
//! // Output:
//! // Matrix:
//! // 1 2 3
//! // 4 5 6
//! // 7 8 9
//! ```
//!
//! ### Zipped Variables
//!
//! Variables in a single repetition layer are automatically zipped together, meaning they iterate in lockstep.
//!
//! ```rust
//! # use ext_format::ext_format;
//! let names = vec!["Alice", "Bob"];
//! let ages = vec![30, 40];
//! let output = ext_format!("Profiles:\n$($names $ages)\n*");
//! // Profiles:
//! // Alice 30
//! // Bob 40
//! ```
//!
//! ### Multiline Strings
//!
//! For multiline strings, `ext_format_unindented` can be used to avoid leading whitespace:
//!
//! ```rust
//! # use ext_format::ext_format_unindented;
//! fn unindented() -> String {
//!     let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
//!     ext_format_unindented!(r#"
//!         void func3() {
//!             $(@{matrix:inner_matrix}printf("$($inner_matrix) *");)(\n    )*
//!         }
//!     "#)
//! }
//! let output = unindented();
//! // Output:
//! // void func3() {
//! //     printf("1 2 3");
//! //     printf("4 5 6");
//! //     printf("7 8 9");
//! // }
//! ```
//!
//! If the regular `ext_format` was used here, it would result in the following:
//! ```rust
//! # use ext_format::ext_format;
//! fn indented() -> String {
//!     let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
//!     ext_format!(r#"
//!         void func3() {
//!             $(@{matrix:inner_matrix}printf("$($inner_matrix) *");)(\n                    )*
//!         }
//!     "#)
//! }
//! let output = indented();
//! // Output:
//! //                void func3() {
//! //                    printf("1 2 3");
//! //                    printf("4 5 6");
//! //                    printf("7 8 9");
//! //                }
//! ```
//! With the indentation of the resulting string depending on the indentation of the function itself.
//!
//! ## License
//!
//! This project is licensed under the MIT License. See the [LICENSE.md](LICENSE.md) file for details.

extern crate core;

use proc_macro::{TokenStream, TokenTree};

mod codegen;
mod parse;
mod util;

use crate::codegen::generate_code;
use crate::parse::parse;
use crate::util::{unescape, unindent};

fn process(source: String) -> TokenStream {
    let unescaped_source = unescape(&source);
    let tokens = parse(&unescaped_source);
    let rust_code = generate_code(tokens);
    rust_code.into()
}

fn get_string_literal(tokens: TokenStream) -> String {
    let tokens: Vec<TokenTree> = tokens.into_iter().collect();

    if let (Some(token), true) = (tokens.get(0), tokens.len() == 1) {
        if let litrs::Literal::String(literal_string) = litrs::Literal::try_from(token).unwrap() {
            return literal_string.value().to_string();
        } else {
            panic!("invalid format");
        }
    } else {
        panic!("invalid format");
    };
}

#[proc_macro]
pub fn ext_format(input: TokenStream) -> TokenStream {
    let literal = get_string_literal(input);
    let res = process(literal);
    res
}

#[proc_macro]
pub fn ext_format_unindented(input: TokenStream) -> TokenStream {
    let literal = get_string_literal(input);
    let unindented = unindent(&literal);
    let res = process(unindented);
    res
}
