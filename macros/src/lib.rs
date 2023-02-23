use std::{env, fs::read_to_string, path::PathBuf};

use ethbind::rust::{BindingBuilder, JsonRuntimeBinder, RustGenerator, ToTokenStream};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, LitStr, Token};

struct Contract {
    pub contract_name: Option<String>,
    pub type_mapping: String,
    pub abi_data: String,
}

impl Parse for Contract {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let contract_name: Option<Ident> = input.parse()?;

        if contract_name.is_some() {
            input.parse::<Token!(,)>()?;
        }

        let type_mapping: LitStr = input.parse()?;

        input.parse::<Token!(,)>()?;

        let abi_data: LitStr = input.parse()?;

        Ok(Self {
            contract_name: contract_name.map(|c| c.to_string()),
            type_mapping: type_mapping.value(),
            abi_data: abi_data.value(),
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
pub fn contract(item: TokenStream) -> TokenStream {
    let contract = parse_macro_input!(item as Contract);

    let type_mapping: JsonRuntimeBinder = load_json_file(&contract.type_mapping)
        .parse()
        .expect("Parse mapping data");

    let abi_data = load_json_file(&contract.abi_data);

    let generator = if let Some(contract_name) = contract.contract_name {
        BindingBuilder::new((RustGenerator::default(), type_mapping))
            .bind(&contract_name, abi_data)
            .finalize()
            .expect("Generate contract/abi binding code")
    } else {
        BindingBuilder::new((RustGenerator::default(), type_mapping))
            .bind_hardhat(abi_data)
            .finalize()
            .expect("Generate contract/abi binding code")
    };

    let contracts = generator.to_token_streams().expect("To token streams");

    quote!(#(#contracts)*).into()
}
