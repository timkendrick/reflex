// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
const FALSE = 0;
const TRUE = 1;
const NULL = -1;

function u32(value) {
  // Convert 32-bit two's complement signed integer to unsigned integer
  return value >= 0 ? value : 0xffffffff + 1 + value;
}

function createTermTypes(runtime) {
  return {
    Application: runtime.TermType_Application.value,
    Boolean: runtime.TermType_Boolean.value,
    Builtin: runtime.TermType_Builtin.value,
    Cell: runtime.TermType_Cell.value,
    Hashmap: runtime.TermType_Hashmap.value,
    Hashset: runtime.TermType_Hashset.value,
    List: runtime.TermType_List.value,
    Condition: runtime.TermType_Condition.value,
    Constructor: runtime.TermType_Constructor.value,
    Date: runtime.TermType_Date.value,
    Effect: runtime.TermType_Effect.value,
    Float: runtime.TermType_Float.value,
    Int: runtime.TermType_Int.value,
    Nil: runtime.TermType_Nil.value,
    Partial: runtime.TermType_Partial.value,
    Pointer: runtime.TermType_Pointer.value,
    Record: runtime.TermType_Record.value,
    Signal: runtime.TermType_Signal.value,
    String: runtime.TermType_String.value,
    Symbol: runtime.TermType_Symbol.value,
    Tree: runtime.TermType_Tree.value,
    Lambda: runtime.TermType_Lambda.value,
    Variable: runtime.TermType_Variable.value,
    Let: runtime.TermType_Let.value,
    EmptyIterator: runtime.TermType_EmptyIterator.value,
    EvaluateIterator: runtime.TermType_EvaluateIterator.value,
    FilterIterator: runtime.TermType_FilterIterator.value,
    FlattenIterator: runtime.TermType_FlattenIterator.value,
    HashmapKeysIterator: runtime.TermType_HashmapKeysIterator.value,
    HashmapValuesIterator: runtime.TermType_HashmapValuesIterator.value,
    IntegersIterator: runtime.TermType_IntegersIterator.value,
    IntersperseIterator: runtime.TermType_IntersperseIterator.value,
    MapIterator: runtime.TermType_MapIterator.value,
    OnceIterator: runtime.TermType_OnceIterator.value,
    RangeIterator: runtime.TermType_RangeIterator.value,
    RepeatIterator: runtime.TermType_RepeatIterator.value,
    SkipIterator: runtime.TermType_SkipIterator.value,
    TakeIterator: runtime.TermType_TakeIterator.value,
    ZipIterator: runtime.TermType_ZipIterator.value,
  };
}

function createConditionTypes(runtime) {
  return {
    Custom: runtime.ConditionType_CustomCondition.value,
    Pending: runtime.ConditionType_PendingCondition.value,
    Error: runtime.ConditionType_ErrorCondition.value,
    TypeError: runtime.ConditionType_TypeErrorCondition.value,
    InvalidFunctionTarget: runtime.ConditionType_InvalidFunctionTargetCondition.value,
    InvalidFunctionArgs: runtime.ConditionType_InvalidFunctionArgsCondition.value,
    InvalidPointer: runtime.ConditionType_InvalidPointerCondition.value,
  };
}

function createStdlib(runtime) {
  return {
    Abs: runtime.Stdlib_Abs.value,
    Accessor: runtime.Stdlib_Accessor.value,
    Add: runtime.Stdlib_Add.value,
    And: runtime.Stdlib_And.value,
    Apply: runtime.Stdlib_Apply.value,
    Car: runtime.Stdlib_Car.value,
    Cdr: runtime.Stdlib_Cdr.value,
    Ceil: runtime.Stdlib_Ceil.value,
    Chain: runtime.Stdlib_Chain.value,
    CollectHashmap: runtime.Stdlib_CollectHashmap.value,
    CollectHashset: runtime.Stdlib_CollectHashset.value,
    CollectList: runtime.Stdlib_CollectList.value,
    CollectString: runtime.Stdlib_CollectString.value,
    CollectTree: runtime.Stdlib_CollectTree.value,
    Cons: runtime.Stdlib_Cons.value,
    Construct: runtime.Stdlib_Construct.value,
    Debug: runtime.Stdlib_Debug.value,
    DecrementVariable: runtime.Stdlib_DecrementVariable.value,
    Divide: runtime.Stdlib_Divide.value,
    Effect: runtime.Stdlib_Effect.value,
    EndsWith: runtime.Stdlib_EndsWith.value,
    Eq: runtime.Stdlib_Eq.value,
    Equal: runtime.Stdlib_Equal.value,
    Flatten: runtime.Stdlib_Flatten.value,
    Floor: runtime.Stdlib_Floor.value,
    Fold: runtime.Stdlib_Fold.value,
    FormatErrorMessage: runtime.Stdlib_FormatErrorMessage.value,
    Get: runtime.Stdlib_Get.value,
    GetVariable: runtime.Stdlib_GetVariable.value,
    Gt: runtime.Stdlib_Gt.value,
    Gte: runtime.Stdlib_Gte.value,
    GraphQlResolver: runtime.Stdlib_GraphQlResolver.value,
    Has: runtime.Stdlib_Has.value,
    Hash: runtime.Stdlib_Hash.value,
    Identity: runtime.Stdlib_Identity.value,
    If: runtime.Stdlib_If.value,
    IfError: runtime.Stdlib_IfError.value,
    IfPending: runtime.Stdlib_IfPending.value,
    IncrementVariable: runtime.Stdlib_IncrementVariable.value,
    IsFinite: runtime.Stdlib_IsFinite.value,
    Iterate: runtime.Stdlib_Iterate.value,
    Keys: runtime.Stdlib_Keys.value,
    Length: runtime.Stdlib_Length.value,
    Log: runtime.Stdlib_Log.value,
    Lt: runtime.Stdlib_Lt.value,
    Lte: runtime.Stdlib_Lte.value,
    Max: runtime.Stdlib_Max.value,
    Merge: runtime.Stdlib_Merge.value,
    Min: runtime.Stdlib_Min.value,
    Multiply: runtime.Stdlib_Multiply.value,
    Not: runtime.Stdlib_Not.value,
    Or: runtime.Stdlib_Or.value,
    ParseDate: runtime.Stdlib_ParseDate.value,
    ParseFloat: runtime.Stdlib_ParseFloat.value,
    ParseInt: runtime.Stdlib_ParseInt.value,
    ParseJson: runtime.Stdlib_ParseJson.value,
    Pow: runtime.Stdlib_Pow.value,
    Push: runtime.Stdlib_Push.value,
    PushFront: runtime.Stdlib_PushFront.value,
    Remainder: runtime.Stdlib_Remainder.value,
    Replace: runtime.Stdlib_Replace.value,
    ResolveArgs: runtime.Stdlib_ResolveArgs.value,
    ResolveDeep: runtime.Stdlib_ResolveDeep.value,
    ResolveQueryBranch: runtime.Stdlib_ResolveQueryBranch.value,
    ResolveQueryLeaf: runtime.Stdlib_ResolveQueryLeaf.value,
    ResolveShallow: runtime.Stdlib_ResolveShallow.value,
    Round: runtime.Stdlib_Round.value,
    Scan: runtime.Stdlib_Scan.value,
    Sequence: runtime.Stdlib_Sequence.value,
    Set: runtime.Stdlib_Set.value,
    SetVariable: runtime.Stdlib_SetVariable.value,
    Skip: runtime.Stdlib_Skip.value,
    Slice: runtime.Stdlib_Slice.value,
    Split: runtime.Stdlib_Split.value,
    StartsWith: runtime.Stdlib_StartsWith.value,
    StringifyJson: runtime.Stdlib_StringifyJson.value,
    Subtract: runtime.Stdlib_Subtract.value,
    Take: runtime.Stdlib_Take.value,
    Throw: runtime.Stdlib_Throw.value,
    ToRequest: runtime.Stdlib_ToRequest.value,
    ToString: runtime.Stdlib_ToString.value,
    Urlencode: runtime.Stdlib_Urlencode.value,
    Values: runtime.Stdlib_Values.value,
    Zip: runtime.Stdlib_Zip.value,
  };
}

export function createRuntime(runtime) {
  const constants = {
    TermType: createTermTypes(runtime),
    ConditionType: createConditionTypes(runtime),
    Stdlib: createStdlib(runtime),
  };
  return {
    FALSE,
    TRUE,
    NULL,
    TermType: constants.TermType,
    ConditionType: constants.ConditionType,
    Stdlib: constants.Stdlib,
    exports: runtime,
    createNil() {
      return runtime.createNil();
    },
    isNil(value) {
      return runtime.isNil(value);
    },
    createBoolean(value) {
      return runtime.createBoolean(value);
    },
    isBoolean(value) {
      return runtime.isBoolean(value);
    },
    getBooleanValue(value) {
      return runtime.getBooleanValue(value) === 1;
    },
    createInt(value) {
      return runtime.createInt(value);
    },
    isInt(value) {
      return runtime.isInt(value);
    },
    getIntValue(value) {
      return runtime.getIntValue(value);
    },
    createFloat(value) {
      return runtime.createFloat(value);
    },
    isFloat(value) {
      return runtime.isFloat(value);
    },
    getFloatValue(value) {
      return runtime.getFloatValue(value);
    },
    createSymbol(id) {
      return runtime.createSymbol(id);
    },
    isSymbol(value) {
      return runtime.isSymbol(value);
    },
    getSymbolId(value) {
      return u32(runtime.getSymbolId(value));
    },
    createString(value) {
      const bytes = new TextEncoder().encode(value);
      const length = bytes.length;
      const instance = runtime.allocateString(length);
      const offset = runtime.getStringOffset(instance);
      new Uint8Array(runtime.memory.buffer, offset, length).set(bytes);
      return runtime.initString(instance, length);
    },
    isString(value) {
      return runtime.isString(value);
    },
    getStringOffset(value) {
      return runtime.getStringOffset(value);
    },
    getStringLength(value) {
      return runtime.getStringLength(value);
    },
    getStringValue(value) {
      const offset = runtime.getStringOffset(value);
      const length = runtime.getStringLength(value);
      return new TextDecoder('utf-8').decode(new Uint8Array(runtime.memory.buffer, offset, length));
    },
    createDate(timestamp) {
      return runtime.createDate(BigInt(timestamp));
    },
    isDate(value) {
      return runtime.isDate(value);
    },
    getDateTimestamp(value) {
      return Number(runtime.getDateTimestamp(value));
    },
    createSignal(condition) {
      return runtime.createSignal(condition);
    },
    isSignal(value) {
      return runtime.isSignal(value);
    },
    getSignalConditions(value) {
      return runtime.getSignalConditions(value);
    },
    isCondition(value) {
      return runtime.isCondition(value);
    },
    getConditionType(value) {
      return u32(runtime.getConditionType(value));
    },
    createCustomCondition(type, payload, token) {
      return runtime.createCustomCondition(type, payload, token);
    },
    getCustomConditionEffectType(value) {
      return runtime.getCustomConditionEffectType(value);
    },
    getCustomConditionEffectPayload(value) {
      return runtime.getCustomConditionEffectPayload(value);
    },
    createErrorCondition(payload) {
      return runtime.createErrorCondition(payload);
    },
    getErrorConditionPayload(value) {
      return runtime.getErrorConditionPayload(value);
    },
    createTypeErrorCondition(expected, received) {
      return runtime.createTypeErrorCondition(expected, received);
    },
    getTypeErrorConditionExpected(value) {
      return runtime.getTypeErrorConditionExpected(value);
    },
    getTypeErrorConditionReceived(value) {
      return runtime.getTypeErrorConditionReceived(value);
    },
    createPendingCondition() {
      return runtime.createPendingCondition();
    },
    createInvalidFunctionTargetCondition(target) {
      return runtime.createInvalidFunctionTargetCondition(target);
    },
    getInvalidFunctionTargetConditionTarget(value) {
      return runtime.getInvalidFunctionTargetConditionTarget(value);
    },
    createInvalidFunctionArgsCondition(target, args) {
      return runtime.createInvalidFunctionArgsCondition(target, args);
    },
    getInvalidFunctionArgsConditionTarget(value) {
      return runtime.getInvalidFunctionArgsConditionTarget(value);
    },
    getInvalidFunctionArgsConditionArgs(value) {
      return runtime.getInvalidFunctionArgsConditionArgs(value);
    },
    createEffect(condition) {
      return runtime.createEffect(condition);
    },
    isEffect(value) {
      return runtime.isEffect(value);
    },
    getEffectCondition(value) {
      return runtime.getEffectCondition(value);
    },
    createBuiltin(target) {
      if (typeof target !== 'number') throw new Error(`Invalid builtin target: ${target}`);
      return runtime.createBuiltin(target);
    },
    isBuiltin(value) {
      return runtime.isBuiltin(value);
    },
    getBuiltinUid(value) {
      return u32(runtime.getBuiltinUid(value));
    },
    createCompiled(target, num_args) {
      return runtime.createCompiled(target, num_args);
    },
    isCompiled(value) {
      return runtime.isCompiled(value);
    },
    getCompiledTarget(value) {
      return u32(runtime.getCompiledTarget(value));
    },
    getCompiledNumArgs(value) {
      return u32(runtime.getCompiledNumArgs(value));
    },
    createPartial(target, args) {
      return runtime.createPartial(target, args);
    },
    isPartial(value) {
      return runtime.isPartial(value);
    },
    getPartialTarget(value) {
      return runtime.getPartialTarget(value);
    },
    getPartialArgs(value) {
      return runtime.getPartialArgs(value);
    },
    createLambda(numArgs, body) {
      return runtime.createLambda(numArgs, body);
    },
    isLambda(value) {
      return runtime.isLambda(value);
    },
    getLambdaNumArgs(value) {
      return u32(runtime.getLambdaNumArgs(value));
    },
    getLambdaBody(value) {
      return runtime.getLambdaBody(value);
    },
    createVariable(stackOffset) {
      return runtime.createVariable(stackOffset);
    },
    isVariable(value) {
      return runtime.isVariable(value);
    },
    getVariableStackOffset(value) {
      return u32(runtime.getVariableStatckOffset(value));
    },
    createLet(initializer, body) {
      return runtime.createLet(initializer, body);
    },
    isLet(value) {
      return runtime.isLet(value);
    },
    getLetInitializer(value) {
      return u32(runtime.getLetInitializer(value));
    },
    getLetBody(value) {
      return runtime.getLetBody(value);
    },
    createApplication(target, args) {
      return runtime.createApplication(target, args);
    },
    isApplication(value) {
      return runtime.isApplication(value);
    },
    getApplicationTarget(value) {
      return runtime.getApplicationTarget(value);
    },
    getApplicationArgs(value) {
      return runtime.getApplicationArgs(value);
    },
    createList(items) {
      const instance = runtime.allocateList(items.length);
      const offset = runtime.getListItems(instance);
      new Uint32Array(runtime.memory.buffer, offset, items.length).set(items);
      return runtime.initList(instance, items.length);
    },
    isList(value) {
      return runtime.isList(value);
    },
    getListLength(list, index) {
      return u32(runtime.getListLength(list, index));
    },
    getListItem(list, index) {
      return runtime.getListItem(list, index);
    },
    getListItems(list) {
      const numItems = runtime.getListLength(list);
      return Array.from({ length: numItems }, (_, index) => runtime.getListItem(list, index));
    },
    createEmptyList() {
      return runtime.createEmptyList();
    },
    createUnitList(value) {
      return runtime.createUnitList(value);
    },
    createPair(left, right) {
      return runtime.createPair(left, right);
    },
    createTriple(first, second, third) {
      return runtime.createTriple(first, second, third);
    },
    createRecord(keys, values) {
      return runtime.createRecord(keys, values);
    },
    isRecord(value) {
      return runtime.isRecord(value);
    },
    getRecordKeys(value) {
      return runtime.getRecordKeys(value);
    },
    getRecordValues(value) {
      return runtime.getRecordValues(value);
    },
    getRecordField(value, key) {
      return runtime.getRecordField(value, key);
    },
    createConstructor(keys) {
      return runtime.createConstructor(keys);
    },
    isConstructor(value) {
      return runtime.isConstructor(value);
    },
    getConstructorKeys(value) {
      return runtime.getConstructorKeys(value);
    },
    createHashmap(entries) {
      const instance = runtime.allocateHashmap(runtime.defaultHashmapCapacity(entries.length));
      entries.forEach(([key, value]) => {
        runtime.insertHashmapEntry(instance, key, value);
      });
      return runtime.initHashmap(instance, entries.length);
    },
    isHashmap(value) {
      return runtime.isHashmap(value);
    },
    getHashmapCapacity(value) {
      return runtime.getHashmapCapacity(value);
    },
    getHashmapNumEntries(value) {
      return runtime.getHashmapNumEntries(value);
    },
    getHashmapValue(value, key) {
      return runtime.getHashmapValue(value, key);
    },
    createHashset(values) {
      const entries = runtime.allocateHashmap(runtime.defaultHashmapCapacity(values.length));
      const nil = runtime.createNil();
      values.forEach((value) => {
        runtime.insertHashmapEntry(entries, value, nil);
      });
      return runtime.createHashset(runtime.initHashmap(entries, values.length));
    },
    isHashset(value) {
      return runtime.isHashset(value);
    },
    getHashsetNumEntries(value) {
      return runtime.getHashsetNumEntries(value);
    },
    hasHashsetValue(value, key) {
      return Boolean(runtime.hasHashsetValue(value, key));
    },
    createTree(left, right) {
      return runtime.createTree(left, right);
    },
    isTree(value) {
      return runtime.isTree(value);
    },
    getTreeLeft(value) {
      return runtime.getTreeLeft(value);
    },
    getTreeRight(value) {
      return runtime.getTreeRight(value);
    },
    getTreeLength(value) {
      return runtime.getTreeLength(value);
    },
    createEmptyIterator() {
      return runtime.createEmptyIterator();
    },
    createOnceIterator(value) {
      return runtime.createOnceIterator(value);
    },
    createRepeatIterator(value) {
      return runtime.createRepeatIterator(value);
    },
    createSkipIterator(source, count) {
      return runtime.createSkipIterator(source, count);
    },
    createTakeIterator(source, count) {
      return runtime.createTakeIterator(source, count);
    },
    createZipIterator(left, right) {
      return runtime.createZipIterator(left, right);
    },
    createMapIterator(source, iteratee) {
      return runtime.createMapIterator(source, iteratee);
    },
    createFilterIterator(source, predicate) {
      return runtime.createFilterIterator(source, predicate);
    },
    createFlattenIterator(source) {
      return runtime.createFlattenIterator(source);
    },
    createEvaluateIterator(source) {
      return runtime.createEvaluateIterator(source);
    },
    createIntegersIterator() {
      return runtime.createIntegersIterator();
    },
    createIntersperseIterator(source, separator) {
      return runtime.createIntersperseIterator(source, separator);
    },
    createRangeIterator(offset, length) {
      return runtime.createRangeIterator(offset, length);
    },
    createHashmapKeysIterator(source) {
      return runtime.createHashmapKeysIterator(source);
    },
    createHashmapValuesIterator(source) {
      return runtime.createHashmapValuesIterator(source);
    },
    hash(value) {
      return u32(runtime.getTermHash(value));
    },
    equals(left, right) {
      return runtime.equals(left, right) === 1;
    },
    evaluate(value, state) {
      return runtime.evaluate(value, state);
    },
    arity(value) {
      return runtime.arity(value);
    },
    format(value) {
      if (value === NULL) return 'NULL';
      const offset = runtime.getAllocatorOffset();
      const length = runtime.debug(value, offset) - offset;
      const bytes = new Uint8Array(runtime.memory.buffer, offset, length);
      const stringValue = new TextDecoder('utf-8').decode(bytes);
      runtime.deallocate(offset + length, length);
      return stringValue;
    },
    display(value) {
      const offset = runtime.getAllocatorOffset();
      const length = runtime.display(value, offset) - offset;
      const bytes = new Uint8Array(runtime.memory.buffer, offset, length);
      const stringValue = new TextDecoder('utf-8').decode(bytes);
      runtime.deallocate(offset + length, length);
      return stringValue;
    },
    inspectHeap(offset, length) {
      return new Uint32Array(runtime.memory.buffer, offset, length);
    },
  };
}
