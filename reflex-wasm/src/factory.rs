// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    cell::{Ref, RefCell},
    iter::{empty, once},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use reflex::{
    core::{
        ApplicationTermType, BooleanTermType, BuiltinTermType, CompiledFunctionTermType,
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
    allocator::{Arena, ArenaAllocator},
    hash::TermSize,
    stdlib::Stdlib,
    term_type::{
        ApplicationTerm, BooleanTerm, BuiltinTerm, ConditionTerm, ConstructorTerm, CustomCondition,
        EffectTerm, ErrorCondition, FloatTerm, HashmapTerm, HashsetTerm, IntTerm, LambdaTerm,
        LetTerm, ListTerm, NilTerm, PartialTerm, PendingCondition, RecordTerm, SignalTerm,
        StringTerm, SymbolTerm, TermType, TermTypeDiscriminants, TreeTerm, TypedTerm, VariableTerm,
        WasmExpression,
    },
    ArenaPointer, ArenaRef, Term,
};

#[derive(Debug)]
pub struct WasmTermFactory<A: Arena> {
    arena: Rc<RefCell<A>>,
}
impl<A: for<'a> ArenaAllocator<Slice<'a> = &'a [u8]> + 'static + Clone> WasmTermFactory<A> {
    pub fn import<T: Expression>(
        &self,
        expression: &T,
        factory: &impl ExpressionFactory<T>,
    ) -> Result<WasmExpression<Self>, T>
    where
        T::Builtin: Into<Stdlib>,
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
            let condition = {
                let condition = term.condition();
                let condition = condition.as_deref();
                let signal_type = condition.signal_type();
                let payload = self.import(condition.payload().as_deref(), factory)?;
                let token = self.import(condition.token().as_deref(), factory)?;
                self.create_signal(signal_type, payload, token)
            };
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
            let target: Stdlib = term.target().into();
            Ok(self.create_builtin_term(target))
        } else if let Some(term) = factory.match_compiled_function_term(expression) {
            let term = term.as_deref();
            let address = term.address();
            let hash = term.hash();
            let required_args = term.required_args();
            let optional_args = term.optional_args();
            Ok(self.create_compiled_function_term(address, hash, required_args, optional_args))
        } else if let Some(term) = factory.match_record_term(expression) {
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
            let values = term
                .values()
                .as_deref()
                .iter()
                .map(|key| self.import(key.as_deref(), factory))
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
                .map(|condition| {
                    let condition = condition.as_deref();
                    let effect_type = condition.signal_type();
                    let payload = self.import(condition.payload().as_deref(), factory)?;
                    let token = self.import(condition.token().as_deref(), factory)?;
                    Ok(self.create_signal(effect_type, payload, token))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let conditions = self.create_signal_list(conditions);
            Ok(self.create_signal_term(conditions))
        } else {
            Err(expression.clone())
        }
    }
}
impl<A: Arena> Clone for WasmTermFactory<A> {
    fn clone(&self) -> Self {
        Self {
            arena: Rc::clone(&self.arena),
        }
    }
}

impl<A: for<'a> Arena<Slice<'a> = &'a [u8]> + 'static> Arena for WasmTermFactory<A> {
    type Slice<'a> = Ref<'a, [u8]>
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
    {
        self.arena.as_slice(offset, length)
    }
}

impl<A: for<'a> ArenaAllocator<Slice<'a> = &'a [u8]> + 'static> ArenaAllocator
    for WasmTermFactory<A>
{
    fn allocate<T: TermSize>(&mut self, value: T) -> ArenaPointer {
        self.arena.allocate(value)
    }
    fn extend(&mut self, offset: ArenaPointer, size: usize) {
        self.arena.extend(offset, size)
    }
    fn shrink(&mut self, offset: ArenaPointer, size: usize) {
        self.arena.shrink(offset, size)
    }
    fn write<T: Sized>(&mut self, offset: ArenaPointer, value: T) {
        self.arena.write(offset, value)
    }
}

impl<A: for<'a> ArenaAllocator<Slice<'a> = &'a [u8]> + 'static + Clone>
    HeapAllocator<ArenaRef<Term, Self>> for WasmTermFactory<A>
{
    fn create_list(
        &self,
        expressions: impl IntoIterator<
            Item = ArenaRef<Term, Self>,
            IntoIter = impl ExactSizeIterator<Item = ArenaRef<Term, Self>>,
        >,
    ) -> <ArenaRef<Term, Self> as Expression>::ExpressionList {
        let pointer = ListTerm::allocate(
            expressions.into_iter().map(|term| {
                debug_assert!(std::ptr::eq(
                    term.arena.arena.deref().borrow().deref(),
                    self.arena.deref().borrow().deref()
                ));
                term.pointer
            }),
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
    ) -> <ArenaRef<Term, Self> as Expression>::ExpressionList {
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
        let second = children.next();
        let remaining = children;

        let root_size = first.as_ref().into_iter().chain(second.as_ref()).count();
        let root_term = Term::new(
            TermType::Tree(TreeTerm {
                left: second.unwrap_or(ArenaPointer::null()),
                right: first.unwrap_or(ArenaPointer::null()),
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
    ) -> <ArenaRef<Term, Self> as Expression>::StructPrototype {
        prototype
    }

    fn create_signal(
        &self,
        effect_type: SignalType,
        payload: ArenaRef<Term, Self>,
        token: ArenaRef<Term, Self>,
    ) -> <ArenaRef<Term, Self> as Expression>::Signal {
        debug_assert!(
            std::ptr::eq(
                payload.arena.arena.deref().borrow().deref(),
                self.arena.deref().borrow().deref(),
            ) && std::ptr::eq(
                token.arena.arena.deref().borrow().deref(),
                self.arena.deref().borrow().deref(),
            )
        );
        let term = Term::new(
            TermType::Condition(match effect_type {
                SignalType::Error => ConditionTerm::Error(ErrorCondition {
                    payload: payload.pointer,
                }),
                SignalType::Pending => ConditionTerm::Pending(PendingCondition),
                SignalType::Custom(effect_type) => {
                    let effect_type = self.create_string(effect_type);
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
    ) -> <ArenaRef<Term, Self> as Expression>::Signal {
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
    ) -> <ArenaRef<Term, Self> as Expression>::String {
        value
    }
}

impl<A: for<'a> ArenaAllocator<Slice<'a> = &'a [u8]> + 'static + Clone>
    ExpressionFactory<ArenaRef<Term, Self>> for WasmTermFactory<A>
{
    fn create_nil_term(&self) -> ArenaRef<Term, Self> {
        let term = Term::new(TermType::Nil(NilTerm), &*self.arena.borrow());
        let pointer = self.arena.borrow_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_boolean_term(&self, value: bool) -> ArenaRef<Term, Self> {
        let term = Term::new(
            TermType::Boolean(BooleanTerm::from(value)),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_int_term(&self, value: IntValue) -> ArenaRef<Term, Self> {
        let term = Term::new(TermType::Int(IntTerm { value }), &*self.arena.borrow());
        let pointer = self.arena.borrow_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_float_term(&self, value: FloatValue) -> ArenaRef<Term, Self> {
        let term = Term::new(
            TermType::Float(FloatTerm::from(value)),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().allocate(term);
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
        let pointer = self.arena.borrow_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    fn create_variable_term(&self, offset: StackOffset) -> ArenaRef<Term, Self> {
        let term = Term::new(
            TermType::Variable(VariableTerm {
                stack_offset: offset as u32,
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().allocate(term);
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
        let pointer = self.arena.borrow_mut().allocate(term);
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
        let pointer = self.arena.borrow_mut().allocate(term);
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
        let pointer = self.arena.borrow_mut().allocate(term);
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
        let pointer = self.arena.borrow_mut().allocate(term);
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
        let pointer = self.arena.borrow_mut().allocate(term);
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
        let condition_pointer = self.arena.borrow_mut().allocate(condition_term);
        let signal_list_term = Term::new(
            TermType::Tree(TreeTerm {
                left: condition_pointer,
                right: ArenaPointer::null(),
                length: 1,
            }),
            &*self.arena.borrow(),
        );
        let signal_list_pointer = self.arena.borrow_mut().allocate(signal_list_term);
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
        let pointer = self.arena.borrow_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
    }

    // TODO: Remove compiled function term type
    fn create_compiled_function_term(
        &self,
        _address: InstructionPointer,
        _hash: HashId,
        _required_args: StackOffset,
        _optional_args: StackOffset,
    ) -> ArenaRef<Term, Self> {
        self.create_signal_term(self.create_signal_list([self.create_signal(
            SignalType::Error,
            self.create_string_term(
                self.create_static_string("Compiled functions not supported in WASM interpreter"),
            ),
            self.create_nil_term(),
        )]))
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
            Some(self.create_hashmap_term(keys.iter().zip(fields.iter())))
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
        let pointer = self.arena.borrow_mut().allocate(term);
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
        let pointer = self.arena.borrow_mut().allocate(term);
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
        let pointer = HashmapTerm::allocate(
            entries
                .into_iter()
                .map(|(key, value)| (key.as_pointer(), value.as_pointer())),
            &mut *self.arena.borrow_mut(),
        );
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
        let entries = self
            .create_hashmap_term(values.into_iter().map(|value| (value, nil.clone())))
            .as_pointer();
        let term = HashsetTerm { entries };
        let pointer = self.arena.borrow_mut().allocate(term);
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
        let pointer = self.arena.borrow_mut().allocate(term);
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
