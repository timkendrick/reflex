// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use walrus::{ExportItem, FunctionBuilder, FunctionId, ValType};

use crate::ArenaPointer;

#[derive(Debug)]
pub enum WasmEntryPointError {
    ModuleLoadError(anyhow::Error),
    FunctionNotFound(String),
    InvalidAstTransformation,
}

impl std::error::Error for WasmEntryPointError {}

impl std::fmt::Display for WasmEntryPointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModuleLoadError(err) => write!(f, "Failed to load WASM module: {err}"),
            Self::FunctionNotFound(name) => write!(f, "Function definition not found: {name}"),
            Self::InvalidAstTransformation => write!(f, "Invalid AST transformation"),
        }
    }
}

pub fn add_module_entry_point(
    wasm_bytes: &[u8],
    export_name: &str,
    expression_pointer: u32,
    state_pointer: Option<u32>,
) -> Result<Vec<u8>, WasmEntryPointError> {
    // Create a new WASM module based on the input bytes
    let mut ast = parse_wasm_ast(wasm_bytes)?;
    // Locate the exported evaluate function
    let evaluate_function_id = get_named_function_id(&ast, "evaluate")
        .ok_or_else(|| WasmEntryPointError::FunctionNotFound(String::from("evaluate")))?;
    // Generate an entry point function that evaluates the given expression with the given state object
    let entry_point_function_id = create_entry_point_function(
        &mut ast,
        expression_pointer,
        state_pointer,
        evaluate_function_id,
    );
    // Export the entry point function with the desired export name
    ast.exports
        .add(export_name, ExportItem::Function(entry_point_function_id));
    // Emit the resulting WASM as bytes
    Ok(ast.emit_wasm())
}

fn create_entry_point_function(
    module: &mut walrus::Module,
    expression_pointer: u32,
    state_pointer: Option<u32>,
    evaluate_function_id: FunctionId,
) -> FunctionId {
    let params = [];
    let results = [ValType::I32, ValType::I32];
    let function_id = {
        // Create the function signature
        let mut builder = FunctionBuilder::new(&mut module.types, &params, &results);
        // Create the function body
        let mut function_body = builder.func_body();
        // Embed the function instructions
        function_body
            // Push the evaluation expression pointer onto the operand stack
            .i32_const(expression_pointer as i32)
            // Push the state pointer onto the operand stack,
            // defaulting to the null pointer if no state object was provided
            .i32_const(state_pointer.unwrap_or(u32::from(ArenaPointer::null())) as i32)
            // Invoke the evaluate function, returning its result
            .call(evaluate_function_id);
        // Add the function to the WASM module
        builder.finish(Vec::new(), &mut module.funcs)
    };
    function_id
}

fn parse_wasm_ast(runtime_wasm: &[u8]) -> Result<walrus::Module, WasmEntryPointError> {
    walrus::Module::from_buffer(runtime_wasm).map_err(WasmEntryPointError::ModuleLoadError)
}

fn get_named_function_id(module: &walrus::Module, export_name: &str) -> Option<FunctionId> {
    parse_exported_functions(module).find_map(|(name, function_id)| {
        if name == export_name {
            Some(function_id)
        } else {
            None
        }
    })
}

fn parse_exported_functions(
    module: &walrus::Module,
) -> impl Iterator<Item = (&str, FunctionId)> + '_ {
    module
        .exports
        .iter()
        .filter_map(|export| match export.item {
            ExportItem::Function(id) => Some((export.name.as_str(), id)),
            _ => None,
        })
}
