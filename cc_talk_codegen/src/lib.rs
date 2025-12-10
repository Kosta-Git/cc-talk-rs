use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

#[proc_macro_attribute]
pub fn command(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let command_name = &input.ident;

    if let Some(error) = ensure_struct(&input) {
        return error;
    }

    // If the strct has no fields, it means we send an empty payload.
    let _has_no_fields = match &input.data {
        Data::Struct(data_struct) => data_struct.fields.is_empty(),
        Data::Enum(_) | Data::Union(_) => false,
    };

    let expanded = quote! {
        #input

        impl cc_talk_core::cc_talk::Command for #command_name {
            type Response = ();

            fn header(&self) -> cc_talk_core::cc_talk::Header {
                cc_talk_core::cc_talk::Header::SimplePoll
            }

            fn data(&self) -> &[u8] {
                &[]
            }

            fn parse_response(
                &self,
                response_payload: &[u8],
            ) -> std::result::Result<Self::Response, cc_talk_core::cc_talk::ParseResponseError> {

                match response_payload.is_empty() {
                    true => Ok(()),
                    false => Err(cc_talk_core::cc_talk::ParseResponseError::DataLengthMismatch(
                        0,
                        response_payload.len(),
                    )),
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn ensure_struct(input: &DeriveInput) -> Option<proc_macro::TokenStream> {
    let is_enum = matches!(input.data, Data::Enum(_));
    let is_union = matches!(input.data, Data::Union(_));

    let got_ident = if is_enum {
        "enum"
    } else if is_union {
        "union"
    } else {
        return None;
    };

    Some(
        syn::Error::new_spanned(
            input.clone(),
            format!("expected `struct` got `{got_ident}`"),
        )
        .to_compile_error()
        .into(),
    )
}
