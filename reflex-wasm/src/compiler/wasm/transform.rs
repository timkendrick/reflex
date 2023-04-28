// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use walrus::{
    ir::{self, InstrSeq},
    FunctionBuilder, LocalFunction,
};

pub trait WasmInstructionTransform<C: WasmTransformContext> {
    type Output: IntoIterator<Item = ir::Instr>;
    fn transform(
        &self,
        instruction: &ir::Instr,
        context: &mut C,
    ) -> Result<Option<Self::Output>, String>;
}

pub trait WasmTransformContext {
    fn enclosing_function(&self) -> &LocalFunction;
    fn builder(&mut self) -> &mut FunctionBuilder;
}

impl WasmTransformContext for LocalFunction {
    fn enclosing_function(&self) -> &LocalFunction {
        self
    }
    fn builder(&mut self) -> &mut FunctionBuilder {
        self.builder_mut()
    }
}

impl<'a> WasmTransformContext for (&'a LocalFunction, &'a mut FunctionBuilder) {
    fn enclosing_function(&self) -> &LocalFunction {
        self.0
    }
    fn builder(&mut self) -> &mut FunctionBuilder {
        self.1
    }
}

pub trait WasmRewriter<F: WasmInstructionTransform<C>, C: WasmTransformContext> {
    type Output: IntoIterator<Item = ir::Instr>;
    fn rewrite(&self, transform: &F, context: &mut C) -> Result<Option<Self::Output>, String>;
}

impl<F: WasmInstructionTransform<C>, C: WasmTransformContext> WasmRewriter<F, C> for ir::InstrSeq {
    type Output = Vec<ir::Instr>;
    fn rewrite(&self, transform: &F, context: &mut C) -> Result<Option<Self::Output>, String> {
        self.instrs
            .iter()
            .map(|(instruction, _)| instruction.rewrite(transform, context))
            .enumerate()
            .fold(Ok(None), |result, (index, rewritten_instruction)| {
                let existing_results = result?;
                let instructions = rewritten_instruction?;
                Ok(match (existing_results, instructions) {
                    (None, None) => None,
                    (None, Some(instructions)) => {
                        let preceding_instructions = self
                            .instrs
                            .iter()
                            .map(|(instruction, _)| instruction)
                            .take(index)
                            .cloned();
                        let combined_instructions = preceding_instructions
                            .chain(instructions)
                            .collect::<Vec<_>>();
                        Some(combined_instructions)
                    }
                    (Some(mut combined_instructions), instructions) => {
                        if let Some(instructions) = instructions {
                            combined_instructions.extend(instructions);
                        } else {
                            let (instruction, _) = &self.instrs[index];
                            combined_instructions.push(instruction.clone());
                        }
                        Some(combined_instructions)
                    }
                })
            })
    }
}

impl<F: WasmInstructionTransform<C>, C: WasmTransformContext> WasmRewriter<F, C>
    for Vec<ir::Instr>
{
    type Output = Vec<ir::Instr>;
    fn rewrite(&self, transform: &F, context: &mut C) -> Result<Option<Self::Output>, String> {
        self.iter()
            .map(|instruction| instruction.rewrite(transform, context))
            .enumerate()
            .fold(Ok(None), |result, (index, rewritten_instruction)| {
                let existing_results = result?;
                let instructions = rewritten_instruction?;
                Ok(match (existing_results, instructions) {
                    (None, None) => None,
                    (None, Some(instructions)) => {
                        let preceding_instructions = (&self[0..index]).iter().cloned();
                        let combined_instructions = preceding_instructions
                            .chain(instructions)
                            .collect::<Vec<_>>();
                        Some(combined_instructions)
                    }
                    (Some(mut combined_instructions), instructions) => {
                        if let Some(instructions) = instructions {
                            combined_instructions.extend(instructions);
                        } else {
                            let instruction = &self[index];
                            combined_instructions.push(instruction.clone());
                        }
                        Some(combined_instructions)
                    }
                })
            })
    }
}

impl<F: WasmInstructionTransform<C>, C: WasmTransformContext> WasmRewriter<F, C> for ir::Instr {
    type Output = std::iter::Chain<
        std::iter::Flatten<std::option::IntoIter<F::Output>>,
        std::option::IntoIter<ir::Instr>,
    >;
    fn rewrite(&self, transform: &F, context: &mut C) -> Result<Option<Self::Output>, String> {
        let transformed = transform.transform(self, context)?;
        if let Some(instructions) = transformed {
            let transformed_instructions = Some(instructions);
            let rewritten_instruction = None;
            let output = transformed_instructions
                .into_iter()
                .flatten()
                .chain(rewritten_instruction);
            Ok(Some(output))
        } else {
            let rewritten_instruction = match self {
                Self::Block(ir::Block { seq }) => rewrite_block(*seq, transform, context)?
                    .map(|block_id| ir::Instr::Block(ir::Block { seq: block_id })),
                Self::Loop(ir::Loop { seq }) => rewrite_block(*seq, transform, context)?
                    .map(|block_id| ir::Instr::Loop(ir::Loop { seq: block_id })),
                Self::IfElse(ir::IfElse {
                    consequent,
                    alternative,
                }) => {
                    match (
                        rewrite_block(*consequent, transform, context)?,
                        rewrite_block(*alternative, transform, context)?,
                    ) {
                        (None, None) => None,
                        (consequent_block_id, alternative_block_id) => {
                            let consequent = consequent_block_id.unwrap_or(*consequent);
                            let alternative = alternative_block_id.unwrap_or(*alternative);
                            Some(ir::Instr::IfElse(ir::IfElse {
                                consequent,
                                alternative,
                            }))
                        }
                    }
                }
                _ => None,
            };
            match rewritten_instruction {
                None => Ok(None),
                Some(instruction) => {
                    let transformed_instructions = None;
                    let rewritten_instruction = Some(instruction);
                    let output = transformed_instructions
                        .into_iter()
                        .flatten()
                        .chain(rewritten_instruction);
                    Ok(Some(output))
                }
            }
        }
    }
}

fn rewrite_block<C: WasmTransformContext>(
    block_id: ir::InstrSeqId,
    transform: &impl WasmInstructionTransform<C>,
    context: &mut C,
) -> Result<Option<ir::InstrSeqId>, String> {
    let block = context.enclosing_function().block(block_id);
    let block_type = block.ty.clone();
    let block_instructions = collect_instruction_sequence(block);
    let instructions = block_instructions.rewrite(transform, context)?;
    match instructions {
        None => Ok(None),
        Some(instructions) => {
            let mut builder = context.builder().dangling_instr_seq(block_type);
            let target_block_id = instructions
                .into_iter()
                .fold(&mut builder, |builder, instruction| {
                    builder.instr(instruction)
                })
                .id();
            Ok(Some(target_block_id))
        }
    }
}

#[must_use]
pub(crate) fn import_instruction_sequence(
    instructions: impl IntoIterator<Item = ir::Instr>,
    source_function: &LocalFunction,
    target_function: &mut FunctionBuilder,
) -> Result<(), String> {
    let instructions = instructions.into_iter().collect::<Vec<_>>();
    let mut context = (source_function, target_function);
    let translated_instructions = instructions
        .rewrite(&ImportInnerBlocks, &mut context)?
        .unwrap_or(instructions);
    let (_source_function, target_function) = context;
    let mut function_body = target_function.func_body();
    translated_instructions
        .into_iter()
        .fold(&mut function_body, |builder, instruction| {
            builder.instr(instruction)
        });
    Ok(())
}

struct ImportInnerBlocks;

impl ImportInnerBlocks {
    fn import_block(
        &self,
        block_id: ir::InstrSeqId,
        context: &mut impl WasmTransformContext,
    ) -> Result<ir::InstrSeqId, String> {
        let block = context.enclosing_function().block(block_id);
        let block_type = block.ty.clone();
        let block_instructions = collect_instruction_sequence(block);
        let instructions = block_instructions
            .rewrite(self, context)?
            .unwrap_or(block_instructions);
        let mut builder = context.builder().dangling_instr_seq(block_type);
        let target_block_id = instructions
            .into_iter()
            .fold(&mut builder, |builder, instruction| {
                builder.instr(instruction)
            })
            .id();
        Ok(target_block_id)
    }
}

impl<C: WasmTransformContext> WasmInstructionTransform<C> for ImportInnerBlocks {
    type Output = [ir::Instr; 1];
    fn transform(
        &self,
        instruction: &ir::Instr,
        context: &mut C,
    ) -> Result<Option<Self::Output>, String> {
        let translated_instruction = match instruction {
            ir::Instr::Block(ir::Block { seq }) => {
                let block_id = self.import_block(*seq, context)?;
                Ok(Some(ir::Instr::Block(ir::Block { seq: block_id })))
            }
            ir::Instr::Loop(ir::Loop { seq }) => {
                let block_id = self.import_block(*seq, context)?;
                Ok(Some(ir::Instr::Loop(ir::Loop { seq: block_id })))
            }
            ir::Instr::IfElse(ir::IfElse {
                consequent,
                alternative,
            }) => {
                let consequent_block_id = self.import_block(*consequent, context)?;
                let alternative_block_id = self.import_block(*alternative, context)?;
                Ok(Some(ir::Instr::IfElse(ir::IfElse {
                    consequent: consequent_block_id,
                    alternative: alternative_block_id,
                })))
            }
            ir::Instr::Br(_) | ir::Instr::BrIf(_) | ir::Instr::BrTable(_) => {
                Err(format!("Branch instructions not currently supported"))
            }
            _ => Ok(None),
        }?;
        Ok(translated_instruction.map(|instruction| [instruction]))
    }
}

pub(crate) fn collect_instruction_sequence(block: &InstrSeq) -> Vec<ir::Instr> {
    block
        .instrs
        .iter()
        .map(|(instruction, _)| instruction.clone())
        .collect::<Vec<_>>()
}
