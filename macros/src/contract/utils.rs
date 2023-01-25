use std::str::FromStr;

use ethers_hardhat_rs::ethabi;

use ethabi::{Param, ParamType};
use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

/// Convert ethabi parame_type to generic param type
pub fn template_param_type(input: &ParamType, index: usize) -> proc_macro2::TokenStream {
    let t_ident = syn::Ident::new(&format!("T{index}"), Span::call_site());
    let u_ident = syn::Ident::new(&format!("U{index}"), Span::call_site());
    match input {
        ParamType::Address => {
            quote! { #t_ident }
        }
        ParamType::Bytes => quote! { #t_ident },
        ParamType::FixedBytes(32) => quote! { #t_ident },
        ParamType::FixedBytes(_) => quote! { #t_ident },
        ParamType::Int(_) => quote! { #t_ident },
        ParamType::Uint(_) => quote! { #t_ident },
        ParamType::Bool => quote! { #t_ident },
        ParamType::String => quote! { #t_ident },
        ParamType::Array(_) => {
            quote! {
                #t_ident, #u_ident
            }
        }
        ParamType::FixedArray(_, _) => {
            quote! {
                #t_ident, #u_ident
            }
        }
        ParamType::Tuple(_) => {
            quote! {
                #t_ident
            }
        }
    }
}

/// Convert ethabi parame_type to generic param type
pub fn template_param_where_clause(input: &ParamType, index: usize) -> proc_macro2::TokenStream {
    let t_ident = syn::Ident::new(&format!("T{index}"), Span::call_site());
    let u_ident = syn::Ident::new(&format!("U{index}"), Span::call_site());
    match input {
        ParamType::Address => {
            quote! { #t_ident: TryInto<ethers_rs::ethabi::Address>, #t_ident::Error: std::fmt::Display + std::fmt::Debug }
        }
        ParamType::Bytes => {
            quote! { #t_ident: TryInto<ethers_rs::ethabi::Bytes>, #t_ident::Error: std::fmt::Display + std::fmt::Debug }
        }
        ParamType::FixedBytes(32) => {
            quote! { #t_ident: TryInto<ethers_rs::ethabi::Hash>, #t_ident::Error: std::fmt::Display + std::fmt::Debug }
        }
        ParamType::FixedBytes(size) => {
            quote! { #t_ident: TryInto<[u8; #size]> , #t_ident::Error: std::fmt::Display + std::fmt::Debug}
        }
        ParamType::Int(_) => {
            quote! { #t_ident: TryInto<ethers_rs::ethabi::Int> , #t_ident::Error: std::fmt::Display + std::fmt::Debug}
        }
        ParamType::Uint(_) => {
            quote! { #t_ident: TryInto<ethers_rs::ethabi::Uint> , #t_ident::Error: std::fmt::Display + std::fmt::Debug}
        }
        ParamType::Bool => {
            quote! { #t_ident: TryInto<bool> , #t_ident::Error: std::fmt::Display + std::fmt::Debug}
        }
        ParamType::String => {
            quote! { #t_ident: TryInto<String> , #t_ident::Error: std::fmt::Display + std::fmt::Debug}
        }
        ParamType::Array(kind) => {
            let t = rust_type(kind);
            quote! {
                #t_ident: IntoIterator<Item = #u_ident>, #u_ident: Into<#t>
            }
        }
        ParamType::FixedArray(ref kind, size) => {
            let t = rust_type(kind);
            quote! {
                #t_ident: Into<[#u_ident; #size]>, #u_ident: Into<#t>
            }
        }
        ParamType::Tuple(ref types) => {
            let types = types.iter().map(|t| rust_type(t)).collect::<Vec<_>>();
            quote! {
                #t_ident: Into<(#(#types,)*)>
            }
        }
    }
}

pub fn rust_type(input: &ParamType) -> proc_macro2::TokenStream {
    match *input {
        ParamType::Address => quote! { ethabi::Address },
        ParamType::Bytes => quote! { ethabi::Bytes },
        ParamType::FixedBytes(32) => quote! { ethabi::Hash },
        ParamType::FixedBytes(size) => quote! { [u8; #size] },
        ParamType::Int(_) => quote! { ethabi::Int },
        ParamType::Uint(_) => quote! { ethabi::Uint },
        ParamType::Bool => quote! { bool },
        ParamType::String => quote! { String },
        ParamType::Array(ref kind) => {
            let t = rust_type(kind);
            quote! { Vec<#t> }
        }
        ParamType::FixedArray(ref kind, size) => {
            let t = rust_type(kind);
            quote! { [#t, #size] }
        }
        ParamType::Tuple(ref types) => {
            let types = types.iter().map(|t| rust_type(t)).collect::<Vec<_>>();

            quote!((#(#types,)*))
        }
    }
}

pub fn input_names(inputs: &[Param]) -> Vec<syn::Ident> {
    inputs
        .iter()
        .enumerate()
        .map(|(index, param)| {
            if param.name.is_empty() {
                syn::Ident::new(&format!("param{index}"), Span::call_site())
            } else {
                syn::Ident::new(&rust_variable(&param.name), Span::call_site())
            }
        })
        .collect()
}

/// Convert input into a rust variable name.
///
/// Avoid using keywords by escaping them.
pub fn rust_variable(name: &str) -> String {
    // avoid keyword parameters
    match name {
        "self" => "r#self".to_string(),
        other => other.to_snake_case(),
    }
}

pub fn from_template_param(input: &ParamType, name: &syn::Ident) -> proc_macro2::TokenStream {
    match *input {
        ParamType::Array(_) => quote! { #name.into_iter().map(Into::into).collect::<Vec<_>>() },
        ParamType::FixedArray(_, _) => {
            quote! { (Box::new(#name.into()) as Box<[_]>).into_vec().into_iter().map(Into::into).collect::<Vec<_>>() }
        }
        _ => quote! { #name.try_into().map_err(ethers_rs::custom_error)? },
    }
}

pub fn to_token(name: &proc_macro2::TokenStream, kind: &ParamType) -> proc_macro2::TokenStream {
    match *kind {
        ParamType::Address => quote! { ethabi::Token::Address(#name) },
        ParamType::Bytes => quote! { ethabi::Token::Bytes(#name) },
        ParamType::FixedBytes(_) => quote! { ethabi::Token::FixedBytes(#name.as_ref().to_vec()) },
        ParamType::Int(_) => quote! { ethabi::Token::Int(#name) },
        ParamType::Uint(_) => quote! { ethabi::Token::Uint(#name) },
        ParamType::Bool => quote! { ethabi::Token::Bool(#name) },
        ParamType::String => quote! { ethabi::Token::String(#name) },
        ParamType::Array(ref kind) => {
            let inner_name = quote! { inner };
            let inner_loop = to_token(&inner_name, kind);
            quote! {
                // note the double {{
                {
                    let v = #name.into_iter().map(|#inner_name| #inner_loop).collect();
                    ethabi::Token::Array(v)
                }
            }
        }
        ParamType::FixedArray(ref kind, _) => {
            let inner_name = quote! { inner };
            let inner_loop = to_token(&inner_name, kind);
            quote! {
                // note the double {{
                {
                    let v = #name.into_iter().map(|#inner_name| #inner_loop).collect();
                    ethabi::Token::FixedArray(v)
                }
            }
        }
        ParamType::Tuple(ref types) => {
            let streams = types
                .iter()
                .enumerate()
                .map(|(index, t)| {
                    let index2 = TokenStream::from_str(&format!("{}", index)).unwrap();
                    let name = quote! { v.#index2 };

                    let stream = to_token(&name, t);

                    let ident = format_ident!("v_{}", index, span = Span::call_site());
                    (
                        quote! {
                            let #ident = #stream;
                        },
                        quote!(#ident),
                    )
                })
                .collect::<Vec<_>>();

            let to_token_streams = streams
                .iter()
                .map(|(to_token, _)| to_token)
                .collect::<Vec<_>>();

            let idents = streams.iter().map(|(_, ident)| ident).collect::<Vec<_>>();

            let types = types.iter().map(|t| rust_type(t)).collect::<Vec<_>>();

            quote! {

                {
                    let v: (#(#types,)*) = #name;

                    #(#to_token_streams)*

                    ethabi::Token::Tuple(vec![#(#idents,)*])
                }
            }
        }
    }
}

pub fn from_token(kind: &ParamType, token: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    match *kind {
        ParamType::Address => quote! { #token.into_address().expect(INTERNAL_ERR) },
        ParamType::Bytes => quote! { #token.into_bytes().expect(INTERNAL_ERR) },
        ParamType::FixedBytes(32) => quote! {
            {
                let mut result = [0u8; 32];
                let v = #token.into_fixed_bytes().expect(INTERNAL_ERR);
                result.copy_from_slice(&v);
                ethabi::Hash::from(result)
            }
        },
        ParamType::FixedBytes(size) => {
            let size: syn::Index = size.into();
            quote! {
                {
                    let mut result = [0u8; #size];
                    let v = #token.into_fixed_bytes().expect(INTERNAL_ERR);
                    result.copy_from_slice(&v);
                    result
                }
            }
        }
        ParamType::Int(_) => quote! { #token.into_int().expect(INTERNAL_ERR) },
        ParamType::Uint(_) => quote! { #token.into_uint().expect(INTERNAL_ERR) },
        ParamType::Bool => quote! { #token.into_bool().expect(INTERNAL_ERR) },
        ParamType::String => quote! { #token.into_string().expect(INTERNAL_ERR) },
        ParamType::Array(ref kind) => {
            let inner = quote! { inner };
            let inner_loop = from_token(kind, &inner);
            quote! {
                #token.into_array().expect(INTERNAL_ERR).into_iter()
                    .map(|#inner| #inner_loop)
                    .collect()
            }
        }
        ParamType::FixedArray(ref kind, size) => {
            let inner = quote! { inner };
            let inner_loop = from_token(kind, &inner);
            let to_array = vec![quote! { iter.next() }; size];
            quote! {
                {
                    let iter = #token.to_array().expect(INTERNAL_ERR).into_iter()
                        .map(|#inner| #inner_loop);
                    [#(#to_array),*]
                }
            }
        }
        ParamType::Tuple(ref types) => {
            let streams = types
                .iter()
                .enumerate()
                .map(|(index, t)| {
                    let index2 = TokenStream::from_str(&format!("{}", index)).unwrap();

                    let token = quote! {
                        v[#index2].clone()
                    };

                    let stream = from_token(t, &token);

                    let ident = format_ident!("v_{}", index, span = Span::call_site());
                    (
                        quote! {
                            let #ident = #stream;
                        },
                        quote!(#ident),
                    )
                })
                .collect::<Vec<_>>();

            let to_token_streams = streams
                .iter()
                .map(|(to_token, _)| to_token)
                .collect::<Vec<_>>();

            let idents = streams.iter().map(|(_, ident)| ident).collect::<Vec<_>>();

            quote! {

                {
                    let v = #token.into_tuple().expect(INTERNAL_ERR);

                    #(#to_token_streams)*

                    (#(#idents,)*)
                }
            }
        }
    }
}

pub fn get_template_names(kinds: &[proc_macro2::TokenStream]) -> Vec<syn::Ident> {
    kinds
        .iter()
        .enumerate()
        .map(|(index, _)| syn::Ident::new(&format!("T{index}"), Span::call_site()))
        .collect()
}

pub fn get_output_kinds(outputs: &[Param]) -> proc_macro2::TokenStream {
    match outputs.len() {
        0 => quote! {()},
        1 => {
            let t = rust_type(&outputs[0].kind);
            quote! { #t }
        }
        _ => {
            let outs: Vec<_> = outputs.iter().map(|param| rust_type(&param.kind)).collect();
            quote! { (#(#outs),*) }
        }
    }
}

pub fn to_ethabi_param_vec<'a, P: 'a>(params: P) -> proc_macro2::TokenStream
where
    P: IntoIterator<Item = &'a Param>,
{
    let p = params
        .into_iter()
        .map(|x| {
            let name = &x.name;
            let kind = to_syntax_string(&x.kind);
            quote! {
                ethers_rs::ethabi::Param {
                    name: #name.to_owned(),
                    kind: #kind,
                    internal_type: None
                }
            }
        })
        .collect::<Vec<_>>();

    quote! { vec![ #(#p),* ] }
}

pub fn to_syntax_string(param_type: &ethabi::ParamType) -> proc_macro2::TokenStream {
    match *param_type {
        ParamType::Address => quote! { ethabi::ParamType::Address },
        ParamType::Bytes => quote! { ethabi::ParamType::Bytes },
        ParamType::Int(x) => quote! { ethabi::ParamType::Int(#x) },
        ParamType::Uint(x) => quote! { ethabi::ParamType::Uint(#x) },
        ParamType::Bool => quote! { ethabi::ParamType::Bool },
        ParamType::String => quote! { ethabi::ParamType::String },
        ParamType::Array(ref param_type) => {
            let param_type_quote = to_syntax_string(param_type);
            quote! { ethabi::ParamType::Array(Box::new(#param_type_quote)) }
        }
        ParamType::FixedBytes(x) => quote! { ethabi::ParamType::FixedBytes(#x) },
        ParamType::FixedArray(ref param_type, ref x) => {
            let param_type_quote = to_syntax_string(param_type);
            quote! { ethabi::ParamType::FixedArray(Box::new(#param_type_quote), #x) }
        }
        ParamType::Tuple(ref types) => {
            let streams = types
                .iter()
                .enumerate()
                .map(|(index, t)| {
                    let stream = to_syntax_string(t);

                    let ident = format_ident!("v_{}", index, span = Span::call_site());
                    (
                        quote! {
                            let #ident = #stream;
                        },
                        quote!(#ident),
                    )
                })
                .collect::<Vec<_>>();

            let to_token_streams = streams
                .iter()
                .map(|(to_token, _)| to_token)
                .collect::<Vec<_>>();

            let idents = streams.iter().map(|(_, ident)| ident).collect::<Vec<_>>();

            quote! {

                {
                    #(#to_token_streams)*

                    ethers_rs::ethabi::ParamType::Tuple(vec![#(#idents,)*])
                }
            }
        }
    }
}
