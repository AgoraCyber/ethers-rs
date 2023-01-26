// Copyright 2015-2019 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Contract derive modify on the basis of [`ethabi`](https://github.com/rust-ethereum/ethabi/blob/master/derive/src/contract.rs)

use std::{env, fs, path::PathBuf};

use ethers_hardhat_rs::ethabi;
use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::{LitStr, Token};

use crate::error::ToSynError;
use crate::gen::CodeGen;

mod constructor;
mod event;
mod function;
mod utils;

fn normalize_path<P>(span: Span, relative_path: P) -> syn::Result<PathBuf>
where
    P: Into<PathBuf>,
{
    let relative_path = relative_path.into();

    let cargo_toml_directory = env::var("CARGO_MANIFEST_DIR")
        .map_err(|e| syn::Error::new(span, format!("load abi file failed, {}", e)))?;

    let mut path: PathBuf = cargo_toml_directory.into();

    path.push(relative_path);

    Ok(path)
}

pub struct Contract {
    #[allow(unused)]
    ident: Ident,
    constructor: Option<constructor::Constructor>,
    functions: Vec<function::Function>,
    events: Vec<event::Event>,
    bytecode: Option<String>,
}

impl Parse for Contract {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let rhs: Option<Token!(,)> = input.parse()?;

        if rhs.is_some() {
            let source: Ident = input.parse()?;
            let _: Token!(=) = input.parse()?;
            let path_or_data: LitStr = input.parse()?;

            match source.to_string().as_str() {
                "file" => {
                    let path = normalize_path(path_or_data.span(), path_or_data.clone().value())?;

                    let source_file = fs::File::open(path).map_err(|e| {
                        syn::Error::new(path_or_data.span(), format!("load abi file failed, {}", e))
                    })?;

                    Self::new_with_contract(
                        ident,
                        ethabi::Contract::load(source_file).map_syn_error(path_or_data.span())?,
                        None,
                    )
                }
                "data" => Self::new_with_contract(
                    ident,
                    ethabi::Contract::load(path_or_data.value().as_bytes())
                        .map_syn_error(path_or_data.span())?,
                    None,
                ),
                _ => Err(syn::Error::new(
                    source.span(),
                    "invalid source, expect file/data",
                )),
            }
        } else {
            let path: PathBuf =
                format!("sol/artifacts/contracts/{}.sol/{}.json", ident, ident).into();

            let path = normalize_path(ident.span(), path)?;

            let source_file = fs::File::open(path).map_err(|e| {
                syn::Error::new(ident.span(), format!("load abi file failed, {}", e))
            })?;

            let artifact =
                ethabi::HardhatArtifact::load(source_file).map_syn_error(ident.span())?;

            Self::new_with_contract(ident, artifact.abi, Some(artifact.bytecode))
        }
    }
}

impl Contract {
    pub fn new_with_contract(
        ident: Ident,
        contract: ethabi::Contract,
        bytecode: Option<String>,
    ) -> syn::Result<Self> {
        Ok(Self {
            ident,
            constructor: contract.constructor.as_ref().map(Into::into),
            functions: contract.functions().map(Into::into).collect(),
            events: contract.events().map(Into::into).collect(),
            bytecode,
        })
    }
}

impl CodeGen for Contract {
    fn gen_ir_code(&self) -> TokenStream {
        let constructor = self.constructor.as_ref().map(|c| c.gen_ir_code());
        let functions: Vec<_> = self.functions.iter().map(|f| f.gen_ir_code()).collect();

        let instance_functions: Vec<_> =
            self.functions.iter().map(|f| f.gen_instance_fn()).collect();

        let events: Vec<_> = self.events.iter().map(|e| e.generate_event()).collect();
        let logs: Vec<_> = self.events.iter().map(|e| e.generate_log()).collect();

        let mod_name = format_ident!(
            "{}",
            self.ident.to_string().to_snake_case(),
            span = self.ident.span(),
        );

        let contract_struture_ident = format_ident!(
            "{}",
            self.ident.to_string().to_upper_camel_case(),
            span = self.ident.span(),
        );

        let deploy_fn = if let Some(bytecode) = self.bytecode.clone() {
            self.constructor.as_ref().map(|c| c.gen_deploy_fn(bytecode))
        } else {
            None
        };

        quote! {
            pub type #contract_struture_ident = #mod_name::#contract_struture_ident;

            pub mod #mod_name {

                use ethers_rs::ethabi;

                const INTERNAL_ERR: &'static str = "`ethers-rs contract macros` internal error";

                use ethers_rs::Provider;
                use ethers_rs::Signer;
                use ethers_rs::ContractContext;
                use ethers_rs::Address;

                /// Solidity contract rust mapping type
                pub struct #contract_struture_ident(ethers_rs::ContractContext);

                impl #contract_struture_ident {

                    #deploy_fn

                    pub fn new<A,C>(address: A, client: C) ->  ethers_rs::Result<Self>
                    where
                    A: TryInto<ethers_rs::Address>,
                    A::Error: std::fmt::Display + std::fmt::Debug,
                    C: Into<ethers_rs::Client>,
                    {
                        let address = address.try_into().map_err(ethers_rs::custom_error)?;

                        Ok(Self(ContractContext{address, client: client.into() }))
                    }

                    pub fn connect<A>(&self, address: A) -> ethers_rs::Result<Self>
                     where
                    A: TryInto<ethers_rs::Address>,
                    A::Error: std::fmt::Display + std::fmt::Debug,
                    {
                        let address = address.try_into().map_err(ethers_rs::custom_error)?;

                        Ok(Self(ContractContext{
                            address,
                            client: self.0.client.clone(),
                        }))
                    }

                    pub fn with_provider(&self, provider: ethers_rs::Provider) -> Self {
                        Self(ContractContext{
                            address: self.0.address.clone(),
                            client: ethers_rs::Client {
                                provider,
                                signer: self.0.client.signer.clone()
                            },
                        })
                    }

                    pub fn with_signer(&self, signer: ethers_rs::Signer) -> Self {
                        Self(ContractContext{
                            address: self.0.address.clone(),
                            client: ethers_rs::Client {
                                provider: self.0.client.provider.clone(),
                                signer: Some(signer),
                            },
                        })
                    }

                    #(#instance_functions)*
                }

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
                    use ethers_rs::ethabi;
                    #(#logs)*
                }


            }

        }
    }
}
