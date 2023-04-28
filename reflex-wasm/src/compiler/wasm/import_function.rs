// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::collections::{HashMap, HashSet};

use walrus::{
    ir, ExportItem, FunctionBuilder, FunctionId, FunctionKind, GlobalId, ImportKind, LocalFunction,
    LocalId, MemoryId, Module, TableId, TypeId,
};

use super::{
    get_locals::WasmLocals,
    transform::{
        collect_instruction_sequence, WasmInstructionTransform, WasmRewriter, WasmTransformContext,
    },
};

pub fn import_function<'a>(
    source_module: &Module,
    function_id: FunctionId,
    target_module: &mut Module,
    substitutions: impl IntoIterator<Item = (&'a str, ExportItem)>,
    entry_points: impl IntoIterator<Item = FunctionId>,
) -> Result<FunctionId, String> {
    let source_function = match &source_module.funcs.get(function_id).kind {
        FunctionKind::Local(function) => Some(function),
        FunctionKind::Import(_) | FunctionKind::Uninitialized(_) => None,
    }
    .ok_or_else(|| format!("Invalid source function"))?;
    let mappings = WasmIdMappings::new(
        source_module,
        source_function,
        target_module,
        substitutions,
        entry_points,
    )?;
    let function_type = source_module.types.get(source_function.ty());
    let mut target_function_builder = FunctionBuilder::new(
        &mut target_module.types,
        function_type.params(),
        function_type.results(),
    );
    {
        let source_function_body = source_function.block(source_function.entry_block());
        let mut context =
            WasmFunctionImportContext::new(source_function, &mut target_function_builder);
        let instructions = source_function_body
            .rewrite(&mappings, &mut context)?
            .unwrap_or_else(|| {
                source_function_body
                    .instrs
                    .iter()
                    .map(|(instruction, _)| instruction.clone())
                    .collect()
            });
        let mut builder = target_function_builder.func_body();
        for instruction in instructions {
            builder.instr(instruction);
        }
    }
    let arg_ids = source_function
        .args
        .iter()
        .filter_map(|arg_id| mappings.local_mappings.get(arg_id))
        .copied()
        .collect::<Vec<_>>();
    Ok(target_function_builder.finish(arg_ids, &mut target_module.funcs))
}

struct WasmFunctionImportContext<'a> {
    source_function: &'a LocalFunction,
    target_function_builder: &'a mut FunctionBuilder,
}

impl<'a> WasmFunctionImportContext<'a> {
    pub fn new(
        function_template: &'a LocalFunction,
        function_builder: &'a mut FunctionBuilder,
    ) -> Self {
        Self {
            source_function: function_template,
            target_function_builder: function_builder,
        }
    }
}

impl<'a> WasmTransformContext for WasmFunctionImportContext<'a> {
    fn enclosing_function(&self) -> &LocalFunction {
        self.source_function
    }
    fn builder(&mut self) -> &mut FunctionBuilder {
        self.target_function_builder
    }
}

pub struct WasmIdMappings {
    function_mappings: HashMap<FunctionId, FunctionId>,
    table_mappings: HashMap<TableId, TableId>,
    memory_mappings: HashMap<MemoryId, MemoryId>,
    global_mappings: HashMap<GlobalId, GlobalId>,
    type_mappings: HashMap<TypeId, TypeId>,
    local_mappings: HashMap<LocalId, LocalId>,
}

impl WasmIdMappings {
    pub fn new<'a>(
        source_module: &Module,
        source_function: &LocalFunction,
        target_module: &mut Module,
        substitutions: impl IntoIterator<Item = (&'a str, ExportItem)>,
        entry_points: impl IntoIterator<Item = FunctionId>,
    ) -> Result<Self, String> {
        let entry_points = entry_points.into_iter().collect::<HashSet<_>>();
        let template_imports = source_module
            .imports
            .iter()
            .filter(|import| import.module.as_str() == "$")
            .collect::<Vec<_>>();
        let substitutions = substitutions.into_iter().collect::<Vec<_>>();
        let function_substitutions = substitutions
            .iter()
            .filter_map(|(export_name, export)| match export {
                ExportItem::Function(function_id) => Some((*export_name, *function_id)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let table_substitutions = substitutions
            .iter()
            .filter_map(|(export_name, export)| match export {
                ExportItem::Table(table_id) => Some((*export_name, *table_id)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let memory_substitutions = substitutions
            .iter()
            .filter_map(|(export_name, export)| match export {
                ExportItem::Memory(memory_id) => Some((*export_name, *memory_id)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let global_substitutions = substitutions
            .iter()
            .filter_map(|(export_name, export)| match export {
                ExportItem::Global(global_id) => Some((*export_name, *global_id)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let template_function_imports = template_imports
            .iter()
            .filter_map(|import| match &import.kind {
                ImportKind::Function(function_id) => Some((*function_id, import.name.as_str())),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let template_table_imports = template_imports
            .iter()
            .filter_map(|import| match &import.kind {
                ImportKind::Table(table_id) => Some((*table_id, import.name.as_str())),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let template_memory_imports = template_imports
            .iter()
            .filter_map(|import| match &import.kind {
                ImportKind::Memory(memory_id) => Some((*memory_id, import.name.as_str())),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let template_global_imports = template_imports
            .iter()
            .filter_map(|import| match &import.kind {
                ImportKind::Global(global_id) => Some((*global_id, import.name.as_str())),
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let function_mappings = source_module
            .funcs
            .iter()
            .filter(|function| !entry_points.contains(&function.id()))
            .map(|function| {
                let template_function_id = function.id();
                // TODO: Allow internally-referenced private functions within templated WASM function
                let import_name = template_function_imports
                    .get(&template_function_id)
                    .copied()
                    .ok_or_else(|| {
                        format!(
                            "Template function not exposed as module import: {template_function_id:?}"
                        )
                    })?;
                let target_function_id =
                    function_substitutions
                        .get(import_name)
                        .copied()
                        .ok_or_else(|| {
                            format!("Template function import not provided: {import_name:?}")
                        })?;
                Ok((template_function_id, target_function_id))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;
        let table_mappings = source_module
            .tables
            .iter()
            .map(|table| {
                let template_table_id = table.id();
                let import_name = template_table_imports
                    .get(&template_table_id)
                    .copied()
                    .ok_or_else(|| {
                        format!(
                            "Template table not exposed as module export: {template_table_id:?}"
                        )
                    })?;
                let target_table_id =
                    table_substitutions
                        .get(import_name)
                        .copied()
                        .ok_or_else(|| {
                            format!("Template table import not provided: {import_name:?}")
                        })?;
                Ok((template_table_id, target_table_id))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;
        let memory_mappings = source_module
            .memories
            .iter()
            .map(|memory| {
                let template_memory_id = memory.id();
                let import_name = template_memory_imports
                    .get(&template_memory_id)
                    .copied()
                    .ok_or_else(|| {
                        format!(
                            "Template memory not exposed as module export: {template_memory_id:?}"
                        )
                    })?;
                let target_memory_id =
                    memory_substitutions
                        .get(import_name)
                        .copied()
                        .ok_or_else(|| {
                            format!("Template memory import not provided: {import_name:?}")
                        })?;
                Ok((template_memory_id, target_memory_id))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;
        let global_mappings = source_module
            .globals
            .iter()
            .map(|global| {
                let template_global_id = global.id();
                let import_name = template_global_imports
                    .get(&template_global_id)
                    .copied()
                    .ok_or_else(|| {
                        format!(
                            "Template global not exposed as module export: {template_global_id:?}"
                        )
                    })?;
                let target_global_id =
                    global_substitutions
                        .get(import_name)
                        .copied()
                        .ok_or_else(|| {
                            format!("Template global import not provided: {import_name:?}")
                        })?;
                Ok((template_global_id, target_global_id))
            })
            .collect::<Result<HashMap<_, _>, String>>()?;
        let type_mappings = source_module
            .types
            .iter()
            .map(|ty| (ty.id(), target_module.types.add(ty.params(), ty.results())))
            .collect::<HashMap<TypeId, TypeId>>();
        let local_mappings = source_function
            .get_locals(source_function)
            .into_iter()
            .map(|template_local| {
                let value_type = source_module.locals.get(template_local).ty();
                (template_local, target_module.locals.add(value_type))
            })
            .collect::<HashMap<_, _>>();
        Ok(Self {
            function_mappings,
            table_mappings,
            memory_mappings,
            global_mappings,
            type_mappings,
            local_mappings,
        })
    }
    fn translate_function_id(&self, id: &FunctionId) -> Result<FunctionId, String> {
        self.function_mappings
            .get(id)
            .copied()
            .ok_or_else(|| format!("Invalid function ID: {id:?}"))
    }
    fn translate_table_id(&self, id: &TableId) -> Result<TableId, String> {
        self.table_mappings
            .get(id)
            .copied()
            .ok_or_else(|| format!("Invalid table ID: {id:?}"))
    }
    fn translate_memory_id(&self, id: &MemoryId) -> Result<MemoryId, String> {
        self.memory_mappings
            .get(id)
            .copied()
            .ok_or_else(|| format!("Invalid memory ID: {id:?}"))
    }
    fn translate_global_id(&self, id: &GlobalId) -> Result<GlobalId, String> {
        self.global_mappings
            .get(id)
            .copied()
            .ok_or_else(|| format!("Invalid global ID: {id:?}"))
    }
    fn translate_type_id(&self, id: &TypeId) -> Result<TypeId, String> {
        self.type_mappings
            .get(id)
            .copied()
            .ok_or_else(|| format!("Invalid type ID: {id:?}"))
    }
    fn translate_local_id(&self, id: &LocalId) -> Result<LocalId, String> {
        self.local_mappings
            .get(id)
            .copied()
            .ok_or_else(|| format!("Invalid local ID: {id:?}"))
    }
    fn import_type(&self, ty: &ir::InstrSeqType) -> Result<ir::InstrSeqType, String> {
        match ty {
            ir::InstrSeqType::Simple(ty) => Ok(ir::InstrSeqType::Simple(*ty)),
            ir::InstrSeqType::MultiValue(ty) => {
                self.translate_type_id(ty).map(ir::InstrSeqType::MultiValue)
            }
        }
    }
    fn import_block(
        &self,
        block_id: ir::InstrSeqId,
        context: &mut impl WasmTransformContext,
    ) -> Result<ir::InstrSeqId, String> {
        let block = context.enclosing_function().block(block_id);
        let block_type = self.import_type(&block.ty)?;
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

impl<C: WasmTransformContext> WasmInstructionTransform<C> for WasmIdMappings {
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
            ir::Instr::Call(ir::Call { func }) => Ok(Some(ir::Instr::Call(ir::Call {
                func: self.translate_function_id(func)?,
            }))),
            ir::Instr::CallIndirect(ir::CallIndirect { ty, table }) => {
                Ok(Some(ir::Instr::CallIndirect(ir::CallIndirect {
                    ty: self.translate_type_id(ty)?,
                    table: self.translate_table_id(table)?,
                })))
            }
            ir::Instr::LocalGet(ir::LocalGet { local }) => {
                Ok(Some(ir::Instr::LocalGet(ir::LocalGet {
                    local: self.translate_local_id(local)?,
                })))
            }
            ir::Instr::LocalSet(ir::LocalSet { local }) => {
                Ok(Some(ir::Instr::LocalSet(ir::LocalSet {
                    local: self.translate_local_id(local)?,
                })))
            }
            ir::Instr::LocalTee(ir::LocalTee { local }) => {
                Ok(Some(ir::Instr::LocalTee(ir::LocalTee {
                    local: self.translate_local_id(local)?,
                })))
            }
            ir::Instr::GlobalGet(ir::GlobalGet { global }) => {
                Ok(Some(ir::Instr::GlobalGet(ir::GlobalGet {
                    global: self.translate_global_id(global)?,
                })))
            }
            ir::Instr::GlobalSet(ir::GlobalSet { global }) => {
                Ok(Some(ir::Instr::GlobalSet(ir::GlobalSet {
                    global: self.translate_global_id(global)?,
                })))
            }
            ir::Instr::Const(_) => Ok(None),
            ir::Instr::Binop(_) => Ok(None),
            ir::Instr::Unop(_) => Ok(None),
            ir::Instr::Select(_) => Ok(None),
            ir::Instr::Unreachable(_) => Ok(None),
            ir::Instr::Br(_) => Err(format!("Branch instructions not currently supported")),
            ir::Instr::BrIf(_) => Err(format!("Branch instructions not currently supported")),
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
            ir::Instr::BrTable(_) => Err(format!("Branch instructions not currently supported")),
            ir::Instr::Drop(ir::Drop {}) => Ok(None),
            ir::Instr::Return(ir::Return {}) => Ok(None),
            ir::Instr::MemorySize(ir::MemorySize { memory }) => {
                Ok(Some(ir::Instr::MemorySize(ir::MemorySize {
                    memory: self.translate_memory_id(memory)?,
                })))
            }
            ir::Instr::MemoryGrow(ir::MemoryGrow { memory }) => {
                Ok(Some(ir::Instr::MemoryGrow(ir::MemoryGrow {
                    memory: self.translate_memory_id(memory)?,
                })))
            }
            ir::Instr::MemoryInit(_) => Err(format!("Data instructions not currently supported")),
            ir::Instr::DataDrop(_) => Err(format!("Data instructions not currently supported")),
            ir::Instr::MemoryCopy(ir::MemoryCopy { src, dst }) => {
                Ok(Some(ir::Instr::MemoryCopy(ir::MemoryCopy {
                    src: self.translate_memory_id(src)?,
                    dst: self.translate_memory_id(dst)?,
                })))
            }
            ir::Instr::MemoryFill(ir::MemoryFill { memory }) => {
                Ok(Some(ir::Instr::MemoryFill(ir::MemoryFill {
                    memory: self.translate_memory_id(memory)?,
                })))
            }
            ir::Instr::Load(ir::Load { memory, kind, arg }) => {
                Ok(Some(ir::Instr::Load(ir::Load {
                    memory: self.translate_memory_id(memory)?,
                    kind: *kind,
                    arg: *arg,
                })))
            }
            ir::Instr::Store(ir::Store { memory, kind, arg }) => {
                Ok(Some(ir::Instr::Store(ir::Store {
                    memory: self.translate_memory_id(memory)?,
                    kind: *kind,
                    arg: *arg,
                })))
            }
            _ => Err(format!("Unsupported instruction: {instruction:?}")),
        }?;
        Ok(translated_instruction.map(|instruction| [instruction]))
    }
}
