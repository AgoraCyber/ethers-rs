use proc_macro2::TokenStream;
use quote::quote;

use crate::gen::CodeGen;

use super::utils::*;

pub struct Constructor {
    inputs_declarations: Vec<TokenStream>,
    inputs_definitions: Vec<TokenStream>,
    tokenize: Vec<TokenStream>,
    recreate_inputs: TokenStream,
}

impl<'a> From<&'a ethabi::Constructor> for Constructor {
    fn from(c: &'a ethabi::Constructor) -> Self {
        let input_names = input_names(&c.inputs);

        let inputs_declarations = c
            .inputs
            .iter()
            .enumerate()
            .map(|(index, param)| template_param_type(&param.kind, index))
            .collect::<Vec<_>>();

        let kinds: Vec<_> = c
            .inputs
            .iter()
            .map(|param| rust_type(&param.kind))
            .collect();

        // [T0, T1, T2]
        let template_names: Vec<_> = get_template_names(&kinds);

        // [param0: T0, hello_world: T1, param2: T2]
        let inputs_definitions = input_names
            .iter()
            .zip(template_names.iter())
            .map(|(param_name, template_name)| quote! { #param_name: #template_name });

        let inputs_definitions = Some(quote! { code: ethabi::Bytes })
            .into_iter()
            .chain(inputs_definitions)
            .collect();

        // [Token::Uint(param0.into()), Token::Bytes(hello_world.into()), Token::Array(param2.into_iter().map(Into::into).collect())]
        let tokenize: Vec<_> = input_names
            .iter()
            .zip(c.inputs.iter())
            .map(|(param_name, param)| {
                to_token(&from_template_param(&param.kind, param_name), &param.kind)
            })
            .collect();

        Self {
            inputs_declarations,
            inputs_definitions,
            tokenize,
            recreate_inputs: to_ethabi_param_vec(&c.inputs),
        }
    }
}

impl CodeGen for Constructor {
    fn gen_ir_code(&self) -> TokenStream {
        let declarations = &self.inputs_declarations;
        let definitions = &self.inputs_definitions;
        let tokenize = &self.tokenize;
        let recreate_inputs = &self.recreate_inputs;

        quote! {
            /// Encodes a call to contract's constructor.
            pub fn constructor<#(#declarations),*>(#(#definitions),*) -> ethabi::Bytes {
                let c = ethabi::Constructor {
                    inputs: #recreate_inputs,
                };
                let tokens = vec![#(#tokenize),*];
                c.encode_input(code, &tokens).expect(INTERNAL_ERR)
            }
        }
    }
}