// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    collections::{hash_map::Entry, HashMap, LinkedList},
    iter::once,
};

use reflex::{
    core::{ArgType, Arity, GraphNode, NodeId, StackOffset},
    hash::IntMap,
};
use reflex_utils::Stack;

use crate::{
    allocator::{Arena, ArenaAllocator, ArenaIterator, VecAllocator},
    compiler::{
        error::{
            CompilerError, InvalidBlockResultTypeError, InvalidLexicalScopeValueTypeError,
            InvalidOperandStackValueTypesError, TypedStackError,
        },
        instruction::CompiledInstruction,
        runtime::builtin::RuntimeBuiltin,
    },
    hash::{TermHashState, TermHasher, TermSize},
    serialize::{Serialize, SerializerState},
    stdlib::Stdlib,
    term_type::*,
    term_type::{
        list::{collect_compiled_list_values, Strictness},
        TermType, TermTypeDiscriminants, WasmExpression,
    },
    ArenaPointer, ArenaRef, Array, IntoArenaRefIterator, PointerIter, Term,
};

pub mod error;
pub mod instruction;
pub mod runtime;
pub mod wasm;

pub type CompilerResult<A> = Result<CompiledBlock, CompilerError<A>>;

pub trait TypedCompilerBlock {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError>;
}

pub trait Internable {
    /// Many terms are capable of being serialized directly into a heap memory snapshot, allowing them to be referenced
    /// directly from the VM heap at runtime via a static pointer address rather than having to be dynamically
    /// constructed at runtime via a potentially costly sequence of compiled interpreter instructions.
    ///
    /// This method determines whether the current term should be serialized into a memory snapshot, or compiled into a
    /// sequence of interpreter instructions.
    ///
    /// - Atomic 'pure data' terms whose evaluation does not affect control flow should always be inlined, as this will
    /// give the same result as constructing them dynamically via compiled interpreter instructions
    /// - Terms such as lambdas or variable declarations can take advantage of compiler optimizations and are therefore
    /// typically not inlined
    /// - Some terms such as signals or function applications can cause control flow to break out of the current scope
    /// when evaluated in a strict context, and therefore should be conditionally inlined depending on the desired
    /// eagerness in order not to lose their additional behavior when evaluated in a strict context
    fn should_intern(&self, eager: ArgType) -> bool;
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypeSignature {
    pub params: ParamsSignature,
    pub results: ParamsSignature,
}

impl TypeSignature {
    pub fn new(params: impl Into<ParamsSignature>, results: impl Into<ParamsSignature>) -> Self {
        Self {
            params: params.into(),
            results: results.into(),
        }
    }
}

impl std::fmt::Display for TypeSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.params, self.results)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ParamsSignature {
    Void,
    Single(ValueType),
    Pair(ValueType, ValueType),
    Triple(ValueType, ValueType, ValueType),
    Multiple(Vec<ValueType>),
}

impl ParamsSignature {
    pub fn iter(&self) -> ParamsSignatureIter<'_> {
        ParamsSignatureIter::new(&self)
    }
    pub fn get(&self, index: usize) -> Option<ValueType> {
        match self {
            &ParamsSignature::Void => None,
            &ParamsSignature::Single(inner) => match index {
                0 => Some(inner),
                _ => None,
            },
            &ParamsSignature::Pair(first, second) => match index {
                0 => Some(first),
                1 => Some(second),
                _ => None,
            },
            &ParamsSignature::Triple(first, second, third) => match index {
                0 => Some(first),
                1 => Some(second),
                2 => Some(third),
                _ => None,
            },
            ParamsSignature::Multiple(values) => values.get(index).copied(),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Void => 0,
            Self::Single(_) => 1,
            Self::Pair(_, _) => 2,
            Self::Triple(_, _, _) => 3,
            Self::Multiple(values) => values.len(),
        }
    }
    pub fn append(self, iter: impl IntoIterator<Item = ValueType>) -> Self {
        iter.into_iter().fold(self, |result, item| match result {
            ParamsSignature::Void => ParamsSignature::Single(item),
            ParamsSignature::Single(item1) => ParamsSignature::Pair(item1, item),
            ParamsSignature::Pair(item1, item2) => ParamsSignature::Triple(item1, item2, item),
            ParamsSignature::Triple(item1, item2, item3) => {
                ParamsSignature::Multiple(vec![item1, item2, item3, item])
            }
            ParamsSignature::Multiple(mut items) => {
                items.push(item);
                ParamsSignature::Multiple(items)
            }
        })
    }
}

impl From<()> for ParamsSignature {
    fn from(_value: ()) -> Self {
        Self::Void
    }
}

impl From<ValueType> for ParamsSignature {
    fn from(value: ValueType) -> Self {
        Self::Single(value)
    }
}

impl From<(ValueType, ValueType)> for ParamsSignature {
    fn from((first, second): (ValueType, ValueType)) -> Self {
        Self::Pair(first, second)
    }
}

impl From<(ValueType, ValueType, ValueType)> for ParamsSignature {
    fn from((first, second, third): (ValueType, ValueType, ValueType)) -> Self {
        Self::Triple(first, second, third)
    }
}

impl From<Option<ValueType>> for ParamsSignature {
    fn from(value: Option<ValueType>) -> Self {
        if let Some(value) = value {
            Self::Single(value)
        } else {
            Self::Void
        }
    }
}

impl IntoIterator for ParamsSignature {
    type Item = ValueType;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter().collect::<Vec<_>>().into_iter()
    }
}

impl FromIterator<ValueType> for ParamsSignature {
    fn from_iter<T: IntoIterator<Item = ValueType>>(iter: T) -> Self {
        ParamsSignature::Void.append(iter)
    }
}

impl Extend<ValueType> for ParamsSignature {
    fn extend<T: IntoIterator<Item = ValueType>>(&mut self, iter: T) {
        let existing = std::mem::replace(self, ParamsSignature::Void);
        let updated = existing.append(iter);
        *self = updated
    }
}

impl std::fmt::Display for ParamsSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({})",
            self.iter()
                .map(|value_type| format!("{}", value_type))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParamsSignatureIter<'a> {
    signature: &'a ParamsSignature,
    start_index: usize,
    end_index: usize,
}

impl<'a> ParamsSignatureIter<'a> {
    fn new(signature: &'a ParamsSignature) -> Self {
        Self {
            start_index: 0,
            end_index: signature.len(),
            signature,
        }
    }
}

impl<'a> DoubleEndedIterator for ParamsSignatureIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start_index >= self.end_index {
            return None;
        }
        let item = self.signature.get(self.end_index - 1);
        self.end_index -= 1;
        item
    }
}

impl<'a> Iterator for ParamsSignatureIter<'a> {
    type Item = ValueType;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start_index >= self.end_index {
            return None;
        }
        let item = self.signature.get(self.start_index);
        self.start_index += 1;
        item
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.end_index - self.start_index;
        (length, Some(length))
    }
}

impl<'a> ExactSizeIterator for ParamsSignatureIter<'a> {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstValue {
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    // Pointer to an address within a linear memory
    HeapPointer(ArenaPointer),
    // Pointer to an indirect function call wrapper
    FunctionPointer(FunctionPointer),
}

impl std::hash::Hash for ConstValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::I32(value) => std::hash::Hash::hash(value, state),
            Self::U32(value) => std::hash::Hash::hash(value, state),
            Self::I64(value) => std::hash::Hash::hash(value, state),
            Self::U64(value) => std::hash::Hash::hash(value, state),
            Self::F32(value) => std::hash::Hash::hash(
                &if value.is_nan() { f32::NAN } else { *value }.to_bits(),
                state,
            ),
            Self::F64(value) => std::hash::Hash::hash(
                &if value.is_nan() { f64::NAN } else { *value }.to_bits(),
                state,
            ),
            Self::HeapPointer(value) => std::hash::Hash::hash(value, state),
            Self::FunctionPointer(value) => std::hash::Hash::hash(value, state),
        }
    }
}

impl ConstValue {
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::I32(_) => ValueType::I32,
            Self::U32(_) => ValueType::U32,
            Self::I64(_) => ValueType::I64,
            Self::U64(_) => ValueType::U64,
            Self::F32(_) => ValueType::F32,
            Self::F64(_) => ValueType::F64,
            Self::HeapPointer(_) => ValueType::HeapPointer,
            Self::FunctionPointer(_) => ValueType::FunctionPointer,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionPointer {
    Stdlib(Stdlib),
    Lambda(CompiledFunctionId),
}
impl From<Stdlib> for FunctionPointer {
    fn from(value: Stdlib) -> Self {
        Self::Stdlib(value)
    }
}
impl From<CompiledFunctionId> for FunctionPointer {
    fn from(value: CompiledFunctionId) -> Self {
        Self::Lambda(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    HeapPointer,
    FunctionPointer,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I32 => write!(f, "i32"),
            Self::U32 => write!(f, "u32"),
            Self::I64 => write!(f, "i64"),
            Self::U64 => write!(f, "u64"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::HeapPointer => write!(f, "<pointer>"),
            Self::FunctionPointer => write!(f, "<function>"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CompilerOptions {
    pub lazy_record_values: ArgType,
    pub lazy_list_items: ArgType,
    pub lazy_variable_initializers: ArgType,
    pub lazy_function_args: bool,
    pub lazy_lambda_args: ArgType,
    pub lazy_constructors: ArgType,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            lazy_record_values: ArgType::Strict,
            lazy_list_items: ArgType::Strict,
            lazy_variable_initializers: ArgType::Eager,
            lazy_function_args: false,
            lazy_lambda_args: ArgType::Strict,
            lazy_constructors: ArgType::Strict,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompiledBlockBuilder {
    result: Result<CompiledBlock, TypedStackError>,
    stack: CompilerStack,
}

impl CompiledBlockBuilder {
    pub fn new(stack: CompilerStack) -> Self {
        Self {
            stack,
            result: Ok(CompiledBlock::default()),
        }
    }
}

impl CompiledBlockBuilder {
    #[must_use]
    pub fn push(self, instruction: impl Into<CompiledInstruction>) -> Self {
        let Self { result, stack } = self;
        let (result, stack) = result
            .and_then(|mut instructions| {
                let instruction = Into::<CompiledInstruction>::into(instruction);
                instruction.get_type(&stack).map(|updated_stack| {
                    instructions.push(instruction);
                    (Ok(instructions), updated_stack)
                })
            })
            .unwrap_or_else(|err| (Err(err), stack));
        Self { result, stack }
    }
    #[must_use]
    fn append_block(self, block: CompiledBlock) -> Self {
        let Self { result, stack } = self;
        let (result, stack) = result
            .and_then(|mut instructions| {
                block.get_type(&stack).map(|updated_stack| {
                    instructions.extend(block);
                    (Ok(instructions), updated_stack)
                })
            })
            .unwrap_or_else(|err| (Err(err), stack));
        Self { result, stack }
    }
    #[must_use]
    pub fn append_inner<E>(
        self,
        factory: impl FnOnce(CompilerStack) -> Result<CompiledBlock, E>,
    ) -> Result<Self, E> {
        let block = factory(self.stack.clone())?;
        Ok(self.append_block(block))
    }
    pub fn finish<E: From<TypedStackError>>(self) -> Result<CompiledBlock, E> {
        self.result.map_err(Into::into)
    }
    pub(crate) fn into_parts(self) -> Result<(CompiledBlock, CompilerStack), TypedStackError> {
        let Self { stack, result } = self;
        result.map(|block| (block, stack))
    }
}

#[derive(Default, PartialEq, Clone, Hash, Debug)]
pub struct CompiledBlock {
    instructions: LinkedList<CompiledInstruction>,
}

impl CompiledBlock {
    fn push(&mut self, instruction: impl Into<CompiledInstruction>) {
        self.instructions.push_back(instruction.into())
    }
    fn extend(&mut self, instructions: CompiledBlock) {
        let CompiledBlock { instructions } = instructions;
        self.instructions.extend(instructions)
    }
    pub fn iter(&self) -> std::collections::linked_list::Iter<'_, CompiledInstruction> {
        self.instructions.iter()
    }
}

impl IntoIterator for CompiledBlock {
    type Item = CompiledInstruction;
    type IntoIter = std::collections::linked_list::IntoIter<CompiledInstruction>;
    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}

impl<'a> IntoIterator for &'a CompiledBlock {
    type Item = &'a CompiledInstruction;
    type IntoIter = std::collections::linked_list::Iter<'a, CompiledInstruction>;
    fn into_iter(self) -> Self::IntoIter {
        self.instructions.iter()
    }
}

impl TypedCompilerBlock for CompiledBlock {
    fn get_type(&self, stack: &CompilerStack) -> Result<CompilerStack, TypedStackError> {
        self.instructions
            .iter()
            .fold(Ok(stack.clone()), |result, instruction| {
                let stack = result?;
                instruction.get_type(&stack)
            })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledFunction {
    pub num_args: usize,
    pub body: Vec<CompiledInstruction>,
}

pub struct CompilerState {
    pub(crate) serializer_state: SerializerState,
    pub(crate) heap: VecAllocator,
    pub(crate) compiled_lambdas: HashMap<CompiledFunctionId, CompiledLambda>,
    pub(crate) compiled_thunks: HashMap<TermHashState, CompiledThunk>,
}

pub struct CompiledLambda {
    pub params: ParamsSignature,
    pub body: CompiledBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompiledFunctionId(TermHashState);

impl<'a, A: Arena> From<&'a ArenaRef<LambdaTerm, A>> for CompiledFunctionId {
    fn from(value: &'a ArenaRef<LambdaTerm, A>) -> Self {
        value.read_value(|term| Self(TermHasher::default().hash(term, &value.arena).finish()))
    }
}

impl From<TermHashState> for CompiledFunctionId {
    fn from(value: TermHashState) -> Self {
        Self(value)
    }
}

impl From<CompiledFunctionId> for TermHashState {
    fn from(value: CompiledFunctionId) -> Self {
        let CompiledFunctionId(value) = value;
        value
    }
}

impl PartialOrd for CompiledFunctionId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        u64::from(self.0).partial_cmp(&u64::from(other.0))
    }
}

impl Ord for CompiledFunctionId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        u64::from(self.0).cmp(&u64::from(other.0))
    }
}

impl std::fmt::Display for CompiledFunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn:{:#08}>", u64::from(self.0))
    }
}

impl CompilerState {
    pub fn from_heap_snapshot<T: TermSize>(bytes: &[u8]) -> Self
    where
        for<'a> ArenaRef<T, &'a VecAllocator>: NodeId,
    {
        let heap = VecAllocator::from_bytes(bytes);
        Self {
            serializer_state: {
                let arena = &heap;
                let start_offset = arena.start_offset();
                let end_offset = arena.end_offset();
                let next_offset = end_offset;
                let allocated_terms = ArenaIterator::<T, _>::new(arena, start_offset, end_offset)
                    .as_arena_refs::<T>(&arena)
                    .map(|term| (term.id(), term.pointer));
                SerializerState::new(allocated_terms, next_offset)
            },
            compiled_lambdas: Default::default(),
            compiled_thunks: Default::default(),
            heap,
        }
    }
    pub fn from_arena<A: Arena + PointerIter + Clone>(arena: &A) -> Self {
        Self::from_heap_values(
            arena
                .iter()
                .map(|pointer| ArenaRef::<Term, _>::new(arena.clone(), pointer)),
        )
    }
    fn from_heap_values<T: Serialize>(values: impl IntoIterator<Item = T>) -> Self {
        let mut destination_arena = VecAllocator::default();
        let next_offset = destination_arena.end_offset();
        let mut serializer_state = SerializerState::new([], next_offset);
        // Serialize all the source terms into the destination arena
        for value in values.into_iter() {
            value.serialize(&mut destination_arena, &mut serializer_state);
        }
        Self {
            serializer_state,
            compiled_lambdas: Default::default(),
            compiled_thunks: Default::default(),
            heap: destination_arena,
        }
    }
    pub fn into_parts(
        self,
    ) -> (
        Vec<u8>,
        HashMap<CompiledFunctionId, CompiledLambda>,
        HashMap<TermHashState, CompiledThunk>,
    ) {
        let Self {
            heap,
            compiled_lambdas,
            compiled_thunks,
            ..
        } = self;
        (heap.into_bytes(), compiled_lambdas, compiled_thunks)
    }
}

pub trait CompileWasm<A: Arena> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A>;
}

#[derive(Default, Clone, Debug)]
pub struct CompilerStack {
    operands: Stack<ValueType>,
    blocks: Stack<ParamsSignature>,
    bindings: CompilerVariableBindings,
}

impl CompilerStack {
    pub fn from_free_variables(
        scope_offsets: impl IntoIterator<Item = (StackOffset, ValueType)>,
    ) -> Self {
        Self {
            operands: Default::default(),
            blocks: Default::default(),
            bindings: CompilerVariableBindings::from_free_variables(scope_offsets),
        }
    }
    pub fn push_operand(&self, value_type: ValueType) -> Self {
        self.push_operands(&ParamsSignature::Single(value_type))
    }
    pub fn pop_operand(&self, value_type: ValueType) -> Result<Self, TypedStackError> {
        self.pop_operands(&ParamsSignature::Single(value_type))
    }
    pub fn assert_operand(&self, value_type: ValueType) -> Result<&Self, TypedStackError> {
        self.assert_operands(&ParamsSignature::Single(value_type))
    }
    pub fn assert_operands(&self, value_types: &ParamsSignature) -> Result<&Self, TypedStackError> {
        let _ = self.pop_operands(value_types)?.push_operands(value_types);
        Ok(self)
    }
    pub fn push_operands(&self, value_types: &ParamsSignature) -> Self {
        Self {
            operands: self.operands.append(value_types.iter()),
            blocks: self.blocks.clone(),
            bindings: self.bindings.clone(),
        }
    }
    pub fn pop_operands(&self, value_types: &ParamsSignature) -> Result<Self, TypedStackError> {
        let updated_operands =
            value_types
                .iter()
                .rev()
                .fold(Ok(self.operands.clone()), |results, value_type| {
                    let operands = results?;
                    let (&existing_type, remaining_operands) =
                        operands.peek_and_pop().ok_or_else(|| {
                            TypedStackError::InvalidOperandStackValueTypes(
                                InvalidOperandStackValueTypesError {
                                    expected: value_types.clone(),
                                    received: ParamsSignature::from_iter(
                                        self.operands
                                            .rev()
                                            .copied()
                                            .collect::<Vec<_>>()
                                            .into_iter()
                                            .rev(),
                                    ),
                                },
                            )
                        })?;
                    if existing_type != value_type {
                        return Err(TypedStackError::InvalidOperandStackValueTypes(
                            InvalidOperandStackValueTypesError {
                                expected: value_types.clone(),
                                received: ParamsSignature::from_iter(
                                    self.operands
                                        .rev()
                                        .copied()
                                        .collect::<Vec<_>>()
                                        .into_iter()
                                        .rev(),
                                ),
                            },
                        ));
                    }
                    Ok(remaining_operands)
                })?;
        Ok(Self {
            operands: updated_operands,
            blocks: self.blocks.clone(),
            bindings: self.bindings.clone(),
        })
    }
    pub fn num_operands(&self) -> usize {
        self.operands.len()
    }
    pub fn operands(&self) -> impl Iterator<Item = ValueType> + '_ {
        self.operands.rev().copied().collect::<Vec<_>>().into_iter()
    }
    pub fn enter_scope(&self, value_type: ValueType) -> Self {
        Self {
            operands: self.operands.clone(),
            blocks: self.blocks.clone(),
            bindings: self.bindings.push_internal(value_type),
        }
    }
    pub fn declare_variable(&self, value_type: ValueType) -> Self {
        Self {
            operands: self.operands.clone(),
            blocks: self.blocks.clone(),
            bindings: self.bindings.push_variable(value_type),
        }
    }
    pub fn lookup_variable(&self, scope_offset: StackOffset) -> Option<usize> {
        self.bindings.lookup_variable(scope_offset)
    }
    pub fn leave_scope(&self, value_type: ValueType) -> Result<Self, TypedStackError> {
        let updated_bindings = self
            .bindings
            .pop()
            .ok_or(None)
            .and_then(|(existing_scope, remaining_scopes)| {
                let existing_type = existing_scope.get_type();
                if existing_type == value_type {
                    Ok(remaining_scopes)
                } else {
                    Err(Some(existing_type))
                }
            })
            .map_err(|existing_type| {
                TypedStackError::InvalidLexicalScopeValueType(InvalidLexicalScopeValueTypeError {
                    scope_offset: 0,
                    scope_offset_types: self.bindings.lexical_scopes().collect::<Vec<_>>(),
                    expected: value_type,
                    received: existing_type,
                })
            })?;
        Ok(Self {
            operands: self.operands.clone(),
            blocks: self.blocks.clone(),
            bindings: updated_bindings,
        })
    }
    pub fn assert_lexical_scope(
        &self,
        scope_offset: usize,
        value_type: ValueType,
    ) -> Result<&Self, TypedStackError> {
        self.bindings
            .get_scope_value_type(scope_offset)
            .ok_or(None)
            .and_then(|existing_type| {
                if existing_type == value_type {
                    Ok(self)
                } else {
                    Err(Some(existing_type))
                }
            })
            .map_err(|existing_type| {
                TypedStackError::InvalidLexicalScopeValueType(InvalidLexicalScopeValueTypeError {
                    scope_offset,
                    scope_offset_types: self.bindings.lexical_scopes().collect::<Vec<_>>(),
                    expected: value_type,
                    received: existing_type,
                })
            })
    }
    pub fn bindings(&self) -> &CompilerVariableBindings {
        &self.bindings
    }
    pub fn enter_block(&self, block_type: &TypeSignature) -> Result<Self, TypedStackError> {
        let TypeSignature { params, results } = block_type;
        // Blocks capture only the specified block parameters from the current operand stack
        let inner_stack = Stack::from_iter(params.iter());
        let updated_blocks = self.blocks.push(results.clone());
        Ok(Self {
            operands: inner_stack,
            blocks: updated_blocks,
            bindings: self.bindings.clone(),
        })
    }
    pub fn leave_block(&self, result_type: &ParamsSignature) -> Result<Self, TypedStackError> {
        let updated_blocks = self
            .blocks
            .peek_and_pop()
            .ok_or(None)
            .and_then(|(existing_type, remaining_blocks)| {
                if existing_type == result_type {
                    Ok(remaining_blocks)
                } else {
                    Err(Some(existing_type.clone()))
                }
            })
            .map_err(|existing_type| {
                TypedStackError::InvalidBlockResultType(InvalidBlockResultTypeError {
                    block_types: self.blocks.rev().cloned().collect::<Vec<_>>(),
                    expected: result_type.clone(),
                    received: existing_type,
                })
            })?;
        Ok(Self {
            operands: self.operands.clone(),
            blocks: updated_blocks,
            bindings: self.bindings.clone(),
        })
    }
    pub fn active_block(&self) -> Option<&ParamsSignature> {
        self.blocks.peek()
    }
    pub fn blocks(&self) -> impl Iterator<Item = &'_ ParamsSignature> + '_ {
        self.blocks.rev().collect::<Vec<_>>().into_iter()
    }
}

#[derive(Default, Debug)]
pub struct CompilerVariableBindings {
    /// Free variables captured from the global environment
    free_variables: Option<std::rc::Rc<CompilerFreeVariables>>,
    /// Stack of 'stack machine' lexical scopes, where each stack frame is one of the predetermined frame types
    local_scopes: Stack<CompilerVariableStackFrame>,
}

#[derive(Debug)]
struct CompilerFreeVariables {
    /// Mapping of 'application' variable scopes to actual 'stack machine' stack offsets
    mappings: IntMap<StackOffset, usize>,
    /// Types of the captured variables, ordered from outermost to innermost
    value_types: ParamsSignature,
}

impl Clone for CompilerVariableBindings {
    fn clone(&self) -> Self {
        Self {
            free_variables: self.free_variables.clone(),
            local_scopes: self.local_scopes.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CompilerVariableStackFrame {
    /// Application-level variable declaration
    Variable(ValueType),
    /// Internal stack machine stack frame
    Internal(ValueType),
}

impl CompilerVariableStackFrame {
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Variable(value_type) => *value_type,
            Self::Internal(value_type) => *value_type,
        }
    }
}

impl CompilerVariableBindings {
    pub fn from_free_variables(
        scope_offsets: impl IntoIterator<Item = (StackOffset, ValueType)>,
    ) -> Self {
        // Sort the free variables from outermost (high-offset) to innermost (low-offset)
        // (this is to reflect the order in which function parameter offsets are defined)
        let sorted_scope_offsets = {
            let mut offsets = scope_offsets.into_iter().collect::<Vec<_>>();
            offsets.sort_by_key(|(scope_offset, _)| *scope_offset);
            offsets.reverse();
            offsets
        };
        // Prepare to map the free variable scope offsets to their corresponding compiler stack offsets and value types
        let (scope_offsets, stack_offsets, variable_types) = {
            // Stack offsets are zero-indexed and sequential; the iterator is reversed in order to emit stack offsets
            // from outermost (high-offset) to innermost (low-offset) to match the order of the sorted free variables
            let stack_offsets = (0..sorted_scope_offsets.len()).rev();
            (
                sorted_scope_offsets
                    .iter()
                    .map(|(scope_offset, _)| *scope_offset),
                stack_offsets,
                sorted_scope_offsets
                    .iter()
                    .map(|(_, value_type)| *value_type),
            )
        };
        Self {
            free_variables: Some(std::rc::Rc::new(CompilerFreeVariables {
                // Create a mapping of global free variable offsets to local compiler stack offsets
                mappings: scope_offsets.zip(stack_offsets).collect::<IntMap<_, _>>(),
                value_types: ParamsSignature::from_iter(variable_types),
            })),
            local_scopes: Default::default(),
        }
    }
    pub fn push_internal(&self, value_type: ValueType) -> Self {
        self.push(CompilerVariableStackFrame::Internal(value_type))
    }
    pub fn push_variable(&self, value_type: ValueType) -> Self {
        self.push(CompilerVariableStackFrame::Variable(value_type))
    }
    pub fn get_scope_value_type(&self, offset: usize) -> Option<ValueType> {
        self.lexical_scopes().skip(offset).next()
    }
    pub fn lookup_variable(&self, scope_offset: StackOffset) -> Option<usize> {
        let Self {
            free_variables,
            local_scopes,
        } = self;
        enum ScopeResult {
            /// Iteration has terminated successfully with the given target stack machine stack offset
            Ok(usize),
            /// Iteration has not yet terminated successfully
            Pending {
                /// Application variable scope offset of desired target variable
                target_offset: StackOffset,
                /// Target stack machine stack offset accumulated so far during the iteration
                num_scopes: usize,
            },
        }
        // Iterate backwards through the local scopes to find a potential match
        let result = local_scopes.rev().fold(
            ScopeResult::Pending {
                target_offset: scope_offset,
                num_scopes: 0,
            },
            |result, local_binding| match result {
                // If we have already found our result, nothing more to do
                ScopeResult::Ok(_) => result,
                // Otherwise process the current scope
                ScopeResult::Pending {
                    target_offset,
                    num_scopes,
                } => {
                    match local_binding {
                        // If this is a 'gap' created by an intermediate stack machine scope, continue iterating
                        CompilerVariableStackFrame::Internal(..) => ScopeResult::Pending {
                            target_offset,
                            num_scopes: num_scopes + 1,
                        },
                        CompilerVariableStackFrame::Variable(..) => {
                            // If we have reached the target offset, we have figured out our desired scope offset
                            if target_offset == 0 {
                                return ScopeResult::Ok(num_scopes);
                            }
                            // This is an unrelated local variable declaration, decrement the target scope offset accordingly and continue iterating
                            ScopeResult::Pending {
                                target_offset: target_offset - 1,
                                num_scopes: num_scopes + 1,
                            }
                        }
                    }
                }
            },
        );
        match result {
            // If the result yielded the stack offset of a local stack machine stack entry, return the result
            ScopeResult::Ok(stack_offset) => Some(stack_offset),
            // Otherwise if all local stack values have been exhausted, look up the variable in the global scope
            ScopeResult::Pending {
                target_offset,
                num_scopes,
            } => free_variables.as_ref().and_then(|free_variables| {
                free_variables
                    .mappings
                    .get(&target_offset)
                    .copied()
                    .map(|free_variable_offset| free_variable_offset + num_scopes)
            }),
        }
    }
    fn lexical_scopes(&self) -> impl Iterator<Item = ValueType> + '_ {
        self.local_scopes.rev().map(|frame| frame.get_type()).chain(
            self.free_variables
                .as_ref()
                .into_iter()
                .flat_map(|free_variables| free_variables.value_types.iter().rev()),
        )
    }
    fn push(&self, binding: CompilerVariableStackFrame) -> Self {
        self.append(once(binding))
    }
    fn pop(&self) -> Option<(CompilerVariableStackFrame, Self)> {
        self.local_scopes
            .peek_and_pop()
            .map(|(value, local_scopes)| {
                (
                    *value,
                    Self {
                        free_variables: self.free_variables.clone(),
                        local_scopes,
                    },
                )
            })
    }
    fn append(&self, scopes: impl IntoIterator<Item = CompilerVariableStackFrame>) -> Self {
        let Self {
            free_variables,
            local_scopes,
        } = self;
        Self {
            free_variables: free_variables.clone(),
            local_scopes: local_scopes.append(scopes),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledFunctionCall<'a, A: Arena + Clone, T> {
    pub builtin: T,
    pub target: ArenaRef<Term, A>,
    pub args: &'a CompiledFunctionCallArgs<A>,
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledFunctionCallArgs<A: Arena + Clone> {
    /// Combined set of function arguments, taking into account direct call site arguments and all preceding partially-applied arguments
    pub args: Vec<ArenaRef<TypedTerm<ListTerm>, A>>,
}

impl<A: Arena + Clone> Default for CompiledFunctionCallArgs<A> {
    fn default() -> Self {
        Self {
            args: Default::default(),
        }
    }
}

impl<A: Arena + Clone> CompiledFunctionCallArgs<A> {
    /// If this function call comprises a single argument list (taking into account partially-applied arguments),
    /// determine whether that argument list is eligible for static term inlining
    pub fn as_internable(&self, eager: ArgType) -> Option<ArenaRef<TypedTerm<ListTerm>, A>> {
        if self.args.len() != 1 {
            return None;
        }
        let arg_list = self.args.first()?;
        if !arg_list.as_term().should_intern(eager) {
            return None;
        }
        return Some(arg_list.clone());
    }
    pub fn iter(&self) -> CompiledFunctionCallArgsIter<A> {
        CompiledFunctionCallArgsIter::new(self)
    }
    pub fn len(&self) -> usize {
        self.args
            .iter()
            .fold(0, |acc, arg_list| acc + arg_list.as_inner().len())
    }
}

impl<A: Arena + Clone> std::fmt::Display for CompiledFunctionCallArgs<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_displayed_items = 100;
        let items = self.iter();
        let num_items = items.len();
        write!(
            f,
            "[{}]",
            if num_items <= max_displayed_items {
                items
                    .map(|item| format!("{}", item))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                items
                    .take(max_displayed_items - 1)
                    .map(|item| format!("{}", item))
                    .chain(once(format!(
                        "...{} more items",
                        num_items - (max_displayed_items - 1)
                    )))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledFunctionCallArgsIter<'a, A: Arena + Clone> {
    target: &'a CompiledFunctionCallArgs<A>,
    num_args: usize,
    outer_index: usize,
    inner_index: usize,
    index: usize,
}

impl<'a, A: Arena + Clone> CompiledFunctionCallArgsIter<'a, A> {
    fn new(target: &'a CompiledFunctionCallArgs<A>) -> Self {
        let num_args = target.len();
        Self {
            target,
            num_args,
            outer_index: 0,
            inner_index: 0,
            index: 0,
        }
    }
}

impl<'a, A: Arena + Clone> Iterator for CompiledFunctionCallArgsIter<'a, A> {
    type Item = WasmExpression<A>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.target.args.get(self.outer_index) {
            None => None,
            Some(arg_list) => match arg_list.as_inner().get(self.inner_index) {
                None => {
                    self.outer_index += 1;
                    self.inner_index = 0;
                    self.next()
                }
                Some(inner) => {
                    self.inner_index += 1;
                    self.index += 1;
                    Some(inner)
                }
            },
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.num_args - self.index;
        (length, Some(length))
    }
}

impl<'a, A: Arena + Clone> ExactSizeIterator for CompiledFunctionCallArgsIter<'a, A> {}

impl<A: Arena + Clone> CompileWasm<A> for CompiledFunctionCallArgs<A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        // If there is a single argument list (taking into account partially-applied arguments), and that argument list
        // can make use of static term inlining, delegate to the existing argument list compilation
        let internable_arg_list = if self.args.len() == 1 {
            self.args
                .first()
                .filter(|arg_list| arg_list.as_term().should_intern(ArgType::Strict))
        } else {
            None
        };
        if let Some(internable_arg_list) = internable_arg_list {
            internable_arg_list.as_term().compile(stack, state, options)
        } else {
            // Otherwise if there are partially-applied arguments, compile the combined argument sequence into a list
            // TODO: Investigate chained iterators for partially-applied arguments
            let num_items = self.len();
            let block = CompiledBlockBuilder::new(stack);
            // Push the list capacity onto the stack
            // => [capacity]
            let block = block.push(instruction::core::Const {
                value: ConstValue::U32(num_items as u32),
            });
            // Allocate the list term
            // => [ListTerm]
            let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                target: RuntimeBuiltin::AllocateList,
            });
            // Assign the list items
            let block = self.iter().enumerate().fold(
                Result::<_, CompilerError<_>>::Ok(block),
                |results, (index, arg)| {
                    let block = results?;
                    // Duplicate the list term pointer onto the stack
                    // => [ListTerm, ListTerm]
                    let block = block.push(instruction::core::Duplicate {
                        value_type: ValueType::HeapPointer,
                    });
                    // Push the item index onto the stack
                    // => [ListTerm, ListTerm, index]
                    let block = block.push(instruction::core::Const {
                        value: ConstValue::U32(index as u32),
                    });
                    // Yield the child item onto the stack
                    // => [ListTerm, ListTerm, index, Term]
                    let block = block.append_inner(|stack| arg.compile(stack, state, options))?;
                    // Set the list term's value at the given index to the child item
                    // => [ListTerm]
                    let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                        target: RuntimeBuiltin::SetListItem,
                    });
                    Ok(block)
                },
            )?;
            // Now that all the items have been added, push the list length onto the stack
            // => [ListTerm, length]
            let block = block.push(instruction::core::Const {
                value: ConstValue::U32(num_items as u32),
            });
            // Initialize the list term with the length that is on the stack
            // => [ListTerm]
            let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                target: RuntimeBuiltin::InitList,
            });
            block.finish()
        }
    }
}

/// Attempt to determine the statically-known arity of the provided application target given the provided compiler options
/// (this will return `None` if the arity cannot be statically determined)
pub(crate) fn get_compiled_function_arity<A: Arena + Clone>(
    target: &WasmExpression<A>,
    options: &CompilerOptions,
) -> Option<Arity> {
    if let Some(term) = target.as_builtin_term() {
        term.as_inner().arity()
    } else if let Some(term) = target.as_constructor_term() {
        let num_properties = term.as_inner().keys().as_inner().len();
        Some(match options.lazy_constructors {
            ArgType::Lazy => Arity::lazy(num_properties, 0, false),
            ArgType::Eager => Arity::eager(num_properties, 0, false),
            ArgType::Strict => Arity::strict(num_properties, 0, false),
        })
    } else if let Some(term) = target.as_lambda_term() {
        let num_args = term.as_inner().num_args() as usize;
        Some(match options.lazy_lambda_args {
            ArgType::Lazy => Arity::lazy(num_args, 0, false),
            ArgType::Eager => Arity::eager(num_args, 0, false),
            ArgType::Strict => Arity::strict(num_args, 0, false),
        })
    } else if let Some(term) = target.as_partial_term() {
        get_compiled_function_arity(&term.as_inner().target(), options)
            .map(|arity| arity.partial(term.as_inner().args().as_inner().len()))
    } else {
        None
    }
}

/// Compile each of the provided initializers, pushing a new variable declaration scope onto the lexical scope stack for each one
pub(crate) fn compile_variable_declarations<A: Arena>(
    initializers: impl IntoIterator<Item = (ValueType, impl CompileWasm<A>)>,
    stack: CompilerStack,
    state: &mut CompilerState,
    options: &CompilerOptions,
) -> CompilerResult<A> {
    // Compile the closed-over variable initializer values separately within the correct lexical scopes
    let block = initializers
        .into_iter()
        .fold(
            Result::<_, CompilerError<A>>::Ok((CompiledBlockBuilder::new(stack.clone()), stack)),
            |result, (value_type, initializer)| {
                let (block, initializer_stack) = result?;
                // Compile the initializer within the correct lexical scope stack,
                // taking into account the lexical scopes created for the preceding initializers
                let (initializer_block, initializer_stack) = {
                    let block = CompiledBlockBuilder::new(initializer_stack.clone());
                    let block =
                        block.append_inner(|stack| initializer.compile(stack, state, options))?;
                    let block = block.finish::<CompilerError<A>>()?;
                    // Create a new stack that reflects a new lexical scope having been created for the initializer
                    // without affecting the variable scope stack
                    (block, initializer_stack.enter_scope(value_type))
                };
                // Yield the initializer value onto the operand stack
                // => [Term]
                let block = block.append_inner(
                    // The variable initializer has already been compiled within the correct lexical scope stack,
                    // so we ignore the expected stack
                    // (this is necessary because the initializers are compiled within intermediate lexical scopes,
                    // whereas the output block encodes the preceding initializers as fully-fledged variables,
                    // which would interfere with the scope offsets of any variable references within the initializers)
                    |_stack| CompilerResult::<A>::Ok(initializer_block),
                )?;
                // Pop the initializer value off the stack and declare a new variable with the initializer value as its value
                // => []
                let block = block.push(instruction::runtime::DeclareVariable { value_type });
                Ok((block, initializer_stack))
            },
        )
        .map(|(block, _initializer_stack)| block)?;
    block.finish()
}

pub(crate) fn intern_static_value<A: Arena + Clone>(
    value: &WasmExpression<A>,
    state: &mut CompilerState,
) -> CompilerResult<A> {
    // TODO: Avoid need to manually track serializer state heap allocator offset during compiler static interning
    state.serializer_state.next_offset = state.heap.end_offset();
    let heap_pointer = Serialize::serialize(value, &mut state.heap, &mut state.serializer_state);
    let block = CompiledBlockBuilder::new(CompilerStack::default());
    let block = block.push(instruction::core::Const {
        value: ConstValue::HeapPointer(heap_pointer),
    });
    block.finish()
}

#[derive(Debug, Clone)]
pub(crate) enum MaybeLazyExpression<A: Arena + Clone> {
    /// Expression is evaluated immediately and its dependencies will be added to the current control flow block's
    /// dependencies. Signal results will short-circuit the current control flow block.
    Strict(WasmExpression<A>),
    /// Expression is evaluated immediately and its dependencies will be added to the current control flow block's
    /// dependencies. Signal results will be caught within a new control flow block and will not short-circuit the
    /// current control flow block.
    Eager(EagerExpression<A>),
    /// Expression is not evaluated; dynamic expressions will be wrapped in a heap-allocated closure application thunk.
    Lazy(LazyExpression<A>),
}

impl<A: Arena + Clone> MaybeLazyExpression<A> {
    pub fn new(expression: WasmExpression<A>, eager: ArgType) -> Self {
        match eager {
            ArgType::Strict => Self::Strict(expression),
            ArgType::Eager => Self::Eager(EagerExpression::new(expression)),
            ArgType::Lazy => Self::Lazy(LazyExpression::new(expression)),
        }
    }
}

impl<A: Arena + Clone> CompileWasm<A> for MaybeLazyExpression<A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        match self {
            MaybeLazyExpression::Strict(inner) => inner.compile(stack, state, options),
            MaybeLazyExpression::Eager(inner) => inner.compile(stack, state, options),
            MaybeLazyExpression::Lazy(inner) => inner.compile(stack, state, options),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EagerExpression<A: Arena + Clone> {
    inner: WasmExpression<A>,
}

impl<A: Arena + Clone> EagerExpression<A> {
    pub fn new(inner: WasmExpression<A>) -> Self {
        Self { inner }
    }
}

impl<A: Arena + Clone> CompileWasm<A> for EagerExpression<A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        if self.inner.should_intern(ArgType::Eager) {
            intern_static_value(&self.inner, state)
        } else {
            // Create a wrapper block to surround the compiled term
            // (this ensures that any signals encountered when evaluating the term will not break out of the current
            // control flow block)
            let block_type = TypeSignature {
                params: ParamsSignature::Void,
                results: ParamsSignature::Single(ValueType::HeapPointer),
            };
            let inner_stack = stack.enter_block(&block_type)?;
            let block = CompiledBlockBuilder::new(stack);
            let block = block.push(instruction::core::Block {
                block_type,
                body: self.inner.compile(inner_stack, state, options)?,
            });
            block.finish::<CompilerError<_>>()
        }
    }
}

/// Expression is evaluated immediately and its dependencies will be captured in a lazy result wrapper and will not be
/// added to the current control flow block's dependencies. Signal results will be caught within a new control flow
/// block and wrapped in a lazy result wrapper and will not short-circuit the current control flow block.
#[derive(Debug, Clone)]
pub(crate) struct DeferredExpression<A: Arena + Clone> {
    inner: WasmExpression<A>,
}

impl<A: Arena + Clone> DeferredExpression<A> {
    pub fn new(inner: WasmExpression<A>) -> Self {
        Self { inner: inner }
    }
}

impl<A: Arena + Clone> CompileWasm<A> for DeferredExpression<A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        if self.inner.should_intern(ArgType::Eager) {
            intern_static_value(&self.inner, state)
        } else {
            let block = CompiledBlockBuilder::new(stack);
            // Declare a new lexical scope used to capture dependencies accumulated when evaluating the term
            // => []
            let block = block.push(instruction::runtime::DeclareDependenciesVariable);
            // Create a wrapper block to surround the compiled term
            // (this ensures that any signals encountered when evaluating the term will not break out of the current
            // control flow block)
            // => [Term]
            let block = block.append_inner(|stack| {
                CompiledBlockBuilder::new(stack.clone())
                    // Compile the instruction into the wrapper block
                    // => [Term]
                    .push({
                        let block_type = TypeSignature {
                            params: ParamsSignature::Void,
                            results: ParamsSignature::Single(ValueType::HeapPointer),
                        };
                        let body = self.inner.compile(stack.clone(), state, options)?;
                        instruction::core::Block { block_type, body }
                    })
                    .finish::<CompilerError<_>>()
            })?;
            // Pop the evaluation result off the operand stack and assign it as the variable for a temporary lexical scope
            // => []
            let block = block.push(instruction::core::ScopeStart {
                value_type: ValueType::HeapPointer,
            });
            // Push a boolean value onto the operand stack that signifies whether any dependencies were encountered during evaluation
            // => [bool]
            let block = {
                // Push the dependencies accumulated during evaluation onto the operand stack
                // => [Option<TreeTerm>]
                let block = block.push(instruction::runtime::GetDependenciesValue);
                // Push a null pointer onto the stack
                // => [Option<TreeTerm>, NULL]
                let block = block.push(instruction::runtime::NullPointer);
                // Compare the two to determine whether any dependencies were encountered during evaluation
                // => [bool]
                let block = block.push(instruction::core::Ne {
                    value_type: ValueType::HeapPointer,
                });
                block
            };
            // Return either a lazy result wrapper or the evaluation result itself, based on whether any dependencies
            // were encountered during evaluation
            // => [Term]
            let block = block.append_inner(|stack| {
                let block_type = TypeSignature {
                    params: ParamsSignature::Void,
                    results: ParamsSignature::Single(ValueType::HeapPointer),
                };
                let inner_stack = stack.enter_block(&block_type)?;
                let (consequent_stack, alternative_stack) = (inner_stack.clone(), inner_stack);
                CompiledBlockBuilder::new(stack)
                    .push(instruction::core::If {
                        block_type,
                        // If dependencies were encountered during evaluation, return a lazy result wrapper comprising
                        // the evaluation result combined with the accumulated dependencies
                        consequent: {
                            let block = CompiledBlockBuilder::new(consequent_stack);
                            // Load the evaluation result from the temporary scope and push onto the stack
                            // => [Term]
                            let block = block.push(instruction::core::GetScopeValue {
                                value_type: ValueType::HeapPointer,
                                scope_offset: 0,
                            });
                            // Push a copy of the accumulated dependencies onto the stack
                            // => [Term]
                            let block = block.push(instruction::runtime::GetDependenciesValue);
                            // Construct a new lazy result wrapper comprising the evaluation result and accumulated dependencies
                            // => [LazyResultTerm]
                            let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                                target: RuntimeBuiltin::CreateLazyResult,
                            });
                            block.finish::<CompilerError<_>>()
                        }?,
                        // Otherwise if no dependencies were encountered during evaluation, return the evaluation result
                        alternative: {
                            let block = CompiledBlockBuilder::new(alternative_stack);
                            // Load the evaluation result from the temporary scope and push onto the stack
                            // => [Term]
                            let block = block.push(instruction::core::GetScopeValue {
                                value_type: ValueType::HeapPointer,
                                scope_offset: 0,
                            });
                            block.finish::<CompilerError<_>>()
                        }?,
                    })
                    .finish::<CompilerError<_>>()
            })?;
            // End the lexical scope block used to store the evaluation result
            // => [Term]
            let block = block.push(instruction::core::ScopeEnd {
                value_type: ValueType::HeapPointer,
            });
            // End the lexical scope block used to capture dependencies
            // => [Term]
            let block = block.push(instruction::core::ScopeEnd {
                value_type: ValueType::HeapPointer,
            });
            block.finish::<CompilerError<_>>()
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LazyExpression<A: Arena + Clone> {
    inner: WasmExpression<A>,
}

impl<A: Arena + Clone> LazyExpression<A> {
    pub fn new(inner: WasmExpression<A>) -> Self {
        Self { inner }
    }
}

impl<A: Arena + Clone> CompileWasm<A> for LazyExpression<A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        if self.inner.should_intern(ArgType::Lazy) {
            intern_static_value(&self.inner, state)
        } else {
            // Compile the thunk
            let thunk_id = self.inner.read_value(|term| term.header.hash);
            let compiled_thunk = match state.compiled_thunks.get(&thunk_id) {
                // If the provided thunk has already been compiled, return the existing copy
                Some(compiled_thunk) => compiled_thunk,
                // Otherwise allocate a new thunk application term in the heap snapshot
                None => {
                    // Retrieve a list of free variables present within the thunk
                    let free_variables = match self.inner.free_variables() {
                        free_variables if free_variables.is_empty() => None,
                        free_variables => Some(free_variables),
                    };
                    // Create a new compiler stack to be used within the thunk body,
                    // with all the free variables captured on the compiler stack as global variable mappings
                    // and a block wrapper to catch short-circuiting signals
                    let inner_stack = match free_variables.as_ref() {
                        Some(free_variables) => CompilerStack::from_free_variables(
                            free_variables
                                .iter()
                                .copied()
                                .map(|scope_offset| (scope_offset, ValueType::HeapPointer)),
                        ),
                        None => CompilerStack::default(),
                    }
                    .enter_block(&TypeSignature {
                        params: ParamsSignature::Void,
                        results: ParamsSignature::Single(ValueType::HeapPointer),
                    })
                    .map_err(CompilerError::StackError)?;
                    let thunk_function_body = self.inner.compile(inner_stack, state, options)?;
                    // Create a placeholder builtin term to represent the compiled function
                    let compiled_function_term = state.heap.allocate(Term::new(
                        TermType::Builtin(BuiltinTerm {
                            // The compiled function ID will be filled in with the actual value by the linker
                            uid: u32::from(ArenaPointer::null()),
                        }),
                        &state.heap,
                    ));
                    let thunk = match free_variables {
                        None => {
                            let empty_list = {
                                let empty_list_term = Term::new(
                                    TermType::List(ListTerm {
                                        items: Array {
                                            capacity: 0,
                                            length: 0,
                                            items: [],
                                        },
                                    }),
                                    &state.heap,
                                );
                                match state
                                    .serializer_state
                                    .allocated_terms
                                    .entry(empty_list_term.id())
                                {
                                    Entry::Occupied(entry) => *(entry.get()),
                                    Entry::Vacant(entry) => {
                                        let heap_pointer = state.heap.allocate(empty_list_term);
                                        entry.insert(heap_pointer);
                                        state.serializer_state.next_offset =
                                            state.heap.end_offset();
                                        heap_pointer
                                    }
                                }
                            };
                            let application_pointer = state.heap.allocate(Term::new(
                                TermType::Application(ApplicationTerm {
                                    target: compiled_function_term,
                                    args: empty_list,
                                }),
                                &state.heap,
                            ));
                            CompiledThunk::Pure(PureThunk {
                                application_term: application_pointer,
                                thunk_function_body,
                                compiled_function_term,
                            })
                        }
                        Some(bindings) => CompiledThunk::Capturing(CapturingThunk {
                            free_variables: bindings.into_iter().collect(),
                            thunk_function_body,
                            compiled_function_term,
                        }),
                    };
                    state.compiled_thunks.entry(thunk_id).or_insert(thunk)
                }
            };
            match compiled_thunk {
                CompiledThunk::Pure(PureThunk {
                    application_term, ..
                }) => {
                    // Pure thunks can be evaluated against an empty stack
                    let block = CompiledBlockBuilder::new(CompilerStack::default());
                    // Push a pointer to the precompiled application term onto the stack
                    // => [ApplicationTerm]
                    let block = block.push(instruction::core::Const {
                        value: ConstValue::HeapPointer(*application_term),
                    });
                    block.finish()
                }
                CompiledThunk::Capturing(CapturingThunk {
                    free_variables,
                    compiled_function_term,
                    ..
                }) => {
                    // Capturing thunks capture their variables from the current stack
                    let block = CompiledBlockBuilder::new(stack);
                    // Push a pointer to the precompiled thunk target function onto the stack
                    // => [BuiltinTerm]
                    let block = block.push(instruction::core::Const {
                        value: ConstValue::HeapPointer(*compiled_function_term),
                    });
                    // Collect a list of captured variable lexical scope offsets, from deepest to shallowest
                    let captured_variable_scope_offsets = {
                        let mut captured_variables = free_variables.clone();
                        captured_variables.sort();
                        captured_variables.reverse();
                        captured_variables
                    };
                    // Yield a list term containing the captured variable values onto the stack
                    // => [BuiltinTerm, ListTerm]
                    let block = block.append_inner(|stack| {
                        collect_compiled_list_values(
                            captured_variable_scope_offsets
                                .into_iter()
                                .map(|scope_offset| {
                                    (ClosureCapture { scope_offset }, Strictness::NonStrict)
                                }),
                            stack,
                            state,
                            options,
                        )
                    })?;
                    // Create an application term that applies the thunk function to the list of captured variables
                    // => [ApplicationTerm]
                    let block = block.push(instruction::runtime::CallRuntimeBuiltin {
                        target: RuntimeBuiltin::CreateApplication,
                    });
                    block.finish()
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum CompiledThunk {
    Pure(PureThunk),
    Capturing(CapturingThunk),
}

#[derive(Clone, Debug)]
pub struct PureThunk {
    /// Bytecode instructions for the compiled thunk
    pub thunk_function_body: CompiledBlock,
    /// Pointer to the heap-allocated application term instance
    /// (pure thunks contain no free variables and therefore the same application term instance can be shared across all usages)
    pub application_term: ArenaPointer,
    /// Pointer to the heap-allocated term instance corresponding to the compiled thunk function.
    ///
    /// Note that the compiled function index cannot be known until the code generation phase, so the allocated term
    /// contains a placeholder value that will need to be patched with the correct value once known
    pub compiled_function_term: ArenaPointer,
}

#[derive(Clone, Debug)]
pub struct CapturingThunk {
    /// List of variable scope offsets of any free variables referenced within the thunk
    pub free_variables: Vec<StackOffset>,
    /// Bytecode instructions for the compiled thunk
    pub thunk_function_body: CompiledBlock,
    /// Pointer to the heap-allocated term instance corresponding to the compiled thunk function
    /// (this can be used as a function application target, passing the free variable values as arguments)
    ///
    /// Note that the compiled function index cannot be known until the code generation phase, so the allocated term
    /// contains a placeholder value that will need to be patched with the correct value once known
    pub compiled_function_term: ArenaPointer,
}

#[derive(Debug, Clone, Copy)]
struct ClosureCapture {
    scope_offset: StackOffset,
}

impl<A: Arena + Clone> CompileWasm<A> for ClosureCapture {
    fn compile(
        &self,
        stack: CompilerStack,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let Self { scope_offset } = self;
        if let Some(stack_offset) = stack.bindings.lookup_variable(*scope_offset) {
            let block = CompiledBlockBuilder::new(stack);
            // Copy the lexically-scoped variable onto the stack
            // => [Term]
            let block = block.push(instruction::core::GetScopeValue {
                value_type: ValueType::HeapPointer,
                scope_offset: stack_offset,
            });
            block.finish()
        } else {
            Err(CompilerError::UnboundVariable(*scope_offset))
        }
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<Term, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        if self.should_intern(ArgType::Strict) {
            let compiled_heap_pointer = intern_static_value(self, state)?;
            let result = if self.is_static() {
                Ok(compiled_heap_pointer)
            } else {
                let block = CompiledBlockBuilder::new(CompilerStack::default());
                let block = block.append_block(compiled_heap_pointer);
                let block = block.push(CompiledInstruction::Evaluate(
                    instruction::runtime::Evaluate,
                ));
                block.finish()
            };
            return result;
        }
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Boolean => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Cell => self
                .as_typed_term::<CellTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Effect => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Float => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Hashmap => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Hashset => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Int => self
                .as_typed_term::<IntTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Lambda => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::LazyResult => self
                .as_typed_term::<LazyResultTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Let => self
                .as_typed_term::<LetTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::List => self
                .as_typed_term::<ListTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Nil => self
                .as_typed_term::<NilTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Partial => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Pointer => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Record => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Signal => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::String => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Symbol => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Timestamp => self
                .as_typed_term::<TimestampTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Tree => self
                .as_typed_term::<TreeTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::Variable => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::IndexedAccessorIterator => self
                .as_typed_term::<IndexedAccessorIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .compile(stack, state, options),
        }
    }
}
