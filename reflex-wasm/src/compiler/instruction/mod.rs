// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use walrus::Module;

use crate::compiler::{
    error::TypedStackError,
    wasm::{GenerateWasm, WasmGeneratorBindings, WasmGeneratorOptions, WasmGeneratorResult},
    CompilerStack, TypedCompilerBlock,
};

/// 'Low-level' instructions (generic stack machine instructions)
pub mod core;
/// 'High-level' instructions (these make assumptions about the runtime environment - i.e. state, dependencies etc)
pub mod runtime;

/// Virtual stack machine instruction set
///
/// This is a proprietary instruction set for a stack machine interpreter, with the intended goal of providing some
/// useful high-level operations while still being straightforward to translate to WebAssembly instructions.
///
/// The instruction set assumes three separate stacks, which correspond to the WebAssembly stack, WebAssembly function locals and WebAssembly blocks respectively:
///   1. The operand stack, which stores function arguments and return values
///   2. The lexical scope stack, which is used store local variables (similar to virtual registers).
///     Each lexical scope stack frame holds a single variable, which can be loaded onto the operand stack from anywhere within
///     the lexical scope. Variables stored in the lexical scopes cannot be mutated (although mutation can be simulated
///     by using a SSA-style approach).
///   3. The control flow block stack, which stores nested code blocks. Nested instructions can break out of enclosing blocks.
#[derive(PartialEq, Clone, Hash, Debug)]
pub enum CompiledInstruction {
    Const(core::Const),
    Duplicate(core::Duplicate),
    Drop(core::Drop),
    ScopeStart(core::ScopeStart),
    ScopeEnd(core::ScopeEnd),
    GetScopeValue(core::GetScopeValue),
    Block(core::Block),
    Break(core::Break),
    ConditionalBreak(core::ConditionalBreak),
    If(core::If),
    Select(core::Select),
    Eq(core::Eq),
    Ne(core::Ne),
    ReadHeapValue(core::ReadHeapValue),
    WriteHeapValue(core::WriteHeapValue),
    NullPointer(runtime::NullPointer),
    DeclareVariable(runtime::DeclareVariable),
    LoadStateValue(runtime::LoadStateValue),
    CallRuntimeBuiltin(runtime::CallRuntimeBuiltin),
    CallStdlib(runtime::CallStdlib),
    CallCompiledFunction(runtime::CallCompiledFunction),
    CallDynamic(runtime::CallDynamic),
    Evaluate(runtime::Evaluate),
    Apply(runtime::Apply),
    CollectSignals(runtime::CollectSignals),
    BreakOnSignal(runtime::BreakOnSignal),
}

impl TypedCompilerBlock for CompiledInstruction {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        match self {
            Self::Const(inner) => inner.get_type(stack),
            Self::Duplicate(inner) => inner.get_type(stack),
            Self::Drop(inner) => inner.get_type(stack),
            Self::ScopeStart(inner) => inner.get_type(stack),
            Self::ScopeEnd(inner) => inner.get_type(stack),
            Self::GetScopeValue(inner) => inner.get_type(stack),
            Self::Block(inner) => inner.get_type(stack),
            Self::Break(inner) => inner.get_type(stack),
            Self::ConditionalBreak(inner) => inner.get_type(stack),
            Self::If(inner) => inner.get_type(stack),
            Self::Select(inner) => inner.get_type(stack),
            Self::Eq(inner) => inner.get_type(stack),
            Self::Ne(inner) => inner.get_type(stack),
            Self::ReadHeapValue(inner) => inner.get_type(stack),
            Self::WriteHeapValue(inner) => inner.get_type(stack),
            Self::NullPointer(inner) => inner.get_type(stack),
            Self::DeclareVariable(inner) => inner.get_type(stack),
            Self::LoadStateValue(inner) => inner.get_type(stack),
            Self::CallRuntimeBuiltin(inner) => inner.get_type(stack),
            Self::CallStdlib(inner) => inner.get_type(stack),
            Self::CallCompiledFunction(inner) => inner.get_type(stack),
            Self::CallDynamic(inner) => inner.get_type(stack),
            Self::Evaluate(inner) => inner.get_type(stack),
            Self::Apply(inner) => inner.get_type(stack),
            Self::CollectSignals(inner) => inner.get_type(stack),
            Self::BreakOnSignal(inner) => inner.get_type(stack),
        }
    }
}

impl GenerateWasm for CompiledInstruction {
    fn emit_wasm(
        &self,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        match self {
            Self::Const(inner) => inner.emit_wasm(module, bindings, options),
            Self::Duplicate(inner) => inner.emit_wasm(module, bindings, options),
            Self::Drop(inner) => inner.emit_wasm(module, bindings, options),
            Self::ScopeStart(inner) => inner.emit_wasm(module, bindings, options),
            Self::ScopeEnd(inner) => inner.emit_wasm(module, bindings, options),
            Self::GetScopeValue(inner) => inner.emit_wasm(module, bindings, options),
            Self::Block(inner) => inner.emit_wasm(module, bindings, options),
            Self::Break(inner) => inner.emit_wasm(module, bindings, options),
            Self::ConditionalBreak(inner) => inner.emit_wasm(module, bindings, options),
            Self::If(inner) => inner.emit_wasm(module, bindings, options),
            Self::Select(inner) => inner.emit_wasm(module, bindings, options),
            Self::Eq(inner) => inner.emit_wasm(module, bindings, options),
            Self::Ne(inner) => inner.emit_wasm(module, bindings, options),
            Self::ReadHeapValue(inner) => inner.emit_wasm(module, bindings, options),
            Self::WriteHeapValue(inner) => inner.emit_wasm(module, bindings, options),
            Self::NullPointer(inner) => inner.emit_wasm(module, bindings, options),
            Self::DeclareVariable(inner) => inner.emit_wasm(module, bindings, options),
            Self::LoadStateValue(inner) => inner.emit_wasm(module, bindings, options),
            Self::CallRuntimeBuiltin(inner) => inner.emit_wasm(module, bindings, options),
            Self::CallStdlib(inner) => inner.emit_wasm(module, bindings, options),
            Self::CallCompiledFunction(inner) => inner.emit_wasm(module, bindings, options),
            Self::CallDynamic(inner) => inner.emit_wasm(module, bindings, options),
            Self::Evaluate(inner) => inner.emit_wasm(module, bindings, options),
            Self::Apply(inner) => inner.emit_wasm(module, bindings, options),
            Self::CollectSignals(inner) => inner.emit_wasm(module, bindings, options),
            Self::BreakOnSignal(inner) => inner.emit_wasm(module, bindings, options),
        }
    }
}

impl std::fmt::Display for CompiledInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Const(inner) => std::fmt::Debug::fmt(inner, f),
            Self::ReadHeapValue(inner) => std::fmt::Debug::fmt(inner, f),
            Self::WriteHeapValue(inner) => std::fmt::Debug::fmt(inner, f),
            Self::Duplicate(inner) => std::fmt::Debug::fmt(inner, f),
            Self::Drop(inner) => std::fmt::Debug::fmt(inner, f),
            Self::ScopeStart(inner) => std::fmt::Debug::fmt(inner, f),
            Self::ScopeEnd(inner) => std::fmt::Debug::fmt(inner, f),
            Self::GetScopeValue(inner) => std::fmt::Debug::fmt(inner, f),
            Self::Block(inner) => std::fmt::Debug::fmt(inner, f),
            Self::Break(inner) => std::fmt::Debug::fmt(inner, f),
            Self::ConditionalBreak(inner) => std::fmt::Debug::fmt(inner, f),
            Self::If(inner) => std::fmt::Debug::fmt(inner, f),
            Self::Select(inner) => std::fmt::Debug::fmt(inner, f),
            Self::Eq(inner) => std::fmt::Debug::fmt(inner, f),
            Self::Ne(inner) => std::fmt::Debug::fmt(inner, f),
            Self::NullPointer(inner) => std::fmt::Debug::fmt(inner, f),
            Self::DeclareVariable(inner) => std::fmt::Debug::fmt(inner, f),
            Self::LoadStateValue(inner) => std::fmt::Debug::fmt(inner, f),
            Self::CallRuntimeBuiltin(inner) => std::fmt::Debug::fmt(inner, f),
            Self::CallStdlib(inner) => std::fmt::Debug::fmt(inner, f),
            Self::CallCompiledFunction(inner) => std::fmt::Debug::fmt(inner, f),
            Self::CallDynamic(inner) => std::fmt::Debug::fmt(inner, f),
            Self::Evaluate(inner) => std::fmt::Debug::fmt(inner, f),
            Self::Apply(inner) => std::fmt::Debug::fmt(inner, f),
            Self::CollectSignals(inner) => std::fmt::Debug::fmt(inner, f),
            Self::BreakOnSignal(inner) => std::fmt::Debug::fmt(inner, f),
        }
    }
}

impl From<self::core::Const> for CompiledInstruction {
    fn from(value: self::core::Const) -> Self {
        Self::Const(value)
    }
}
impl From<self::core::ReadHeapValue> for CompiledInstruction {
    fn from(value: self::core::ReadHeapValue) -> Self {
        Self::ReadHeapValue(value)
    }
}
impl From<self::core::WriteHeapValue> for CompiledInstruction {
    fn from(value: self::core::WriteHeapValue) -> Self {
        Self::WriteHeapValue(value)
    }
}
impl From<self::core::Duplicate> for CompiledInstruction {
    fn from(value: self::core::Duplicate) -> Self {
        Self::Duplicate(value)
    }
}
impl From<self::core::Drop> for CompiledInstruction {
    fn from(value: self::core::Drop) -> Self {
        Self::Drop(value)
    }
}
impl From<self::core::ScopeStart> for CompiledInstruction {
    fn from(value: self::core::ScopeStart) -> Self {
        Self::ScopeStart(value)
    }
}
impl From<self::core::ScopeEnd> for CompiledInstruction {
    fn from(value: self::core::ScopeEnd) -> Self {
        Self::ScopeEnd(value)
    }
}
impl From<self::core::GetScopeValue> for CompiledInstruction {
    fn from(value: self::core::GetScopeValue) -> Self {
        Self::GetScopeValue(value)
    }
}
impl From<self::core::Block> for CompiledInstruction {
    fn from(value: self::core::Block) -> Self {
        Self::Block(value)
    }
}
impl From<self::core::Break> for CompiledInstruction {
    fn from(value: self::core::Break) -> Self {
        Self::Break(value)
    }
}
impl From<self::core::ConditionalBreak> for CompiledInstruction {
    fn from(value: self::core::ConditionalBreak) -> Self {
        Self::ConditionalBreak(value)
    }
}
impl From<self::core::If> for CompiledInstruction {
    fn from(value: self::core::If) -> Self {
        Self::If(value)
    }
}
impl From<self::core::Select> for CompiledInstruction {
    fn from(value: self::core::Select) -> Self {
        Self::Select(value)
    }
}
impl From<self::core::Eq> for CompiledInstruction {
    fn from(value: self::core::Eq) -> Self {
        Self::Eq(value)
    }
}
impl From<self::core::Ne> for CompiledInstruction {
    fn from(value: self::core::Ne) -> Self {
        Self::Ne(value)
    }
}
impl From<self::runtime::NullPointer> for CompiledInstruction {
    fn from(value: self::runtime::NullPointer) -> Self {
        Self::NullPointer(value)
    }
}
impl From<self::runtime::DeclareVariable> for CompiledInstruction {
    fn from(value: self::runtime::DeclareVariable) -> Self {
        Self::DeclareVariable(value)
    }
}
impl From<self::runtime::LoadStateValue> for CompiledInstruction {
    fn from(value: self::runtime::LoadStateValue) -> Self {
        Self::LoadStateValue(value)
    }
}
impl From<self::runtime::CallRuntimeBuiltin> for CompiledInstruction {
    fn from(value: self::runtime::CallRuntimeBuiltin) -> Self {
        Self::CallRuntimeBuiltin(value)
    }
}
impl From<self::runtime::CallStdlib> for CompiledInstruction {
    fn from(value: self::runtime::CallStdlib) -> Self {
        Self::CallStdlib(value)
    }
}
impl From<self::runtime::CallCompiledFunction> for CompiledInstruction {
    fn from(value: self::runtime::CallCompiledFunction) -> Self {
        Self::CallCompiledFunction(value)
    }
}
impl From<self::runtime::CallDynamic> for CompiledInstruction {
    fn from(value: self::runtime::CallDynamic) -> Self {
        Self::CallDynamic(value)
    }
}
impl From<self::runtime::Evaluate> for CompiledInstruction {
    fn from(value: self::runtime::Evaluate) -> Self {
        Self::Evaluate(value)
    }
}
impl From<self::runtime::Apply> for CompiledInstruction {
    fn from(value: self::runtime::Apply) -> Self {
        Self::Apply(value)
    }
}
impl From<self::runtime::CollectSignals> for CompiledInstruction {
    fn from(value: self::runtime::CollectSignals) -> Self {
        Self::CollectSignals(value)
    }
}
impl From<self::runtime::BreakOnSignal> for CompiledInstruction {
    fn from(value: self::runtime::BreakOnSignal) -> Self {
        Self::BreakOnSignal(value)
    }
}
