#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

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
