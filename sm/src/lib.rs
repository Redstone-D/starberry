use proc_macro::TokenStream;
use quote::quote; 
use syn::{
    parse::{Parse, ParseStream}, 
    parse_macro_input, ItemFn, Result as SynResult, Expr, Token, LitStr, Ident
}; 

#[proc_macro_attribute]
pub fn log_func_info(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident; 
    let func_block = &func.block; 
    let output = quote! {
        {
            println!("fun {} starts", stringify!(#func_name));
            let __log_result = { #func_block };
            println!("fun {} ends", stringify!(#func_name));
            __log_result
        }
    };
    func.block = syn::parse2(output).unwrap();
    quote! { #func }.into()
} 

use starberry_core::app::urls as Url; 

struct RegisterUrlArgs {
    pub app: Ident,
    pub literal: LitStr, 
} 

impl Parse for RegisterUrlArgs { 
    fn parse(input: ParseStream) -> SynResult<Self> { 
        let app: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let literal: LitStr = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("expected exactly two arguments: `app_var, \"literal\"`"));
        }
        Ok(RegisterUrlArgs { app, literal })
    }
} 

#[proc_macro_attribute]
pub fn lit_url(attr: TokenStream, function: TokenStream) -> TokenStream {
    let RegisterUrlArgs { app, literal } = parse_macro_input!(attr as RegisterUrlArgs);
    let func = parse_macro_input!(function as ItemFn);
    let func_ident = &func.sig.ident;

    let register_fn_name = format!("__register_{}", func_ident);
    let register_fn_ident = syn::Ident::new(&register_fn_name, func_ident.span()); 
    
    let inserted_call = quote! { 
        #func

        #[ctor::ctor]
        fn #register_fn_ident() {
            #app.literal_url(#literal, ::std::sync::Arc::new(#func_ident));
        } 
    };
    inserted_call.into() 
}
