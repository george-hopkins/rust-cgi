use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, ReturnType, Type};

fn looks_like_result(return_type: &ReturnType) -> bool {
    if let ReturnType::Type(_, ty) = return_type {
        if let Type::Path(p) = &**ty {
            if let Some(seg) = p.path.segments.last() {
                if seg.ident == "Result" {
                    return true;
                }
            }
        }
    }

    false
}

/// Enables a CGI main function.
///
/// # Examples
///
/// ```ignore
/// #[cgi::main]
/// fn main(request: cgi::Request) -> cgi::Response {
///     todo!()
/// }
/// ```
//#[cfg(not(test))] // NOTE: exporting main breaks tests, we should file an issue.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let inputs = &input.sig.inputs;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;
    let vis = &input.vis;

    if name != "main" {
        return TokenStream::from(quote_spanned! { name.span() =>
            compile_error!("only the main function can be tagged with #[cgi::main]"),
        });
    }

    if input.sig.asyncness.is_some() {
        return TokenStream::from(quote_spanned! { input.span() =>
            compile_error!("async not supported"),
        });
    }

    let inner = if looks_like_result(ret) {
        quote! {
            cgi::handle(|request: cgi::Request| {
                match inner_main(request) {
                    Ok(resp) => resp,
                    Err(err) => {
                        eprintln!("{:?}", err);
                        cgi::empty_response(500)
                    }
                }
            })
        }
    } else {
        quote! {
            cgi::handle(inner_main)
        }
    };

    let result = quote! {
        #vis fn main() {
            #(#attrs)*
            fn inner_main(#inputs) #ret {
                #body
            }

            #inner
        }

    };

    result.into()
}
