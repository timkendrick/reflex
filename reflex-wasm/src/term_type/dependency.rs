// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{collections::HashSet, marker::PhantomData};

use reflex::core::{ArgType, DependencyList, GraphNode, SerializeJson, StackOffset};
use serde_json::Value as JsonValue;
use strum_macros::EnumDiscriminants;

use crate::{
    allocator::Arena,
    compiler::{
        instruction, runtime::builtin::RuntimeBuiltin, CompileWasm, CompiledBlockBuilder,
        CompilerOptions, CompilerResult, CompilerStack, CompilerState, ConstValue, Internable,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::{ConditionTerm, TypedTerm},
    utils::chunks_to_u64,
    ArenaPointer, ArenaRef, PointerIter, Term,
};

#[derive(Clone, Copy, Debug, EnumDiscriminants)]
#[repr(C)]
pub enum DependencyTerm {
    Cache(CacheDependency),
    State(StateDependency),
}

#[derive(Debug, Clone)]
pub enum DependencyTermPointerIter {
    Cache(CacheDependencyPointerIter),
    State(StateDependencyPointerIter),
}

impl Iterator for DependencyTermPointerIter {
    type Item = ArenaPointer;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Cache(inner) => inner.next(),
            Self::State(inner) => inner.next(),
        }
    }
}

impl DependencyTerm {
    fn dependency_type(&self) -> DependencyTermDiscriminants {
        DependencyTermDiscriminants::from(self)
    }
}
impl TermSize for DependencyTerm {
    fn size_of(&self) -> usize {
        let discriminant_size = std::mem::size_of::<u32>();
        let value_size = match self {
            Self::Cache(dependency) => dependency.size_of(),
            Self::State(dependency) => dependency.size_of(),
        };
        discriminant_size + value_size
    }
}
impl TermHash for DependencyTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let hasher = hasher.write_u8(self.dependency_type() as u8);
        match self {
            Self::Cache(dependency) => dependency.hash(hasher, arena),
            Self::State(dependency) => dependency.hash(hasher, arena),
        }
    }
}

impl<A: Arena + Clone> ArenaRef<DependencyTerm, A> {
    pub(crate) fn dependency_type(&self) -> DependencyTermDiscriminants {
        self.read_value(|term| term.dependency_type())
    }
    pub(crate) fn as_typed_dependency<V>(&self) -> &ArenaRef<TypedDependency<V>, A> {
        unsafe {
            std::mem::transmute::<&ArenaRef<DependencyTerm, A>, &ArenaRef<TypedDependency<V>, A>>(
                self,
            )
        }
    }
    pub fn as_cache_dependency(&self) -> Option<&ArenaRef<TypedDependency<CacheDependency>, A>> {
        match self.read_value(|term| term.dependency_type()) {
            DependencyTermDiscriminants::Cache => {
                Some(self.as_typed_dependency::<CacheDependency>())
            }
            _ => None,
        }
    }
    pub fn as_state_dependency(&self) -> Option<&ArenaRef<TypedDependency<StateDependency>, A>> {
        match self.read_value(|term| term.dependency_type()) {
            DependencyTermDiscriminants::State => {
                Some(self.as_typed_dependency::<StateDependency>())
            }
            _ => None,
        }
    }
}

impl<A: Arena + Clone> PointerIter for ArenaRef<DependencyTerm, A> {
    type Iter<'a> = DependencyTermPointerIter
    where
        Self: 'a;
    fn iter<'a>(&'a self) -> Self::Iter<'a>
    where
        Self: 'a,
    {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => DependencyTermPointerIter::Cache(
                self.as_typed_dependency::<CacheDependency>()
                    .as_inner()
                    .iter(),
            ),
            DependencyTermDiscriminants::State => DependencyTermPointerIter::State(
                self.as_typed_dependency::<StateDependency>()
                    .as_inner()
                    .iter(),
            ),
        }
    }
}

#[repr(transparent)]
pub struct TypedDependency<V> {
    dependency: DependencyTerm,
    _type: PhantomData<V>,
}
impl<V> TypedDependency<V> {
    pub(crate) fn get_inner(&self) -> &V {
        unsafe {
            match &self.dependency {
                DependencyTerm::Cache(inner) => std::mem::transmute::<&CacheDependency, &V>(inner),
                DependencyTerm::State(inner) => std::mem::transmute::<&StateDependency, &V>(inner),
            }
        }
    }
}

impl<A: Arena + Clone, V> ArenaRef<TypedDependency<V>, A> {
    pub fn as_inner(&self) -> ArenaRef<V, A> {
        self.inner_ref(|dependency| dependency.get_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<DependencyTerm, A> {
    fn size(&self) -> usize {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => self
                .as_typed_dependency::<CacheDependency>()
                .as_inner()
                .size(),
            DependencyTermDiscriminants::State => self
                .as_typed_dependency::<StateDependency>()
                .as_inner()
                .size(),
        }
    }
    fn capture_depth(&self) -> StackOffset {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => self
                .as_typed_dependency::<CacheDependency>()
                .as_inner()
                .capture_depth(),
            DependencyTermDiscriminants::State => self
                .as_typed_dependency::<StateDependency>()
                .as_inner()
                .capture_depth(),
        }
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => self
                .as_typed_dependency::<CacheDependency>()
                .as_inner()
                .free_variables(),
            DependencyTermDiscriminants::State => self
                .as_typed_dependency::<StateDependency>()
                .as_inner()
                .free_variables(),
        }
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => self
                .as_typed_dependency::<CacheDependency>()
                .as_inner()
                .count_variable_usages(offset),
            DependencyTermDiscriminants::State => self
                .as_typed_dependency::<StateDependency>()
                .as_inner()
                .count_variable_usages(offset),
        }
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => self
                .as_typed_dependency::<CacheDependency>()
                .as_inner()
                .dynamic_dependencies(deep),
            DependencyTermDiscriminants::State => self
                .as_typed_dependency::<StateDependency>()
                .as_inner()
                .dynamic_dependencies(deep),
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => self
                .as_typed_dependency::<CacheDependency>()
                .as_inner()
                .has_dynamic_dependencies(deep),
            DependencyTermDiscriminants::State => self
                .as_typed_dependency::<StateDependency>()
                .as_inner()
                .has_dynamic_dependencies(deep),
        }
    }
    fn is_static(&self) -> bool {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => self
                .as_typed_dependency::<CacheDependency>()
                .as_inner()
                .is_static(),
            DependencyTermDiscriminants::State => self
                .as_typed_dependency::<StateDependency>()
                .as_inner()
                .is_static(),
        }
    }
    fn is_atomic(&self) -> bool {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => self
                .as_typed_dependency::<CacheDependency>()
                .as_inner()
                .is_atomic(),
            DependencyTermDiscriminants::State => self
                .as_typed_dependency::<StateDependency>()
                .as_inner()
                .is_atomic(),
        }
    }
    fn is_complex(&self) -> bool {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => self
                .as_typed_dependency::<CacheDependency>()
                .as_inner()
                .is_complex(),
            DependencyTermDiscriminants::State => self
                .as_typed_dependency::<StateDependency>()
                .as_inner()
                .is_complex(),
        }
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<DependencyTerm, A> {
    fn should_intern(&self, eager: ArgType) -> bool {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => self
                .as_typed_dependency::<CacheDependency>()
                .as_inner()
                .should_intern(eager),
            DependencyTermDiscriminants::State => self
                .as_typed_dependency::<StateDependency>()
                .as_inner()
                .should_intern(eager),
        }
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<DependencyTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Err(format!("Unable to serialize term: {}", self))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        Err(format!(
            "Unable to create patch for terms: {}, {}",
            self, target
        ))
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<DependencyTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        match (
            self.dependency_type(),
            other.read_value(|term| term.dependency_type()),
        ) {
            (DependencyTermDiscriminants::Cache, DependencyTermDiscriminants::Cache) => {
                self.as_typed_dependency::<CacheDependency>().as_inner()
                    == other.as_typed_dependency::<CacheDependency>().as_inner()
            }
            (DependencyTermDiscriminants::State, DependencyTermDiscriminants::State) => {
                self.as_typed_dependency::<StateDependency>().as_inner()
                    == other.as_typed_dependency::<StateDependency>().as_inner()
            }
            _ => false,
        }
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<DependencyTerm, A> {}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<DependencyTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => {
                std::fmt::Display::fmt(&self.as_typed_dependency::<CacheDependency>().as_inner(), f)
            }
            DependencyTermDiscriminants::State => {
                std::fmt::Display::fmt(&self.as_typed_dependency::<StateDependency>().as_inner(), f)
            }
        }
    }
}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<DependencyTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct CacheDependency {
    pub key: [u32; 2],
}
impl TermSize for CacheDependency {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for CacheDependency {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let value = chunks_to_u64(self.key);
        hasher.hash(&value, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<CacheDependency, A> {
    pub fn key(&self) -> u64 {
        self.read_value(|term| chunks_to_u64(term.key))
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<CacheDependency, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        0
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        HashSet::new()
    }
    fn count_variable_usages(&self, _offset: StackOffset) -> usize {
        0
    }
    fn dynamic_dependencies(&self, _deep: bool) -> DependencyList {
        DependencyList::empty()
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        false
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        false
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<CacheDependency, A> {
    fn should_intern(&self, _eager: ArgType) -> bool {
        true
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<CacheDependency, A> {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<CacheDependency, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<CacheDependency, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<CacheDependency, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<#{}>", self.key())
    }
}

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct StateDependency {
    pub condition: ArenaPointer,
}
impl TermSize for StateDependency {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for StateDependency {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        let state_token_hash = arena.read_value::<Term, _>(self.condition, |term| term.id());
        hasher.hash(&state_token_hash, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<StateDependency, A> {
    pub fn condition(&self) -> ArenaRef<TypedTerm<ConditionTerm>, A> {
        ArenaRef::<TypedTerm<ConditionTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.condition),
        )
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<StateDependency, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        0
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        Default::default()
    }
    fn count_variable_usages(&self, _offset: StackOffset) -> usize {
        0
    }
    fn dynamic_dependencies(&self, _deep: bool) -> DependencyList {
        DependencyList::empty()
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        false
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<StateDependency, A> {
    fn should_intern(&self, eager: ArgType) -> bool {
        self.condition().as_term().should_intern(eager)
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<StateDependency, A> {
    fn eq(&self, other: &Self) -> bool {
        self.condition().as_term() == other.condition().as_term()
    }
}

impl<A: Arena + Clone> Eq for ArenaRef<StateDependency, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<StateDependency, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<StateDependency, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<!{}>", self.condition())
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<DependencyTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        match self.dependency_type() {
            DependencyTermDiscriminants::Cache => self
                .as_typed_dependency::<CacheDependency>()
                .as_inner()
                .compile(stack, state, options),
            DependencyTermDiscriminants::State => self
                .as_typed_dependency::<StateDependency>()
                .as_inner()
                .compile(stack, state, options),
        }
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<CacheDependency, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let key = self.key();
        let block = CompiledBlockBuilder::new(stack);
        // Push the key onto the stack
        // => [i64]
        let block = block.push(instruction::core::Const {
            value: ConstValue::U64(key),
        });
        // Invoke the term constructor
        // => [DependencyTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateCacheDependency,
        });
        block.finish()
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<StateDependency, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let state_token = self.condition();
        let state_token = state_token.as_term();
        let block = CompiledBlockBuilder::new(stack);
        // Yield the state token onto the stack
        // => [Term]
        let block = block.append_inner(|stack| state_token.compile(stack, state, options))?;
        // Invoke the term constructor
        // => [DependencyTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateStateDependency,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn condition() {
        assert_eq!(std::mem::size_of::<DependencyTerm>(), 12);
        assert_eq!(DependencyTermDiscriminants::Cache as u32, 0);
        assert_eq!(DependencyTermDiscriminants::State as u32, 1);
    }

    #[test]
    fn dependency_expression() {
        assert_eq!(
            TermType::Dependency(DependencyTerm::Cache(CacheDependency {
                key: [0x98765, 0x54321],
            }))
            .as_bytes(),
            [
                TermTypeDiscriminants::Dependency as u32,
                DependencyTermDiscriminants::Cache as u32,
                0x98765,
                0x54321,
            ],
        );
    }

    #[test]
    fn dependency_state() {
        assert_eq!(
            TermType::Dependency(DependencyTerm::State(StateDependency {
                condition: ArenaPointer(0x54321),
            }))
            .as_bytes(),
            [
                TermTypeDiscriminants::Dependency as u32,
                DependencyTermDiscriminants::State as u32,
                0x54321,
            ],
        );
    }
}
