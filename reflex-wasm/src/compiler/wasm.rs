// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::HashMap;

use crate::{
    compiler::{
        instruction::core::Block, runtime::builtin::RuntimeBuiltin, CompiledBlock,
        CompiledFunctionId, ParamsSignature, TypeSignature, ValueType,
    },
    stdlib::Stdlib,
    utils::from_twos_complement_i32,
    FunctionIndex,
};

use reflex::core::{Arity, StackOffset};
use reflex_utils::Stack;
use walrus::{
    ir::{self, BinaryOp, Instr, InstrSeqId, InstrSeqType},
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
    InvalidCompiledFunction(CompiledFunctionId),
    InvalidBlockOffset(usize),
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
    body: CompiledBlock,
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
    let function_id = {
        // Create the function signature
        let mut builder = FunctionBuilder::new(&mut module.types, &params, &results);
        // Create the function body
        let mut function_body = builder.func_body();
        // Embed the function pre-instructions
        function_body
            // Initialize the dependencies local to the null pointer
            .global_get(export_mappings.globals.null_pointer)
            .local_set(dependencies_id);
        // Embed the compiled function body
        {
            // Create a control flow block to wrap the compiled function body
            // (this allows any signals encountered within the function to short-circuit by breaking out of the block).
            let block = Block {
                block_type: TypeSignature {
                    params: ParamsSignature::Void,
                    results: ParamsSignature::Single(ValueType::HeapPointer),
                },
                body,
            };
            // Generate the WASM instructions for the block
            let instructions = block.emit_wasm(module, &mut bindings, options)?;
            // Inject the generated instructions into the function body
            let _ = assemble_wasm(&mut function_body, Default::default(), instructions)?;
        };
        // Embed the function post-instructions
        function_body
            // Append the value of the dependencies local to the function body return value
            .local_get(dependencies_id);
        // Add the function to the WASM module
        builder.finish(args, &mut module.funcs)
    };
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
            Self::InvalidCompiledFunction(id) => write!(f, "Invalid compiled function ID: {id}"),
            Self::InvalidBlockOffset(target) => write!(f, "Invalid parent block offset: {target}"),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct WasmCompiledFunctionMappings {
    mappings: HashMap<CompiledFunctionId, (FunctionId, FunctionIndex, Arity)>,
}
impl WasmCompiledFunctionMappings {
    pub fn insert(
        &mut self,
        target: CompiledFunctionId,
        function_id: FunctionId,
        function_index: FunctionIndex,
        arity: Arity,
    ) {
        self.mappings
            .insert(target, (function_id, function_index, arity));
    }
    pub fn get_function_id(&self, target: CompiledFunctionId) -> Option<FunctionId> {
        self.mappings
            .get(&target)
            .map(|(function_id, _, _)| *function_id)
    }
    pub fn get_indirect_call_function_index(
        &self,
        target: CompiledFunctionId,
    ) -> Option<FunctionIndex> {
        self.mappings
            .get(&target)
            .map(|(_, function_index, _)| *function_index)
    }
    pub fn get_function_arity(&self, target: CompiledFunctionId) -> Option<Arity> {
        self.mappings.get(&target).map(|(_, _, num_args)| *num_args)
    }
    pub fn iter(
        &self,
    ) -> std::collections::hash_map::Iter<CompiledFunctionId, (FunctionId, FunctionIndex, Arity)>
    {
        self.mappings.iter()
    }
}

pub struct WasmGeneratorBindings<'a> {
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
    /// Lookup table mapping term hashes to compiled function ids
    compiled_function_mappings: &'a WasmCompiledFunctionMappings,
    /// Struct containing IDs of runtime builtin functions
    export_mappings: &'a RuntimeExportMappings,
    /// Locals currrently accessible on the lexical scope stack (this list will grow and shrink as new lexical scopes are created and disposed)
    stack: Vec<LocalId>,
}

impl<'a> WasmGeneratorBindings<'a> {
    pub fn memory_id(&self) -> MemoryId {
        self.memory_id
    }
    pub fn main_function_table_id(&self) -> TableId {
        self.main_function_table_id
    }
    pub fn state_id(&self) -> LocalId {
        self.state_id
    }
    pub fn dependencies_id(&self) -> LocalId {
        self.dependencies_id
    }
    pub fn temp_id(&self) -> LocalId {
        self.temp_id
    }
    pub fn null_pointer(&self) -> GlobalId {
        self.export_mappings.globals.null_pointer
    }
    pub fn builtins(&self) -> &RuntimeBuiltinMappings {
        &self.export_mappings.builtins
    }
    pub fn get_stdlib_function_id(&self, target: Stdlib) -> FunctionId {
        self.export_mappings.stdlib.get_function_id(target)
    }
    pub fn get_stdlib_indirect_call_function_index(&self, target: Stdlib) -> FunctionIndex {
        self.export_mappings
            .stdlib
            .get_indirect_call_function_index(target)
    }
    pub fn get_compiled_function_id(&self, target: CompiledFunctionId) -> Option<FunctionId> {
        self.compiled_function_mappings.get_function_id(target)
    }
    pub fn get_compiled_function_indirect_call_function_index(
        &self,
        target: CompiledFunctionId,
    ) -> Option<FunctionIndex> {
        self.compiled_function_mappings
            .get_indirect_call_function_index(target)
    }
    pub fn enter_scope(&mut self, local_id: LocalId) -> LocalId {
        self.stack.push(local_id);
        local_id
    }
    pub fn leave_scope(&mut self) -> Option<LocalId> {
        self.stack.pop()
    }
    pub fn get_local(&self, offset: StackOffset) -> Result<LocalId, WasmGeneratorError> {
        if offset < self.stack.len() {
            Ok(self.stack[self.stack.len() - 1 - offset])
        } else {
            Err(WasmGeneratorError::StackError)
        }
    }
}

pub fn parse_block_type_signature(
    signature: &TypeSignature,
    types: &mut ModuleTypes,
) -> InstrSeqType {
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

pub fn parse_function_type_signature(signature: &TypeSignature) -> (Vec<ValType>, Vec<ValType>) {
    let TypeSignature { params, results } = signature;
    let params = params.iter().map(parse_value_type).collect::<Vec<_>>();
    let results = results.iter().map(parse_value_type).collect::<Vec<_>>();
    (params, results)
}

pub fn parse_value_type(ty: ValueType) -> ValType {
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
pub struct WasmGeneratorOutput {
    instructions: Vec<WasmInstruction>,
}

impl WasmGeneratorOutput {
    pub fn push(&mut self, instruction: impl Into<Instr>) {
        self.instructions
            .push(WasmInstruction::Instruction(instruction.into()));
    }
    pub fn block(
        &mut self,
        block_type: &TypeSignature,
        body: WasmGeneratorOutput,
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
                    WasmInstruction::Instruction(ir::Instr::LocalGet(ir::LocalGet {
                        local: param_id,
                    }))
                }));
            let block_type = parse_block_type_signature(
                &TypeSignature {
                    params: ParamsSignature::Void,
                    results: block_type.results.clone(),
                },
                &mut module.types,
            );
            let body = {
                let mut instructions = block_header;
                instructions.push_chunk(body);
                instructions
            };
            // Emit the rewritten branch instruction
            self.instructions
                .push(WasmInstruction::Block { block_type, body });
        } else {
            // Otherwise if we are not manually capturing any stack values, emit the branch instruction as-is
            let block_type = parse_block_type_signature(block_type, &mut module.types);
            self.instructions
                .push(WasmInstruction::Block { block_type, body });
        };
    }
    pub fn br(&mut self, target_block: usize) {
        self.instructions
            .push(WasmInstruction::Break { target_block });
    }
    pub fn br_if(&mut self, target_block: usize) {
        self.instructions
            .push(WasmInstruction::ConditionalBreak { target_block });
    }
    pub fn if_else(
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
                    WasmInstruction::Instruction(ir::Instr::LocalGet(ir::LocalGet {
                        local: param_id,
                    }))
                }));
            let block_type = parse_block_type_signature(
                &TypeSignature {
                    params: ParamsSignature::Void,
                    results: block_type.results.clone(),
                },
                &mut module.types,
            );
            let (consequent_header, alternative_header) = (block_header.clone(), block_header);
            let consequent = {
                let mut instructions = consequent_header;
                instructions.push_chunk(consequent);
                instructions
            };
            let alternative = {
                let mut instructions = alternative_header;
                instructions.push_chunk(alternative);
                instructions
            };
            // Emit the rewritten branch instruction
            self.instructions.push(WasmInstruction::IfElse {
                block_type,
                consequent,
                alternative,
            });
        } else {
            // Otherwise if we are not manually capturing any stack values, emit the branch instruction as-is
            let block_type = parse_block_type_signature(block_type, &mut module.types);
            self.instructions.push(WasmInstruction::IfElse {
                block_type,
                consequent,
                alternative,
            });
        };
    }
    pub fn iter(&self) -> std::slice::Iter<'_, WasmInstruction> {
        self.instructions.iter()
    }
    fn push_chunk(&mut self, chunk: WasmGeneratorOutput) {
        self.instructions.extend(chunk);
    }
}

impl FromIterator<WasmInstruction> for WasmGeneratorOutput {
    fn from_iter<T: IntoIterator<Item = WasmInstruction>>(iter: T) -> Self {
        Self {
            instructions: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for WasmGeneratorOutput {
    type Item = WasmInstruction;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}

impl<'a> IntoIterator for &'a WasmGeneratorOutput {
    type Item = &'a WasmInstruction;
    type IntoIter = std::slice::Iter<'a, WasmInstruction>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Extend<WasmInstruction> for WasmGeneratorOutput {
    fn extend<T: IntoIterator<Item = WasmInstruction>>(&mut self, iter: T) {
        self.instructions.extend(iter)
    }
}

#[derive(Debug, Clone)]
pub enum WasmInstruction {
    /// Standard WASM instruction
    Instruction(Instr),
    /// Control flow block
    Block {
        block_type: InstrSeqType,
        body: WasmGeneratorOutput,
    },
    /// Unconditional break instruction
    Break {
        /// Index of the block to break out of (where `0` is the current block, `1` is the immediate parent of the current block, etc)
        target_block: usize,
    },
    /// Conditional break instruction
    ConditionalBreak {
        /// Index of the block to break out of (where `0` is the current block, `1` is the immediate parent of the current block, etc)
        target_block: usize,
    },
    /// If-else conditional branch instruction
    IfElse {
        block_type: InstrSeqType,
        consequent: WasmGeneratorOutput,
        alternative: WasmGeneratorOutput,
    },
}

pub type WasmGeneratorResult = Result<WasmGeneratorOutput, WasmGeneratorError>;

#[must_use]
fn assemble_wasm(
    builder: &mut InstrSeqBuilder,
    enclosing_blocks: Stack<InstrSeqId>,
    instructions: impl IntoIterator<Item = WasmInstruction>,
) -> Result<(), WasmGeneratorError> {
    instructions
        .into_iter()
        .fold(Ok(()), |result, instruction| {
            let _ = result?;
            match instruction {
                WasmInstruction::Instruction(instruction) => {
                    builder.instr(instruction);
                    Ok(())
                }
                WasmInstruction::Block { block_type, body } => {
                    let block_id = {
                        let mut block_builder = builder.dangling_instr_seq(block_type);
                        let block_id = block_builder.id();
                        assemble_wasm(&mut block_builder, enclosing_blocks.push(block_id), body)?;
                        block_id
                    };
                    builder.instr(ir::Block { seq: block_id });
                    Ok(())
                }
                WasmInstruction::Break { target_block } => {
                    let block_id = enclosing_blocks
                        .rev()
                        .copied()
                        .skip(target_block)
                        .next()
                        .ok_or_else(|| WasmGeneratorError::InvalidBlockOffset(target_block))?;
                    builder.instr(ir::Br { block: block_id });
                    Ok(())
                }
                WasmInstruction::ConditionalBreak { target_block } => {
                    let block_id = enclosing_blocks
                        .rev()
                        .copied()
                        .skip(target_block)
                        .next()
                        .ok_or_else(|| WasmGeneratorError::InvalidBlockOffset(target_block))?;
                    builder.instr(ir::BrIf { block: block_id });
                    Ok(())
                }
                WasmInstruction::IfElse {
                    block_type,
                    consequent,
                    alternative,
                } => {
                    let consequent_block_id = {
                        let body = consequent;
                        let mut block_builder = builder.dangling_instr_seq(block_type);
                        let block_id = block_builder.id();
                        assemble_wasm(&mut block_builder, enclosing_blocks.push(block_id), body)?;
                        block_id
                    };
                    let alternative_block_id = {
                        let body = alternative;
                        let mut block_builder = builder.dangling_instr_seq(block_type);
                        let block_id = block_builder.id();
                        assemble_wasm(&mut block_builder, enclosing_blocks.push(block_id), body)?;
                        block_id
                    };
                    builder.instr(ir::IfElse {
                        consequent: consequent_block_id,
                        alternative: alternative_block_id,
                    });
                    Ok(())
                }
            }
        })
}

pub trait GenerateWasm {
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
            instructions.push_chunk(instruction.emit_wasm(module, bindings, options)?);
        }
        Ok(instructions)
    }
}
