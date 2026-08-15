//! Derive macros for `mcproto-types`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result, Type, parse_macro_input};

/// Derives [`mcproto_types::ProtocolEnum`] and [`mcproto_types::TypeCodec`] for
/// a fieldless enum with a numeric protocol representation.
///
/// # Example
///
/// ```ignore
/// #[derive(ProtocolEnum)]
/// #[protocol_enum(repr = VarInt)]
/// enum GameMode {
///     Survival = 0,
///     Creative = 1,
/// }
/// ```
///
/// The `repr` value must implement [`mcproto_types::EnumRepr`]. Built-in
/// numeric protocol types, including `VarInt`, `VarLong`, and fixed-width
/// integer types, implement that trait.
#[proc_macro_derive(ProtocolEnum, attributes(protocol_enum))]
pub fn derive_protocol_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_protocol_enum(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_protocol_enum(input: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    let repr = enum_repr(input)?;
    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "ProtocolEnum can only be derived for enums",
        ));
    };
    if data.variants.is_empty() {
        return Err(Error::new_spanned(
            input,
            "ProtocolEnum requires at least one enum variant",
        ));
    }

    let mut variants = Vec::with_capacity(data.variants.len());
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(Error::new_spanned(
                variant,
                "ProtocolEnum only supports fieldless enum variants",
            ));
        }
        variants.push(&variant.ident);
    }

    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::mcproto_types::ProtocolEnum for #name #type_generics #where_clause {
            type Repr = #repr;

            fn discriminant(&self) -> i128 {
                match self {
                    #(Self::#variants => Self::#variants as i128,)*
                }
            }

            fn to_repr(&self) -> ::std::option::Option<Self::Repr> {
                <Self::Repr as ::mcproto_types::EnumRepr>::from_discriminant(
                    <Self as ::mcproto_types::ProtocolEnum>::discriminant(self),
                )
            }

            fn from_repr(repr: Self::Repr) -> ::std::option::Option<Self> {
                let value = <Self::Repr as ::mcproto_types::EnumRepr>::discriminant(&repr);
                match value {
                    #(value if value == Self::#variants as i128 => ::std::option::Option::Some(Self::#variants),)*
                    _ => ::std::option::Option::None,
                }
            }
        }

        impl #impl_generics ::mcproto_types::TypeCodec for #name #type_generics #where_clause {
            fn encode(
                &self,
                writer: &mut impl ::std::io::Write,
            ) -> ::std::result::Result<(), ::mcproto_types::__private::CodecError> {
                ::mcproto_types::__private::encode_protocol_enum(self, writer)
            }

            fn decode(
                reader: &mut impl ::std::io::Read,
            ) -> ::std::result::Result<Self, ::mcproto_types::__private::CodecError> {
                ::mcproto_types::__private::decode_protocol_enum(reader)
            }
        }
    })
}

fn enum_repr(input: &DeriveInput) -> Result<Type> {
    let mut repr = None;

    for attribute in &input.attrs {
        if !attribute.path().is_ident("protocol_enum") {
            continue;
        }

        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("repr") {
                return Err(meta.error("expected `repr = Type`"));
            }
            if repr.is_some() {
                return Err(meta.error("duplicate `repr` argument"));
            }

            repr = Some(meta.value()?.parse()?);
            Ok(())
        })?;
    }

    repr.ok_or_else(|| {
        Error::new_spanned(
            input,
            "ProtocolEnum requires `#[protocol_enum(repr = Type)]`",
        )
    })
}
