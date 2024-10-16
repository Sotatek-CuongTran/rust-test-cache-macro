use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn custom_cached(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: {}", attr);
    println!("item: {}", item);
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_inputs = &input.sig.inputs;
    let fn_block = &input.block;
    let return_type = &input.sig.output;

    let cache_time = parse_macro_input!(attr as syn::ExprLit);
    let cache_time = match cache_time {
        syn::ExprLit {
            lit: syn::Lit::Int(lit),
            ..
        } => lit.base10_parse::<u64>().unwrap_or(10),
        _ => 10,
    };

    let concat_parts: Vec<_> = fn_inputs.iter().enumerate().map(|(_i, arg)| {
      let arg_name = match arg {
          syn::FnArg::Typed(pat) => &pat.pat,
          _ => panic!("Unexpected argument type"),
      };
      quote! {
          #arg_name.to_string()
      }
  }).collect();
    let expanded = quote! {
        pub async fn #fn_name(#fn_inputs) #return_type {
            use redis::AsyncCommands;
            use serde::{Serialize, Deserialize};
            use serde_json;
            use std::time::Duration;

            let concated_log = vec![
                #(#concat_parts),*
            ].join("-");
            let key = format!("{}:{}", stringify!(#fn_name), hash_inputs(&(concated_log)));
            let cache_time = Duration::from_secs(#cache_time);

            let client = redis::Client::open("redis://127.0.0.1/").unwrap();
            let mut con = client.get_multiplexed_async_connection().await.unwrap();

            // Try fetching the cached value
            println!("key: {}", key);
            if let Ok(value) = con.get::<_, String>(&key).await {
                if let Ok(deserialized) = serde_json::from_str(&value) {
                    println!("Cache hit for key: {}", key);
                    return Ok(deserialized);
                }
            }
            // Cache miss, calculate the value using the original function
            println!("Cache miss for key: {}", key);
            let result = (|| async #fn_block)().await;
            if result.is_err() {
              return Err(result.err().unwrap());
            }
            if let Ok(serialized) = serde_json::to_string(&result.as_ref().unwrap()) {
                let _: () = con.set_ex(&key, &serialized, cache_time.as_secs() as u64).await.unwrap();
            }
            result
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn test_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("item: {}", item);
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_inputs = &input.sig.inputs;
    let fn_block = &input.block;
    let return_type = &input.sig.output;

    let mut concated_log: String = String::new();
    let log_values: Vec<_> = fn_inputs
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            let arg_name = match arg {
                syn::FnArg::Typed(pat) => &pat.pat,
                _ => panic!("Unexpected argument type"),
            };
            concated_log.push_str(format!("Argument {}: {:?}", i, arg_name).as_str());
            quote! {
                println!("Argument {}: {:?}", #i, #arg_name);
            }
        })
        .collect();
    let expanded = quote! {
        pub async fn #fn_name(#fn_inputs) #return_type {
          #(#log_values)*
          let key = format!("{}:{}", stringify!(#fn_name), hash_inputs(&(#concated_log)));
          println!("key: {}", key);
          let result = (|| async #fn_block)().await;
          result
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rust_decorator(_attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let func = parse_macro_input!(input as ItemFn);
    let fn_name = &func.sig.ident;
    let fn_args = &func.sig.inputs;
    let fn_block = &func.block;
    let fn_return_type = &func.sig.output;

    // Generate the expanded code
    let expanded = quote! {
        fn #fn_name(#fn_args) #fn_return_type {
            println!("Decorator: Before calling the function");
            let result = (|| #fn_block)();  // Original function body
            println!("Decorator: After calling the function");
            result
        }
    };

    // Return the generated tokens
    TokenStream::from(expanded)
}
