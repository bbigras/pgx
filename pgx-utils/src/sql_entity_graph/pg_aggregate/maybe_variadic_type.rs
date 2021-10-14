use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{Expr, Type, parse::{Parse, ParseStream}, parse_quote};

#[derive(Debug, Clone)]
pub(crate) struct MaybeVariadicTypeList {
    pub(crate) found: Vec<MaybeVariadicType>,
    pub(crate) original: syn::Type,
}

impl MaybeVariadicTypeList {
    pub(crate) fn new(maybe_type_list: syn::Type) -> Result<Self, syn::Error> {
        match &maybe_type_list {
            Type::Tuple(tuple) => {
                let mut coll = Vec::new();
                for elem in &tuple.elems {
                    let parsed_elem = MaybeVariadicType::new(elem.clone())?;
                    coll.push(parsed_elem);
                }
                Ok(Self {
                    found: coll,
                    original: maybe_type_list,
                })
            }
            ty => Ok(Self {
                found: vec![MaybeVariadicType::new(ty.clone())?],
                original: maybe_type_list,
            }),
        }
    }

    pub(crate) fn entity_tokens(&self) -> Expr {
        let found = self.found.iter().map(|x| x.entity_tokens());
        parse_quote! {
            vec![#(#found),*]
        }
    }
}

impl Parse for MaybeVariadicTypeList {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        Self::new(input.parse()?)
    }
}

impl ToTokens for MaybeVariadicTypeList {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.original.to_tokens(tokens)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MaybeVariadicType {
    pub(crate) ty: Type,
    /// The inner of a variadic, if it exists.
    pub(crate) variadic_ty: Option<Type>,
}

impl MaybeVariadicType {
    pub(crate) fn new(ty: syn::Type) -> Result<Self, syn::Error> {
        let variadic_ty = match &ty {
            syn::Type::Macro(ty_macro) => {
                let mut found_pgx = false;
                let mut found_variadic = false;
                // We don't actually have type resolution here, this is a "Best guess".
                for (idx, segment) in ty_macro.mac.path.segments.iter().enumerate() {
                    match segment.ident.to_string().as_str() {
                        "pgx" if idx == 1 => found_pgx = true,
                        "variadic" => found_variadic = true,
                        _ => (),
                    }
                }
                if (ty_macro.mac.path.segments.len() == 1 && found_variadic)
                    || (found_pgx && found_variadic)
                {
                    let ty: syn::Type = syn::parse2(ty_macro.mac.tokens.clone())?;
                    Some(ty)
                } else {
                    None
                }
            }
            _ => None,
        };
        let retval = Self { ty, variadic_ty };
        Ok(retval)
    }

    fn entity_tokens(&self) -> Expr {
        let ty = self.variadic_ty.as_ref().unwrap_or(&self.ty);
        let variadic = self.variadic_ty.is_some();
        parse_quote! {
            pgx::datum::sql_entity_graph::aggregate::MaybeVariadicAggregateType {
                agg_ty: pgx::datum::sql_entity_graph::aggregate::AggregateType {
                    ty_source: stringify!(#ty),
                    ty_id: core::any::TypeId::of::<#ty>(),
                    full_path: core::any::type_name::<#ty>(),
                },
                variadic: #variadic,
            }
        }
    }
}

impl ToTokens for MaybeVariadicType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.ty.to_tokens(tokens)
    }
}

impl Parse for MaybeVariadicType {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        Self::new(input.parse()?)
    }
}