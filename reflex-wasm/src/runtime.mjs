// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
const TermType = {
  Application: 0,
  Partial: 1,
  Builtin: 2,
  Effect: 3,
  Signal: 4,
  Condition: 5,
  Nil: 6,
  Boolean: 7,
  Int: 8,
  Float: 9,
  Symbol: 10,
  String: 11,
  List: 12,
  Record: 13,
  Hashmap: 14,
  Tree: 15,
  EmptyIterator: 16,
  OnceIterator: 17,
  RepeatIterator: 18,
  SkipIterator: 19,
  TakeIterator: 20,
  ChainIterator: 21,
  ZipIterator: 22,
  MapIterator: 23,
  FilterIterator: 24,
  FlattenIterator: 25,
  EvaluateIterator: 26,
  IntegersIterator: 27,
  RangeIterator: 28,
  HashmapKeysIterator: 29,
  HashmapValuesIterator: 30,
  Cell: 31,
  Pointer: 32,
};

const ConditionType = {
  Custom: 0,
  Pending: 1,
  Error: 2,
  TypeError: 3,
  InvalidFunctionTarget: 4,
  InvalidFunctionArgs: 5,
  InvalidAccessor: 6,
  InvalidJson: 7,
  InvalidPointer: 8,
};

const FALSE = 0;
const TRUE = 1;
const NULL = -1;

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
    ParseJson: runtime.Stdlib_ParseJson.value,
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
  };
}

export function createRuntime(runtime) {
  return {
    FALSE,
    TRUE,
    NULL,
    ConditionType,
    Stdlib: createStdlib(runtime),
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
    createTypeErrorCondition(type, payload) {
      return runtime.createErrorCondition(payload);
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
      return runtime.getErrorConditionType(value);
    },
    getTypeErrorConditionValue(value) {
      return runtime.getErrorConditionValue(value);
    },
    createPendingCondition() {
      return runtime.createPendingCondition();
    },
    getInvalidFunctionTargetConditionTarget(value) {
      return runtime.getInvalidFunctionTargetConditionTarget(value);
    },
    getInvalidFunctionArgsConditionTarget(value) {
      return runtime.getInvalidFunctionArgsConditionTarget(value);
    },
    getInvalidFunctionArgsConditionArgs(value) {
      return runtime.getInvalidFunctionArgsConditionArgs(value);
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
    createChainIterator(sources) {
      const instance = runtime.allocateChainIterator(sources.length);
      sources.forEach((source, index) => runtime.setChainIteratorSource(instance, index, source));
      return runtime.initChainIterator(instance);
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
    format(value) {
      if (value === NULL) return 'NULL';
      return formatTerm(runtime, value);
    },
    toJson(value) {
      return runtime.toJson(value);
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

function formatTerm(runtime, value) {
  switch (runtime.getTermType(value)) {
    case TermType.Nil:
      return formatNil(runtime, value);
    case TermType.Boolean:
      return formatBoolean(runtime, value);
    case TermType.Int:
      return formatInt(runtime, value);
    case TermType.Float:
      return formatFloat(runtime, value);
    case TermType.String:
      return formatString(runtime, value);
    case TermType.Symbol:
      return formatSymbol(runtime, value);
    case TermType.Builtin:
      return formatBuiltin(runtime, value);
    case TermType.Partial:
      return formatPartial(runtime, value);
    case TermType.Application:
      return formatApplication(runtime, value);
    case TermType.List:
      return formatList(runtime, value);
    case TermType.Record:
      return formatRecord(runtime, value);
    case TermType.Hashmap:
      return formatHashmap(runtime, value);
    case TermType.Tree:
      return formatTree(runtime, value);
    case TermType.Iterator:
      return formatIterator(runtime, value);
    case TermType.Signal:
      return formatSignal(runtime, value);
    case TermType.Condition:
      return formatCondition(runtime, value);
    case TermType.Effect:
      return formatEffect(runtime, value);
    case TermType.Cell:
      return formatCell(runtime, value);
    case TermType.Pointer:
      return formatPointer(runtime, value);
    default: {
      throw new Error(`Unexpected term type at offset ${value}: ${runtime.getTermType(value)}`);
    }
  }
}

function formatNil(_runtime, _value) {
  return 'null';
}

function formatBoolean(runtime, value) {
  return runtime.getBooleanValue(value) === 1 ? 'true' : 'false';
}

function formatInt(runtime, value) {
  return runtime.getIntValue(value).toString(10);
}

function formatFloat(runtime, value) {
  const floatValue = runtime.getFloatValue(value);
  if (floatValue === Math.floor(floatValue)) {
    return floatValue.toFixed(1);
  } else {
    return floatValue.toString(10);
  }
}

function formatString(runtime, value) {
  const offset = runtime.getStringOffset(value);
  const length = runtime.getStringLength(value);
  return JSON.stringify(
    new TextDecoder('utf-8').decode(new Uint8Array(runtime.memory.buffer, offset, length)),
  );
}

function formatSymbol(runtime, value) {
  return `Symbol(${u32(runtime.getSymbolId(value))})`;
}

function formatSignal(runtime, value) {
  return `{${formatTerm(runtime, runtime.getSignalConditions(value))}}`;
}

function formatCondition(runtime, value) {
  const type = u32(runtime.getConditionType(value));
  const signalName = getEnumVariantName(ConditionType, type);
  if (!signalName) throw new Error(`Unexpected condition type: ${type}`);
  const payload = formatConditionPayload(runtime, type, value);
  return `<${signalName}${payload ? `:${payload}` : ''}>`;
}

function formatConditionPayload(runtime, type, value) {
  switch (type) {
    case ConditionType.Custom:
      return `${formatTerm(runtime, runtime.getCustomConditionEffectType(value))}:${formatTerm(
        runtime,
        runtime.getCustomConditionEffectPayload(value),
      )}`;
    case ConditionType.Error:
      return formatTerm(runtime, runtime.getErrorConditionPayload(value));
    case ConditionType.TypeError: {
      const type = runtime.getTypeErrorConditionType(value);
      return `${type === NULL ? '' : `${getEnumVariantName(TermType, type)}`}:${formatTerm(
        runtime,
        runtime.getTypeErrorConditionValue(value),
      )}`;
    }
    case ConditionType.InvalidFunctionTarget:
      return formatTerm(runtime, runtime.getInvalidFunctionTargetConditionTarget(value));
    case ConditionType.InvalidFunctionArgs:
      return `${formatTerm(
        runtime,
        runtime.getInvalidFunctionArgsConditionTarget(value),
      )}(${getListItems(runtime, runtime.getInvalidFunctionArgsConditionArgs(value))
        .map((arg) => formatTerm(runtime, arg))
        .join(', ')})`;
    case ConditionType.InvalidAccessor:
      return `${formatTerm(runtime, runtime.getInvalidAccessorConditionTarget(value))},${formatTerm(
        runtime,
        runtime.getInvalidAccessorConditionKey(value),
      )}`;
    case ConditionType.InvalidJson:
      return `${formatTerm(
        runtime,
        runtime.getInvalidJsonConditionSource(value),
      )}:${runtime.getInvalidJsonConditionOffset(value)}`;
    case ConditionType.Pending:
    case ConditionType.InvalidPointer:
    default:
      return null;
  }
}

function formatEffect(runtime, value) {
  return `(!${formatTerm(runtime, runtime.getEffectCondition(value))})`;
}

function formatBuiltin(runtime, value) {
  const target = u32(runtime.getBuiltinUid(value));
  return getEnumVariantName(createStdlib(runtime), target);
}

function formatPartial(runtime, value) {
  const target = runtime.getPartialTarget(value);
  const args = runtime.getPartialArgs(value);
  return `${formatTerm(runtime, target)}.bind(${getListItems(runtime, args)
    .map((arg) => formatTerm(runtime, arg))
    .join(', ')})`;
}

function formatApplication(runtime, value) {
  const target = runtime.getApplicationTarget(value);
  const args = runtime.getApplicationArgs(value);
  return `${formatTerm(runtime, target)}(${getListItems(runtime, args)
    .map((arg) => formatTerm(runtime, arg))
    .join(', ')})`;
}

function formatList(runtime, value) {
  return `[${getListItems(runtime, value)
    .map((item) => formatTerm(runtime, item))
    .join(', ')}]`;
}

function formatRecord(runtime, value) {
  const keys = getListItems(runtime, runtime.getRecordKeys(value));
  const values = getListItems(runtime, runtime.getRecordValues(value));
  const entries = keys.map((key, index) => [key, values[index]]);
  if (entries.length == 0) return '{}';
  return `{ ${entries
    .map(([key, value]) => `${formatTerm(runtime, key)}: ${formatTerm(runtime, value)}`)
    .join(', ')} }`;
}

function formatHashmap(runtime, value) {
  return `Map({${runtime.getHashmapNumEntries(value)}})`;
}

function formatTree(runtime, value) {
  const left = runtime.getTreeLeft(value);
  const right = runtime.getTreeRight(value);
  return `(${left === NULL ? 'NULL' : formatTerm(runtime, left)} . ${
    right === NULL ? 'NULL' : formatTerm(runtime, right)
  })`;
}

function formatIterator(runtime, value) {
  const length = u32(runtime.getIteratorSizeHint(value));
  return `[...${length === 0xffffffff ? '' : `${length.toString(10)} items`}]`;
}

function formatCell(runtime, value) {
  const numFields = runtime.getCellNumFields(value);
  return `Cell({${Array.from({ length: numFields }, (_, index) =>
    runtime.getCellField(value, index),
  ).join(', ')}})`;
}

function formatPointer(runtime, value) {
  const target = runtime.getPointerTarget(value);
  return `Pointer(${formatTerm(runtime, target)})`;
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
