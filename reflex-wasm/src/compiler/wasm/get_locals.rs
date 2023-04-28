// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::HashSet;

use walrus::{ir, LocalFunction, LocalId};

pub trait WasmLocals {
    fn get_locals(&self, enclosing_function: &LocalFunction) -> Vec<LocalId>;
}

impl WasmLocals for LocalFunction {
    fn get_locals(&self, _enclosing_function: &LocalFunction) -> Vec<LocalId> {
        let function_body = self.block(self.entry_block());
        // Inspect the function signature to get a list of locals used for parameter arguments
        let param_locals = self.args.iter().copied();
        // Inspect the function body to get all locals used within the function body
        // (some of which will likely be references to the parameters, so we will need to deduplicate these)
        let body_locals = function_body.get_locals(self);
        // Combine the function parameter locals with the locals used within the function body
        let all_locals = param_locals.chain(body_locals);
        // Deduplicate to remove locals that occur both in parameters and in the function body
        let deduplicated_locals = all_locals.collect::<HashSet<_>>();
        deduplicated_locals.into_iter().collect()
    }
}

impl WasmLocals for ir::Instr {
    fn get_locals(&self, enclosing_function: &LocalFunction) -> Vec<LocalId> {
        match self {
            Self::LocalGet(ir::LocalGet { local }) => vec![*local],
            Self::LocalSet(ir::LocalSet { local }) => vec![*local],
            Self::LocalTee(ir::LocalTee { local }) => vec![*local],
            Self::Block(ir::Block { seq }) => {
                let block = enclosing_function.block(*seq);
                block.get_locals(enclosing_function)
            }
            Self::Loop(ir::Loop { seq }) => {
                let block = enclosing_function.block(*seq);
                block.get_locals(enclosing_function)
            }
            Self::IfElse(ir::IfElse {
                consequent,
                alternative,
            }) => {
                let consequent = enclosing_function.block(*consequent);
                let alternative = enclosing_function.block(*alternative);
                consequent
                    .get_locals(enclosing_function)
                    .into_iter()
                    .chain(alternative.get_locals(enclosing_function))
                    .collect()
            }
            _ => Default::default(),
        }
    }
}

impl WasmLocals for ir::InstrSeq {
    fn get_locals(&self, enclosing_function: &LocalFunction) -> Vec<LocalId> {
        self.instrs
            .iter()
            .flat_map(|(instruction, _)| instruction.get_locals(enclosing_function))
            .collect()
    }
}
