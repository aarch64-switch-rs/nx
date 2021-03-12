use proc_macro::TokenStream;
use syn::parse_quote;
use quote::quote;

#[proc_macro_attribute]
pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut func = syn::parse_macro_input!(input as syn::ItemFn);

    // ensure abi is extern Rust
    func.sig.abi = Some(parse_quote!(extern "Rust"));

    let ident = &func.sig.ident;

    quote!(
        // export the applied function as __nx_internal_main to be linked against by `nx` itself
        #[export_name = "__nx_internal_main"]
        #func

        // type check the function being applied to
        const _: extern "Rust" fn() -> ::nx::result::Result<()> = #ident;
    ).into()
}

#[proc_macro_attribute]
pub fn heap(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemStatic);

    if input.mutability.is_none() {
        // TODO: improve error handling
        panic!("Heap must be marked mutable");
    }

    let ident = &input.ident;

    quote!(
        #input

        #[no_mangle]
        pub fn __nx_initialize_heap(_hbl_heap: ::nx::util::PointerAndSize) -> ::nx::util::PointerAndSize {
            unsafe {
                ::nx::util::PointerAndSize::new(#ident.as_mut_ptr(), #ident.len())
            }
        }
    ).into()
}
