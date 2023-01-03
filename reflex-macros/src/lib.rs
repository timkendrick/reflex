// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Chris Campbell <c.campbell@mwam.com> https://github.com/c-campbell-mwam
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use pointer_iter::pointer_iter;
use proc_macro::TokenStream;

mod blanket_trait;
mod dispatcher;
mod matcher;
mod named;
mod pointer_iter;
mod task_factory_enum;
mod utils;

#[proc_macro_derive(Matcher, attributes(matcher))]
pub fn derive_matcher(input: TokenStream) -> TokenStream {
    matcher::execute(input)
}

#[proc_macro_derive(PointerIter)]
pub fn derive_pointer_iter(input: TokenStream) -> TokenStream {
    pointer_iter(input)
}

#[proc_macro_derive(Named)]
pub fn named_matcher(input: TokenStream) -> TokenStream {
    named::execute(input)
}

#[proc_macro]
pub fn dispatcher(input: TokenStream) -> TokenStream {
    dispatcher::execute(input)
}

#[proc_macro]
pub fn blanket_trait(input: TokenStream) -> TokenStream {
    blanket_trait::execute(input)
}

#[proc_macro]
pub fn task_factory_enum(input: TokenStream) -> TokenStream {
    task_factory_enum::execute(input)
}
