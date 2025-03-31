use proc_macro::{Delimiter, TokenStream, TokenTree};
use quote::{quote, ToTokens}; 
use syn::{
    braced, bracketed, parse::{Parse, ParseStream}, parse_macro_input, parse_quote, punctuated::Punctuated, Expr, Ident, ItemFn, LitInt, LitStr, Result as SynResult, Token
}; 
use proc_macro2::TokenStream as TokenStream2; 

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

struct UrlMethodArgs {
    pub url: Expr,
    pub path_pattern: Expr,
}

impl Parse for UrlMethodArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let url: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let path_pattern: Expr = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("expected exactly two arguments: `url, path_pattern`"));
        }
        Ok(UrlMethodArgs { url, path_pattern })
    }
}

#[proc_macro_attribute]
pub fn url(attr: TokenStream, function: TokenStream) -> TokenStream {
    // Parse the attribute arguments and the function.
    let args = parse_macro_input!(attr as UrlMethodArgs);
    let url_expr = args.url;
    let path_expr = args.path_pattern;
    let func = parse_macro_input!(function as ItemFn);
    let func_ident = &func.sig.ident;

    // Create a unique registration function name.
    let register_fn_name = format!("__register_{}", func_ident);
    let register_fn_ident = syn::Ident::new(&register_fn_name, func_ident.span());

    // Generate the code that registers the function.
    let expanded = quote! {
        #func

        // This function will be executed at startup (using the ctor crate).
        #[ctor::ctor]
        fn #register_fn_ident() {
            let child_url = match #url_expr.get_child_or_create(#path_expr){ 
                Ok(child_url) => child_url,
                Err(e) => dangling_url(), 
            };
            child_url.set_method(Arc::new(#func_ident)); 
            child_url.set_middlewares(#url_expr.middlewares.read().unwrap().get_middlewares()); 
        }
    };

    expanded.into()
} 

#[proc_macro_attribute]
pub fn middleware(_attr: TokenStream, item: TokenStream) -> TokenStream {
    use syn::parse_macro_input;
    use syn::ItemFn;
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = input_fn.sig.ident;
    let fn_block = input_fn.block; // capture the function body

    // Create a middleware struct name by appending "Middleware" to the function name.
    let middleware_name = syn::Ident::new(&format!("{}", fn_name), fn_name.span());

    let expanded = quote! {
        // Define the generated middleware struct.
        pub struct #middleware_name;

        use starberry_core::app::middleware::AsyncMiddleware;
        use starberry_core::http::request::HttpRequest;
        use starberry_core::http::response::HttpResponse;
        use std::pin::Pin;
        use std::future::Future;
        use std::any::Any;

        impl AsyncMiddleware for #middleware_name {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn handle<'a>(
                &self,
                req: HttpRequest,
                next: Box<
                    dyn Fn(HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>>
                        + Send
                        + Sync
                        + 'static,
                >,
            ) -> Pin<Box<dyn Future<Output = HttpResponse> + Send + 'static>> {
                std::pin::Pin::from(Box::new(async move {
                    (#fn_block).await // This should be optimized, it should not call await for wny middleware 
                }))
            }

            fn return_self() -> Self where Self: Sized {
                #middleware_name
            }
        }
    };

    TokenStream::from(expanded)
} 

/// A macro to create an Object from a literal or expression.
/// It can handle dictionaries, lists, booleans, strings, and numeric values. 
#[proc_macro]
pub fn object(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as ObjectExpr);
    let expanded = generate_code(&expr);
    TokenStream::from(expanded)
}

/// A macro that returns a JSON response containing the provided object
#[proc_macro]
pub fn akari_json(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as ObjectExpr);
    let object_code = generate_code(&expr);
    
    let expanded = quote! {
        json_response(#object_code)
    };
    
    TokenStream::from(expanded)
}

/// A macro for rendering templates with context data.
/// 
/// # Example
/// ```no_run
/// use starberry_macro::akari_render; 
/// use starberry_core::http::response::request_templates::template_response; 
/// use starberry_core::Object;
/// use starberry_core::object;
/// // Simple template with no context
/// akari_render!("template.html"); 
///
/// // Template with context variables
/// akari_render!("template.html", 
///     user={
///         name: "John", 
///         age: 30, 
///         roles: ["admin", "editor"]
///     },
///     page_title="Dashboard"
/// ); 
/// ``` 
#[proc_macro]
pub fn akari_render(input: TokenStream) -> TokenStream {
    let render_args = parse_macro_input!(input as RenderArgs);
    let expanded = generate_render_code(render_args);
    TokenStream::from(expanded) 
}

// Define our custom syntax structures
enum ObjectExpr {
    Dict(Dict),
    List(List),
    Other(syn::Expr),
}

struct Dict {
    entries: Vec<(String, ObjectExpr)>,
}

struct List {
    items: Vec<ObjectExpr>,
}

// Custom parsing for dictionary
impl Parse for Dict {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let content;
        braced!(content in input);
        let mut entries = Vec::new();
        
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Token![:]>()?;
            let value: ObjectExpr = content.parse()?;
            
            entries.push((key.to_string(), value));
            
            if content.is_empty() {
                break;
            }
            
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            } else {
                break;
            }
        }
        
        Ok(Dict { entries })
    }
}

// Custom parsing for list
impl Parse for List {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let content;
        bracketed!(content in input);
        let mut items = Vec::new();
        
        while !content.is_empty() {
            let item: ObjectExpr = content.parse()?;
            items.push(item);
            
            if content.is_empty() {
                break;
            }
            
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            } else {
                break;
            }
        }
        
        Ok(List { items })
    }
}

// Implement parsing for our custom syntax
impl Parse for ObjectExpr {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if input.peek(syn::token::Brace) {
            let dict = Dict::parse(input)?;
            Ok(ObjectExpr::Dict(dict))
        } else if input.peek(syn::token::Bracket) {
            let list = List::parse(input)?;
            Ok(ObjectExpr::List(list))
        } else {
            // Any other expression
            let expr: syn::Expr = input.parse()?;
            Ok(ObjectExpr::Other(expr))
        }
    }
}

// Generate code for each type of ObjectExpr
fn generate_code(expr: &ObjectExpr) -> TokenStream2 {
    match expr {
        ObjectExpr::Dict(dict) => {
            let entries = dict.entries.iter().map(|(key, value)| {
                let value_code = generate_code(value);
                quote! {
                    map.insert(#key.to_string(), #value_code);
                }
            });
            
            quote! {{
                let mut map = ::std::collections::HashMap::new();
                #(#entries)*
                Object::Dictionary(map)
            }}
        },
        ObjectExpr::List(list) => {
            let items = list.items.iter().map(|item| {
                let item_code = generate_code(item);
                quote! {
                    vec.push(#item_code);
                }
            });
            
            quote! {{
                let mut vec = Vec::new();
                #(#items)*
                Object::List(vec)
            }}
        },
        ObjectExpr::Other(expr) => {
            match expr {
                syn::Expr::Lit(lit_expr) => {
                    match &lit_expr.lit {
                        syn::Lit::Bool(b) => {
                            let value = b.value;
                            quote! { Object::new(#value) }
                        },
                        syn::Lit::Str(s) => {
                            let value = &s.value();
                            quote! { Object::new(#value) }
                        },
                        syn::Lit::Int(_) | syn::Lit::Float(_) => {
                            quote! { Object::new(#expr) }
                        },
                        _ => quote! { Object::new(#expr) }
                    }
                },
                _ => quote! { Object::new(#expr) }
            }
        },
    }
}

// RenderArgs structure to parse akari_render arguments
struct RenderArgs {
    template_path: LitStr,
    context: Vec<(Ident, ObjectExpr)>,
}

impl Parse for RenderArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Parse the template path first (must be a string literal)
        let template_path = input.parse()?;
        
        let mut context = Vec::new();
        
        // If there's a comma after the path, expect context variables
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            
            // Parse key-value pairs (key = value, key = value, ...)
            while !input.is_empty() {
                let key: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let value: ObjectExpr = input.parse()?;
                
                context.push((key, value));
                
                // Check if there's a comma for another pair
                if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                }
                
                if input.is_empty() {
                    break;
                }
            }
        }
        
        Ok(RenderArgs { template_path, context })
    }
}

// Generate code for akari_render
fn generate_render_code(args: RenderArgs) -> TokenStream2 {
    let template_path = args.template_path;
    
    // If there are no context variables, just return the template
    if args.context.is_empty() {
        return quote! {
            template_response(#template_path, ::std::collections::HashMap::new())
        };
    }
    
    // Otherwise, create a HashMap with all context variables
    let context_entries = args.context.iter().map(|(key, value)| {
        let key_str = key.to_string();
        let value_code = generate_code(value);
        
        quote! {
            context.insert(#key_str.to_string(), #value_code);
        }
    });
    
    quote! {{
        let mut context = ::std::collections::HashMap::new();
        #(#context_entries)*
        template_response(#template_path, context)
    }}
} 
