// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use strum_macros::EnumIter;

use crate::compiler::{TypeSignature, ValueType};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, EnumIter)]
pub enum RuntimeBuiltin {
    Initialize,
    Evaluate,
    Apply,
    CombineDependencies,
    CombineSignals,
    IsSignal,
    AllocateCell,
    AllocateHashmap,
    AllocateList,
    AllocateString,
    CreateApplication,
    CreateBoolean,
    CreateBuiltin,
    CreateCustomCondition,
    CreatePendingCondition,
    CreateErrorCondition,
    CreateTypeErrorCondition,
    CreateInvalidFunctionTargetCondition,
    CreateInvalidFunctionArgsCondition,
    CreateInvalidPointerCondition,
    CreateConstructor,
    CreateEmptyList,
    CreateEffect,
    CreateFloat,
    CreateHashset,
    CreateInt,
    CreateLambda,
    CreateLazyResult,
    CreateNil,
    CreatePartial,
    CreatePointer,
    CreateRecord,
    CreateTimestamp,
    CreateSignal,
    CreateSymbol,
    CreateTree,
    CreateEmptyIterator,
    CreateEvaluateIterator,
    CreateFilterIterator,
    CreateFlattenIterator,
    CreateHashmapKeysIterator,
    CreateHashmapValuesIterator,
    CreateIndexedAccessorIterator,
    CreateIntegersIterator,
    CreateIntersperseIterator,
    CreateMapIterator,
    CreateOnceIterator,
    CreateRangeIterator,
    CreateRepeatIterator,
    CreateSkipIterator,
    CreateTakeIterator,
    CreateZipIterator,
    GetBooleanValue,
    GetListItem,
    GetListLength,
    GetStateValue,
    GetStringCharOffset,
    InitHashmap,
    InitList,
    InitString,
    InsertHashmapEntry,
    SetCellField,
    SetListItem,
}

impl RuntimeBuiltin {
    pub fn signature(self) -> TypeSignature {
        match self {
            RuntimeBuiltin::Initialize => TypeSignature::new((), ()),
            RuntimeBuiltin::Evaluate => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::Apply => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CombineDependencies => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CombineSignals => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::IsSignal => TypeSignature::new(ValueType::HeapPointer, ValueType::U32),
            RuntimeBuiltin::AllocateCell => {
                TypeSignature::new(ValueType::U32, ValueType::HeapPointer)
            }
            RuntimeBuiltin::AllocateHashmap => {
                TypeSignature::new(ValueType::U32, ValueType::HeapPointer)
            }
            RuntimeBuiltin::AllocateList => {
                TypeSignature::new(ValueType::U32, ValueType::HeapPointer)
            }
            RuntimeBuiltin::AllocateString => {
                TypeSignature::new(ValueType::U32, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateApplication => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateBoolean => {
                TypeSignature::new(ValueType::U32, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateBuiltin => {
                TypeSignature::new(ValueType::FunctionPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateCustomCondition => TypeSignature::new(
                (
                    ValueType::HeapPointer,
                    ValueType::HeapPointer,
                    ValueType::HeapPointer,
                ),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreatePendingCondition => {
                TypeSignature::new((), ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateErrorCondition => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateTypeErrorCondition => TypeSignature::new(
                (ValueType::U32, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateInvalidFunctionTargetCondition => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateInvalidFunctionArgsCondition => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateInvalidPointerCondition => {
                TypeSignature::new((), ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateConstructor => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateEmptyList => TypeSignature::new((), ValueType::HeapPointer),
            RuntimeBuiltin::CreateEffect => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateFloat => {
                TypeSignature::new(ValueType::F64, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateHashset => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateInt => TypeSignature::new(ValueType::F64, ValueType::HeapPointer),
            RuntimeBuiltin::CreateLambda => TypeSignature::new(
                (ValueType::U32, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateLazyResult => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateNil => TypeSignature::new((), ValueType::HeapPointer),
            RuntimeBuiltin::CreatePartial => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreatePointer => {
                TypeSignature::new(ValueType::U32, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateRecord => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateSignal => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateSymbol => {
                TypeSignature::new(ValueType::I64, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateTimestamp => {
                TypeSignature::new(ValueType::I64, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateTree => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateEmptyIterator => TypeSignature::new((), ValueType::HeapPointer),
            RuntimeBuiltin::CreateEvaluateIterator => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateFilterIterator => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateFlattenIterator => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateHashmapKeysIterator => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateHashmapValuesIterator => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateIndexedAccessorIterator => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::U32),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateIntegersIterator => {
                TypeSignature::new((), ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateIntersperseIterator => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateMapIterator => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateOnceIterator => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateRangeIterator => {
                TypeSignature::new((ValueType::I64, ValueType::U32), ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateRepeatIterator => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::CreateSkipIterator => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::U32),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateTakeIterator => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::U32),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::CreateZipIterator => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::GetBooleanValue => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::U32)
            }
            RuntimeBuiltin::GetListItem => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::U32),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::GetListLength => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::U32)
            }
            RuntimeBuiltin::GetStateValue => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::HeapPointer),
                (ValueType::HeapPointer, ValueType::HeapPointer),
            ),
            RuntimeBuiltin::GetStringCharOffset => {
                TypeSignature::new((ValueType::HeapPointer, ValueType::U32), ValueType::U32)
            }
            RuntimeBuiltin::InitHashmap => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::InitList => TypeSignature::new(
                (ValueType::HeapPointer, ValueType::U32),
                ValueType::HeapPointer,
            ),
            RuntimeBuiltin::InitString => {
                TypeSignature::new(ValueType::HeapPointer, ValueType::HeapPointer)
            }
            RuntimeBuiltin::InsertHashmapEntry => TypeSignature::new(
                (
                    ValueType::HeapPointer,
                    ValueType::HeapPointer,
                    ValueType::HeapPointer,
                ),
                (),
            ),
            RuntimeBuiltin::SetCellField => TypeSignature::new(
                (
                    ValueType::HeapPointer,
                    ValueType::U32,
                    ValueType::HeapPointer,
                ),
                (),
            ),
            RuntimeBuiltin::SetListItem => TypeSignature::new(
                (
                    ValueType::HeapPointer,
                    ValueType::U32,
                    ValueType::HeapPointer,
                ),
                (),
            ),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            RuntimeBuiltin::Initialize => "_initialize",
            RuntimeBuiltin::Evaluate => "evaluate",
            RuntimeBuiltin::Apply => "apply",
            RuntimeBuiltin::CombineDependencies => "combineDependencies",
            RuntimeBuiltin::CombineSignals => "combineSignals",
            RuntimeBuiltin::IsSignal => "isSignal",
            RuntimeBuiltin::AllocateCell => "allocateCell",
            RuntimeBuiltin::AllocateHashmap => "allocateHashmap",
            RuntimeBuiltin::AllocateList => "allocateList",
            RuntimeBuiltin::AllocateString => "allocateString",
            RuntimeBuiltin::CreateApplication => "createApplication",
            RuntimeBuiltin::CreateBoolean => "createBoolean",
            RuntimeBuiltin::CreateBuiltin => "createBuiltin",
            RuntimeBuiltin::CreateCustomCondition => "createCustomCondition",
            RuntimeBuiltin::CreatePendingCondition => "createPendingCondition",
            RuntimeBuiltin::CreateErrorCondition => "createErrorCondition",
            RuntimeBuiltin::CreateTypeErrorCondition => "createTypeErrorCondition",
            RuntimeBuiltin::CreateInvalidFunctionTargetCondition => {
                "createInvalidFunctionTargetCondition"
            }
            RuntimeBuiltin::CreateInvalidFunctionArgsCondition => {
                "createInvalidFunctionArgsCondition"
            }
            RuntimeBuiltin::CreateInvalidPointerCondition => "createInvalidPointerCondition",
            RuntimeBuiltin::CreateConstructor => "createConstructor",
            RuntimeBuiltin::CreateEmptyList => "createEmptyList",
            RuntimeBuiltin::CreateEffect => "createEffect",
            RuntimeBuiltin::CreateFloat => "createFloat",
            RuntimeBuiltin::CreateHashset => "createHashset",
            RuntimeBuiltin::CreateInt => "createInt",
            RuntimeBuiltin::CreateLambda => "createLambda",
            RuntimeBuiltin::CreateLazyResult => "createLazyResult",
            RuntimeBuiltin::CreateNil => "createNil",
            RuntimeBuiltin::CreatePartial => "createPartial",
            RuntimeBuiltin::CreatePointer => "createPointer",
            RuntimeBuiltin::CreateRecord => "createRecord",
            RuntimeBuiltin::CreateSignal => "createSignal",
            RuntimeBuiltin::CreateSymbol => "createSymbol",
            RuntimeBuiltin::CreateTimestamp => "createTimestamp",
            RuntimeBuiltin::CreateTree => "createTree",
            RuntimeBuiltin::CreateEmptyIterator => "createEmptyIterator",
            RuntimeBuiltin::CreateEvaluateIterator => "createEvaluateIterator",
            RuntimeBuiltin::CreateFilterIterator => "createFilterIterator",
            RuntimeBuiltin::CreateFlattenIterator => "createFlattenIterator",
            RuntimeBuiltin::CreateHashmapKeysIterator => "createHashmapKeysIterator",
            RuntimeBuiltin::CreateHashmapValuesIterator => "createHashmapValuesIterator",
            RuntimeBuiltin::CreateIndexedAccessorIterator => "createIndexedAccessorIterator",
            RuntimeBuiltin::CreateIntegersIterator => "createIntegersIterator",
            RuntimeBuiltin::CreateIntersperseIterator => "createIntersperseIterator",
            RuntimeBuiltin::CreateMapIterator => "createMapIterator",
            RuntimeBuiltin::CreateOnceIterator => "createOnceIterator",
            RuntimeBuiltin::CreateRangeIterator => "createRangeIterator",
            RuntimeBuiltin::CreateRepeatIterator => "createRepeatIterator",
            RuntimeBuiltin::CreateSkipIterator => "createSkipIterator",
            RuntimeBuiltin::CreateTakeIterator => "createTakeIterator",
            RuntimeBuiltin::CreateZipIterator => "createZipIterator",
            RuntimeBuiltin::GetBooleanValue => "getBooleanValue",
            RuntimeBuiltin::GetListItem => "getListItem",
            RuntimeBuiltin::GetListLength => "getListLength",
            RuntimeBuiltin::GetStateValue => "getStateValue",
            RuntimeBuiltin::GetStringCharOffset => "getStringCharOffset",
            RuntimeBuiltin::InitHashmap => "initHashmap",
            RuntimeBuiltin::InitList => "initList",
            RuntimeBuiltin::InitString => "initString",
            RuntimeBuiltin::InsertHashmapEntry => "insertHashmapEntry",
            RuntimeBuiltin::SetCellField => "setCellField",
            RuntimeBuiltin::SetListItem => "setListItem",
        }
    }
}
