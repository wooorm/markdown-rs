//! Derive macros for `markdown`, which re-exports them behind its `derive`
//! feature.
//! Use them through `markdown`, not this crate.

#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields, Ident, LitStr, Path};

/// Implement `markdown::extension::TokenType` for an enum of unit variants.
///
/// It writes `ALL` with every variant, in order, `name`, and `content`.
/// A variant’s name is its own name with the first letter in lowercase,
/// such as `mentionName` for `MentionName`, unless it has
/// `#[token(name = "…")]`.
/// `#[token(content = text)]` or `#[token(content = document)]` makes a
/// variant a content token.
/// On the enum, `#[token(crate = "…")]` gives the path to `markdown`, if
/// the crate is renamed.
///
/// ```
/// use markdown::extension::{ContentType, TokenType};
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, TokenType)]
/// enum DirectiveToken {
///     Directive,
///     #[token(name = "directiveName")]
///     Name,
///     #[token(content = text)]
///     DirectiveLabel,
/// }
///
/// assert_eq!(
///     DirectiveToken::ALL,
///     &[DirectiveToken::Directive, DirectiveToken::Name, DirectiveToken::DirectiveLabel]
/// );
/// assert_eq!(DirectiveToken::Directive.name(), "directive");
/// assert_eq!(DirectiveToken::Name.name(), "directiveName");
/// assert_eq!(DirectiveToken::DirectiveLabel.name(), "directiveLabel");
/// assert_eq!(DirectiveToken::DirectiveLabel.content(), Some(ContentType::Text));
/// assert_eq!(DirectiveToken::Directive.content(), None);
/// ```
///
/// Only an enum of unit variants, without generics, works:
///
/// ```compile_fail
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, markdown::extension::TokenType)]
/// struct NotAnEnum;
/// ```
///
/// ```compile_fail
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, markdown::extension::TokenType)]
/// enum WithData {
///     A(u8),
/// }
/// ```
///
/// ```compile_fail
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, markdown::extension::TokenType)]
/// enum Generic<T: Copy + Eq + core::fmt::Debug + 'static> {
///     A,
///     B(core::marker::PhantomData<T>),
/// }
/// ```
///
/// Two variants cannot have the same name:
///
/// ```compile_fail
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, markdown::extension::TokenType)]
/// enum Twice {
///     A,
///     #[token(name = "a")]
///     B,
/// }
/// ```
///
/// Content is `text` or `document`:
///
/// ```compile_fail
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, markdown::extension::TokenType)]
/// enum Content {
///     #[token(content = phrasing)]
///     A,
/// }
/// ```
///
/// Other keys are errors:
///
/// ```compile_fail
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, markdown::extension::TokenType)]
/// enum Unknown {
///     #[token(nmae = "a")]
///     A,
/// }
/// ```
#[proc_macro_derive(TokenType, attributes(token))]
pub fn derive_token_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    token_type(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// A variant: its name, its token name, and its content type, if any.
struct Variant {
    ident: Ident,
    name: String,
    content: Option<Ident>,
}

fn token_type(input: &DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    let data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(Error::new_spanned(
                &input.ident,
                "expected an enum of unit variants for `TokenType`",
            ))
        }
    };

    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        return Err(Error::new_spanned(
            &input.generics,
            "expected an enum without generics for `TokenType`",
        ));
    }

    let krate = krate(input)?;
    let variants = data
        .variants
        .iter()
        .map(variant)
        .collect::<Result<Vec<_>, Error>>()?;

    for (index, (variant, span)) in variants.iter().enumerate() {
        if variants[..index]
            .iter()
            .any(|(known, _)| known.name == variant.name)
        {
            return Err(Error::new(
                *span,
                format!("duplicate token name `{}`", variant.name),
            ));
        }
    }

    let variants = variants
        .into_iter()
        .map(|(variant, _)| variant)
        .collect::<Vec<_>>();
    let ident = &input.ident;
    let all = variants.iter().map(|variant| &variant.ident);
    let name = name_fn(&variants);
    let content = content_fn(&variants, &krate);

    Ok(quote! {
        impl #krate::extension::TokenType for #ident {
            const ALL: &'static [Self] = &[#(Self::#all),*];

            #name

            #content
        }
    })
}

/// Path to `markdown`, from `#[token(crate = "…")]` on the enum.
fn krate(input: &DeriveInput) -> Result<Path, Error> {
    let mut krate: Path = syn::parse_quote!(::markdown);

    for attr in &input.attrs {
        if attr.path().is_ident("token") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("crate") {
                    krate = meta.value()?.parse::<LitStr>()?.parse()?;
                    Ok(())
                } else {
                    Err(meta.error("expected `crate` on the enum"))
                }
            })?;
        }
    }

    Ok(krate)
}

/// A variant with its `#[token(…)]` attributes, and where its name is.
fn variant(variant: &syn::Variant) -> Result<(Variant, proc_macro2::Span), Error> {
    if !matches!(variant.fields, Fields::Unit) {
        return Err(Error::new_spanned(
            variant,
            "expected a variant without fields for `TokenType`",
        ));
    }

    let mut name = None;
    let mut content = None;

    for attr in &variant.attrs {
        if attr.path().is_ident("token") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    name = Some(meta.value()?.parse::<LitStr>()?);
                    Ok(())
                } else if meta.path.is_ident("content") {
                    let value: Ident = meta.value()?.parse()?;
                    if value == "text" || value == "document" {
                        content = Some(value);
                        Ok(())
                    } else {
                        Err(Error::new_spanned(value, "expected `text` or `document`"))
                    }
                } else {
                    Err(meta.error("expected `name` or `content`"))
                }
            })?;
        }
    }

    let (name, span) = match name {
        Some(literal) => (literal.value(), literal.span()),
        None => (camel_case(&variant.ident.to_string()), variant.ident.span()),
    };

    Ok((
        Variant {
            ident: variant.ident.clone(),
            name,
            content,
        },
        span,
    ))
}

/// `TokenType::name`.
fn name_fn(variants: &[Variant]) -> proc_macro2::TokenStream {
    let arms = variants
        .iter()
        .map(|Variant { ident, name, .. }| quote!(Self::#ident => #name,));

    quote! {
        fn name(self) -> &'static str {
            match self {
                #(#arms)*
            }
        }
    }
}

/// `TokenType::content`, if a variant is content; else the default.
fn content_fn(variants: &[Variant], krate: &Path) -> proc_macro2::TokenStream {
    if variants.iter().all(|variant| variant.content.is_none()) {
        return quote!();
    }

    let arms = variants.iter().map(|Variant { ident, content, .. }| {
        let value = match content {
            Some(kind) if kind == "text" => {
                quote!(::core::option::Option::Some(#krate::extension::ContentType::Text))
            }
            Some(_) => {
                quote!(::core::option::Option::Some(#krate::extension::ContentType::Document))
            }
            None => quote!(::core::option::Option::None),
        };
        quote!(Self::#ident => #value,)
    });

    quote! {
        fn content(self) -> ::core::option::Option<#krate::extension::ContentType> {
            match self {
                #(#arms)*
            }
        }
    }
}

/// `MentionName` as `mentionName`.
fn camel_case(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::camel_case;

    #[test]
    fn lowercases_the_first_letter() {
        assert_eq!(camel_case("MentionName"), "mentionName");
        assert_eq!(camel_case("A"), "a");
        assert_eq!(camel_case("already"), "already");
    }
}
