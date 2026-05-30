use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(Codec)]
pub fn codec_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Codec only supports structs with named fields"),
        },
        _ => panic!("Codec can only be derived for structs"),
    };

    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    // 生成 encode 代码
    let encode_fields = field_names.iter().map(|field_name| {
        quote! { self.#field_name.encode(buf)?; }
    });

    // 生成 decode 代码
    let decode_fields = field_names.iter().map(|field_name| {
        quote! { let #field_name = Codec::decode(buf)?; }
    });

    TokenStream::from(quote! {
        impl Codec for #name {
            fn encode(&self, buf: &mut Vec<u8>) -> Result<(), TypeCodecError> {
                #(#encode_fields)*
                Ok(())
            }

            fn decode(buf: &mut &[u8]) -> Result<Self, TypeCodecError> {
                #(#decode_fields)*
                Ok(Self {
                    #(#field_names),*
                })
            }
        }
    })
}