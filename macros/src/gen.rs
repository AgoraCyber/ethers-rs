use proc_macro2::TokenStream;

pub trait CodeGen {
    fn gen_ir_code(&self) -> TokenStream;
}
