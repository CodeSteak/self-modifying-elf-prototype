#![recursion_limit = "512"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::Span;

static GENERIC_NAME: [&'static str; 16] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
];

static NULL: &'static str = "null";

#[proc_macro_attribute]
pub fn service(args: TokenStream, fun: TokenStream) -> TokenStream {
    let args: syn::AttributeArgs = syn::parse_macro_input!(args);

    let args_lit = args
        .into_iter()
        .map(|arg| match arg {
            syn::NestedMeta::Literal(lit) => lit,
            other => panic!("Argument {:?} is unsupported!", quote!(#other)),
        })
        .collect();

    let args_lit_br = &args_lit;

    let fun: syn::ItemFn = syn::parse_macro_input!(fun);

    let fun_args = fun.decl.inputs.clone();

    let fun_args_types_only: Vec<syn::Type> = fun_args
        .iter()
        .skip(1)
        .map(|arg| match arg {
            syn::FnArg::Captured(arg) => arg.ty.clone(),
            syn::FnArg::Ignored(ty) => ty.clone(),
            _ => panic!("This form of argument is not supported! {:?}", quote!(#arg)),
        })
        .collect();

    let fun_args_types_only_br = &fun_args_types_only;

    let fun_args_fill_nulls: Vec<syn::Ident> = fun_args_types_only
        .iter()
        .map(|_| syn::Ident::new(NULL, Span::call_site()))
        .collect();

    let fun_args_generic_names: Vec<syn::Ident> = fun_args_types_only
        .iter()
        .enumerate()
        .map(|(i, _)| syn::Ident::new(&GENERIC_NAME[i], Span::call_site()))
        .collect();
    let fun_args_generic_names_br = &fun_args_generic_names;

    let ident = &fun.ident;
    let register_name = make_name(args_lit_br);
    let register_ident = syn::Ident::new(&register_name, Span::call_site());

    let fun_out = &fun.decl.output;
    let info_string = format!(
        " * {} : {} \n\t\t\t {}\n\n",
        quote!(#(#args_lit_br)*),
        quote!(#(#fun_args_types_only_br)*),
        quote!(#fun_out)
    );

    let info_str = syn::LitStr::new(&info_string, Span::call_site());

    if fun_args_generic_names.is_empty() {
        quote!(
            #fun

            #[used]
            #[cfg_attr(target_os = "linux", link_section = ".ctors")]
            #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
            #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
            static #register_ident : extern fn() = {
                extern fn #register_ident() {
                    use crate::ipc_derived____;
                    use crate::RoutingContext____;

                    let inner = | context : &mut RoutingContext____ , nothing : Vec<i32> | {
                        assert_eq!(nothing.len(), 0);

                        #ident(context.into())
                    };

                    let null : Option<()> = None;
                    ipc_derived____::register(Box::new(FnHandlerState(
                        (#(#args_lit_br),*) , inner)));

                    ipc_derived____::register_info(#info_str);
                }
                #register_ident
            };
        )
        .into()
    } else {
        quote!(
            #fun

            #[used]
            #[cfg_attr(target_os = "linux", link_section = ".ctors")]
            #[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
            #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
            static #register_ident : extern fn() = {
                extern fn #register_ident() {
                    use crate::ipc_derived____;
                    use crate::RoutingContext____;

                    let inner = | context : &mut RoutingContext____, ( #( #fun_args_generic_names_br ),* ,) | {
                        #ident(context.into(), #( #fun_args_generic_names_br ),* )
                    };

                    let null : Option<()> = None;
                    ipc_derived____::register(Box::new(FnHandlerState(
                        (#(#args_lit_br),* , #(#fun_args_fill_nulls),*) , inner)));

                    ipc_derived____::register_info(#info_str);
                }
                #register_ident
            };
        )
            .into()
    }
}

fn make_name(arg: &Vec<syn::Lit>) -> String {
    format!("SERVICE__{}", quote!(#(#arg),*))
        .chars()
        .map(|c| if c == ',' { '_' } else { c })
        .filter(|c| c.is_ascii_alphabetic() || *c == '_')
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

#[proc_macro_attribute]
pub fn context(_args: TokenStream, t: TokenStream) -> TokenStream {
    let s: syn::ItemStruct = syn::parse_macro_input!(t);
    let ty = &s.ident;

    quote!(

        #s

        pub type RoutingContext____ = #ty;

        impl Into<()> for &mut RoutingContext____ {
            fn into(self) -> () { () }
        }

        impl Into<()> for &RoutingContext____ {
            fn into(self) -> () { () }
        }

        impl Into<()> for RoutingContext____ {
            fn into(self) -> () { () }
        }

        impl DefaultRouter<#ty> for RoutingContext____ {
            fn default_router() -> Router<#ty> {
                ipc_derived____::get()
            }

            fn routing_info() -> String {
                ipc_derived____::info()
            }
        }

        pub(crate) mod ipc_derived____ {
            use ipc::*;

            use super::RoutingContext____;

            static mut INITIAL_ROUTING : Option<Router<RoutingContext____>> = None;
            static mut INITIAL_ROUTING_INFO : Option<String> = None;
            static mut SEAL : bool = false;

            // This is called via ctors, which is single threaded.
            pub(crate) fn register(handler : Box<dyn Handler<RoutingContext____>>) {
                unsafe {
                    if SEAL { return; }
                    if INITIAL_ROUTING.is_none() {
                        INITIAL_ROUTING = Some(Router::default());
                    }
                    INITIAL_ROUTING
                        .as_mut()
                        .unwrap()
                        .register(handler);
                }
            }

             pub(crate) fn register_info(inf : &str) {
                unsafe {
                    if SEAL { return; }
                    if INITIAL_ROUTING_INFO.is_none() {
                        INITIAL_ROUTING_INFO = Some(String::new())
                    }
                    INITIAL_ROUTING_INFO
                        .as_mut()
                        .unwrap()
                        .push_str(inf);
                }
            }

            pub(crate) fn get() -> Router<RoutingContext____> {
                unsafe {
                    SEAL = true;
                    if let Some(ref routing_table) = INITIAL_ROUTING {
                        routing_table.clone()
                    } else {
                        Router::default()
                    }
                }
            }

            pub(crate) fn info() -> String {
                unsafe {
                    SEAL = true;
                    if let Some(ref inf) = INITIAL_ROUTING_INFO {
                        inf.clone()
                    } else {
                        String::new()
                    }
                }
            }
        }
    )
    .into()
}
