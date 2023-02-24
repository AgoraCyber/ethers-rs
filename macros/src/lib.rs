use std::{env, fs::read_to_string, path::PathBuf};

use ethbind::rust::{BindingBuilder, JsonRuntimeBinder, RustGenerator, ToTokenStream};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, LitStr, Token};

struct Contract {
    pub contract_name: String,
    pub abi_data_path: Option<String>,
}

impl Parse for Contract {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let contract_name: Ident = input.parse()?;

        let abi_data_path = if input.parse::<Option<Token!(,)>>()?.is_some() {
            Some(input.parse::<LitStr>()?.value())
        } else {
            None
        };

        Ok(Self {
            contract_name: contract_name.to_string(),
            abi_data_path,
        })
    }
}

mod contract {
    syn::custom_keyword!(hardhat);
}

fn load_json_file(path: &str) -> String {
    let dir = env::var("CARGO_MANIFEST_DIR").expect("Find CARGO_MANIFEST_DIR");

    let path = PathBuf::from(dir).join(path);

    read_to_string(path.clone()).expect(&format!("Read json file: {:?}", path))
}

#[proc_macro]
pub fn hardhat(item: TokenStream) -> TokenStream {
    let contract = parse_macro_input!(item as Contract);

    let type_mapping: JsonRuntimeBinder = include_str!("./mapping.json")
        .parse()
        .expect("Parse mapping.json");

    let abi_data = if let Some(abi_data_path) = contract.abi_data_path {
        load_json_file(&abi_data_path)
    } else {
        load_json_file(&format!(
            "sol/artifacts/contracts/{}.sol/{}.json",
            &contract.contract_name, &contract.contract_name
        ))
    };

    let generator = BindingBuilder::new((RustGenerator::default(), type_mapping))
        .bind_hardhat(abi_data)
        .finalize()
        .expect("Generate contract/abi binding code");

    let contracts = generator.to_token_streams().expect("To token streams");

    quote!(#(#contracts)*).into()
}
