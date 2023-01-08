// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
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
    ArenaPointer, ArenaRef, IntoArenaRefIterator, PointerIter, Term,
};

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
    instrs: LinkedList<CompiledInstruction>,
}

impl FromIterator<CompiledInstruction> for CompiledExpression {
    fn from_iter<I: IntoIterator<Item = CompiledInstruction>>(iter: I) -> Self {
        let mut list = LinkedList::new();
        list.extend(iter);
        CompiledExpression { instrs: list }
    }
}

impl CompiledExpression {
    pub fn push_back(&mut self, item: CompiledInstruction) {
        self.instrs.push_back(item)
    }
}

impl CompiledExpression {
    pub fn iter(&self) -> Iter<'_, CompiledInstruction> {
        self.instrs.iter()
    }
}

impl Extend<CompiledInstruction> for CompiledExpression {
    fn extend<T: IntoIterator<Item = CompiledInstruction>>(&mut self, iter: T) {
        self.instrs.extend(iter)
    }
}

impl IntoIterator for CompiledExpression {
    type Item = CompiledInstruction;

    type IntoIter = IntoIter<CompiledInstruction>;

    fn into_iter(self) -> Self::IntoIter {
        self.instrs.into_iter()
    }
}

impl<'a> IntoIterator for &'a CompiledExpression {
    type Item = &'a CompiledInstruction;

    type IntoIter = Iter<'a, CompiledInstruction>;

    fn into_iter(self) -> Self::IntoIter {
        self.instrs.iter()
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
            TermTypeDiscriminants::Boolean => {
                todo!();
                // self.as_typed_term::<BooleanTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Cell => {
                todo!();
                // self.as_typed_term::<CellTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Compiled => {
                todo!();
                // self.as_typed_term::<CompiledTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Condition => {
                todo!();
                // self.as_typed_term::<ConditionTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Constructor => {
                todo!();
                // self.as_typed_term::<ConstructorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Date => {
                todo!();
                // self.as_typed_term::<DateTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Effect => {
                todo!();
                // self.as_typed_term::<EffectTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Float => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Hashmap => {
                todo!();
                // self.as_typed_term::<HashmapTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Hashset => {
                todo!();
                // self.as_typed_term::<HashsetTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Int => self
                .as_typed_term::<IntTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Lambda => {
                todo!();
                // self.as_typed_term::<LambdaTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Let => {
                todo!();
                // self.as_typed_term::<LetTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::List => self
                .as_typed_term::<ListTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Nil => {
                todo!();
                // self.as_typed_term::<NilTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Partial => {
                todo!();
                // self.as_typed_term::<PartialTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Pointer => {
                todo!();
                // self.as_typed_term::<PointerTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Record => {
                todo!();
                // self.as_typed_term::<RecordTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Signal => {
                todo!();
                // self.as_typed_term::<SignalTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::String => {
                todo!();
                // self.as_typed_term::<StringTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Symbol => {
                todo!();
                // self.as_typed_term::<SymbolTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Tree => {
                todo!();
                // self.as_typed_term::<TreeTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Variable => {
                todo!();
                // self.as_typed_term::<VariableTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::EmptyIterator => {
                todo!();
                // self.as_typed_term::<EmptyIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::EvaluateIterator => {
                todo!();
                // self.as_typed_term::<EvaluateIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::FilterIterator => {
                todo!();
                // self.as_typed_term::<FilterIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::FlattenIterator => {
                todo!();
                // self.as_typed_term::<FlattenIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::HashmapKeysIterator => {
                todo!();
                // self.as_typed_term::<HashmapKeysIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::HashmapValuesIterator => {
                todo!();
                // self.as_typed_term::<HashmapValuesIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::IntegersIterator => {
                todo!();
                // self.as_typed_term::<IntegersIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::IntersperseIterator => {
                todo!();
                // self.as_typed_term::<IntersperseIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::MapIterator => {
                todo!();
                // self.as_typed_term::<MapIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::OnceIterator => {
                todo!();
                // self.as_typed_term::<OnceIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::RangeIterator => {
                todo!();
                // self.as_typed_term::<RangeIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::RepeatIterator => {
                todo!();
                // self.as_typed_term::<RepeatIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::SkipIterator => {
                todo!();
                // self.as_typed_term::<SkipIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::TakeIterator => {
                todo!();
                // self.as_typed_term::<TakeIteratorTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::ZipIterator => {
                todo!();
                // self.as_typed_term::<ZipIteratorTerm>().as_inner().compile(eager, state, options)
            }
        }
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<IntTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.value()),
        })));

        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateInt,
        ));

        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<FloatTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::F64(self.value()),
        })));

        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateFloat,
        ));

        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<BooleanTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.value() as i32),
        })));

        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateBoolean,
        ));

        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<ListTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        // Get the list capacity -> Capacity
        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.len() as i32),
        })));

        // Allocate the list -> List
        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::AllocateList,
        ));

        for (idx, item) in self.iter().enumerate() {
            // Duplicate the list pointer -> List List
            instructions.push_back(CompiledInstruction::Duplicate);
            // Push the item index -> List List Idx
            instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
                value: Value::I32(idx as i32),
            })));

            // Compile the child item -> List List Idx Item
            instructions.extend(item.compile(eager, state, options)?);
            // Set the item to the index -> List
            instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::SetListItem,
            ));
        }

        // Instantiate List with correct length -> List Length
        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(self.len() as i32),
        })));

        // Initialize the list term -> List
        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::InitList,
        ));

        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<BuiltinTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        // Put the builtin term's number onto the stack
        instructions.push_back(CompiledInstruction::Wasm(Instr::Const(Const {
            value: Value::I32(u32::from(self.target()) as i32),
        })));

        // Create builtin term
        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateBuiltin,
        ));

        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm for ArenaRef<ApplicationTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult {
        let mut instructions = CompiledExpression::default();

        // Compile the target
        instructions.extend(self.target().compile(eager, state, options)?);

        // Compile the args list
        instructions.extend(self.args().as_term().compile(eager, state, options)?);

        // Create application term
        instructions.push_back(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateApplication,
        ));

        Ok(instructions)
    }
}

pub enum TermPointerIterator {
    Application(ApplicationTermPointerIter),
    Boolean(BooleanTermPointerIter),
    Builtin(BuiltinTermPointerIter),
    Cell(CellTermPointerIter),
    Compiled(CompiledTermPointerIter),
    Condition(ConditionTermPointerIter),
    Constructor(ConstructorTermPointerIter),
    Date(DateTermPointerIter),
    Effect(EffectTermPointerIter),
    Float(FloatTermPointerIter),
    Hashmap(HashmapTermPointerIter),
    Hashset(HashsetTermPointerIter),
    Int(IntTermPointerIter),
    Lambda(LambdaTermPointerIter),
    Let(LetTermPointerIter),
    List(ListTermPointerIter),
    Nil(NilTermPointerIter),
    Partial(PartialTermPointerIter),
    Pointer(PointerTermPointerIter),
    Record(RecordTermPointerIter),
    Signal(SignalTermPointerIter),
    String(StringTermPointerIter),
    Symbol(SymbolTermPointerIter),
    Tree(TreeTermPointerIter),
    Variable(VariableTermPointerIter),
    EmptyIterator(EmptyIteratorTermPointerIter),
    EvaluateIterator(EvaluateIteratorTermPointerIter),
    FilterIterator(FilterIteratorTermPointerIter),
    FlattenIterator(FlattenIteratorTermPointerIter),
    HashmapKeysIterator(HashmapKeysIteratorTermPointerIter),
    HashmapValuesIterator(HashmapValuesIteratorTermPointerIter),
    IntegersIterator(IntegersIteratorTermPointerIter),
    IntersperseIterator(IntersperseIteratorTermPointerIter),
    MapIterator(MapIteratorTermPointerIter),
    OnceIterator(OnceIteratorTermPointerIter),
    RangeIterator(RangeIteratorTermPointerIter),
    RepeatIterator(RepeatIteratorTermPointerIter),
    SkipIterator(SkipIteratorTermPointerIter),
    TakeIterator(TakeIteratorTermPointerIter),
    ZipIterator(ZipIteratorTermPointerIter),
}

impl Iterator for TermPointerIterator {
    type Item = ArenaPointer;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TermPointerIterator::Application(inner) => inner.next(),
            TermPointerIterator::Boolean(inner) => inner.next(),
            TermPointerIterator::Builtin(inner) => inner.next(),
            TermPointerIterator::Cell(inner) => inner.next(),
            TermPointerIterator::Compiled(inner) => inner.next(),
            TermPointerIterator::Condition(inner) => inner.next(),
            TermPointerIterator::Constructor(inner) => inner.next(),
            TermPointerIterator::Date(inner) => inner.next(),
            TermPointerIterator::Effect(inner) => inner.next(),
            TermPointerIterator::Float(inner) => inner.next(),
            TermPointerIterator::Hashmap(inner) => inner.next(),
            TermPointerIterator::Hashset(inner) => inner.next(),
            TermPointerIterator::Int(inner) => inner.next(),
            TermPointerIterator::Lambda(inner) => inner.next(),
            TermPointerIterator::Let(inner) => inner.next(),
            TermPointerIterator::List(inner) => inner.next(),
            TermPointerIterator::Nil(inner) => inner.next(),
            TermPointerIterator::Partial(inner) => inner.next(),
            TermPointerIterator::Pointer(inner) => inner.next(),
            TermPointerIterator::Record(inner) => inner.next(),
            TermPointerIterator::Signal(inner) => inner.next(),
            TermPointerIterator::String(inner) => inner.next(),
            TermPointerIterator::Symbol(inner) => inner.next(),
            TermPointerIterator::Tree(inner) => inner.next(),
            TermPointerIterator::Variable(inner) => inner.next(),
            TermPointerIterator::EmptyIterator(inner) => inner.next(),
            TermPointerIterator::EvaluateIterator(inner) => inner.next(),
            TermPointerIterator::FilterIterator(inner) => inner.next(),
            TermPointerIterator::FlattenIterator(inner) => inner.next(),
            TermPointerIterator::HashmapKeysIterator(inner) => inner.next(),
            TermPointerIterator::HashmapValuesIterator(inner) => inner.next(),
            TermPointerIterator::IntegersIterator(inner) => inner.next(),
            TermPointerIterator::IntersperseIterator(inner) => inner.next(),
            TermPointerIterator::MapIterator(inner) => inner.next(),
            TermPointerIterator::OnceIterator(inner) => inner.next(),
            TermPointerIterator::RangeIterator(inner) => inner.next(),
            TermPointerIterator::RepeatIterator(inner) => inner.next(),
            TermPointerIterator::SkipIterator(inner) => inner.next(),
            TermPointerIterator::TakeIterator(inner) => inner.next(),
            TermPointerIterator::ZipIterator(inner) => inner.next(),
        }
    }
}

impl<A: Arena + Clone> PointerIter for ArenaRef<Term, A> {
    type Iter<'a> = TermPointerIterator
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => TermPointerIterator::Application(
                self.as_typed_term::<ApplicationTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::Boolean => {
                TermPointerIterator::Boolean(self.as_typed_term::<BooleanTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Builtin => {
                TermPointerIterator::Builtin(self.as_typed_term::<BuiltinTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Cell => {
                TermPointerIterator::Cell(self.as_typed_term::<CellTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Compiled => TermPointerIterator::Compiled(
                self.as_typed_term::<CompiledTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::Condition => TermPointerIterator::Condition(
                self.as_typed_term::<ConditionTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::Constructor => TermPointerIterator::Constructor(
                self.as_typed_term::<ConstructorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::Date => {
                TermPointerIterator::Date(self.as_typed_term::<DateTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Effect => {
                TermPointerIterator::Effect(self.as_typed_term::<EffectTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Float => {
                TermPointerIterator::Float(self.as_typed_term::<FloatTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Hashmap => {
                TermPointerIterator::Hashmap(self.as_typed_term::<HashmapTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Hashset => {
                TermPointerIterator::Hashset(self.as_typed_term::<HashsetTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Int => {
                TermPointerIterator::Int(self.as_typed_term::<IntTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Lambda => {
                TermPointerIterator::Lambda(self.as_typed_term::<LambdaTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Let => {
                TermPointerIterator::Let(self.as_typed_term::<LetTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::List => TermPointerIterator::List(PointerIter::iter(
                &self.as_typed_term::<ListTerm>().as_inner(),
            )),
            TermTypeDiscriminants::Nil => {
                TermPointerIterator::Nil(self.as_typed_term::<NilTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Partial => {
                TermPointerIterator::Partial(self.as_typed_term::<PartialTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Pointer => {
                TermPointerIterator::Pointer(self.as_typed_term::<PointerTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Record => {
                TermPointerIterator::Record(self.as_typed_term::<RecordTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Signal => {
                TermPointerIterator::Signal(self.as_typed_term::<SignalTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::String => {
                TermPointerIterator::String(self.as_typed_term::<StringTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Symbol => {
                TermPointerIterator::Symbol(self.as_typed_term::<SymbolTerm>().as_inner().iter())
            }
            TermTypeDiscriminants::Tree => TermPointerIterator::Tree(PointerIter::iter(
                &self.as_typed_term::<TreeTerm>().as_inner(),
            )),
            TermTypeDiscriminants::Variable => TermPointerIterator::Variable(
                self.as_typed_term::<VariableTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::EmptyIterator => TermPointerIterator::EmptyIterator(
                self.as_typed_term::<EmptyIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::EvaluateIterator => TermPointerIterator::EvaluateIterator(
                self.as_typed_term::<EvaluateIteratorTerm>()
                    .as_inner()
                    .iter(),
            ),
            TermTypeDiscriminants::FilterIterator => TermPointerIterator::FilterIterator(
                self.as_typed_term::<FilterIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::FlattenIterator => TermPointerIterator::FlattenIterator(
                self.as_typed_term::<FlattenIteratorTerm>()
                    .as_inner()
                    .iter(),
            ),
            TermTypeDiscriminants::HashmapKeysIterator => TermPointerIterator::HashmapKeysIterator(
                self.as_typed_term::<HashmapKeysIteratorTerm>()
                    .as_inner()
                    .iter(),
            ),
            TermTypeDiscriminants::HashmapValuesIterator => {
                TermPointerIterator::HashmapValuesIterator(
                    self.as_typed_term::<HashmapValuesIteratorTerm>()
                        .as_inner()
                        .iter(),
                )
            }
            TermTypeDiscriminants::IntegersIterator => TermPointerIterator::IntegersIterator(
                self.as_typed_term::<IntegersIteratorTerm>()
                    .as_inner()
                    .iter(),
            ),
            TermTypeDiscriminants::IntersperseIterator => TermPointerIterator::IntersperseIterator(
                self.as_typed_term::<IntersperseIteratorTerm>()
                    .as_inner()
                    .iter(),
            ),
            TermTypeDiscriminants::MapIterator => TermPointerIterator::MapIterator(
                self.as_typed_term::<MapIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::OnceIterator => TermPointerIterator::OnceIterator(
                self.as_typed_term::<OnceIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::RangeIterator => TermPointerIterator::RangeIterator(
                self.as_typed_term::<RangeIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::RepeatIterator => TermPointerIterator::RepeatIterator(
                self.as_typed_term::<RepeatIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::SkipIterator => TermPointerIterator::SkipIterator(
                self.as_typed_term::<SkipIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::TakeIterator => TermPointerIterator::TakeIterator(
                self.as_typed_term::<TakeIteratorTerm>().as_inner().iter(),
            ),
            TermTypeDiscriminants::ZipIterator => TermPointerIterator::ZipIterator(
                self.as_typed_term::<ZipIteratorTerm>().as_inner().iter(),
            ),
        }
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<Term, A> {
    fn should_intern(&self, eager: Eagerness) -> bool {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Boolean => self
                .as_typed_term::<BooleanTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Builtin => self
                .as_typed_term::<BuiltinTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Cell => self
                .as_typed_term::<CellTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Compiled => self
                .as_typed_term::<CompiledTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Condition => self
                .as_typed_term::<ConditionTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Constructor => self
                .as_typed_term::<ConstructorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Date => self
                .as_typed_term::<DateTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Effect => self
                .as_typed_term::<EffectTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Float => self
                .as_typed_term::<FloatTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Hashmap => self
                .as_typed_term::<HashmapTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Hashset => self
                .as_typed_term::<HashsetTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Int => self
                .as_typed_term::<IntTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Lambda => self
                .as_typed_term::<LambdaTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Let => self
                .as_typed_term::<LetTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::List => self
                .as_typed_term::<ListTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Nil => self
                .as_typed_term::<NilTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Partial => self
                .as_typed_term::<PartialTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Pointer => self
                .as_typed_term::<PointerTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Record => self
                .as_typed_term::<RecordTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Signal => self
                .as_typed_term::<SignalTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::String => self
                .as_typed_term::<StringTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Symbol => self
                .as_typed_term::<SymbolTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Tree => self
                .as_typed_term::<TreeTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::Variable => self
                .as_typed_term::<VariableTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::EmptyIterator => self
                .as_typed_term::<EmptyIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::EvaluateIterator => self
                .as_typed_term::<EvaluateIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::FilterIterator => self
                .as_typed_term::<FilterIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::FlattenIterator => self
                .as_typed_term::<FlattenIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::HashmapKeysIterator => self
                .as_typed_term::<HashmapKeysIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::HashmapValuesIterator => self
                .as_typed_term::<HashmapValuesIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::IntegersIterator => self
                .as_typed_term::<IntegersIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::IntersperseIterator => self
                .as_typed_term::<IntersperseIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::MapIterator => self
                .as_typed_term::<MapIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::OnceIterator => self
                .as_typed_term::<OnceIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::RangeIterator => self
                .as_typed_term::<RangeIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::RepeatIterator => self
                .as_typed_term::<RepeatIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::SkipIterator => self
                .as_typed_term::<SkipIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::TakeIterator => self
                .as_typed_term::<TakeIteratorTerm>()
                .as_inner()
                .should_intern(eager),
            TermTypeDiscriminants::ZipIterator => self
                .as_typed_term::<ZipIteratorTerm>()
                .as_inner()
                .should_intern(eager),
        }
    }
}
