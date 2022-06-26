// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
const FALSE = 0;
const TRUE = 1;
const NULL = -1;

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
    InvalidAccessor: runtime.ConditionType_InvalidAccessorCondition.value,
    InvalidJson: runtime.ConditionType_InvalidJsonCondition.value,
    InvalidPointer: runtime.ConditionType_InvalidPointerCondition.value,
  };
}

function createStdlib(runtime) {
  return {
    Abs: runtime.Stdlib_Abs.value,
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
    Divide: runtime.Stdlib_Divide.value,
    Effect: runtime.Stdlib_Effect.value,
    EndsWith: runtime.Stdlib_EndsWith.value,
    Eq: runtime.Stdlib_Eq.value,
    Equal: runtime.Stdlib_Equal.value,
    Floor: runtime.Stdlib_Floor.value,
    Fold: runtime.Stdlib_Fold.value,
    Get: runtime.Stdlib_Get.value,
    Gt: runtime.Stdlib_Gt.value,
    Gte: runtime.Stdlib_Gte.value,
    Has: runtime.Stdlib_Has.value,
    Hash: runtime.Stdlib_Hash.value,
    Identity: runtime.Stdlib_Identity.value,
    If: runtime.Stdlib_If.value,
    IfPending: runtime.Stdlib_IfPending.value,
    IfError: runtime.Stdlib_IfError.value,
    Iterate: runtime.Stdlib_Iterate.value,
    Keys: runtime.Stdlib_Keys.value,
    Length: runtime.Stdlib_Length.value,
    Lt: runtime.Stdlib_Lt.value,
    Lte: runtime.Stdlib_Lte.value,
    Max: runtime.Stdlib_Max.value,
    Min: runtime.Stdlib_Min.value,
    Multiply: runtime.Stdlib_Multiply.value,
    Not: runtime.Stdlib_Not.value,
    Or: runtime.Stdlib_Or.value,
    Pow: runtime.Stdlib_Pow.value,
    Push: runtime.Stdlib_Push.value,
    PushFront: runtime.Stdlib_PushFront.value,
    Remainder: runtime.Stdlib_Remainder.value,
    Replace: runtime.Stdlib_Replace.value,
    ResolveDeep: runtime.Stdlib_ResolveDeep.value,
    ResolveShallow: runtime.Stdlib_ResolveShallow.value,
    Round: runtime.Stdlib_Round.value,
    Sequence: runtime.Stdlib_Sequence.value,
    Set: runtime.Stdlib_Set.value,
    Skip: runtime.Stdlib_Skip.value,
    Slice: runtime.Stdlib_Slice.value,
    Split: runtime.Stdlib_Split.value,
    StartsWith: runtime.Stdlib_StartsWith.value,
    Subtract: runtime.Stdlib_Subtract.value,
    Take: runtime.Stdlib_Take.value,
    Values: runtime.Stdlib_Values.value,
    Zip: runtime.Stdlib_Zip.value,
    ParseJson: runtime.Stdlib_ParseJson.value,
    StringifyJson: runtime.Stdlib_StringifyJson.value,
    ResolveQueryBranch: runtime.Stdlib_ResolveQueryBranch.value,
    ResolveQueryLeaf: runtime.Stdlib_ResolveQueryLeaf.value,
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
    asNil(value) {
      return runtime.isNil(value) ? value : null;
    },
    createBoolean(value) {
      return runtime.createBoolean(value);
    },
    isBoolean(value) {
      return runtime.isBoolean(value);
    },
    asBoolean(value) {
      return runtime.isBoolean(value) ? value : null;
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
    asInt(value) {
      return runtime.isInt(value) ? value : null;
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
    asFloat(value) {
      return runtime.isFloat(value) ? value : null;
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
    asSymbol(value) {
      return runtime.isSymbol(value) ? value : null;
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
    asString(value) {
      return runtime.isString(value) ? value : null;
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
      return new TextDecoder().decode(new Uint8Array(runtime.memory.buffer, offset, length));
    },
    createSignal(condition) {
      return runtime.createSignal(condition);
    },
    isSignal(value) {
      return runtime.isSignal(value);
    },
    asSignal(value) {
      return runtime.isSignal(value) ? value : null;
    },
    getSignalConditions(value) {
      return runtime.getSignalConditions(value);
    },
    isCondition(value) {
      return runtime.isCondition(value);
    },
    asCondition(value) {
      return runtime.isCondition(value) ? value : null;
    },
    getConditionType(value) {
      return u32(runtime.getConditionType(value));
    },
    createCustomCondition(type, payload) {
      return runtime.createCustomCondition(type, payload);
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
    getTypeErrorConditionType(value) {
      return runtime.getTypeErrorConditionType(value);
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
    createInvalidAccessorCondition(target, key) {
      return runtime.createInvalidAccessorCondition(target, key);
    },
    getInvalidAccessorConditionTarget(value) {
      return runtime.getInvalidAccessorConditionTarget(value);
    },
    getInvalidAccessorConditionKey(value) {
      return runtime.getInvalidAccessorConditionKey(value);
    },
    createEffect(condition) {
      return runtime.createEffect(condition);
    },
    isEffect(value) {
      return runtime.isEffect(value);
    },
    asEffect(value) {
      return runtime.isEffect(value) ? value : null;
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
    asBuiltin(value) {
      return runtime.isBuiltin(value) ? value : null;
    },
    getBuiltinUid(value) {
      return u32(runtime.getBuiltinUid(value));
    },
    createPartial(target, args) {
      return runtime.createPartial(target, args);
    },
    isPartial(value) {
      return runtime.isPartial(value);
    },
    asPartial(value) {
      return runtime.isPartial(value) ? value : null;
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
    asLambda(value) {
      return runtime.isLambda(value) ? value : null;
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
    asVariable(value) {
      return runtime.isVariable(value) ? value : null;
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
    asLet(value) {
      return runtime.isLet(value) ? value : null;
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
    asApplication(value) {
      return runtime.isApplication(value) ? value : null;
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
    asList(value) {
      return runtime.isList(value) ? value : null;
    },
    getListLength(list, index) {
      return u32(runtime.getListLength(list, index));
    },
    getListItem(list, index) {
      return runtime.getListItem(list, index);
    },
    getListItems(list) {
      return getListItems(runtime, list);
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
    asRecord(value) {
      return runtime.isRecord(value) ? value : null;
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
    asConstructor(value) {
      return runtime.isConstructor(value) ? value : null;
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
    asHashmap(value) {
      return runtime.isHashmap(value) ? value : null;
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
    asHashset(value) {
      return runtime.isHashset(value) ? value : null;
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
    asTree(value) {
      return runtime.isTree(value) ? value : null;
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
      return formatTerm(runtime, value, constants);
    },
    inspectHeap(offset, length) {
      return new Uint32Array(runtime.memory.buffer, offset, length);
    },
  };
}

function u32(value) {
  // Convert 32-bit two's complement signed integer to unsigned integer
  return value >= 0 ? value : 0xffffffff + 1 + value;
}

function formatTerm(runtime, value, constants) {
  switch (runtime.getTermType(value)) {
    case constants.TermType.Nil:
      return formatNil(runtime, value, constants);
    case constants.TermType.Boolean:
      return formatBoolean(runtime, value, constants);
    case constants.TermType.Int:
      return formatInt(runtime, value, constants);
    case constants.TermType.Float:
      return formatFloat(runtime, value, constants);
    case constants.TermType.String:
      return formatString(runtime, value, constants);
    case constants.TermType.Symbol:
      return formatSymbol(runtime, value, constants);
    case constants.TermType.Builtin:
      return formatBuiltin(runtime, value, constants);
    case constants.TermType.Partial:
      return formatPartial(runtime, value, constants);
    case constants.TermType.Application:
      return formatApplication(runtime, value, constants);
    case constants.TermType.Constructor:
      return formatConstructor(runtime, value, constants);
    case constants.TermType.Lambda:
      return formatLambda(runtime, value, constants);
    case constants.TermType.Variable:
      return formatVariable(runtime, value, constants);
    case constants.TermType.Let:
      return formatLet(runtime, value, constants);
    case constants.TermType.List:
      return formatList(runtime, value, constants);
    case constants.TermType.Record:
      return formatRecord(runtime, value, constants);
    case constants.TermType.Hashmap:
      return formatHashmap(runtime, value, constants);
    case constants.TermType.Hashset:
      return formatHashset(runtime, value, constants);
    case constants.TermType.Tree:
      return formatTree(runtime, value, constants);
    case constants.TermType.Signal:
      return formatSignal(runtime, value, constants);
    case constants.TermType.Condition:
      return formatCondition(runtime, value, constants);
    case constants.TermType.Effect:
      return formatEffect(runtime, value, constants);
    case constants.TermType.Cell:
      return formatCell(runtime, value, constants);
    case constants.TermType.Pointer:
      return formatPointer(runtime, value, constants);
    default: {
      const typeName = getEnumVariantName(constants.TermType, runtime.getTermType(value));
      if (typeName) {
        return typeName;
      } else {
        throw new Error(`Unexpected term type at offset ${value}: ${runtime.getTermType(value)}`);
      }
    }
  }
}

function formatNil(_runtime, _value, _constants) {
  return 'null';
}

function formatBoolean(runtime, value, _constants) {
  return runtime.getBooleanValue(value) === 1 ? 'true' : 'false';
}

function formatInt(runtime, value, _constants) {
  return runtime.getIntValue(value).toString(10);
}

function formatFloat(runtime, value, _constants) {
  const floatValue = runtime.getFloatValue(value);
  if (floatValue === Math.floor(floatValue)) {
    return floatValue.toFixed(1);
  } else {
    return floatValue.toString(10);
  }
}

function formatString(runtime, value, _constants) {
  const offset = runtime.getStringOffset(value);
  const length = runtime.getStringLength(value);
  return JSON.stringify(
    new TextDecoder('utf-8').decode(new Uint8Array(runtime.memory.buffer, offset, length)),
  );
}

function formatSymbol(runtime, value, _constants) {
  return `Symbol(${u32(runtime.getSymbolId(value))})`;
}

function formatSignal(runtime, value, constants) {
  return `{${formatTerm(runtime, runtime.getSignalConditions(value), constants)}}`;
}

function formatCondition(runtime, value, constants) {
  const type = u32(runtime.getConditionType(value));
  const signalName = getEnumVariantName(constants.ConditionType, type);
  if (!signalName) throw new Error(`Unexpected condition type: ${type}`);
  const payload = formatConditionPayload(runtime, type, value, constants);
  return `<${signalName}${payload ? `:${payload}` : ''}>`;
}

function formatConditionPayload(runtime, type, value, constants) {
  switch (type) {
    case constants.ConditionType.Custom:
      return `${formatTerm(
        runtime,
        runtime.getCustomConditionEffectType(value),
        constants,
      )}:${formatTerm(runtime, runtime.getCustomConditionPayload(value), constants)}`;
    case constants.ConditionType.Error:
      return formatTerm(runtime, runtime.getErrorConditionPayload(value), constants);
    case constants.ConditionType.TypeError: {
      const expected = runtime.getTypeErrorConditionExpected(value);
      return `${
        expected === NULL ? '' : `${getEnumVariantName(constants.TermType, expected)}`
      }:${formatTerm(runtime, runtime.getTypeErrorConditionReceived(value), constants)}`;
    }
    case constants.ConditionType.InvalidFunctionTarget:
      console.error(`INVALID TARGET: ${runtime.getInvalidFunctionTargetConditionTarget(value)}`);
      return formatTerm(runtime, runtime.getInvalidFunctionTargetConditionTarget(value), constants);
    case constants.ConditionType.InvalidFunctionArgs:
      return `${formatTerm(
        runtime,
        runtime.getInvalidFunctionArgsConditionTarget(value),
        constants,
      )}(${getListItems(runtime, runtime.getInvalidFunctionArgsConditionArgs(value))
        .map((arg) => formatTerm(runtime, arg, constants))
        .join(', ')})`;
    case constants.ConditionType.InvalidAccessor:
      return `${formatTerm(
        runtime,
        runtime.getInvalidAccessorConditionTarget(value),
        constants,
      )},${formatTerm(runtime, runtime.getInvalidAccessorConditionKey(value), constants)}`;
    case constants.ConditionType.InvalidJson:
      return `${formatTerm(
        runtime,
        runtime.getInvalidJsonConditionSource(value),
        constants,
      )}:${runtime.getInvalidJsonConditionOffset(value)}`;
    case constants.ConditionType.Pending:
    case constants.ConditionType.InvalidPointer:
    default:
      return null;
  }
}

function formatEffect(runtime, value, constants) {
  return `(!${formatTerm(runtime, runtime.getEffectCondition(value), constants)})`;
}

function formatBuiltin(runtime, value, constants) {
  const target = u32(runtime.getBuiltinUid(value));
  return getEnumVariantName(constants.Stdlib, target);
}

function formatPartial(runtime, value, constants) {
  const target = runtime.getPartialTarget(value);
  const args = runtime.getPartialArgs(value);
  return `${formatTerm(runtime, target, constants)}.bind(${getListItems(runtime, args)
    .map((arg) => formatTerm(runtime, arg, constants))
    .join(', ')})`;
}

function formatLambda(runtime, value, constants) {
  const numArgs = u32(runtime.getLambdaNumArgs(value));
  const body = runtime.getLetBody(value);
  return `(${numArgs}) => ${formatTerm(runtime, body, constants)}`;
}

function formatVariable(runtime, value, _constants) {
  return `Variable(${u32(runtime.getVariableStackOffset(value))})`;
}

function formatLet(runtime, value, constants) {
  const initializer = runtime.getLetInitializer(value);
  const body = runtime.getLetBody(value);
  return `{let ${formatTerm(runtime, initializer, constants)}; ${formatTerm(
    runtime,
    body,
    constants,
  )}}`;
}

function formatApplication(runtime, value, constants) {
  const target = runtime.getApplicationTarget(value);
  const args = runtime.getApplicationArgs(value);
  return `${formatTerm(runtime, target, constants)}(${getListItems(runtime, args)
    .map((arg) => formatTerm(runtime, arg, constants))
    .join(', ')})`;
}

function formatList(runtime, value, constants) {
  return `[${getListItems(runtime, value)
    .map((item) => formatTerm(runtime, item, constants))
    .join(', ')}]`;
}

function formatConstructor(runtime, value, constants) {
  const keys = getListItems(runtime, runtime.getConstructorKeys(value));
  return `Constructor({${keys.map((key) => formatTerm(runtime, key, constants)).join(', ')}})`;
}

function formatRecord(runtime, value, constants) {
  const keys = getListItems(runtime, runtime.getRecordKeys(value));
  const values = getListItems(runtime, runtime.getRecordValues(value));
  const entries = keys.map((key, index) => [key, values[index]]);
  if (entries.length == 0) return '{}';
  return `{ ${entries
    .map(
      ([key, value]) =>
        `${formatTerm(runtime, key, constants)}: ${formatTerm(runtime, value, constants)}`,
    )
    .join(', ')} }`;
}

function formatHashmap(runtime, value, _constants) {
  return `Map({${runtime.getHashmapNumEntries(value)}})`;
}

function formatHashset(runtime, value, _constants) {
  return `Set({${runtime.getHashsetNumEntries(value)}})`;
}

function formatTree(runtime, value, constants) {
  const left = runtime.getTreeLeft(value);
  const right = runtime.getTreeRight(value);
  return `(${left === NULL ? 'NULL' : formatTerm(runtime, left, constants)} . ${
    right === NULL ? 'NULL' : formatTerm(runtime, right, constants)
  })`;
}

function formatCell(runtime, value, _constants) {
  const numFields = runtime.getCellNumFields(value);
  return `Cell({${Array.from({ length: numFields }, (_, index) =>
    runtime.getCellField(value, index),
  ).join(', ')}})`;
}

function formatPointer(runtime, value, constants) {
  const target = runtime.getPointerTarget(value);
  return `Pointer(${formatHex(value)}:${formatTerm(runtime, target, constants)})`;
}

function getListItems(runtime, value) {
  const numItems = runtime.getListLength(value);
  return Array.from({ length: numItems }, (_, index) => runtime.getListItem(value, index));
}

function getEnumVariantName(values, variant) {
  return Object.entries(values)
    .filter(([_key, index]) => index == variant)
    .map(([key, _index]) => key)[0];
}

function formatHex(value) {
  return `0x${leftPad(value.toString(16), 8)}`;
}

function leftPad(value, length) {
  return `${Array.from({ length: length - value.toString().length }, () => '0').join('')}${value}`;
}
