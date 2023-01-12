// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::iter::repeat;

use reflex::core::{ArgType, Arity, Eagerness, RefType};
use reflex_utils::WithExactSizeIterator;

use crate::{
    allocator::Arena,
    compiler::{
        CompileWasm, CompiledExpression, CompiledInstruction, CompilerError, CompilerOptions,
        CompilerResult, CompilerScope, CompilerState, RuntimeBuiltin,
    },
    term_type::*,
    ArenaRef, Term,
};

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<ApplicationTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let target = self.target();
        let args = self.args();
        // TODO: Special-case short-circuiting applications (if/and/or/etc) in compiler
        // TODO: Invoke compiled builtin applications directly using static calls
        let mut instructions = CompiledExpression::default();
        // Push the application target onto the stack
        // => [Term]
        instructions.extend(target.compile(eager, scope, state, options)?);
        // Determine the arity of the callee if the callee is statically known,
        // otherwise assume the correct number of arguments have been provided
        let num_provided_args = args.as_inner().len();
        let arity = target
            .arity()
            .unwrap_or_else(|| Arity::lazy(num_provided_args, 0, false));
        // If an incorrect number of arguments were provided, return a compiler error
        if num_provided_args < arity.required().len()
            || (num_provided_args > arity.required().len() + arity.optional().len()
                && arity.variadic().is_none())
        {
            return Err(CompilerError::InvalidFunctionArgs {
                target: target,
                arity: arity,
                args: args.as_inner().iter().collect(),
            });
        }
        // If all arguments are lazy, push the argument list onto the stack as-is
        if arity
            .iter()
            .take(num_provided_args)
            .all(|arg_type| matches!(arg_type, ArgType::Lazy))
        {
            // Yield the argument list onto the stack
            // => [Term, ListTerm]
            instructions.extend(args.as_inner().compile(
                Eagerness::Eager,
                scope,
                state,
                options,
            )?);
        // Otherwise allocate a list containing the resolved arguments onto the stack
        } else {
            let args = args.as_inner();
            let args_with_eagerness =
                args.iter().zip(arity.iter().map(|arg_type| match arg_type {
                    ArgType::Strict | ArgType::Eager => Eagerness::Eager,
                    ArgType::Lazy => Eagerness::Lazy,
                }));
            // Allocate the argument list onto the stack
            // => [Term, ListTerm]
            instructions.extend(compile_list(
                WithExactSizeIterator::new(num_provided_args, args_with_eagerness),
                scope,
                state,
                options,
            )?);
        }
        // Invoke the term constructor
        // => [ApplicationTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateApplication,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<BooleanTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let value = self.value();
        let mut instructions = CompiledExpression::default();
        // Push the value argument onto the stack
        // => [value]
        instructions.push(CompiledInstruction::u32_const(value as u32));
        // Invoke the term constructor
        // => [BooleanTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateBoolean,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<BuiltinTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let target = self.target();
        let mut instructions = CompiledExpression::default();
        // Push the function index argument onto the stack
        // => [index]
        instructions.push(CompiledInstruction::u32_const(u32::from(target)));
        // Invoke the term constructor
        // => [BuiltinTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateBuiltin,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<CellTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let capacity = self.capacity();
        let fields = self.fields();
        let mut instructions = CompiledExpression::default();
        // Push the capacity argument onto the stack
        // => [capacity]
        instructions.push(CompiledInstruction::u32_const(capacity));
        // Allocate the cell term
        // => [CellTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateBoolean,
        ));
        // Write the cell contents into the newly-allocated cell term
        for (index, value) in fields.enumerate() {
            // Duplicate the cell term pointer onto the stack
            // => [CellTerm, CellTerm]
            instructions.push(CompiledInstruction::Duplicate);
            // Push the field index onto the stack
            // => [CellTerm, CellTerm, index]
            instructions.push(CompiledInstruction::u32_const(index as u32));
            // Push the cell value onto the stack
            // => [CellTerm, CellTerm, index, value]
            instructions.push(CompiledInstruction::u32_const(value));
            // Update the cell term's field at the given index
            // => [CellTerm]
            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::SetCellField,
            ));
        }
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<ConditionTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        match self.condition_type() {
            ConditionTermDiscriminants::Custom => self
                .as_typed_condition::<CustomCondition>()
                .as_inner()
                .compile(eager, scope, state, options),
            ConditionTermDiscriminants::Pending => self
                .as_typed_condition::<PendingCondition>()
                .as_inner()
                .compile(eager, scope, state, options),
            ConditionTermDiscriminants::Error => self
                .as_typed_condition::<ErrorCondition>()
                .as_inner()
                .compile(eager, scope, state, options),
            ConditionTermDiscriminants::TypeError => self
                .as_typed_condition::<TypeErrorCondition>()
                .as_inner()
                .compile(eager, scope, state, options),
            ConditionTermDiscriminants::InvalidFunctionTarget => self
                .as_typed_condition::<InvalidFunctionTargetCondition>()
                .as_inner()
                .compile(eager, scope, state, options),
            ConditionTermDiscriminants::InvalidFunctionArgs => self
                .as_typed_condition::<InvalidFunctionArgsCondition>()
                .as_inner()
                .compile(eager, scope, state, options),
            ConditionTermDiscriminants::InvalidPointer => self
                .as_typed_condition::<InvalidPointerCondition>()
                .as_inner()
                .compile(eager, scope, state, options),
        }
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<CustomCondition, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let effect_type = self.effect_type();
        let payload = self.payload();
        let token = self.token();
        let mut instructions = CompiledExpression::default();
        // Yield the effect type onto the stack
        // => [Term]
        instructions.extend(effect_type.compile(eager, scope, state, options)?);
        // Yield the payload onto the stack
        // => [Term, Term]
        instructions.extend(payload.compile(eager, scope, state, options)?);
        // Yield the token onto the stack
        // => [Term, Term, Token]
        instructions.extend(token.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [ConditionTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateCustomCondition,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<PendingCondition, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let mut instructions = CompiledExpression::default();
        // Invoke the term constructor
        // => [ConditionTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreatePendingCondition,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<ErrorCondition, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let payload = self.payload();
        let mut instructions = CompiledExpression::default();
        // Yield the payload onto the stack
        // => [Term]
        instructions.extend(payload.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [ConditionTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateErrorCondition,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<TypeErrorCondition, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let expected = self.expected();
        let payload = self.payload();
        let mut instructions = CompiledExpression::default();
        // Push the expected type identifier onto the stack
        // => [type_id]
        instructions.push(CompiledInstruction::u32_const(expected));
        // Yield the received payload onto the stack
        // => [type_id, Term]
        instructions.extend(payload.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [ConditionTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateTypeErrorCondition,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<InvalidFunctionTargetCondition, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let target = self.target();
        let mut instructions = CompiledExpression::default();
        // Yield the target onto the stack
        // => [Term]
        instructions.extend(target.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [ConditionTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateInvalidFunctionTargetCondition,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<InvalidFunctionArgsCondition, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let target = self.target();
        let args = self.args();
        let mut instructions = CompiledExpression::default();
        // Yield the target onto the stack
        // => [Option<Term>]
        match target {
            Some(target) => {
                instructions.extend(target.compile(eager, scope, state, options)?);
            }
            None => instructions.push(CompiledInstruction::Null),
        }
        // Yield the argument list onto the stack
        // => [Option<Term>, ListTerm]
        instructions.extend(args.as_inner().compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [ConditionTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateInvalidFunctionArgsCondition,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<InvalidPointerCondition, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let mut instructions = CompiledExpression::default();
        // Invoke the term constructor
        // => [ConditionTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateInvalidPointerCondition,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<ConstructorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let keys = self.keys();
        let mut instructions = CompiledExpression::default();
        // Push the prototype key list onto the stack
        // => [ListTerm]
        instructions.extend(keys.as_deref().as_inner().compile(
            Eagerness::Eager,
            scope,
            state,
            options,
        )?);
        // Invoke the term constructor
        // => [ConstructorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateConstructor,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<DateTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let timestamp = self.timestamp();
        let mut instructions = CompiledExpression::default();
        // Push the value argument onto the stack
        // => [value]
        instructions.push(CompiledInstruction::i64_const(timestamp));
        // Invoke the term constructor
        // => [DateTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateDate,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<EffectTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        // TODO: Compile eager effect terms properly
        let condition = self.condition();
        let mut instructions = CompiledExpression::default();
        // Push the condition argument onto the stack
        // => [ListTerm]
        instructions.extend(condition.as_inner().compile(
            Eagerness::Eager,
            scope,
            state,
            options,
        )?);
        // Invoke the term constructor
        // => [EffectTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateEffect,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<FloatTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let value = self.value();
        let mut instructions = CompiledExpression::default();
        // Push the value argument onto the stack
        // => [value]
        instructions.push(CompiledInstruction::f64_const(value));
        // Invoke the term constructor
        // => [FloatTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateFloat,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<HashmapTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let capacity = self.capacity();
        let num_entries = self.num_entries();
        let keys = self.keys();
        let values = self.values();
        let mut instructions = CompiledExpression::default();
        // Push the capacity onto the stack
        // => [capacity]
        instructions.push(CompiledInstruction::u32_const(capacity as u32));
        // Allocate the hashmap term
        // => [HashmapTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::AllocateHashmap,
        ));
        // Assign the hashmap entries
        for (key, value) in keys.zip(values) {
            // Duplicate the hashmap term pointer onto the stack
            // => [HashmapTerm, HashmapTerm]
            instructions.push(CompiledInstruction::Duplicate);
            // Yield the entry's key onto the stack
            // => [HashmapTerm, HashmapTerm, index, Term]
            instructions.extend(key.compile(eager, scope, state, options)?);
            // Yield the entry's value onto the stack
            // => [HashmapTerm, HashmapTerm, index, Term]
            instructions.extend(value.compile(eager, scope, state, options)?);
            // Insert the entry into the hashmap
            // => [HashmapTerm]
            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::InsertHashmapEntry,
            ));
        }
        // Now that the hashmap entries have been added, push the number of entries onto the stack
        // => [HashmapTerm, num_entries]
        instructions.push(CompiledInstruction::u32_const(num_entries as u32));
        // Initialize the hashmap term with the length that is on the stack
        // => [HashmapTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::InitHashmap,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<HashsetTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let entries = self.entries();
        let mut instructions = CompiledExpression::default();
        // Allocate the entries argument onto the stack
        // => [HashmapTerm]
        instructions.extend(entries.as_inner().compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [HashsetTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateHashset,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<IntTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let value = self.value();
        let mut instructions = CompiledExpression::default();
        // Push the value argument onto the stack
        // => [value]
        instructions.push(CompiledInstruction::i32_const(value));
        // Invoke the term constructor
        // => [IntTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateInt,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<LambdaTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        // TODO: Compile lambda terms properly
        let num_args = self.num_args();
        let body = self.body();
        let mut instructions = CompiledExpression::default();
        // Push the number of arguments onto the stack
        // => [num_args]
        instructions.push(CompiledInstruction::u32_const(num_args));
        // Yield the function body onto the stack
        // => [num_args, Term]
        // FIXME: compile lambda term body as eager expression within child scope
        instructions.extend(body.compile(Eagerness::Lazy, scope, state, options)?);
        // Invoke the term constructor
        // => [LambdaTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateLambda,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<LetTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        // TODO: Compile let terms properly
        let initializer = self.initializer();
        let body = self.body();
        let mut instructions = CompiledExpression::default();
        // Yield the initializer term onto the stack
        // => [Term]
        instructions.extend(initializer.compile(eager, scope, state, options)?);
        // Yield the expression body onto the stack
        // => [Term, Term]
        // FIXME: compile let term body as eager expression within child scope
        instructions.extend(body.compile(Eagerness::Lazy, scope, state, options)?);
        // Invoke the term constructor
        // => [LetTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateLet,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<ListTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let num_items = self.len();
        let items = self.iter();
        let items_with_eagerness =
            WithExactSizeIterator::new(num_items, items.zip(repeat(Eagerness::Lazy)));
        compile_list(items_with_eagerness, scope, state, options)
    }
}

fn compile_list<A: Arena + Clone>(
    items: impl IntoIterator<
        Item = (ArenaRef<Term, A>, Eagerness),
        IntoIter = impl ExactSizeIterator<Item = (ArenaRef<Term, A>, Eagerness)>,
    >,
    scope: &CompilerScope,
    state: &mut CompilerState,
    options: &CompilerOptions,
) -> CompilerResult<A> {
    let items = items.into_iter();
    let num_items = items.len();
    let mut instructions = CompiledExpression::default();
    // Push the list capacity onto the stack
    // => [capacity]
    instructions.push(CompiledInstruction::u32_const(num_items as u32));
    // Allocate the list term
    // => [ListTerm]
    instructions.push(CompiledInstruction::CallRuntimeBuiltin(
        RuntimeBuiltin::AllocateList,
    ));
    // Assign the list items
    for (index, (item, eager)) in items.enumerate() {
        // Duplicate the list term pointer onto the stack
        // => [ListTerm, ListTerm]
        instructions.push(CompiledInstruction::Duplicate);
        // Push the item index onto the stack
        // => [ListTerm, ListTerm, index]
        instructions.push(CompiledInstruction::u32_const(index as u32));
        // Yield the child item onto the stack
        // => [ListTerm, ListTerm, index, Term]
        instructions.extend(item.compile(eager, scope, state, options)?);
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

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<NilTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let mut instructions = CompiledExpression::default();
        // Invoke the term constructor
        // => [NilTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateNil,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<PartialTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let target = self.target();
        let args = self.args();
        let mut instructions = CompiledExpression::default();
        // Push the partial application target onto the stack
        // => [Term]
        instructions.extend(target.compile(Eagerness::Lazy, scope, state, options)?);
        // Push the partial application arguments onto the stack
        // => [Term, ListTerm]
        instructions.extend(
            args.as_inner()
                .compile(Eagerness::Lazy, scope, state, options)?,
        );
        // Invoke the term constructor
        // => [PartialTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreatePartial,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<PointerTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let target = self.target();
        let mut instructions = CompiledExpression::default();
        // Push the target argument onto the stack
        // => [target]
        instructions.push(CompiledInstruction::u32_const(u32::from(target)));
        // Invoke the term constructor
        // => [PointerTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreatePointer,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<RecordTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let keys = self.keys();
        let values = self.values();
        let mut instructions = CompiledExpression::default();
        // Push the keys argument onto the stack
        // => [ListTerm]
        instructions.extend(
            keys.as_inner()
                .compile(Eagerness::Eager, scope, state, options)?,
        );
        // Push the values argument onto the stack
        // => [ListTerm]
        instructions.extend(
            values
                .as_inner()
                .compile(Eagerness::Eager, scope, state, options)?,
        );
        // Invoke the term constructor
        // => [RecordTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateRecord,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<SignalTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let conditions = self.conditions();
        let mut instructions = CompiledExpression::default();
        // Push the conditions argument onto the stack
        // => [ListTerm]
        instructions.extend(conditions.as_inner().compile(
            Eagerness::Eager,
            scope,
            state,
            options,
        )?);
        // Invoke the term constructor
        // => [SignalTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateSignal,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<StringTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let num_chars = self.len();
        let data = self.data();
        let mut instructions = CompiledExpression::default();
        // Push the string length onto the stack
        // => [length]
        instructions.push(CompiledInstruction::u32_const(num_chars as u32));
        // Allocate the string term
        // => [StringTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::AllocateString,
        ));
        // Assign the string contents
        for (chunk_index, chunk) in data.iter().enumerate() {
            let char_index = chunk_index * std::mem::size_of::<u32>();
            // Duplicate the string term pointer onto the stack
            // => [StringTerm, StringTerm]
            instructions.push(CompiledInstruction::Duplicate);
            // Push the chunk index onto the stack
            // => [StringTerm, StringTerm, index]
            instructions.push(CompiledInstruction::u32_const(char_index as u32));
            // Get the character offset for the chunk at the given index
            // => [StringTerm, offset]
            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::GetStringCharOffset,
            ));
            // Push the chunk onto the stack
            // => [StringTerm, offset, chunk]
            instructions.push(CompiledInstruction::u32_const(chunk));
            // Write the chunk value to the string contents
            // => [StringTerm]
            instructions.push(CompiledInstruction::CallRuntimeBuiltin(
                RuntimeBuiltin::Write,
            ));
        }
        // Now that the string contents has been added, push the string length onto the stack
        // => [StringTerm, length]
        instructions.push(CompiledInstruction::u32_const(num_chars as u32));
        // Initialize the string term with the length that is on the stack
        // => [StringTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::InitString,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<SymbolTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let id = self.id();
        let mut instructions = CompiledExpression::default();
        // Push the id argument onto the stack
        // => [id]
        instructions.push(CompiledInstruction::u32_const(id));
        // Invoke the term constructor
        // => [IntTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateInt,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<TreeTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let left = self.left();
        let right = self.right();
        let mut instructions = CompiledExpression::default();
        // Push the left argument onto the stack
        // => [Option<Term>]
        if let Some(term) = left {
            instructions.extend(term.compile(Eagerness::Lazy, scope, state, options)?);
        } else {
            instructions.push(CompiledInstruction::Null);
        }
        // Push the right argument onto the stack
        // => [Option<Term>, Option<Term>]
        if let Some(term) = right {
            instructions.extend(term.compile(Eagerness::Lazy, scope, state, options)?);
        } else {
            instructions.push(CompiledInstruction::Null);
        }
        // Invoke the term constructor
        // => [TreeTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateTree,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<VariableTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        // TODO: Compile variable terms properly
        let stack_offset = self.stack_offset();
        let mut instructions = CompiledExpression::default();
        // Push the variable offset onto the stack
        // => [offset]
        instructions.push(CompiledInstruction::u32_const(stack_offset as u32));
        // Invoke the term constructor
        // => [VariableTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateVariable,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<EmptyIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let mut instructions = CompiledExpression::default();
        // Invoke the term constructor
        // => [EmptyIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateEmptyIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<EvaluateIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let source = self.source();
        let mut instructions = CompiledExpression::default();
        // Push the source argument onto the stack
        // => [Term]
        instructions.extend(source.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [EvaluateIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateEvaluateIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<FilterIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let source = self.source();
        let predicate = self.predicate();
        let mut instructions = CompiledExpression::default();
        // Push the source argument onto the stack
        // => [Term]
        instructions.extend(source.compile(eager, scope, state, options)?);
        // Push the predicate argument onto the stack
        // => [Term, Term]
        instructions.extend(predicate.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [FilterIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateFilterIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<FlattenIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let source = self.source();
        let mut instructions = CompiledExpression::default();
        // Push the source argument onto the stack
        // => [Term]
        instructions.extend(source.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [FlattenIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateFlattenIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<HashmapKeysIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let source = self.source();
        let mut instructions = CompiledExpression::default();
        // Push the source argument onto the stack
        // => [Term]
        instructions.extend(source.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [HashmapKeysIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateHashmapKeysIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<HashmapValuesIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let source = self.source();
        let mut instructions = CompiledExpression::default();
        // Push the source argument onto the stack
        // => [Term]
        instructions.extend(source.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [HashmapValuesIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateHashmapValuesIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<IntegersIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let mut instructions = CompiledExpression::default();
        // Invoke the term constructor
        // => [IntegersIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateIntegersIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<IntersperseIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let source = self.source();
        let separator = self.separator();
        let mut instructions = CompiledExpression::default();
        // Push the source argument onto the stack
        // => [Term]
        instructions.extend(source.compile(eager, scope, state, options)?);
        // Push the separator argument onto the stack
        // => [Term, Term]
        instructions.extend(separator.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [IntersperseIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateIntersperseIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<MapIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let source = self.source();
        let iteratee = self.iteratee();
        let mut instructions = CompiledExpression::default();
        // Push the source argument onto the stack
        // => [Term]
        instructions.extend(source.compile(eager, scope, state, options)?);
        // Push the predicate argument onto the stack
        // => [Term, Term]
        instructions.extend(iteratee.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [MapIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateMapIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<OnceIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let value = self.value();
        let mut instructions = CompiledExpression::default();
        // Push the value argument onto the stack
        // => [Term]
        instructions.extend(value.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [OnceIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateOnceIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<RangeIteratorTerm, A> {
    fn compile(
        &self,
        _eager: Eagerness,
        _scope: &CompilerScope,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let offset = self.offset();
        let length = self.length();
        let mut instructions = CompiledExpression::default();
        // Push the offset argument onto the stack
        // => [offset]
        instructions.push(CompiledInstruction::i32_const(offset));
        // Push the length argument onto the stack
        // => [offset, length]
        instructions.push(CompiledInstruction::u32_const(length));
        // Invoke the term constructor
        // => [RangeIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateRangeIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<RepeatIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let value = self.value();
        let mut instructions = CompiledExpression::default();
        // Push the value argument onto the stack
        // => [Term]
        instructions.extend(value.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [RepeatIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateRepeatIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<SkipIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let source = self.source();
        let count = self.count();
        let mut instructions = CompiledExpression::default();
        // Push the source argument onto the stack
        // => [Term]
        instructions.extend(source.compile(eager, scope, state, options)?);
        // Push the count argument onto the stack
        // => [Term, count]
        instructions.push(CompiledInstruction::u32_const(count));
        // Invoke the term constructor
        // => [SkipIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateSkipIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<TakeIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let source = self.source();
        let count = self.count();
        let mut instructions = CompiledExpression::default();
        // Push the source argument onto the stack
        // => [Term]
        instructions.extend(source.compile(eager, scope, state, options)?);
        // Push the count argument onto the stack
        // => [Term, count]
        instructions.push(CompiledInstruction::u32_const(count));
        // Invoke the term constructor
        // => [TakeIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateTakeIterator,
        ));
        Ok(instructions)
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<ZipIteratorTerm, A> {
    fn compile(
        &self,
        eager: Eagerness,
        scope: &CompilerScope,
        state: &mut CompilerState,
        options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let left = self.left();
        let right = self.right();
        let mut instructions = CompiledExpression::default();
        // Push the left argument onto the stack
        // => [Term]
        instructions.extend(left.compile(eager, scope, state, options)?);
        // Push the right argument onto the stack
        // => [Term, Term]
        instructions.extend(right.compile(eager, scope, state, options)?);
        // Invoke the term constructor
        // => [ZipIteratorTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateZipIterator,
        ));
        Ok(instructions)
    }
}
