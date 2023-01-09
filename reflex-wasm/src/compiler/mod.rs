// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::{
    linked_list::{IntoIter, Iter},
    LinkedList,
};

use reflex::core::{Eagerness, Internable, NodeId};
use strum_macros::EnumIter;
use walrus::ir::{Const, Instr, Value};

use crate::{
    allocator::{Arena, ArenaIterator, VecAllocator},
    hash::TermSize,
    serialize::{Serialize, SerializerState},
    stdlib::Stdlib,
    term_type::*,
    ArenaRef, IntoArenaRefIterator, PointerIter, Term,
};

pub mod term;

#[derive(Clone, Debug)]
pub enum CompilerError {}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compilation failed")
    }
}

impl std::error::Error for CompilerError {}

type CompilerResult = Result<CompiledExpression, CompilerError>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, EnumIter)]
pub enum RuntimeBuiltin {
    Initialize,
    Evaluate,
    CreateApplication,
    CreateBoolean,
    CreateBuiltin,
    CreateFloat,
    CreateInt,
    AllocateList,
    InitList,
    SetListItem,
}

impl RuntimeBuiltin {
    pub fn name(self) -> &'static str {
        match self {
            RuntimeBuiltin::Initialize => "_initialize",
            RuntimeBuiltin::Evaluate => "evaluate",
            RuntimeBuiltin::CreateApplication => "createApplication",
            RuntimeBuiltin::CreateBoolean => "createBoolean",
            RuntimeBuiltin::CreateBuiltin => "createBuiltin",
            RuntimeBuiltin::CreateFloat => "createFloat",
            RuntimeBuiltin::CreateInt => "createInt",
            RuntimeBuiltin::AllocateList => "allocateList",
            RuntimeBuiltin::InitList => "initList",
            RuntimeBuiltin::SetListItem => "setListItem",
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

pub trait CompileWasm {
    fn compile(
        &self,
        eager: Eagerness,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult;
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<Term, A> {
    fn compile(
        &self,
        eager: Eagerness,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult {
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
                .compile(eager, state, options),
            TermTypeDiscriminants::Boolean => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Cell => self
                .as_typed_term::<CellTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Compiled => self
                .as_typed_term::<CompiledTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Date => self
                .as_typed_term::<DateTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Effect => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Float => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Hashmap => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Hashset => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Int => self
                .as_typed_term::<IntTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Lambda => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Let => self
                .as_typed_term::<LetTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::List => self
                .as_typed_term::<ListTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Nil => self
                .as_typed_term::<NilTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Partial => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Pointer => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Record => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Signal => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::String => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Symbol => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Tree => self
                .as_typed_term::<TreeTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Variable => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .compile(eager, state, options),
        }
    }
}
