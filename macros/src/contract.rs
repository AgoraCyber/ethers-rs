// Copyright 2015-2019 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Contract derive modify on the basis of [`ethabi`](https://github.com/rust-ethereum/ethabi/blob/master/derive/src/contract.rs)

use std::{env, fs, path::PathBuf};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemStruct, LitStr};

use crate::error::ToSynError;
use crate::gen::CodeGen;

mod constructor;
mod event;
mod function;
mod utils;

fn normalize_path(relative_path: LitStr) -> syn::Result<PathBuf> {
    // workaround for https://github.com/rust-lang/rust/issues/43860

    let cargo_toml_directory = env::var("CARGO_MANIFEST_DIR").map_err(|e| {
        syn::Error::new_spanned(
            relative_path.clone(),
            format!("load abi file failed, {}", e),
        )
    })?;

    let mut path: PathBuf = cargo_toml_directory.into();

    path.push(relative_path.value());

    Ok(path)
}

pub struct Contract {
    constructor: Option<constructor::Constructor>,
    functions: Vec<function::Function>,
    events: Vec<event::Event>,
}

impl Contract {
    pub fn new(item: ItemStruct) -> syn::Result<Contract> {
        let mut contract = None;

        for attr in &item.attrs {
            if let Some(path) = attr.path.get_ident() {
                let name = path.to_string();
                match name.as_str() {
                    "abi_file" => {
                        let abi_file: LitStr = attr.parse_args()?;

                        let path = normalize_path(abi_file.clone())?;

                        eprintln!("abi file, {:?}", path);

                        let source_file = fs::File::open(path).map_err(|e| {
                            syn::Error::new_spanned(attr, format!("load abi file failed, {}", e))
                        })?;

                        contract = Some(
                            ethabi::Contract::load(source_file).map_syn_error(abi_file.span())?,
                        );
                    }
                    "abi" => {
                        let abi_data = attr.parse_args::<LitStr>()?;

                        contract = Some(
                            ethabi::Contract::load(abi_data.value().as_bytes())
                                .map_syn_error(abi_data.span())?,
                        );
                    }
                    _ => {
                        continue;
                    }
                };
            }
        }

        #[allow(unused)]
        let contract = contract.ok_or(syn::Error::new_spanned(
            item,
            r#"Use #[abi_file("xx")]/#[abi("xxx")] to specify abi data"#,
        ))?;

        Ok(Self {
            constructor: contract.constructor.as_ref().map(Into::into),
            functions: contract.functions().map(Into::into).collect(),
            events: contract.events().map(Into::into).collect(),
        })
    }
}

impl CodeGen for Contract {
    fn gen_ir_code(&self) -> TokenStream {
        let constructor = self.constructor.as_ref().map(|c| c.gen_ir_code());
        let functions: Vec<_> = self.functions.iter().map(|f| f.gen_ir_code()).collect();
        let events: Vec<_> = self.events.iter().map(|e| e.generate_event()).collect();
        let logs: Vec<_> = self.events.iter().map(|e| e.generate_log()).collect();

        quote! {
            use ethabi;
            const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";

            #constructor

            /// Contract's functions.
            pub mod functions {
                use super::INTERNAL_ERR;
                #(#functions)*
            }

            /// Contract's events.
            pub mod events {
                use super::INTERNAL_ERR;
                #(#events)*
            }

            /// Contract's logs.
            pub mod logs {
                use super::INTERNAL_ERR;
                use ethabi;
                #(#logs)*
            }
        }
    }
}
