//! Derive macros for `mcproto-types` and `mcproto-network`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Fields, Ident, Index, LitInt, LitStr, Member, Result, Type,
    parse_macro_input, parse_quote,
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
    let kind = type_struct_codec_kind(input)?;
    expand_struct_codec(input, &kind, quote!(::mcproto_types))
}

fn expand_struct_codec(
    input: &DeriveInput,
    kind: &Ident,
    types: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream> {
    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "struct codec derives can only be used on structs",
        ));
    };
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
            .push(parse_quote!(#field_type: #types::TypeCodec));
    }
    let (impl_generics, _, where_clause) = bounded_generics.split_for_impl();
    let (_, type_generics, _) = input.generics.split_for_impl();

    let encode_fields = fields.iter().map(|(member, _)| {
        quote! {
            #types::TypeCodec::encode(&self.#member, writer)
                .map_err(|error| error.with_context(
                    #types::__private::CodecKind::#kind,
                ))?;
        }
    });
    let decode_fields: Vec<_> = fields
        .iter()
        .map(|(_, field_type)| {
            quote! {
                <#field_type as #types::TypeCodec>::decode(reader)
                    .map_err(|error| error.with_context(
                        #types::__private::CodecKind::#kind,
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
        impl #impl_generics #types::TypeCodec for #name #type_generics #where_clause {
            fn encode(
                &self,
                writer: &mut impl ::std::io::Write,
            ) -> ::std::result::Result<(), #types::__private::CodecError> {
                #(#encode_fields)*
                ::std::result::Result::Ok(())
            }

            fn decode(
                reader: &mut impl ::std::io::Read,
            ) -> ::std::result::Result<Self, #types::__private::CodecError> {
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

/// Derives direction-specific packet-body coding and packet metadata.
///
/// Serverbound packet fields are encoded in declaration order. Clientbound
/// packet fields are decoded in declaration order. The packet name and numeric
/// wire ID are declared as metadata for one protocol version.
///
/// ```ignore
/// #[derive(PacketCodec)]
/// #[packet(
///     name = "status_request",
///     id = 0x00,
///     state = Status,
///     direction = Serverbound,
/// )]
/// struct StatusRequest;
/// ```
#[proc_macro_derive(PacketCodec, attributes(packet))]
pub fn derive_packet_codec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_packet_codec(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_packet_codec(input: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    let metadata = packet_metadata(input)?;
    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "PacketCodec can only be derived for structs",
        ));
    };
    let name = &input.ident;
    let packet_name = &metadata.name;
    let packet_id = &metadata.id;
    let state = &metadata.state;
    let direction = &metadata.direction;

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
            .push(parse_quote!(#field_type: ::mcproto_network::__types::TypeCodec));
    }
    let (bounded_impl_generics, _, bounded_where_clause) = bounded_generics.split_for_impl();
    let (impl_generics, _, where_clause) = input.generics.split_for_impl();
    let (_, type_generics, _) = input.generics.split_for_impl();

    let directional_codec = if direction == "Serverbound" {
        let encode_fields = fields.iter().map(|(member, _)| {
            quote! {
                ::mcproto_network::__types::TypeCodec::encode(&self.#member, writer)
                    .map_err(|error| error.with_context(
                        ::mcproto_network::__types::__private::CodecKind::TypeStruct,
                    ))?;
            }
        });

        quote! {
            impl #bounded_impl_generics ::mcproto_network::EncodePacket
                for #name #type_generics #bounded_where_clause
            {
                fn encode_body(
                    &self,
                    writer: &mut impl ::std::io::Write,
                ) -> ::std::result::Result<
                    (),
                    ::mcproto_network::__types::__private::CodecError,
                > {
                    #(#encode_fields)*
                    ::std::result::Result::Ok(())
                }
            }
        }
    } else {
        let decode_fields: Vec<_> = fields
            .iter()
            .map(|(_, field_type)| {
                quote! {
                    <#field_type as ::mcproto_network::__types::TypeCodec>::decode(reader)
                        .map_err(|error| error.with_context(
                            ::mcproto_network::__types::__private::CodecKind::TypeStruct,
                        ))?
                }
            })
            .collect();
        let construct = match &data.fields {
            Fields::Named(fields) => {
                let names = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().expect("named field"));
                quote! { Self { #(#names: #decode_fields,)* } }
            }
            Fields::Unnamed(_) => quote! { Self(#(#decode_fields,)*) },
            Fields::Unit => quote! { Self },
        };

        quote! {
            impl #bounded_impl_generics ::mcproto_network::DecodePacket
                for #name #type_generics #bounded_where_clause
            {
                fn decode_body(
                    reader: &mut impl ::std::io::Read,
                ) -> ::std::result::Result<
                    Self,
                    ::mcproto_network::__types::__private::CodecError,
                > {
                    ::std::result::Result::Ok(#construct)
                }
            }
        }
    };

    Ok(quote! {
        impl #impl_generics ::mcproto_network::Packet for #name #type_generics #where_clause {
            const NAME: ::mcproto_network::PacketName =
                ::mcproto_network::PacketName::new(#packet_name);
            const ID: ::mcproto_network::PacketId =
                match ::mcproto_network::PacketId::new(#packet_id) {
                    ::std::option::Option::Some(id) => id,
                    ::std::option::Option::None => ::core::panic!("packet ID must be non-negative"),
                };
            const STATE: ::mcproto_network::ProtocolState =
                ::mcproto_network::ProtocolState::#state;
            const DIRECTION: ::mcproto_network::Direction =
                ::mcproto_network::Direction::#direction;
        }

        #directional_codec
    })
}

struct PacketMetadata {
    name: LitStr,
    id: LitInt,
    state: Ident,
    direction: Ident,
}

fn packet_metadata(input: &DeriveInput) -> Result<PacketMetadata> {
    let mut name = None;
    let mut id = None;
    let mut state = None;
    let mut direction = None;
    let mut found_attribute = false;

    for attribute in &input.attrs {
        if !attribute.path().is_ident("packet") {
            continue;
        }
        found_attribute = true;

        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                if name.is_some() {
                    return Err(meta.error("duplicate `name` argument"));
                }
                name = Some(meta.value()?.parse::<LitStr>()?);
                return Ok(());
            }
            if meta.path.is_ident("id") {
                if id.is_some() {
                    return Err(meta.error("duplicate `id` argument"));
                }
                id = Some(meta.value()?.parse::<LitInt>()?);
                return Ok(());
            }
            if meta.path.is_ident("state") {
                if state.is_some() {
                    return Err(meta.error("duplicate `state` argument"));
                }
                state = Some(meta.value()?.parse::<Ident>()?);
                return Ok(());
            }
            if meta.path.is_ident("direction") {
                if direction.is_some() {
                    return Err(meta.error("duplicate `direction` argument"));
                }
                direction = Some(meta.value()?.parse::<Ident>()?);
                return Ok(());
            }

            Err(meta.error("expected `name`, `id`, `state`, or `direction`"))
        })?;
    }

    if !found_attribute {
        return Err(Error::new_spanned(
            input,
            "PacketCodec requires `#[packet(name = \"...\", id = ..., state = ..., direction = ...)]`",
        ));
    }

    let name = name.ok_or_else(|| Error::new_spanned(input, "missing packet `name` argument"))?;
    if !is_valid_packet_name(&name.value()) {
        return Err(Error::new_spanned(
            &name,
            "packet name must use the official lower_case form",
        ));
    }

    let id = id.ok_or_else(|| Error::new_spanned(input, "missing packet `id` argument"))?;
    id.base10_parse::<i32>().map_err(|_| {
        Error::new_spanned(
            &id,
            "packet ID must be a non-negative integer no greater than i32::MAX",
        )
    })?;

    let state =
        state.ok_or_else(|| Error::new_spanned(input, "missing packet `state` argument"))?;
    validate_ident_variant(
        &state,
        &["Handshaking", "Status", "Login", "Configuration", "Play"],
        "packet state",
    )?;

    let direction = direction
        .ok_or_else(|| Error::new_spanned(input, "missing packet `direction` argument"))?;
    validate_ident_variant(
        &direction,
        &["Serverbound", "Clientbound"],
        "packet direction",
    )?;

    Ok(PacketMetadata {
        name,
        id,
        state,
        direction,
    })
}

fn validate_ident_variant(ident: &Ident, allowed: &[&str], description: &str) -> Result<()> {
    let value = ident.to_string();
    if allowed.contains(&value.as_str()) {
        Ok(())
    } else {
        Err(Error::new_spanned(
            ident,
            format!(
                "invalid {description} `{value}`; expected one of {}",
                allowed.join(", ")
            ),
        ))
    }
}

fn is_valid_packet_name(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_lowercase() {
        return false;
    }

    let mut previous_was_underscore = false;
    for &byte in &bytes[1..] {
        if byte == b'_' {
            if previous_was_underscore {
                return false;
            }
            previous_was_underscore = true;
        } else if byte.is_ascii_lowercase() {
            previous_was_underscore = false;
        } else {
            return false;
        }
    }

    !previous_was_underscore
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
