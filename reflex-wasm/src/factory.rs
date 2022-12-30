// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{cell::RefCell, rc::Rc};

use reflex::{
    core::{
        Expression, ExpressionFactory, FloatValue, InstructionPointer, IntValue, StackOffset,
        SymbolId,
    },
    hash::HashId,
};

use crate::{
    allocator::ArenaAllocator,
    hash::TermSize,
    term_type::{
        ApplicationTerm, BooleanTerm, BuiltinTerm, CompiledFunctionIndex, CompiledTerm,
        ConditionTerm, ConstructorTerm, EffectTerm, ErrorCondition, FloatTerm, HashmapTerm,
        HashsetTerm, IntTerm, LambdaTerm, LetTerm, ListTerm, NilTerm, PartialTerm, RecordTerm,
        SignalTerm, StringTerm, SymbolTerm, TermType, TermTypeDiscriminants, TreeTerm, TypedTerm,
        VariableTerm,
    },
    ArenaRef, Term, TermPointer,
};

#[derive(Debug)]
struct ArenaFactory<A: ArenaAllocator> {
    arena: Rc<RefCell<A>>,
}
impl<A: ArenaAllocator> Clone for ArenaFactory<A> {
    fn clone(&self) -> Self {
        Self {
            arena: Rc::clone(&self.arena),
        }
    }
}

impl<A: ArenaAllocator> ArenaAllocator for ArenaFactory<A> {
    fn len(&self) -> usize {
        self.arena.len()
    }
    fn allocate<T: TermSize>(&mut self, value: T) -> TermPointer {
        self.arena.allocate(value)
    }
    fn extend(&mut self, offset: TermPointer, size: usize) {
        self.arena.extend(offset, size)
    }
    fn shrink(&mut self, offset: TermPointer, size: usize) {
        self.arena.shrink(offset, size)
    }
    fn write<T: Sized>(&mut self, offset: TermPointer, value: T) {
        self.arena.write(offset, value)
    }
    fn read_value<T, V>(&self, offset: TermPointer, selector: impl FnOnce(&T) -> V) -> V {
        self.arena.read_value::<T, V>(offset, selector)
    }
    fn inner_pointer<T, V>(
        &self,
        offset: TermPointer,
        selector: impl FnOnce(&T) -> &V,
    ) -> TermPointer {
        self.arena.inner_pointer::<T, V>(offset, selector)
    }
}

impl<A: ArenaAllocator + Clone> ExpressionFactory<ArenaRef<Term, Self>> for ArenaFactory<A> {
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
                right: TermPointer::null(),
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

    fn create_compiled_function_term(
        &self,
        address: InstructionPointer,
        _hash: HashId,
        required_args: StackOffset,
        optional_args: StackOffset,
    ) -> ArenaRef<Term, Self> {
        let term = Term::new(
            TermType::Compiled(CompiledTerm {
                target: CompiledFunctionIndex::from(address.get() as u32),
                num_args: (required_args + optional_args) as u32,
            }),
            &*self.arena.borrow(),
        );
        let pointer = self.arena.borrow_mut().allocate(term);
        ArenaRef::<Term, Self>::new(self.clone(), pointer)
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
        let term = Term::new(
            TermType::Record(RecordTerm {
                keys: prototype.as_pointer(),
                values: fields.as_pointer(),
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
        expression: &'a ArenaRef<Term, Self>,
    ) -> Option<&'a <ArenaRef<Term, Self> as Expression>::CompiledFunctionTerm> {
        match expression.read_value(|term| term.type_id()) {
            TermTypeDiscriminants::Compiled => Some(expression.as_typed_term::<CompiledTerm>()),
            _ => None,
        }
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
