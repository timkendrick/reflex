// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::{
    linked_list::{IntoIter, Iter},
    HashMap, LinkedList,
};

use reflex::{
    core::{Eagerness, NodeId},
    hash::{HashId, IntMap},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use walrus::{
    ir::{Call, Const, Instr, LocalGet, LocalTee, Value},
    ExportItem, FunctionId, LocalId,
};

use crate::{
    allocator::ArenaAllocator, hash::TermSize, stdlib::Stdlib, term_type::*, ArenaRef, PointerIter,
    Term, TermPointer,
};

#[derive(Clone, Debug)]
pub enum CompilerError {
    MissingRuntimeBuiltin(&'static str),
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::MissingRuntimeBuiltin(function_name) => {
                write!(f, "Missing runtime builtin: {}", function_name)
            }
        }
    }
}

impl std::error::Error for CompilerError {}

type CompilerResult = Result<CompiledExpression, CompilerError>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, EnumIter)]
pub enum RuntimeBuiltin {
    CreateInt,
    Evaluate,
    CreateFloat,
    CreateBoolean,
    CreateApplication,
    CreateBuiltin,
    AllocateList,
    SetListItem,
    InitList,
}

impl RuntimeBuiltin {
    pub fn name(self) -> &'static str {
        match self {
            RuntimeBuiltin::CreateInt => "createInt",
            RuntimeBuiltin::Evaluate => "evaluate",
            RuntimeBuiltin::CreateFloat => "createFloat",
            RuntimeBuiltin::CreateBoolean => "createBoolean",
            RuntimeBuiltin::CreateApplication => "createApplication",
            RuntimeBuiltin::CreateBuiltin => "createBuiltin",
            RuntimeBuiltin::AllocateList => "allocateList",
            RuntimeBuiltin::SetListItem => "setListItem",
            RuntimeBuiltin::InitList => "initList",
        }
    }
}

pub type ModuleLinkTable = HashMap<RuntimeBuiltin, FunctionId>;

pub fn generate_link_table(
    exports: &walrus::ModuleExports,
) -> Result<ModuleLinkTable, CompilerError> {
    let function_map = exports
        .iter()
        .filter_map(|e| {
            if let ExportItem::Function(fid) = e.item {
                Some((e.name.clone(), fid))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    let link_table = RuntimeBuiltin::iter()
        .map(|rb| {
            function_map
                .get(rb.name())
                .map(|id| (rb, *id))
                .ok_or(CompilerError::MissingRuntimeBuiltin(rb.name()))
        })
        .collect::<Result<_, _>>();

    link_table
}

#[derive(Debug, Clone)]
pub enum CompiledInstruction {
    Wasm(walrus::ir::Instr),
    CallRuntimeBuiltin(RuntimeBuiltin),
    CallStdlib(Stdlib),
    /// Duplicates the top item of the stack
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
    pub fn link_instrs(
        self,
        temp_local: LocalId,
        link_table: &HashMap<RuntimeBuiltin, FunctionId>,
    ) -> Vec<Instr> {
        self.instrs
            .into_iter()
            .flat_map(|ci| match ci {
                CompiledInstruction::Wasm(i) => vec![i],
                CompiledInstruction::CallRuntimeBuiltin(rb) => vec![Instr::Call(Call {
                    func: link_table.get(&rb).unwrap().clone(),
                })],
                CompiledInstruction::CallStdlib(_) => todo!(),
                CompiledInstruction::Duplicate => vec![
                    Instr::LocalTee(LocalTee { local: temp_local }),
                    Instr::LocalGet(LocalGet { local: temp_local }),
                ],
            })
            .collect()
    }

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

#[derive(Default)]
pub struct CompilerState {}

pub trait CompileWasm {
    fn compile(
        &self,
        eager: Eagerness,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult;
}

impl<A: ArenaAllocator + Clone> CompileWasm for ArenaRef<Term, A> {
    fn compile(
        &self,
        eager: Eagerness,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult {
        match self.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => self
                .as_typed_term::<ApplicationTerm>()
                .as_inner()
                .compile(eager, state, options),
            TermTypeDiscriminants::Boolean => {
                todo!();
                // self.as_typed_term::<BooleanTerm>().as_inner().compile(eager, state, options)
            }
            TermTypeDiscriminants::Builtin => {
                // todo!();
                self.as_typed_term::<BuiltinTerm>()
                    .as_inner()
                    .compile(eager, state, options)
            }
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

impl<A: ArenaAllocator + Clone> CompileWasm for ArenaRef<IntTerm, A> {
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

impl<A: ArenaAllocator + Clone> CompileWasm for ArenaRef<FloatTerm, A> {
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

impl<A: ArenaAllocator + Clone> CompileWasm for ArenaRef<BooleanTerm, A> {
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

impl<A: ArenaAllocator + Clone> CompileWasm for ArenaRef<ListTerm, A> {
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

impl<A: ArenaAllocator + Clone> CompileWasm for ArenaRef<BuiltinTerm, A> {
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

impl<A: ArenaAllocator + Clone> CompileWasm for ArenaRef<ApplicationTerm, A> {
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

#[derive(Default)]
pub struct SerializerState {
    allocated_terms: IntMap<HashId, TermPointer>,
}

pub trait Serialize {
    fn serialize<A: ArenaAllocator>(
        &self,
        destination: &mut A,
        state: &mut SerializerState,
    ) -> TermPointer;
}

impl<ASource: ArenaAllocator + Clone, T: NodeId + TermSize + Clone> Serialize
    for ArenaRef<T, ASource>
where
    ArenaRef<T, ASource>: PointerIter,
{
    fn serialize<ADest: ArenaAllocator>(
        &self,
        destination: &mut ADest,
        state: &mut SerializerState,
    ) -> TermPointer {
        // Check if we have already serialized this before
        let cached_result = state
            .allocated_terms
            .get(&self.read_value(|term| term.id()));
        if let Some(existing) = cached_result {
            return *existing;
        }

        let children = PointerIter::iter(self)
            .filter_map(|inner_pointer| {
                let value_pointer = self
                    .arena
                    .read_value(inner_pointer, |target_pointer: &TermPointer| {
                        *target_pointer
                    })
                    .as_non_null()?;
                Some((inner_pointer, value_pointer))
            })
            .map(|(inner_pointer, value_pointer)| {
                (
                    // The offset of the field of the term within the struct
                    u32::from(inner_pointer) - u32::from(self.pointer),
                    ArenaRef::<T, ASource>::new(self.arena.clone(), value_pointer)
                        .serialize(destination, state),
                )
            })
            .collect::<Vec<_>>();

        let new_term = destination.allocate(self.read_value(|t| t.clone()));

        for (delta, child_pointer) in children {
            destination.write(new_term.offset(delta), child_pointer)
        }

        state
            .allocated_terms
            .insert(self.read_value(|term| term.id()), new_term);

        new_term
    }
}

pub enum TermPointerIterator<'a, A: ArenaAllocator + Clone + 'a> {
    Application(<ArenaRef<ApplicationTerm, A> as PointerIter>::Iter<'a>),
    Boolean(<ArenaRef<BooleanTerm, A> as PointerIter>::Iter<'a>),
    Builtin(<ArenaRef<BuiltinTerm, A> as PointerIter>::Iter<'a>),
    Cell(<ArenaRef<CellTerm, A> as PointerIter>::Iter<'a>),
    Compiled(<ArenaRef<CompiledTerm, A> as PointerIter>::Iter<'a>),
    Condition(<ArenaRef<ConditionTerm, A> as PointerIter>::Iter<'a>),
    Constructor(<ArenaRef<ConstructorTerm, A> as PointerIter>::Iter<'a>),
    Date(<ArenaRef<DateTerm, A> as PointerIter>::Iter<'a>),
    Effect(<ArenaRef<EffectTerm, A> as PointerIter>::Iter<'a>),
    Float(<ArenaRef<FloatTerm, A> as PointerIter>::Iter<'a>),
    Hashmap(<ArenaRef<HashmapTerm, A> as PointerIter>::Iter<'a>),
    Hashset(<ArenaRef<HashsetTerm, A> as PointerIter>::Iter<'a>),
    Int(<ArenaRef<IntTerm, A> as PointerIter>::Iter<'a>),
    Lambda(<ArenaRef<LambdaTerm, A> as PointerIter>::Iter<'a>),
    Let(<ArenaRef<LetTerm, A> as PointerIter>::Iter<'a>),
    List(<ArenaRef<ListTerm, A> as PointerIter>::Iter<'a>),
    Nil(<ArenaRef<NilTerm, A> as PointerIter>::Iter<'a>),
    Partial(<ArenaRef<PartialTerm, A> as PointerIter>::Iter<'a>),
    Pointer(<ArenaRef<PointerTerm, A> as PointerIter>::Iter<'a>),
    Record(<ArenaRef<RecordTerm, A> as PointerIter>::Iter<'a>),
    Signal(<ArenaRef<SignalTerm, A> as PointerIter>::Iter<'a>),
    String(<ArenaRef<StringTerm, A> as PointerIter>::Iter<'a>),
    Symbol(<ArenaRef<SymbolTerm, A> as PointerIter>::Iter<'a>),
    Tree(<ArenaRef<TreeTerm, A> as PointerIter>::Iter<'a>),
    Variable(<ArenaRef<VariableTerm, A> as PointerIter>::Iter<'a>),
    EmptyIterator(<ArenaRef<EmptyIteratorTerm, A> as PointerIter>::Iter<'a>),
    EvaluateIterator(<ArenaRef<EvaluateIteratorTerm, A> as PointerIter>::Iter<'a>),
    FilterIterator(<ArenaRef<FilterIteratorTerm, A> as PointerIter>::Iter<'a>),
    FlattenIterator(<ArenaRef<FlattenIteratorTerm, A> as PointerIter>::Iter<'a>),
    HashmapKeysIterator(<ArenaRef<HashmapKeysIteratorTerm, A> as PointerIter>::Iter<'a>),
    HashmapValuesIterator(<ArenaRef<HashmapValuesIteratorTerm, A> as PointerIter>::Iter<'a>),
    IntegersIterator(<ArenaRef<IntegersIteratorTerm, A> as PointerIter>::Iter<'a>),
    IntersperseIterator(<ArenaRef<IntersperseIteratorTerm, A> as PointerIter>::Iter<'a>),
    MapIterator(<ArenaRef<MapIteratorTerm, A> as PointerIter>::Iter<'a>),
    OnceIterator(<ArenaRef<OnceIteratorTerm, A> as PointerIter>::Iter<'a>),
    RangeIterator(<ArenaRef<RangeIteratorTerm, A> as PointerIter>::Iter<'a>),
    RepeatIterator(<ArenaRef<RepeatIteratorTerm, A> as PointerIter>::Iter<'a>),
    SkipIterator(<ArenaRef<SkipIteratorTerm, A> as PointerIter>::Iter<'a>),
    TakeIterator(<ArenaRef<TakeIteratorTerm, A> as PointerIter>::Iter<'a>),
    ZipIterator(<ArenaRef<ZipIteratorTerm, A> as PointerIter>::Iter<'a>),
}

impl<'a, A: ArenaAllocator + Clone> Iterator for TermPointerIterator<'a, A> {
    type Item = TermPointer;

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

impl<A: ArenaAllocator + Clone> PointerIter for ArenaRef<Term, A> {
    type Iter<'a> = TermPointerIterator<'a, A>
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
