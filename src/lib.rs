extern crate quote;
extern crate proc_macro;
extern crate syn;

use quote::quote;
use proc_macro::TokenStream;
use syn::{parse_macro_input, parse_quote, parse_str, Fields, ItemStruct, Lit, MacroDelimiter, Meta};

#[proc_macro_attribute]
pub fn remote_converter(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut struct_ = parse_macro_input!(item as ItemStruct);

    // Find the path/name of the remote type.
    let mut remote_struct_name = None;
    for attr in struct_.attrs.iter() {
        if let Meta::List(metalist) = &attr.meta {
            if metalist.path.is_ident("serde") {
                if let MacroDelimiter::Paren(_) = metalist.delimiter {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("remote") {
                            let expr: Lit = meta.value()?.parse()?;
                            if let Lit::Str(expr) = expr {
                                let path: syn::Path = parse_str(&expr.value())?;
                                remote_struct_name = Some(path);
                            } else {
                                return Err(syn::Error::new_spanned(expr, "expected string literal"));
                            }
                        }
                        Ok(())
                    }).expect("Cannot parse #[serde(remote = \"..\")]");
                }
            }
        }
    }
    let remote_struct_name = remote_struct_name.expect("Cannot #[serde(remote = \"..\")]");

    // Struct name and generics
    let struct_name = struct_.ident.clone();
    let (impl_generics, ty_generics, where_clause) = struct_.generics.split_for_impl();

    // Fields inside struct.
    let fields = match struct_.fields.clone() {
        Fields::Named(named) => named.named
            .into_iter()
            .map(|field| (field.ident, field.ty))
            .collect::<Vec<_>>(),
        _ => panic!("remote_convereted can only be used with structs with named fields"),
    };
    let field_names = fields.iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    let field_types = fields.into_iter()
        .map(|(_, ty)| ty)
        .collect::<Vec<_>>();

    // Update struct definition with #[serde(getter = ...)]
    for field in &mut struct_.fields {
        let getter = format!("Self::{}", field.ident.as_ref().unwrap());
        field.attrs.push(parse_quote!(#[serde(getter = #getter)]))
    }

    // Produce new struct definition and the impl block for getters.
    let tokens = quote! {
        #struct_

        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(fn #field_names (i: & #remote_struct_name #ty_generics) -> & #field_types {
                &unsafe { std::mem::transmute::<_, &Self>(i) }.#field_names
            })*
        }
    };

    tokens.into()
}
