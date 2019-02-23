extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::Span;

#[proc_macro_attribute]
pub fn service(args: TokenStream, fun: TokenStream) -> TokenStream {
    let fun: syn::ItemFn = syn::parse_macro_input!(fun);
    let args: syn::Expr = syn::parse_macro_input!(args);

    let ident = &fun.ident;
    let register_name = make_name(&args);
    let register_ident = syn::Ident::new(&register_name, Span::call_site());

    quote! (
        #fun

        #[used]
        #[cfg_attr(target_os = "linux", link_section = ".ctors")]
        #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
        #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
        static #register_ident : extern fn() = {
            extern fn #register_ident() {
                use crate::ROUTING_TABLE;
                ROUTING_TABLE.lock().unwrap().register(Box::new(FnHandler(#args, #ident)));
            }
            #register_ident
        };
    )
    .into()
}

#[proc_macro_attribute]
pub fn service_state(args: TokenStream, fun: TokenStream) -> TokenStream {
    let fun: syn::ItemFn = syn::parse_macro_input!(fun);
    let args: syn::Expr = syn::parse_macro_input!(args);

    let ident = &fun.ident;
    let register_name = make_name(&args);
    let register_ident = syn::Ident::new(&register_name, Span::call_site());

    quote! (
        #fun

        #[used]
        #[cfg_attr(target_os = "linux", link_section = ".ctors")]
        #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
        #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
        static #register_ident : extern fn() = {
            extern fn #register_ident () {
                use crate::ROUTING_TABLE;
                ROUTING_TABLE.lock().unwrap().register(Box::new(FnHandlerState(#args, #ident)));
            }
            #register_ident
        };
    )
    .into()
}

fn make_name(arg: &syn::Expr) -> String {
    format!("SERVICE__{}", quote!(#arg))
        .chars()
        .map(|c| if c == ',' { '_' } else { c })
        .filter(|c| c.is_ascii_alphabetic() || *c == '_')
        .map(|c| c.to_ascii_uppercase())
        .collect()
}
