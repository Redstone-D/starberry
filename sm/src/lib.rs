use proc_macro::{Delimiter, TokenStream, TokenTree};
use quote::{quote, ToTokens}; 
use syn::{
    braced, bracketed, parse::{Parse, ParseStream}, parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma, Block, Expr, FnArg, Ident, ItemFn, LitInt, LitStr, Pat, PatIdent, Result as SynResult, ReturnType, Token, Type
}; 
use proc_macro2::{Span, TokenStream as TokenStream2}; 

// #[proc_macro_attribute]
// pub fn log_func_info(_: TokenStream, input: TokenStream) -> TokenStream {
//     let mut func = parse_macro_input!(input as ItemFn);
//     let func_name = &func.sig.ident; 
//     let func_block = &func.block; 
//     let output = quote! {
//         {
//             println!("fun {} starts", stringify!(#func_name));
//             let __log_result = { #func_block };
//             println!("fun {} ends", stringify!(#func_name));
//             __log_result
//         }
//     };
//     func.block = syn::parse2(output).unwrap();
//     quote! { #func }.into()
// } 

// struct RegisterUrlArgs {
//     pub app: Ident,
//     pub literal: LitStr, 
// } 

// impl Parse for RegisterUrlArgs { 
//     fn parse(input: ParseStream) -> SynResult<Self> { 
//         let app: Ident = input.parse()?;
//         input.parse::<Token![,]>()?;
//         let literal: LitStr = input.parse()?;
//         if !input.is_empty() {
//             return Err(input.error("expected exactly two arguments: `app_var, \"literal\"`"));
//         }
//         Ok(RegisterUrlArgs { app, literal })
//     }
// } 

// #[proc_macro_attribute]
// pub fn lit_url(attr: TokenStream, function: TokenStream) -> TokenStream {
//     let RegisterUrlArgs { app, literal } = parse_macro_input!(attr as RegisterUrlArgs);
//     let func = parse_macro_input!(function as ItemFn);
//     let func_ident = &func.sig.ident;

//     let register_fn_name = format!("__register_{}", func_ident);
//     let register_fn_ident = syn::Ident::new(&register_fn_name, func_ident.span()); 
    
//     let inserted_call = quote! { 
//         #func

//         #[ctor::ctor]
//         fn #register_fn_ident() {
//             #app.literal_url(#literal, ::std::sync::Arc::new(#func_ident));
//         } 
//     };
//     inserted_call.into() 
// } 

struct UrlMethodArgs {
    pub url_expr: Expr,
    pub config: Option<Vec<Expr>>,
    pub middlewares: Option<Vec<Expr>> 
} 

impl Parse for UrlMethodArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Parse the required URL expression first
        let url_expr: Expr = input.parse()?;
        
        // Initialize optional parameters
        let mut config: Option<Vec<Expr>> = None;
        let mut middlewares: Option<Vec<Expr>> = None;
        
        // If there are more tokens, process named parameters
        while !input.is_empty() {
            // Expect a comma before each parameter
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                return Err(input.error("expected comma before parameter"));
            }
            
            // Parse parameter name
            if input.peek(Ident) {
                let param_name: Ident = input.parse()?;
                let param_name_str = param_name.to_string();
                
                // Expect an equals sign
                input.parse::<Token![=]>()?;
                
                // Parse parameter value based on name
                match param_name_str.as_str() {
                    "config" => {
                        let content;
                        syn::bracketed!(content in input);
                        let list = Punctuated::<Expr, Comma>::parse_terminated(&content)?;
                        config = Some(list.into_iter().collect());
                    },
                    "middleware" => {
                        let list = Punctuated::<Expr, Comma>::parse_terminated(input)?;
                        middlewares = Some(list.into_iter().collect());
                    },
                    _ => return Err(input.error(format!("unknown parameter: {}", param_name_str))),
                }
            } else {
                return Err(input.error("expected parameter name"));
            }
        }
        
        Ok(UrlMethodArgs {
            url_expr,
            config, 
            middlewares  
        })
    }
} 

#[proc_macro_attribute]
pub fn url(attr: TokenStream, function: TokenStream) -> TokenStream {
    // Parse the attribute arguments and the function.
    let args = parse_macro_input!(attr as UrlMethodArgs);
    let url_expr = args.url_expr;
    let mut func = parse_macro_input!(function as ItemFn);
    let func_ident = &func.sig.ident;

    // Create a unique registration function name.
    let register_fn_name = format!("__register_{}", func_ident);
    let register_fn_ident = syn::Ident::new(&register_fn_name, func_ident.span());

    // Generate code for setting optional parameters
    let config_setup = if let Some(config_expr) = args.config {
        let set_calls = config_expr.iter().map(|expr| {
            quote! { child_url.set_params(#expr); }
        });
        quote! { #(#set_calls)* }
    } else {
        quote! {}
    }; 

    let middleware_setup = if let Some(middleware_expr) = args.middlewares {
        quote! { 
            let mut middlewares: Vec<std::sync::Arc<(dyn starberry::starberry_core::app::middleware::AsyncMiddleware<_> + 'static)>> = vec![]; 
            middlewares.append(vec![#(Arc::new(#middleware_expr)),*]) 
            child_url.set_middlewares(middlewares);  
        }
    } else {
        quote! { 
        }
    }; 

    // Check if the function has a parameter
    let has_param = !func.sig.inputs.is_empty();
    
    // Get return type of function
    let returns_http_response = if let syn::ReturnType::Type(_, ret_type) = &func.sig.output {
        // Check if return type is HttpResponse
        match ret_type.as_ref() {
            syn::Type::Path(type_path) => {
                let last_segment = type_path.path.segments.last().unwrap();
                last_segment.ident.to_string() == "HttpResponse"
            }
            _ => false,
        }
    } else {
        // No return type specified, assume it's Rc
        false
    };

    // Create a new function with modified signature if needed
    let wrapper_func_ident = syn::Ident::new(&format!("__wrapper_{}", func_ident), func_ident.span());
    
    // Generate wrapper code based on parameter presence and return type
    let (wrapper_code, param_name) = if has_param {
        // Extract the first parameter
        if let syn::FnArg::Typed(pat_type) = &func.sig.inputs[0] {
            // Get parameter name
            let param_name = if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
                pat_ident.ident.clone()
            } else {
                syn::Ident::new("req", func_ident.span())
            };
            
            // Generate code based on return type
            if returns_http_response {
                // Update the function signature to use &mut Rc instead of Rc
                if let syn::FnArg::Typed(ref mut pat_type) = func.sig.inputs[0] {
                    // Create &mut Rc type
                    let rc_path = syn::parse_str::<syn::Path>("HttpReqCtx").unwrap();
                    let rc_type = syn::TypePath { 
                        qself: None,
                        path: rc_path
                    };
                    
                    let mut_type = syn::TypeReference {
                        and_token: syn::token::And::default(),
                        lifetime: None,
                        mutability: Some(syn::token::Mut::default()),
                        elem: Box::new(syn::Type::Path(rc_type)),
                    };
                    
                    // Replace the type in the function signature
                    pat_type.ty = Box::new(syn::Type::Reference(mut_type));
                }
                
                // Create wrapper function
                (quote! {
                    async fn #wrapper_func_ident(mut rc: HttpReqCtx) -> HttpReqCtx {
                        let response = #func_ident(&mut rc).await;
                        rc.response = response;
                        rc
                    }
                }, param_name)
            } else {
                // Returning Rc directly, no wrapper needed
                (quote! {}, param_name)
            }
        } else {
            // Unexpected parameter type, use default
            let param_name = syn::Ident::new("req", func_ident.span());
            
            if returns_http_response {
                (quote! {
                    async fn #wrapper_func_ident(mut rc: HttpReqCtx) -> HttpReqCtx {
                        let response = #func_ident(&mut rc).await;
                        rc.response = response;
                        rc
                    }
                }, param_name)
            } else {
                (quote! {}, param_name)
            }
        }
    } else {
        // No parameters, add default req parameter
        let param_name = syn::Ident::new("req", func_ident.span());
        
        // Modify the original function to add the req parameter
        let mut new_inputs = syn::punctuated::Punctuated::new();
        
        if returns_http_response {
            // Create &mut HttpReqCtx type for parameter
            let rc_path = syn::parse_str::<syn::Path>("HttpReqCtx").unwrap();
            let rc_type = syn::TypePath { 
                qself: None,
                path: rc_path
            };
            
            let mut_type = syn::TypeReference {
                and_token: syn::token::And::default(),
                lifetime: None,
                mutability: Some(syn::token::Mut::default()),
                elem: Box::new(syn::Type::Path(rc_type)),
            };
            
            let pat_ident = syn::PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: None, // No need for mut since the reference is already mut
                ident: param_name.clone(),
                subpat: None,
            };
            
            let param = syn::FnArg::Typed(syn::PatType {
                attrs: vec![],
                pat: Box::new(syn::Pat::Ident(pat_ident)),
                colon_token: syn::token::Colon::default(),
                ty: Box::new(syn::Type::Reference(mut_type)),
            });
            
            new_inputs.push(param);
        } else {
            // For HttpReqCtx return type, keep original behavior with mut HttpReqCtx parameter
            let param_path = syn::TypePath { 
                qself: None,
                path: syn::Path::from(syn::Ident::new("HttpReqCtx", func_ident.span()))
            };
            
            let param_type = syn::Type::Path(param_path);
            let pat_ident = syn::PatIdent {
                attrs: vec![],
                by_ref: None,
                mutability: Some(syn::token::Mut::default()),
                ident: param_name.clone(),
                subpat: None,
            };
            
            let param = syn::FnArg::Typed(syn::PatType {
                attrs: vec![],
                pat: Box::new(syn::Pat::Ident(pat_ident)),
                colon_token: syn::token::Colon::default(),
                ty: Box::new(param_type),
            });
            
            new_inputs.push(param);
        }
        
        func.sig.inputs = new_inputs;

        if returns_http_response {
            (quote! {
                async fn #wrapper_func_ident(mut rc: HttpReqCtx) -> HttpReqCtx {
                    let response = #func_ident(&mut rc).await;
                    rc.response = response;
                    rc
                }
            }, param_name)
        } else {
            (quote! {}, param_name)
        }
    }; 

    // Choose which function to register
    let register_function = if returns_http_response { 
        func.attrs.push(syn::parse_quote!(#[allow(unused_mut)]));
        func.attrs.push(syn::parse_quote!(#[allow(unused_variables)])); 
        quote! { #wrapper_func_ident }
    } else { 
        func.attrs.push(syn::parse_quote!(#[allow(unused_mut)]));
        func.attrs.push(syn::parse_quote!(#[allow(unused_variables)])); 
        quote! { #func_ident }
    };

    // Generate the final code
    let expanded = quote! {
        #func

        #wrapper_code

        // This function will be executed at startup (using the ctor crate).
        #[ctor::ctor]
        fn #register_fn_ident() {
            let mut child_url = #url_expr;  
            #config_setup 
            #middleware_setup 
            child_url.set_method(Arc::new(#register_function)); 
            // child_url.set_middlewares(child_url.middlewares.read().unwrap().get_middlewares()); 
        }
    };

    expanded.into()
} 

#[proc_macro_attribute]
pub fn middleware(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the async fn we're given
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = input_fn.sig.ident.clone();

    // Enforce async functions only:
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            fn_name, 
            "#[middleware] can only be used on async fn"
        )
        .to_compile_error()
        .into();
    } 

    // parse the type parameter R from the attribute (or default to HttpReqCtx)
    let ty_tokens = if attr.is_empty() {
        quote! { HttpReqCtx }
    } else {
        let ty: Type = syn::parse(attr).expect("Expected a single type in #[middleware<…>]");
        quote! { #ty }
    };

    // Extract first argument's name and type
    let mut param_ident = syn::Ident::new("req", fn_name.span());
    let mut param_is_mut = false; 
    
    if let Some(first_arg) = input_fn.sig.inputs.first() {
        if let FnArg::Typed(pat_type) = first_arg {
            // pattern = identifier?
            if let Pat::Ident(ref pat_ident) = *pat_type.pat {
                param_ident = pat_ident.ident.clone();
                param_is_mut = pat_ident.mutability.is_some();
            } 
        }
    }

    // How we bind the incoming context into a mutable (or not) local
    let param_binding = if param_is_mut {
        quote! { let #param_ident = context; }
    } else {
        quote! { let mut #param_ident = context; }
    };

    // The original function body (a Block)
    let fn_body = &input_fn.block;

    // Generate:
    //  pub struct Foo;
    //  impl AsyncMiddleware<ParamType> for Foo { ... }
    let expanded = quote! {
        // drop the original free function; we only emit the struct+impl
        pub struct #fn_name;

        impl AsyncMiddleware<#ty_tokens> for #fn_name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn return_self() -> Self
            where
                Self: Sized,
            {
                #fn_name
            }

            fn handle<'a>(
                &'a self,
                context: #ty_tokens,
                next: Box<
                    dyn Fn(#ty_tokens) -> std::pin::Pin<Box<dyn std::future::Future<Output = #ty_tokens> + Send>>
                        + Send
                        + Sync
                        + 'static,
                >,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = #ty_tokens> + Send + 'static>> {
                Box::pin(async move {
                    #param_binding
                    // original user code:
                    #fn_body
                })
            }
        }
    };

    TokenStream::from(expanded)
} 

/// A macro to create an Value from a literal or expression.
/// It can handle dictionaries, lists, booleans, strings, and numeric values. 
#[proc_macro]
pub fn object(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as ValueExpr);
    let expanded = generate_code(&expr);
    TokenStream::from(expanded)
}

/// A macro that returns a JSON response containing the provided object
#[proc_macro]
pub fn akari_json(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as ValueExpr);
    let object_code = generate_code(&expr);
    
    let expanded = quote! {
        json_response(#object_code)
    };
    
    TokenStream::from(expanded)
} 

#[proc_macro]
pub fn reg(input: TokenStream) -> TokenStream {
    // Parse the comma-separated items inside reg![ ... ]
    let items = parse_macro_input!(input with parse_items);

    // We expect at least one item (the "ancestor" which can be App/Url in various forms)
    if items.is_empty() {
        return syn::Error::new_spanned(
            quote!(),
            "reg! macro requires at least one argument"
        )
        .to_compile_error()
        .into();
    }

    let first = &items[0];        // The first argument (App/Url/Arc<App>/Arc<Url> etc.)
    let rest = &items[1..];       // Subsequent arguments (UrlPattern, &str, etc.)

    // We'll convert the rest into a Vec<PathPattern> expression
    // for example, by wrapping string-likes with LitUrl(), or calling .clone() for references.
    // In a real-world scenario, you'd do more complex type checks, but here's a simplified approach:
    let mut path_segments = Vec::new();

    for expr in rest {
        path_segments.push(convert_expr_to_pathpattern(expr));
    }

    // Decide expansion depending on the first argument type.
    // In a full-blown macro you'd likely do advanced type-checking or pattern matching with `syn`,
    // but for illustration, we produce code that calls either .reg_from(...) or .register(...)
    // based on whether it "looks" like App or Url. In practice, you'd do more robust matching.

    // Simplistic check: if the first token string contains "Url", call .register(...),
    // otherwise call .reg_from(...). This is purely demonstrative.
    let first_str = quote! { #first }.to_string();
    let expansion = if first_str.contains("Url") {
        // Url path
        quote! {
            {
                let ancestor = #first;
                // Suppose function=None, middleware = ancestor.get_middlewares(), path=our segments:
                let _segments: Vec<PathPattern> = vec![#(#path_segments),*];
                // Call the .register(...) method
                ancestor
                    .register(
                        _segments,
                        None, 
                        ancestor.get_middlewares(), 
                        Params::default()
                    )
                    .map_err(|e| e.to_string())
            }
        }
    } else {
        // App path
        quote! {
            {
                let ancestor = #first;
                let _segments: Vec<PathPattern> = vec![#(#path_segments),*];
                // Call .reg_from() if the type is an App-like
                ancestor.reg_from::<HttpReqCtx>(&_segments)
            }
        }
    };

    TokenStream::from(expansion)
}

/// Parse the items inside the macro call as a list of expressions
fn parse_items(input: syn::parse::ParseStream) -> syn::Result<Vec<Expr>> {
    let items = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
    Ok(items.into_iter().collect())
}

/// Convert an expression (e.g. string literal, UrlPattern, etc.) into a PathPattern expression
fn convert_expr_to_pathpattern(expr: &Expr) -> proc_macro2::TokenStream {
    // Very naive approach: if it's a literal string, call LitUrl(...).
    // If it's something else, assume we can clone() or pass it directly as a PathPattern.
    match expr {
        Expr::Lit(expr_lit) => {
            // We'll wrap with LitUrl(...) for string-literal or numeric-literal (simplified approach)
            quote! {
                PathPattern::LitUrl(#expr_lit.to_string())
            }
        }
        Expr::Path(_) | Expr::Call(_) | Expr::Reference(_) => {
            // We assume it's a UrlPattern or something that can be tried as clone() or used directly.
            // If you need .clone(), you'd do something like `#expr.clone()`.
            quote! {
                (#expr).clone()
            }
        }
        _ => {
            // Fallback, just pass directly (in a real macro you'd add more refined handling).
            quote! { #expr }
        }
    }
} 

/// A macro for rendering templates with context data.
/// 
/// # Example
/// ```no_run
/// use starberry_macro::akari_render; 
/// use starberry_core::http::response::request_templates::template_response; 
/// use starberry_core::Value;
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
enum ValueExpr {
    Dict(Dict),
    List(List),
    Other(syn::Expr),
}

struct Dict {
    entries: Vec<(String, ValueExpr)>,
}

struct List {
    items: Vec<ValueExpr>,
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
            let value: ValueExpr = content.parse()?;
            
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
            let item: ValueExpr = content.parse()?;
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
impl Parse for ValueExpr {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if input.peek(syn::token::Brace) {
            let dict = Dict::parse(input)?;
            Ok(ValueExpr::Dict(dict))
        } else if input.peek(syn::token::Bracket) {
            let list = List::parse(input)?;
            Ok(ValueExpr::List(list))
        } else {
            // Any other expression
            let expr: syn::Expr = input.parse()?;
            Ok(ValueExpr::Other(expr))
        }
    }
}

// Generate code for each type of ValueExpr
fn generate_code(expr: &ValueExpr) -> TokenStream2 {
    match expr {
        ValueExpr::Dict(dict) => {
            let entries = dict.entries.iter().map(|(key, value)| {
                let value_code = generate_code(value);
                quote! {
                    map.insert(#key.to_string(), #value_code);
                }
            });
            
            quote! {{
                let mut map = ::std::collections::HashMap::new();
                #(#entries)*
                Value::Dict(map)
            }}
        },
        ValueExpr::List(list) => {
            let items = list.items.iter().map(|item| {
                let item_code = generate_code(item);
                quote! {
                    vec.push(#item_code);
                }
            });
            
            quote! {{
                let mut vec = Vec::new();
                #(#items)*
                Value::List(vec)
            }}
        },
        ValueExpr::Other(expr) => {
            match expr {
                syn::Expr::Lit(lit_expr) => {
                    match &lit_expr.lit {
                        syn::Lit::Bool(b) => {
                            let value = b.value;
                            quote! { Value::new(#value) }
                        },
                        syn::Lit::Str(s) => {
                            let value = &s.value();
                            quote! { Value::new(#value) }
                        },
                        syn::Lit::Int(_) | syn::Lit::Float(_) => {
                            quote! { Value::new(#expr) }
                        },
                        _ => quote! { Value::new(#expr) }
                    }
                },
                _ => quote! { Value::new(#expr) }
            }
        },
    }
}

// RenderArgs structure to parse akari_render arguments
struct RenderArgs {
    template_path: LitStr,
    context: Vec<(Ident, ValueExpr)>,
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
                let value: ValueExpr = input.parse()?;
                
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
