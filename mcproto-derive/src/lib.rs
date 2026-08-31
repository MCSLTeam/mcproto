//! Derive macros for `mcproto-types` and `mcproto-network`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Fields, Ident, Index, LitInt, LitStr, Member, Result, Type,
    parse_macro_input, parse_quote,
};

struct PacketField<'a> {
    member: Member,
    ident: Option<&'a Ident>,
    ty: &'a Type,
    presence: Option<Ident>,
}

fn parse_presence_attribute(field: &syn::Field) -> Result<Option<Ident>> {
    let mut presence = None;
    for attribute in &field.attrs {
        if !attribute.path().is_ident("context") {
            continue;
        }
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("presence") {
                return Err(meta.error("expected `presence = field_name`"));
            }
            if presence.is_some() {
                return Err(meta.error("duplicate `presence` argument"));
            }
            presence = Some(meta.value()?.parse::<Ident>()?);
            Ok(())
        })?;
    }
    Ok(presence)
}

fn packet_fields(input: &DeriveInput) -> Result<Vec<PacketField<'_>>> {
    let data = &input.data;
    let Data::Struct(data) = data else {
        return Err(Error::new_spanned(
            input,
            "packet codec derives can only be used on structs",
        ));
    };
    match &data.fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().expect("named field");
                Ok(PacketField {
                    member: Member::Named(ident.clone()),
                    ident: Some(ident),
                    ty: &field.ty,
                    presence: parse_presence_attribute(field)?,
                })
            })
            .collect(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, field)| {
                if field
                    .attrs
                    .iter()
                    .any(|attribute| attribute.path().is_ident("context"))
                {
                    return Err(Error::new_spanned(
                        field,
                        "context attributes require named packet fields",
                    ));
                }
                Ok(PacketField {
                    member: Member::Unnamed(Index::from(index)),
                    ident: None,
                    ty: &field.ty,
                    presence: None,
                })
            })
            .collect(),
        Fields::Unit => Ok(Vec::new()),
    }
}

fn validate_presence_fields(fields: &[PacketField<'_>]) -> Result<()> {
    for (index, field) in fields.iter().enumerate() {
        let Some(source) = &field.presence else {
            continue;
        };
        let Some(source_index) = fields
            .iter()
            .position(|candidate| candidate.ident == Some(source))
        else {
            return Err(Error::new_spanned(
                source,
                format!("unknown presence context field `{source}`"),
            ));
        };
        if source_index >= index {
            return Err(Error::new_spanned(
                source,
                "presence context field must precede the contextual field",
            ));
        }
    }
    Ok(())
}

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

/// Derives [`mcproto_types::ContextualCodec`] for a structure whose fields may
/// depend on the presence of an earlier boolean field.
///
/// A dependent field is annotated as follows:
///
/// ```ignore
/// #[derive(ContextualStructCodec)]
/// #[contextual_struct_codec(kind = TypeStruct)]
/// struct Example {
///     has_value: Boolean,
///     #[context(presence = has_value)]
///     value: Optional<UnsignedByte>,
/// }
/// ```
#[proc_macro_derive(ContextualStructCodec, attributes(contextual_struct_codec, context))]
pub fn derive_contextual_struct_codec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_contextual_struct_codec(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_contextual_struct_codec(input: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    let kind = contextual_struct_codec_kind(input)?;
    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "ContextualStructCodec can only be derived for structs",
        ));
    };
    let Fields::Named(named_fields) = &data.fields else {
        return Err(Error::new_spanned(
            &data.fields,
            "ContextualStructCodec requires named fields",
        ));
    };

    let fields: Vec<_> = named_fields
        .named
        .iter()
        .map(|field| {
            let ident = field.ident.as_ref().expect("named field");
            let mut presence = None;
            for attribute in &field.attrs {
                if !attribute.path().is_ident("context") {
                    continue;
                }
                attribute.parse_nested_meta(|meta| {
                    if !meta.path.is_ident("presence") {
                        return Err(meta.error("expected `presence = field_name`"));
                    }
                    if presence.is_some() {
                        return Err(meta.error("duplicate `presence` argument"));
                    }
                    presence = Some(meta.value()?.parse::<Ident>()?);
                    Ok(())
                })?;
            }
            Ok((ident, &field.ty, presence))
        })
        .collect::<Result<Vec<_>>>()?;

    for (index, (_, _, presence)) in fields.iter().enumerate() {
        let Some(presence) = presence else { continue };
        let Some(context_index) = fields.iter().position(|(ident, _, _)| *ident == presence) else {
            return Err(Error::new_spanned(
                presence,
                format!("unknown presence context field `{presence}`"),
            ));
        };
        if context_index >= index {
            return Err(Error::new_spanned(
                presence,
                "presence context field must precede the contextual field",
            ));
        }
    }

    let name = &input.ident;
    let mut bounded_generics = input.generics.clone();
    for (_, field_type, _) in &fields {
        bounded_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#field_type: ::mcproto_types::ContextualCodec));
    }
    let (impl_generics, type_generics, where_clause) = bounded_generics.split_for_impl();

    let encode_fields = fields.iter().map(|(ident, _, presence)| {
        let context = match presence {
            Some(source) => quote! {
                ::mcproto_types::contextual::Context::new(self.#source.0)
            },
            None => quote! {
                ::mcproto_types::contextual::Context::present()
            },
        };
        quote! {
            ::mcproto_types::ContextualCodec::encode_with_context(
                &self.#ident,
                writer,
                &#context,
            )
            .map_err(|error| error.with_context(
                ::mcproto_types::__private::CodecKind::#kind,
            ))?;
        }
    });

    let decode_fields = fields.iter().map(|(ident, field_type, presence)| {
        let context = match presence {
            Some(source) => quote! {
                ::mcproto_types::contextual::Context::new(#source.0)
            },
            None => quote! {
                ::mcproto_types::contextual::Context::present()
            },
        };
        quote! {
            let #ident = <#field_type as ::mcproto_types::ContextualCodec>::decode_with_context(
                reader,
                &#context,
            )
            .map_err(|error| error.with_context(
                ::mcproto_types::__private::CodecKind::#kind,
            ))?;
        }
    });
    let field_names = fields.iter().map(|(ident, _, _)| ident);

    Ok(quote! {
        impl #impl_generics ::mcproto_types::ContextualCodec for #name #type_generics #where_clause {
            fn encode_with_context(
                &self,
                writer: &mut impl ::std::io::Write,
                _context: &::mcproto_types::contextual::Context,
            ) -> ::std::result::Result<(), ::mcproto_types::__private::CodecError> {
                #(#encode_fields)*
                ::std::result::Result::Ok(())
            }

            fn decode_with_context(
                reader: &mut impl ::std::io::Read,
                _context: &::mcproto_types::contextual::Context,
            ) -> ::std::result::Result<Self, ::mcproto_types::__private::CodecError> {
                #(#decode_fields)*
                ::std::result::Result::Ok(Self { #(#field_names,)* })
            }
        }
    })
}

fn contextual_struct_codec_kind(input: &DeriveInput) -> Result<Ident> {
    let mut kind = None;
    for attribute in &input.attrs {
        if !attribute.path().is_ident("contextual_struct_codec") {
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
            "ContextualStructCodec requires `#[contextual_struct_codec(kind = CodecKindVariant)]`",
        )
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
/// A field whose wire presence depends on an earlier boolean field may use
/// `#[context(presence = field_name)]`. The referenced field must precede it
/// and expose a boolean value as its tuple field, for example:
///
/// ```ignore
/// #[derive(PacketCodec)]
/// #[packet(name = "example", id = 0x00, state = Configuration, direction = Serverbound)]
/// struct Example {
///     has_value: Boolean,
///     #[context(presence = has_value)]
///     value: Optional<UnsignedByte>,
/// }
/// ```
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
#[proc_macro_derive(PacketCodec, attributes(packet, context))]
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

    let fields = packet_fields(input)?;
    validate_presence_fields(&fields)?;
    let has_presence_context = fields.iter().any(|field| field.presence.is_some());

    let mut bounded_generics = input.generics.clone();
    for field in &fields {
        let field_type = field.ty;
        let bound = if field.presence.is_some() {
            quote!(::mcproto_network::__types::ContextualCodec)
        } else {
            quote!(::mcproto_network::__types::TypeCodec)
        };
        bounded_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#field_type: #bound));
    }
    let (bounded_impl_generics, _, bounded_where_clause) = bounded_generics.split_for_impl();
    let (impl_generics, _, where_clause) = input.generics.split_for_impl();
    let (_, type_generics, _) = input.generics.split_for_impl();

    let directional_codec = if direction == "Serverbound" {
        let encode_fields = fields.iter().map(|field| {
            let member = &field.member;
            match &field.presence {
                Some(source) => quote! {
                    ::mcproto_network::__types::ContextualCodec::encode_with_context(
                        &self.#member,
                        writer,
                        &::mcproto_network::__types::contextual::Context::new(self.#source.0),
                    )
                    .map_err(|error| error.with_context(
                        ::mcproto_network::__types::__private::CodecKind::TypeStruct,
                    ))?;
                },
                None => quote! {
                    ::mcproto_network::__types::TypeCodec::encode(&self.#member, writer)
                        .map_err(|error| error.with_context(
                            ::mcproto_network::__types::__private::CodecKind::TypeStruct,
                        ))?;
                },
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
        if has_presence_context {
            let decode_fields = fields.iter().map(|field| {
                let ident = field.ident.expect("context fields are named");
                let field_type = field.ty;
                let decode = match &field.presence {
                    Some(source) => quote! {
                        <#field_type as ::mcproto_network::__types::ContextualCodec>::decode_with_context(
                            reader,
                            &::mcproto_network::__types::contextual::Context::new(#source.0),
                        )
                    },
                    None => quote! {
                        <#field_type as ::mcproto_network::__types::TypeCodec>::decode(reader)
                    },
                };
                quote! {
                    let #ident = #decode
                        .map_err(|error| error.with_context(
                            ::mcproto_network::__types::__private::CodecKind::TypeStruct,
                        ))?;
                }
            });
            let names = fields.iter().map(|field| field.ident.expect("named field"));
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
                        #(#decode_fields)*
                        ::std::result::Result::Ok(Self { #(#names,)* })
                    }
                }
            }
        } else {
            let decode_fields: Vec<_> = fields
                .iter()
                .map(|field| {
                    let field_type = field.ty;
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
