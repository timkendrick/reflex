// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::{
    linked_list::{IntoIter, Iter},
    LinkedList,
};

use reflex::core::{Arity, Eagerness, Internable, NodeId};
use strum_macros::EnumIter;
use walrus::ir::{Const, Instr, Value};

use crate::{
    allocator::{Arena, ArenaIterator, VecAllocator},
    hash::TermSize,
    serialize::{Serialize, SerializerState},
    stdlib::Stdlib,
    term_type::*,
    utils::from_twos_complement_i32,
    ArenaRef, IntoArenaRefIterator, PointerIter, Term,
};

pub mod term_type;

#[derive(Clone)]
pub enum CompilerError<A: Arena> {
    InvalidFunctionArgs {
        target: ArenaRef<Term, A>,
        arity: Arity,
        args: Vec<ArenaRef<Term, A>>,
    },
}

impl<A: Arena + Clone> std::fmt::Display for CompilerError<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::InvalidFunctionArgs {
                target,
                arity,
                args,
            } => write!(
                f,
                "Invalid function application for {target}: expected {} arguments, received ({})",
                arity.required().len(),
                args.iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

type CompilerResult<A> = Result<CompiledExpression, CompilerError<A>>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, EnumIter)]
pub enum RuntimeGlobal {
    NullPointer,
}

impl RuntimeGlobal {
    pub fn name(self) -> &'static str {
        match self {
            RuntimeGlobal::NullPointer => "NULL",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, EnumIter)]
pub enum RuntimeBuiltin {
    Initialize,
    Evaluate,
    AllocateCell,
    AllocateHashmap,
    AllocateList,
    AllocateString,
    CreateApplication,
    CreateBoolean,
    CreateBuiltin,
    CreateCustomCondition,
    CreatePendingCondition,
    CreateErrorCondition,
    CreateTypeErrorCondition,
    CreateInvalidFunctionTargetCondition,
    CreateInvalidFunctionArgsCondition,
    CreateInvalidPointerCondition,
    CreateConstructor,
    CreateDate,
    CreateEffect,
    CreateFloat,
    CreateHashset,
    CreateInt,
    CreateLambda,
    CreateLet,
    CreateNil,
    CreatePartial,
    CreatePointer,
    CreateRecord,
    CreateSignal,
    CreateTree,
    CreateVariable,
    CreateEmptyIterator,
    CreateEvaluateIterator,
    CreateFilterIterator,
    CreateFlattenIterator,
    CreateHashmapKeysIterator,
    CreateHashmapValuesIterator,
    CreateIntegersIterator,
    CreateIntersperseIterator,
    CreateMapIterator,
    CreateOnceIterator,
    CreateRangeIterator,
    CreateRepeatIterator,
    CreateSkipIterator,
    CreateTakeIterator,
    CreateZipIterator,
    GetStringCharOffset,
    InitHashmap,
    InitList,
    InitString,
    InsertHashmapEntry,
    SetCellField,
    SetListItem,
    Write,
}

impl RuntimeBuiltin {
    pub fn name(self) -> &'static str {
        match self {
            RuntimeBuiltin::Initialize => "_initialize",
            RuntimeBuiltin::Evaluate => "evaluate",
            RuntimeBuiltin::AllocateCell => "allocateCell",
            RuntimeBuiltin::AllocateHashmap => "allocateHashmap",
            RuntimeBuiltin::AllocateList => "allocateList",
            RuntimeBuiltin::AllocateString => "allocateString",
            RuntimeBuiltin::CreateApplication => "createApplication",
            RuntimeBuiltin::CreateBoolean => "createBoolean",
            RuntimeBuiltin::CreateBuiltin => "createBuiltin",
            RuntimeBuiltin::CreateCustomCondition => "createCustomCondition",
            RuntimeBuiltin::CreatePendingCondition => "createPendingCondition",
            RuntimeBuiltin::CreateErrorCondition => "createErrorCondition",
            RuntimeBuiltin::CreateTypeErrorCondition => "createTypeErrorCondition",
            RuntimeBuiltin::CreateInvalidFunctionTargetCondition => {
                "createInvalidFunctionTargetCondition"
            }
            RuntimeBuiltin::CreateInvalidFunctionArgsCondition => {
                "createInvalidFunctionArgsCondition"
            }
            RuntimeBuiltin::CreateInvalidPointerCondition => "createInvalidPointerCondition",
            RuntimeBuiltin::CreateConstructor => "createConstructor",
            RuntimeBuiltin::CreateDate => "createDate",
            RuntimeBuiltin::CreateEffect => "createEffect",
            RuntimeBuiltin::CreateFloat => "createFloat",
            RuntimeBuiltin::CreateHashset => "createHashset",
            RuntimeBuiltin::CreateInt => "createInt",
            RuntimeBuiltin::CreateLambda => "createLambda",
            RuntimeBuiltin::CreateLet => "createLet",
            RuntimeBuiltin::CreateNil => "createNil",
            RuntimeBuiltin::CreatePartial => "createPartial",
            RuntimeBuiltin::CreatePointer => "createPointer",
            RuntimeBuiltin::CreateRecord => "createRecord",
            RuntimeBuiltin::CreateSignal => "createSignal",
            RuntimeBuiltin::CreateTree => "createTree",
            RuntimeBuiltin::CreateVariable => "createVariable",
            RuntimeBuiltin::CreateEmptyIterator => "createEmptyIterator",
            RuntimeBuiltin::CreateEvaluateIterator => "createEvaluateIterator",
            RuntimeBuiltin::CreateFilterIterator => "createFilterIterator",
            RuntimeBuiltin::CreateFlattenIterator => "createFlattenIterator",
            RuntimeBuiltin::CreateHashmapKeysIterator => "createHashmapKeysIterator",
            RuntimeBuiltin::CreateHashmapValuesIterator => "createHashmapValuesIterator",
            RuntimeBuiltin::CreateIntegersIterator => "createIntegersIterator",
            RuntimeBuiltin::CreateIntersperseIterator => "createIntersperseIterator",
            RuntimeBuiltin::CreateMapIterator => "createMapIterator",
            RuntimeBuiltin::CreateOnceIterator => "createOnceIterator",
            RuntimeBuiltin::CreateRangeIterator => "createRangeIterator",
            RuntimeBuiltin::CreateRepeatIterator => "createRepeatIterator",
            RuntimeBuiltin::CreateSkipIterator => "createSkipIterator",
            RuntimeBuiltin::CreateTakeIterator => "createTakeIterator",
            RuntimeBuiltin::CreateZipIterator => "createZipIterator",
            RuntimeBuiltin::GetStringCharOffset => "getStringCharOffset",
            RuntimeBuiltin::InitHashmap => "initHashmap",
            RuntimeBuiltin::InitList => "initList",
            RuntimeBuiltin::InitString => "initString",
            RuntimeBuiltin::InsertHashmapEntry => "insertHashmapEntry",
            RuntimeBuiltin::SetCellField => "setCellField",
            RuntimeBuiltin::SetListItem => "setListItem",
            RuntimeBuiltin::Write => "write",
        }
    }
}

#[derive(Debug, Clone)]
pub enum CompiledInstruction {
    // Arbitrary WASM bytecode instruction
    Wasm(walrus::ir::Instr),
    // Call one of the interpreter builtin functions
    CallRuntimeBuiltin(RuntimeBuiltin),
    // Call a userland standard library function
    CallStdlib(Stdlib),
    /// Duplicate the top item of the stack
    Duplicate,
    /// Push a null pointer onto the stack
    Null,
}
impl CompiledInstruction {
    pub fn i32_const(value: i32) -> Self {
        Self::const_value(walrus::ir::Const {
            value: walrus::ir::Value::I32(value),
        })
    }
    pub fn u32_const(value: u32) -> Self {
        Self::i32_const(from_twos_complement_i32(value))
    }
    pub fn i64_const(value: i64) -> Self {
        Self::const_value(walrus::ir::Const {
            value: walrus::ir::Value::I64(value),
        })
    }
    pub fn f32_const(value: f32) -> Self {
        Self::const_value(walrus::ir::Const {
            value: walrus::ir::Value::F32(value),
        })
    }
    pub fn f64_const(value: f64) -> Self {
        Self::const_value(walrus::ir::Const {
            value: walrus::ir::Value::F64(value),
        })
    }
    fn const_value(value: walrus::ir::Const) -> Self {
        Self::Wasm(walrus::ir::Instr::Const(value))
    }
}

impl From<walrus::ir::Instr> for CompiledInstruction {
    fn from(value: walrus::ir::Instr) -> Self {
        CompiledInstruction::Wasm(value)
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct CompilerOptions {}

#[derive(Default, Debug)]
pub struct CompiledExpression {
    instructions: LinkedList<CompiledInstruction>,
}

impl FromIterator<CompiledInstruction> for CompiledExpression {
    fn from_iter<I: IntoIterator<Item = CompiledInstruction>>(iter: I) -> Self {
        CompiledExpression {
            instructions: iter.into_iter().collect::<LinkedList<_>>(),
        }
    }
}

impl CompiledExpression {
    pub fn push(&mut self, item: CompiledInstruction) {
        self.instructions.push_back(item)
    }
}

impl CompiledExpression {
    pub fn iter(&self) -> Iter<'_, CompiledInstruction> {
        self.instructions.iter()
    }
}

impl Extend<CompiledInstruction> for CompiledExpression {
    fn extend<T: IntoIterator<Item = CompiledInstruction>>(&mut self, iter: T) {
        self.instructions.extend(iter)
    }
}

impl IntoIterator for CompiledExpression {
    type Item = CompiledInstruction;

    type IntoIter = IntoIter<CompiledInstruction>;

    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}

impl<'a> IntoIterator for &'a CompiledExpression {
    type Item = &'a CompiledInstruction;

    type IntoIter = Iter<'a, CompiledInstruction>;

    fn into_iter(self) -> Self::IntoIter {
        self.instructions.iter()
    }
}

pub struct CompilerState {
    serializer_state: SerializerState,
    heap_arena: VecAllocator,
}

impl CompilerState {
    pub fn from_heap_snapshot<T: TermSize>(bytes: &[u8]) -> Self
    where
        for<'a> ArenaRef<T, &'a VecAllocator>: NodeId,
    {
        let heap_arena = VecAllocator::from_bytes(bytes);
        Self {
            serializer_state: {
                let arena = &heap_arena;
                let start_offset = arena.start_offset();
                let end_offset = arena.end_offset();
                let next_offset = end_offset;
                let allocated_terms = ArenaIterator::<T, _>::new(arena, start_offset, end_offset)
                    .as_arena_refs::<T>(&arena)
                    .map(|term| (term.id(), term.pointer));
                SerializerState::new(allocated_terms, next_offset)
            },
            heap_arena,
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
            heap_arena: destination_arena,
        }
    }
    pub fn into_linear_memory(self) -> Vec<u8> {
        let Self {
            heap_arena: arena, ..
        } = self;

        // Convert the underlying arena contents from Vec<u32> into Vec<u8>
        arena
            .into_inner()
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>()
    }
}

pub trait CompileWasm<A: Arena> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A>;
}

#[derive(Default)]
pub struct CompilerScope;

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<Term, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        if self.should_intern(eager) {
            let ptr =
                Serialize::serialize(self, &mut state.heap_arena, &mut state.serializer_state);
            return Ok(CompiledExpression::from_iter([CompiledInstruction::Wasm(
                Instr::Const(Const {
                    value: Value::I32(u32::from(ptr) as i32),
                }),
            )]));
        }
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Boolean => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Cell => self
                .as_typed_term::<CellTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Date => self
                .as_typed_term::<DateTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Effect => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Float => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Hashmap => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Hashset => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Int => self
                .as_typed_term::<IntTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Lambda => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Let => self
                .as_typed_term::<LetTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::List => self
                .as_typed_term::<ListTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Nil => self
                .as_typed_term::<NilTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Partial => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Pointer => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Record => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Signal => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::String => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Symbol => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Tree => self
                .as_typed_term::<TreeTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::Variable => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .compile(eager, scope, state, options),
        }
    }
}
