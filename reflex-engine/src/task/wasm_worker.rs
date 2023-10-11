// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    iter::once,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
    str::FromStr,
    sync::Arc,
    time::Instant,
};

use metrics::histogram;
use reflex::{
    core::{
        Arity, ConditionListType, ConditionType, DependencyList, EvaluationResult, Expression,
        ExpressionFactory, HeapAllocator, NodeId, RefType, SignalTermType, SignalType, StateToken,
        StringValue,
    },
    hash::{HashId, IntMap, IntSet},
};
use reflex_dispatcher::{
    Action, ActorEvents, HandlerContext, MessageData, MessageOffset, NoopDisposeCallback,
    ProcessId, SchedulerCommand, SchedulerMode, SchedulerTransition, TaskFactory, TaskInbox,
};
use reflex_macros::{blanket_trait, dispatcher, Named};
use reflex_runtime::{
    action::bytecode_interpreter::{
        BytecodeInterpreterEvaluateAction, BytecodeInterpreterGcAction,
        BytecodeInterpreterInitAction, BytecodeInterpreterResultAction,
    },
    action::bytecode_interpreter::{BytecodeInterpreterGcCompleteAction, BytecodeWorkerStatistics},
    AsyncExpression, AsyncExpressionFactory, AsyncHeapAllocator, QueryEvaluationMode,
};
use reflex_utils::{
    dag::{
        reporter::{CounterDagReporter, NoopDagReporter},
        DagEdgeDirection, DagReporter, DagVisitor, IntDag,
    },
    Visitable,
};
use reflex_wasm::{
    allocator::{Arena, ArenaAllocator, ArenaMut, VecAllocator},
    cache::{EvaluationCache, EvaluationCacheBucket},
    factory::WasmTermFactory,
    interpreter::{InterpreterError, UnboundEvaluationResult, WasmInterpreter, WasmProgram},
    serialize::SerializerState,
    term_type::{
        symbol::SymbolTerm, ApplicationTerm, CellTerm, ConditionTerm, HashmapTerm, ListTerm,
        PointerTerm, TermType, TreeTerm, TypedTerm, WasmExpression,
    },
    wasmtime::Val,
    ArenaPointer, ArenaPointerIterator, ArenaRef, FunctionIndex, Term,
};
use serde::{Deserialize, Serialize};

use crate::task::bytecode_worker::BytecodeWorkerAction;

const EFFECT_TYPE_CACHE: &str = "reflex::cache";

/// Criteria governing whether to dump the state of the heap at the point of evaluation
#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WasmHeapDumpMode {
    evaluation_type: WasmHeapDumpEvaluationType,
    result_type: WasmHeapDumpResultType,
}

impl WasmHeapDumpMode {
    pub fn new(
        evaluation_type: WasmHeapDumpEvaluationType,
        result_type: WasmHeapDumpResultType,
    ) -> Self {
        Self {
            evaluation_type,
            result_type,
        }
    }
    fn should_dump_heap<T: Expression, E>(
        &self,
        query_type: QueryEvaluationMode,
        result: Result<&T, E>,
        factory: &impl ExpressionFactory<T>,
    ) -> bool {
        let query_type_match = match (self.evaluation_type, query_type) {
            (WasmHeapDumpEvaluationType::All, _) => true,
            (WasmHeapDumpEvaluationType::Query, QueryEvaluationMode::Query) => true,
            _ => false,
        };
        if !query_type_match {
            return false;
        }
        match self.result_type {
            WasmHeapDumpResultType::All => true,
            WasmHeapDumpResultType::Error => match result {
                Err(_) => true,
                Ok(result) => is_error_result(result, factory),
            },
            WasmHeapDumpResultType::Pending => match result {
                Err(_) => true,
                Ok(result) => is_pending_result(result, factory),
            },
            WasmHeapDumpResultType::Result => match result {
                Err(_) => true,
                Ok(result) => !is_unresolved_result(result, factory),
            },
        }
    }
}

impl FromStr for WasmHeapDumpMode {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "all" => Ok(WasmHeapDumpMode::new(
                WasmHeapDumpEvaluationType::All,
                WasmHeapDumpResultType::All,
            )),
            "error" => Ok(WasmHeapDumpMode::new(
                WasmHeapDumpEvaluationType::All,
                WasmHeapDumpResultType::Error,
            )),
            "pending" => Ok(WasmHeapDumpMode::new(
                WasmHeapDumpEvaluationType::All,
                WasmHeapDumpResultType::Pending,
            )),
            "result" => Ok(WasmHeapDumpMode::new(
                WasmHeapDumpEvaluationType::All,
                WasmHeapDumpResultType::Result,
            )),
            "query-all" => Ok(WasmHeapDumpMode::new(
                WasmHeapDumpEvaluationType::Query,
                WasmHeapDumpResultType::All,
            )),
            "query-error" => Ok(WasmHeapDumpMode::new(
                WasmHeapDumpEvaluationType::Query,
                WasmHeapDumpResultType::Error,
            )),
            "query-pending" => Ok(WasmHeapDumpMode::new(
                WasmHeapDumpEvaluationType::Query,
                WasmHeapDumpResultType::Pending,
            )),
            "query-result" => Ok(WasmHeapDumpMode::new(
                WasmHeapDumpEvaluationType::Query,
                WasmHeapDumpResultType::Result,
            )),
            _ => Err(format!("Unrecognized heap dump mode: {}", input)),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WasmHeapDumpEvaluationType {
    /// Dump heap only for top-level queries
    Query,
    /// Dump heap for all queries and sub-queries
    All,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum WasmHeapDumpResultType {
    /// Dump heap on error results
    Error,
    /// Dump heap on pending results
    Pending,
    /// Dump heap on all results
    Result,
    /// Dump heap on intermediate evaluations
    All,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WasmWorkerMetricNames {
    pub query_worker_compile_duration: Cow<'static, str>,
    pub query_worker_evaluate_duration: Cow<'static, str>,
    pub query_worker_gc_duration: Cow<'static, str>,
}

blanket_trait!(
    pub trait WasmWorkerTaskAction<T: Expression>: BytecodeWorkerAction<T> {}
);

pub trait WasmWorkerTask<T, TFactory, TAllocator>:
    From<WasmWorkerTaskFactory<T, TFactory, TAllocator>>
where
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
{
}
impl<_Self, T, TFactory, TAllocator> WasmWorkerTask<T, TFactory, TAllocator> for _Self
where
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
    Self: From<WasmWorkerTaskFactory<T, TFactory, TAllocator>>,
{
}

#[derive(Named, Clone, Serialize, Deserialize)]
#[serde(bound(
    serialize = "T: Serialize, <T as Expression>::Signal: Serialize",
    deserialize = "T: Deserialize<'de>, <T as Expression>::Signal: Deserialize<'de>"
))]
pub struct WasmWorkerTaskFactory<
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    pub cache_key: T::Signal,
    pub query: T,
    pub graph_root_factory_export_name: String,
    pub evaluation_mode: QueryEvaluationMode,
    pub wasm_module: Arc<WasmProgram>,
    pub metric_names: WasmWorkerMetricNames,
    pub caller_pid: ProcessId,
    pub dump_heap_snapshot: Option<WasmHeapDumpMode>,
    pub _expression: PhantomData<T>,
    pub _factory: PhantomData<TFactory>,
    pub _allocator: PhantomData<TAllocator>,
}

impl<T, TFactory, TAllocator, TAction, TTask> TaskFactory<TAction, TTask>
    for WasmWorkerTaskFactory<T, TFactory, TAllocator>
where
    T: AsyncExpression,
    TFactory: AsyncExpressionFactory<T> + Default,
    TAllocator: AsyncHeapAllocator<T> + Default,
    T::Builtin: Into<reflex_wasm::stdlib::Stdlib>,
    TAction: Action + WasmWorkerAction<T> + Send + 'static,
    TTask: TaskFactory<TAction, TTask>,
{
    type Actor = WasmWorker<T, TFactory, TAllocator>;
    fn create(self) -> Self::Actor {
        let Self {
            cache_key,
            query,
            graph_root_factory_export_name,
            evaluation_mode,
            wasm_module,
            metric_names,
            caller_pid,
            dump_heap_snapshot,
            _expression,
            _factory,
            _allocator,
        } = self;
        let factory = TFactory::default();
        let allocator = TAllocator::default();
        WasmWorker {
            cache_key,
            query,
            graph_root_factory_export_name,
            evaluation_mode,
            wasm_module,
            factory,
            allocator,
            metric_names,
            caller_pid,
            dump_heap_snapshot,
        }
    }
}

#[derive(Named, Clone, Debug)]
pub struct WasmWorker<T, TFactory, TAllocator>
where
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
{
    cache_key: T::Signal,
    query: T,
    graph_root_factory_export_name: String,
    evaluation_mode: QueryEvaluationMode,
    wasm_module: Arc<WasmProgram>,
    factory: TFactory,
    allocator: TAllocator,
    metric_names: WasmWorkerMetricNames,
    caller_pid: ProcessId,
    dump_heap_snapshot: Option<WasmHeapDumpMode>,
}

pub enum WasmWorkerState<T: Expression> {
    Uninitialized,
    Initialized(WasmWorkerInitializedState<T>),
    Error(WasmWorkerError<T>),
}

impl<T: Expression> Default for WasmWorkerState<T> {
    fn default() -> Self {
        Self::Uninitialized
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct EvaluationCacheKey(HashId);

impl reflex::hash::IsEnabled for EvaluationCacheKey {}

impl EvaluationCacheKey {
    fn key(&self) -> HashId {
        let Self(inner) = self;
        *inner
    }
}

impl From<HashId> for EvaluationCacheKey {
    fn from(value: HashId) -> Self {
        Self(value)
    }
}

impl From<EvaluationCacheKey> for HashId {
    fn from(value: EvaluationCacheKey) -> Self {
        let EvaluationCacheKey(value) = value;
        value
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct EvaluationResultDependencyTreeId(HashId);

impl reflex::hash::IsEnabled for EvaluationResultDependencyTreeId {}

impl EvaluationResultDependencyTreeId {
    fn id(&self) -> HashId {
        let Self(inner) = self;
        *inner
    }
}

impl<'a, A: Arena> From<&'a ArenaRef<TypedTerm<TreeTerm>, A>> for EvaluationResultDependencyTreeId {
    fn from(value: &'a ArenaRef<TypedTerm<TreeTerm>, A>) -> Self {
        Self(value.read_value(|term| term.id()))
    }
}

impl From<EvaluationResultDependencyTreeId> for HashId {
    fn from(value: EvaluationResultDependencyTreeId) -> Self {
        let EvaluationResultDependencyTreeId(value) = value;
        value
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct EvaluationResultDependencyTreePointer(ArenaPointer);

impl EvaluationResultDependencyTreePointer {
    fn as_pointer(&self) -> ArenaPointer {
        let Self(inner) = self;
        *inner
    }
    fn as_arena_ref<A: Arena>(&self, arena: A) -> ArenaRef<TypedTerm<TreeTerm>, A> {
        ArenaRef::<TypedTerm<TreeTerm>, _>::new(arena, self.as_pointer())
    }
}

impl From<ArenaPointer> for EvaluationResultDependencyTreePointer {
    fn from(value: ArenaPointer) -> Self {
        Self(value)
    }
}

impl From<EvaluationResultDependencyTreePointer> for ArenaPointer {
    fn from(value: EvaluationResultDependencyTreePointer) -> Self {
        let EvaluationResultDependencyTreePointer(value) = value;
        value
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct EvaluationCacheGlobalPointer(ArenaPointer);

impl EvaluationCacheGlobalPointer {
    fn as_pointer(&self) -> ArenaPointer {
        let Self(inner) = self;
        *inner
    }
    fn as_arena_ref<A: Arena>(&self, arena: A) -> ArenaRef<TypedTerm<PointerTerm>, A> {
        ArenaRef::<TypedTerm<PointerTerm>, _>::new(arena, self.as_pointer())
    }
}

impl From<ArenaPointer> for EvaluationCacheGlobalPointer {
    fn from(value: ArenaPointer) -> Self {
        Self(value)
    }
}

impl From<EvaluationCacheGlobalPointer> for ArenaPointer {
    fn from(value: EvaluationCacheGlobalPointer) -> Self {
        let EvaluationCacheGlobalPointer(value) = value;
        value
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct EvaluationCacheCellPointer(ArenaPointer);

impl EvaluationCacheCellPointer {
    fn as_pointer(&self) -> ArenaPointer {
        let Self(inner) = self;
        *inner
    }
    fn as_arena_ref<A: Arena>(&self, arena: A) -> ArenaRef<TypedTerm<CellTerm>, A> {
        ArenaRef::<TypedTerm<CellTerm>, _>::new(arena, self.as_pointer())
    }
}

impl From<ArenaPointer> for EvaluationCacheCellPointer {
    fn from(value: ArenaPointer) -> Self {
        Self(value)
    }
}

impl From<EvaluationCacheCellPointer> for ArenaPointer {
    fn from(value: EvaluationCacheCellPointer) -> Self {
        let EvaluationCacheCellPointer(value) = value;
        value
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct EvaluationCacheInstancePointer(ArenaPointer);

impl EvaluationCacheInstancePointer {
    fn as_pointer(&self) -> ArenaPointer {
        let Self(inner) = self;
        *inner
    }
    fn as_arena_ref<A: Arena>(&self, arena: A) -> ArenaRef<EvaluationCache, A> {
        ArenaRef::<EvaluationCache, _>::new(arena, self.as_pointer())
    }
}

impl From<ArenaPointer> for EvaluationCacheInstancePointer {
    fn from(value: ArenaPointer) -> Self {
        Self(value)
    }
}

impl From<EvaluationCacheInstancePointer> for ArenaPointer {
    fn from(value: EvaluationCacheInstancePointer) -> Self {
        let EvaluationCacheInstancePointer(value) = value;
        value
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct EvaluationCacheNumEntriesPointer(ArenaPointer);

impl EvaluationCacheNumEntriesPointer {
    fn as_pointer(&self) -> ArenaPointer {
        let Self(inner) = self;
        *inner
    }
}

impl From<ArenaPointer> for EvaluationCacheNumEntriesPointer {
    fn from(value: ArenaPointer) -> Self {
        Self(value)
    }
}

impl From<EvaluationCacheNumEntriesPointer> for ArenaPointer {
    fn from(value: EvaluationCacheNumEntriesPointer) -> Self {
        let EvaluationCacheNumEntriesPointer(value) = value;
        value
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash, Debug)]
struct EvaluationCacheBucketPointer(ArenaPointer);

impl reflex::hash::IsEnabled for EvaluationCacheBucketPointer {}

impl EvaluationCacheBucketPointer {
    fn as_pointer(&self) -> ArenaPointer {
        let Self(inner) = self;
        *inner
    }
    fn as_arena_ref<A: Arena>(&self, arena: A) -> ArenaRef<EvaluationCacheBucket, A> {
        ArenaRef::<EvaluationCacheBucket, _>::new(arena, self.as_pointer())
    }
}

impl From<ArenaPointer> for EvaluationCacheBucketPointer {
    fn from(value: ArenaPointer) -> Self {
        Self(value)
    }
}

impl From<EvaluationCacheBucketPointer> for ArenaPointer {
    fn from(value: EvaluationCacheBucketPointer) -> Self {
        let EvaluationCacheBucketPointer(value) = value;
        value
    }
}

pub struct WasmWorkerInitializedState<T: Expression> {
    instance: WasmInterpreter,
    // Snapshot of linear memory before any evaluation takes place
    // (this is effectively the 'static' portion of linear memory and can be assumed to be immutable,
    // with the notable exception of the global evaluation cache pointer which is mutable and must be manually updated)
    initial_heap_snapshot: Vec<u8>,
    // Arity metadata for for all dynamically-callable compiled function wrappers
    indirect_call_arity: HashMap<FunctionIndex, Arity>,
    // Linear memory address of the (mutable) global pointer term that points to the cell term that holds the current evaluation cache instance
    evaluation_cache_global_pointer: EvaluationCacheGlobalPointer,
    // Linear memory address of the cell containing the empty evaluation cache instance
    evaluation_cache_initial_cell: EvaluationCacheCellPointer,
    // Linear memory address of the expression to evaluate
    entry_point: ArenaPointer,
    // ID of the most recent state update that has been processed by this worker
    state_index: Option<MessageOffset>,
    // Result of the most recent evaluation
    latest_result: Option<WasmWorkerEvaluationResult<T>>,
    // Mapping of condition IDs to the corresponding (key, value) term pointers allocated within linear memory
    state_values: IntMap<StateToken, (ArenaPointer, ArenaPointer)>,
}

#[derive(Debug)]
struct WasmWorkerEvaluationResult<T: Expression> {
    /// Parsed result and dependencies
    result: EvaluationResult<T>,
    /// Pointer to the overall value returned by the WASM runtime evaluation
    value: ArenaPointer,
    /// Pointer to the top-level dependency tree term returned by the WASM runtime evaluation
    dependencies: Option<EvaluationResultDependencyTreePointer>,
    /// Mapping from WASM condition ID to parsed condition term instance
    dependency_mappings: IntMap<DependencyGraphKey, T::Signal>,
    /// Mapping from exported condition ID to WASM condition ID
    state_dependencies: IntMap<StateToken, DependencyGraphKey>,
    invalidation_metadata: WasmWorkerInvalidationMetadata,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DependencyGraphKey {
    Tree(EvaluationResultDependencyTreeId),
    Cache(EvaluationCacheKey),
    State(StateToken),
}

impl std::hash::Hash for DependencyGraphKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(match self {
            // All variants are wrappers for either node IDs or cache keys,
            // all of which are hashes and assumed not to collide
            Self::Tree(dependency_id) => dependency_id.id(),
            Self::Cache(cache_key) => cache_key.key(),
            Self::State(state_token) => *state_token,
        })
    }
}

impl reflex::hash::IsEnabled for DependencyGraphKey {}

#[derive(Debug)]
struct WasmWorkerInvalidationMetadata {
    /// Pointer to the cell term containing current evaluation cache instance
    evaluation_cache_cell: EvaluationCacheCellPointer,
    /// Pointer to the current evaluation cache instance within the cell term
    evaluation_cache_instance: EvaluationCacheInstancePointer,
    /// Pointer to the hashmap entry count within the current evaluation cache instance
    evaluation_cache_num_entries: EvaluationCacheNumEntriesPointer,
    /// Mapping of all occupied buckets in the evaluation cache instance to the correpsponding dependency graph key
    evaluation_cache_bucket_dependency_keys:
        IntMap<EvaluationCacheBucketPointer, DependencyGraphKey>,
    /// Mapping from dependency graph key to the corresponding cache bucket pointer
    evaluation_cache_bucket_lookup: IntMap<EvaluationCacheKey, EvaluationCacheBucketPointer>,
    /// Graph of all stateful/memoized dependencies used for invalidating cached results
    dependency_graph: IntDag<DependencyGraphKey, ()>,
}

fn get_live_dependency_terms<'a, A: Arena + Clone>(
    arena: &'a A,
    dependencies: &'a ArenaRef<TreeTerm, A>,
    evaluation_cache_bucket_lookup: &'a IntMap<EvaluationCacheKey, EvaluationCacheBucketPointer>,
    state_values: &'a IntMap<StateToken, (ArenaPointer, ArenaPointer)>,
) -> impl Iterator<Item = ArenaPointer> + 'a {
    dependencies
        .typed_nodes::<ConditionTerm>()
        .flat_map(|dependency| {
            let (term, state_value) = if let Some(cache_key) = parse_cache_dependency(&dependency) {
                let bucket_pointer = evaluation_cache_bucket_lookup.get(&cache_key);
                match bucket_pointer {
                    Some(bucket_pointer) => {
                        let bucket = bucket_pointer.as_arena_ref(arena.clone());
                        (
                            Some(bucket.value().as_pointer()),
                            bucket
                                .dependencies()
                                .map(|dependencies| dependencies.as_pointer()),
                        )
                    }
                    _ => (None, None),
                }
            } else {
                let state_token = ConditionType::id(&dependency);
                match state_values.get(&state_token).copied() {
                    Some((key_pointer, value_pointer)) => (Some(key_pointer), Some(value_pointer)),
                    None => (None, None),
                }
            };
            term.into_iter().chain(state_value)
        })
}

fn gc_vm_heap<T: Expression>(cache_key: &T::Signal, state: &mut WasmWorkerInitializedState<T>) {
    match state.latest_result.as_mut() {
        None => {
            let arena = &mut state.instance;
            // Reset the global pointer address to point to the empty evaluation cache instance
            update_global_evaluation_cache_instance(
                arena,
                state.evaluation_cache_global_pointer,
                state.evaluation_cache_initial_cell,
            )
        }
        Some(existing_result) => {
            let source_arena = &state.instance;
            let source_value = existing_result.value;
            let source_dependencies = existing_result.dependencies;
            let WasmWorkerInvalidationMetadata {
                evaluation_cache_cell: source_evaluation_cache_cell,
                evaluation_cache_instance: source_evaluation_cache_instance,
                evaluation_cache_bucket_lookup: source_evaluation_cache_bucket_lookup,
                ..
            } = &existing_result.invalidation_metadata;
            // Get a set of all allocated terms that are cached as dependencies of the current evaluation
            let live_intermediate_terms = time_operation(
                cache_key.id(),
                "Determining live intermediate terms",
                || match source_dependencies {
                    None => IntSet::default(),
                    Some(source_dependencies) => {
                        let source_dependency_tree =
                            source_dependencies.as_arena_ref(source_arena.clone());
                        let source_dependency_tree = source_dependency_tree.as_inner();
                        get_live_dependency_terms(
                            &source_arena,
                            &source_dependency_tree,
                            source_evaluation_cache_bucket_lookup,
                            &state.state_values,
                        )
                        .collect::<IntSet<_>>()
                    }
                },
            );
            // Create a new linear memory from the initial heap snapshot
            let mut target_arena = VecAllocator::from_bytes(&state.initial_heap_snapshot);
            // Migrate all live terms from the existing heap to the new heap
            let mut serializer_state = SerializerState::new([], target_arena.end_offset());
            let (target_value, target_dependencies) =
                time_operation(cache_key.id(), "Copying live heap terms", || {
                    // Migrate the result value from the existing heap to the new heap
                    let target_value = copy_term(
                        source_value,
                        &source_arena,
                        &mut target_arena,
                        &mut serializer_state,
                    );
                    // Migrate the dependencies tree from the existing heap to the new heap
                    let target_dependencies = source_dependencies.map(|source_dependencies| {
                        copy_term(
                            source_dependencies.as_pointer(),
                            &source_arena,
                            &mut target_arena,
                            &mut serializer_state,
                        )
                    });
                    // Migrate any intermediate terms that are dependencies of the most recent evaluation
                    for source_term in live_intermediate_terms {
                        copy_term(
                            source_term,
                            &source_arena,
                            &mut target_arena,
                            &mut serializer_state,
                        );
                    }
                    (
                        target_value,
                        target_dependencies.map(EvaluationResultDependencyTreePointer::from),
                    )
                });
            // Regenerate the global evaluation cache instance
            let (target_value, target_dependencies, invalidation_metadata) =
                time_operation(cache_key.id(), "Migrating evaluation cache", || {
                    // Copy the evaluation cache instance from the existing heap to the new heap
                    // (the cell term will be copied verbatim as a block of opaque bytes, so any
                    // internal pointers will not be migrated to the target arena)
                    let target_evaluation_cache_cell = EvaluationCacheCellPointer::from(copy_term(
                        source_evaluation_cache_cell.as_pointer(),
                        &source_arena,
                        &mut target_arena,
                        &mut serializer_state,
                    ));
                    let target_evaluation_cache_instance = EvaluationCacheInstancePointer::from({
                        let cell_fields = target_evaluation_cache_cell
                            .as_arena_ref(&target_arena)
                            .as_inner()
                            .inner_ref(|term| &term.fields);
                        let instance_pointer = cell_fields.item_offset(0);
                        instance_pointer
                    });
                    // Iterate through the contents of the evaluation cache instance, migrating any
                    // internal pointers into the target arena, so that all cache buckets reference
                    // values allocated in the new heap
                    let bucket_migrations = get_occupied_evaluation_cache_buckets(
                        *source_evaluation_cache_instance,
                        &source_arena,
                    )
                    .map({
                        // Determine the delta between the pointers for the cache instance in the old arena
                        // vs the instance in the new arena
                        let evaluation_cache_instance_pointer_offset = {
                            let source_address =
                                u32::from(source_evaluation_cache_instance.as_pointer());
                            let target_address =
                                u32::from(target_evaluation_cache_instance.as_pointer());
                            // Offset can be negative, so need to convert to signed integer
                            (target_address as i64) - (source_address as i64)
                        };
                        move |(source_bucket_pointer, source_bucket_contents)| {
                            // Apply the pointer delta to the source bucket address to determine the target bucket address
                            let target_bucket_pointer =
                                EvaluationCacheBucketPointer::from(ArenaPointer::from(
                                    ((u32::from(source_bucket_pointer.as_pointer()) as i64)
                                        + evaluation_cache_instance_pointer_offset)
                                        as u32,
                                ));
                            (source_bucket_contents, target_bucket_pointer)
                        }
                    });
                    for (source_bucket_contents, target_bucket_pointer) in bucket_migrations {
                        // Copy the referenced value term from the old arena to the new arena
                        let (source_bucket_value, source_bucket_dependencies) = (
                            source_bucket_contents.value(),
                            source_bucket_contents.dependencies(),
                        );
                        let target_bucket_value = copy_term(
                            source_bucket_value.as_pointer(),
                            &source_arena,
                            &mut target_arena,
                            &mut serializer_state,
                        );
                        let target_bucket_dependencies =
                            source_bucket_dependencies.map(|source_bucket_dependencies| {
                                copy_term(
                                    source_bucket_dependencies.as_pointer(),
                                    &source_arena,
                                    &mut target_arena,
                                    &mut serializer_state,
                                )
                            });
                        // Overwrite the term pointer in the bucket's `value` and `dependencies` fields
                        let target_bucket = target_bucket_pointer.as_arena_ref(&target_arena);
                        let target_bucket_value_pointer =
                            target_bucket.inner_pointer(|bucket| &bucket.value);
                        let target_bucket_dependencies_pointer =
                            target_bucket.inner_pointer(|bucket| &bucket.dependencies);
                        target_arena.write::<ArenaPointer>(
                            target_bucket_value_pointer,
                            target_bucket_value,
                        );
                        target_arena.write::<ArenaPointer>(
                            target_bucket_dependencies_pointer,
                            target_bucket_dependencies.unwrap_or(ArenaPointer::null()),
                        );
                    }
                    // Update the global evaluation cache pointer address within the target arena to point to the migrated evaluation cache instance
                    update_global_evaluation_cache_instance(
                        &mut target_arena,
                        state.evaluation_cache_global_pointer,
                        target_evaluation_cache_cell,
                    );
                    // Regenerate the evaluation cache metadata based on the migrated term pointer addresses
                    let invalidation_metadata = {
                        let target_arena = &target_arena;
                        // Note that as well as updating the evaluation cache metadata, this will additionally regenerate
                        // the dependency graph, which isn't technically necessary as that doesn't contain any concrete
                        // pointer addresses (only dependency tree hashes, which are stable across reallocations)
                        // FIXME: Prevent unnecessarily regenerating dependency graph after garbage collection
                        compute_evaluation_cache_metadata(
                            &target_arena,
                            state.evaluation_cache_global_pointer,
                            target_dependencies,
                            None,
                        )
                    };
                    (target_value, target_dependencies, invalidation_metadata)
                });
            existing_result.value = target_value;
            existing_result.dependencies = target_dependencies;
            existing_result.invalidation_metadata = invalidation_metadata;
            time_operation(cache_key.id(), "Overwriting linear memory", || {
                // Overwrite the existing linear memory contents with the garbage-collected heap data,
                // zero-filling any reclaimed space
                let linear_memory = state.instance.data_mut();
                let compacted_memory = target_arena.into_bytes();
                let compacted_offset = serializer_state.end_offset();
                let compacted_size = u32::from(compacted_offset) as usize;
                linear_memory[0..compacted_size].clone_from_slice(&compacted_memory);
                linear_memory[compacted_size..].fill(0);
            });
        }
    }
}

fn copy_term<ASource: Arena + Clone, ADest: ArenaAllocator>(
    source_pointer: ArenaPointer,
    source_arena: &ASource,
    target_arena: &mut ADest,
    serializer_state: &mut SerializerState,
) -> ArenaPointer
where
    ArenaRef<Term, ASource>: Visitable<ArenaPointer> + NodeId,
{
    use reflex_wasm::serialize::Serialize;
    let term = ArenaRef::<Term, _>::new(source_arena.clone(), source_pointer);
    term.serialize(target_arena, serializer_state)
}

fn compute_evaluation_cache_metadata(
    arena: &(impl Arena + Clone),
    evaluation_cache_global_pointer: EvaluationCacheGlobalPointer,
    dependencies_pointer: Option<EvaluationResultDependencyTreePointer>,
    previous: Option<(
        Option<EvaluationResultDependencyTreePointer>,
        WasmWorkerInvalidationMetadata,
    )>,
) -> WasmWorkerInvalidationMetadata {
    let (evaluation_cache_cell, evaluation_cache_instance, evaluation_cache_num_entries) =
        get_evaluation_cache_instance(arena, evaluation_cache_global_pointer);
    // If the evaluation cache has been reallocated, rebuild the metadata from scratch
    // TODO: Rather than rebuilding from scratch after evaluation cache reallocations, first remap all the bucket pointers to the corresponding buckets in the updated cache instance and then proceed as usual with the dependency graph patching
    let previous = previous.filter(|(_, previous_invalidation_metadata)| {
        evaluation_cache_instance == previous_invalidation_metadata.evaluation_cache_instance
    });
    match previous {
        Some((previous_dependencies_pointer, previous_invalidation_metadata))
            if previous_dependencies_pointer == dependencies_pointer =>
        {
            previous_invalidation_metadata
        }
        previous => {
            let (
                evaluation_cache_bucket_dependency_keys,
                evaluation_cache_bucket_lookup,
                dependency_graph,
            ) = match previous {
                Some((
                    _previous_dependencies_pointer,
                    WasmWorkerInvalidationMetadata {
                        evaluation_cache_bucket_dependency_keys,
                        evaluation_cache_bucket_lookup,
                        dependency_graph,
                        ..
                    },
                )) => (
                    evaluation_cache_bucket_dependency_keys,
                    evaluation_cache_bucket_lookup,
                    dependency_graph,
                ),
                None => (Default::default(), Default::default(), Default::default()),
            };
            // Update the cached dependency metadata for any updated cache results
            let (evaluation_cache_bucket_dependency_keys, evaluation_cache_bucket_lookup) =
                get_occupied_evaluation_cache_buckets(evaluation_cache_instance, arena).fold(
                    (
                        evaluation_cache_bucket_dependency_keys,
                        evaluation_cache_bucket_lookup,
                    ),
                    |(
                        mut evaluation_cache_bucket_dependency_keys,
                        mut evaluation_cache_bucket_lookup,
                    ),
                     (bucket_pointer, bucket)| {
                        let bucket_cache_key = EvaluationCacheKey::from(bucket.key());
                        let graph_key = DependencyGraphKey::Cache(bucket_cache_key);
                        evaluation_cache_bucket_dependency_keys.insert(bucket_pointer, graph_key);
                        evaluation_cache_bucket_lookup.insert(bucket_cache_key, bucket_pointer);
                        (
                            evaluation_cache_bucket_dependency_keys,
                            evaluation_cache_bucket_lookup,
                        )
                    },
                );
            let dependency_graph = match dependencies_pointer {
                None => IntDag::default(),
                Some(dependencies_pointer) => {
                    let dependencies = dependencies_pointer.as_arena_ref(arena.clone());
                    parse_dependency_graph(&dependencies, Some(dependency_graph))
                }
            };
            WasmWorkerInvalidationMetadata {
                evaluation_cache_cell,
                evaluation_cache_instance,
                evaluation_cache_num_entries,
                evaluation_cache_bucket_dependency_keys,
                evaluation_cache_bucket_lookup,
                dependency_graph,
            }
        }
    }
}

fn parse_cache_dependency<'a, A: Arena + Clone>(
    condition: &'a ArenaRef<TypedTerm<ConditionTerm>, A>,
) -> Option<EvaluationCacheKey> {
    condition
        .as_inner()
        .as_custom_condition()
        .and_then(|condition| {
            let is_cache_dependency = condition
                .as_inner()
                .effect_type()
                .as_string_term()
                .map(|effect_type| effect_type.as_inner().as_str().deref() == EFFECT_TYPE_CACHE)
                .unwrap_or(false);
            if is_cache_dependency {
                condition
                    .as_inner()
                    .payload()
                    .as_int_term()
                    .map(|term| EvaluationCacheKey::from(term.as_inner().value() as u64))
            } else {
                None
            }
        })
}

fn parse_dependency_graph(
    dependencies: &ArenaRef<TypedTerm<TreeTerm>, impl Arena + Clone>,
    existing_graph: Option<IntDag<DependencyGraphKey, ()>>,
) -> IntDag<DependencyGraphKey, ()> {
    struct DependencyGraphVisitorQueueEntry<A: Arena> {
        node: ArenaRef<TypedTerm<TreeTerm>, A>,
        parent: Option<DependencyGraphKey>,
    }

    enum ParsedDependencyGraphBranch<A: Arena> {
        State(StateToken),
        Cache(EvaluationCacheKey),
        Tree(ArenaRef<TypedTerm<TreeTerm>, A>),
    }

    impl<'a, A: Arena> From<&'a ParsedDependencyGraphBranch<A>> for DependencyGraphKey {
        fn from(value: &'a ParsedDependencyGraphBranch<A>) -> Self {
            match value {
                ParsedDependencyGraphBranch::State(state_token) => {
                    DependencyGraphKey::State(*state_token)
                }
                ParsedDependencyGraphBranch::Cache(cache_key) => {
                    DependencyGraphKey::Cache(*cache_key)
                }
                ParsedDependencyGraphBranch::Tree(branch) => {
                    DependencyGraphKey::Tree(EvaluationResultDependencyTreeId::from(branch))
                }
            }
        }
    }

    let mut graph = existing_graph.unwrap_or_default();
    let mut queue = Vec::<DependencyGraphVisitorQueueEntry<_>>::with_capacity(
        dependencies.as_inner().depth() as usize,
    );
    queue.push(DependencyGraphVisitorQueueEntry {
        node: dependencies.clone(),
        parent: None,
    });
    while let Some(DependencyGraphVisitorQueueEntry { node, parent }) = queue.pop() {
        let node_key = DependencyGraphKey::Tree(EvaluationResultDependencyTreeId::from(&node));
        let already_processed = graph
            .add_node(node_key, Default::default(), CounterDagReporter::default())
            .added_nodes
            == 0;
        // Register the current node with its parent
        if let Some(parent_key) = parent {
            graph.add_edge(parent_key, node_key, NoopDagReporter);
        }
        // If this node's children have already been processed by a preceding visitor, nothing more to do
        if already_processed {
            continue;
        }
        let branch = node.as_inner();
        let left = parse_child_branch(branch.left().as_ref());
        let right = parse_child_branch(branch.right().as_ref());
        let (left, right, parent_key) = match (left, right) {
            // If this is a cached branch, create an intermediate cache node to use as the invalidation parent for the child branch
            (left, Some(ParsedDependencyGraphBranch::Cache(cache_key))) => {
                // Create the intermediate cache node
                let cache_node_key = DependencyGraphKey::Cache(cache_key);
                graph.add_node(cache_node_key, (), NoopDagReporter);
                graph.add_edge(node_key, cache_node_key, NoopDagReporter);
                // Right node has already been processed
                let right = None;
                // Use the intermediate cache node as the invalidation parent when processing child nodes
                let parent_key = cache_node_key;
                (left, right, parent_key)
            }
            // Otherwise use the current node as the invalidation parent when processing child nodes
            (left, right) => {
                let parent_key = node_key;
                (left, right, parent_key)
            }
        };
        let (left_leaf, left_branch) = match left {
            Some(ParsedDependencyGraphBranch::Tree(branch)) => (None, Some(branch)),
            Some(leaf) => (Some(DependencyGraphKey::from(&leaf)), None),
            None => (None, None),
        };
        let (right_leaf, right_branch) = match right {
            Some(ParsedDependencyGraphBranch::Tree(branch)) => (None, Some(branch)),
            Some(leaf) => (Some(DependencyGraphKey::from(&leaf)), None),
            None => (None, None),
        };
        // Register any leaf dependencies for this node
        for leaf_key in [left_leaf, right_leaf].into_iter().flatten() {
            graph.add_node(leaf_key, (), NoopDagReporter);
            graph.add_edge(parent_key, leaf_key, NoopDagReporter);
        }
        // Push any child branches of this node onto the stack for processing
        for branch in [left_branch, right_branch].into_iter().flatten() {
            queue.push(DependencyGraphVisitorQueueEntry {
                node: branch,
                parent: Some(parent_key),
            });
        }

        fn parse_child_branch<A: Arena + Clone>(
            child: Option<&ArenaRef<Term, A>>,
        ) -> Option<ParsedDependencyGraphBranch<A>> {
            match child {
                None => None,
                Some(child) => {
                    if let Some(child) = child.as_tree_term() {
                        Some(ParsedDependencyGraphBranch::Tree(child.clone()))
                    } else if let Some(condition) = child.as_condition_term() {
                        if let Some(cache_key) = parse_cache_dependency(condition) {
                            Some(ParsedDependencyGraphBranch::Cache(cache_key))
                        } else {
                            let state_token = ConditionType::id(condition);
                            Some(ParsedDependencyGraphBranch::State(state_token))
                        }
                    } else {
                        None
                    }
                }
            }
        }
    }
    graph
}

fn get_occupied_evaluation_cache_buckets<A: Arena + Clone>(
    evaluation_cache_instance: EvaluationCacheInstancePointer,
    arena: &A,
) -> impl Iterator<
    Item = (
        EvaluationCacheBucketPointer,
        ArenaRef<EvaluationCacheBucket, A>,
    ),
> + '_ {
    get_occupied_evaluation_cache_bucket_pointers(evaluation_cache_instance, arena).map(|pointer| {
        let bucket_pointer = EvaluationCacheBucketPointer::from(pointer);
        (bucket_pointer, bucket_pointer.as_arena_ref(arena.clone()))
    })
}

fn get_occupied_evaluation_cache_bucket_pointers<A: Arena + Clone>(
    evaluation_cache_instance: EvaluationCacheInstancePointer,
    arena: &A,
) -> impl Iterator<Item = EvaluationCacheBucketPointer> + '_ {
    get_evaluation_cache_bucket_pointers(evaluation_cache_instance, arena)
        .map(ArenaPointer::from)
        .skip_uninitialized_pointers(arena)
        .map(EvaluationCacheBucketPointer::from)
}

fn get_evaluation_cache_bucket_pointers(
    evaluation_cache_instance: EvaluationCacheInstancePointer,
    arena: &(impl Arena + Clone),
) -> impl Iterator<Item = EvaluationCacheBucketPointer> + Clone + '_ {
    let evaluation_cache = evaluation_cache_instance.as_arena_ref(arena.clone());
    let evaluation_cache_buckets_array = evaluation_cache.inner_ref(|term| &term.buckets);
    evaluation_cache_buckets_array
        .item_offsets()
        .map(EvaluationCacheBucketPointer::from)
}

fn invalidate_evaluation_cache_entries<T: Expression>(
    cache_key: &impl ConditionType<T>,
    state: &mut WasmWorkerInvalidationMetadata,
    invalidated_nodes: impl IntoIterator<Item = DependencyGraphKey>,
    arena: &mut impl ArenaMut,
) {
    let WasmWorkerInvalidationMetadata {
        evaluation_cache_num_entries,
        evaluation_cache_bucket_dependency_keys,
        evaluation_cache_bucket_lookup,
        dependency_graph,
        ..
    } = state;

    #[derive(Default, Debug)]
    struct InvalidatedEvaluationCacheEntriesTracker {
        invalidated_cache_entries: Vec<EvaluationCacheKey>,
        num_visited_nodes: usize,
        num_removed_nodes: usize,
    }

    impl<V> DagVisitor<DependencyGraphKey, V> for InvalidatedEvaluationCacheEntriesTracker {
        fn visit_node(mut self, _key: &DependencyGraphKey, _value: &V) -> Self {
            self.num_visited_nodes += 1;
            self
        }
    }

    impl<V> DagReporter<DependencyGraphKey, V> for InvalidatedEvaluationCacheEntriesTracker {
        fn remove_node(mut self, key: DependencyGraphKey, _value: V) -> Self {
            self.num_removed_nodes += 1;
            if let DependencyGraphKey::Cache(cache_key) = key {
                self.invalidated_cache_entries.push(cache_key);
            }
            self
        }
    }

    let InvalidatedEvaluationCacheEntriesTracker {
        invalidated_cache_entries,
        num_visited_nodes,
        num_removed_nodes,
    } = dependency_graph.remove_deep(
        invalidated_nodes,
        DagEdgeDirection::Inbound,
        InvalidatedEvaluationCacheEntriesTracker::default(),
    );
    println!(
        "[{}] {} cache dependencies invalidated ({num_visited_nodes} nodes visited of {})",
        cache_key.id(),
        invalidated_cache_entries.len(),
        {
            let num_remaining_nodes = dependency_graph.len();
            num_remaining_nodes + num_removed_nodes
        }
    );
    // Clear the cache entries for the invalidated buckets by overwriting the bucket contents with an uninitialized bucket
    // (the entries will be recreated during the next evaluation pass)
    let mut num_removed_buckets = 0;
    for dependency_key in invalidated_cache_entries {
        if let Some(bucket_pointer) = evaluation_cache_bucket_lookup.remove(&dependency_key) {
            evaluation_cache_bucket_dependency_keys.remove(&bucket_pointer);
            clear_evaluation_cache_bucket(arena, bucket_pointer);
            num_removed_buckets += 1;
        }
    }
    // If any buckets were cleared, decrement the entry count accordingly
    if num_removed_buckets > 0 {
        update_evaluation_cache_num_entries(
            arena,
            *evaluation_cache_num_entries,
            |num_existing_entries| num_existing_entries - num_removed_buckets,
        );
    }
}

impl<T: Expression> WasmWorkerInitializedState<T> {
    fn get_statistics(&self) -> BytecodeWorkerStatistics {
        BytecodeWorkerStatistics {
            state_dependency_count: self
                .latest_result
                .as_ref()
                .map(|latest_result| latest_result.result.dependencies().len())
                .unwrap_or(0),
            evaluation_cache_entry_count: 0,
            evaluation_cache_deep_size: 0,
        }
    }
}

dispatcher!({
    pub enum WasmWorkerAction<T: Expression> {
        Inbox(BytecodeInterpreterInitAction<T>),
        Inbox(BytecodeInterpreterEvaluateAction<T>),
        Inbox(BytecodeInterpreterGcAction<T>),

        Outbox(BytecodeInterpreterResultAction<T>),
        Outbox(BytecodeInterpreterGcCompleteAction<T>),
    }

    impl<T, TFactory, TAllocator, TAction, TTask> Dispatcher<TAction, TTask>
        for WasmWorker<T, TFactory, TAllocator>
    where
        T: Expression,
        TFactory: ExpressionFactory<T>,
        TAllocator: HeapAllocator<T>,
        T::Builtin: Into<reflex_wasm::stdlib::Stdlib>,
        TAction: Action,
        TTask: TaskFactory<TAction, TTask>,
    {
        type State = WasmWorkerState<T>;
        type Events<TInbox: TaskInbox<TAction>> = TInbox;
        type Dispose = NoopDisposeCallback;

        fn init(&self) -> Self::State {
            Default::default()
        }
        fn events<TInbox: TaskInbox<TAction>>(
            &self,
            inbox: TInbox,
        ) -> ActorEvents<TInbox, Self::Events<TInbox>, Self::Dispose> {
            ActorEvents::Sync(inbox)
        }

        fn accept(&self, _action: &BytecodeInterpreterInitAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &BytecodeInterpreterInitAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Async)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &BytecodeInterpreterInitAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_bytecode_interpreter_init(state, action, metadata, context)
        }

        fn accept(&self, _action: &BytecodeInterpreterEvaluateAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &BytecodeInterpreterEvaluateAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Blocking)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &BytecodeInterpreterEvaluateAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_bytecode_interpreter_evaluate(state, action, metadata, context)
        }

        fn accept(&self, _action: &BytecodeInterpreterGcAction<T>) -> bool {
            true
        }
        fn schedule(
            &self,
            _action: &BytecodeInterpreterGcAction<T>,
            _state: &Self::State,
        ) -> Option<SchedulerMode> {
            Some(SchedulerMode::Blocking)
        }
        fn handle(
            &self,
            state: &mut Self::State,
            action: &BytecodeInterpreterGcAction<T>,
            metadata: &MessageData,
            context: &mut impl HandlerContext,
        ) -> Option<SchedulerTransition<TAction, TTask>> {
            self.handle_bytecode_interpreter_gc(state, action, metadata, context)
        }
    }
});

#[derive(Debug)]
pub enum WasmWorkerError<T: Expression> {
    Unititialized,
    ImpureModuleEntryPoint,
    InvalidFunctionTable,
    InvalidEvaluationCache,
    InvalidFunctionTableArityLookup,
    InterpreterError(InterpreterError),
    SerializationError(T),
}

impl<T: Expression + std::fmt::Display> std::fmt::Display for WasmWorkerError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unititialized => write!(f, "WebAssembly module not initialized"),
            Self::ImpureModuleEntryPoint => {
                write!(
                    f,
                    "Module definition cannot reference runtime values at top level"
                )
            }
            Self::InvalidFunctionTable => write!(f, "Invalid function table definition"),
            Self::InvalidEvaluationCache => write!(f, "Invalid evaluation cache definition"),
            Self::InvalidFunctionTableArityLookup => {
                write!(f, "Invalid function table arity lookup function")
            }
            Self::InterpreterError(err) => {
                write!(f, "WebAssembly interpreter error: {}", err)
            }
            Self::SerializationError(term) => write!(
                f,
                "WebAssembly serialization error: unable to serialize term: {}",
                term
            ),
        }
    }
}

enum MaybeOwned<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}
impl<'a, T: 'a> Deref for MaybeOwned<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(value) => &value,
            Self::Borrowed(value) => value,
        }
    }
}

impl<T, TFactory, TAllocator> WasmWorker<T, TFactory, TAllocator>
where
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
    T::Builtin: Into<reflex_wasm::stdlib::Stdlib>,
{
    fn handle_bytecode_interpreter_init<TAction, TTask>(
        &self,
        state: &mut WasmWorkerState<T>,
        action: &BytecodeInterpreterInitAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action
            + From<BytecodeInterpreterResultAction<T>>
            + From<BytecodeInterpreterGcCompleteAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let BytecodeInterpreterInitAction { cache_key } = action;
        if cache_key.id() != self.cache_key.id() {
            return None;
        }
        match state {
            WasmWorkerState::Uninitialized => {
                *state = match {
                    let compiler_start_time = Instant::now();
                    WasmInterpreter::instantiate(&self.wasm_module, "memory")
                        .map_err(WasmWorkerError::InterpreterError)
                        .and_then(|mut instance| {
                            // TODO: Move WASM indirect call arity lookup generation and graph root retrieval to startup phase
                            let indirect_call_table_size = instance
                                .get_table_size("__indirect_function_table")
                                .ok_or_else(|| WasmWorkerError::InvalidFunctionTable)?;
                            let indirect_call_arity = (0..indirect_call_table_size)
                                .map(FunctionIndex::from)
                                .map(|function_index| {
                                    let (num_positional_args, has_variadic_args) = instance
                                        .call::<u32, (u32, u32)>(
                                            "__indirect_function_arity",
                                            u32::from(function_index),
                                        )
                                        .map_err(WasmWorkerError::InterpreterError)?;
                                    let num_positional_args = num_positional_args as usize;
                                    let has_variadic_args = match has_variadic_args {
                                        1 => true,
                                        _ => false,
                                    };
                                    if let Some(builtin) =
                                        reflex_wasm::stdlib::Stdlib::try_from(function_index).ok()
                                    {
                                        let arity = builtin.arity();
                                        if num_positional_args
                                            == arity.required().len() + arity.optional().len()
                                            && has_variadic_args == arity.variadic().is_some()
                                        {
                                            Ok((function_index, arity))
                                        } else {
                                            Err(WasmWorkerError::InvalidFunctionTableArityLookup)
                                        }
                                    } else {
                                        let required_args = num_positional_args;
                                        let optional_args = 0;
                                        Ok((
                                            function_index,
                                            // TODO: Differentiate between eager/strict/lazy lambda arguments
                                            Arity::eager(
                                                required_args,
                                                optional_args,
                                                has_variadic_args,
                                            ),
                                        ))
                                    }
                                })
                                .collect::<Result<HashMap<_, _>, _>>()?;
                            let evaluation_cache_global = EvaluationCacheGlobalPointer::from(
                                instance
                                    .get_global("__cache")
                                    .and_then(|value| match value {
                                        Val::I32(heap_pointer) => {
                                            Some(ArenaPointer::from(heap_pointer as u32))
                                        }
                                        _ => None,
                                    })
                                    .ok_or_else(|| WasmWorkerError::InvalidEvaluationCache)?,
                            );
                            let (evaluation_cache_cell, _, _) = {
                                let arena = &instance;
                                get_evaluation_cache_instance(&arena, evaluation_cache_global)
                            };
                            let graph_root = instance
                                .call::<u32, (u32, u32)>(
                                    &self.graph_root_factory_export_name,
                                    u32::from(ArenaPointer::null()),
                                )
                                .map_err(WasmWorkerError::InterpreterError)
                                .and_then(|(graph_root, dependencies)| {
                                    if ArenaPointer::from(dependencies).is_null() {
                                        Ok(ArenaPointer::from(graph_root))
                                    } else {
                                        Err(WasmWorkerError::ImpureModuleEntryPoint)
                                    }
                                })?;
                            let mut wasm_factory =
                                WasmTermFactory::from(Rc::new(RefCell::new(&mut instance)));
                            let entry_point = match self.evaluation_mode {
                                QueryEvaluationMode::Query => compile_graphql_query(
                                    graph_root,
                                    &self.query,
                                    &self.factory,
                                    &mut wasm_factory,
                                ),
                                QueryEvaluationMode::Standalone => compile_wasm_expression(
                                    &self.query,
                                    &self.factory,
                                    &mut wasm_factory,
                                ),
                            };
                            let initial_heap_snapshot = Vec::from(
                                &instance.data()[0..u32::from(instance.end_offset()) as usize],
                            );
                            let elapsed_time = compiler_start_time.elapsed();
                            {
                                match &self.metric_names.query_worker_compile_duration {
                                    Cow::Borrowed(metric_name) => {
                                        histogram!(*metric_name, elapsed_time.as_secs_f64())
                                    }
                                    Cow::Owned(metric_name) => {
                                        histogram!(metric_name.clone(), elapsed_time.as_secs_f64())
                                    }
                                }
                            }
                            entry_point
                                .map_err(WasmWorkerError::SerializationError)
                                .map(|entry_point| WasmWorkerInitializedState {
                                    instance,
                                    initial_heap_snapshot,
                                    indirect_call_arity,
                                    evaluation_cache_global_pointer: evaluation_cache_global,
                                    evaluation_cache_initial_cell: evaluation_cache_cell,
                                    entry_point,
                                    state_index: Default::default(),
                                    state_values: Default::default(),
                                    latest_result: Default::default(),
                                })
                        })
                } {
                    Ok(state) => WasmWorkerState::Initialized(state),
                    Err(err) => WasmWorkerState::Error(err),
                };
                None
            }
            WasmWorkerState::Error(_) | WasmWorkerState::Initialized(_) => None,
        }
    }
    fn handle_bytecode_interpreter_evaluate<TAction, TTask>(
        &self,
        state: &mut WasmWorkerState<T>,
        action: &BytecodeInterpreterEvaluateAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action
            + From<BytecodeInterpreterResultAction<T>>
            + From<BytecodeInterpreterGcCompleteAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let BytecodeInterpreterEvaluateAction {
            cache_key,
            state_index,
            state_updates,
        } = action;
        if cache_key.id() != self.cache_key.id() {
            return None;
        }
        let result = match state {
            WasmWorkerState::Uninitialized => {
                Err(MaybeOwned::Owned(WasmWorkerError::Unititialized))
            }
            WasmWorkerState::Error(err) => Err(MaybeOwned::Borrowed(&*err)),
            WasmWorkerState::Initialized(worker_state) => {
                let state_index = *state_index;
                worker_state.state_index = state_index;
                let state_update_status =
                    state_updates.iter().fold(Ok(()), |result, (key, value)| {
                        let _ = result?;
                        let wasm_factory = WasmTermFactory::from(Rc::new(RefCell::new(
                            &mut worker_state.instance,
                        )));
                        let state_token = key.id();
                        let value_pointer =
                            import_wasm_expression(value, &self.factory, &wasm_factory)
                                .map_err(WasmWorkerError::SerializationError)?;
                        match worker_state.state_values.entry(state_token) {
                            Entry::Occupied(mut entry) => {
                                let (_key, state_value) = entry.get_mut();
                                *state_value = value_pointer;
                            }
                            Entry::Vacant(entry) => {
                                let key_pointer =
                                    import_wasm_condition(key, &self.factory, &wasm_factory)
                                        .map_err(WasmWorkerError::SerializationError)?;
                                entry.insert((key_pointer, value_pointer));
                            }
                        }
                        Ok(())
                    });
                match state_update_status {
                    Err(err) => Err(MaybeOwned::Owned(err)),
                    Ok(_) => {
                        // Clear any evaluation cache buckets that have been invalidated by the current update batch
                        if let Some(previous_result) = worker_state.latest_result.as_mut() {
                            println!(
                                "[{}] Updating {} state entries",
                                self.cache_key.id(),
                                state_updates.len()
                            );
                            invalidate_evaluation_cache_entries(
                                &self.cache_key,
                                &mut previous_result.invalidation_metadata,
                                state_updates.iter().filter_map(|(condition, _)| {
                                    let state_token = ConditionType::id(condition);
                                    previous_result
                                        .state_dependencies
                                        .get(&state_token)
                                        .copied()
                                }),
                                &mut worker_state.instance,
                            );
                        }
                        // Allocate a new hashmap for the global state key/value lookup
                        let runtime_state = if worker_state.state_values.is_empty() {
                            ArenaPointer::null()
                        } else {
                            HashmapTerm::allocate(
                                worker_state
                                    .state_values
                                    .values()
                                    .map(|(key, value)| (*key, *value)),
                                &mut worker_state.instance,
                            )
                        };
                        // Grab the result of the previous evaluation for later use
                        let previous_result = worker_state.latest_result.take();
                        // Keep track of the bump allocator offset before evaluation
                        let existing_heap_size = worker_state.instance.end_offset();
                        let start_time = Instant::now();
                        let result = time_operation(
                            self.cache_key.id(),
                            "Evaluate WASM VM interpreter expression",
                            || {
                                worker_state
                                    .instance
                                    .evaluate(worker_state.entry_point, runtime_state)
                            },
                        );
                        let elapsed_time = start_time.elapsed();
                        match &self.metric_names.query_worker_evaluate_duration {
                            Cow::Borrowed(metric_name) => {
                                histogram!(*metric_name, elapsed_time.as_secs_f64())
                            }
                            Cow::Owned(metric_name) => {
                                histogram!(metric_name.clone(), elapsed_time.as_secs_f64())
                            }
                        }
                        let (result, previous_evaluation_cache) = match result {
                            Ok(UnboundEvaluationResult {
                                result_pointer,
                                dependencies_pointer,
                            }) => {
                                let dependencies_pointer = dependencies_pointer
                                    .map(EvaluationResultDependencyTreePointer::from);
                                let (result, newly_added_dependency_mappings) = {
                                    let arena = Rc::new(RefCell::new(&mut worker_state.instance));
                                    let value =
                                        ArenaRef::<Term, _>::new(Rc::clone(&arena), result_pointer);
                                    let dependencies =
                                        dependencies_pointer.map(|dependencies_pointer| {
                                            dependencies_pointer.as_arena_ref(Rc::clone(&arena))
                                        });
                                    let existing_dependency_mappings =
                                        previous_result.as_ref().map(|previous_result| {
                                            &previous_result.dependency_mappings
                                        });
                                    parse_wasm_interpreter_result(
                                        &self.cache_key,
                                        &value,
                                        dependencies.as_ref(),
                                        existing_dependency_mappings,
                                        &self.factory,
                                        &self.allocator,
                                        &arena,
                                        &worker_state.indirect_call_arity,
                                    )
                                };
                                // Take the existing dependency mappings and state dependencies from the previous result
                                let (
                                    previous_evaluation_cache,
                                    previous_cache_metadata,
                                    previous_dependency_mappings,
                                    previous_state_dependencies,
                                ) = match previous_result {
                                    Some(WasmWorkerEvaluationResult {
                                        dependencies,
                                        invalidation_metadata,
                                        dependency_mappings,
                                        state_dependencies,
                                        ..
                                    }) => (
                                        Some((
                                            invalidation_metadata.evaluation_cache_cell,
                                            invalidation_metadata.evaluation_cache_instance,
                                            invalidation_metadata.evaluation_cache_num_entries,
                                        )),
                                        Some((dependencies, invalidation_metadata)),
                                        dependency_mappings,
                                        state_dependencies,
                                    ),
                                    None => (
                                        Default::default(),
                                        Default::default(),
                                        Default::default(),
                                        Default::default(),
                                    ),
                                };
                                let invalidation_metadata = {
                                    let arena = &worker_state.instance;
                                    time_operation(
                                        self.cache_key.id(),
                                        "Computing dependency metadata",
                                        || {
                                            compute_evaluation_cache_metadata(
                                                &arena,
                                                worker_state.evaluation_cache_global_pointer,
                                                dependencies_pointer,
                                                previous_cache_metadata,
                                            )
                                        },
                                    )
                                };
                                let result_dependencies = result.dependencies();
                                let state_dependencies = {
                                    // Take the existing state token mappings from the previous result
                                    let mut state_dependencies = previous_state_dependencies;
                                    // Remove entries for state token mappings that are no longer needed by the most recent result
                                    state_dependencies.retain(|state_token, _dependency_key| {
                                        result_dependencies.contains(*state_token)
                                    });
                                    // Add entries for newly-encountered state token mappings from the most recent result
                                    state_dependencies.extend(
                                        newly_added_dependency_mappings.iter().map(
                                            |(dependency_key, condition)| {
                                                (ConditionType::id(condition), *dependency_key)
                                            },
                                        ),
                                    );
                                    state_dependencies
                                };
                                let dependency_mappings = {
                                    // Take the existing dependency mappings from the previous result
                                    let mut dependency_mappings = previous_dependency_mappings;
                                    // Remove entries for dependency mappings that are no longer needed by the most recent result
                                    dependency_mappings.retain(|_dependency_key, condition| {
                                        result_dependencies.contains(ConditionType::id(condition))
                                    });
                                    // Add entries for newly-encountered dependency mappings from the most recent result
                                    dependency_mappings.extend(newly_added_dependency_mappings);
                                    dependency_mappings
                                };
                                worker_state.latest_result = Some(WasmWorkerEvaluationResult {
                                    result: result.clone(),
                                    value: result_pointer,
                                    dependencies: dependencies_pointer,
                                    invalidation_metadata,
                                    state_dependencies,
                                    dependency_mappings,
                                });
                                (
                                    Ok((result, worker_state.get_statistics())),
                                    previous_evaluation_cache,
                                )
                            }
                            Err(err) => {
                                let previous_evaluation_cache = previous_result.as_ref().map(
                                    |WasmWorkerEvaluationResult {
                                         invalidation_metadata,
                                         ..
                                     }| {
                                        (
                                            invalidation_metadata.evaluation_cache_cell,
                                            invalidation_metadata.evaluation_cache_instance,
                                            invalidation_metadata.evaluation_cache_num_entries,
                                        )
                                    },
                                );
                                (
                                    Err(MaybeOwned::Owned(WasmWorkerError::InterpreterError(err))),
                                    previous_evaluation_cache,
                                )
                            }
                        };
                        let should_dump_heap_snapshot = self
                            .dump_heap_snapshot
                            .as_ref()
                            .map(|criteria| {
                                criteria.should_dump_heap(
                                    self.evaluation_mode,
                                    result.as_ref().map(|(result, _)| result.result()),
                                    &self.factory,
                                )
                            })
                            .unwrap_or(false);
                        if should_dump_heap_snapshot {
                            let heap_snapshot = {
                                let mut bytes = worker_state.instance.dump_heap();
                                // Ignore any heap values allocated during this evaluation
                                let snapshot_heap_size = existing_heap_size;
                                bytes.truncate(u32::from(snapshot_heap_size) as usize);
                                // Purge global evaluation memoization cache
                                // (this is necessary because the global cache pointer can be mutated within a given
                                // evaluation, leaving dangling pointers to terms that were only allocated during the
                                // current evaluation pass and which therefore will not exist in the pre-evaluation heap
                                // snapshot)
                                match previous_evaluation_cache {
                                    Some((
                                        evaluation_cache_cell,
                                        evaluation_cache_instance,
                                        evaluation_cache_num_entries,
                                    )) => {
                                        let invalidated_buckets =
                                            get_evaluation_cache_instance_dangling_pointers(
                                                &bytes.deref(),
                                                evaluation_cache_instance,
                                                snapshot_heap_size,
                                            )
                                            .collect::<Vec<_>>();
                                        clean_evaluation_cache_instance_buckets(
                                            &mut bytes.deref_mut(),
                                            evaluation_cache_num_entries,
                                            invalidated_buckets,
                                        );
                                        update_global_evaluation_cache_instance(
                                            &mut bytes.deref_mut(),
                                            worker_state.evaluation_cache_global_pointer,
                                            evaluation_cache_cell,
                                        )
                                    }
                                    None => update_global_evaluation_cache_instance(
                                        &mut bytes.deref_mut(),
                                        worker_state.evaluation_cache_global_pointer,
                                        worker_state.evaluation_cache_initial_cell,
                                    ),
                                }
                                bytes
                            };
                            let output_filename = format!(
                                "{}_{}_{}_{}.bin",
                                cache_key.id(),
                                state_index.map(usize::from).unwrap_or(0),
                                u32::from(worker_state.entry_point),
                                u32::from(runtime_state)
                            );
                            println!(
                                "Dumping {} bytes to {output_filename}...",
                                heap_snapshot.len()
                            );
                            std::fs::write(output_filename, heap_snapshot)
                                .expect("Failed to dump heap");
                            println!("Heap dump complete");
                            println!(
                                "Invoking function evaluate({}, {})",
                                u32::from(worker_state.entry_point),
                                u32::from(runtime_state)
                            );
                        }
                        result
                    }
                }
            }
        };
        match result {
            Ok((result, statistics)) => {
                Some(SchedulerTransition::new(once(SchedulerCommand::Send(
                    self.caller_pid,
                    BytecodeInterpreterResultAction {
                        cache_key: cache_key.clone(),
                        state_index: *state_index,
                        result,
                        statistics,
                    }
                    .into(),
                ))))
            }
            Err(err) => {
                let message = format!("{}", err.deref());
                let result = EvaluationResult::new(
                    create_error_expression(message, &self.factory, &self.allocator),
                    Default::default(),
                );
                Some(SchedulerTransition::new(once(SchedulerCommand::Send(
                    self.caller_pid,
                    BytecodeInterpreterResultAction {
                        cache_key: cache_key.clone(),
                        state_index: *state_index,
                        result,
                        statistics: Default::default(),
                    }
                    .into(),
                ))))
            }
        }
    }
    fn handle_bytecode_interpreter_gc<TAction, TTask>(
        &self,
        state: &mut WasmWorkerState<T>,
        action: &BytecodeInterpreterGcAction<T>,
        _metadata: &MessageData,
        _context: &mut impl HandlerContext,
    ) -> Option<SchedulerTransition<TAction, TTask>>
    where
        TAction: Action + From<BytecodeInterpreterGcCompleteAction<T>>,
        TTask: TaskFactory<TAction, TTask>,
    {
        let BytecodeInterpreterGcAction {
            cache_key,
            state_index,
        } = action;
        if cache_key.id() != self.cache_key.id() {
            return None;
        }
        match state {
            WasmWorkerState::Uninitialized | WasmWorkerState::Error(_) => None,
            WasmWorkerState::Initialized(state) => {
                let latest_state_index = &state.state_index;
                if state_index < latest_state_index {
                    return None;
                }
                let start_time = Instant::now();
                // Garbage-collect unused heap values
                let should_gc_heap = {
                    // FIXME: current GC algorithm copies inner terms via recursion, causing stack overflows for deeply-nested results
                    !matches!(self.evaluation_mode, QueryEvaluationMode::Query)
                };
                if should_gc_heap {
                    time_operation(
                        self.cache_key.id(),
                        "Garbage-collection total for linear memory",
                        || {
                            gc_vm_heap(&self.cache_key, state);
                        },
                    );
                }
                // Garbage-collect unused state values
                time_operation(
                    self.cache_key.id(),
                    "Garbage-collecting state values",
                    || {
                        let empty_dependencies = DependencyList::default();
                        let retained_state_tokens = state
                            .latest_result
                            .as_ref()
                            .map(|latest_result| latest_result.result.dependencies())
                            .unwrap_or(&empty_dependencies);
                        if retained_state_tokens.len() < state.state_values.len() {
                            state
                                .state_values
                                .retain(|key, _| retained_state_tokens.contains(*key));
                        }
                    },
                );
                let elapsed_time = start_time.elapsed();
                match &self.metric_names.query_worker_gc_duration {
                    Cow::Borrowed(metric_name) => {
                        histogram!(*metric_name, elapsed_time.as_secs_f64())
                    }
                    Cow::Owned(metric_name) => {
                        histogram!(metric_name.clone(), elapsed_time.as_secs_f64())
                    }
                }
                Some(SchedulerTransition::new(once(SchedulerCommand::Send(
                    self.caller_pid,
                    BytecodeInterpreterGcCompleteAction {
                        cache_key: cache_key.clone(),
                        statistics: state.get_statistics(),
                    }
                    .into(),
                ))))
            }
        }
    }
}

fn get_evaluation_cache_instance_dangling_pointers(
    arena: &(impl Arena + Clone),
    evaluation_cache_instance: EvaluationCacheInstancePointer,
    end_offset: ArenaPointer,
) -> impl Iterator<Item = EvaluationCacheBucketPointer> + '_ {
    get_occupied_evaluation_cache_buckets(evaluation_cache_instance, arena).filter_map(
        move |(bucket_pointer, bucket)| {
            if bucket.value().as_pointer() >= end_offset
                || bucket
                    .dependencies()
                    .map(|dependencies| dependencies.as_pointer() >= end_offset)
                    .unwrap_or(false)
            {
                Some(bucket_pointer)
            } else {
                None
            }
        },
    )
}

fn clean_evaluation_cache_instance_buckets(
    arena: &mut impl ArenaMut,
    evaluation_cache_num_entries: EvaluationCacheNumEntriesPointer,
    buckets: impl IntoIterator<Item = EvaluationCacheBucketPointer>,
) {
    let mut num_removed_buckets = 0;
    for bucket_pointer in buckets.into_iter() {
        clear_evaluation_cache_bucket(arena, bucket_pointer);
        num_removed_buckets += 1;
    }
    // If any buckets were cleared, decrement the entry count accordingly
    if num_removed_buckets > 0 {
        update_evaluation_cache_num_entries(
            arena,
            evaluation_cache_num_entries,
            |num_existing_entries| num_existing_entries - num_removed_buckets,
        );
    }
}

fn clear_evaluation_cache_bucket(
    arena: &mut impl ArenaMut,
    bucket_pointer: EvaluationCacheBucketPointer,
) {
    // Overwrite the existing cache bucket with an uninitialized entry
    arena.write::<EvaluationCacheBucket>(
        bucket_pointer.as_pointer(),
        EvaluationCacheBucket::uninitialized(),
    );
}

fn update_evaluation_cache_num_entries(
    arena: &mut impl ArenaMut,
    evaluation_cache_num_entries: EvaluationCacheNumEntriesPointer,
    updater: impl Fn(u32) -> u32,
) {
    let num_entries_pointer = evaluation_cache_num_entries.as_pointer();
    let existing_value = arena.read_value::<u32, u32>(num_entries_pointer, |value| *value);
    let updated_value = updater(existing_value);
    arena.write::<u32>(num_entries_pointer, updated_value);
}

fn parse_wasm_interpreter_result<'heap, T: Expression>(
    cache_key: &T::Signal,
    result: &WasmExpression<Rc<RefCell<&'heap mut WasmInterpreter>>>,
    dependencies: Option<&ArenaRef<TypedTerm<TreeTerm>, Rc<RefCell<&'heap mut WasmInterpreter>>>>,
    existing_dependency_mappings: Option<&IntMap<DependencyGraphKey, T::Signal>>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
    arena: &Rc<RefCell<&'heap mut WasmInterpreter>>,
    indirect_call_arity: &HashMap<FunctionIndex, Arity>,
) -> (EvaluationResult<T>, IntMap<DependencyGraphKey, T::Signal>) {
    let (result, dependencies, state_dependencies) =
        time_operation(cache_key.id(), "Translating WASM result", || {
            parse_wasm_expression(result, factory, allocator, arena, indirect_call_arity)
        })
        .and_then(|(result, dependency_mappings)| {
            let dependencies = match dependencies {
                None => DependencyList::default(),
                Some(dependencies) => time_operation(
                    cache_key.id(),
                    format!(
                        "Parsing WASM result dependencies (length: {}, depth: {})",
                        dependencies.as_inner().len(),
                        dependencies.as_inner().depth(),
                    ),
                    || {
                        parse_wasm_interpreter_result_dependencies(
                            dependencies,
                            &dependency_mappings,
                            existing_dependency_mappings,
                        )
                    },
                ),
            };
            Ok((result, dependencies, dependency_mappings))
        })
        .unwrap_or_else(|term| {
            (
                create_error_expression(
                    if let Some(condition) = term.as_condition_term() {
                        format!("{}", condition)
                    } else {
                        format!("Unable to translate evaluation result: {}", term)
                    },
                    factory,
                    allocator,
                ),
                DependencyList::default(),
                IntMap::<DependencyGraphKey, T::Signal>::default(),
            )
        });
    (
        EvaluationResult::new(result, dependencies),
        state_dependencies,
    )
}

fn parse_wasm_interpreter_result_dependencies<'heap, T: Expression, S: ConditionType<T>>(
    dependencies: &ArenaRef<TypedTerm<TreeTerm>, Rc<RefCell<&'heap mut WasmInterpreter>>>,
    dependency_mappings: &IntMap<DependencyGraphKey, S>,
    existing_dependency_mappings: Option<&IntMap<DependencyGraphKey, S>>,
) -> DependencyList {
    let empty_dependency_mappings = IntMap::<DependencyGraphKey, S>::default();
    let existing_dependency_mappings =
        existing_dependency_mappings.unwrap_or(&empty_dependency_mappings);
    dependencies
        .as_inner()
        .typed_nodes::<ConditionTerm>()
        .filter_map(|dependency| {
            let condition = match parse_cache_dependency(&dependency) {
                Some(_) => None,
                None => Some(dependency),
            }?;
            let dependency_key = DependencyGraphKey::State(ConditionType::id(&condition));
            let parsed_condition = existing_dependency_mappings
                .get(&dependency_key)
                .or_else(|| dependency_mappings.get(&dependency_key))?;
            Some(ConditionType::id(parsed_condition))
        })
        .collect::<DependencyList>()
}

fn create_error_expression<T: Expression>(
    message: String,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T {
    factory.create_signal_term(allocator.create_signal_list(once(allocator.create_signal(
        SignalType::Error {
            payload: factory.create_string_term(allocator.create_string(message)),
        },
    ))))
}

fn compile_graphql_query<'heap, T: Expression>(
    graph_root_factory: ArenaPointer,
    query: &T,
    factory: &impl ExpressionFactory<T>,
    arena: &mut WasmTermFactory<&'heap mut WasmInterpreter>,
) -> Result<ArenaPointer, T>
where
    T::Builtin: Into<reflex_wasm::stdlib::Stdlib>,
{
    let compiled_query_function = compile_wasm_expression(query, factory, arena)?;
    let graph_root = {
        // Graph root factory evaluates to a 1-argument function that takes the query token as an argument
        let query_token = arena.allocate(Term::new(
            TermType::Symbol(SymbolTerm {
                id: (query.id() & 0x00000000FFFFFFFF) as u32,
            }),
            arena,
        ));
        let factory_args = ListTerm::allocate([query_token], arena);
        let factory_call = arena.allocate(Term::new(
            TermType::Application(ApplicationTerm {
                target: graph_root_factory,
                args: factory_args,
            }),
            arena,
        ));
        factory_call
    };
    let query = {
        // Create an expression that applies the query function to the graph root
        let query_term = Term::new(
            TermType::Application(ApplicationTerm {
                target: compiled_query_function,
                args: ListTerm::allocate([graph_root], arena),
            }),
            &*arena,
        );
        arena.allocate(query_term)
    };
    Ok(query)
}

fn compile_wasm_expression<'heap, T: Expression>(
    expression: &T,
    factory: &impl ExpressionFactory<T>,
    wasm_factory: &WasmTermFactory<&'heap mut WasmInterpreter>,
) -> Result<ArenaPointer, T>
where
    T::Builtin: Into<reflex_wasm::stdlib::Stdlib>,
{
    import_wasm_expression(expression, factory, wasm_factory)
}

fn import_wasm_expression<'heap, T: Expression>(
    expression: &T,
    factory: &impl ExpressionFactory<T>,
    wasm_factory: &WasmTermFactory<&'heap mut WasmInterpreter>,
) -> Result<ArenaPointer, T>
where
    T::Builtin: Into<reflex_wasm::stdlib::Stdlib>,
{
    let term = wasm_factory.import(expression, factory)?;
    Ok(term.as_pointer())
}

fn import_wasm_condition<'heap, T: Expression>(
    condition: &T::Signal,
    factory: &impl ExpressionFactory<T>,
    wasm_factory: &WasmTermFactory<&'heap mut WasmInterpreter>,
) -> Result<ArenaPointer, T>
where
    T::Builtin: Into<reflex_wasm::stdlib::Stdlib>,
{
    let term = wasm_factory.import_condition(condition, factory)?;
    Ok(term.as_pointer())
}

fn parse_wasm_expression<'heap, T: Expression>(
    expression: &WasmExpression<Rc<RefCell<&'heap mut WasmInterpreter>>>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
    arena: &Rc<RefCell<&'heap mut WasmInterpreter>>,
    indirect_call_arity: &HashMap<FunctionIndex, Arity>,
) -> Result<
    (T, IntMap<DependencyGraphKey, T::Signal>),
    WasmExpression<Rc<RefCell<&'heap mut WasmInterpreter>>>,
> {
    if let Some(term) = expression.as_signal_term() {
        let signal_term = term.as_inner();
        signal_term
            .signals()
            .iter()
            .filter(|condition| parse_cache_dependency(condition).is_none())
            .fold(
                Ok((
                    Vec::<T::Signal>::new(),
                    IntMap::<DependencyGraphKey, T::Signal>::default(),
                )),
                |result, condition| {
                    let (mut parsed_conditions, mut state_dependencies) = result?;
                    let dependency_key = DependencyGraphKey::State(ConditionType::id(&condition));
                    if let Entry::Vacant(entry) = state_dependencies.entry(dependency_key) {
                        let parsed_condition = parse_wasm_condition(
                            &condition,
                            factory,
                            allocator,
                            arena,
                            indirect_call_arity,
                        )?;
                        entry.insert(parsed_condition.clone());
                        parsed_conditions.push(parsed_condition);
                    }
                    Ok((parsed_conditions, state_dependencies))
                },
            )
            .map(|(parsed_conditions, state_dependencies)| {
                let result =
                    factory.create_signal_term(allocator.create_signal_list(parsed_conditions));
                (result, state_dependencies)
            })
    } else {
        let wasm_factory = WasmTermFactory::from(Rc::clone(arena));
        wasm_factory
            .export(expression, factory, allocator, indirect_call_arity)
            .map(|result| (result, Default::default()))
    }
}

fn parse_wasm_condition<'heap, T: Expression>(
    condition: &ArenaRef<TypedTerm<ConditionTerm>, Rc<RefCell<&'heap mut WasmInterpreter>>>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
    arena: &Rc<RefCell<&'heap mut WasmInterpreter>>,
    indirect_call_arity: &HashMap<FunctionIndex, Arity>,
) -> Result<T::Signal, WasmExpression<Rc<RefCell<&'heap mut WasmInterpreter>>>> {
    let wasm_factory = WasmTermFactory::from(Rc::clone(arena));
    wasm_factory.export_condition(condition, factory, allocator, indirect_call_arity)
}

fn get_evaluation_cache_instance(
    arena: &(impl Arena + Clone),
    evaluation_cache_global_pointer: EvaluationCacheGlobalPointer,
) -> (
    EvaluationCacheCellPointer,
    EvaluationCacheInstancePointer,
    EvaluationCacheNumEntriesPointer,
) {
    let evaluation_cache_global = evaluation_cache_global_pointer.as_arena_ref(arena.clone());
    let evaluation_cache_cell_pointer =
        EvaluationCacheCellPointer::from(evaluation_cache_global.as_inner().target());
    let evaluation_cache_cell = evaluation_cache_cell_pointer.as_arena_ref(arena.clone());
    let evaluation_cache_instance_pointer = EvaluationCacheInstancePointer::from({
        let cell_fields = evaluation_cache_cell
            .as_inner()
            .inner_ref(|term| &term.fields);
        let instance_pointer = cell_fields.item_offset(0);
        instance_pointer
    });
    let evaluation_cache_instance = evaluation_cache_instance_pointer.as_arena_ref(arena.clone());
    let evaluation_cache_num_entries_pointer = EvaluationCacheNumEntriesPointer::from(
        evaluation_cache_instance.inner_pointer(|cache| &cache.num_entries),
    );
    (
        evaluation_cache_cell_pointer,
        evaluation_cache_instance_pointer,
        evaluation_cache_num_entries_pointer,
    )
}

fn update_global_evaluation_cache_instance(
    arena: &mut impl ArenaMut,
    evaluation_cache_global: EvaluationCacheGlobalPointer,
    evaluation_cache_cell: EvaluationCacheCellPointer,
) {
    // Overwrite the existing cache pointer term with a pointer that points to the new instance
    let evaluation_cache_pointer_term = Term::new(
        TermType::Pointer(PointerTerm {
            target: evaluation_cache_cell.as_pointer(),
        }),
        &*arena,
    );
    arena.write(
        evaluation_cache_global.as_pointer(),
        evaluation_cache_pointer_term,
    );
}

fn is_error_result<T: Expression>(result: &T, factory: &impl ExpressionFactory<T>) -> bool {
    factory
        .match_signal_term(result)
        .filter(|result| {
            result.signals().as_deref().iter().any(|condition| {
                matches!(condition.as_deref().signal_type(), SignalType::Error { .. })
            })
        })
        .is_some()
}

fn is_blocked_result<T: Expression>(result: &T, factory: &impl ExpressionFactory<T>) -> bool {
    !is_error_result(result, factory)
        && factory
            .match_signal_term(result)
            .filter(|result| {
                result.signals().as_deref().iter().any(|condition| {
                    matches!(
                        condition.as_deref().signal_type(),
                        SignalType::Custom { .. }
                    )
                })
            })
            .is_some()
}

fn is_pending_result<T: Expression>(result: &T, factory: &impl ExpressionFactory<T>) -> bool {
    !is_error_result(result, factory)
        && !is_blocked_result(result, factory)
        && factory
            .match_signal_term(result)
            .filter(|result| {
                result.signals().as_deref().iter().any(|condition| {
                    matches!(condition.as_deref().signal_type(), SignalType::Pending)
                })
            })
            .is_some()
}

fn is_unresolved_result<T: Expression>(result: &T, factory: &impl ExpressionFactory<T>) -> bool {
    !is_error_result(result, factory)
        && (is_blocked_result(result, factory) || is_pending_result(result, factory))
}

fn time_operation<T>(
    pid: impl std::fmt::Display,
    message: impl std::fmt::Display,
    operation: impl FnOnce() -> T,
) -> T {
    let start_time = Instant::now();
    let result = operation();
    let elapsed_time = start_time.elapsed();
    println!("[{pid}] {message}: {elapsed_time:?}");
    result
}
