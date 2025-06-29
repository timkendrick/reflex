// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use chrono::{DateTime, NaiveDateTime};
use wasi_common::WasiCtx;
use wasmtime::{AsContext, AsContextMut, Caller, Extern, Memory, StoreContext};

use crate::{
    interpreter::{InterpreterError, WasmContextBuilder},
    term_type::timestamp::UtcTimestamp,
    ArenaPointer, WASM_PAGE_SIZE,
};

pub fn add_wasm_runtime_imports(
    builder: WasmContextBuilder,
    memory_name: &'static str,
) -> Result<WasmContextBuilder, InterpreterError> {
    builder
        .add_import("Debugger", "debug", |value_pointer: u32| {
            let value = ArenaPointer::from(value_pointer).as_non_null();
            if let Some(value) = value {
                eprintln!("[DEBUG] {value:?}")
            } else {
                eprintln!("[DEBUG] NULL")
            }
        })?
        .add_import(
            "Date",
            "parse",
            |mut caller: Caller<'_, WasiCtx>, offset: u32, length: u32| -> i64 {
                let timestamp = caller
                    .get_export(memory_name)
                    .and_then(|export| match export {
                        Extern::Memory(memory) => Some(memory),
                        _ => None,
                    })
                    .and_then(|memory| {
                        let slice =
                            read_linear_memory_slice(&memory, caller.as_context(), offset, length);
                        std::str::from_utf8(slice)
                            .ok()
                            .and_then(|timestamp| parse_string_timestamp(timestamp))
                    });
                timestamp.unwrap_or(-1)
            },
        )?
        .add_import(
            "Date",
            "toISOString",
            |mut caller: Caller<'_, WasiCtx>, timestamp: i64, dest_pointer: u32| -> u32 {
                let formatted = format!("{}", UtcTimestamp(timestamp));
                let formatted_bytes = formatted.as_bytes();
                match get_linear_memory(&mut caller, memory_name).and_then(|mut memory| {
                    write_linear_memory_bytes(
                        &mut memory,
                        &mut caller,
                        dest_pointer,
                        formatted_bytes,
                    )
                    .and_then(|_| {
                        update_linear_memory_allocator_offset(
                            &mut memory,
                            &mut caller,
                            dest_pointer + formatted_bytes.len() as u32,
                        )
                    })
                    .ok()
                }) {
                    Some(_) => formatted_bytes.len() as u32,
                    None => u32::from(ArenaPointer::null()),
                }
            },
        )?
        .add_import(
            "Number",
            "toString",
            |mut caller: Caller<'_, WasiCtx>, value: f64, dest_pointer: u32| -> u32 {
                let formatted = format!("{}", value);
                let formatted_bytes = formatted.as_bytes();
                match get_linear_memory(&mut caller, memory_name).and_then(|mut memory| {
                    write_linear_memory_bytes(
                        &mut memory,
                        &mut caller,
                        dest_pointer,
                        formatted_bytes,
                    )
                    .and_then(|_| {
                        update_linear_memory_allocator_offset(
                            &mut memory,
                            &mut caller,
                            dest_pointer + formatted_bytes.len() as u32,
                        )
                    })
                    .ok()
                }) {
                    Some(_) => formatted_bytes.len() as u32,
                    None => u32::from(ArenaPointer::null()),
                }
            },
        )?
        .add_import("Math", "remainder", |left: f64, right: f64| -> f64 {
            left % right
        })?
        .add_import("Math", "acos", |value: f64| -> f64 { value.acos() })?
        .add_import("Math", "acosh", |value: f64| -> f64 { value.acosh() })?
        .add_import("Math", "asin", |value: f64| -> f64 { value.asin() })?
        .add_import("Math", "asinh", |value: f64| -> f64 { value.asinh() })?
        .add_import("Math", "atan", |value: f64| -> f64 { value.atan() })?
        .add_import("Math", "atan2", |left: f64, right: f64| -> f64 {
            left.atan2(right)
        })?
        .add_import("Math", "atanh", |value: f64| -> f64 { value.atanh() })?
        .add_import("Math", "cbrt", |value: f64| -> f64 { value.cbrt() })?
        .add_import("Math", "cos", |value: f64| -> f64 { value.cos() })?
        .add_import("Math", "cosh", |value: f64| -> f64 { value.cosh() })?
        .add_import("Math", "exp", |value: f64| -> f64 { value.exp() })?
        .add_import("Math", "expm1", |value: f64| -> f64 { value.exp() - 1.0 })?
        .add_import("Math", "hypot", |left: f64, right: f64| -> f64 {
            left.hypot(right)
        })?
        .add_import("Math", "log", |value: f64| -> f64 { value.ln() })?
        .add_import("Math", "log2", |value: f64| -> f64 { value.log2() })?
        .add_import("Math", "log10", |value: f64| -> f64 { value.log10() })?
        .add_import("Math", "log1p", |value: f64| -> f64 { (value + 1.0).ln() })?
        .add_import("Math", "pow", |left: f64, right: f64| -> f64 {
            left.powf(right)
        })?
        .add_import("Math", "sin", |value: f64| -> f64 { value.sin() })?
        .add_import("Math", "sinh", |value: f64| -> f64 { value.sinh() })?
        .add_import("Math", "sqrt", |value: f64| -> f64 { value.sqrt() })?
        .add_import("Math", "tan", |value: f64| -> f64 { value.tan() })?
        .add_import("Math", "tanh", |value: f64| -> f64 { value.tanh() })
}

fn get_linear_memory(caller: &mut Caller<WasiCtx>, memory_name: &str) -> Option<Memory> {
    caller
        .get_export(memory_name)
        .and_then(|export| match export {
            Extern::Memory(memory) => Some(memory),
            _ => None,
        })
}

fn write_linear_memory_bytes(
    memory: &mut Memory,
    caller: &mut Caller<WasiCtx>,
    dest_pointer: u32,
    bytes: &[u8],
) -> Result<(), ()> {
    if bytes.len() > (u32::MAX as usize) {
        return Err(());
    }
    // Ensure enough pages of linear memory have been allocated to contain the output bytes
    let end_offset = dest_pointer + (bytes.len() as u32);
    let _ = ensure_linear_memory_size(memory, caller, end_offset).map_err(|_| ())?;
    // Write the bytes into linear memory
    let _ = memory
        .write(caller.as_context_mut(), dest_pointer as usize, bytes)
        .map_err(|_| ())?;
    Ok(())
}

fn update_linear_memory_allocator_offset(
    memory: &mut Memory,
    caller: &mut Caller<WasiCtx>,
    offset: u32,
) -> Result<(), ()> {
    // Update the bump allocator offset at heap pointer address 0 to reflect the updated heap size
    let _ = memory
        .write(caller.as_context_mut(), 0, &offset.to_le_bytes())
        .map_err(|_| ())?;
    Ok(())
}

#[must_use]
fn ensure_linear_memory_size<'a, T>(
    memory: &mut Memory,
    caller: &mut Caller<'a, T>,
    len: u32,
) -> Result<(), anyhow::Error> {
    let required_size = len as usize;
    let existing_size = memory.data_size(caller.as_context());
    if existing_size >= required_size {
        return Ok(());
    }
    let num_existing_pages = existing_size / WASM_PAGE_SIZE;
    let num_required_pages = pad_to_next(required_size, WASM_PAGE_SIZE);
    let _ = memory.grow(
        caller.as_context_mut(),
        (num_required_pages - num_existing_pages) as u64,
    )?;
    Ok(())
}

fn read_linear_memory_slice<'a, T: 'a>(
    memory: &'a Memory,
    store: impl Into<StoreContext<'a, T>>,
    offset: u32,
    length: u32,
) -> &'a [u8] {
    let offset = offset as usize;
    let length = length as usize;
    let heap = memory.data(store);
    &heap[offset..offset + length]
}

fn pad_to_next(value: usize, interval: usize) -> usize {
    1 + (value.saturating_sub(1) / interval)
}

fn parse_string_timestamp(timestamp: &str) -> Option<i64> {
    None.or_else(|| {
        DateTime::parse_from_rfc3339(timestamp)
            .ok()
            .map(|date| date.timestamp_millis())
    })
    .or_else(|| {
        NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%.f")
            .ok()
            .map(|date| date.timestamp_millis())
    })
}
