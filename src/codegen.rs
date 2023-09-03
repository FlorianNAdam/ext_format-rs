use crate::parse::QuoteToken;
use proc_macro2::Ident;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};

fn get_macro_definitions() -> TokenStream {
    quote!(
        #[allow(unused_macros)]
        macro_rules! nested_tuple {
                (@inner [] $($r:tt)*) => {$($r),*};
                (@inner [$i:tt|$($r:tt|)*] $($rev:tt)*) => (
                    nested_tuple!(@inner [$($r|)*] ($($rev)*, $i))
                );
                ($i:tt) => ($i);
                ($i:tt, $($r:tt),* $(,)?) => (
                    nested_tuple!(@inner [$($r|)*] $i)
                )
            }

        #[allow(unused_macros)]
        macro_rules! fizip {
                (@inner [] $($a:tt)*) => ($($a)*);
                (@inner [$i:expr => $($r:expr => )*] $($a:tt)*) => (
                    fizip!(@inner [$($r => )*] $($a)*.zip($i))
                );
                ($i:expr) => ($i);
                ($i:expr, $($r:expr),* $(,)?) => (
                    fizip!(@inner [$($r => )*] $i)
                )
            }
    )
    .into()
}

pub(crate) fn generate_code(tokens: Vec<QuoteToken>) -> TokenStream {
    let mut rust_tokens: Vec<TokenStream> = vec![];
    for token in generate_inner_code(tokens, HashMap::new()) {
        rust_tokens.push(token.into());
    }
    let inner_stream: TokenStream = TokenStream::from_iter(rust_tokens).into();

    let macro_tokens = get_macro_definitions();

    quote!({
        #macro_tokens

        let mut res = String::new();
        #inner_stream
        res
    })
    .into()
}

fn generate_inner_code(
    tokens: Vec<QuoteToken>,
    mut mapping: HashMap<String, String>,
) -> TokenStream {
    let mut rust_tokens: Vec<TokenStream> = vec![];
    for token in tokens {
        let new_tokens = match token {
            QuoteToken::Literal(literal) => generate_literal_code(literal),
            QuoteToken::Variable(ident, inner_ident) => {
                generate_variable_code(ident, inner_ident, &mut mapping)
            }
            QuoteToken::HiddenVariable(ident, inner_ident) => {
                generate_hidden_variable_code(ident, inner_ident, &mut mapping)
            }
            QuoteToken::Group(tokens, separator) => generate_group_code(tokens, separator),
        };
        rust_tokens.push(new_tokens);
    }
    TokenStream::from_iter(rust_tokens)
}

fn generate_literal_code(literal: String) -> TokenStream {
    let new_tokens = quote!(res.push_str(#literal););
    new_tokens.into()
}

fn generate_variable_code(
    ident: String,
    inner_ident: Option<String>,
    mapping: &mut HashMap<String, String>,
) -> TokenStream {
    let new_name = mapping.get(&ident).unwrap_or(&ident);
    let var_ident = Ident::new(new_name, Span::call_site());
    let new_tokens = if let Some(inner_ident) = inner_ident {
        let inner_var_ident = Ident::new(&inner_ident, Span::call_site());
        quote!(
            let #inner_var_ident = #var_ident;
            res.push_str(&#inner_var_ident.to_string());
        )
    } else {
        quote!(
            res.push_str(&#var_ident.to_string());
        )
    };
    new_tokens.into()
}

fn generate_hidden_variable_code(
    ident: String,
    inner_ident: Option<String>,
    mapping: &mut HashMap<String, String>,
) -> TokenStream {
    let new_name = mapping.get(&ident).unwrap_or(&ident);
    let var_ident = Ident::new(new_name, Span::call_site());
    if let Some(inner_ident) = inner_ident {
        if inner_ident != "_" {
            let inner_var_ident = Ident::new(&inner_ident, Span::call_site());
            return quote!(
                let #inner_var_ident = #var_ident;
            );
        }
    }
    TokenStream::new()
}

fn get_variable_names(tokens: &[QuoteToken]) -> Vec<(String, String)> {
    let mut variables = vec![];
    let mut inner_variables = HashSet::new();
    for token in tokens.iter() {
        let (variable, inner) = match token {
            QuoteToken::Variable(ref variable, ref inner) => (variable, inner),
            QuoteToken::HiddenVariable(ref variable, ref inner) => (variable, inner),
            _ => continue,
        };
        if !inner_variables.contains(variable) {
            if let Some(inner) = inner {
                inner_variables.insert(inner);
                variables.push((variable.clone(), inner.clone()))
            } else {
                let inner_name = "__ext_format_inner_".to_string() + variable;
                variables.push((variable.clone(), inner_name))
            }
        }
    }
    variables
}

fn generate_group_code(tokens: Vec<QuoteToken>, separator: Option<String>) -> TokenStream {
    let variables = get_variable_names(&tokens);

    let mut mapping = HashMap::new();
    let mut idents = vec![];
    let mut inner_idents = vec![];

    for (variable, inner) in variables.iter() {
        mapping.insert(variable.clone(), inner.clone());
        idents.push(Ident::new(&variable, Span::call_site()));
        inner_idents.push(Ident::new(&inner, Span::call_site()));
    }

    let token_stream: TokenStream = generate_inner_code(tokens, mapping).into();

    let separator_stream = if let Some(separator) = separator {
        quote!(
            if i < iterator.len() - 1 {
                res.push_str(#separator);
            }
        )
    } else {
        TokenStream::new().into()
    };

    quote!(
        let mut iterator = fizip!(#(#idents.iter()),*).collect::<Vec<_>>();
        if !iterator.is_empty() {
            for (i, nested_tuple!(#(#inner_idents),*)) in iterator.iter().enumerate() {
                #token_stream
                #separator_stream
            }
        };
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::QuoteToken::*;
    use super::*;
    use crate::util::unindent;

    #[test]
    fn test_generate_inner_code_literal() {
        let tokens = vec![Literal("Hello".to_string())];
        let output = generate_inner_code(tokens, HashMap::new());
        let output_str = output.to_string();

        assert_eq!(output_str, r#"res . push_str ("Hello") ;"#);
    }

    #[test]
    fn test_generate_inner_code_variable() {
        let mut mapping = HashMap::new();
        mapping.insert("var".to_string(), "var_mapped".to_string());

        let tokens = vec![Variable("var".to_string(), None)];
        let output = generate_inner_code(tokens, mapping);
        let output_str = output.to_string();

        assert_eq!(
            output_str,
            r#"res . push_str (& var_mapped . to_string ()) ;"#
        );
    }

    #[test]
    fn test_generate_inner_code_hidden_variable() {
        let tokens = vec![HiddenVariable("var".to_string(), None)];
        let output = generate_inner_code(tokens, HashMap::new());
        let output_str = output.to_string();

        assert_eq!(output_str, "");
    }

    #[test]
    fn test_generate_inner_code_group_with_separator() {
        let mut mapping = HashMap::new();
        mapping.insert("var".to_string(), "var_mapped".to_string());

        let group_tokens = vec![
            Literal("Literal".to_string()),
            Variable("var".to_string(), None),
        ];

        let tokens = vec![Group(group_tokens, Some(",".to_string()))];

        let output = generate_inner_code(tokens, mapping);
        let output_str = output.to_string();

        let expected = unindent(
            r#"
            let mut iterator = fizip ! (var . iter ()) . collect :: < Vec < _ >> () ;
            @ if ! iterator . is_empty () { 
            @for (i , nested_tuple ! (__ext_format_inner_var)) in iterator . iter () . enumerate () {
            @ res . push_str ("Literal") ;
            @ res . push_str (& __ext_format_inner_var . to_string ()) ;
            @ if i < iterator . len () - 1 { res . push_str (",") ; } } } ;
        "#,
        ).trim().replace("\n@", "");

        assert_eq!(output_str, expected);
    }

    #[test]
    fn test_generate_inner_code_group_without_separator() {
        let mut mapping = HashMap::new();
        mapping.insert("var".to_string(), "var_mapped".to_string());

        let group_tokens = vec![
            Literal("Literal".to_string()),
            Variable("var".to_string(), None),
        ];

        let tokens = vec![Group(group_tokens, None)];

        let output = generate_inner_code(tokens, mapping);
        let output_str = output.to_string();

        let expected = unindent(
            r#"
            let mut iterator = fizip ! (var . iter ()) . collect :: < Vec < _ >> () ;
            @ if ! iterator . is_empty () { 
            @for (i , nested_tuple ! (__ext_format_inner_var)) in iterator . iter () . enumerate () {
            @ res . push_str ("Literal") ;
            @ res . push_str (& __ext_format_inner_var . to_string ()) ; } } ;
        "#,
        ).trim().replace("\n@", "");

        assert_eq!(output_str, expected);
    }

    #[test]
    fn test_generate_code_group_with_hidden_variable() {
        let mut mapping = HashMap::new();
        mapping.insert("var1".to_string(), "mapped_var1".to_string());
        mapping.insert("hidden_var".to_string(), "_".to_string());

        let group = Group(
            vec![
                Literal("A".to_string()),
                Variable("var1".to_string(), Some("mapped_var1".to_string())),
                HiddenVariable("hidden_var".to_string(), Some("_".to_string())),
            ],
            Some(", ".to_string()),
        );

        let output = generate_inner_code(vec![group], mapping);
        let output_str = output.to_string();

        let expected = unindent(
            r#"
            let mut iterator = fizip ! (var1 . iter () , hidden_var . iter ()) . collect :: < Vec < _ >> () ;
            @ if ! iterator . is_empty () { for (i , nested_tuple ! (mapped_var1 , _)) in iterator . iter () . enumerate () { res . push_str ("A") ;
            @ let mapped_var1 = mapped_var1 ;
            @ res . push_str (& mapped_var1 . to_string ()) ;
            @ if i < iterator . len () - 1 { res . push_str (", ") ; } } } ;"#,
        ).trim().replace("\n@", "");

        assert_eq!(output_str, expected);
    }
}
