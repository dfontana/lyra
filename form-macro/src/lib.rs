use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn;

#[proc_macro_derive(FormResult)]
pub fn form_result_derive(input: TokenStream) -> TokenStream {
  let ast: syn::DeriveInput = syn::parse(input).unwrap();
  let name = &ast.ident;
  let form_name = format_ident!("{name}FormResult");

  let fields = match ast {
    syn::DeriveInput {
      data:
        syn::Data::Struct(syn::DataStruct {
          fields: syn::Fields::Named(syn::FieldsNamed { named: fields, .. }),
          ..
        }),
      ..
    } => fields,
    _ => unimplemented!("derive(Builder) only supports structs with named fields"),
  };

  let form_fields = fields.iter().filter_map(|field| {
    let field = field.clone();
    let ty = extract_field_data_type(&field.ty)?;
    let id = field.ident.unwrap();
    let res = quote! {
        #id: #ty
    };
    Some(res)
  });

  let form_unwraps = fields
    .iter()
    .filter(|f| extract_field_data_type(&f.ty).is_some())
    .map(|field| {
      let field = field.clone();
      let id = field.ident.unwrap();
      quote! { #id: value.#id.value.clone()? }
    });

  let gen = quote! {
    struct #form_name {
      #(#form_fields),*
    }
    impl TryFrom<&#name> for #form_name {
        type Error = String;
        fn try_from(value: &LyraSettings) -> Result<Self, Self::Error> {
            Ok(#form_name {
                #(#form_unwraps),*
            })
        }
    }
  };
  gen.into()
}

fn extract_field_data_type(ty: &syn::Type) -> Option<syn::Type> {
  // Get the object pulled out, there's a bunch of data here
  let segments = match ty {
    syn::Type::Path(syn::TypePath {
      path: syn::Path { segments, .. },
      ..
    }) if segments.len() == 1 => segments.clone(),
    _ => return None,
  };

  // Then find the generic type inside if the type is FieldData
  // Technically there can be many args but let's just look for one
  let args = match &segments[0] {
    syn::PathSegment {
      ident,
      arguments:
        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }),
    } if ident == "FieldData" && args.len() == 1 => args,
    _ => return None,
  };

  // Now extract the inner type, which technically could have things
  // like lifetimes but we won't go there
  let ty = match &args[0] {
    syn::GenericArgument::Type(t) => t,
    _ => return None,
  };

  Some(ty.clone())
}
