use syn::{parse, punctuated as punct};

pub(crate) struct Rule {
    condition: syn::Expr,
    msg: syn::Expr,
}

impl parse::Parse for Rule {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let span = proc_macro2::Span::call_site();

        let mut content = 
            punct::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated(input)?;
        let args = content.len();
        if args != 1 && args != 2 {
            let msg = format!("`rule` macro requires 1 or 2 arguments, got {}", args);
            return Err(parse::Error::new(span, &msg));
        }

        let msg = if args == 2 {
            content.pop().unwrap().into_value()
        } else {
            syn::Expr::Lit(syn::ExprLit {
                attrs: vec![],
                lit: syn::Lit::Str(syn::LitStr::new("No message provided", span)),
            })
        };

        let condition = content.pop().unwrap().into_value();

        Ok(Self { condition, msg, })
    }
}

impl Rule {
    pub(crate) fn finish(self) -> proc_macro2::TokenStream {
        let Self { condition, msg } = self;
        quote::quote! {
            if !{#condition} {
                errors.push({ #msg }.into());
            }
        }
    }
}