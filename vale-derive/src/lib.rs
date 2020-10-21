mod derive;
mod rule;
mod ruleset;

#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validator(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ds = syn::parse_macro_input!(ts as derive::Validate);
    ds.finish().into()
}


#[proc_macro]
pub fn rule(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ds = syn::parse_macro_input!(ts as rule::Rule);
    ds.finish().into()
}

#[proc_macro_attribute]
pub fn ruleset(_: proc_macro::TokenStream, ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ds = syn::parse_macro_input!(ts as ruleset::Ruleset);
    ds.finish().into()
}
