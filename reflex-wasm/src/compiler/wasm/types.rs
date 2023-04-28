// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use walrus::{self, ir::InstrSeqType, ModuleTypes, ValType};

use crate::compiler::{ParamsSignature, TypeSignature, ValueType};

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
