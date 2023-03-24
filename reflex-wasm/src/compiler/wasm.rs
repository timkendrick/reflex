// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    collections::{HashMap, VecDeque},
    iter::once,
};

use crate::{
    compiler::{
        builtin::RuntimeBuiltin, CompiledBlock, CompiledFunctionId, CompiledInstruction,
        ConstValue, FunctionPointer, ParamsSignature, TypeSignature, ValueType,
    },
    stdlib::Stdlib,
    utils::{from_twos_complement_i32, from_twos_complement_i64},
    FunctionIndex,
};

use reflex::core::StackOffset;
use walrus::{
    ir::{self, BinaryOp, Instr, InstrSeqType, LoadKind, MemArg, StoreKind, Value},
    FunctionBuilder, FunctionId, GlobalId, InstrSeqBuilder, LocalId, MemoryId, Module, ModuleTypes,
    TableId, ValType,
};

#[derive(Clone, Copy, Debug)]
pub struct RuntimeExportMappings {
    pub globals: RuntimeGlobalMappings,
    pub builtins: RuntimeBuiltinMappings,
    pub stdlib: RuntimeStdlibMappings,
}

#[derive(Clone, Copy, Debug)]
pub struct RuntimeGlobalMappings {
    pub null_pointer: GlobalId,
}

#[derive(Clone, Copy, Debug)]
pub struct RuntimeBuiltinMappings {
    pub initialize: FunctionId,
    pub evaluate: FunctionId,
    pub apply: FunctionId,
    pub combine_dependencies: FunctionId,
    pub combine_signals: FunctionId,
    pub is_signal: FunctionId,
    pub is_truthy: FunctionId,
    pub allocate_cell: FunctionId,
    pub allocate_hashmap: FunctionId,
    pub allocate_list: FunctionId,
    pub allocate_string: FunctionId,
    pub create_application: FunctionId,
    pub create_boolean: FunctionId,
    pub create_builtin: FunctionId,
    pub create_custom_condition: FunctionId,
    pub create_pending_condition: FunctionId,
    pub create_error_condition: FunctionId,
    pub create_type_error_condition: FunctionId,
    pub create_invalid_function_target_condition: FunctionId,
    pub create_invalid_function_args_condition: FunctionId,
    pub create_invalid_pointer_condition: FunctionId,
    pub create_constructor: FunctionId,
    pub create_date: FunctionId,
    pub create_effect: FunctionId,
    pub create_float: FunctionId,
    pub create_hashset: FunctionId,
    pub create_int: FunctionId,
    pub create_lambda: FunctionId,
    pub create_nil: FunctionId,
    pub create_partial: FunctionId,
    pub create_pointer: FunctionId,
    pub create_record: FunctionId,
    pub create_signal: FunctionId,
    pub create_tree: FunctionId,
    pub create_empty_iterator: FunctionId,
    pub create_evaluate_iterator: FunctionId,
    pub create_filter_iterator: FunctionId,
    pub create_flatten_iterator: FunctionId,
    pub create_hashmap_keys_iterator: FunctionId,
    pub create_hashmap_values_iterator: FunctionId,
    pub create_indexed_accessor_iterator: FunctionId,
    pub create_integers_iterator: FunctionId,
    pub create_intersperse_iterator: FunctionId,
    pub create_map_iterator: FunctionId,
    pub create_once_iterator: FunctionId,
    pub create_range_iterator: FunctionId,
    pub create_repeat_iterator: FunctionId,
    pub create_skip_iterator: FunctionId,
    pub create_take_iterator: FunctionId,
    pub create_zip_iterator: FunctionId,
    pub get_list_item: FunctionId,
    pub get_list_length: FunctionId,
    pub get_state_value: FunctionId,
    pub get_string_char_offset: FunctionId,
    pub init_hashmap: FunctionId,
    pub init_list: FunctionId,
    pub init_string: FunctionId,
    pub insert_hashmap_entry: FunctionId,
    pub set_cell_field: FunctionId,
    pub set_list_item: FunctionId,
}

impl RuntimeBuiltinMappings {
    pub fn get(&self, builtin: RuntimeBuiltin) -> FunctionId {
        match builtin {
            RuntimeBuiltin::Initialize => self.initialize,
            RuntimeBuiltin::Evaluate => self.evaluate,
            RuntimeBuiltin::Apply => self.apply,
            RuntimeBuiltin::CombineDependencies => self.combine_dependencies,
            RuntimeBuiltin::CombineSignals => self.combine_signals,
            RuntimeBuiltin::IsSignal => self.is_signal,
            RuntimeBuiltin::IsTruthy => self.is_truthy,
            RuntimeBuiltin::AllocateCell => self.allocate_cell,
            RuntimeBuiltin::AllocateHashmap => self.allocate_hashmap,
            RuntimeBuiltin::AllocateList => self.allocate_list,
            RuntimeBuiltin::AllocateString => self.allocate_string,
            RuntimeBuiltin::CreateApplication => self.create_application,
            RuntimeBuiltin::CreateBoolean => self.create_boolean,
            RuntimeBuiltin::CreateBuiltin => self.create_builtin,
            RuntimeBuiltin::CreateCustomCondition => self.create_custom_condition,
            RuntimeBuiltin::CreatePendingCondition => self.create_pending_condition,
            RuntimeBuiltin::CreateErrorCondition => self.create_error_condition,
            RuntimeBuiltin::CreateTypeErrorCondition => self.create_type_error_condition,
            RuntimeBuiltin::CreateInvalidFunctionTargetCondition => {
                self.create_invalid_function_target_condition
            }
            RuntimeBuiltin::CreateInvalidFunctionArgsCondition => {
                self.create_invalid_function_args_condition
            }
            RuntimeBuiltin::CreateInvalidPointerCondition => self.create_invalid_pointer_condition,
            RuntimeBuiltin::CreateConstructor => self.create_constructor,
            RuntimeBuiltin::CreateDate => self.create_date,
            RuntimeBuiltin::CreateEffect => self.create_effect,
            RuntimeBuiltin::CreateFloat => self.create_float,
            RuntimeBuiltin::CreateHashset => self.create_hashset,
            RuntimeBuiltin::CreateInt => self.create_int,
            RuntimeBuiltin::CreateLambda => self.create_lambda,
            RuntimeBuiltin::CreateNil => self.create_nil,
            RuntimeBuiltin::CreatePartial => self.create_partial,
            RuntimeBuiltin::CreatePointer => self.create_pointer,
            RuntimeBuiltin::CreateRecord => self.create_record,
            RuntimeBuiltin::CreateSignal => self.create_signal,
            RuntimeBuiltin::CreateTree => self.create_tree,
            RuntimeBuiltin::CreateEmptyIterator => self.create_empty_iterator,
            RuntimeBuiltin::CreateEvaluateIterator => self.create_evaluate_iterator,
            RuntimeBuiltin::CreateFilterIterator => self.create_filter_iterator,
            RuntimeBuiltin::CreateFlattenIterator => self.create_flatten_iterator,
            RuntimeBuiltin::CreateHashmapKeysIterator => self.create_hashmap_keys_iterator,
            RuntimeBuiltin::CreateHashmapValuesIterator => self.create_hashmap_values_iterator,
            RuntimeBuiltin::CreateIndexedAccessorIterator => self.create_indexed_accessor_iterator,
            RuntimeBuiltin::CreateIntegersIterator => self.create_integers_iterator,
            RuntimeBuiltin::CreateIntersperseIterator => self.create_intersperse_iterator,
            RuntimeBuiltin::CreateMapIterator => self.create_map_iterator,
            RuntimeBuiltin::CreateOnceIterator => self.create_once_iterator,
            RuntimeBuiltin::CreateRangeIterator => self.create_range_iterator,
            RuntimeBuiltin::CreateRepeatIterator => self.create_repeat_iterator,
            RuntimeBuiltin::CreateSkipIterator => self.create_skip_iterator,
            RuntimeBuiltin::CreateTakeIterator => self.create_take_iterator,
            RuntimeBuiltin::CreateZipIterator => self.create_zip_iterator,
            RuntimeBuiltin::GetListItem => self.get_list_item,
            RuntimeBuiltin::GetListLength => self.get_list_length,
            RuntimeBuiltin::GetStateValue => self.get_state_value,
            RuntimeBuiltin::GetStringCharOffset => self.get_string_char_offset,
            RuntimeBuiltin::InitHashmap => self.init_hashmap,
            RuntimeBuiltin::InitList => self.init_list,
            RuntimeBuiltin::InitString => self.init_string,
            RuntimeBuiltin::InsertHashmapEntry => self.insert_hashmap_entry,
            RuntimeBuiltin::SetCellField => self.set_cell_field,
            RuntimeBuiltin::SetListItem => self.set_list_item,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RuntimeStdlibMappings {
    pub abs: FunctionId,
    pub accessor: FunctionId,
    pub add: FunctionId,
    pub and: FunctionId,
    pub apply: FunctionId,
    pub car: FunctionId,
    pub cdr: FunctionId,
    pub ceil: FunctionId,
    pub chain: FunctionId,
    pub collect_hashmap: FunctionId,
    pub collect_hashset: FunctionId,
    pub collect_list: FunctionId,
    pub collect_signal: FunctionId,
    pub collect_string: FunctionId,
    pub collect_tree: FunctionId,
    pub cons: FunctionId,
    pub construct: FunctionId,
    pub construct_hashmap: FunctionId,
    pub construct_hashset: FunctionId,
    pub construct_list: FunctionId,
    pub construct_record: FunctionId,
    pub debug: FunctionId,
    pub decrement_variable: FunctionId,
    pub divide: FunctionId,
    pub effect: FunctionId,
    pub ends_with: FunctionId,
    pub eq: FunctionId,
    pub equal: FunctionId,
    pub filter: FunctionId,
    pub flatten: FunctionId,
    pub floor: FunctionId,
    pub fold: FunctionId,
    pub format_error_message: FunctionId,
    pub get: FunctionId,
    pub get_variable: FunctionId,
    pub graph_ql_resolver: FunctionId,
    pub gt: FunctionId,
    pub gte: FunctionId,
    pub has: FunctionId,
    pub hash: FunctionId,
    pub identity: FunctionId,
    pub r#if: FunctionId,
    pub if_error: FunctionId,
    pub if_pending: FunctionId,
    pub increment_variable: FunctionId,
    pub is_finite: FunctionId,
    pub iterate: FunctionId,
    pub keys: FunctionId,
    pub length: FunctionId,
    pub log: FunctionId,
    pub lt: FunctionId,
    pub lte: FunctionId,
    pub map: FunctionId,
    pub max: FunctionId,
    pub merge: FunctionId,
    pub min: FunctionId,
    pub multiply: FunctionId,
    pub not: FunctionId,
    pub or: FunctionId,
    pub parse_date: FunctionId,
    pub parse_float: FunctionId,
    pub parse_int: FunctionId,
    pub parse_json: FunctionId,
    pub pow: FunctionId,
    pub push: FunctionId,
    pub push_front: FunctionId,
    pub raise: FunctionId,
    pub remainder: FunctionId,
    pub replace: FunctionId,
    pub resolve_args: FunctionId,
    pub resolve_deep: FunctionId,
    pub resolve_hashmap: FunctionId,
    pub resolve_hashset: FunctionId,
    pub resolve_list: FunctionId,
    pub resolve_query_branch: FunctionId,
    pub resolve_query_leaf: FunctionId,
    pub resolve_record: FunctionId,
    pub resolve_shallow: FunctionId,
    pub resolve_tree: FunctionId,
    pub round: FunctionId,
    pub scan: FunctionId,
    pub sequence: FunctionId,
    pub set: FunctionId,
    pub set_variable: FunctionId,
    pub skip: FunctionId,
    pub slice: FunctionId,
    pub split: FunctionId,
    pub starts_with: FunctionId,
    pub stringify_json: FunctionId,
    pub subtract: FunctionId,
    pub take: FunctionId,
    pub throw: FunctionId,
    pub to_request: FunctionId,
    pub to_string: FunctionId,
    pub urlencode: FunctionId,
    pub unzip: FunctionId,
    pub values: FunctionId,
    pub zip: FunctionId,
}

impl RuntimeStdlibMappings {
    fn get_function_id(&self, target: Stdlib) -> FunctionId {
        match target {
            Stdlib::Abs(_) => self.abs,
            Stdlib::Accessor(_) => self.accessor,
            Stdlib::Add(_) => self.add,
            Stdlib::And(_) => self.and,
            Stdlib::Apply(_) => self.apply,
            Stdlib::Car(_) => self.car,
            Stdlib::Cdr(_) => self.cdr,
            Stdlib::Ceil(_) => self.ceil,
            Stdlib::Chain(_) => self.chain,
            Stdlib::CollectHashmap(_) => self.collect_hashmap,
            Stdlib::CollectHashset(_) => self.collect_hashset,
            Stdlib::CollectList(_) => self.collect_list,
            Stdlib::CollectSignal(_) => self.collect_signal,
            Stdlib::CollectString(_) => self.collect_string,
            Stdlib::CollectTree(_) => self.collect_tree,
            Stdlib::Cons(_) => self.cons,
            Stdlib::Construct(_) => self.construct,
            Stdlib::ConstructHashmap(_) => self.construct_hashmap,
            Stdlib::ConstructHashset(_) => self.construct_hashset,
            Stdlib::ConstructList(_) => self.construct_list,
            Stdlib::ConstructRecord(_) => self.construct_record,
            Stdlib::Debug(_) => self.debug,
            Stdlib::DecrementVariable(_) => self.decrement_variable,
            Stdlib::Divide(_) => self.divide,
            Stdlib::Effect(_) => self.effect,
            Stdlib::EndsWith(_) => self.ends_with,
            Stdlib::Eq(_) => self.eq,
            Stdlib::Equal(_) => self.equal,
            Stdlib::Filter(_) => self.filter,
            Stdlib::Flatten(_) => self.flatten,
            Stdlib::Floor(_) => self.floor,
            Stdlib::Fold(_) => self.fold,
            Stdlib::FormatErrorMessage(_) => self.format_error_message,
            Stdlib::Get(_) => self.get,
            Stdlib::GetVariable(_) => self.get_variable,
            Stdlib::GraphQlResolver(_) => self.graph_ql_resolver,
            Stdlib::Gt(_) => self.gt,
            Stdlib::Gte(_) => self.gte,
            Stdlib::Has(_) => self.has,
            Stdlib::Hash(_) => self.hash,
            Stdlib::Identity(_) => self.identity,
            Stdlib::If(_) => self.r#if,
            Stdlib::IfError(_) => self.if_error,
            Stdlib::IfPending(_) => self.if_pending,
            Stdlib::IncrementVariable(_) => self.increment_variable,
            Stdlib::IsFinite(_) => self.is_finite,
            Stdlib::Iterate(_) => self.iterate,
            Stdlib::Keys(_) => self.keys,
            Stdlib::Length(_) => self.length,
            Stdlib::Log(_) => self.log,
            Stdlib::Lt(_) => self.lt,
            Stdlib::Lte(_) => self.lte,
            Stdlib::Map(_) => self.map,
            Stdlib::Max(_) => self.max,
            Stdlib::Merge(_) => self.merge,
            Stdlib::Min(_) => self.min,
            Stdlib::Multiply(_) => self.multiply,
            Stdlib::Not(_) => self.not,
            Stdlib::Or(_) => self.or,
            Stdlib::ParseDate(_) => self.parse_date,
            Stdlib::ParseFloat(_) => self.parse_float,
            Stdlib::ParseInt(_) => self.parse_int,
            Stdlib::ParseJson(_) => self.parse_json,
            Stdlib::Pow(_) => self.pow,
            Stdlib::Push(_) => self.push,
            Stdlib::PushFront(_) => self.push_front,
            Stdlib::Raise(_) => self.raise,
            Stdlib::Remainder(_) => self.remainder,
            Stdlib::Replace(_) => self.replace,
            Stdlib::ResolveArgs(_) => self.resolve_args,
            Stdlib::ResolveDeep(_) => self.resolve_deep,
            Stdlib::ResolveHashmap(_) => self.resolve_hashmap,
            Stdlib::ResolveHashset(_) => self.resolve_hashset,
            Stdlib::ResolveList(_) => self.resolve_list,
            Stdlib::ResolveQueryBranch(_) => self.resolve_query_branch,
            Stdlib::ResolveQueryLeaf(_) => self.resolve_query_leaf,
            Stdlib::ResolveRecord(_) => self.resolve_record,
            Stdlib::ResolveShallow(_) => self.resolve_shallow,
            Stdlib::ResolveTree(_) => self.resolve_tree,
            Stdlib::Round(_) => self.round,
            Stdlib::Scan(_) => self.scan,
            Stdlib::Sequence(_) => self.sequence,
            Stdlib::Set(_) => self.set,
            Stdlib::SetVariable(_) => self.set_variable,
            Stdlib::Skip(_) => self.skip,
            Stdlib::Slice(_) => self.slice,
            Stdlib::Split(_) => self.split,
            Stdlib::StartsWith(_) => self.starts_with,
            Stdlib::StringifyJson(_) => self.stringify_json,
            Stdlib::Subtract(_) => self.subtract,
            Stdlib::Take(_) => self.take,
            Stdlib::Throw(_) => self.throw,
            Stdlib::ToRequest(_) => self.to_request,
            Stdlib::ToString(_) => self.to_string,
            Stdlib::Urlencode(_) => self.urlencode,
            Stdlib::Unzip(_) => self.unzip,
            Stdlib::Values(_) => self.values,
            Stdlib::Zip(_) => self.zip,
        }
    }
    fn get_indirect_call_function_index(&self, target: Stdlib) -> FunctionIndex {
        // TODO: Determine function call indexes by parsing runtime module
        FunctionIndex::from(target)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct WasmGeneratorOptions {
    pub disable_block_params: bool,
}

#[derive(Debug, Clone)]
pub enum WasmGeneratorError {
    StackError,
    InvalidBlockReturnType(TypeSignature),
    InvalidCompiledFunction(CompiledFunctionId),
    InvalidContinuation,
}

/// Create a function from the given pre-compiled function body instructions with the requisite parameters to transparently handle stateful reactivity
///
/// The resulting function takes the following parameters:
///
///   - positional parameters specified in the `arg_types` argument
///   - a final positional parameter which is a term pointer to the global state object term
///
/// ...and returns two values:
///   - a pointer to the result term
///   - a pointer to the dependency list term
///
/// The body of the generated wrapper function will be a WASM translation of whatever instructions were passed as the body argument.
pub(crate) fn generate_stateful_function(
    module: &mut Module,
    arg_types: impl Iterator<Item = ValueType> + Clone,
    body: &CompiledBlock,
    export_mappings: &RuntimeExportMappings,
    memory_id: MemoryId,
    main_function_table_id: TableId,
    compiled_function_mappings: &WasmCompiledFunctionMappings,
    options: &WasmGeneratorOptions,
) -> Result<FunctionId, WasmGeneratorError> {
    // Define the function signature
    let params = {
        let args = arg_types.clone().map(|ty| parse_value_type(ty));
        let state = ValType::I32;
        args.chain([state]).collect::<Vec<_>>()
    };
    let results = {
        let value = ValType::I32;
        let dependencies = ValType::I32;
        [value, dependencies]
    };

    // Define locals to hold the function arguments
    let arg_ids = arg_types
        .map(|ty| module.locals.add(parse_value_type(ty)))
        .collect::<Vec<_>>();
    // Define a local to hold the state argument
    let state_id = module.locals.add(ValType::I32);
    // Combine the function arguments with the state argument
    let args = arg_ids
        .iter()
        .copied()
        .chain([state_id])
        .collect::<Vec<_>>();

    // Define a local to keep track of the accumulated dependencies within the function body
    let dependencies_id = module.locals.add(ValType::I32);

    // Define a local variable to use for the general-purpose temporary register
    let temp_id = module.locals.add(ValType::I32);

    // Initialize the function's locals stack with the function parameters
    // (latest argument is at the top of the stack, first argument at the bottom)
    let arg_stack = arg_ids;

    let mut bindings = WasmGeneratorBindings {
        memory_id,
        main_function_table_id,
        state_id,
        dependencies_id,
        temp_id,
        stack: arg_stack,
        compiled_function_mappings,
        export_mappings,
    };

    // Create the function signature
    let mut builder = FunctionBuilder::new(&mut module.types, &params, &results);
    // Create the function body
    {
        let mut function_body_builder = builder.func_body();
        // Initialize the dependencies local to the null pointer
        function_body_builder
            .global_get(export_mappings.globals.null_pointer)
            .local_set(dependencies_id);
        // Generate the WASM instructions for the rest of the function body
        let mut instructions = body.emit_wasm(module, &mut bindings, options)?;
        // Append the value of the dependencies local to the function body return value
        instructions.push(ir::LocalGet {
            local: dependencies_id,
        });
        // Append the generated instructions into the function body
        compile_wasm(&mut function_body_builder, instructions)?;
    }
    // Add the function to the WASM module
    let function_id = builder.finish(args, &mut module.funcs);
    Ok(function_id)
}

/// Create a wrapper function which can be used to make dynamic calls to the provided function
///
/// The resulting function takes two parameters:
///   - a pointer to an 'argument list' list term
///   - a pointer to the global state object term
///
/// ...and returns two values:
///   - a pointer to the result term
///   - a pointer to the dependency list term
///
/// The term pointers within the argument list will be passed to the inner function as positional arguments.
///
/// The body of the generated wrapper function first asserts that the argument list contains sufficient arguments,
/// then invokes the provided function with the correct number of arguments extracted from the argument list
pub(crate) fn generate_indirect_function_wrapper(
    module: &mut Module,
    function_id: FunctionId,
    num_args: usize,
    export_mappings: &RuntimeExportMappings,
) -> FunctionId {
    // Define the function signature
    let params = {
        let args = ValType::I32;
        let state = ValType::I32;
        [args, state]
    };
    let results = {
        let value = ValType::I32;
        let dependencies = ValType::I32;
        [value, dependencies]
    };

    // Define a local to hold the argument list pointer
    let arglist_id = module.locals.add(ValType::I32);
    // Define a local to hold the state argument
    let state_id = module.locals.add(ValType::I32);
    // Combine the arguments
    let args = vec![arglist_id, state_id];

    let mut builder = FunctionBuilder::new(&mut module.types, &params, &results);
    // Create the function body
    {
        let builder = &mut builder.func_body();
        // Determine whether insufficient arguments have been passed
        builder
            .local_get(arglist_id)
            .call(export_mappings.builtins.get_list_length)
            .i32_const(from_twos_complement_i32(num_args as u32))
            .binop(BinaryOp::I32LtU)
            .if_else(
                InstrSeqType::new(&mut module.types, &[], &results),
                |builder| {
                    // If insufficient arguments were passed, return an error signal term followed by the null dependencies pointer
                    builder
                        .global_get(export_mappings.globals.null_pointer)
                        .local_get(arglist_id)
                        .call(
                            export_mappings
                                .builtins
                                .create_invalid_function_args_condition,
                        )
                        .call(export_mappings.builtins.create_signal)
                        .global_get(export_mappings.globals.null_pointer);
                },
                |builder| {
                    // If sufficient arguments were passed, call the inner function with the correct number of positional args
                    // Extract the arguments from the argument list, pushing each argument onto the stack
                    for index in 0..num_args {
                        builder
                            .local_get(arglist_id)
                            .i32_const(from_twos_complement_i32(index as u32))
                            .call(export_mappings.builtins.get_list_item);
                    }
                    // Invoke the static function target, passing the state value as the final argument
                    builder.local_get(state_id).call(function_id);
                },
            );
    }
    // Add the function to the WASM module
    let function_id = builder.finish(args, &mut module.funcs);
    function_id
}

impl std::error::Error for WasmGeneratorError {}

impl std::fmt::Display for WasmGeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StackError => write!(f, "Invalid stack access"),
            Self::InvalidBlockReturnType(ty) => write!(f, "Invalid block return type: {ty:?}"),
            Self::InvalidCompiledFunction(id) => write!(f, "Invalid compiled function ID: {id}"),
            Self::InvalidContinuation => write!(f, "Invalid continuation instruction"),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct WasmCompiledFunctionMappings {
    mappings: HashMap<CompiledFunctionId, (FunctionId, FunctionIndex)>,
}
impl WasmCompiledFunctionMappings {
    pub fn insert(
        &mut self,
        target: CompiledFunctionId,
        function_id: FunctionId,
        function_index: FunctionIndex,
    ) {
        self.mappings.insert(target, (function_id, function_index));
    }
    pub fn get_function_id(&self, target: CompiledFunctionId) -> Option<FunctionId> {
        self.mappings
            .get(&target)
            .map(|(function_id, _)| *function_id)
    }
    pub fn get_indirect_call_function_index(
        &self,
        target: CompiledFunctionId,
    ) -> Option<FunctionIndex> {
        self.mappings
            .get(&target)
            .map(|(_, function_index)| *function_index)
    }
}

struct WasmGeneratorBindings<'a> {
    /// Heap linear memory ID
    memory_id: MemoryId,
    /// Dynamic function lookup table
    main_function_table_id: TableId,
    /// Global state argument parameter ID
    state_id: LocalId,
    /// Variable to keep track of any dependencies encountered during this evaluation
    dependencies_id: LocalId,
    /// Temporary free-use register
    temp_id: LocalId,
    /// Locals currrently accessible on the lexical scope stack (this list will grow and shrink as new lexical scopes are created and disposed)
    stack: Vec<LocalId>,
    /// Lookup table mapping term hashes to compiled function ids
    compiled_function_mappings: &'a WasmCompiledFunctionMappings,
    /// Struct containing IDs of runtime builtin functions
    export_mappings: &'a RuntimeExportMappings,
}
impl<'a> WasmGeneratorBindings<'a> {
    fn enter_scope(&mut self, local_id: LocalId) -> LocalId {
        self.stack.push(local_id);
        local_id
    }
    fn leave_scope(&mut self) -> Option<LocalId> {
        self.stack.pop()
    }
    fn get_local(&self, offset: StackOffset) -> Result<LocalId, WasmGeneratorError> {
        if offset < self.stack.len() {
            Ok(self.stack[self.stack.len() - 1 - offset])
        } else {
            Err(WasmGeneratorError::StackError)
        }
    }
}

fn parse_block_type_signature(signature: &TypeSignature, types: &mut ModuleTypes) -> InstrSeqType {
    let TypeSignature { params, results } = signature;
    if let ParamsSignature::Void = params {
        match results {
            ParamsSignature::Void => InstrSeqType::Simple(None),
            ParamsSignature::Single(result1) => {
                InstrSeqType::Simple(Some(parse_value_type(*result1)))
            }
            ParamsSignature::Pair(result1, result2) => InstrSeqType::new(
                types,
                &[],
                &[parse_value_type(*result1), parse_value_type(*result2)],
            ),
            ParamsSignature::Triple(result1, result2, result3) => InstrSeqType::new(
                types,
                &[],
                &[
                    parse_value_type(*result1),
                    parse_value_type(*result2),
                    parse_value_type(*result3),
                ],
            ),
            ParamsSignature::Multiple(results) => InstrSeqType::new(
                types,
                &[],
                &results
                    .iter()
                    .copied()
                    .map(parse_value_type)
                    .collect::<Vec<_>>(),
            ),
        }
    } else {
        let params = params.iter().map(parse_value_type).collect::<Vec<_>>();
        let results = results.iter().map(parse_value_type).collect::<Vec<_>>();
        InstrSeqType::new(types, &params, &results)
    }
}

fn parse_function_type_signature(signature: &TypeSignature) -> (Vec<ValType>, Vec<ValType>) {
    let TypeSignature { params, results } = signature;
    let params = params.iter().map(parse_value_type).collect::<Vec<_>>();
    let results = results.iter().map(parse_value_type).collect::<Vec<_>>();
    (params, results)
}

fn parse_value_type(ty: ValueType) -> ValType {
    match ty {
        ValueType::I32 | ValueType::U32 | ValueType::HeapPointer | ValueType::FunctionPointer => {
            ValType::I32
        }
        ValueType::I64 | ValueType::U64 => ValType::I64,
        ValueType::F32 => ValType::F32,
        ValueType::F64 => ValType::F64,
    }
}

#[derive(Default, Clone, Debug)]
struct WasmGeneratorOutput {
    instructions: Vec<WasmGeneratorDirective>,
}
impl WasmGeneratorOutput {
    fn push(&mut self, instruction: impl Into<Instr>) {
        self.instructions
            .push(WasmGeneratorDirective::Instruction(instruction.into()));
    }
    fn branch(
        &mut self,
        block_type: &TypeSignature,
        consequent: WasmGeneratorOutput,
        alternative: WasmGeneratorOutput,
        module: &mut Module,
        bindings: &WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) {
        // If the compiler output format does not support block input params, emulate this feature by using locals to
        // save the stack values before branching, then once within the child block we can load the stack values back
        // out from the locals (these temporary locals are typically compiled away in a later optimization pass)
        if options.disable_block_params && (block_type.params.len() > 0) {
            // First create temporary locals to hold the stack values
            let param_ids = block_type
                .params
                .iter()
                .map(|param_type| module.locals.add(parse_value_type(param_type)))
                .collect::<Vec<_>>();
            // Temporarily pop the branch condition into the temporary local
            self.push(ir::LocalSet {
                local: bindings.temp_id,
            });
            // Pop all the captured operand stack values into their respective locals
            for param_id in param_ids.iter().rev() {
                self.push(ir::LocalSet { local: *param_id });
            }
            // Push the branch condition back onto the operand stack
            self.push(ir::LocalGet {
                local: bindings.temp_id,
            });
            // Prepare the child block header that pushes the captured values back onto the operand stack
            let block_header =
                WasmGeneratorOutput::from_iter(param_ids.into_iter().map(|param_id| {
                    WasmGeneratorDirective::Instruction(ir::Instr::LocalGet(ir::LocalGet {
                        local: param_id,
                    }))
                }));
            let (consequent_header, alternative_header) = (block_header.clone(), block_header);
            // Emit the rewritten branch instruction
            self.instructions.push(WasmGeneratorDirective::Branch {
                block_type: parse_block_type_signature(
                    &TypeSignature {
                        params: ParamsSignature::Void,
                        results: block_type.results.clone(),
                    },
                    &mut module.types,
                ),
                consequent: {
                    let mut instructions = consequent_header;
                    instructions.append_block(consequent);
                    instructions
                },
                alternative: {
                    let mut instructions = alternative_header;
                    instructions.append_block(alternative);
                    instructions
                },
            })
        } else {
            // Otherwise if we are not manually capturing any stack values, emit the branch instruction as-is
            self.instructions.push(WasmGeneratorDirective::Branch {
                block_type: parse_block_type_signature(block_type, &mut module.types),
                consequent,
                alternative,
            });
        }
    }
    fn continuation(&mut self) {
        self.instructions.push(WasmGeneratorDirective::Continuation);
    }
    fn append_block(&mut self, block: WasmGeneratorOutput) {
        self.instructions.extend(block);
    }
    fn iter(&self) -> std::slice::Iter<'_, WasmGeneratorDirective> {
        self.instructions.iter()
    }
}
impl FromIterator<WasmGeneratorDirective> for WasmGeneratorOutput {
    fn from_iter<T: IntoIterator<Item = WasmGeneratorDirective>>(iter: T) -> Self {
        Self {
            instructions: iter.into_iter().collect(),
        }
    }
}
impl IntoIterator for WasmGeneratorOutput {
    type Item = WasmGeneratorDirective;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}
impl<'a> IntoIterator for &'a WasmGeneratorOutput {
    type Item = &'a WasmGeneratorDirective;
    type IntoIter = std::slice::Iter<'a, WasmGeneratorDirective>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl Extend<WasmGeneratorDirective> for WasmGeneratorOutput {
    fn extend<T: IntoIterator<Item = WasmGeneratorDirective>>(&mut self, iter: T) {
        self.instructions.extend(iter)
    }
}

#[derive(Debug, Clone)]
enum WasmGeneratorDirective {
    /// Standard WASM instruction
    Instruction(Instr),
    /// If-else conditional branch instruction
    Branch {
        block_type: InstrSeqType,
        consequent: WasmGeneratorOutput,
        alternative: WasmGeneratorOutput,
    },
    /// 'Hole' representing an injection point to insert any remaining instructions
    ///
    /// This is useful if e.g. you want to provide a branch instruction, where control flow continues in one of the two branches
    Continuation,
}
type WasmGeneratorResult = Result<WasmGeneratorOutput, WasmGeneratorError>;

fn compile_wasm(
    builder: &mut InstrSeqBuilder,
    instructions: impl IntoIterator<Item = WasmGeneratorDirective>,
) -> Result<(), WasmGeneratorError> {
    let (flattened_instructions, _) = flatten_continuations(instructions, Default::default())?;
    assemble_wasm(builder, flattened_instructions);
    Ok(())
}

#[derive(Debug, Clone)]
enum WasmInstruction {
    Instruction(Instr),
    Branch {
        block_type: InstrSeqType,
        consequent: Vec<WasmInstruction>,
        alternative: Vec<WasmInstruction>,
    },
}
impl TryFrom<WasmGeneratorDirective> for WasmInstruction {
    type Error = WasmGeneratorError;
    fn try_from(value: WasmGeneratorDirective) -> Result<Self, Self::Error> {
        match value {
            WasmGeneratorDirective::Instruction(instruction) => Ok(Self::Instruction(instruction)),
            WasmGeneratorDirective::Branch {
                block_type,
                consequent,
                alternative,
            } => Ok(Self::Branch {
                block_type,
                consequent: consequent
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
                alternative: alternative
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            WasmGeneratorDirective::Continuation => Err(WasmGeneratorError::InvalidContinuation),
        }
    }
}

fn assemble_wasm(
    builder: &mut InstrSeqBuilder,
    instructions: impl IntoIterator<Item = WasmInstruction>,
) {
    for instruction in instructions {
        match instruction {
            WasmInstruction::Instruction(instruction) => {
                builder.instr(instruction);
            }
            WasmInstruction::Branch {
                block_type,
                consequent,
                alternative,
            } => {
                let consequent_block_id = {
                    let mut block_builder = builder.dangling_instr_seq(block_type);
                    assemble_wasm(&mut block_builder, consequent);
                    block_builder.id()
                };
                let alternate_block_id = {
                    let mut block_builder = builder.dangling_instr_seq(block_type);
                    assemble_wasm(&mut block_builder, alternative);
                    block_builder.id()
                };
                builder.instr(ir::IfElse {
                    consequent: consequent_block_id,
                    alternative: alternate_block_id,
                });
            }
        }
    }
}

fn flatten_continuations(
    instructions: impl IntoIterator<Item = WasmGeneratorDirective>,
    mut continuation: VecDeque<WasmGeneratorDirective>,
) -> Result<(Vec<WasmInstruction>, VecDeque<WasmGeneratorDirective>), WasmGeneratorError> {
    let mut results = Vec::new();
    let mut remaining_instructions = instructions.into_iter().collect::<VecDeque<_>>();
    while let Some(instruction) = remaining_instructions.pop_front() {
        match instruction {
            WasmGeneratorDirective::Instruction(instruction) => {
                results.push(WasmInstruction::Instruction(instruction))
            }
            WasmGeneratorDirective::Branch {
                block_type,
                consequent,
                alternative,
            } => {
                let consequent = {
                    let continuation = std::mem::take(&mut remaining_instructions);
                    let (block_instructions, continuation) =
                        flatten_continuations(consequent, continuation)?;
                    remaining_instructions = continuation;
                    block_instructions
                };
                let alternative = {
                    let continuation = std::mem::take(&mut remaining_instructions);
                    let (block_instructions, continuation) =
                        flatten_continuations(alternative, continuation)?;
                    remaining_instructions = continuation;
                    block_instructions
                };
                results.push(WasmInstruction::Branch {
                    block_type,
                    consequent,
                    alternative,
                })
            }
            WasmGeneratorDirective::Continuation => {
                let continuation = std::mem::take(&mut continuation);
                let (instructions, _) = flatten_continuations(continuation, Default::default())?;
                results.extend(instructions);
            }
        }
    }
    Ok((results, continuation))
}

trait GenerateWasm {
    fn emit_wasm(
        &self,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult;
}

impl GenerateWasm for CompiledBlock {
    fn emit_wasm(
        &self,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        let mut instructions = WasmGeneratorOutput::default();
        for instruction in self.instructions.iter() {
            instructions.append_block(instruction.emit_wasm(module, bindings, options)?);
        }
        Ok(instructions)
    }
}

impl GenerateWasm for CompiledInstruction {
    fn emit_wasm(
        &self,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) -> WasmGeneratorResult {
        match self {
            Self::Const(value) => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::Const {
                    value: match value {
                        ConstValue::I32(value) => Value::I32(*value),
                        ConstValue::U32(value) => Value::I32(from_twos_complement_i32(*value)),
                        ConstValue::I64(value) => Value::I64(*value),
                        ConstValue::U64(value) => Value::I64(from_twos_complement_i64(*value)),
                        ConstValue::F32(value) => Value::F32(*value),
                        ConstValue::F64(value) => Value::F64(*value),
                        ConstValue::HeapPointer(value) => {
                            Value::I32(from_twos_complement_i32(u32::from(*value)))
                        }
                        ConstValue::FunctionPointer(value) => {
                            let function_index = match value {
                                FunctionPointer::Stdlib(target) => Ok(bindings
                                    .export_mappings
                                    .stdlib
                                    .get_indirect_call_function_index(*target)),
                                FunctionPointer::Lambda(target_hash) => bindings
                                    .compiled_function_mappings
                                    .get_indirect_call_function_index(*target_hash)
                                    .ok_or_else(|| {
                                        WasmGeneratorError::InvalidCompiledFunction(*target_hash)
                                    }),
                            }?;
                            Value::I32(from_twos_complement_i32(u32::from(function_index)))
                        }
                    },
                });
                Ok(instructions)
            }
            Self::ReadHeapValue(value_type) => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::Load {
                    memory: bindings.memory_id,
                    kind: match value_type {
                        ValueType::I32
                        | ValueType::U32
                        | ValueType::HeapPointer
                        | ValueType::FunctionPointer => LoadKind::I32 {
                            atomic: Default::default(),
                        },
                        ValueType::I64 | ValueType::U64 => LoadKind::I64 {
                            atomic: Default::default(),
                        },
                        ValueType::F32 => LoadKind::F32,
                        ValueType::F64 => LoadKind::F64,
                    },
                    arg: MemArg {
                        align: Default::default(),
                        offset: Default::default(),
                    },
                });
                Ok(instructions)
            }
            Self::WriteHeapValue(value_type) => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::Store {
                    memory: bindings.memory_id,
                    kind: match value_type {
                        ValueType::I32
                        | ValueType::U32
                        | ValueType::HeapPointer
                        | ValueType::FunctionPointer => StoreKind::I32 {
                            atomic: Default::default(),
                        },
                        ValueType::I64 | ValueType::U64 => StoreKind::I64 {
                            atomic: Default::default(),
                        },
                        ValueType::F32 => StoreKind::F32,
                        ValueType::F64 => StoreKind::F64,
                    },
                    arg: MemArg {
                        align: Default::default(),
                        offset: Default::default(),
                    },
                });
                Ok(instructions)
            }
            Self::Duplicate(_) => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::LocalTee {
                    local: bindings.temp_id,
                });
                instructions.push(ir::LocalGet {
                    local: bindings.temp_id,
                });
                Ok(instructions)
            }
            Self::Drop(_) => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::Drop {});
                Ok(instructions)
            }
            Self::ScopeStart(value_type) => {
                let scope_local_id = module.locals.add(parse_value_type(*value_type));
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::LocalSet {
                    local: scope_local_id,
                });
                bindings.enter_scope(scope_local_id);
                Ok(instructions)
            }
            Self::ScopeEnd(_) => match bindings.leave_scope() {
                Some(_) => Ok(Default::default()),
                None => Err(WasmGeneratorError::StackError),
            },
            Self::GetScopeValue { scope_offset, .. } => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::LocalGet {
                    local: bindings.get_local(*scope_offset)?,
                });
                Ok(instructions)
            }
            Self::Select(value_type) => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::Select {
                    ty: Some(parse_value_type(*value_type)),
                });
                Ok(instructions)
            }
            Self::If {
                block_type,
                consequent,
                alternative,
            } => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.branch(
                    block_type,
                    consequent.emit_wasm(module, bindings, options)?,
                    alternative.emit_wasm(module, bindings, options)?,
                    module,
                    bindings,
                    options,
                );
                Ok(instructions)
            }
            Self::ConditionalBreak {
                block_type,
                handler,
            } => {
                // If a signal was encountered, we need to prevent further execution of the current block.
                // This is achieved via an if-else block, whose consequent represents the short-circuit return, and
                // whose alternative represents the continuation of the current block.
                let mut instructions = WasmGeneratorOutput::default();
                // Pop the boolean from the stack and continue with either the early-return block or the continuation
                // branch as appropriate
                instructions.branch(
                    // Create a type signature for the if-else branches, where the params encode the number of preceding
                    // items to capture from the operand stack, and whose results correspond to the result type of the
                    // handler block (which must be the same as the result type of the enclosing block), with an
                    // additional term pointer for the accumulated dependencies
                    &{
                        let TypeSignature { params, results } = block_type;
                        let dependency_list_type = ValueType::HeapPointer;
                        TypeSignature {
                            params: params.clone(),
                            results: ParamsSignature::from_iter(
                                results.iter().chain(once(dependency_list_type)),
                            ),
                        }
                    },
                    // Construct the early-return block
                    {
                        let mut short_circuit_block = WasmGeneratorOutput::default();
                        // Invoke the handler instructions
                        short_circuit_block
                            .append_block(handler.emit_wasm(module, bindings, options)?);
                        // Push the accumulated dependencies onto the stack before terminating the block
                        short_circuit_block.push(ir::LocalGet {
                            local: bindings.dependencies_id,
                        });
                        short_circuit_block
                    },
                    // Construct the continuation branch
                    {
                        let mut continuation_block = WasmGeneratorOutput::default();
                        // Insertion point for remaining instructions (the captured block input values remain on top of the stack)
                        continuation_block.continuation();
                        continuation_block
                    },
                    module,
                    bindings,
                    options,
                );
                Ok(instructions)
            }
            Self::Eq(value_type) => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::Binop {
                    op: match value_type {
                        ValueType::I32
                        | ValueType::U32
                        | ValueType::HeapPointer
                        | ValueType::FunctionPointer => BinaryOp::I32Eq,
                        ValueType::I64 | ValueType::U64 => BinaryOp::I64Eq,
                        ValueType::F32 => BinaryOp::F32Eq,
                        ValueType::F64 => BinaryOp::F64Eq,
                    },
                });
                Ok(instructions)
            }
            Self::NullPointer => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::GlobalGet {
                    global: bindings.export_mappings.globals.null_pointer,
                });
                Ok(instructions)
            }
            Self::LoadStateValue => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::LocalGet {
                    local: bindings.state_id,
                });
                instructions.push(ir::Call {
                    func: bindings.export_mappings.builtins.get_state_value,
                });
                instructions.push(ir::LocalGet {
                    local: bindings.dependencies_id,
                });
                instructions.push(ir::Call {
                    func: bindings.export_mappings.builtins.combine_dependencies,
                });
                instructions.push(ir::LocalSet {
                    local: bindings.dependencies_id,
                });
                Ok(instructions)
            }
            Self::CallRuntimeBuiltin(builtin) => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::Call {
                    func: bindings.export_mappings.builtins.get(*builtin),
                });
                Ok(instructions)
            }
            Self::CallStdlib(target) => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::LocalGet {
                    local: bindings.state_id,
                });
                instructions.push(ir::Call {
                    func: bindings.export_mappings.stdlib.get_function_id(*target),
                });
                instructions.push(ir::LocalGet {
                    local: bindings.dependencies_id,
                });
                instructions.push(ir::Call {
                    func: bindings.export_mappings.builtins.combine_dependencies,
                });
                instructions.push(ir::LocalSet {
                    local: bindings.dependencies_id,
                });
                Ok(instructions)
            }
            Self::CallCompiledFunction { target, .. } => {
                match bindings.compiled_function_mappings.get_function_id(*target) {
                    None => Err(WasmGeneratorError::InvalidCompiledFunction(*target)),
                    Some(function_id) => {
                        let mut instructions = WasmGeneratorOutput::default();
                        instructions.push(ir::LocalGet {
                            local: bindings.state_id,
                        });
                        instructions.push(ir::Call { func: function_id });
                        instructions.push(ir::LocalGet {
                            local: bindings.dependencies_id,
                        });
                        instructions.push(ir::Call {
                            func: bindings.export_mappings.builtins.combine_dependencies,
                        });
                        instructions.push(ir::LocalSet {
                            local: bindings.dependencies_id,
                        });
                        Ok(instructions)
                    }
                }
            }
            Self::CallDynamic(type_signature) => {
                let (params, results) = parse_function_type_signature(type_signature);
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::LocalGet {
                    local: bindings.state_id,
                });
                instructions.push(ir::CallIndirect {
                    ty: module.types.add(&params, &results),
                    table: bindings.main_function_table_id,
                });
                instructions.push(ir::LocalGet {
                    local: bindings.dependencies_id,
                });
                instructions.push(ir::Call {
                    func: bindings.export_mappings.builtins.combine_dependencies,
                });
                instructions.push(ir::LocalSet {
                    local: bindings.dependencies_id,
                });
                Ok(instructions)
            }
            Self::Evaluate => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::LocalGet {
                    local: bindings.state_id,
                });
                instructions.push(ir::Call {
                    func: bindings.export_mappings.builtins.evaluate,
                });
                instructions.push(ir::LocalGet {
                    local: bindings.dependencies_id,
                });
                instructions.push(ir::Call {
                    func: bindings.export_mappings.builtins.combine_dependencies,
                });
                instructions.push(ir::LocalSet {
                    local: bindings.dependencies_id,
                });
                Ok(instructions)
            }
            Self::Apply => {
                let mut instructions = WasmGeneratorOutput::default();
                instructions.push(ir::LocalGet {
                    local: bindings.state_id,
                });
                instructions.push(ir::Call {
                    func: bindings.export_mappings.builtins.apply,
                });
                instructions.push(ir::LocalGet {
                    local: bindings.dependencies_id,
                });
                instructions.push(ir::Call {
                    func: bindings.export_mappings.builtins.combine_dependencies,
                });
                instructions.push(ir::LocalSet {
                    local: bindings.dependencies_id,
                });
                Ok(instructions)
            }
        }
    }
}
