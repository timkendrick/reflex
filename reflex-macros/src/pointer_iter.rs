// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
extern crate proc_macro;
use quote::{quote, ToTokens};
use syn;

use proc_macro::TokenStream;

pub fn pointer_iter(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    // Build the impl
    let gen = impl_pointer_iter(&ast);

    // Return the generated impl
    gen
}

fn impl_pointer_iter(ast: &syn::DeriveInput) -> TokenStream {
    match &ast.data {
        syn::Data::Struct(s) => impl_pointer_iter_struct(s, &ast.ident, &ast.vis),
        syn::Data::Enum(_) => panic!("Derive macro not supported for enums"),
        syn::Data::Union(_) => panic!("Derive macro not supported for unions"),
    }
}

fn impl_pointer_iter_struct(
    struct_data: &syn::DataStruct,
    name: &syn::Ident,
    visibility: &syn::Visibility,
) -> TokenStream {
    let fields = match &struct_data.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .cloned()
            .filter(|field| field.ty.to_token_stream().to_string() == "ArenaPointer")
            .map(|field| field.ident.unwrap())
            .collect(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .cloned()
            .filter(|field| field.ty.to_token_stream().to_string() == "ArenaPointer")
            .map(|field| field.ident.unwrap())
            .collect(),
        syn::Fields::Unit => vec![],
    };

    let num_fields = fields.len();

    let iter_name = syn::Ident::new(&format!("{}PointerIter", name), name.span());

    quote!(

        #visibility type #iter_name = ::std::array::IntoIter<crate::ArenaPointer, #num_fields>;

        #[automatically_derived]
        impl<A: crate::Arena> crate::PointerIter for crate::ArenaRef<#name, A> {
            type Iter<'a> = #iter_name
            where
                Self: 'a;

            fn iter<'a>(&'a self) -> Self::Iter<'a>
            where
                Self: 'a
            {
                [#(self.inner_pointer(|term| &term.#fields)),*].into_iter()
            }
        }
    )
    .into()
}
