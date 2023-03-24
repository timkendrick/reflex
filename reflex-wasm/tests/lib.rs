// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use reflex::core::{Expression, ExpressionFactory, HeapAllocator};
use reflex_wasm::compiler::CompilerOptions;

mod compiler;

pub trait WasmTestScenario<T: Expression, TFactory: ExpressionFactory<T>> {
    fn options(&self) -> CompilerOptions {
        Default::default()
    }
    fn input(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> T;
    fn state(&self, factory: &TFactory, allocator: &impl HeapAllocator<T>) -> Vec<(T::Signal, T)> {
        let _ = factory;
        let _ = allocator;
        Default::default()
    }
    fn expected(
        &self,
        factory: &TFactory,
        allocator: &impl HeapAllocator<T>,
    ) -> (T, Vec<T::Signal>);
}
