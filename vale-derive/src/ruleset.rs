use syn::{parse, punctuated as punct, token};

pub(crate) struct Ruleset {
    visibility: syn::Visibility,
    _fn_keyword: syn::Token![fn],
    name: syn::Ident,
    _parens: token::Paren,
    args: punct::Punctuated<syn::FnArg, syn::Token![,]>,
    _arrow: syn::Token![->],
    return_type: syn::Type,
    fn_body: syn::Block,
}

impl parse::Parse for Ruleset {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let args;
        Ok(Self { 
            visibility: input.parse()?,
            _fn_keyword: input.parse()?,
            name: input.parse()?,
            _parens: syn::parenthesized!(args in input),
            args: args.parse_terminated(syn::FnArg::parse)?,
            _arrow: input.parse()?,
            return_type: input.parse()?,
            fn_body: input.parse()?,
        })
    }
}

impl Ruleset {
    pub(crate) fn finish(self) -> proc_macro2::TokenStream {
        let Self { visibility, name, args, return_type, fn_body, .. } = self;
        let syn::Block { stmts , .. } = fn_body;
        let args = args.into_iter();
        let stmts = stmts.into_iter();
        quote::quote!{
            #visibility fn #name(#(#args, )*) -> #return_type {
                let mut errors = Vec::new();
                #(#stmts; )*;
                if errors.len() != 0 {
                    Err(errors)
                } else {
                    Ok(())
                }
            }
        }
    }
}
