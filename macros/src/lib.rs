use contract::Contract;
use gen::CodeGen;
use proc_macro::TokenStream;
use syn::parse_macro_input;

mod contract;
mod error;
mod gen;

#[proc_macro_derive(Contract, attributes(abi_file))]
pub fn table(item: TokenStream) -> TokenStream {
    let stream = Contract::new(parse_macro_input!(item))
        .expect("Derive contract structure")
        .gen_ir_code();

    eprintln!("{}", stream);

    stream.into()
}
