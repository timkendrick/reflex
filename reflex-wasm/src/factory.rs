// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    cell::RefCell,
    collections::HashMap,
    iter::{empty, once},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use reflex::{
    core::{
        ApplicationTermType, Arity, BooleanTermType, BuiltinTermType, CompiledFunctionTermType,
        ConditionListType, ConditionType, ConstructorTermType, EffectTermType, Expression,
        ExpressionFactory, ExpressionListType, FloatTermType, FloatValue, HashmapTermType,
        HashsetTermType, HeapAllocator, InstructionPointer, IntTermType, IntValue, LambdaTermType,
        LetTermType, ListTermType, PartialApplicationTermType, RecordTermType, RecursiveTermType,
        RefType, SignalTermType, SignalType, StackOffset, StringTermType, StringValue,
        StructPrototypeType, SymbolId, SymbolTermType, VariableTermType,
    },
    hash::HashId,
};
use reflex_utils::WithExactSizeIterator;

use crate::{
    self as reflex_wasm,
    allocator::{Arena, ArenaAllocator},
    hash::TermSize,
    term_type::{
        ApplicationTerm, BooleanTerm, BuiltinTerm, ConditionTerm, ConstructorTerm, CustomCondition,
        EffectTerm, ErrorCondition, FloatTerm, HashmapTerm, HashsetTerm, IntTerm, LambdaTerm,
        LetTerm, ListTerm, NilTerm, PartialTerm, PendingCondition, RecordTerm, SignalTerm,
        StringTerm, SymbolTerm, TermType, TermTypeDiscriminants, TreeTerm, TypedTerm, VariableTerm,
        WasmExpression,
    },
    ArenaPointer, ArenaRef, FunctionIndex, Term,
};

#[derive(Debug)]
pub struct WasmTermFactory<A: Arena> {
    arena: Rc<RefCell<A>>,
}

impl<A: Arena> From<Rc<RefCell<A>>> for WasmTermFactory<A> {
    fn from(value: Rc<RefCell<A>>) -> Self {
        Self { arena: value }
    }
}

impl<A: Arena> WasmTermFactory<A>
where
    A: ArenaAllocator,
    Rc<RefCell<A>>: Arena,
{
    pub fn import<T: Expression>(
        &self,
        expression: &T,
        factory: &impl ExpressionFactory<T>,
    ) -> Result<WasmExpression<Self>, T>
    where
        T::Builtin: Into<crate::stdlib::Stdlib>,
    {
        if let Some(_) = factory.match_nil_term(expression) {
            Ok(self.create_nil_term())
        } else if let Some(term) = factory.match_boolean_term(expression) {
            Ok(self.create_boolean_term(term.value()))
        } else if let Some(term) = factory.match_int_term(expression) {
            Ok(self.create_int_term(term.value()))
        } else if let Some(term) = factory.match_float_term(expression) {
            Ok(self.create_float_term(term.value()))
        } else if let Some(term) = factory.match_string_term(expression) {
            let value = self.create_string(term.value().as_deref().as_str());
            Ok(self.create_string_term(value))
        } else if let Some(term) = factory.match_symbol_term(expression) {
            Ok(self.create_symbol_term(term.id()))
        } else if let Some(term) = factory.match_variable_term(expression) {
            Ok(self.create_variable_term(term.offset()))
        } else if let Some(term) = factory.match_effect_term(expression) {
            let condition = self.import_condition(term.condition().as_deref().deref(), factory)?;
            Ok(self.create_effect_term(condition))
        } else if let Some(term) = factory.match_let_term(expression) {
            let initializer = self.import(term.initializer().as_deref(), factory)?;
            let body = self.import(term.body().as_deref(), factory)?;
            Ok(self.create_let_term(initializer, body))
        } else if let Some(term) = factory.match_lambda_term(expression) {
            let num_args = term.num_args();
            let body = self.import(term.body().as_deref(), factory)?;
            Ok(self.create_lambda_term(num_args, body))
        } else if let Some(term) = factory.match_application_term(expression) {
            let target = self.import(term.target().as_deref(), factory)?;
            let args = term
                .args()
                .as_deref()
                .iter()
                .map(|arg| self.import(arg.as_deref(), factory))
                .collect::<Result<Vec<_>, _>>()?;
            let args = self.create_list(args);
            Ok(self.create_application_term(target, args))
        } else if let Some(term) = factory.match_partial_application_term(expression) {
            let target = self.import(term.target().as_deref(), factory)?;
            let args = term
                .args()
                .as_deref()
                .iter()
                .map(|arg| self.import(arg.as_deref(), factory))
                .collect::<Result<Vec<_>, _>>()?;
            let args = self.create_list(args);
            Ok(self.create_partial_application_term(target, args))
        } else if let Some(term) = factory.match_recursive_term(expression) {
            let body = self.import(term.factory().as_deref(), factory)?;
            Ok(self.create_recursive_term(body))
        } else if let Some(term) = factory.match_builtin_term(expression) {
            Ok(self.create_builtin_term(term.target()))
        } else if let Some(term) = factory.match_compiled_function_term(expression) {
            let term = term.as_deref();
            let address = term.address();
            let hash = term.hash();
            let required_args = term.required_args();
            let optional_args = term.optional_args();
            let variadic_args = term.variadic_args();
            Ok(self.create_compiled_function_term(
                address,
                hash,
                required_args,
                optional_args,
                variadic_args,
            ))
        } else if let Some(term) = factory.match_record_term(expression) {
            let keys = term
                .prototype()
                .as_deref()
                .keys()
                .as_deref()
                .iter()
                .map(|item| self.import(item.as_deref(), factory))
                .collect::<Result<Vec<_>, _>>()?;
            let keys = self.create_list(keys);
            let prototype = self.create_struct_prototype(keys);
            let values = term
                .values()
                .as_deref()
                .iter()
                .map(|item| self.import(item.as_deref(), factory))
                .collect::<Result<Vec<_>, _>>()?;
            let values = self.create_list(values);
            Ok(self.create_record_term(prototype, values))
        } else if let Some(term) = factory.match_constructor_term(expression) {
            let keys = term
                .prototype()
                .as_deref()
                .keys()
                .as_deref()
                .iter()
                .map(|key| self.import(key.as_deref(), factory))
                .collect::<Result<Vec<_>, _>>()?;
            let keys = self.create_list(keys);
            let prototype = self.create_struct_prototype(keys);
            Ok(self.create_constructor_term(prototype))
        } else if let Some(term) = factory.match_list_term(expression) {
            let items = term
                .items()
                .as_deref()
                .iter()
                .map(|key| self.import(key.as_deref(), factory))
                .collect::<Result<Vec<_>, _>>()?;
            let items = self.create_list(items);
            Ok(self.create_list_term(items))
        } else if let Some(term) = factory.match_hashmap_term(expression) {
            let keys = term
                .keys()
                .map(|term| self.import(term.as_deref(), factory));
            let values = term
                .values()
                .map(|term| self.import(term.as_deref(), factory));
            let entries = keys
                .zip(values)
                .map(|(key, value)| {
                    let key = key?;
                    let value = value?;
                    Ok((key, value))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(self.create_hashmap_term(entries))
        } else if let Some(term) = factory.match_hashset_term(expression) {
            let values = term
                .values()
                .map(|term| self.import(term.as_deref(), factory))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(self.create_hashset_term(values))
        } else if let Some(term) = factory.match_signal_term(expression) {
            let conditions = term
                .signals()
                .as_deref()
                .iter()
                .map(|condition| self.import_condition(condition.as_deref().deref(), factory))
                .collect::<Result<Vec<_>, _>>()?;
            let conditions = self.create_signal_list(conditions);
            Ok(self.create_signal_term(conditions))
        } else {
            Err(expression.clone())
        }
    }
    pub fn export<T: Expression>(
        &self,
        expression: &WasmExpression<Rc<RefCell<A>>>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        indirect_call_arity: &HashMap<FunctionIndex, Arity>,
    ) -> Result<T, WasmExpression<Rc<RefCell<A>>>> {
        if let Some(term) = expression.as_nil_term() {
            let _term = term.as_inner();
            Ok(factory.create_nil_term())
        } else if let Some(term) = expression.as_boolean_term() {
            let term = term.as_inner();
            Ok(factory.create_boolean_term(term.value()))
        } else if let Some(term) = expression.as_int_term() {
            let term = term.as_inner();
            Ok(factory.create_int_term(term.value()))
        } else if let Some(term) = expression.as_float_term() {
            let term = term.as_inner();
            Ok(factory.create_float_term(term.value()))
        } else if let Some(term) = expression.as_string_term() {
            let value = allocator.create_string(term.value().as_deref().as_str());
            Ok(factory.create_string_term(value))
        } else if let Some(term) = expression.as_symbol_term() {
            Ok(factory.create_symbol_term(term.id()))
        } else if let Some(term) = expression.as_variable_term() {
            let term = term.as_inner();
            Ok(factory.create_variable_term(term.offset()))
        } else if let Some(term) = expression.as_effect_term() {
            let term = term.as_inner();
            let condition = self.export_condition(
                term.condition().as_deref(),
                factory,
                allocator,
                indirect_call_arity,
            )?;
            Ok(factory.create_effect_term(condition))
        } else if let Some(term) = expression.as_let_term() {
            let term = term.as_inner();
            let initializer = self.export(
                term.initializer().as_deref(),
                factory,
                allocator,
                indirect_call_arity,
            )?;
            let body = self.export(
                term.body().as_deref(),
                factory,
                allocator,
                indirect_call_arity,
            )?;
            Ok(factory.create_let_term(initializer, body))
        } else if let Some(term) = expression.as_lambda_term() {
            let num_args = term.num_args();
            let body = self.export(
                term.body().as_deref(),
                factory,
                allocator,
                indirect_call_arity,
            )?;
            Ok(factory.create_lambda_term(num_args, body))
        } else if let Some(term) = expression.as_application_term() {
            let term = term.as_inner();
            let target = self.export(
                term.target().as_deref(),
                factory,
                allocator,
                indirect_call_arity,
            )?;
            let args = term
                .args()
                .as_deref()
                .iter()
                .map(|arg| self.export(arg.as_deref(), factory, allocator, indirect_call_arity))
                .collect::<Result<Vec<_>, _>>()?;
            let args = allocator.create_list(args);
            Ok(factory.create_application_term(target, args))
        } else if let Some(term) = expression.as_partial_term() {
            let term = term.as_inner();
            let target = self.export(
                term.target().as_deref(),
                factory,
                allocator,
                indirect_call_arity,
            )?;
            let args = term
                .args()
                .as_deref()
                .iter()
                .map(|arg| self.export(arg.as_deref(), factory, allocator, indirect_call_arity))
                .collect::<Result<Vec<_>, _>>()?;
            let args = allocator.create_list(args);
            Ok(factory.create_partial_application_term(target, args))
        } else if let Some(builtin) = expression.as_builtin_term() {
            let index = FunctionIndex::from(builtin.as_inner().read_value(|term| term.uid));
            match indirect_call_arity.get(&index).copied() {
                None => Err(expression.clone()),
                Some(arity) => {
                    let builtin_term_hash = {
                        let arena = expression.arena();
                        Term::new(
                            TermType::Builtin(BuiltinTerm {
                                uid: u32::from(index),
                            }),
                            arena,
                        )
                        .id()
                    };
                    Ok(factory.create_compiled_function_term(
                        InstructionPointer::new(u32::from(index) as usize),
                        builtin_term_hash,
                        arity.required().len(),
                        arity.optional().len(),
                        arity.variadic().is_some(),
                    ))
                }
            }
        } else if let Some(term) = expression.as_record_term() {
            let term = term.as_inner();
            let keys = term
                .prototype()
                .as_deref()
                .keys()
                .as_deref()
                .iter()
                .map(|key| self.export(key.as_deref(), factory, allocator, indirect_call_arity))
                .collect::<Result<Vec<_>, _>>()?;
            let keys = allocator.create_list(keys);
            let prototype = allocator.create_struct_prototype(keys);
            let values = term
                .values()
                .as_deref()
                .iter()
                .map(|key| self.export(key.as_deref(), factory, allocator, indirect_call_arity))
                .collect::<Result<Vec<_>, _>>()?;
            let values = allocator.create_list(values);
            Ok(factory.create_record_term(prototype, values))
        } else if let Some(term) = expression.as_constructor_term() {
            let term = term.as_inner();
            let keys = term
                .prototype()
                .as_deref()
                .keys()
                .as_deref()
                .iter()
                .map(|key| self.export(key.as_deref(), factory, allocator, indirect_call_arity))
                .collect::<Result<Vec<_>, _>>()?;
            let keys = allocator.create_list(keys);
            let prototype = allocator.create_struct_prototype(keys);
            Ok(factory.create_constructor_term(prototype))
        } else if let Some(term) = expression.as_list_term() {
            let items = term
                .items()
                .as_deref()
                .iter()
                .map(|key| self.export(key.as_deref(), factory, allocator, indirect_call_arity))
                .collect::<Result<Vec<_>, _>>()?;
            let items = allocator.create_list(items);
            Ok(factory.create_list_term(items))
        } else if let Some(term) = expression.as_hashmap_term() {
            let term = term.as_inner();
            let keys = term
                .keys()
                .map(|term| self.export(term.as_deref(), factory, allocator, indirect_call_arity));
            let values = term
                .values()
                .map(|term| self.export(term.as_deref(), factory, allocator, indirect_call_arity));
            let entries = keys
                .zip(values)
                .map(|(key, value)| {
                    let key = key?;
                    let value = value?;
                    Ok((key, value))
                })
                .collect::<Result<Vec<_>, WasmExpression<Rc<RefCell<A>>>>>()?;
            Ok(factory.create_hashmap_term(entries))
        } else if let Some(term) = expression.as_hashset_term() {
            let term = term.as_inner();
            let values = term
                .values()
                .map(|term| self.export(term.as_deref(), factory, allocator, indirect_call_arity))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(factory.create_hashset_term(values))
        } else if let Some(term) = expression.as_signal_term() {
            let term = term.as_inner();
            let conditions = term
                .signals()
                .as_deref()
                .iter()
                .map(|condition| {
                    self.export_condition(
                        condition.as_deref(),
                        factory,
                        allocator,
                        indirect_call_arity,
                    )
                })
                .collect::<Result<Vec<_>, WasmExpression<Rc<RefCell<A>>>>>()?;
            let conditions = allocator.create_signal_list(conditions);
            Ok(factory.create_signal_term(conditions))
        } else {
            Err(expression.clone())
        }
    }
    pub fn import_condition<T: Expression>(
        &self,
        condition: &T::Signal,
        factory: &impl ExpressionFactory<T>,
    ) -> Result<ArenaRef<TypedTerm<ConditionTerm>, Self>, T>
    where
        T::Builtin: Into<crate::stdlib::Stdlib>,
    {
        let signal_type = match condition.signal_type() {
            SignalType::Custom {
                effect_type,
                payload,
                token,
            } => {
                let effect_type = self.import(&effect_type, factory)?;
                let payload = self.import(&payload, factory)?;
                let token = self.import(&token, factory)?;
                Ok(SignalType::Custom {
                    effect_type,
                    payload,
                    token,
                })
            }
            SignalType::Pending => Ok(SignalType::Pending),
            SignalType::Error { payload } => {
                let payload = self.import(&payload, factory)?;
                Ok(SignalType::Error { payload })
            }
        }?;
        Ok(self.create_signal(signal_type))
    }
    pub fn export_condition<T: Expression>(
        &self,
        condition: &ArenaRef<TypedTerm<ConditionTerm>, Rc<RefCell<A>>>,
        factory: &impl ExpressionFactory<T>,
        allocator: &impl HeapAllocator<T>,
        indirect_call_arity: &HashMap<FunctionIndex, Arity>,
    ) -> Result<T::Signal, WasmExpression<Rc<RefCell<A>>>> {
        let signal_type = match condition.signal_type() {
            SignalType::Custom {
                effect_type,
                payload,
                token,
            } => {
                let effect_type =
                    self.export(&effect_type, factory, allocator, indirect_call_arity)?;
                let payload = self.export(&payload, factory, allocator, indirect_call_arity)?;
                let token = self.export(&token, factory, allocator, indirect_call_arity)?;
                SignalType::Custom {
                    effect_type,
                    payload,
                    token,
                }
            }
            SignalType::Pending => SignalType::Pending,
            SignalType::Error { payload } => {
                let is_unserialized_error_condition =
                    payload.as_pointer() == condition.as_term().as_pointer();
                if is_unserialized_error_condition {
                    SignalType::Error {
                        payload: factory
                            .create_string_term(allocator.create_string(format!("{condition}"))),
                    }
                } else {
                    let payload = self.export(&payload, factory, allocator, indirect_call_arity)?;
                    SignalType::Error { payload }
                }
            }
        };
        Ok(allocator.create_signal(signal_type))
    }
}

impl<A: Arena> Clone for WasmTermFactory<A> {
    fn clone(&self) -> Self {
        Self {
            arena: Rc::clone(&self.arena),
        }
    }
}

impl<A: Arena> Arena for WasmTermFactory<A>
where
    Rc<RefCell<A>>: Arena,
{
    type Slice<'a> = <Rc<RefCell<A>> as Arena>::Slice<'a>
    where
        Self: 'a;
    fn read_value<T, V>(&self, offset: ArenaPointer, selector: impl FnOnce(&T) -> V) -> V {
        self.arena.read_value::<T, V>(offset, selector)
    }
    fn inner_pointer<T, V>(
        &self,
        offset: ArenaPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> ArenaPointer {
        self.arena.inner_pointer::<T, V>(offset, selector)
    }
    fn as_slice<'a>(&'a self, offset: ArenaPointer, length: usize) -> Self::Slice<'a>
    where
        Self::Slice<'a>: 'a,
        Self: 'a,
    {
        self.arena.as_slice(offset, length)
    }
}

impl<A: Arena> ArenaAllocator for WasmTermFactory<A>
where
    A: ArenaAllocator,
    Rc<RefCell<A>>: Arena,
{
    fn allocate<T: TermSize>(&mut self, value: T) -> ArenaPointer {
        self.arena.borrow_mut().deref_mut().allocate(value)
    }
    fn extend(&mut self, offset: ArenaPointer, size: usize) {
        self.arena.borrow_mut().deref_mut().extend(offset, size)
    }
    fn shrink(&mut self, offset: ArenaPointer, size: usize) {
        self.arena.borrow_mut().deref_mut().shrink(offset, size)
    }
    fn write<T: Sized>(&mut self, offset: ArenaPointer, value: T) {
        self.arena.borrow_mut().deref_mut().write(offset, value)
    }
}

impl<A: Arena> HeapAllocator<ArenaRef<Term, Self>> for WasmTermFactory<A>
where
    A: ArenaAllocator,
    Rc<RefCell<A>>: Arena,
{
    fn create_list(
        &self,
        expressions: impl IntoIterator<
            Item = ArenaRef<Term, Self>,
            IntoIter = impl ExactSizeIterator<Item = ArenaRef<Term, Self>>,
        >,
    ) -> <ArenaRef<Term, Self> as Expression>::ExpressionList {
        let pointer = ListTerm::allocate(
            // TODO: Add debug assertions to check that WASM factory list items are from the correct arena
            // (this may require collecting the list items first to prevent runtime borrow errors)
            expressions.into_iter().map(|term| term.pointer),
            self.arena.deref().borrow_mut().deref_mut(),
        );
        ArenaRef::<TypedTerm<ListTerm>, Self>::new(self.clone(), pointer)
    }

    fn create_unsized_list(
        &self,
        expressions: impl IntoIterator<Item = ArenaRef<Term, Self>>,
    ) -> <ArenaRef<Term, Self> as Expression>::ExpressionList {
        self.create_list(expressions.into_iter().collect::<Vec<_>>())
    }

    fn create_sized_list(
        &self,
        size: usize,
        expressions: impl IntoIterator<Item = ArenaRef<Term, Self>>,
    ) -> <ArenaRef<Term, Self> as Expression>::ExpressionList {
        self.create_list(WithExactSizeIterator::new(size, expressions.into_iter()))
    }

    fn create_empty_list(&self) -> <ArenaRef<Term, Self> as Expression>::ExpressionList {
        self.create_list(empty())
    }

    fn create_unit_list(
        &self,
        value: ArenaRef<Term, Self>,
    ) -> <ArenaRef<Term, Self> as Expression>::ExpressionList {
        self.create_list(once(value))
    }

    fn create_pair(
        &self,
        left: ArenaRef<Term, Self>,
        right: ArenaRef<Term, Self>,
    ) -> <ArenaRef<Term, Self> as Expression>::ExpressionList {
        self.create_list([left, right])
    }

    fn create_triple(
        &self,
        first: ArenaRef<Term, Self>,
        second: ArenaRef<Term, Self>,
        third: ArenaRef<Term, Self>,
    ) -> <ArenaRef<Term, Self> as Expression>::ExpressionList {
        self.create_list([first, second, third])
    }

    fn clone_list<'a>(
        &self,
        expressions: <ArenaRef<Term, Self> as Expression>::ExpressionListRef<'a>,
    ) -> <ArenaRef<Term, Self> as Expression>::ExpressionList
    where
        Self: 'a,
    {
        expressions
    }

    fn create_signal_list(
        &self,
        signals: impl IntoIterator<Item = <ArenaRef<Term, Self> as Expression>::Signal>,
    ) -> <ArenaRef<Term, Self> as Expression>::SignalList {
        let mut children = signals.into_iter().map(|condition| {
            debug_assert!(std::ptr::eq(
                condition.arena.arena.deref().borrow().deref(),
                self.arena.deref().borrow().deref()
            ));
            condition.pointer
        });
        let first = children.next();
        let remaining = children;
        let root_size = first.as_ref().into_iter().count();
        let root_term = Term::new(
            TermType::Tree(TreeTerm {
                left: first.unwrap_or(ArenaPointer::null()),
                right: ArenaPointer::null(),
                length: root_size as u32,
            }),
            self.arena.deref().borrow().deref(),
        );
        let root_pointer = self
            .arena
            .deref()
            .borrow_mut()
            .deref_mut()
            .allocate(root_term);

        let pointer = remaining
            .enumerate()
            .fold(root_pointer, move |acc, (index, condition)| {
                let length = root_size + index + 1;
                let term = Term::new(
                    TermType::Tree(TreeTerm {
                        left: condition,
                        right: acc,
                        length: length as u32,
                    }),
                    self.arena.deref().borrow().deref(),
                );
                self.arena.deref().borrow_mut().deref_mut().allocate(term)
            });
        ArenaRef::<TypedTerm<TreeTerm>, Self>::new(self.clone(), pointer)
    }

    fn create_struct_prototype(
        &self,
        keys: <ArenaRef<Term, Self> as Expression>::ExpressionList,
    ) -> <ArenaRef<Term, Self> as Expression>::StructPrototype {
        keys
    }

    fn clone_struct_prototype<'a>(
        &self,
        prototype: <ArenaRef<Term, Self> as Expression>::StructPrototypeRef<'a>,
    ) -> <ArenaRef<Term, Self> as Expression>::StructPrototype
    where
        Self: 'a,
    {
        prototype
    }

    fn create_signal(
        &self,
        effect_type: SignalType<ArenaRef<Term, Self>>,
    ) -> <ArenaRef<Term, Self> as Expression>::Signal {
        let term = Term::new(
            TermType::Condition(match effect_type {
                SignalType::Error { payload } => {
                    debug_assert!(std::ptr::eq(
                        payload.arena.arena.deref().borrow().deref(),
                        self.arena.deref().borrow().deref(),
                    ));
                    ConditionTerm::Error(ErrorCondition {
                        payload: payload.pointer,
                    })
                }
                SignalType::Pending => ConditionTerm::Pending(PendingCondition),
                SignalType::Custom {
                    effect_type,
                    payload,
                    token,
                } => {
                    debug_assert!(
                        std::ptr::eq(
                            effect_type.arena.arena.deref().borrow().deref(),
                            self.arena.deref().borrow().deref(),
                        ) && std::ptr::eq(
                            payload.arena.arena.deref().borrow().deref(),
                            self.arena.deref().borrow().deref(),
                        ) && std::ptr::eq(
                            token.arena.arena.deref().borrow().deref(),
                            self.arena.deref().borrow().deref(),
                        )
                    );
                    ConditionTerm::Custom(CustomCondition {
                        effect_type: effect_type.pointer,
                        payload: payload.pointer,
                        token: token.pointer,
                    })
                }
            }),
            self.arena.deref().borrow().deref(),
        );
        let pointer = self.arena.deref().borrow_mut().deref_mut().allocate(term);
        ArenaRef::<TypedTerm<ConditionTerm>, Self>::new(self.clone(), pointer)
    }

    fn clone_signal<'a>(
        &self,
        signal: <ArenaRef<Term, Self> as Expression>::SignalRef<'a>,
    ) -> <ArenaRef<Term, Self> as Expression>::Signal
    where
        Self: 'a,
    {
        signal
    }

    fn create_string(
        &self,
        value: impl Into<String>,
    ) -> <ArenaRef<Term, Self> as Expression>::String {
        let pointer =
            StringTerm::allocate(&value.into(), self.arena.deref().borrow_mut().deref_mut());
        ArenaRef::<TypedTerm<StringTerm>, Self>::new(self.clone(), pointer)
    }

    fn create_static_string(
        &self,
        value: &'static str,
    ) -> <ArenaRef<Term, Self> as Expression>::String {
        self.create_string(value)
    }

    fn clone_string<'a>(
        &self,
        value: <ArenaRef<Term, Self> as Expression>::StringRef<'a>,
    ) -> <ArenaRef<Term, Self> as Expression>::String
    where
        Self: 'a,
    {
        value
    }
}

impl<A: Arena> ExpressionFactory<ArenaRef<Term, Self>> for WasmTermFactory<A>
where
    A: ArenaAllocator,
    Rc<RefCell<A>>: Arena,
{
    fn create_nil_term(&self) -> ArenaRef<Term, Self> {
        let term = Term::new(TermType::Nil(NilTerm), &*self.arena.borrow());
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_boolean_term(&self, value: bool) -> ArenaRef<Term, Self> {
        let term = Term::new(
            TermType::Boolean(BooleanTerm::from(value)),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_int_term(&self, value: IntValue) -> ArenaRef<Term, Self> {
        let term = Term::new(TermType::Int(IntTerm::from(value)), &*self.arena.borrow());
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_float_term(&self, value: FloatValue) -> ArenaRef<Term, Self> {
        let term = Term::new(
            TermType::Float(FloatTerm::from(value)),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_string_term(
        &self,
        value: <ArenaRef<Term, Self> as Expression>::String,
    ) -> ArenaRef<Term, Self> {
        debug_assert!(std::ptr::eq(
            &*value.arena.arena.borrow(),
            &*self.arena.borrow()
        ));
        value.as_term().clone()
    }

    fn create_symbol_term(&self, value: SymbolId) -> ArenaRef<Term, Self> {
        let term = Term::new(
            TermType::Symbol(SymbolTerm {
                // TODO: Change SymbolId type to u32
                id: (value & 0x00000000FFFFFFFF) as u32,
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_variable_term(&self, offset: StackOffset) -> ArenaRef<Term, Self> {
        let term = Term::new(
            TermType::Variable(VariableTerm {
                stack_offset: offset as u32,
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_effect_term(
        &self,
        condition: <ArenaRef<Term, Self> as Expression>::Signal,
    ) -> ArenaRef<Term, Self> {
        debug_assert!(std::ptr::eq(
            &*condition.arena.arena.borrow(),
            &*self.arena.borrow()
        ));
        let term = Term::new(
            TermType::Effect(EffectTerm {
                condition: condition.as_pointer(),
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_let_term(
        &self,
        initializer: ArenaRef<Term, Self>,
        body: ArenaRef<Term, Self>,
    ) -> ArenaRef<Term, Self> {
        debug_assert!(
            std::ptr::eq(&*initializer.arena.arena.borrow(), &*self.arena.borrow())
                && std::ptr::eq(&*body.arena.arena.borrow(), &*self.arena.borrow())
        );
        let term = Term::new(
            TermType::Let(LetTerm {
                initializer: initializer.as_pointer(),
                body: body.as_pointer(),
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_lambda_term(
        &self,
        num_args: StackOffset,
        body: ArenaRef<Term, Self>,
    ) -> ArenaRef<Term, Self> {
        debug_assert!(std::ptr::eq(
            &*body.arena.arena.borrow(),
            &*self.arena.borrow()
        ));
        let term = Term::new(
            TermType::Lambda(LambdaTerm {
                num_args: num_args as u32,
                body: body.as_pointer(),
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_application_term(
        &self,
        target: ArenaRef<Term, Self>,
        args: <ArenaRef<Term, Self> as Expression>::ExpressionList,
    ) -> ArenaRef<Term, Self> {
        debug_assert!(
            std::ptr::eq(&*target.arena.arena.borrow(), &*self.arena.borrow())
                && std::ptr::eq(&*args.arena.arena.borrow(), &*self.arena.borrow())
        );
        let term = Term::new(
            TermType::Application(ApplicationTerm {
                target: target.as_pointer(),
                args: args.as_pointer(),
                cache: Default::default(),
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_partial_application_term(
        &self,
        target: ArenaRef<Term, Self>,
        args: <ArenaRef<Term, Self> as Expression>::ExpressionList,
    ) -> ArenaRef<Term, Self> {
        debug_assert!(
            std::ptr::eq(&*target.arena.arena.borrow(), &*self.arena.borrow())
                && std::ptr::eq(&*args.arena.arena.borrow(), &*self.arena.borrow())
        );
        let term = Term::new(
            TermType::Partial(PartialTerm {
                target: target.as_pointer(),
                args: args.as_pointer(),
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_recursive_term(&self, factory: ArenaRef<Term, Self>) -> ArenaRef<Term, Self> {
        debug_assert!(std::ptr::eq(
            &*factory.arena.arena.borrow(),
            &*self.arena.borrow()
        ));
        // TODO: support recursive WASM terms
        let error_message = StringTerm::allocate(
            "Recursive terms not currently supported",
            &mut *self.arena.borrow_mut(),
        );
        let condition_term = Term::new(
            TermType::Condition(ConditionTerm::Error(ErrorCondition {
                payload: error_message,
            })),
            &*self.arena.borrow(),
        );
        let condition_pointer = self.arena.borrow_mut().deref_mut().allocate(condition_term);
        let signal_list_term = Term::new(
            TermType::Tree(TreeTerm {
                left: condition_pointer,
                right: ArenaPointer::null(),
                length: 1,
            }),
            &*self.arena.borrow(),
        );
        let signal_list_pointer = self
            .arena
            .borrow_mut()
            .deref_mut()
            .allocate(signal_list_term);
        let signal_list =
            ArenaRef::<TypedTerm<TreeTerm>, _>::new(self.clone(), signal_list_pointer);
        self.create_signal_term(signal_list)
    }

    fn create_builtin_term(
        &self,
        target: impl Into<<ArenaRef<Term, Self> as Expression>::Builtin>,
    ) -> ArenaRef<Term, Self> {
        let target = target.into();
        let term = Term::new(
            TermType::Builtin(BuiltinTerm::from(target)),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    // TODO: Remove compiled function term type
    fn create_compiled_function_term(
        &self,
        address: InstructionPointer,
        _hash: HashId,
        _required_args: StackOffset,
        _optional_args: StackOffset,
        _variadic_args: bool,
    ) -> ArenaRef<Term, Self> {
        let index = address.get();
        if index < 0x000000000000FFFF {
            let target = FunctionIndex::from(index as u32);
            let term = Term::new(
                TermType::Builtin(BuiltinTerm {
                    uid: u32::from(target),
                }),
                &*self.arena.borrow(),
            );
            let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
            ArenaRef::<Term, Self>::new(self.clone(), pointer)
        } else {
            let message = self.create_string(format!("Invalid WASM function index: {}", index));
            let signal = self.create_signal(SignalType::Error {
                payload: self.create_string_term(message),
            });
            let signal_list = self.create_signal_list([signal]);
            self.create_signal_term(signal_list)
        }
    }

    fn create_record_term(
        &self,
        prototype: <ArenaRef<Term, Self> as Expression>::StructPrototype,
        fields: <ArenaRef<Term, Self> as Expression>::ExpressionList,
    ) -> ArenaRef<Term, Self> {
        debug_assert!(
            std::ptr::eq(&*prototype.arena.arena.borrow(), &*self.arena.borrow())
                && std::ptr::eq(&*fields.arena.arena.borrow(), &*self.arena.borrow())
        );
        let keys = prototype.keys();
        let keys = keys.as_deref();
        let lookup_table = if keys.len() >= 16 {
            let entries = keys.iter().zip(fields.iter()).collect::<Vec<_>>();
            Some(self.create_hashmap_term(entries))
        } else {
            None
        };
        let keys = prototype.as_pointer();
        let values = fields.as_pointer();
        let term = Term::new(
            TermType::Record(RecordTerm {
                keys,
                values,
                lookup_table: match lookup_table {
                    Some(term) => term.pointer,
                    None => ArenaPointer::null(),
                },
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_constructor_term(
        &self,
        prototype: <ArenaRef<Term, Self> as Expression>::StructPrototype,
    ) -> ArenaRef<Term, Self> {
        debug_assert!(std::ptr::eq(
            &*prototype.arena.arena.borrow(),
            &*self.arena.borrow()
        ));
        let term = Term::new(
            TermType::Constructor(ConstructorTerm {
                keys: prototype.as_pointer(),
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_list_term(
        &self,
        items: <ArenaRef<Term, Self> as Expression>::ExpressionList,
    ) -> ArenaRef<Term, Self> {
        debug_assert!(std::ptr::eq(
            &*items.arena.arena.borrow(),
            &*self.arena.borrow()
        ));
        items.as_term().clone()
    }

    fn create_hashmap_term(
        &self,
        entries: impl IntoIterator<
            Item = (ArenaRef<Term, Self>, ArenaRef<Term, Self>),
            IntoIter = impl ExactSizeIterator<Item = (ArenaRef<Term, Self>, ArenaRef<Term, Self>)>,
        >,
    ) -> ArenaRef<Term, Self> {
        let entries = entries
            .into_iter()
            .map(|(key, value)| (key.as_pointer(), value.as_pointer()))
            .collect::<Vec<_>>();
        let pointer = HashmapTerm::allocate(entries, &mut *self.arena.borrow_mut());
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_hashset_term(
        &self,
        values: impl IntoIterator<
            Item = ArenaRef<Term, Self>,
            IntoIter = impl ExactSizeIterator<Item = ArenaRef<Term, Self>>,
        >,
    ) -> ArenaRef<Term, Self> {
        let nil = self.create_nil_term();
        let entries = values
            .into_iter()
            .map(|value| (value, nil.clone()))
            .collect::<Vec<_>>();
        let entries = self.create_hashmap_term(entries);
        let term = Term::new(
            TermType::Hashset(HashsetTerm {
                entries: entries.as_pointer(),
            }),
            &self.arena,
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_signal_term(
        &self,
        signals: <ArenaRef<Term, Self> as Expression>::SignalList,
    ) -> ArenaRef<Term, Self> {
        debug_assert!(std::ptr::eq(
            &*signals.arena.arena.borrow(),
            &*self.arena.borrow()
        ));
        let term = Term::new(
            TermType::Signal(SignalTerm {
                conditions: signals.as_pointer(),
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().deref_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn match_nil_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::NilTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Nil => Some(expression.as_typed_term::<NilTerm>()),
            _ => None,
        }
    }

    fn match_boolean_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::BooleanTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Boolean => Some(expression.as_typed_term::<BooleanTerm>()),
            _ => None,
        }
    }

    fn match_int_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::IntTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Int => Some(expression.as_typed_term::<IntTerm>()),
            _ => None,
        }
    }

    fn match_float_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::FloatTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Float => Some(expression.as_typed_term::<FloatTerm>()),
            _ => None,
        }
    }

    fn match_string_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::StringTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::String => Some(expression.as_typed_term::<StringTerm>()),
            _ => None,
        }
    }

    fn match_symbol_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::SymbolTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Symbol => Some(expression.as_typed_term::<SymbolTerm>()),
            _ => None,
        }
    }

    fn match_variable_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::VariableTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Variable => Some(expression.as_typed_term::<VariableTerm>()),
            _ => None,
        }
    }

    fn match_effect_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::EffectTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Effect => Some(expression.as_typed_term::<EffectTerm>()),
            _ => None,
        }
    }

    fn match_let_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::LetTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Let => Some(expression.as_typed_term::<LetTerm>()),
            _ => None,
        }
    }

    fn match_lambda_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::LambdaTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Lambda => Some(expression.as_typed_term::<LambdaTerm>()),
            _ => None,
        }
    }

    fn match_application_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::ApplicationTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Application => {
                Some(expression.as_typed_term::<ApplicationTerm>())
            }
            _ => None,
        }
    }

    fn match_partial_application_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::PartialApplicationTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Partial => Some(expression.as_typed_term::<PartialTerm>()),
            _ => None,
        }
    }

    fn match_recursive_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::RecursiveTerm> {
        match expression.read_value(|term| term.type_id()) {
            //Discriminants TODO: Implement WASM recursive term
            _ => None,
        }
    }

    fn match_builtin_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::BuiltinTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Builtin => Some(expression.as_typed_term::<BuiltinTerm>()),
            _ => None,
        }
    }

    fn match_compiled_function_term<'a>(
        &self,
        _expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::CompiledFunctionTerm> {
        None
    }

    fn match_record_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::RecordTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Record => Some(expression.as_typed_term::<RecordTerm>()),
            _ => None,
        }
    }

    fn match_constructor_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::ConstructorTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Constructor => {
                Some(expression.as_typed_term::<ConstructorTerm>())
            }
            _ => None,
        }
    }

    fn match_list_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::ListTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::List => Some(expression.as_typed_term::<ListTerm>()),
            _ => None,
        }
    }

    fn match_hashmap_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::HashmapTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Hashmap => Some(expression.as_typed_term::<HashmapTerm>()),
            _ => None,
        }
    }

    fn match_hashset_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::HashsetTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Hashset => Some(expression.as_typed_term::<HashsetTerm>()),
            _ => None,
        }
    }

    fn match_signal_term<'a>(
        &self,
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::SignalTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Signal => Some(expression.as_typed_term::<SignalTerm>()),
            _ => None,
        }
    }
}

impl From<reflex_stdlib::stdlib::Stdlib> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Stdlib) -> Self {
        match value {
            reflex_stdlib::stdlib::Stdlib::Abs => {
                reflex_wasm::stdlib::Stdlib::Abs(reflex_wasm::stdlib::Abs)
            }
            reflex_stdlib::stdlib::Stdlib::Add => {
                reflex_wasm::stdlib::Stdlib::Add(reflex_wasm::stdlib::Add)
            }
            reflex_stdlib::stdlib::Stdlib::And => {
                reflex_wasm::stdlib::Stdlib::And(reflex_wasm::stdlib::And)
            }
            reflex_stdlib::stdlib::Stdlib::Apply => {
                reflex_wasm::stdlib::Stdlib::Apply(reflex_wasm::stdlib::Apply)
            }
            reflex_stdlib::stdlib::Stdlib::Car => {
                reflex_wasm::stdlib::Stdlib::Car(reflex_wasm::stdlib::Car)
            }
            reflex_stdlib::stdlib::Stdlib::Cdr => {
                reflex_wasm::stdlib::Stdlib::Cdr(reflex_wasm::stdlib::Cdr)
            }
            reflex_stdlib::stdlib::Stdlib::Ceil => {
                reflex_wasm::stdlib::Stdlib::Ceil(reflex_wasm::stdlib::Ceil)
            }
            reflex_stdlib::stdlib::Stdlib::Chain => {
                reflex_wasm::stdlib::Stdlib::Chain(reflex_wasm::stdlib::Chain)
            }
            reflex_stdlib::stdlib::Stdlib::CollectHashMap => {
                reflex_wasm::stdlib::Stdlib::CollectHashmap(reflex_wasm::stdlib::CollectHashmap)
            }
            reflex_stdlib::stdlib::Stdlib::CollectHashSet => {
                reflex_wasm::stdlib::Stdlib::CollectHashset(reflex_wasm::stdlib::CollectHashset)
            }
            reflex_stdlib::stdlib::Stdlib::CollectList => {
                reflex_wasm::stdlib::Stdlib::CollectList(reflex_wasm::stdlib::CollectList)
            }
            reflex_stdlib::stdlib::Stdlib::CollectSignal => {
                reflex_wasm::stdlib::Stdlib::CollectSignal(reflex_wasm::stdlib::CollectSignal)
            }
            reflex_stdlib::stdlib::Stdlib::Concat => {
                reflex_wasm::stdlib::Stdlib::CollectString(reflex_wasm::stdlib::CollectString)
            }
            reflex_stdlib::stdlib::Stdlib::Cons => {
                reflex_wasm::stdlib::Stdlib::Cons(reflex_wasm::stdlib::Cons)
            }
            reflex_stdlib::stdlib::Stdlib::ConstructHashMap => {
                reflex_wasm::stdlib::Stdlib::ConstructHashmap(reflex_wasm::stdlib::ConstructHashmap)
            }
            reflex_stdlib::stdlib::Stdlib::ConstructHashSet => {
                reflex_wasm::stdlib::Stdlib::ConstructHashset(reflex_wasm::stdlib::ConstructHashset)
            }
            reflex_stdlib::stdlib::Stdlib::ConstructRecord => {
                reflex_wasm::stdlib::Stdlib::ConstructRecord(reflex_wasm::stdlib::ConstructRecord)
            }
            reflex_stdlib::stdlib::Stdlib::ConstructList => {
                reflex_wasm::stdlib::Stdlib::ConstructList(reflex_wasm::stdlib::ConstructList)
            }
            reflex_stdlib::stdlib::Stdlib::Contains => {
                reflex_wasm::stdlib::Stdlib::Has(reflex_wasm::stdlib::Has)
            }
            reflex_stdlib::stdlib::Stdlib::Divide => {
                reflex_wasm::stdlib::Stdlib::Divide(reflex_wasm::stdlib::Divide)
            }
            reflex_stdlib::stdlib::Stdlib::Effect => {
                reflex_wasm::stdlib::Stdlib::Effect(reflex_wasm::stdlib::Effect)
            }
            reflex_stdlib::stdlib::Stdlib::EndsWith => {
                reflex_wasm::stdlib::Stdlib::EndsWith(reflex_wasm::stdlib::EndsWith)
            }
            reflex_stdlib::stdlib::Stdlib::Eq => {
                reflex_wasm::stdlib::Stdlib::Eq(reflex_wasm::stdlib::Eq)
            }
            reflex_stdlib::stdlib::Stdlib::Equal => {
                reflex_wasm::stdlib::Stdlib::Equal(reflex_wasm::stdlib::Equal)
            }
            reflex_stdlib::stdlib::Stdlib::Filter => {
                reflex_wasm::stdlib::Stdlib::Filter(reflex_wasm::stdlib::Filter)
            }
            reflex_stdlib::stdlib::Stdlib::Flatten => {
                reflex_wasm::stdlib::Stdlib::Flatten(reflex_wasm::stdlib::Flatten)
            }
            reflex_stdlib::stdlib::Stdlib::Floor => {
                reflex_wasm::stdlib::Stdlib::Floor(reflex_wasm::stdlib::Floor)
            }
            reflex_stdlib::stdlib::Stdlib::Get => {
                reflex_wasm::stdlib::Stdlib::Get(reflex_wasm::stdlib::Get)
            }
            reflex_stdlib::stdlib::Stdlib::Gt => {
                reflex_wasm::stdlib::Stdlib::Gt(reflex_wasm::stdlib::Gt)
            }
            reflex_stdlib::stdlib::Stdlib::Gte => {
                reflex_wasm::stdlib::Stdlib::Gte(reflex_wasm::stdlib::Gte)
            }
            reflex_stdlib::stdlib::Stdlib::Hash => {
                reflex_wasm::stdlib::Stdlib::Hash(reflex_wasm::stdlib::Hash)
            }
            reflex_stdlib::stdlib::Stdlib::If => {
                reflex_wasm::stdlib::Stdlib::If(reflex_wasm::stdlib::If)
            }
            reflex_stdlib::stdlib::Stdlib::IfError => {
                reflex_wasm::stdlib::Stdlib::IfError(reflex_wasm::stdlib::IfError)
            }
            reflex_stdlib::stdlib::Stdlib::IfPending => {
                reflex_wasm::stdlib::Stdlib::IfPending(reflex_wasm::stdlib::IfPending)
            }
            reflex_stdlib::stdlib::Stdlib::Insert => {
                reflex_wasm::stdlib::Stdlib::Set(reflex_wasm::stdlib::Set)
            }
            reflex_stdlib::stdlib::Stdlib::Keys => {
                reflex_wasm::stdlib::Stdlib::Keys(reflex_wasm::stdlib::Keys)
            }
            reflex_stdlib::stdlib::Stdlib::Length => {
                reflex_wasm::stdlib::Stdlib::Length(reflex_wasm::stdlib::Length)
            }
            reflex_stdlib::stdlib::Stdlib::Lt => {
                reflex_wasm::stdlib::Stdlib::Lt(reflex_wasm::stdlib::Lt)
            }
            reflex_stdlib::stdlib::Stdlib::Lte => {
                reflex_wasm::stdlib::Stdlib::Lte(reflex_wasm::stdlib::Lte)
            }
            reflex_stdlib::stdlib::Stdlib::Map => {
                reflex_wasm::stdlib::Stdlib::Map(reflex_wasm::stdlib::Map)
            }
            reflex_stdlib::stdlib::Stdlib::Max => {
                reflex_wasm::stdlib::Stdlib::Max(reflex_wasm::stdlib::Max)
            }
            reflex_stdlib::stdlib::Stdlib::Merge => {
                reflex_wasm::stdlib::Stdlib::Merge(reflex_wasm::stdlib::Merge)
            }
            reflex_stdlib::stdlib::Stdlib::Min => {
                reflex_wasm::stdlib::Stdlib::Min(reflex_wasm::stdlib::Min)
            }
            reflex_stdlib::stdlib::Stdlib::Multiply => {
                reflex_wasm::stdlib::Stdlib::Multiply(reflex_wasm::stdlib::Multiply)
            }
            reflex_stdlib::stdlib::Stdlib::Not => {
                reflex_wasm::stdlib::Stdlib::Not(reflex_wasm::stdlib::Not)
            }
            reflex_stdlib::stdlib::Stdlib::Or => {
                reflex_wasm::stdlib::Stdlib::Or(reflex_wasm::stdlib::Or)
            }
            reflex_stdlib::stdlib::Stdlib::Pow => {
                reflex_wasm::stdlib::Stdlib::Pow(reflex_wasm::stdlib::Pow)
            }
            reflex_stdlib::stdlib::Stdlib::Push => {
                reflex_wasm::stdlib::Stdlib::Push(reflex_wasm::stdlib::Push)
            }
            reflex_stdlib::stdlib::Stdlib::PushFront => {
                reflex_wasm::stdlib::Stdlib::PushFront(reflex_wasm::stdlib::PushFront)
            }
            reflex_stdlib::stdlib::Stdlib::Raise => {
                reflex_wasm::stdlib::Stdlib::Raise(reflex_wasm::stdlib::Raise)
            }
            reflex_stdlib::stdlib::Stdlib::Reduce => {
                reflex_wasm::stdlib::Stdlib::Fold(reflex_wasm::stdlib::Fold)
            }
            reflex_stdlib::stdlib::Stdlib::Remainder => {
                reflex_wasm::stdlib::Stdlib::Remainder(reflex_wasm::stdlib::Remainder)
            }
            reflex_stdlib::stdlib::Stdlib::Replace => {
                reflex_wasm::stdlib::Stdlib::Replace(reflex_wasm::stdlib::Replace)
            }
            reflex_stdlib::stdlib::Stdlib::ResolveArgs => {
                reflex_wasm::stdlib::Stdlib::ResolveArgs(reflex_wasm::stdlib::ResolveArgs)
            }
            reflex_stdlib::stdlib::Stdlib::ResolveDeep => {
                reflex_wasm::stdlib::Stdlib::ResolveDeep(reflex_wasm::stdlib::ResolveDeep)
            }
            reflex_stdlib::stdlib::Stdlib::ResolveHashMap => {
                reflex_wasm::stdlib::Stdlib::ResolveHashmap(reflex_wasm::stdlib::ResolveHashmap)
            }
            reflex_stdlib::stdlib::Stdlib::ResolveHashSet => {
                reflex_wasm::stdlib::Stdlib::ResolveHashset(reflex_wasm::stdlib::ResolveHashset)
            }
            reflex_stdlib::stdlib::Stdlib::ResolveShallow => {
                reflex_wasm::stdlib::Stdlib::ResolveShallow(reflex_wasm::stdlib::ResolveShallow)
            }
            reflex_stdlib::stdlib::Stdlib::ResolveRecord => {
                reflex_wasm::stdlib::Stdlib::ResolveRecord(reflex_wasm::stdlib::ResolveRecord)
            }
            reflex_stdlib::stdlib::Stdlib::ResolveList => {
                reflex_wasm::stdlib::Stdlib::ResolveList(reflex_wasm::stdlib::ResolveList)
            }
            reflex_stdlib::stdlib::Stdlib::Round => {
                reflex_wasm::stdlib::Stdlib::Round(reflex_wasm::stdlib::Round)
            }
            reflex_stdlib::stdlib::Stdlib::Sequence => {
                reflex_wasm::stdlib::Stdlib::Sequence(reflex_wasm::stdlib::Sequence)
            }
            reflex_stdlib::stdlib::Stdlib::Slice => {
                reflex_wasm::stdlib::Stdlib::Slice(reflex_wasm::stdlib::Slice)
            }
            reflex_stdlib::stdlib::Stdlib::Split => {
                reflex_wasm::stdlib::Stdlib::Split(reflex_wasm::stdlib::Split)
            }
            reflex_stdlib::stdlib::Stdlib::StartsWith => {
                reflex_wasm::stdlib::Stdlib::StartsWith(reflex_wasm::stdlib::StartsWith)
            }
            reflex_stdlib::stdlib::Stdlib::Subtract => {
                reflex_wasm::stdlib::Stdlib::Subtract(reflex_wasm::stdlib::Subtract)
            }
            reflex_stdlib::stdlib::Stdlib::Unzip => {
                reflex_wasm::stdlib::Stdlib::Unzip(reflex_wasm::stdlib::Unzip)
            }
            reflex_stdlib::stdlib::Stdlib::Values => {
                reflex_wasm::stdlib::Stdlib::Values(reflex_wasm::stdlib::Values)
            }
            reflex_stdlib::stdlib::Stdlib::Zip => {
                reflex_wasm::stdlib::Stdlib::Zip(reflex_wasm::stdlib::Zip)
            }
        }
    }
}

impl From<reflex_json::stdlib::Stdlib> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_json::stdlib::Stdlib) -> Self {
        match value {
            reflex_json::stdlib::Stdlib::JsonDeserialize => {
                reflex_wasm::stdlib::Stdlib::ParseJson(reflex_wasm::stdlib::ParseJson)
            }
            reflex_json::stdlib::Stdlib::JsonSerialize => {
                reflex_wasm::stdlib::Stdlib::StringifyJson(reflex_wasm::stdlib::StringifyJson)
            }
        }
    }
}

impl From<reflex_js::stdlib::Stdlib> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::Stdlib) -> Self {
        match value {
            reflex_js::stdlib::Stdlib::Accessor => {
                reflex_wasm::stdlib::Stdlib::Accessor(reflex_wasm::stdlib::Accessor)
            }
            reflex_js::stdlib::Stdlib::Construct => {
                reflex_wasm::stdlib::Stdlib::Construct(reflex_wasm::stdlib::Construct)
            }
            reflex_js::stdlib::Stdlib::ParseDate => {
                reflex_wasm::stdlib::Stdlib::ParseDate(reflex_wasm::stdlib::ParseDate)
            }
            reflex_js::stdlib::Stdlib::EncodeUriComponent => {
                reflex_wasm::stdlib::Stdlib::Urlencode(reflex_wasm::stdlib::Urlencode)
            }
            reflex_js::stdlib::Stdlib::FormatErrorMessage => {
                reflex_wasm::stdlib::Stdlib::FormatErrorMessage(
                    reflex_wasm::stdlib::FormatErrorMessage,
                )
            }
            reflex_js::stdlib::Stdlib::IsFinite => {
                reflex_wasm::stdlib::Stdlib::IsFinite(reflex_wasm::stdlib::IsFinite)
            }
            reflex_js::stdlib::Stdlib::Log => {
                reflex_wasm::stdlib::Stdlib::Log(reflex_wasm::stdlib::Log)
            }
            reflex_js::stdlib::Stdlib::LogArgs => {
                reflex_wasm::stdlib::Stdlib::Log(reflex_wasm::stdlib::Log)
            }
            reflex_js::stdlib::Stdlib::ParseFloat => {
                reflex_wasm::stdlib::Stdlib::ParseFloat(reflex_wasm::stdlib::ParseFloat)
            }
            reflex_js::stdlib::Stdlib::ParseInt => {
                reflex_wasm::stdlib::Stdlib::ParseInt(reflex_wasm::stdlib::ParseInt)
            }
            reflex_js::stdlib::Stdlib::Throw => {
                reflex_wasm::stdlib::Stdlib::Throw(reflex_wasm::stdlib::Throw)
            }
            reflex_js::stdlib::Stdlib::ToString => {
                reflex_wasm::stdlib::Stdlib::ToString(reflex_wasm::stdlib::ToString)
            }
        }
    }
}

impl From<reflex_graphql::stdlib::Stdlib> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_graphql::stdlib::Stdlib) -> Self {
        match value {
            reflex_graphql::stdlib::Stdlib::CollectQueryListItems => {
                reflex_wasm::stdlib::Stdlib::CollectList(reflex_wasm::stdlib::CollectList)
            }
            reflex_graphql::stdlib::Stdlib::DynamicQueryBranch => {
                reflex_wasm::stdlib::Stdlib::ResolveQueryBranch(
                    reflex_wasm::stdlib::ResolveQueryBranch,
                )
            }
            reflex_graphql::stdlib::Stdlib::FlattenDeep => {
                reflex_wasm::stdlib::Stdlib::ResolveQueryLeaf(reflex_wasm::stdlib::ResolveQueryLeaf)
            }
            reflex_graphql::stdlib::Stdlib::GraphQlResolver => {
                reflex_wasm::stdlib::Stdlib::GraphQlResolver(reflex_wasm::stdlib::GraphQlResolver)
            }
        }
    }
}

impl From<reflex_handlers::stdlib::Stdlib> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_handlers::stdlib::Stdlib) -> Self {
        match value {
            reflex_handlers::stdlib::Stdlib::Scan => {
                reflex_wasm::stdlib::Stdlib::Scan(reflex_wasm::stdlib::Scan)
            }
            reflex_handlers::stdlib::Stdlib::ToRequest => {
                reflex_wasm::stdlib::Stdlib::ToRequest(reflex_wasm::stdlib::ToRequest)
            }
            reflex_handlers::stdlib::Stdlib::GetVariable => {
                reflex_wasm::stdlib::Stdlib::GetVariable(reflex_wasm::stdlib::GetVariable)
            }
            reflex_handlers::stdlib::Stdlib::SetVariable => {
                reflex_wasm::stdlib::Stdlib::SetVariable(reflex_wasm::stdlib::SetVariable)
            }
            reflex_handlers::stdlib::Stdlib::IncrementVariable => {
                reflex_wasm::stdlib::Stdlib::IncrementVariable(
                    reflex_wasm::stdlib::IncrementVariable,
                )
            }
            reflex_handlers::stdlib::Stdlib::DecrementVariable => {
                reflex_wasm::stdlib::Stdlib::DecrementVariable(
                    reflex_wasm::stdlib::DecrementVariable,
                )
            }
        }
    }
}

impl From<reflex_stdlib::stdlib::Abs> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Abs) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Add> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Add) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::And> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::And) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Apply> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Apply) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Car> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Car) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Cdr> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Cdr) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Ceil> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Ceil) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Chain> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Chain) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::CollectHashMap> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::CollectHashMap) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::CollectHashSet> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::CollectHashSet) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::CollectList> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::CollectList) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::CollectSignal> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::CollectSignal) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Concat> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Concat) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Cons> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Cons) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::ConstructHashMap> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::ConstructHashMap) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::ConstructHashSet> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::ConstructHashSet) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::ConstructRecord> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::ConstructRecord) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::ConstructList> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::ConstructList) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Contains> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Contains) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Divide> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Divide) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Effect> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Effect) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::EndsWith> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::EndsWith) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Eq> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Eq) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Equal> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Equal) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Filter> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Filter) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Flatten> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Flatten) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Floor> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Floor) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Get> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Get) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Gt> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Gt) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Gte> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Gte) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Hash> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Hash) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::If> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::If) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::IfError> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::IfError) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::IfPending> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::IfPending) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Insert> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Insert) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Keys> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Keys) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Length> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Length) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Lt> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Lt) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Lte> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Lte) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Map> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Map) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Max> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Max) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Merge> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Merge) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Min> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Min) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Multiply> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Multiply) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Not> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Not) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Or> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Or) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Pow> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Pow) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Push> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Push) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::PushFront> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::PushFront) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Raise> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Raise) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Reduce> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Reduce) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Remainder> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Remainder) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Replace> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Replace) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::ResolveArgs> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::ResolveArgs) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::ResolveDeep> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::ResolveDeep) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::ResolveHashMap> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::ResolveHashMap) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::ResolveHashSet> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::ResolveHashSet) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::ResolveShallow> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::ResolveShallow) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::ResolveRecord> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::ResolveRecord) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::ResolveList> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::ResolveList) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Round> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Round) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Sequence> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Sequence) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Slice> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Slice) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Split> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Split) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::StartsWith> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::StartsWith) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Subtract> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Subtract) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Unzip> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Unzip) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Values> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Values) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_stdlib::stdlib::Zip> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_stdlib::stdlib::Zip) -> Self {
        Self::from(reflex_stdlib::stdlib::Stdlib::from(value))
    }
}

impl From<reflex_json::stdlib::JsonDeserialize> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_json::stdlib::JsonDeserialize) -> Self {
        Self::from(reflex_json::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_json::stdlib::JsonSerialize> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_json::stdlib::JsonSerialize) -> Self {
        Self::from(reflex_json::stdlib::Stdlib::from(value))
    }
}

impl From<reflex_js::stdlib::Accessor> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::Accessor) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::Construct> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::Construct) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::EncodeUriComponent> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::EncodeUriComponent) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::FormatErrorMessage> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::FormatErrorMessage) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::IsFinite> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::IsFinite) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::Log> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::Log) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::LogArgs> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::LogArgs) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::ParseDate> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::ParseDate) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::ParseFloat> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::ParseFloat) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::ParseInt> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::ParseInt) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::Throw> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::Throw) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_js::stdlib::ToString> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_js::stdlib::ToString) -> Self {
        Self::from(reflex_js::stdlib::Stdlib::from(value))
    }
}

impl From<reflex_graphql::stdlib::CollectQueryListItems> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_graphql::stdlib::CollectQueryListItems) -> Self {
        Self::from(reflex_graphql::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_graphql::stdlib::DynamicQueryBranch> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_graphql::stdlib::DynamicQueryBranch) -> Self {
        Self::from(reflex_graphql::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_graphql::stdlib::FlattenDeep> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_graphql::stdlib::FlattenDeep) -> Self {
        Self::from(reflex_graphql::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_graphql::stdlib::GraphQlResolver> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_graphql::stdlib::GraphQlResolver) -> Self {
        Self::from(reflex_graphql::stdlib::Stdlib::from(value))
    }
}

impl From<reflex_handlers::stdlib::Scan> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_handlers::stdlib::Scan) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_handlers::stdlib::ToRequest> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_handlers::stdlib::ToRequest) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_handlers::stdlib::GetVariable> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_handlers::stdlib::GetVariable) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_handlers::stdlib::SetVariable> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_handlers::stdlib::SetVariable) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_handlers::stdlib::IncrementVariable> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_handlers::stdlib::IncrementVariable) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
impl From<reflex_handlers::stdlib::DecrementVariable> for reflex_wasm::stdlib::Stdlib {
    fn from(value: reflex_handlers::stdlib::DecrementVariable) -> Self {
        Self::from(reflex_handlers::stdlib::Stdlib::from(value))
    }
}
