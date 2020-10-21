use syn::parse;
use quote::ToTokens;

pub(crate) struct Validate {
    name: syn::Ident,
    validations: Vec<FieldValidation>,
}

impl parse::Parse for Validate {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let span = proc_macro2::Span::call_site();
        let derive_input = syn::DeriveInput::parse(input)?;
        let data = match derive_input.data {
            syn::Data::Struct(data) => data,
            syn::Data::Enum(_) | syn::Data::Union(_) => {
                return Err(parse::Error::new(span, "enums and unions are not supported"));
            },
        };
        let fields = match data.fields {
            syn::Fields::Named(fields) => fields,
            syn::Fields::Unnamed(_) => {
                return Err(parse::Error::new(span, "can't validate a tuple struct"));
            }
            syn::Fields::Unit => {
                return Err(parse::Error::new(span, "can't validate a unit struct"));
            }
        }.named;
        let mut validations = Vec::new();
        for field in fields.into_iter() {
            validations.push(FieldValidation::parse(field)?);
        }
        Ok(Self { name: derive_input.ident, validations })
    }
}

impl Validate {
    pub(crate) fn finish(self) -> proc_macro2::TokenStream {
        let name = self.name;
        let conditions: Vec<proc_macro2::TokenStream> = self
            .validations
            .iter()
            .flat_map(move |FieldValidation { name, conditions }| {
                conditions.iter().map(move |c| (c, name))
            })
            .map(|(c, name)| c.finish(name).unwrap())
            .collect();

        quote::quote! {
            impl vale::Validate for #name {
                #[vale::ruleset]
                fn validate(&mut self) -> Result<(), Vec<String>> {
                    #(#conditions;)*
                }
            }
        }
    }
}

struct FieldValidation {
    name: syn::Ident,
    conditions: Vec<Condition>
}

impl FieldValidation {
    fn parse(field: syn::Field) -> parse::Result<Self> {
        let mut conditions: Vec<Condition> = Vec::new();
        for attr in field.attrs.into_iter() {
            conditions.extend(Condition::parse(attr)?);
        }
        Ok(Self {
            name: field.ident.unwrap(),
            conditions,
        })
    }
}

#[derive(Debug)]
struct Condition {
    name: syn::Ident,
    // _parens: Option<token::Paren>,
    content: Option<proc_macro2::TokenStream>,
}

impl Condition {
    fn parse(tokens: syn::Attribute) -> parse::Result<Vec<Self>> {
        let span = proc_macro2::Span::call_site();
        let meta_list = match tokens.parse_meta()? {
            syn::Meta::List(l) => l,
            syn::Meta::Path(_) | syn::Meta::NameValue(_) => {
                return Err(parse::Error::new(span, "validations not formatted correctly"));
            }
        };
        let path = if let Some(path) = meta_list.path.get_ident() {
            path
        } else {
            return Err(parse::Error::new(span, "validations must start with #[validate]"));
        };
        if path != "validate" {
            return Err(parse::Error::new(span, "validations must start with #[validate]"));
        }
        let mut result = vec![];
        for nmeta in meta_list.nested {
            match nmeta {
                syn::NestedMeta::Meta(syn::Meta::List(mut l)) => {
                    let name = l.path.segments.pop().unwrap().into_value().ident;
                    let content = l.nested.pop().unwrap().into_value().into_token_stream();
                    result.push(Self {
                        name,
                        content: Some(content),
                    })
                },
                syn::NestedMeta::Meta(syn::Meta::Path(mut p)) => {
                    let name = p.segments.pop().unwrap().into_value().ident;
                    result.push(Self {
                        name,
                        content: None,
                    })
                },
                _ => return Err(parse::Error::new(span, "malformed validation")),
            };
        }
        Ok(result)
    }

    fn finish(&self, field_name: &syn::Ident) -> parse::Result<proc_macro2::TokenStream> {
        let kind = ValidationKind::parse(&self.name, self.content.as_ref())?;

        Ok(kind.finish(field_name))
    }
}

enum ValidationKind {
    Lt(proc_macro2::TokenStream),
    Eq(proc_macro2::TokenStream),
    Gt(proc_macro2::TokenStream),
    Neq(proc_macro2::TokenStream),
    LenLt(proc_macro2::TokenStream),
    LenEq(proc_macro2::TokenStream),
    LenGt(proc_macro2::TokenStream),
    LenNeq(proc_macro2::TokenStream),
    With(proc_macro2::TokenStream),
    Trim,
    ToLowerCase,
}

impl ValidationKind {
    fn parse(name: &syn::Ident, content: Option<&proc_macro2::TokenStream>) -> parse::Result<Self> {
        let span = proc_macro2::Span::call_site();
        let res = match name.to_string().as_str() {
            "lt" => Self::Lt(content.unwrap().clone()),
            "eq" => Self::Eq(content.unwrap().clone()),
            "gt" => Self::Gt(content.unwrap().clone()),
            "neq" => Self::Neq(content.unwrap().clone()),
            "len_lt" => Self::LenLt(content.unwrap().clone()),
            "len_eq" => Self::LenEq(content.unwrap().clone()),
            "len_gt" => Self::LenGt(content.unwrap().clone()),
            "len_neq" => Self::LenNeq(content.unwrap().clone()),
            "with" => Self::With(content.unwrap().clone()),
            "trim" => Self::Trim,
            "to_lower_case" => Self::ToLowerCase,
            otherwise => return Err(parse::Error::new(span, &format!("unrecognised attribute: {}", otherwise)))
        };

        Ok(res)
    }

    fn finish(self, name: &syn::Ident) -> proc_macro2::TokenStream {
        match self {
            Self::Lt(stream) => quote::quote! {
                vale::rule!(
                    self.#name < #stream,
                    format!("Failed to validate field `{}`, value too high", stringify!(#name)),
                )
            },
            Self::Eq(stream) => quote::quote! {
                vale::rule!(
                    self.#name == #stream,
                    format!("Failed to validate field `{}`, value incorrect", stringify!(#name)),
                )
            },
            Self::Gt(stream) => quote::quote! {
                vale::rule!(
                    self.#name > #stream,
                    format!("Failed to validate field `{}`, value too low", stringify!(#name)),
                )
            },
            Self::Neq(stream) => quote::quote! {
                vale::rule!(
                    self.#name != #stream,
                    format!("Failed to validate field `{}`, value not allowed", stringify!(#name)),
                )
            },
            Self::LenLt(stream) => quote::quote! {
                vale::rule!(
                    self.#name.len() < #stream,
                    format!("Failed to validate field `{}`, value too long", stringify!(#name)),
                )
            },
            Self::LenEq(stream) => quote::quote! {
                vale::rule!(
                    self.#name.len ()== #stream,
                    format!("Failed to validate field `{}`, value of incorrect length", stringify!(#name)),
                )
            },
            Self::LenGt(stream) => quote::quote! {
                vale::rule!(
                    self.#name.len() > #stream,
                    format!("Failed to validate field `{}`, value too short", stringify!(#name)),
                )
            },
            Self::LenNeq(stream) => quote::quote! {
                vale::rule!(
                    self.#name.len() != #stream,
                    format!("Failed to validate field `{}`, value of disallowed length", stringify!(#name)),
                )
            },
            Self::With(stream) => quote::quote! {
                vale::rule!(
                    #stream(&mut self.#name),
                    format!("Failed to validate field `{}`, value did not pass test", stringify!(#name)),
                )
            },
            Self::Trim => quote::quote! {
                self.#name = self.#name.trim().into();
            },
            Self::ToLowerCase => quote::quote! {
                self.#name = self.#name.to_lowercase().into();
            },
        }
    }
}


// pub(crate) struct DeriveState {
//     name: syn::Ident,
//     validations: Vec<Validation>,
// }

// impl parse::Parse for DeriveState {
//     fn parse(input: parse::ParseStream) -> parse::Result<Self> {
//         let span = proc_macro2::Span::call_site();

//         let derive_input = syn::DeriveInput::parse(input)?;
//         let data = match derive_input.data {
//             syn::Data::Struct(data) => data,
//             syn::Data::Enum(_data) => todo!(), // maybe do this?
//             _ => return Err(parse::Error::new(span, "you suck")),
//         };

//         let validations = Self::parse_fields(data.fields, span)?;
//         Ok(Self {
//             name: derive_input.ident,
//             validations,
//         })
//     }
// }

// impl DeriveState {
//     fn parse_fields(input: syn::Fields, span: proc_macro2::Span) -> parse::Result<Vec<Validation>> {
//         let fields = match input {
//             syn::Fields::Named(fields) => fields,
//             syn::Fields::Unnamed(_) => {
//                 return Err(parse::Error::new(span, "can't validate a tuple struct"))
//             }
//             syn::Fields::Unit => {
//                 return Err(parse::Error::new(span, "can't validate a unit struct"))
//             }
//         };
//         Ok(fields
//             .named
//             .iter()
//             // we can safely unwrap `f.ident` because we have validated that we are not paring unit
//             // structs
//             .flat_map(|f| f.attrs.iter().map(move |a| (a, f.ident.as_ref().unwrap())))
//             .map(|(a, i)| Validation::from_stream(i.clone(), a.tokens.clone()))
//             .collect::<parse::Result<Vec<Vec<Validation>>>>()?
//             .into_iter()
//             .flat_map(|vs| vs.into_iter())
//             .collect())
//     }

//     pub(crate) fn finish(&self) -> proc_macro2::TokenStream {
//         let Self { name, validations } = self;
//         let validations = validations.iter().map(Validation::code);
//         quote::quote! {
//             impl vale::Validate for #name {
//                 #[vale::ruleset]
//                 fn validate(&mut self) -> Result<(), Vec<String>> {
//                     #(#validations;)*
//                 }
//             }
//         }
//     }
// }

// struct Validation {
//     name: syn::Ident,
//     kind: ValidationKind,
// }

// impl Validation {
//     fn from_stream(name: syn::Ident, tokens: proc_macro2::TokenStream) -> parse::Result<Vec<Self>> {
//         let mut validations = vec![];
//         for token in dbg!(tokens) {
//             let group = match token {
//                 proc_macro2::TokenTree::Group(group) => group,
//                 _ => continue,
//             };
//             let span = group.span();
//             let mut stream = group.stream().into_iter();
//             let mut need_punct = false;
            

//             while let Some(kind) = dbg!(stream.next()) {
//                 let kind = match kind {
//                     proc_macro2::TokenTree::Ident(ident) => {
//                         if need_punct {
//                             return Err(parse::Error::new(span, "expected `,`, found identifier"));
//                         }
//                         ident.to_string()
//                     }
//                     proc_macro2::TokenTree::Punct(p) => {
//                         if !need_punct {
//                             return Err(parse::Error::new(
//                                 span,
//                                 &format!("expected identifier, found `{}`", p.to_string()),
//                             ));
//                         }
//                         need_punct = false;
//                         continue;
//                     }
//                     _ => {
//                         return Err(parse::Error::new(
//                             span,
//                             "validation list should contain an identifier",
//                         ))
//                     }
//                 };
//                 let mut next_ident = || {
//                     let tt = match stream.next() {
//                         Some(thing) => thing,
//                         None => return Err(parse::Error::new(span, "empty validation list")),
//                     };
//                     let group = match tt {
//                         proc_macro2::TokenTree::Group(group) => group,
//                         _ => {
//                             return Err(parse::Error::new(
//                                 span,
//                                 "validator argument should be an identifier",
//                             ))
//                         }
//                     };
//                     let ident = match group.stream().into_iter().next() {
//                         Some(tt) => quote::quote! { #tt },
//                         _ => {
//                             return Err(parse::Error::new(
//                                 span,
//                                 "validator argument list should not be empty",
//                             ))
//                         }
//                     };
//                     Ok(ident)
//                 };
//                 let kind = match kind.as_str() {
//                     "lt" => ValidationKind::Lt(next_ident()?),
//                     "eq" => ValidationKind::Eq(next_ident()?),
//                     "gt" => ValidationKind::Gt(next_ident()?),
//                     "neq" => ValidationKind::Neq(next_ident()?),
//                     "len_lt" => ValidationKind::LenLt(next_ident()?),
//                     "len_eq" => ValidationKind::LenEq(next_ident()?),
//                     "len_gt" => ValidationKind::LenGt(next_ident()?),
//                     "len_neq" => ValidationKind::LenNeq(next_ident()?),
//                     "with" => ValidationKind::With(next_ident()?),
//                     "trim" => ValidationKind::Trim,
//                     "to_lower_case" => ValidationKind::ToLowerCase,
//                     otherwise => {
//                         return Err(parse::Error::new(
//                             span,
//                             &format!("unknown validator option: {}", otherwise),
//                         ))
//                     }
//                 };
//                 validations.push(Self {
//                     name: name.clone(),
//                     kind,
//                 });
//                 need_punct = true;
//             }
//         }
//         Ok(validations)
//     }

//     fn code(&self) -> proc_macro2::TokenStream {
//         let validation = self.validate_code();
//         quote::quote! {
//             #validation
//         }
//     }

//     fn validate_code(&self) -> proc_macro2::TokenStream {
//         let Self { name, kind } = self;
//         match kind {
//             ValidationKind::Lt(ident) => quote::quote! {
//                 vale::rule!(
//                     self.#name < #ident,
//                     format!("Failed to validate field `{}`, value too high", stringify!(#name)),
//                 )
//             },
//             ValidationKind::Eq(ident) => quote::quote! {
//                 vale::rule!(
//                     self.#name == #ident,
//                     format!("Failed to validate field `{}`, value incorrect", stringify!(#name)),
//                 )
//             },
//             ValidationKind::Gt(ident) => quote::quote! {
//                 vale::rule!(
//                     self.#name > #ident,
//                     format!("Failed to validate field `{}`, value too low", stringify!(#name)),
//                 )
//             },
//             ValidationKind::Neq(ident) => quote::quote! {
//                 vale::rule!(
//                     self.#name != #ident,
//                     format!("Failed to validate field `{}`, value not allowed", stringify!(#name)),
//                 )
//             },
//             ValidationKind::LenLt(ident) => quote::quote! {
//                 vale::rule!(
//                     self.#name.len() < #ident,
//                     format!("Failed to validate field `{}`, value too long", stringify!(#name)),
//                 )
//             },
//             ValidationKind::LenEq(ident) => quote::quote! {
//                 vale::rule!(
//                     self.#name.len ()== #ident,
//                     format!("Failed to validate field `{}`, value of incorrect length", stringify!(#name)),
//                 )
//             },
//             ValidationKind::LenGt(ident) => quote::quote! {
//                 vale::rule!(
//                     self.#name.len() > #ident,
//                     format!("Failed to validate field `{}`, value too short", stringify!(#name)),
//                 )
//             },
//             ValidationKind::LenNeq(ident) => quote::quote! {
//                 vale::rule!(
//                     self.#name.len() != #ident,
//                     format!("Failed to validate field `{}`, value of disallowed length", stringify!(#name)),
//                 )
//             },
//             ValidationKind::With(ident) => quote::quote! {
//                 vale::rule!(
//                     #ident(&mut self.#name),
//                     format!("Failed to validate field `{}`, value did not pass test", stringify!(#name)),
//                 )
//             },
//             ValidationKind::Trim => quote::quote! {
//                 self.#name = self.#name.trim().into();
//             },
//             ValidationKind::ToLowerCase => quote::quote! {
//                 self.#name = self.#name.to_lowercase().into();
//             },
//         }
//     }
// }
