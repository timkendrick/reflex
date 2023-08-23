// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

use crate::{
    compiler::{
        instruction::core::Block, runtime::builtin::RuntimeBuiltin, CompiledBlock,
        CompiledFunctionId, FunctionPointer, ParamsSignature, TypeSignature, ValueType,
    },
    stdlib::Stdlib,
    utils::from_twos_complement_i32,
    FunctionIndex,
};

use reflex::core::{Arity, StackOffset};
use reflex_utils::Stack;
use walrus::{
    ir, ExportItem, FunctionBuilder, FunctionId, GlobalId, Import, ImportKind, InstrSeqBuilder,
    LocalId, MemoryId, Module, TableId, ValType,
};

use super::{
    expand_function_args::expand_function_args,
    import_function::import_function,
    types::{parse_block_type_signature, parse_value_type},
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
    pub create_dependency_tree: FunctionId,
    pub create_empty_list: FunctionId,
    pub create_effect: FunctionId,
    pub create_float: FunctionId,
    pub create_hashset: FunctionId,
    pub create_int: FunctionId,
    pub create_lambda: FunctionId,
    pub create_lazy_result: FunctionId,
    pub create_nil: FunctionId,
    pub create_partial: FunctionId,
    pub create_pointer: FunctionId,
    pub create_record: FunctionId,
    pub create_signal: FunctionId,
    pub create_symbol: FunctionId,
    pub create_timestamp: FunctionId,
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
    pub get_boolean_value: FunctionId,
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
            RuntimeBuiltin::CreateDependencyTree => self.create_dependency_tree,
            RuntimeBuiltin::CreateEmptyList => self.create_empty_list,
            RuntimeBuiltin::CreateEffect => self.create_effect,
            RuntimeBuiltin::CreateFloat => self.create_float,
            RuntimeBuiltin::CreateHashset => self.create_hashset,
            RuntimeBuiltin::CreateInt => self.create_int,
            RuntimeBuiltin::CreateLambda => self.create_lambda,
            RuntimeBuiltin::CreateLazyResult => self.create_lazy_result,
            RuntimeBuiltin::CreateNil => self.create_nil,
            RuntimeBuiltin::CreatePartial => self.create_partial,
            RuntimeBuiltin::CreatePointer => self.create_pointer,
            RuntimeBuiltin::CreateRecord => self.create_record,
            RuntimeBuiltin::CreateSignal => self.create_signal,
            RuntimeBuiltin::CreateSymbol => self.create_symbol,
            RuntimeBuiltin::CreateTimestamp => self.create_timestamp,
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
            RuntimeBuiltin::GetBooleanValue => self.get_boolean_value,
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
    pub collect_constructor: FunctionId,
    pub collect_hashmap: FunctionId,
    pub collect_hashset: FunctionId,
    pub collect_list: FunctionId,
    pub collect_record: FunctionId,
    pub collect_signal: FunctionId,
    pub collect_string: FunctionId,
    pub collect_tree: FunctionId,
    pub cons: FunctionId,
    pub construct: FunctionId,
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
    pub intersperse: FunctionId,
    pub is_finite: FunctionId,
    pub is_truthy: FunctionId,
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
    pub resolve_loader_results: FunctionId,
    pub resolve_query_branch: FunctionId,
    pub resolve_query_leaf: FunctionId,
    pub resolve_record: FunctionId,
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
            Stdlib::CollectConstructor(_) => self.collect_constructor,
            Stdlib::CollectHashmap(_) => self.collect_hashmap,
            Stdlib::CollectHashset(_) => self.collect_hashset,
            Stdlib::CollectList(_) => self.collect_list,
            Stdlib::CollectRecord(_) => self.collect_record,
            Stdlib::CollectSignal(_) => self.collect_signal,
            Stdlib::CollectString(_) => self.collect_string,
            Stdlib::CollectTree(_) => self.collect_tree,
            Stdlib::Cons(_) => self.cons,
            Stdlib::Construct(_) => self.construct,
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
            Stdlib::Intersperse(_) => self.intersperse,
            Stdlib::IsFinite(_) => self.is_finite,
            Stdlib::IsTruthy(_) => self.is_truthy,
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
            Stdlib::ResolveLoaderResults(_) => self.resolve_loader_results,
            Stdlib::ResolveQueryBranch(_) => self.resolve_query_branch,
            Stdlib::ResolveQueryLeaf(_) => self.resolve_query_leaf,
            Stdlib::ResolveRecord(_) => self.resolve_record,
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
    InvalidModuleImport {
        expected: Import,
        received: Option<ExportItem>,
    },
    InvalidBlockOffset(usize),
}

/// Create a function with the given signature within the provided module.
///
/// The provided body factory function will be used to build the function body based on the function's argument locals
pub(crate) fn generate_function(
    module: &mut Module,
    params: &[ValType],
    results: &[ValType],
    body: impl FnOnce(&[LocalId], &mut InstrSeqBuilder) -> (),
) -> FunctionId {
    let mut builder = FunctionBuilder::new(&mut module.types, params, results);
    let arg_ids = params
        .iter()
        .copied()
        .map(|arg_type| module.locals.add(arg_type))
        .collect::<Vec<_>>();
    let mut function_body = builder.func_body();
    body(&arg_ids, &mut function_body);
    builder.finish(arg_ids, &mut module.funcs)
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
        dependencies_id: NonEmptyStack::new(dependencies_id),
        temp_id,
        stack: arg_stack,
        enclosing_blocks: Default::default(),
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
            // Initialize the dependencies local
            .call(export_mappings.builtins.create_dependency_tree)
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
            // Generate the WASM instructions within the control flow block
            let instructions = {
                // Enter the control flow block
                bindings.enclosing_blocks.push(WasmBlockType::Explicit);
                // Generate the WASM instructions within the block context
                let compiled_instructions = block.emit_wasm(module, &mut bindings, options);
                // Leave the control flow block
                bindings.enclosing_blocks.pop();
                // Return the generated instructions
                compiled_instructions
            }?;
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

/// Create a wrapper function that invokes the provided function and memoizes the results for all unique combinations of
/// input arguments via a a globally-shared runtime cache
pub(crate) fn generate_cached_function_wrapper(
    module: &mut Module,
    function_identifier: FunctionPointer,
    params: ParamsSignature,
    compiled_function_id: FunctionId,
    function_wrapper_template: (&mut Module, FunctionId),
) -> Result<FunctionId, anyhow::Error> {
    let (template_module, template_function_id) = function_wrapper_template;
    let template_args_placeholder_id = template_module
        .imports
        .iter()
        .find_map(
            |import| match (&import.kind, import.module.as_str(), import.name.as_str()) {
                (ImportKind::Function(function_id), "$", "__ARGS") => Some(*function_id),
                _ => None,
            },
        )
        .ok_or_else(|| anyhow::anyhow!("Unable to locate __ARGS import in template source"))?;
    // Create a new temporary function within the template module that prepends a parameter for each argument,
    // replacing any calls to the argument placeholder function within the template function body with a sequence of
    // instructions that pushes all the argument values onto the operand stack
    let expanded_template_function_id = expand_function_args(
        template_module,
        template_function_id,
        template_args_placeholder_id,
        params.iter().map(parse_value_type),
    )
    .map_err(|err| anyhow::anyhow!("{}", err))?;
    // Generate a temporary placeholder function in the target module that will be used in the template
    // to represent pushing each function argument in turn onto the operand stack (this is used as a marker to locate
    // argument insertion points, and will be removed from the target module once the arguments have been injected)
    let args_placeholder_id = generate_noop_function(module);
    // Generate a function in the target module that can be used to hash the provided function arguments
    // (this is used to obtain a unique invocation hash that represents calling this function with a
    // specific combination of arguments)
    let function_hash_id = generate_function_arg_hasher(
        module,
        function_identifier,
        &params,
        get_exported_hash_functions(module)?,
    )?;
    // Get the set of template variables to be injected into the template
    let substitutions = get_cached_function_template_substitutions(
        template_module,
        module,
        compiled_function_id,
        function_hash_id,
        args_placeholder_id,
    )?;
    // Import the substituted template into the target module
    let wrapper_id = import_function(
        template_module,
        expanded_template_function_id,
        module,
        substitutions,
        [template_function_id, expanded_template_function_id],
    )
    .map_err(|err| anyhow::anyhow!("{}", err))?;
    // Remove the temporary parameter-expanded function from the template module
    if expanded_template_function_id != template_function_id {
        template_module.funcs.delete(expanded_template_function_id);
    }
    // Remove the temporary argument placholder function from the target function
    module.funcs.delete(args_placeholder_id);
    // Return the function ID of the completed wrapper function
    Ok(wrapper_id)
}

fn get_exported_hash_functions(module: &Module) -> Result<HashFunctionIds, anyhow::Error> {
    fn get_exported_function(
        export_name: &str,
        exported_functions: &HashMap<&str, FunctionId>,
    ) -> Result<FunctionId, anyhow::Error> {
        exported_functions
            .get(&export_name)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Invalid module export: {export_name:?}"))
    }
    let exported_functions = module
        .exports
        .iter()
        .filter_map(|export| match &export.item {
            ExportItem::Function(function_id) => Some((export.name.as_str(), *function_id)),
            _ => None,
        })
        .collect::<HashMap<_, _>>();
    Ok(HashFunctionIds {
        write_term: get_exported_function("writeTermHash", &exported_functions)?,
        write_i32: get_exported_function("writeI32Hash", &exported_functions)?,
        write_i64: get_exported_function("writeI64Hash", &exported_functions)?,
        write_f32: get_exported_function("writeF32Hash", &exported_functions)?,
        write_f64: get_exported_function("writeF64Hash", &exported_functions)?,
    })
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
            .binop(ir::BinaryOp::I32LtU)
            .if_else(
                ir::InstrSeqType::new(&mut module.types, &[], &results),
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

fn generate_noop_function(module: &mut Module) -> FunctionId {
    generate_function(module, &[], &[], |_args, _builder| {})
}

struct HashFunctionIds {
    write_term: FunctionId,
    write_i32: FunctionId,
    write_i64: FunctionId,
    write_f32: FunctionId,
    write_f64: FunctionId,
}
impl HashFunctionIds {
    fn get(&self, value_type: ValueType) -> Option<FunctionId> {
        match value_type {
            ValueType::I32 | ValueType::U32 => Some(self.write_i32),
            ValueType::I64 | ValueType::U64 => Some(self.write_i64),
            ValueType::F32 => Some(self.write_f32),
            ValueType::F64 => Some(self.write_f64),
            ValueType::HeapPointer => Some(self.write_term),
            ValueType::FunctionPointer => Some(self.write_i32),
        }
    }
}

fn generate_function_arg_hasher(
    module: &mut Module,
    function_identifier: FunctionPointer,
    params: &ParamsSignature,
    hash_function_ids: HashFunctionIds,
) -> Result<FunctionId, anyhow::Error> {
    // Generate a unique value to disambiguate calls to this function from calls to other functions called with identical arguments
    let function_hash = {
        let mut hasher = DefaultHasher::default();
        function_identifier.hash(&mut hasher);
        hasher.finish()
    };
    // Parse the function parameter types, for each one getting the corresponding WASM type
    // and the corresponding WASM function used to hash that type
    let (arg_types, arg_hashers): (Vec<_>, Vec<_>) = params
        .iter()
        .map(|arg_type| {
            hash_function_ids
                .get(arg_type)
                .ok_or_else(|| anyhow::anyhow!("Unsupported function argument type: {arg_type:?}"))
                .map(|hasher_id| (parse_value_type(arg_type), hasher_id))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .unzip();
    // Generate a function that accepts a hash state and argument values, and returns a corresponding function invocation hash
    Ok(generate_function(
        module,
        &([ValType::I64]
            .into_iter()
            .chain(arg_types)
            .collect::<Vec<_>>()),
        &[ValType::I64],
        move |arg_ids, builder| {
            let args_with_hashers = arg_ids.iter().copied().skip(1).zip(arg_hashers);
            args_with_hashers.fold(
                builder
                    // Start with the provided hash state
                    .local_get(arg_ids[0])
                    // Write the unique function hash to the hash state
                    .i64_const(function_hash as i64)
                    .call(hash_function_ids.write_i64),
                // Write the hash for each argument
                |builder, (arg_id, arg_hasher)| builder.local_get(arg_id).call(arg_hasher),
            );
        },
    ))
}

fn get_cached_function_template_substitutions<'a>(
    template_module: &'a Module,
    target_module: &Module,
    function_id: FunctionId,
    function_hash_id: FunctionId,
    function_args_id: FunctionId,
) -> Result<impl IntoIterator<Item = (&'a str, ExportItem)>, WasmGeneratorError> {
    // In addition to certain predefined templated variables, templates can import any of the target module's named
    // exports (this is indicated within the template by importing from the custom "$" module namespace)
    // TODO: establish separate namespaces within templates for a) template variables, b) exports, c) private members (identified via debug names)
    let target_module_exports = target_module
        .exports
        .iter()
        .map(|export| (export.name.as_str(), export))
        .collect::<HashMap<_, _>>();
    template_module
        .imports
        .iter()
        .filter(|import| is_template_variable_import(import))
        .map(|import| {
            let import_name = import.name.as_str();
            match (&import.kind, import_name) {
                // Predefined template variables
                (ImportKind::Function(_), "__INNER") => Ok(ExportItem::Function(function_id)),
                (ImportKind::Function(_), "__HASH") => Ok(ExportItem::Function(function_hash_id)),
                (ImportKind::Function(_), "__ARGS") => Ok(ExportItem::Function(function_args_id)),
                // Fall back to importing from target module
                (import_kind, import_name) => match target_module_exports.get(import_name).copied()
                {
                    None => Err(None),
                    Some(export) => match (import_kind, &export.item) {
                        (ImportKind::Function(_), ExportItem::Function(target_id)) => {
                            Ok(ExportItem::Function(*target_id))
                        }
                        (ImportKind::Table(_), ExportItem::Table(target_id)) => {
                            Ok(ExportItem::Table(*target_id))
                        }
                        (ImportKind::Memory(_), ExportItem::Memory(target_id)) => {
                            Ok(ExportItem::Memory(*target_id))
                        }
                        (ImportKind::Global(_), ExportItem::Global(target_id)) => {
                            Ok(ExportItem::Global(*target_id))
                        }
                        _ => Err(Some(export.item)),
                    },
                },
            }
            .map(|exported_value| (import_name, exported_value))
            .map_err(|exported_value| WasmGeneratorError::InvalidModuleImport {
                expected: import.clone(),
                received: exported_value,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

fn is_template_variable_import(import: &Import) -> bool {
    import.module.as_str() == "$"
}

impl std::error::Error for WasmGeneratorError {}

impl std::fmt::Display for WasmGeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StackError => write!(f, "Invalid stack access"),
            Self::InvalidCompiledFunction(id) => write!(f, "Invalid compiled function ID: {id}"),
            Self::InvalidModuleImport { expected, received } => write!(
                f,
                "Invalid module import: Expected {expected:?}, received {received:?}"
            ),
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
    /// Stack of variables to keep track of any dependencies encountered during this evaluation
    dependencies_id: NonEmptyStack<LocalId>,
    /// Temporary free-use register
    temp_id: LocalId,
    /// Lookup table mapping term hashes to compiled function ids
    compiled_function_mappings: &'a WasmCompiledFunctionMappings,
    /// Struct containing IDs of runtime builtin functions
    export_mappings: &'a RuntimeExportMappings,
    /// Locals currrently accessible on the lexical scope stack (this list will grow and shrink as new lexical scopes are created and disposed)
    stack: Vec<LocalId>,
    /// Stack of parent blocks
    enclosing_blocks: Vec<WasmBlockType>,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum WasmBlockType {
    /// Block that has been explicitly created via a `block` or `loop` instruction
    Explicit,
    /// Block that has been implicitly created via e.g. an `else` / `then` instruction
    Implicit,
}

struct NonEmptyStack<T> {
    /// The value currently at the top of the stack
    current: T,
    /// Stack of parent values
    parents: Vec<T>,
}

impl<T> NonEmptyStack<T> {
    fn new(current: T) -> Self {
        Self {
            current,
            parents: Default::default(),
        }
    }
    fn push(&mut self, value: T) {
        let existing_value = std::mem::replace(&mut self.current, value);
        self.parents.push(existing_value);
    }
    fn pop(&mut self) -> Option<T> {
        let parent_value = self.parents.pop()?;
        let existing_value = std::mem::replace(&mut self.current, parent_value);
        Some(existing_value)
    }
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
        self.dependencies_id.current
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
    pub fn enter_dependencies_scope(&mut self, local_id: LocalId) -> LocalId {
        self.dependencies_id.push(local_id);
        self.enter_scope(local_id)
    }
    pub fn leave_scope(&mut self) -> Option<LocalId> {
        let local_id = self.stack.pop()?;
        if self.dependencies_id.current == local_id {
            self.dependencies_id.pop();
        }
        Some(local_id)
    }
    pub fn enter_block(&mut self, block_type: WasmBlockType) {
        self.enclosing_blocks.push(block_type);
    }
    pub fn leave_block(&mut self) -> Option<WasmBlockType> {
        self.enclosing_blocks.pop()
    }
    pub fn get_target_block_offset(&self, target_block: usize) -> Option<usize> {
        let blocks = &self.enclosing_blocks;
        let num_blocks = blocks.len();
        let num_hidden_blocks = (0..=target_block).fold(Some(0), |result, target_block| {
            let mut num_hidden_blocks = result?;
            while let WasmBlockType::Implicit = {
                if target_block + num_hidden_blocks >= num_blocks {
                    None
                } else {
                    self.enclosing_blocks
                        .get(num_blocks - 1 - (target_block + num_hidden_blocks))
                }
            }? {
                num_hidden_blocks += 1;
            }
            Some(num_hidden_blocks)
        })?;
        Some(target_block + num_hidden_blocks)
    }
    pub fn get_local(&self, offset: StackOffset) -> Result<LocalId, WasmGeneratorError> {
        if offset < self.stack.len() {
            Ok(self.stack[self.stack.len() - 1 - offset])
        } else {
            Err(WasmGeneratorError::StackError)
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct WasmGeneratorOutput {
    instructions: Vec<WasmInstruction>,
}

impl WasmGeneratorOutput {
    pub fn push(&mut self, instruction: impl Into<ir::Instr>) {
        self.instructions
            .push(WasmInstruction::Instruction(instruction.into()));
    }
    #[must_use]
    pub fn block(
        &mut self,
        block_type: &TypeSignature,
        body: &impl GenerateWasm,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) -> Result<(), WasmGeneratorError> {
        let instructions = generate_control_flow_instructions(
            &WasmBlockControlFlowGenerator { body },
            block_type,
            module,
            bindings,
            options,
        )?;
        self.instructions.extend(instructions);
        Ok(())
    }
    #[must_use]
    pub fn r#loop(
        &mut self,
        block_type: &TypeSignature,
        body: &impl GenerateWasm,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) -> Result<(), WasmGeneratorError> {
        let instructions = generate_control_flow_instructions(
            &WasmLoopControlFlowGenerator { body },
            block_type,
            module,
            bindings,
            options,
        )?;
        self.instructions.extend(instructions);
        Ok(())
    }
    #[must_use]
    pub fn if_else<T: GenerateWasm>(
        &mut self,
        block_type: &TypeSignature,
        consequent: &T,
        alternative: &T,
        module: &mut Module,
        bindings: &mut WasmGeneratorBindings,
        options: &WasmGeneratorOptions,
    ) -> Result<(), WasmGeneratorError> {
        let instructions = generate_control_flow_instructions(
            &WasmIfElseControlFlowGenerator {
                consequent,
                alternative,
            },
            block_type,
            module,
            bindings,
            options,
        )?;
        self.instructions.extend(instructions);
        Ok(())
    }
    #[must_use]
    pub fn br(
        &mut self,
        target_block: usize,
        bindings: &WasmGeneratorBindings,
    ) -> Result<(), WasmGeneratorError> {
        let target_block_offset = bindings
            .get_target_block_offset(target_block)
            .ok_or_else(|| WasmGeneratorError::InvalidBlockOffset(target_block))?;
        self.instructions.push(WasmInstruction::Break {
            target_block: target_block_offset,
        });
        Ok(())
    }
    #[must_use]
    pub fn br_if(
        &mut self,
        target_block: usize,
        bindings: &WasmGeneratorBindings,
    ) -> Result<(), WasmGeneratorError> {
        let target_block_offset = bindings
            .get_target_block_offset(target_block)
            .ok_or_else(|| WasmGeneratorError::InvalidBlockOffset(target_block))?;
        self.instructions.push(WasmInstruction::ConditionalBreak {
            target_block: target_block_offset,
        });
        Ok(())
    }
    pub fn iter(&self) -> std::slice::Iter<'_, WasmInstruction> {
        self.instructions.iter()
    }
    fn push_chunk(&mut self, chunk: WasmGeneratorOutput) {
        self.instructions.extend(chunk);
    }
}

/// Generic helper used when generating structured control flow that combines one or more child blocks
trait WasmControlFlowGenerator<T: GenerateWasm, const N: usize> {
    /// Produce a list of child branches to be compiled within their own control flow blocks
    fn blocks(&self) -> [(WasmBlockType, &T); N];
    /// Combine the compiled child branches into the current control flow block
    fn generate(
        &self,
        block_type: ir::InstrSeqType,
        compiled_blocks: [WasmGeneratorResult; N],
    ) -> WasmGeneratorResult;
}

struct WasmBlockControlFlowGenerator<'a, T: GenerateWasm> {
    body: &'a T,
}

impl<'a, T: GenerateWasm> WasmControlFlowGenerator<T, 1> for WasmBlockControlFlowGenerator<'a, T> {
    fn blocks(&self) -> [(WasmBlockType, &T); 1] {
        [(WasmBlockType::Explicit, self.body)]
    }
    fn generate(
        &self,
        block_type: ir::InstrSeqType,
        compiled_blocks: [WasmGeneratorResult; 1],
    ) -> WasmGeneratorResult {
        let [body] = compiled_blocks;
        let body = body?;
        let mut instructions = WasmGeneratorOutput::default();
        instructions
            .instructions
            .push(WasmInstruction::Block { block_type, body });
        Ok(instructions)
    }
}

struct WasmLoopControlFlowGenerator<'a, T: GenerateWasm> {
    body: &'a T,
}

impl<'a, T: GenerateWasm> WasmControlFlowGenerator<T, 1> for WasmLoopControlFlowGenerator<'a, T> {
    fn blocks(&self) -> [(WasmBlockType, &T); 1] {
        [(WasmBlockType::Explicit, self.body)]
    }
    fn generate(
        &self,
        block_type: ir::InstrSeqType,
        compiled_blocks: [WasmGeneratorResult; 1],
    ) -> WasmGeneratorResult {
        let [body] = compiled_blocks;
        let body = body?;
        let mut instructions = WasmGeneratorOutput::default();
        instructions
            .instructions
            .push(WasmInstruction::Loop { block_type, body });
        Ok(instructions)
    }
}

struct WasmIfElseControlFlowGenerator<'a, T: GenerateWasm> {
    consequent: &'a T,
    alternative: &'a T,
}

impl<'a, T: GenerateWasm> WasmControlFlowGenerator<T, 2> for WasmIfElseControlFlowGenerator<'a, T> {
    fn blocks(&self) -> [(WasmBlockType, &T); 2] {
        [
            (WasmBlockType::Implicit, self.consequent),
            (WasmBlockType::Implicit, self.alternative),
        ]
    }
    fn generate(
        &self,
        block_type: ir::InstrSeqType,
        compiled_blocks: [WasmGeneratorResult; 2],
    ) -> WasmGeneratorResult {
        let [consequent, alternative] = compiled_blocks;
        let consequent = consequent?;
        let alternative = alternative?;
        let mut instructions = WasmGeneratorOutput::default();
        instructions.instructions.push(WasmInstruction::IfElse {
            block_type,
            consequent,
            alternative,
        });
        Ok(instructions)
    }
}

/// Generate structured control flow that combines one or more child blocks
fn generate_control_flow_instructions<T: GenerateWasm, const N: usize>(
    generator: &impl WasmControlFlowGenerator<T, N>,
    block_type: &TypeSignature,
    module: &mut Module,
    bindings: &mut WasmGeneratorBindings,
    options: &WasmGeneratorOptions,
) -> WasmGeneratorResult {
    let blocks = generator.blocks();
    let (prelude, block_type, block_header) = {
        // If the compiler output format does not support block input params, emulate this feature by using locals to
        // save the stack values before branching, then once within the child block we can load the stack values back
        // out from the locals (these temporary locals are typically compiled away in a later optimization pass)
        if options.disable_block_params && (block_type.params.len() > 0) {
            let mut prelude = WasmGeneratorOutput::default();
            // First create temporary locals to hold the stack values
            let param_ids = block_type
                .params
                .iter()
                .map(|param_type| module.locals.add(parse_value_type(param_type)))
                .collect::<Vec<_>>();
            // Temporarily pop the branch condition into the temporary local
            prelude.push(ir::LocalSet {
                local: bindings.temp_id,
            });
            // Pop all the captured operand stack values into their respective locals
            for param_id in param_ids.iter().rev() {
                prelude.push(ir::LocalSet { local: *param_id });
            }
            // Push the branch condition back onto the operand stack
            prelude.push(ir::LocalGet {
                local: bindings.temp_id,
            });
            let block_type = parse_block_type_signature(
                &TypeSignature {
                    params: ParamsSignature::Void,
                    results: block_type.results.clone(),
                },
                &mut module.types,
            );
            // Prepare the child block header that pushes the captured values back onto the operand stack
            let block_header = Some(WasmGeneratorOutput::from_iter(param_ids.into_iter().map(
                |param_id| {
                    WasmInstruction::Instruction(ir::Instr::LocalGet(ir::LocalGet {
                        local: param_id,
                    }))
                },
            )));
            (Some(prelude), block_type, block_header)
        } else {
            // Otherwise if we are not manually capturing any stack values, emit the branch instruction as-is
            let prelude = None;
            let block_type = parse_block_type_signature(block_type, &mut module.types);
            let block_header = None;
            (prelude, block_type, block_header)
        }
    };
    let compiled_blocks = blocks.map(|(block_type, body)| {
        // Compile the block body within the correct block scope
        let body_instructions = {
            bindings.enter_block(block_type);
            let instructions = body.emit_wasm(module, bindings, options);
            bindings.leave_block();
            instructions
        }?;
        // Prepend the block body with the block header
        let block_instructions = if let Some(block_header) = block_header.as_ref() {
            let mut block_instructions = block_header.clone();
            block_instructions.push_chunk(body_instructions);
            block_instructions
        } else {
            body_instructions
        };
        Ok(block_instructions)
    });
    // Emit the top-level instructions
    let mut instructions = prelude.unwrap_or_default();
    let compiled_instructions = generator.generate(block_type, compiled_blocks)?;
    instructions.push_chunk(compiled_instructions);
    Ok(instructions)
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
    Instruction(ir::Instr),
    /// Control flow block
    Block {
        block_type: ir::InstrSeqType,
        body: WasmGeneratorOutput,
    },
    /// Control flow loop
    Loop {
        block_type: ir::InstrSeqType,
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
        block_type: ir::InstrSeqType,
        consequent: WasmGeneratorOutput,
        alternative: WasmGeneratorOutput,
    },
}

pub type WasmGeneratorResult = Result<WasmGeneratorOutput, WasmGeneratorError>;

#[must_use]
fn assemble_wasm(
    builder: &mut InstrSeqBuilder,
    enclosing_blocks: Stack<ir::InstrSeqId>,
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
                WasmInstruction::Loop { block_type, body } => {
                    let block_id = {
                        let mut block_builder = builder.dangling_instr_seq(block_type);
                        let block_id = block_builder.id();
                        assemble_wasm(&mut block_builder, enclosing_blocks.push(block_id), body)?;
                        block_id
                    };
                    builder.instr(ir::Loop { seq: block_id });
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
