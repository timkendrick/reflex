// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use walrus::{ir, FunctionBuilder, FunctionId, FunctionKind, LocalId, Module, ValType};

use super::transform::{
    collect_instruction_sequence, import_instruction_sequence, WasmInstructionTransform,
    WasmRewriter, WasmTransformContext,
};

pub fn expand_function_args(
    module: &mut Module,
    function_id: FunctionId,
    args_placeholder_id: FunctionId,
    arg_types: impl IntoIterator<Item = ValType>,
) -> Result<FunctionId, String> {
    let source_function = match &mut module.funcs.get_mut(function_id).kind {
        FunctionKind::Local(function) => Some(function),
        _ => None,
    }
    .ok_or_else(|| format!("Invalid function type"))?;
    let (expanded_arg_types, expanded_arg_ids): (Vec<_>, Vec<_>) = arg_types
        .into_iter()
        .map(|arg_type| (arg_type, module.locals.add(arg_type)))
        .unzip();
    // Rewrite the function body, transforming any accesses of of the arguments placeholder with a sequence of accesses of all expanded arguments
    let transform = ExpandInstructionArgsTransform {
        args_placeholder_id,
        expanded_arg_ids: &expanded_arg_ids,
    };
    let function_body = source_function.block(source_function.entry_block());
    let function_body_instructions = collect_instruction_sequence(function_body);
    let updated_instructions = function_body_instructions.rewrite(&transform, source_function)?;
    if updated_instructions.is_none() && expanded_arg_ids.is_empty() {
        let unmodified_function_id = function_id;
        Ok(unmodified_function_id)
    } else {
        // Create a new function wrapper, prepending the expanded arguments before any existing arguments
        let source_function_type = module.types.get(source_function.ty());
        let params = expanded_arg_types
            .into_iter()
            .chain(source_function_type.params().iter().copied())
            .collect::<Vec<_>>();
        let args = expanded_arg_ids
            .iter()
            .copied()
            .chain(source_function.args.iter().copied())
            .collect::<Vec<_>>();
        let results = Vec::from(source_function_type.results());
        let mut builder = FunctionBuilder::new(&mut module.types, &params, &results);
        let instructions = updated_instructions.unwrap_or_else(|| function_body_instructions);
        let _ = import_instruction_sequence(instructions, source_function, &mut builder)?;
        let expanded_function_id = builder.finish(args, &mut module.funcs);
        Ok(expanded_function_id)
    }
}

struct ExpandInstructionArgsTransform<'a> {
    args_placeholder_id: FunctionId,
    expanded_arg_ids: &'a [LocalId],
}

impl<'a, C: WasmTransformContext> WasmInstructionTransform<C>
    for ExpandInstructionArgsTransform<'a>
{
    type Output = Vec<ir::Instr>;
    fn transform(
        &self,
        instruction: &ir::Instr,
        _context: &mut C,
    ) -> Result<Option<Self::Output>, String> {
        match instruction {
            ir::Instr::Call(ir::Call { func }) if *func == self.args_placeholder_id => Ok(Some(
                self.expanded_arg_ids
                    .iter()
                    .copied()
                    .map(|arg_id| ir::Instr::LocalGet(ir::LocalGet { local: arg_id }))
                    .collect::<Vec<_>>(),
            )),
            _ => Ok(None),
        }
    }
}
