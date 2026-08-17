//! Derive macros for `mcproto-types`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Fields, Ident, Index, Member, Result, Type, parse_macro_input,
    parse_quote,
};

/// Derives [`mcproto_types::TypeCodec`] for a protocol structure.
///
/// Fields are encoded and decoded in declaration order. A codec kind must be
/// supplied so errors from individual fields retain the enclosing structure:
///
/// ```ignore
/// #[derive(TypeStructCodec)]
/// #[type_struct_codec(kind = Slot)]
/// struct Item {
///     id: VarInt,
///     count: VarInt,
/// }
/// ```
#[proc_macro_derive(TypeStructCodec, attributes(type_struct_codec))]
pub fn derive_type_struct_codec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_type_struct_codec(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_type_struct_codec(input: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "TypeStructCodec can only be derived for structs",
        ));
    };
    let kind = type_struct_codec_kind(input)?;
    let name = &input.ident;

    let fields: Vec<(Member, &Type)> = match &data.fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                (
                    Member::Named(field.ident.clone().expect("named field")),
                    &field.ty,
                )
            })
            .collect(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, field)| (Member::Unnamed(Index::from(index)), &field.ty))
            .collect(),
        Fields::Unit => Vec::new(),
    };

    let mut bounded_generics = input.generics.clone();
    for (_, field_type) in &fields {
        bounded_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#field_type: ::mcproto_types::TypeCodec));
    }
    let (impl_generics, _, where_clause) = bounded_generics.split_for_impl();
    let (_, type_generics, _) = input.generics.split_for_impl();

    let encode_fields = fields.iter().map(|(member, _)| {
        quote! {
            ::mcproto_types::TypeCodec::encode(&self.#member, writer)
                .map_err(|error| error.with_context(
                    ::mcproto_types::__private::CodecKind::#kind,
                ))?;
        }
    });
    let decode_fields: Vec<_> = fields
        .iter()
        .map(|(_, field_type)| {
            quote! {
                <#field_type as ::mcproto_types::TypeCodec>::decode(reader)
                    .map_err(|error| error.with_context(
                        ::mcproto_types::__private::CodecKind::#kind,
                    ))?
            }
        })
        .collect();
    let construct = match &data.fields {
        Fields::Named(fields) => {
            let names = fields
                .named
                .iter()
                .map(|field| field.ident.as_ref().unwrap());
            quote! { Self { #(#names: #decode_fields,)* } }
        }
        Fields::Unnamed(_) => quote! { Self(#(#decode_fields,)*) },
        Fields::Unit => quote! { Self },
    };

    Ok(quote! {
        impl #impl_generics ::mcproto_types::TypeCodec for #name #type_generics #where_clause {
            fn encode(
                &self,
                writer: &mut impl ::std::io::Write,
            ) -> ::std::result::Result<(), ::mcproto_types::__private::CodecError> {
                #(#encode_fields)*
                ::std::result::Result::Ok(())
            }

            fn decode(
                reader: &mut impl ::std::io::Read,
            ) -> ::std::result::Result<Self, ::mcproto_types::__private::CodecError> {
                ::std::result::Result::Ok(#construct)
            }
        }
    })
}

fn type_struct_codec_kind(input: &DeriveInput) -> Result<Ident> {
    let mut kind = None;
    for attribute in &input.attrs {
        if !attribute.path().is_ident("type_struct_codec") {
            continue;
        }
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("kind") {
                return Err(meta.error("expected `kind = CodecKindVariant`"));
            }
            if kind.is_some() {
                return Err(meta.error("duplicate `kind` argument"));
            }
            kind = Some(meta.value()?.parse()?);
            Ok(())
        })?;
    }
    kind.ok_or_else(|| {
        Error::new_spanned(
            input,
            "TypeStructCodec requires `#[type_struct_codec(kind = CodecKindVariant)]`",
        )
    })
}

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
