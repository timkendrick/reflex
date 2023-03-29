// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use derivative::Derivative;
use reflex::core::{Arity, StackOffset};

use crate::{
    allocator::Arena,
    compiler::{ParamsSignature, ValueType},
    ArenaRef, FunctionIndex, Term,
};

#[derive(Derivative)]
#[derivative(Clone(bound = "A: Clone"), Debug(bound = "A: Clone"))]
pub enum CompilerError<A: Arena> {
    InvalidFunctionTarget(FunctionIndex),
    InvalidFunctionArgs {
        target: ArenaRef<Term, A>,
        arity: Arity,
        args: Vec<ArenaRef<Term, A>>,
    },
    UnboundVariable(StackOffset),
    StackError(TypedStackError),
}

impl<A: Arena + Clone> std::fmt::Display for CompilerError<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFunctionTarget(target) => {
                write!(f, "Invalid function index: {target}",)
            }
            Self::InvalidFunctionArgs {
                target,
                arity,
                args,
            } => write!(
                f,
                "Invalid function invocation for {target}: expected {} arguments, received ({})",
                arity.required().len(),
                args.iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::UnboundVariable(scope_offset) => {
                write!(f, "Unbound variable scope offset {scope_offset}")
            }
            Self::StackError(err) => write!(f, "Stack error: {}", err),
        }
    }
}

impl<A: Arena> From<TypedStackError> for CompilerError<A> {
    fn from(value: TypedStackError) -> Self {
        Self::StackError(value)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TypedStackError {
    InvalidOperandStackValueTypes(InvalidOperandStackValueTypesError),
    InvalidLexicalScopeValueType(InvalidLexicalScopeValueTypeError),
    InvalidBlockResultType(InvalidBlockResultTypeError),
}

impl std::fmt::Display for TypedStackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidOperandStackValueTypes(inner) => std::fmt::Display::fmt(inner, f),
            Self::InvalidLexicalScopeValueType(inner) => std::fmt::Display::fmt(inner, f),
            Self::InvalidBlockResultType(inner) => std::fmt::Display::fmt(inner, f),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InvalidOperandStackValueTypesError {
    pub expected: ParamsSignature,
    pub received: ParamsSignature,
}

impl std::fmt::Display for InvalidOperandStackValueTypesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid values on operand stack: Expected {}, received {}",
            self.expected, self.received
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InvalidLexicalScopeValueTypeError {
    pub scope_offset: usize,
    pub scope_offset_types: Vec<ValueType>,
    pub expected: ValueType,
    pub received: Option<ValueType>,
}

impl std::fmt::Display for InvalidLexicalScopeValueTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = write!(
            f,
            "Invalid lexical scope value type at scope offset {}: Expected {}, received ",
            self.scope_offset, self.expected,
        )?;
        let _ = if let Some(received) = self.received.as_ref() {
            write!(f, "{received}")
        } else {
            write!(f, "<none>")
        }?;
        self.scope_offset_types.iter().enumerate().fold(
            Ok(()),
            |result, (scope_offset, value_type)| {
                let _ = result?;
                write!(f, "\n {scope_offset}: {value_type}")
            },
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InvalidBlockResultTypeError {
    pub block_types: Vec<ParamsSignature>,
    pub expected: ParamsSignature,
    pub received: Option<ParamsSignature>,
}

impl std::fmt::Display for InvalidBlockResultTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = write!(
            f,
            "Invalid block result type: Expected {}, received ",
            self.expected,
        );
        let _ = if let Some(received) = self.received.as_ref() {
            write!(f, "{received}")
        } else {
            write!(f, "<none>")
        }?;
        self.block_types
            .iter()
            .enumerate()
            .fold(Ok(()), |result, (block_offset, result_type)| {
                let _ = result?;
                write!(f, "\n {block_offset}: {result_type}")
            })
    }
}
