use super::utils::*;
use crate::gen::CodeGen;
use ethabi::StateMutability;
use ethers_hardhat_rs::ethabi;
use heck::ToSnakeCase;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;

struct TemplateParam {
    /// Template param declaration.
    ///
    /// ```text
    /// [T0: Into<Uint>, T1: Into<Bytes>, T2: IntoIterator<Item = U2>, U2 = Into<Uint>]
    /// ```
    declaration: TokenStream,
    /// Template param definition.
    ///
    /// ```text
    /// [param0: T0, hello_world: T1, param2: T2]
    /// ```
    definition: TokenStream,

    /// Template param where clause.
    ///
    /// ```text
    /// [T0: Into<Uint>, T1: Into<Bytes>, T2: IntoIterator<Item = U2>, U2 = Into<Uint>]
    /// ```
    where_clause: TokenStream,
}

struct Inputs {
    /// Collects template params into vector.
    ///
    /// ```text
    /// [Token::Uint(param0.into()), Token::Bytes(hello_world.into()), Token::Array(param2.into_iter().map(Into::into).collect())]
    /// ```
    tokenize: Vec<TokenStream>,
    /// Template params.
    template_params: Vec<TemplateParam>,
    /// Quote used to recreate `Vec<ethabi::Param>`
    recreate_quote: TokenStream,
}

struct Outputs {
    /// Decoding implementation.
    implementation: TokenStream,
    /// Decode result.
    result: TokenStream,
    /// Quote used to recreate `Vec<ethabi::Param>`.
    recreate_quote: TokenStream,
}

/// Structure used to generate contract's function interface.
pub struct Function {
    /// Function name.
    name: String,
    /// Function input params.
    inputs: Inputs,
    /// Function output params.
    outputs: Outputs,
    #[deprecated(
        note = "The constant attribute was removed in Solidity 0.5.0 and has been \
				replaced with stateMutability."
    )]
    /// Constant function.
    constant: bool,
    /// Whether the function reads or modifies blockchain state
    state_mutability: ethabi::StateMutability,
}

impl<'a> From<&'a ethabi::Function> for Function {
    fn from(f: &'a ethabi::Function) -> Self {
        // [param0, hello_world, param2]
        let input_names = input_names(&f.inputs);

        // [T0: Into<Uint>, T1: Into<Bytes>, T2: IntoIterator<Item = U2>, U2 = Into<Uint>]
        let declarations = f
            .inputs
            .iter()
            .enumerate()
            .map(|(index, param)| template_param_type(&param.kind, index));

        let where_clauses = f
            .inputs
            .iter()
            .enumerate()
            .map(|(index, param)| template_param_where_clause(&param.kind, index));

        // [Uint, Bytes, Vec<Uint>]
        let kinds: Vec<_> = f
            .inputs
            .iter()
            .map(|param| rust_type(&param.kind))
            .collect();

        // [T0, T1, T2]
        let template_names: Vec<_> = get_template_names(&kinds);

        // [param0: T0, hello_world: T1, param2: T2]
        let definitions = input_names
            .iter()
            .zip(template_names.iter())
            .map(|(param_name, template_name)| quote! { #param_name: #template_name });

        let template_params = declarations
            .zip(definitions)
            .zip(where_clauses)
            .map(|((declaration, definition), where_clause)| TemplateParam {
                declaration,
                definition,
                where_clause,
            })
            .collect();

        // [Token::Uint(param0.into()), Token::Bytes(hello_world.into()), Token::Array(param2.into_iter().map(Into::into).collect())]
        let tokenize: Vec<_> = input_names
            .iter()
            .zip(f.inputs.iter())
            .map(|(param_name, param)| {
                to_token(&from_template_param(&param.kind, param_name), &param.kind)
            })
            .collect();

        let output_result = get_output_kinds(&f.outputs);

        let output_implementation = match f.outputs.len() {
            0 => quote! {
                let _output = output;
                Ok(())
            },
            1 => {
                let o = quote! { out };
                let from_first = from_token(&f.outputs[0].kind, &o);
                quote! {
                    let out = self.0.decode_output(output)?.into_iter().next().expect(INTERNAL_ERR);
                    Ok(#from_first)
                }
            }
            _ => {
                let o = quote! { out.next().expect(INTERNAL_ERR) };
                let outs: Vec<_> = f
                    .outputs
                    .iter()
                    .map(|param| from_token(&param.kind, &o))
                    .collect();

                quote! {
                    let mut out = self.0.decode_output(output)?.into_iter();
                    Ok(( #(#outs),* ))
                }
            }
        };

        // The allow deprecated only applies to the field 'constant', but
        // due to this issue: https://github.com/rust-lang/rust/issues/60681
        // it must go on the entire struct
        #[allow(deprecated)]
        Function {
            name: f.name.clone(),
            inputs: Inputs {
                tokenize,
                template_params,
                recreate_quote: to_ethabi_param_vec(&f.inputs),
            },
            outputs: Outputs {
                implementation: output_implementation,
                result: output_result,
                recreate_quote: to_ethabi_param_vec(&f.outputs),
            },
            constant: f.constant.unwrap_or_default(),
            state_mutability: f.state_mutability,
        }
    }
}

impl Function {
    pub fn gen_instance_fn(&self) -> TokenStream {
        let module_name = syn::Ident::new(&self.name.to_snake_case(), Span::call_site());
        let tokenize = &self.inputs.tokenize;
        let declarations: &Vec<_> = &self
            .inputs
            .template_params
            .iter()
            .map(|i| &i.declaration)
            .collect();
        let definitions: &Vec<_> = &self
            .inputs
            .template_params
            .iter()
            .map(|i| &i.definition)
            .collect();

        let where_clauses: &Vec<_> = &self
            .inputs
            .template_params
            .iter()
            .map(|i| &i.where_clause)
            .collect();

        let outputs_result = &self.outputs.result;

        #[allow(unused)]
        match self.state_mutability {
            StateMutability::Pure | StateMutability::View => quote! {
                pub async fn #module_name<#(#declarations),*>(&mut self, #(#definitions),*) -> ethers_rs::Result<#outputs_result>
                where #(#where_clauses,)*
                {
                    let f = functions::#module_name::function();
                    let tokens = vec![#(#tokenize),*];

                    let bytes = f.encode_input(&tokens).expect(INTERNAL_ERR);

                    let bytes = self.0.eth_call(bytes).await?;

                    let result = functions::#module_name::decode_output(&bytes)?;

                    Ok(result)
                }
            },
            StateMutability::NonPayable => quote! {
                pub async fn #module_name<#(#declarations),*>(&mut self, #(#definitions),*) -> ethers_rs::Result<ethers_rs::TransactionReceipter<ethers_rs::Timeout>>
                where #(#where_clauses,)*
                {
                    let f = functions::#module_name::function();
                    let tokens = vec![#(#tokenize),*];

                    let bytes = f.encode_input(&tokens).expect(INTERNAL_ERR);

                    let tx_hash = self.0.send_raw_transaction(stringify!(#module_name), bytes, Default::default()).await?;

                    self.0.client.provider.register_transaction_listener(tx_hash)
                }
            },
            StateMutability::Payable => quote! {
                pub async fn #module_name<#(#declarations),*, Ops>(&mut self, #(#definitions,)* ops: Ops) -> ethers_rs::Result<ethers_rs::TransactionReceipter<ethers_rs::Timeout>>
                where
                #(#where_clauses,)*
                Ops: TryInto<ethers_rs::TxOptions>,
                Ops::Error: std::error::Error + Send + Sync + 'static,
                {
                    let ops = ops.try_into()?;

                    let f = functions::#module_name::function();
                    let tokens = vec![#(#tokenize),*];

                    let bytes = f.encode_input(&tokens).expect(INTERNAL_ERR);

                    let tx_hash = self.0.send_raw_transaction(stringify!(#module_name), bytes, ops).await?;

                    self.0.client.provider.register_transaction_listener(tx_hash)
                }
            },
        }
    }
}

impl CodeGen for Function {
    fn gen_ir_code(&self) -> TokenStream {
        let name = &self.name;
        let module_name = syn::Ident::new(&self.name.to_snake_case(), Span::call_site());
        let tokenize = &self.inputs.tokenize;
        let declarations: &Vec<_> = &self
            .inputs
            .template_params
            .iter()
            .map(|i| &i.declaration)
            .collect();
        let definitions: &Vec<_> = &self
            .inputs
            .template_params
            .iter()
            .map(|i| &i.definition)
            .collect();
        let recreate_inputs = &self.inputs.recreate_quote;
        let recreate_outputs = &self.outputs.recreate_quote;
        #[allow(deprecated)]
        let constant = self.constant;
        let state_mutability = match self.state_mutability {
            ethabi::StateMutability::Pure => quote! { ethers_rs::ethabi::StateMutability::Pure },
            ethabi::StateMutability::Payable => {
                quote! { ethers_rs::ethabi::StateMutability::Payable }
            }
            ethabi::StateMutability::NonPayable => {
                quote! { ethers_rs::ethabi::StateMutability::NonPayable }
            }
            ethabi::StateMutability::View => quote! { ethers_rs::ethabi::StateMutability::View },
        };
        let outputs_result = &self.outputs.result;
        let outputs_implementation = &self.outputs.implementation;

        let where_clauses: &Vec<_> = &self
            .inputs
            .template_params
            .iter()
            .map(|i| &i.where_clause)
            .collect();

        quote! {
            pub mod #module_name {
                use ethers_rs::ethabi;
                use super::INTERNAL_ERR;

                pub fn function() -> ethers_rs::ethabi::Function {
                    #[allow(deprecated)]
                    ethers_rs::ethabi::Function {
                        name: #name.into(),
                        inputs: #recreate_inputs,
                        outputs: #recreate_outputs,
                        constant: Some(#constant),
                        state_mutability: #state_mutability
                    }
                }

                /// Generic function output decoder.
                pub struct Decoder(ethabi::Function);

                impl ethabi::FunctionOutputDecoder for Decoder {
                    type Output = #outputs_result;

                    fn decode(&self, output: &[u8]) -> ethabi::Result<Self::Output> {
                        #outputs_implementation
                    }
                }

                /// Encodes function input.
                pub fn encode_input<#(#declarations),*>(#(#definitions),*) -> ethers_rs::Result<ethers_rs::ethabi::Bytes>
                where #(#where_clauses,)*
                {
                    let f = function();
                    let tokens = vec![#(#tokenize),*];

                    let bytes = f.encode_input(&tokens)?;

                    Ok(bytes)
                }

                /// Decodes function output.
                pub fn decode_output(output: &[u8]) -> ethabi::Result<#outputs_result> {
                    ethabi::FunctionOutputDecoder::decode(&Decoder(function()), output)
                }

            }
        }
    }
}
