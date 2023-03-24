// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    collections::{hash_map::Entry, HashMap, LinkedList},
    iter::{once, repeat},
};

use reflex::{
    core::{Arity, Eagerness, GraphNode, Internable, NodeId, StackOffset},
    hash::IntMap,
};

use crate::{
    allocator::{Arena, ArenaAllocator, ArenaIterator, VecAllocator},
    compiler::builtin::RuntimeBuiltin,
    hash::{TermHashState, TermHasher, TermSize},
    serialize::{Serialize, SerializerState},
    stdlib::Stdlib,
    term_type::*,
    term_type::{list::compile_list, TermType, TermTypeDiscriminants, TypedTerm, WasmExpression},
    ArenaPointer, ArenaRef, Array, FunctionIndex, IntoArenaRefIterator, PointerIter, Term,
};

pub mod builtin;
pub mod globals;
pub mod wasm;

#[derive(Clone)]
pub enum CompilerError<A: Arena> {
    InvalidFunctionTarget(FunctionIndex),
    InvalidFunctionArgs {
        target: ArenaRef<Term, A>,
        arity: Arity,
        args: Vec<ArenaRef<Term, A>>,
    },
    UnboundVariable(StackOffset),
}

impl<A: Arena + Clone> std::fmt::Display for CompilerError<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFunctionTarget(target) => {
                write!(f, "Invalid function index: {target}",)
            }
            Self::InvalidFunctionArgs {
                target,
                arity,
                args,
            } => write!(
                f,
                "Invalid function invocation for {target}: expected {} arguments, received ({})",
                arity.required().len(),
                args.iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::UnboundVariable(scope_offset) => {
                write!(f, "Unbound variable scope offset {scope_offset}")
            }
        }
    }
}

pub type CompilerResult<A> = Result<CompiledBlock, CompilerError<A>>;

/// Virtual stack machine instruction set
///
/// This is a proprietary instruction set for a stack machine interpreter, with the intended goal of providing some
/// useful high-level operations while still being straightforward to translate to WebAssembly instructions.
///
/// The instruction set assumes two separate stacks, which correspond to the WebAssembly stack and WebAssembly function locals respectively:
///   1. The operand stack, which stores function arguments and return values
///   2. The lexical scope stack, which is used store local variables (similar to  virtual registers).
///     Each lexical scope stack frame holds a single variable, which can be loaded onto the operand stack from anywhere within
///     the lexical scope. Variables stored in the lexical scopes cannot be mutated (although mutation can be simulated
///     by using a SSA-style approach).
#[derive(Debug, Clone, PartialEq)]
pub enum CompiledInstruction {
    // 'Low-level' instructions (generic stack machine instructions)
    /// Push a constant value onto the operand stack
    Const(ConstValue),
    /// Pop a target heap pointer address from the operand stack, read the value at that offset within the heap memory and push the value onto the operand stack
    ReadHeapValue(ValueType),
    /// Pop a value from the operand stack, then pop the target heap pointer address from the operand stack, then write the value to that offset in the heap memory
    WriteHeapValue(ValueType),
    /// Duplicate the value at the top of the operand stack
    Duplicate(ValueType),
    /// Pop the term at the top of the operand stack and discard it
    Drop(ValueType),
    /// Pop the top item of the operand stack and enter a new lexical scope whose variable is assigned to that value
    ScopeStart(ValueType),
    /// Pop the latest lexical scope
    ScopeEnd(ValueType),
    /// Push a variable defined in a containing lexical scope onto the operand stack (where `0` is the current lexical scope, `1` is the immediate parent of the current lexical scope, etc)
    GetScopeValue {
        value_type: ValueType,
        scope_offset: StackOffset,
    },
    /// Pop the top three values from the operand stack, and if the top item is not `0`, push the bottom value back onto the stack, otherwise push the middle value onto the stack
    Select(ValueType),
    // Pop the top item of the operand stack, and if the value is not `0` then enter the 'consequent' block, otherwise enter the 'alternative' block
    If {
        // TODO: Determine block type signature by analyzing inner blocks
        block_type: TypeSignature,
        consequent: CompiledBlock,
        alternative: CompiledBlock,
    },
    /// Pop the top item from the operand stack, and if the value is not `0` then enter the `handler` block, otherwise continue with the current block
    ConditionalBreak {
        /// Description of operand stack items to carry over into the continuation block, along with the result type
        /// of the inner block
        /// (note that the inner block result type must be identical to the overall result type of the enclosing block)
        // TODO: Determine evaluation continuation block stack depth automatically by analyzing preceding instructions
        block_type: TypeSignature,
        /// Handler block to invoke if the condition was triggered
        ///
        /// This will have access to any captured stack values, and must return the same result type as the main block
        handler: CompiledBlock,
    },
    /// Pop the top two values from the operand stack, and push a constant `1` if they are equal, or `0` if not
    Eq(ValueType),
    /// Push a null pointer onto the operand stack
    NullPointer,

    // 'High-level' instructions (these make assumptions about the runtime environment - i.e. state, dependencies etc)
    /// Pop a term pointer from the operand stack, look that key up in the global state object,
    /// and push either the corresponding value term reference or a null pointer depending on whether the key exists
    LoadStateValue,
    /// Invoke an interpreter builtin function, popping the required number of arguments from the operand stack
    /// (arguments are passed to the function in the same order they were added to the operand stack, i.e. not reversed)
    CallRuntimeBuiltin(RuntimeBuiltin),
    /// Invoke a standard library function known at compile-time, popping the required number of arguments from the operand stack
    /// (arguments are passed to the function in the same order they were added to the operand stack, i.e. not reversed)
    /// If the function has variadic arguments, the final argument is assumed to be a heap pointer to a list term containing the variadic arguments
    CallStdlib(Stdlib),
    /// Invoke a user-defined function known at compile-time, popping the required number of arguments from the operand stack
    /// (arguments are passed to the function in the same order they were added to the operand stack, i.e. not reversed)
    CallCompiledFunction {
        /// Type signature of the target function
        signature: TypeSignature,
        /// ID of the target function
        target: CompiledFunctionId,
    },
    /// Pop the argument list term pointer from the operand stack, then pop the function target index from the operand stack,
    /// then invoke the corresponding function, pushing the result onto the operand stack
    CallDynamic(TypeSignature),
    /// Pop the top item of the operand stack, evaluate it, and push the result onto the operand stack.
    Evaluate,
    /// Pop the argument list term pointer from the operand stack, then pop the target term pointer from the operand stack,
    /// then apply the corresponding target term to the arguments list, pushing the result onto the operand stack
    Apply,
}

impl TypedCompilerBlock for CompiledInstruction {
    fn get_type(
        &self,
        _operand_stack: &[ValueType],
        lexical_scopes: &[ValueType],
    ) -> TypedStackResult<TypedStackType> {
        || -> TypedStackResult<TypedStackType> {
            match self {
                CompiledInstruction::Const(value) => {
                    Ok(TypeSignature::new((), value.get_type()).into())
                }
                CompiledInstruction::ReadHeapValue(value_type) => {
                    Ok(TypeSignature::new(ValueType::HeapPointer, *value_type).into())
                }
                CompiledInstruction::WriteHeapValue(value_type) => {
                    Ok(TypeSignature::new((ValueType::HeapPointer, *value_type), ()).into())
                }
                CompiledInstruction::Duplicate(value_type) => {
                    Ok(TypeSignature::new(*value_type, (*value_type, *value_type)).into())
                }
                CompiledInstruction::Drop(value_type) => {
                    Ok(TypeSignature::new(*value_type, ()).into())
                }
                CompiledInstruction::ScopeStart(value_type) => Ok(TypedStackType {
                    operand_stack: TypeSignature::new(*value_type, ()),
                    lexical_scopes: Some(LexicalScopeDelta::Start {
                        value_types: vec![*value_type],
                    }),
                }),
                CompiledInstruction::ScopeEnd(value_type) => Ok(TypedStackType {
                    operand_stack: TypeSignature::new((), ()),
                    lexical_scopes: Some(LexicalScopeDelta::End {
                        value_types: vec![*value_type],
                    }),
                }),
                CompiledInstruction::GetScopeValue {
                    value_type,
                    scope_offset: _,
                } => Ok(TypeSignature::new((), *value_type).into()),
                CompiledInstruction::Select(value_type) => Ok(TypeSignature::new(
                    (*value_type, *value_type, ValueType::U32),
                    *value_type,
                )
                .into()),
                CompiledInstruction::If {
                    block_type,
                    consequent,
                    alternative,
                } => {
                    let inner_params = block_type.params.iter().collect::<Vec<_>>();
                    let consequent_type = consequent.get_type(&inner_params, lexical_scopes)?;
                    let alternative_type = alternative.get_type(&inner_params, lexical_scopes)?;
                    if &consequent_type.operand_stack != block_type {
                        Err(
                            TypedStackErrorReason::InvalidBlockType(InvalidBlockTypeError {
                                expected: block_type.clone().into(),
                                received: consequent_type,
                            })
                            .into(),
                        )
                    } else if &alternative_type.operand_stack != block_type {
                        Err(
                            TypedStackErrorReason::InvalidBlockType(InvalidBlockTypeError {
                                expected: block_type.clone().into(),
                                received: alternative_type,
                            })
                            .into(),
                        )
                    } else {
                        Ok(TypeSignature::new(
                            ParamsSignature::from_iter(
                                block_type.params.iter().chain([ValueType::U32]),
                            ),
                            block_type.results.clone(),
                        )
                        .into())
                    }
                }
                CompiledInstruction::ConditionalBreak {
                    block_type,
                    handler,
                } => {
                    let inner_params = block_type.params.iter().collect::<Vec<_>>();
                    let handler_type = handler.get_type(&inner_params, lexical_scopes)?;
                    // TODO: Assert that the overall enclosing block type is identical to conditional break handler block type
                    if &handler_type.operand_stack != block_type {
                        Err(
                            TypedStackErrorReason::InvalidBlockType(InvalidBlockTypeError {
                                expected: block_type.clone().into(),
                                received: handler_type,
                            })
                            .into(),
                        )
                    } else {
                        Ok(TypeSignature::new(
                            {
                                let condition_type = ValueType::U32;
                                let params = ParamsSignature::from_iter(
                                    block_type.params.iter().chain(once(condition_type)),
                                );
                                params
                            },
                            block_type.params.clone(),
                        )
                        .into())
                    }
                }
                CompiledInstruction::Eq(value_type) => {
                    Ok(TypeSignature::new((*value_type, *value_type), ValueType::U32).into())
                }
                CompiledInstruction::NullPointer => {
                    Ok(TypeSignature::new((), ValueType::HeapPointer).into())
                }
                CompiledInstruction::LoadStateValue => {
                    Ok(TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer).into())
                }
                CompiledInstruction::CallRuntimeBuiltin(builtin) => Ok(builtin.signature().into()),
                CompiledInstruction::CallStdlib(stdlib) => {
                    let arity = stdlib.arity();
                    let num_positional_args = arity.required().len() + arity.optional().len();
                    // Variadic arguments are passed as a heap pointer to an argument list
                    let num_variadic_args = arity.variadic().map(|_| 1).unwrap_or(0);
                    let num_args = num_positional_args + num_variadic_args;
                    let arg_types = (0..num_args).map(|_| ValueType::HeapPointer);
                    Ok(TypeSignature::new(
                        ParamsSignature::from_iter(arg_types),
                        ValueType::HeapPointer,
                    )
                    .into())
                }
                CompiledInstruction::CallCompiledFunction {
                    signature,
                    target: _,
                } => Ok(signature.clone().into()),
                CompiledInstruction::CallDynamic(_) => Ok(TypeSignature::new(
                    (ValueType::U32, ValueType::HeapPointer),
                    ValueType::HeapPointer,
                )
                .into()),
                CompiledInstruction::Evaluate => {
                    Ok(TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer).into())
                }
                CompiledInstruction::Apply => Ok(TypeSignature::new(
                    (ValueType::HeapPointer, ValueType::HeapPointer),
                    ValueType::HeapPointer,
                )
                .into()),
            }
        }()
        .map_err(|err| match &err.instruction {
            Some(_) => err,
            None => TypedStackError {
                instruction: Some(self.clone()),
                error: err.error,
            },
        })
    }
}

impl std::fmt::Display for CompiledInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Const(..) => write!(f, "Const"),
            Self::ReadHeapValue(..) => write!(f, "ReadHeapValue"),
            Self::WriteHeapValue(..) => write!(f, "WriteHeapValue"),
            Self::Duplicate(..) => write!(f, "Duplicate"),
            Self::Drop(..) => write!(f, "Drop"),
            Self::ScopeStart(..) => write!(f, "ScopeStart"),
            Self::ScopeEnd(..) => write!(f, "ScopeEnd"),
            Self::GetScopeValue { .. } => write!(f, "GetScopeValue"),
            Self::Select(..) => write!(f, "Select"),
            Self::If { .. } => write!(f, "If"),
            Self::ConditionalBreak { .. } => write!(f, "ConditionalBreak"),
            Self::Eq(..) => write!(f, "Eq"),
            Self::NullPointer => write!(f, "NullPointer"),
            Self::LoadStateValue => write!(f, "LoadStateValue"),
            Self::CallRuntimeBuiltin(..) => write!(f, "CallRuntimeBuiltin"),
            Self::CallStdlib(..) => write!(f, "CallStdlib"),
            Self::CallCompiledFunction { .. } => write!(f, "CallCompiledFunction"),
            Self::CallDynamic(..) => write!(f, "CallDynamic"),
            Self::Evaluate => write!(f, "Evaluate"),
            Self::Apply => write!(f, "Apply"),
        }
    }
}

pub trait TypedCompilerBlock {
    fn get_type(
        &self,
        operand_stack: &[ValueType],
        lexical_scopes: &[ValueType],
    ) -> TypedStackResult<TypedStackType>;
}

pub type TypedStackResult<T> = Result<T, TypedStackError>;

#[derive(Debug, PartialEq, Clone)]
pub struct TypedStackError {
    instruction: Option<CompiledInstruction>,
    error: TypedStackErrorReason,
}

impl std::error::Error for TypedStackError {}

impl std::fmt::Display for TypedStackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.instruction {
            Some(instruction) => write!(f, "Invalid {} instruction: {}", instruction, self.error),
            None => write!(f, "{}", self.error),
        }
    }
}

impl From<TypedStackErrorReason> for TypedStackError {
    fn from(value: TypedStackErrorReason) -> Self {
        Self {
            instruction: None,
            error: value,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypedStackErrorReason {
    InsufficientOperandStackDepth(InsufficientOperandStackErrorDepth),
    InsufficientLexicalScopeDepth(InsufficientLexicalScopeDepthError),
    InvalidStackValueTypes(InvalidStackValueTypesError),
    InvalidBlockType(InvalidBlockTypeError),
}

impl std::fmt::Display for TypedStackErrorReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientOperandStackDepth(inner) => std::fmt::Display::fmt(inner, f),
            Self::InsufficientLexicalScopeDepth(inner) => std::fmt::Display::fmt(inner, f),
            Self::InvalidStackValueTypes(inner) => std::fmt::Display::fmt(inner, f),
            Self::InvalidBlockType(inner) => std::fmt::Display::fmt(inner, f),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InsufficientOperandStackErrorDepth {
    expected: ParamsSignature,
    received: ParamsSignature,
}

impl std::fmt::Display for InsufficientOperandStackErrorDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Insufficient items on stack: Expected {}, received {}",
            self.expected, self.received
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct InsufficientLexicalScopeDepthError {
    expected: StackOffset,
    received: StackOffset,
}

impl std::fmt::Display for InsufficientLexicalScopeDepthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Insufficient lexical scope depth: Expected {}, received {}",
            self.expected, self.received
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InvalidStackValueTypesError {
    expected: ParamsSignature,
    received: ParamsSignature,
}

impl std::fmt::Display for InvalidStackValueTypesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid values on stack: Expected {}, received {}",
            self.expected, self.received
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InvalidBlockTypeError {
    expected: TypedStackType,
    received: TypedStackType,
}

impl std::fmt::Display for InvalidBlockTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid block type: Expected {}, received {}",
            self.expected, self.received
        )
    }
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
        ParamsSignatureIter {
            signature: self,
            index: 0,
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
    index: usize,
}

impl<'a> Iterator for ParamsSignatureIter<'a> {
    type Item = ValueType;
    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.signature {
            ParamsSignature::Void => None,
            ParamsSignature::Single(inner) => match self.index {
                0 => Some(*inner),
                _ => None,
            },
            ParamsSignature::Pair(first, second) => match self.index {
                0 => Some(*first),
                1 => Some(*second),
                _ => None,
            },
            ParamsSignature::Triple(first, second, third) => match self.index {
                0 => Some(*first),
                1 => Some(*second),
                2 => Some(*third),
                _ => None,
            },
            ParamsSignature::Multiple(values) => values.get(self.index).copied(),
        };
        if item.is_some() {
            self.index += 1;
        }
        item
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.signature.len() - self.index;
        (length, Some(length))
    }
}

impl<'a> ExactSizeIterator for ParamsSignatureIter<'a> {}

impl CompiledInstruction {
    pub fn i32_const(value: i32) -> Self {
        Self::Const(ConstValue::I32(value))
    }
    pub fn u32_const(value: u32) -> Self {
        Self::Const(ConstValue::U32(value))
    }
    pub fn i64_const(value: i64) -> Self {
        Self::Const(ConstValue::I64(value))
    }
    pub fn f32_const(value: f32) -> Self {
        Self::Const(ConstValue::F32(value))
    }
    pub fn f64_const(value: f64) -> Self {
        Self::Const(ConstValue::F64(value))
    }
    pub fn heap_pointer(value: ArenaPointer) -> Self {
        Self::Const(ConstValue::HeapPointer(value))
    }
    pub fn function_pointer(value: impl Into<FunctionPointer>) -> Self {
        Self::Const(ConstValue::FunctionPointer(value.into()))
    }
}

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

#[derive(Debug, Copy, Clone, Default)]
pub struct CompilerOptions {
    pub lazy_record_values: bool,
    pub lazy_list_items: bool,
    pub lazy_variable_initializers: bool,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CompiledBlock {
    instructions: LinkedList<CompiledInstruction>,
}

impl TypedCompilerBlock for CompiledBlock {
    fn get_type(
        &self,
        operand_stack: &[ValueType],
        lexical_scopes: &[ValueType],
    ) -> TypedStackResult<TypedStackType> {
        validate_block_type(
            self.instructions.iter(),
            operand_stack.iter().copied().collect(),
            lexical_scopes.iter().copied().collect(),
        )
    }
}

fn validate_block_type<'a>(
    instructions: impl IntoIterator<Item = &'a CompiledInstruction>,
    operand_stack: Vec<ValueType>,
    lexical_scopes: Vec<ValueType>,
) -> TypedStackResult<TypedStackType> {
    let params = ParamsSignature::from_iter(operand_stack.iter().copied());
    // Perform type checking for the block by simulating a virtual stack
    let (operand_stack, lexical_scopes) = instructions.into_iter().fold(
        Ok((operand_stack, lexical_scopes)),
        |result, instruction| {
            let (mut operand_stack, mut lexical_scopes) = result?;
            let TypedStackType {
                operand_stack: stack_updates,
                lexical_scopes: lexical_scope_updates,
            } = instruction.get_type(&operand_stack, &lexical_scopes)?;
            let num_existing_operands = operand_stack.len();
            let num_popped_operands = stack_updates.params.len();
            // Assert that enough operands are present on the simulated operand stack
            if num_popped_operands > num_existing_operands {
                return Err(TypedStackError {
                    instruction: Some(instruction.clone()),
                    error: TypedStackErrorReason::InsufficientOperandStackDepth(
                        InsufficientOperandStackErrorDepth {
                            expected: stack_updates.params,
                            received: ParamsSignature::from_iter(operand_stack),
                        },
                    ),
                });
            }
            // Pop the instruction parameter types from the simulated operand stack
            let popped_items = operand_stack
                .drain((operand_stack.len() - num_popped_operands)..)
                .collect::<Vec<_>>();
            // Assert that all operands on the simulated operand stack are the expected types
            for (expected, received) in stack_updates
                .params
                .iter()
                .zip(popped_items.iter().copied())
            {
                if expected != received {
                    return Err(TypedStackError {
                        instruction: Some(instruction.clone()),
                        error: TypedStackErrorReason::InvalidStackValueTypes(
                            InvalidStackValueTypesError {
                                expected: stack_updates.params,
                                received: ParamsSignature::from_iter(popped_items),
                            },
                        ),
                    });
                }
            }
            // Push the instruction result types onto the simulated operand stack
            operand_stack.extend(stack_updates.results);
            // Enter/leave any lexical scopes as appropriate
            if let Some(scope_updates) = lexical_scope_updates {
                match scope_updates {
                    LexicalScopeDelta::Start { value_types } => {
                        lexical_scopes.extend(value_types);
                    }
                    LexicalScopeDelta::End { value_types } => {
                        let num_existing_scopes = lexical_scopes.len();
                        let num_popped_scopes = value_types.len();
                        // Assert that enough scopes are present on the simulated lexical scope stack
                        if num_popped_scopes > num_existing_scopes {
                            return Err(TypedStackError {
                                instruction: Some(instruction.clone()),
                                error: TypedStackErrorReason::InsufficientLexicalScopeDepth(
                                    InsufficientLexicalScopeDepthError {
                                        expected: num_popped_scopes,
                                        received: num_existing_scopes,
                                    },
                                ),
                            });
                        }
                        // Pop the scoped variable types from the simulated lexical scope stack
                        let popped_items = lexical_scopes
                            .drain((lexical_scopes.len() - num_popped_scopes)..)
                            .collect::<Vec<_>>();
                        // Assert that all variables on the simulated lexical scope stack are the expected types
                        for (expected, received) in value_types
                            .iter()
                            .copied()
                            .zip(popped_items.iter().copied())
                        {
                            if expected != received {
                                return Err(TypedStackError {
                                    instruction: Some(instruction.clone()),
                                    error: TypedStackErrorReason::InvalidStackValueTypes(
                                        InvalidStackValueTypesError {
                                            expected: ParamsSignature::from_iter(value_types),
                                            received: ParamsSignature::from_iter(popped_items),
                                        },
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            Ok((operand_stack, lexical_scopes))
        },
    )?;
    Ok(TypedStackType {
        operand_stack: TypeSignature::new(params, ParamsSignature::from_iter(operand_stack)),
        lexical_scopes: if lexical_scopes.is_empty() {
            None
        } else {
            Some(LexicalScopeDelta::Start {
                value_types: lexical_scopes,
            })
        },
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedStackType {
    pub operand_stack: TypeSignature,
    pub lexical_scopes: Option<LexicalScopeDelta>,
}

impl From<TypeSignature> for TypedStackType {
    fn from(value: TypeSignature) -> Self {
        Self {
            operand_stack: value,
            lexical_scopes: None,
        }
    }
}

impl std::fmt::Display for TypedStackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = write!(f, "{}", self.operand_stack)?;
        if let Some(scope_updates) = self.lexical_scopes.as_ref() {
            write!(f, " {} {}", self.operand_stack, scope_updates)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexicalScopeDelta {
    Start { value_types: Vec<ValueType> },
    End { value_types: Vec<ValueType> },
}

impl std::fmt::Display for LexicalScopeDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Start { value_types } => write!(
                f,
                "+[{}]",
                value_types
                    .iter()
                    .map(|item| format!("{}", item))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Self::End { value_types } => write!(
                f,
                "-[{}]",
                value_types
                    .iter()
                    .map(|item| format!("{}", item))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledFunction {
    pub num_args: usize,
    pub body: CompiledBlock,
}

impl CompiledBlock {
    pub fn push(&mut self, item: CompiledInstruction) {
        self.instructions.push_back(item)
    }
    pub fn append_block(&mut self, block: CompiledBlock) {
        self.instructions.extend(block)
    }
    pub fn iter(&self) -> std::collections::linked_list::Iter<'_, CompiledInstruction> {
        self.instructions.iter()
    }
}

impl FromIterator<CompiledInstruction> for CompiledBlock {
    fn from_iter<I: IntoIterator<Item = CompiledInstruction>>(iter: I) -> Self {
        CompiledBlock {
            instructions: iter.into_iter().collect::<LinkedList<_>>(),
        }
    }
}

impl Extend<CompiledInstruction> for CompiledBlock {
    fn extend<T: IntoIterator<Item = CompiledInstruction>>(&mut self, iter: T) {
        self.instructions.extend(iter)
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

#[derive(Clone, Debug)]
pub enum CompiledThunk {
    Pure {
        instructions: CompiledBlock,
        application_term: ArenaPointer,
        // Placeholder pointer to where the compiled lambda ID should be written once known
        target_uid_pointer: ArenaPointer,
    },
    Capturing {
        free_variables: Vec<StackOffset>,
        instructions: CompiledBlock,
        target_term: ArenaPointer,
        // Placeholder pointer to where the compiled lambda ID should be written once known
        target_uid_pointer: ArenaPointer,
    },
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
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A>;
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompilerStack {
    values: Vec<CompilerStackValue>,
}

impl CompilerStack {
    #[must_use]
    pub fn push_strict(&self) -> Self {
        self.append(once(CompilerStackValue::Strict))
    }
    #[must_use]
    pub fn push_lazy(&self, value_type: ValueType) -> Self {
        self.append(once(CompilerStackValue::Lazy(value_type)))
    }
    #[must_use]
    pub fn pop(&self) -> Self {
        Self::from_iter(self.values.iter().copied().take(self.values.len() - 1))
    }
    #[must_use]
    pub fn append(&self, values: impl IntoIterator<Item = CompilerStackValue>) -> Self {
        Self::from_iter(self.values.iter().copied().chain(values))
    }
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = CompilerStackValue> + ExactSizeIterator + DoubleEndedIterator + '_
    {
        self.values.iter().copied()
    }
    pub fn rev(
        &self,
    ) -> impl Iterator<Item = CompilerStackValue> + ExactSizeIterator + DoubleEndedIterator + '_
    {
        self.iter().rev()
    }
    pub fn value_types(
        &self,
    ) -> impl Iterator<Item = ValueType> + ExactSizeIterator + DoubleEndedIterator + '_ {
        self.iter().map(|value| value.get_type())
    }
    pub fn depth(&self) -> usize {
        self.values.len()
    }
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl FromIterator<CompilerStackValue> for CompilerStack {
    fn from_iter<T: IntoIterator<Item = CompilerStackValue>>(iter: T) -> Self {
        Self {
            values: iter.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompilerStackValue {
    /// Strict stack values point to terms in the the heap, and will be checked for signals before returning from the active block
    Strict,
    /// Lazy stack values will be disposed when exiting the active block without checking for signals
    Lazy(ValueType),
}

impl CompilerStackValue {
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Strict => ValueType::HeapPointer,
            Self::Lazy(value_type) => *value_type,
        }
    }
    pub fn is_strict(&self) -> bool {
        matches!(self, Self::Strict)
    }
}

#[derive(Default, Clone, Debug)]
pub struct CompilerVariableBindings<'a> {
    /// Mapping of 'application' variable scopes to actual 'compiler' stack offsets
    free_variables: Option<&'a IntMap<StackOffset, StackOffset>>,
    /// Stack of 'compiler' lexical scopes, where each stack frame is one of the predetermined frame types
    local_scopes: Vec<CompilerVariableStackFrame>,
}

#[derive(Clone, Copy, Debug)]
pub enum CompilerVariableStackFrame {
    /// Application-level variable declaration
    Local,
    /// Application-level variable alias
    Alias(StackOffset),
    /// Internal compiler stack frame
    Internal,
}

impl<'a> CompilerVariableBindings<'a> {
    pub fn from_mappings(mappings: &'a IntMap<StackOffset, StackOffset>) -> Self {
        Self {
            free_variables: Some(mappings),
            local_scopes: Default::default(),
        }
    }
    pub fn get(&self, scope_offset: StackOffset) -> Option<StackOffset> {
        let Self {
            free_variables,
            local_scopes,
        } = self;
        enum ScopeResult {
            /// Iteration has terminated successfully with the given target compiler stack offset
            Ok(StackOffset),
            /// Iteration has not yet terminated successfully
            Pending {
                /// Application variable scope offset of desired target variable
                target_offset: StackOffset,
                /// Target compiler stack offset accumulated so far during the iteration
                num_scopes: StackOffset,
            },
        }
        // Iterate backwards through the local scopes to find a potential match
        let result = local_scopes.iter().rev().fold(
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
                    // If we have reached the target offset, inspect the current local scope binding and respond accordingly
                    if target_offset == 0 {
                        match local_binding {
                            // If this is a local variable declararation, we have reached our desired scope offset
                            CompilerVariableStackFrame::Local => ScopeResult::Ok(num_scopes),
                            // If this is a local variable alias, continue iterating to locate the alias target
                            CompilerVariableStackFrame::Alias(alias_offset) => {
                                ScopeResult::Pending {
                                    target_offset: *alias_offset,
                                    // Aliases are 'invisible' to the compiler stack, so no need to increment the number of accumulated scopes
                                    num_scopes,
                                }
                            }
                            // Otherwise if this is a 'gap' created by an intermediate compiler scope, continue iterating
                            CompilerVariableStackFrame::Internal => ScopeResult::Pending {
                                target_offset,
                                num_scopes: num_scopes + 1,
                            },
                        }
                    } else {
                        match local_binding {
                            // If this is a local variable declaration, decrement the target scope offset accordingly and continue iterating
                            CompilerVariableStackFrame::Local => ScopeResult::Pending {
                                target_offset: target_offset - 1,
                                num_scopes: num_scopes + 1,
                            },
                            // If this is a local variable alias, decrement the target scope offset accordingly and continue iterating
                            CompilerVariableStackFrame::Alias(_) => ScopeResult::Pending {
                                target_offset: target_offset - 1,
                                // Aliases are 'invisible' to the compiler stack, so no need to increment the number of accumulated scopes
                                num_scopes,
                            },
                            // Otherwise if this is a 'gap' created by an intermediate compiler scope, continue iterating
                            CompilerVariableStackFrame::Internal => ScopeResult::Pending {
                                target_offset,
                                num_scopes: num_scopes + 1,
                            },
                        }
                    }
                }
            },
        );
        match result {
            // If the result yielded the compiler stack offset of a local variable, return the result
            ScopeResult::Ok(stack_offset) => Some(stack_offset),
            // Otherwise if all local variables have been exhausted, look up the variable in the global scope
            ScopeResult::Pending {
                target_offset,
                num_scopes,
            } => free_variables.and_then(|free_variables| {
                free_variables
                    .get(&target_offset)
                    .copied()
                    .map(|free_variable_offset| free_variable_offset + num_scopes)
            }),
        }
    }
    pub fn offset(&self, depth: StackOffset) -> Self {
        self.append(repeat(CompilerVariableStackFrame::Internal).take(depth))
    }
    pub fn push_local(&self) -> Self {
        self.push(CompilerVariableStackFrame::Local)
    }
    pub fn push_alias(&self, scope_offset: StackOffset) -> Self {
        self.push(CompilerVariableStackFrame::Alias(scope_offset))
    }
    fn push(&self, binding: CompilerVariableStackFrame) -> Self {
        self.append(once(binding))
    }
    fn append(&self, scopes: impl IntoIterator<Item = CompilerVariableStackFrame>) -> Self {
        let Self {
            free_variables,
            local_scopes,
        } = self;
        Self {
            free_variables: *free_variables,
            local_scopes: local_scopes.iter().copied().chain(scopes).collect(),
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
    pub args: ArenaRef<ListTerm, A>,
    pub partial_args: Vec<WasmExpression<A>>,
}

impl<A: Arena + Clone> CompiledFunctionCallArgs<A> {
    pub fn iter(&self) -> CompiledFunctionCallArgsIter<A> {
        CompiledFunctionCallArgsIter {
            arg_list: self,
            index: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.partial_args.len() + self.args.len()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledFunctionCallArgsIter<'a, A: Arena + Clone> {
    arg_list: &'a CompiledFunctionCallArgs<A>,
    index: usize,
}

impl<'a, A: Arena + Clone> Iterator for CompiledFunctionCallArgsIter<'a, A> {
    type Item = WasmExpression<A>;
    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.arg_list.partial_args.get(self.index) {
            Some(arg) => Some(arg.clone()),
            None => self
                .arg_list
                .args
                .get(self.index.saturating_sub(self.arg_list.partial_args.len())),
        };
        if item.is_some() {
            self.index += 1;
        }
        item
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.arg_list.len() - self.index;
        (length, Some(length))
    }
}

impl<'a, A: Arena + Clone> ExactSizeIterator for CompiledFunctionCallArgsIter<'a, A> {}

impl<A: Arena + Clone> CompileWasm<A> for CompiledFunctionCallArgs<A> {
    fn compile(
        &self,
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        if self.partial_args.is_empty() {
            // If there are no partially-applied arguments, delegate to the existing argument list compilation
            // (this can make use of static term inlining)
            self.args.compile(state, bindings, options, stack)
        } else {
            // Otherwise if there are partially-applied arguments, compile the combined argument sequence into a list
            // TODO: Investigate chained iterators for partially-applied arguments
            let num_items = self.len();
            let mut instructions = CompiledBlock::default();
            // Push the list capacity onto the stack
            // => [capacity]
            instructions.push(CompiledInstruction::u32_const(num_items as u32));
            // Allocate the list term
            // => [ListTerm]
            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::AllocateList,
            ));
            // Assign the list items
            for (index, arg) in self.iter().enumerate() {
                // Duplicate the list term pointer onto the stack
                // => [ListTerm, ListTerm]
                instructions.push(CompiledInstruction::Duplicate(ValueType::HeapPointer));
                // Push the item index onto the stack
                // => [ListTerm, ListTerm, index]
                instructions.push(CompiledInstruction::u32_const(index as u32));
                // Yield the child item onto the stack
                // => [ListTerm, ListTerm, index, Term]
                let child_stack = stack.append([
                    CompilerStackValue::Lazy(ValueType::HeapPointer),
                    CompilerStackValue::Lazy(ValueType::HeapPointer),
                    CompilerStackValue::Lazy(ValueType::U32),
                ]);
                let child_block = arg.compile(state, bindings, options, &child_stack)?;
                instructions.append_block(child_block);
                // Set the list term's value at the given index to the child item
                // => [ListTerm]
                instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                    RuntimeBuiltin::SetListItem,
                ));
            }
            // Now that all the items have been added, push the list length onto the stack
            // => [ListTerm, length]
            instructions.push(CompiledInstruction::u32_const(num_items as u32));
            // Initialize the list term with the length that is on the stack
            // => [ListTerm]
            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::InitList,
            ));
            Ok(instructions)
        }
    }
}

pub(crate) fn intern_static_value<A: Arena + Clone>(
    value: &WasmExpression<A>,
    state: &mut CompilerState,
) -> CompilerResult<A> {
    // TODO: Avoid need to manually track serializer state heap allocator offset during compiler static interning
    state.serializer_state.next_offset = state.heap.end_offset();
    let heap_pointer = Serialize::serialize(value, &mut state.heap, &mut state.serializer_state);
    return Ok(CompiledBlock::from_iter([CompiledInstruction::Const(
        ConstValue::HeapPointer(heap_pointer),
    )]));
}

#[derive(Debug, Clone)]
pub(crate) enum MaybeLazyExpression<A: Arena + Clone> {
    Eager(WasmExpression<A>),
    Lazy(LazyExpression<A>),
}

impl<A: Arena + Clone> MaybeLazyExpression<A> {
    pub fn new(expression: WasmExpression<A>, eagerness: Eagerness) -> Self {
        match eagerness {
            Eagerness::Eager => Self::Eager(expression),
            Eagerness::Lazy => Self::Lazy(LazyExpression::new(expression)),
        }
    }
}

impl<A: Arena + Clone> CompileWasm<A> for MaybeLazyExpression<A> {
    fn compile(
        &self,
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        match self {
            MaybeLazyExpression::Eager(inner) => inner.compile(state, bindings, options, stack),
            MaybeLazyExpression::Lazy(inner) => inner.compile(state, bindings, options, stack),
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
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        if self.inner.is_static() {
            return self.inner.compile(state, bindings, options, stack);
        }
        if self.inner.should_intern(Eagerness::Lazy) {
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
                    let free_variable_mappings = free_variables
                        .as_ref()
                        .map(|free_variables| {
                            // Construct a mapping of application-level scope offsets to captured compiler stack offsets
                            let mut scope_offsets =
                                free_variables.iter().copied().collect::<Vec<_>>();
                            scope_offsets.sort();
                            scope_offsets
                                .into_iter()
                                .enumerate()
                                .map(|(captured_offset, scope_offset)| {
                                    (scope_offset, captured_offset)
                                })
                                .rev()
                                .collect::<IntMap<_, _>>()
                        })
                        .unwrap_or_default();
                    let inner_bindings =
                        CompilerVariableBindings::from_mappings(&free_variable_mappings);
                    let thunk_function_body = self.inner.compile(
                        state,
                        &inner_bindings,
                        options,
                        &CompilerStack::default(),
                    )?;
                    // Create a placeholder term to represent the compiled function
                    let (target_term, target_uid_pointer) = {
                        let term_pointer = state.heap.allocate(Term::new(
                            TermType::Builtin(BuiltinTerm {
                                // The compiled function ID will be filled in with the actual value by the linker
                                uid: u32::from(ArenaPointer::null()),
                            }),
                            &state.heap,
                        ));
                        let uid_pointer = {
                            let term = ArenaRef::<TypedTerm<BuiltinTerm>, _>::new(
                                &state.heap,
                                term_pointer,
                            );
                            term.as_inner().inner_pointer(|term| &term.uid)
                        };
                        (term_pointer, uid_pointer)
                    };
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
                                    target: target_term,
                                    args: empty_list,
                                    cache: Default::default(),
                                }),
                                &state.heap,
                            ));
                            CompiledThunk::Pure {
                                application_term: application_pointer,
                                instructions: thunk_function_body,
                                target_uid_pointer,
                            }
                        }
                        Some(bindings) => CompiledThunk::Capturing {
                            free_variables: bindings.into_iter().collect(),
                            instructions: thunk_function_body,
                            target_term,
                            target_uid_pointer,
                        },
                    };
                    state.compiled_thunks.entry(thunk_id).or_insert(thunk)
                }
            };
            let mut instructions = CompiledBlock::default();
            match compiled_thunk {
                CompiledThunk::Pure {
                    application_term, ..
                } => {
                    instructions.push(CompiledInstruction::Const(ConstValue::HeapPointer(
                        *application_term,
                    )));
                }
                CompiledThunk::Capturing {
                    free_variables,
                    target_term,
                    ..
                } => {
                    let captured_variables = {
                        let mut captured_variables = free_variables
                            .iter()
                            .copied()
                            .map(|stack_offset| {
                                (
                                    ClosureCapture {
                                        scope_offset: stack_offset,
                                    },
                                    Eagerness::Lazy,
                                )
                            })
                            .collect::<Vec<_>>();
                        captured_variables.sort_by_key(
                            |(
                                ClosureCapture {
                                    scope_offset: stack_offset,
                                },
                                _,
                            )| *stack_offset,
                        );
                        captured_variables.reverse();
                        captured_variables
                    };
                    instructions.push(CompiledInstruction::Const(ConstValue::HeapPointer(
                        *target_term,
                    )));
                    let inner_stack = stack.push_lazy(ValueType::HeapPointer);
                    instructions.append_block(compile_list(
                        captured_variables,
                        state,
                        bindings,
                        options,
                        &inner_stack,
                    )?);
                    instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                        RuntimeBuiltin::CreateApplication,
                    ));
                }
            }
            Ok(instructions)
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ClosureCapture {
    scope_offset: StackOffset,
}
impl<A: Arena + Clone> CompileWasm<A> for ClosureCapture {
    fn compile(
        &self,
        _state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        _options: &CompilerOptions,
        _stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let Self { scope_offset } = self;
        if let Some(stack_offset) = bindings.get(*scope_offset) {
            let mut instructions = CompiledBlock::default();
            // Copy the lexically-scoped variable onto the stack
            // => [Term]
            instructions.push(CompiledInstruction::GetScopeValue {
                value_type: ValueType::HeapPointer,
                scope_offset: stack_offset,
            });
            Ok(instructions)
        } else {
            Err(CompilerError::UnboundVariable(*scope_offset))
        }
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<Term, A> {
    fn compile(
        &self,
        state: &mut CompilerState,
        bindings: &CompilerVariableBindings,
        options: &CompilerOptions,
        stack: &CompilerStack,
    ) -> CompilerResult<A> {
        if self.should_intern(Eagerness::Eager) {
            return intern_static_value(self, state);
        }
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Boolean => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Cell => self
                .as_typed_term::<CellTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Date => self
                .as_typed_term::<DateTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Effect => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Float => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Hashmap => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Hashset => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Int => self
                .as_typed_term::<IntTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Lambda => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Let => self
                .as_typed_term::<LetTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::List => self
                .as_typed_term::<ListTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Nil => self
                .as_typed_term::<NilTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Partial => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Pointer => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Record => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Signal => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::String => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Symbol => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Tree => self
                .as_typed_term::<TreeTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::Variable => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::IndexedAccessorIterator => self
                .as_typed_term::<IndexedAccessorIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .compile(state, bindings, options, stack),
        }
    }
}

pub mod tests {
    use std::{
        cell::RefCell,
        ops::{Deref, DerefMut},
        rc::Rc,
    };

    use debug_ignore::DebugIgnore;
    use derivative::Derivative;
    use reflex::{
        core::{Expression, ExpressionFactory, NodeId},
        hash::{HashId, IntMap},
    };

    use crate::{
        allocator::{Arena, ArenaAllocator, VecAllocator},
        cli::compile::{
            compile_module, parse_inline_memory_snapshot, WasmCompilerError, WasmCompilerMode,
            WasmCompilerOptions, WasmProgram,
        },
        compiler::{
            CompileWasm, CompilerOptions, CompilerStack, CompilerState, CompilerVariableBindings,
            TypedCompilerBlock, TypedStackError, TypedStackType,
        },
        factory::WasmTermFactory,
        interpreter::{InterpreterError, WasmInterpreter},
        stdlib::Stdlib,
        term_type::{
            hashmap::HashmapTerm, lambda::LambdaTerm, tree::TreeTerm, TermType, TypedTerm,
            WasmExpression,
        },
        ArenaPointer, ArenaRef, Term,
    };

    const RUNTIME_BYTES: &[u8] = include_bytes!("../../build/runtime.wasm");

    #[derive(Derivative)]
    #[derivative(Default(bound = ""), Debug(bound = ""), Clone(bound = ""))]
    pub struct WasmDependencyList<A: Arena + Clone> {
        dependencies: IntMap<HashId, WasmExpression<A>>,
    }

    impl<A: Arena + Clone> WasmDependencyList<A> {
        fn iter(&self) -> std::collections::hash_map::Values<'_, HashId, WasmExpression<A>> {
            self.dependencies.values()
        }
    }

    impl<A: Arena + Clone> IntoIterator for WasmDependencyList<A> {
        type Item = WasmExpression<A>;
        type IntoIter = std::collections::hash_map::IntoValues<HashId, WasmExpression<A>>;
        fn into_iter(self) -> Self::IntoIter {
            self.dependencies.into_values()
        }
    }

    impl<A: Arena + Clone> FromIterator<WasmExpression<A>> for WasmDependencyList<A> {
        fn from_iter<T: IntoIterator<Item = WasmExpression<A>>>(iter: T) -> Self {
            Self {
                dependencies: iter.into_iter().map(|item| (item.id(), item)).collect(),
            }
        }
    }

    impl<A: Arena + Clone> PartialEq for WasmDependencyList<A> {
        fn eq(&self, other: &Self) -> bool {
            self.dependencies.len() == other.dependencies.len()
                && self.dependencies.iter().all(|(key, value)| {
                    other
                        .dependencies
                        .get(key)
                        .map(|other_value| value == other_value)
                        .unwrap_or(false)
                })
        }
    }

    impl<A: Arena + Clone> Eq for WasmDependencyList<A> {}

    #[derive(Derivative)]
    #[derivative(
        Debug(bound = ""),
        Clone(bound = ""),
        PartialEq(bound = ""),
        Eq(bound = "")
    )]
    pub struct WasmEvaluationResult<A: Arena + Clone> {
        pub result: WasmExpression<A>,
        pub dependencies: WasmDependencyList<A>,
    }

    impl<A: Arena + Clone> WasmEvaluationResult<A> {
        pub fn into_parts(self) -> (WasmExpression<A>, WasmDependencyList<A>) {
            let Self {
                result,
                dependencies,
            } = self;
            (result, dependencies)
        }
    }

    impl<A: Arena + Clone> std::fmt::Display for WasmEvaluationResult<A> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{} ({})",
                self.result,
                self.dependencies
                    .iter()
                    .map(|term| format!("{}", term))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    #[derive(Debug)]
    pub enum CompilerTestError<T: Expression> {
        Allocator(T),
        Compiler(WasmCompilerError),
        Interpreter(InterpreterError, DebugIgnore<WasmProgram>),
    }

    impl<T: Expression> std::error::Error for CompilerTestError<T> {}

    impl<T: Expression> std::fmt::Display for CompilerTestError<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Allocator(term) => write!(f, "Failed to allocate expression: {term}"),
                Self::Compiler(err) => write!(f, "Failed to compile expression: {err}"),
                Self::Interpreter(err, _) => write!(f, "Failed to interpret expression: {err}"),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    pub enum BytecodeValidationError<T: Expression> {
        Deserialize(T),
        Compiler(String),
        StackValidation(TypedStackError),
    }

    impl<T: Expression + std::fmt::Display> std::fmt::Display for BytecodeValidationError<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Deserialize(term) => write!(f, "Failed to deserialize expression: {term}"),
                Self::Compiler(err) => write!(f, "Compiler error: {err}"),
                Self::StackValidation(err) => write!(f, "Stack error: {err}"),
            }
        }
    }

    pub fn validate_bytecode<T: Expression>(
        expression: &T,
        factory: &impl ExpressionFactory<T>,
        compiler_options: &CompilerOptions,
    ) -> Result<TypedStackType, BytecodeValidationError<T>>
    where
        T::Builtin: Into<crate::stdlib::Stdlib>,
    {
        let mut allocator = VecAllocator::default();
        let arena = Rc::new(RefCell::new(&mut allocator));
        let wasm_factory = WasmTermFactory::from(Rc::clone(&arena));
        let expression = wasm_factory
            .import(expression, factory)
            .map_err(BytecodeValidationError::Deserialize)?;
        let mut compiler_state =
            CompilerState::from_heap_snapshot::<Term>(arena.borrow().as_bytes());
        let compiled_expression = expression
            .compile(
                &mut compiler_state,
                &CompilerVariableBindings::default(),
                compiler_options,
                &CompilerStack::default(),
            )
            .map_err(|err| format!("{}", err))
            .map_err(BytecodeValidationError::Compiler)?;
        let block_type = compiled_expression
            .get_type(&Vec::new(), &Vec::new())
            .map_err(BytecodeValidationError::StackValidation)?;
        Ok(block_type)
    }

    pub fn evaluate_compiled<T: Expression>(
        expression: T,
        state: impl IntoIterator<Item = (T::Signal, T)>,
        factory: &impl ExpressionFactory<T>,
        compiler_options: &WasmCompilerOptions,
    ) -> Result<WasmEvaluationResult<Rc<RefCell<VecAllocator>>>, CompilerTestError<T>>
    where
        T::Builtin: Into<Stdlib>,
    {
        let export_name = "__root__";
        let initial_heap_snapshot = parse_inline_memory_snapshot(RUNTIME_BYTES).unwrap();
        let mut allocator = VecAllocator::from_bytes(&initial_heap_snapshot);
        let mut arena = Rc::new(RefCell::new(&mut allocator));
        let wasm_factory = WasmTermFactory::from(Rc::clone(&arena));
        let wasm_expression = wasm_factory
            .import(&expression, factory)
            .map_err(CompilerTestError::Allocator)?;
        let entry_point = {
            let term = Term::new(
                TermType::Lambda(LambdaTerm {
                    num_args: 0,
                    body: wasm_expression.as_pointer(),
                }),
                &arena,
            );
            let pointer = arena
                .deref()
                .borrow_mut()
                .deref_mut()
                .deref_mut()
                .allocate(term);
            ArenaRef::<TypedTerm<LambdaTerm>, _>::new(Rc::clone(&arena), pointer)
        };
        let state_entries = state
            .into_iter()
            .map(|(key, value)| {
                let key = wasm_factory.import_condition(&key, factory)?;
                let value = wasm_factory.import(&value, factory)?;
                Ok((key.as_pointer(), value.as_pointer()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(CompilerTestError::Allocator)?;
        let state = if state_entries.is_empty() {
            ArenaPointer::null()
        } else {
            HashmapTerm::allocate(state_entries, &mut arena)
        };
        let linear_memory = Vec::from(arena.deref().borrow().deref().deref().as_bytes());
        let wasm_module = compile_module(
            [(String::from(export_name), entry_point)],
            &RUNTIME_BYTES,
            WasmCompilerMode::Wasm,
            Some(&linear_memory),
            compiler_options,
            true,
        )
        .map_err(CompilerTestError::Compiler)?;

        let (interpreter, result, dependencies) =
            WasmInterpreter::instantiate(&wasm_module, "memory")
                .and_then(|mut interpreter| {
                    interpreter
                        .call::<u32, (u32, u32)>(export_name, u32::from(state))
                        .map(|(result, dependencies)| (interpreter, result, dependencies))
                })
                .map_err(|err| CompilerTestError::Interpreter(err, wasm_module.into()))?;
        let allocator = VecAllocator::from_bytes(&interpreter.dump_heap());
        let arena = Rc::new(RefCell::new(allocator));
        let dependencies = ArenaPointer::from(dependencies)
            .as_non_null()
            .map(|pointer| ArenaRef::<TypedTerm<TreeTerm>, _>::new(Rc::clone(&arena), pointer))
            .map(|dependency_tree| {
                dependency_tree
                    .as_inner()
                    .nodes()
                    .collect::<WasmDependencyList<_>>()
            })
            .unwrap_or_default();
        let result = ArenaRef::<Term, _>::new(arena, ArenaPointer::from(result));
        Ok(WasmEvaluationResult {
            result,
            dependencies,
        })
    }
}
