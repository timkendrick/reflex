// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{collections::HashMap, iter::once, ops::Deref, path::Path};

use reflex::core::{
    as_integer, create_record, Builtin, Expression, ExpressionFactory, FloatTermType,
    HeapAllocator, IntTermType, RefType, StringTermType, StringValue,
};
use reflex_macros::blanket_trait;
use swc_common::{source_map::Pos, sync::Lrc, FileName, SourceMap, Span, Spanned};
use swc_ecma_ast::{
    ArrayLit, ArrowExpr, BinExpr, BinaryOp, BindingIdent, BlockStmt, BlockStmtOrExpr, Bool,
    CallExpr, Callee, CondExpr, Decl, EsVersion, Expr, ExprStmt, Ident, ImportDecl,
    ImportSpecifier, Lit, MemberExpr, MemberProp, Module, ModuleDecl, ModuleExportName, ModuleItem,
    NewExpr, Null, Number, ObjectLit, ObjectPatProp, Pat, Prop, PropName, PropOrSpread, Stmt, Str,
    TaggedTpl, Tpl, TplElement, UnaryExpr, UnaryOp, VarDeclKind, VarDeclarator,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

use crate::stdlib::{
    Accessor, Add, And, Apply, Chain, CollectList, CollectString, Construct, Divide, Eq,
    FormatErrorMessage, Gt, Gte, Has, If, IfError, Lt, Lte, Merge, Multiply, Not, Or, Pow, Push,
    PushFront, Remainder, ResolveDeep, Subtract, Throw, ToString,
};

#[derive(Clone)]
pub struct Env<T: Expression> {
    globals: HashMap<&'static str, T>,
}
impl<T: Expression> Env<T> {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
    }
    pub fn with_globals(mut self, values: impl IntoIterator<Item = (&'static str, T)>) -> Self {
        self.globals.extend(values);
        self
    }
    pub fn with_global(mut self, key: &'static str, value: T) -> Self {
        self.globals.insert(key, value);
        self
    }
    pub fn global(&self, name: &str) -> Option<T> {
        self.globals.get(name).cloned()
    }
}

pub type ParserResult<T> = Result<T, ParserError>;
pub type ParserError = String;

blanket_trait!(
    pub trait WasmParserBuiltin:
        Builtin
        + From<Throw>
        + From<ResolveDeep>
        + From<ToString>
        + From<CollectString>
        + From<Merge>
        + From<Push>
        + From<PushFront>
        + From<Chain>
        + From<Subtract>
        + From<Add>
        + From<Not>
        + From<Multiply>
        + From<Divide>
        + From<Remainder>
        + From<Pow>
        + From<Lt>
        + From<Gt>
        + From<Lte>
        + From<Gte>
        + From<Eq>
        + From<And>
        + From<Or>
        + From<Has>
        + From<If>
        + From<IfError>
        + From<FormatErrorMessage>
        + From<CollectList>
        + From<Accessor>
        + From<Apply>
        + From<Construct>
    {
    }
);

fn err<T: std::fmt::Debug>(message: &str, _node: T) -> ParserError {
    String::from(message)
}

fn err_unimplemented<T: std::fmt::Debug>(node: T) -> String {
    err("Unsupported syntax", node)
}

fn err_unreachable<T: std::fmt::Debug>(node: T) -> String {
    err("Unreachable code", node)
}

#[derive(Clone)]
struct LexicalScope {
    bindings: Vec<Option<String>>,
}
impl LexicalScope {
    fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }
    fn from(identifiers: impl IntoIterator<Item = Option<String>>) -> Self {
        Self {
            bindings: identifiers.into_iter().collect(),
        }
    }
    fn depth(&self) -> usize {
        self.bindings.len()
    }
    fn create_child(&self, identifiers: impl IntoIterator<Item = Option<String>>) -> LexicalScope {
        LexicalScope {
            bindings: self.bindings.iter().cloned().chain(identifiers).collect(),
        }
    }
    fn get(&self, identifier: &str) -> Option<usize> {
        Some(
            self.bindings
                .iter()
                .rev()
                .enumerate()
                .find(|(_, key)| match key {
                    None => false,
                    Some(key) => *key == identifier,
                })
                .map(|(i, _)| i)?,
        )
    }
}

pub fn parse<T: Expression>(
    input: &str,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let program = parse_ast(input, None)?;
    parse_script_contents(program.body.into_iter(), env, factory, allocator)
}

pub fn parse_module<T: Expression>(
    input: &str,
    env: &Env<T>,
    path: &Path,
    loader: &impl Fn(&str, &Path) -> Option<Result<T, String>>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let program = parse_ast(input, Some(path))?;
    parse_module_contents(
        program.body.into_iter(),
        env,
        path,
        loader,
        factory,
        allocator,
    )
}

fn format_source_error(location: Span, message: &str, source_map: &SourceMap) -> String {
    let location = match source_map.span_to_lines(location) {
        Ok(regions) => match regions.lines.len() {
            0 => format!("{}", regions.file.name),
            1 => {
                let line = regions.lines.first().unwrap();
                format!(
                    "{}:{}:{}-{}",
                    regions.file.name,
                    line.line_index + 1,
                    line.start_col.to_usize() + 1,
                    line.end_col.to_usize() + 1
                )
            }
            _ => {
                let first = regions.lines.first().unwrap();
                let last = regions.lines.last().unwrap();
                format!(
                    "{}:{}:{}-{}:{}",
                    regions.file.name,
                    first.line_index + 1,
                    first.start_col.to_usize() + 1,
                    last.line_index + 1,
                    last.end_col.to_usize() + 1
                )
            }
        },
        Err(_) => format!("{}", source_map.span_to_filename(location)),
    };
    format!("{}: {}", location, message)
}

fn parse_ast(input: &str, path: Option<&Path>) -> ParserResult<Module> {
    let source_map: Lrc<SourceMap> = Default::default();
    let source = source_map.new_source_file(
        match path {
            Some(path) => FileName::Real(path.to_path_buf()),
            None => FileName::Anon,
        },
        String::from(input),
    );
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        EsVersion::latest(),
        StringInput::from(&*source),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    let syntax_errors = parser.take_errors();
    if !syntax_errors.is_empty() {
        return Err(syntax_errors
            .into_iter()
            .map(|error| format_source_error(error.span(), &error.into_kind().msg(), &source_map))
            .collect::<Vec<_>>()
            .join("\n"));
    }

    parser
        .parse_module()
        .map_err(|err| format_source_error(err.span(), &err.into_kind().msg(), &source_map))
}

fn parse_script_contents<T: Expression>(
    program: impl IntoIterator<Item = ModuleItem> + ExactSizeIterator,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let body = program
        .into_iter()
        .map(|node| match node {
            ModuleItem::Stmt(node) => Ok(node),
            _ => Err(err_unimplemented(node)),
        })
        .collect::<ParserResult<Vec<_>>>()?;
    match parse_block(&body, &LexicalScope::new(), &env, factory, allocator)? {
        None => Err(String::from("No expression to evaluate")),
        Some(expression) => Ok(expression),
    }
}

fn parse_module_contents<T: Expression>(
    program: impl IntoIterator<Item = ModuleItem> + ExactSizeIterator,
    env: &Env<T>,
    path: &Path,
    loader: &impl Fn(&str, &Path) -> Option<Result<T, String>>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let num_statements = program.len();
    let (body, import_bindings) = program.into_iter().fold(
        Ok((Vec::with_capacity(num_statements), Vec::new())),
        |results, node| {
            let (mut body, mut import_bindings) = results?;
            match node {
                ModuleItem::ModuleDecl(node) => match node {
                    ModuleDecl::Import(node) => {
                        let bindings =
                            parse_module_import(&node, path, loader, factory, allocator)?;
                        import_bindings.extend(bindings);
                        Ok((body, import_bindings))
                    }
                    ModuleDecl::ExportDecl(node) => Err(err_unimplemented(node)),
                    ModuleDecl::ExportNamed(node) => Err(err_unimplemented(node)),
                    ModuleDecl::ExportDefaultDecl(node) => Err(err_unimplemented(node)),
                    ModuleDecl::ExportDefaultExpr(node) => {
                        body.push(Stmt::Expr(ExprStmt {
                            span: node.span,
                            expr: node.expr,
                        }));
                        Ok((body, import_bindings))
                    }
                    ModuleDecl::ExportAll(_) => Err(err_unimplemented(node)),
                    _ => Err(err_unimplemented(node)),
                },
                ModuleItem::Stmt(node) => {
                    body.push(node);
                    Ok((body, import_bindings))
                }
            }
        },
    )?;
    let (import_keys, import_initializers): (Vec<_>, Vec<_>) = import_bindings.into_iter().unzip();
    let scope = LexicalScope::from(import_keys.into_iter().map(Some));
    match parse_block(&body, &scope, &env, factory, allocator)? {
        None => Err(String::from("Missing default module export")),
        Some(expression) => Ok(if import_initializers.is_empty() {
            expression
        } else {
            create_declaration_block(import_initializers, expression, factory)
        }),
    }
}

fn parse_module_import<T: Expression>(
    node: &ImportDecl,
    path: &Path,
    loader: &impl Fn(&str, &Path) -> Option<Result<T, String>>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<Vec<(String, T)>>
where
    T::Builtin: WasmParserBuiltin,
{
    let module_path = parse_string(&node.src);
    let module = match loader(&module_path, path)
        .unwrap_or_else(|| Err(String::from("No compatible loaders registered")))
    {
        Ok(module) => Ok(module),
        Err(error) => Err(err(
            &format!("Failed to import '{}': {}", module_path, error),
            node,
        )),
    }?;
    Ok(node
        .specifiers
        .iter()
        .map(|specifier| {
            let (identifier, value) = match specifier {
                ImportSpecifier::Default(node) => {
                    let identifier = parse_identifier(&node.local);
                    let value = get_static_field(module.clone(), "default", factory, allocator);
                    (identifier, value)
                }
                ImportSpecifier::Namespace(node) => {
                    let identifier = parse_identifier(&node.local);
                    let value = module.clone();
                    (identifier, value)
                }
                ImportSpecifier::Named(node) => {
                    let identifier = parse_identifier(&node.local);
                    let imported_field = node
                        .imported
                        .as_ref()
                        .map(|export_name| match export_name {
                            ModuleExportName::Ident(name) => String::from(parse_identifier(name)),
                            ModuleExportName::Str(name) => parse_string(name),
                        })
                        .unwrap_or_else(|| String::from(identifier));
                    let value =
                        get_static_field(module.clone(), &imported_field, factory, allocator);
                    (identifier, value)
                }
            };
            (String::from(identifier), value)
        })
        .collect())
}

fn parse_block<'a, T: Expression>(
    body: impl IntoIterator<Item = &'a Stmt>,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<Option<T>>
where
    T::Builtin: WasmParserBuiltin,
{
    parse_block_statements(body, None, scope, env, factory, allocator)
}

fn parse_block_statements<'a, T: Expression>(
    remaining: impl IntoIterator<Item = &'a Stmt>,
    result: Option<T>,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<Option<T>>
where
    T::Builtin: WasmParserBuiltin,
{
    let mut remaining = remaining.into_iter();
    let node = remaining.next();
    match node {
        None => Ok(result),
        Some(statement) => {
            if result.is_some() {
                return Err(err_unreachable(statement));
            }
            match statement {
                Stmt::Decl(node) => match node {
                    Decl::Var(node) => match node.kind {
                        VarDeclKind::Const => {
                            let (initializers, child_scope) = parse_variable_declarators(
                                &node.decls,
                                &scope,
                                env,
                                factory,
                                allocator,
                            )?;
                            let body_scope = child_scope.as_ref().unwrap_or(scope);
                            let body = parse_block_statements(
                                remaining, result, body_scope, env, factory, allocator,
                            )?;
                            match body {
                                None => Ok(None),
                                Some(body) => {
                                    Ok(Some(create_declaration_block(initializers, body, factory)))
                                }
                            }
                        }
                        _ => Err(err_unimplemented(node)),
                    },
                    Decl::Fn(node) => Err(err_unimplemented(node)),
                    Decl::Class(node) => Err(err_unimplemented(node)),
                    _ => Err(err_unimplemented(node)),
                },
                Stmt::Expr(node) => {
                    let expression = parse_expression(&node.expr, &scope, env, factory, allocator)?;
                    let result = Some(expression);
                    parse_block_statements(remaining, result, scope, env, factory, allocator)
                }
                Stmt::Return(node) => match &node.arg {
                    None => Err(err("Missing return value", node)),
                    Some(node) => {
                        let expression = parse_expression(node, &scope, env, factory, allocator)?;
                        let result = Some(expression);
                        parse_block_statements(remaining, result, scope, env, factory, allocator)
                    }
                },
                Stmt::Throw(node) => {
                    let expression =
                        parse_throw_statement(&node.arg, &scope, env, factory, allocator)?;
                    let result = Some(expression);
                    parse_block_statements(remaining, result, scope, env, factory, allocator)
                }
                Stmt::If(node) => {
                    let condition = parse_expression(&node.test, scope, env, factory, allocator)?;
                    let consequent = parse_if_branch(&node.cons, scope, env, factory, allocator)?;
                    match &node.alt {
                        Some(node) => {
                            let alternate = parse_if_branch(&node, scope, env, factory, allocator)?;
                            let expression = create_if_expression(
                                condition, consequent, alternate, factory, allocator,
                            );
                            let result = Some(expression);
                            parse_block_statements(
                                remaining, result, scope, env, factory, allocator,
                            )
                        }
                        None => {
                            let alternate = parse_branch(
                                &statement, remaining, scope, env, factory, allocator,
                            )?;
                            let result = create_if_expression(
                                condition, consequent, alternate, factory, allocator,
                            );
                            Ok(Some(result))
                        }
                    }
                }
                Stmt::Try(node) => {
                    if let Some(node) = &node.finalizer {
                        Err(err_unimplemented(node))
                    } else if let Some(handler) = &node.handler {
                        let error_identifier = match &handler.param {
                            Some(pattern) => match pattern {
                                Pat::Ident(identifier) => Ok(Some(identifier)),
                                // TODO: Support destructuring patterns in catch variable assignment
                                _ => Err(err_unimplemented(pattern)),
                            },
                            None => Ok(None),
                        }?;
                        let BlockStmt { stmts: body, .. } = &node.block;
                        let BlockStmt { stmts: handler, .. } = &handler.body;
                        let expression = create_try_catch_expression(
                            &statement,
                            body,
                            handler,
                            error_identifier,
                            scope,
                            env,
                            factory,
                            allocator,
                        )?;
                        let result = Some(expression);
                        parse_block_statements(remaining, result, scope, env, factory, allocator)
                    } else {
                        Err(err_unimplemented(node))
                    }
                }
                Stmt::Switch(_) => Err(err_unimplemented(statement)),
                Stmt::Empty(_) => {
                    parse_block_statements(remaining, result, scope, env, factory, allocator)
                }
                _ => Err(err_unimplemented(statement)),
            }
        }
    }
}

fn create_declaration_block<T: Expression>(
    initializers: impl IntoIterator<Item = T, IntoIter = impl DoubleEndedIterator<Item = T>>,
    body: T,
    factory: &impl ExpressionFactory<T>,
) -> T {
    initializers
        .into_iter()
        .rev()
        .fold(body, |body, initializer| {
            factory.create_let_term(initializer, body)
        })
}

fn parse_variable_declarators<T: Expression>(
    declarators: &[VarDeclarator],
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<(
    impl IntoIterator<Item = T, IntoIter = impl DoubleEndedIterator<Item = T>>,
    Option<LexicalScope>,
)>
where
    T::Builtin: WasmParserBuiltin,
{
    declarators
        .iter()
        .fold(Ok((Vec::new(), None)), |results, node| {
            let (mut results, existing_scope) = results?;
            let current_scope = existing_scope.as_ref().unwrap_or(scope);
            let (initializers, child_scope) =
                parse_variable_declarator(node, current_scope, env, factory, allocator)?;
            results.extend(initializers);
            Ok((results, child_scope.or(existing_scope)))
        })
}

fn parse_variable_declarator<T: Expression>(
    node: &VarDeclarator,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<(impl IntoIterator<Item = T>, Option<LexicalScope>)>
where
    T::Builtin: WasmParserBuiltin,
{
    let init = node
        .init
        .as_ref()
        .ok_or_else(|| err("Missing variable initializer", node))?;
    let value = parse_expression(&init, scope, env, factory, allocator)?;
    match &node.name {
        Pat::Ident(node) => {
            let identifier = parse_identifier(&node.id);
            Ok((
                vec![value],
                Some(scope.create_child(once(Some(String::from(identifier))))),
            ))
        }
        Pat::Object(node) => {
            let (initializers, child_scope) = parse_object_destructuring_pattern_bindings(
                value,
                &node.props,
                scope,
                env,
                factory,
                allocator,
            )?;
            Ok((initializers.into_iter().collect::<Vec<_>>(), child_scope))
        }
        Pat::Array(node) => {
            let (initializers, child_scope) = parse_array_destructuring_pattern_bindings(
                value,
                &node.elems,
                scope,
                env,
                factory,
                allocator,
            )?;
            Ok((initializers.into_iter().collect::<Vec<_>>(), child_scope))
        }
        Pat::Rest(_) => Err(err_unimplemented(node)),
        Pat::Assign(_) => Err(err_unimplemented(node)),
        _ => Err(err_unimplemented(node)),
    }
}

fn parse_object_destructuring_pattern_bindings<T: Expression>(
    target: T,
    properties: &[ObjectPatProp],
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<(Vec<T>, Option<LexicalScope>)>
where
    T::Builtin: WasmParserBuiltin,
{
    let properties = properties
        .iter()
        .map(|property| match property {
            ObjectPatProp::KeyValue(node) => {
                let identifier = match &*node.value {
                    Pat::Ident(node) => Ok(parse_identifier(&node.id)),
                    Pat::Object(_) => Err(err_unimplemented(node)),
                    Pat::Array(_) => Err(err_unimplemented(node)),
                    Pat::Rest(_) => Err(err_unimplemented(node)),
                    Pat::Assign(_) => Err(err_unimplemented(node)),
                    _ => Err(err_unimplemented(node)),
                }?;
                let field_name = parse_prop_name(&node.key, scope, env, factory, allocator)?;
                let field_accessor =
                    factory.create_string_term(allocator.create_string(field_name));
                Ok((identifier, field_accessor))
            }
            ObjectPatProp::Assign(node) => {
                if node.value.is_some() {
                    Err(err_unimplemented(node))
                } else {
                    let identifier = parse_identifier(&node.key);
                    let field_accessor = factory
                        .create_string_term(allocator.create_string(String::from(identifier)));
                    Ok((identifier, field_accessor))
                }
            }
            ObjectPatProp::Rest(node) => Err(err_unimplemented(node)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    match properties.len() {
        0 => Ok((Vec::new(), None)),
        1 => {
            let (identifier, field_accessor) = properties.into_iter().next().unwrap();
            let initializer = get_dynamic_field(target, field_accessor, factory, allocator);
            Ok((
                vec![initializer],
                Some(scope.create_child(once(Some(String::from(identifier))))),
            ))
        }
        _ => {
            let mut initializers = Vec::with_capacity(1 + properties.len());
            let initializer_scope = scope.create_child(once(None));
            initializers.push(target.clone());
            let initializer_depth = initializer_scope.depth();
            properties
                .into_iter()
                .fold(
                    Ok((initializers, scope.create_child(once(None)))),
                    |result, property| {
                        let (mut initializers, existing_scope) = result?;
                        let (identifier, field_accessor) = property;
                        let scope_offset = existing_scope.depth() - initializer_depth;
                        let initializer = get_dynamic_field(
                            factory.create_variable_term(scope_offset),
                            field_accessor,
                            factory,
                            allocator,
                        );
                        initializers.push(initializer);
                        let next_scope =
                            existing_scope.create_child(once(Some(String::from(identifier))));
                        Ok((initializers, next_scope))
                    },
                )
                .map(|(initializers, scope)| (initializers, Some(scope)))
        }
    }
}

fn parse_array_destructuring_pattern_bindings<T: Expression>(
    target: T,
    accessors: &[Option<Pat>],
    scope: &LexicalScope,
    _env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<(impl IntoIterator<Item = T>, Option<LexicalScope>)>
where
    T::Builtin: WasmParserBuiltin,
{
    let accessors = accessors
        .iter()
        .enumerate()
        .filter_map(|(index, accessor)| match accessor {
            Some(accessor) => Some((index, accessor)),
            None => None,
        })
        .map(|(index, accessor)| match accessor {
            Pat::Ident(identifier) => {
                let identifier = parse_identifier(&identifier.id);
                Ok((identifier, index))
            }
            Pat::Rest(_) => Err(err_unimplemented(accessor)),
            _ => Err(err_unimplemented(accessor)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    match accessors.len() {
        0 => Ok((Vec::new(), None)),
        1 => {
            let (identifier, index) = accessors.into_iter().next().unwrap();
            let initializer = get_indexed_field(target, index, factory, allocator);
            Ok((
                vec![initializer],
                Some(scope.create_child(once(Some(String::from(identifier))))),
            ))
        }
        _ => {
            let mut initializers = Vec::with_capacity(1 + accessors.len());
            let initializer_scope = scope.create_child(once(None));
            initializers.push(target.clone());
            let initializer_depth = initializer_scope.depth();
            accessors
                .into_iter()
                .fold(Ok((initializers, initializer_scope)), |result, property| {
                    let (mut initializers, existing_scope) = result?;
                    let (identifier, index) = property;
                    let scope_offset = existing_scope.depth() - initializer_depth;
                    let initializer = get_indexed_field(
                        factory.create_variable_term(scope_offset),
                        index,
                        factory,
                        allocator,
                    );
                    initializers.push(initializer);
                    let next_scope =
                        existing_scope.create_child(once(Some(String::from(identifier))));
                    Ok((initializers, next_scope))
                })
                .map(|(initializers, scope)| (initializers, Some(scope)))
        }
    }
}

fn parse_identifier(node: &Ident) -> &str {
    &node.sym
}

fn parse_prop_name<T: Expression>(
    node: &PropName,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<String>
where
    T::Builtin: WasmParserBuiltin,
{
    match node {
        PropName::Ident(key) => Ok(String::from(parse_identifier(key))),
        PropName::Str(key) => Ok(parse_string(key)),
        PropName::Num(key) => {
            let key = key.value;
            match as_integer(key) {
                Some(key) => Ok(format!("{}", key)),
                _ => Ok(format!("{}", key)),
            }
        }
        PropName::BigInt(key) => Err(err_unimplemented(key)),
        PropName::Computed(key) => {
            let dynamic_key = parse_expression(&key.expr, scope, env, factory, allocator)?;
            if let Some(term) = factory.match_string_term(&dynamic_key) {
                Ok(String::from(term.value().as_deref().as_str().deref()))
            } else if let Some(_) = factory.match_nil_term(&dynamic_key) {
                Ok(format!("{}", dynamic_key))
            } else if let Some(_) = factory.match_boolean_term(&dynamic_key) {
                Ok(format!("{}", dynamic_key))
            } else if let Some(_) = factory.match_int_term(&dynamic_key) {
                Ok(format!("{}", dynamic_key))
            } else if let Some(term) = factory.match_float_term(&dynamic_key) {
                Ok(if let Some(value) = as_integer(term.value()) {
                    format!("{}", value)
                } else {
                    format!("{}", dynamic_key)
                })
            } else {
                Err(err_unimplemented(dynamic_key))
            }
        }
    }
}

fn parse_throw_statement<T: Expression>(
    value: &Expr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let error = parse_expression(value, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Throw),
        allocator.create_unit_list(factory.create_application_term(
            factory.create_builtin_term(ResolveDeep),
            allocator.create_unit_list(error),
        )),
    ))
}

fn parse_if_branch<T: Expression>(
    node: &Stmt,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    match node {
        Stmt::Block(block) => {
            let BlockStmt { stmts: body, .. } = block;
            parse_branch(node, body, scope, env, factory, allocator)
        }
        _ => parse_branch(node, &vec![node.clone()], scope, env, factory, allocator),
    }
}

fn parse_branch<'a, T: Expression>(
    node: &Stmt,
    body: impl IntoIterator<Item = &'a Stmt>,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let expression = parse_block(body, scope, env, factory, allocator)?;
    match expression {
        None => Err(err("Unterminated branch", node)),
        Some(expression) => Ok(expression),
    }
}

fn parse_expression<T: Expression>(
    node: &Expr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    match node {
        Expr::Paren(node) => parse_expression(&node.expr, scope, env, factory, allocator),
        Expr::Ident(node) => parse_variable_reference(node, scope, env, factory),
        Expr::Lit(node) => parse_literal(node, factory, allocator),
        Expr::Tpl(node) => parse_template_literal(node, scope, env, factory, allocator),
        Expr::TaggedTpl(node) => parse_tagged_template(node, scope, env, factory, allocator),
        Expr::Unary(node) => parse_unary_expression(node, scope, env, factory, allocator),
        Expr::Bin(node) => parse_binary_expression(node, scope, env, factory, allocator),
        Expr::Cond(node) => parse_conditional_expression(node, scope, env, factory, allocator),
        Expr::Arrow(node) => parse_arrow_function_expression(node, scope, env, factory, allocator),
        Expr::Member(node) => parse_member_expression(node, scope, env, factory, allocator),
        Expr::Call(node) => parse_call_expression(node, scope, env, factory, allocator),
        Expr::New(node) => parse_constructor_expression(node, scope, env, factory, allocator),
        Expr::Object(node) => parse_object_literal(node, scope, env, factory, allocator),
        Expr::Array(node) => parse_array_literal(node, scope, env, factory, allocator),
        _ => Err(err_unimplemented(node)),
    }
}

fn parse_expressions<'a, T: Expression>(
    expressions: impl IntoIterator<Item = &'a Expr>,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<Vec<T>>
where
    T::Builtin: WasmParserBuiltin,
{
    expressions
        .into_iter()
        .map(|node| parse_expression(node, scope, env, factory, allocator))
        .collect::<Result<Vec<_>, _>>()
}

fn parse_variable_reference<T: Expression>(
    node: &Ident,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
) -> ParserResult<T> {
    let name = parse_identifier(node);
    let offset = scope.get(name);
    match offset {
        Some(offset) => Ok(factory.create_variable_term(offset)),
        None => match env.global(name) {
            Some(value) => Ok(value),
            None => Err(err(&format!("Invalid reference: '{}'", name), node)),
        },
    }
}

fn parse_literal<T: Expression>(
    node: &Lit,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T> {
    match node {
        Lit::Null(node) => parse_null_literal(node, factory),
        Lit::Bool(node) => parse_boolean_literal(node, factory),
        Lit::Num(node) => parse_number_literal(node, factory),
        Lit::Str(node) => parse_string_literal(node, factory, allocator),
        _ => Err(err_unimplemented(node)),
    }
}

fn parse_null_literal<T: Expression>(
    _node: &Null,
    factory: &impl ExpressionFactory<T>,
) -> ParserResult<T> {
    Ok(factory.create_nil_term())
}

fn parse_boolean_literal<T: Expression>(
    node: &Bool,
    factory: &impl ExpressionFactory<T>,
) -> ParserResult<T> {
    Ok(factory.create_boolean_term(node.value))
}

fn parse_number_literal<T: Expression>(
    node: &Number,
    factory: &impl ExpressionFactory<T>,
) -> ParserResult<T> {
    Ok(factory.create_float_term(node.value))
}

fn parse_string_literal<T: Expression>(
    node: &Str,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T> {
    let value = parse_string(node);
    Ok(factory.create_string_term(allocator.create_string(value)))
}

fn parse_string(node: &Str) -> String {
    parse_escaped_string(&node.value)
}

fn parse_escaped_string(value: &str) -> String {
    value
        .chars()
        .fold(
            (String::with_capacity(value.len()), false),
            |(mut result, is_escape), current| {
                if current == '\\' && !is_escape {
                    (result, true)
                } else {
                    result.push(current);
                    (result, false)
                }
            },
        )
        .0
}

fn parse_template_literal<T: Expression>(
    node: &Tpl,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let args = node
        .quasis
        .iter()
        .map(|quasi| {
            let value = parse_template_element(quasi);
            if value.is_empty() {
                None
            } else {
                Some(factory.create_string_term(allocator.create_string(value)))
            }
        })
        .zip(
            node.exprs
                .iter()
                .map(|expression| {
                    let value = parse_expression(expression, scope, env, factory, allocator)?;
                    Ok(Some(factory.create_application_term(
                        factory.create_builtin_term(ToString),
                        allocator.create_unit_list(value),
                    )))
                })
                .chain(once(Ok(None))),
        )
        .flat_map(|(quasi, expression)| once(Ok(quasi)).chain(once(expression)))
        .filter_map(|arg| match arg {
            Err(error) => Some(Err(error)),
            Ok(Some(arg)) => Some(Ok(arg)),
            Ok(None) => None,
        })
        .collect::<ParserResult<Vec<_>>>()?;
    Ok(match args.len() {
        0 => factory.create_string_term(allocator.create_static_string("")),
        1 => args.into_iter().next().unwrap(),
        _ => factory.create_application_term(
            factory.create_builtin_term(CollectString),
            allocator.create_unit_list(factory.create_list_term(allocator.create_list(args))),
        ),
    })
}

fn parse_template_element(node: &TplElement) -> String {
    node.cooked
        .as_ref()
        .map(|value| {
            let value: &str = &value;
            String::from(value)
        })
        .unwrap_or_else(|| parse_escaped_string(&node.raw))
}

fn parse_tagged_template<T: Expression>(
    node: &TaggedTpl,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    parse_template_literal(&node.tpl, scope, env, factory, allocator)
}

fn parse_object_literal<T: Expression>(
    node: &ObjectLit,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    enum ObjectLiteralField<T> {
        Property(String, T),
        Spread(T),
    }
    let elements =
        node.props
            .iter()
            .fold(Ok(Vec::with_capacity(node.props.len())), |results, node| {
                let mut elements = results?;
                match node {
                    PropOrSpread::Prop(prop) => match &**prop {
                        Prop::KeyValue(prop) => {
                            let key = parse_prop_name(&prop.key, scope, env, factory, allocator)?;
                            let value =
                                parse_expression(&prop.value, scope, env, factory, allocator)?;
                            elements.push(ObjectLiteralField::Property(key, value));
                            Ok(elements)
                        }
                        Prop::Shorthand(prop) => {
                            let key = String::from(parse_identifier(&prop));
                            let value = parse_variable_reference(&prop, scope, env, factory)?;
                            elements.push(ObjectLiteralField::Property(key, value));
                            Ok(elements)
                        }
                        Prop::Method(_) => Err(err_unimplemented(prop)),
                        Prop::Getter(_) => Err(err_unimplemented(prop)),
                        Prop::Setter(_) => Err(err_unimplemented(prop)),
                        _ => Err(err_unimplemented(prop)),
                    },
                    PropOrSpread::Spread(node) => {
                        let value = parse_expression(&node.expr, scope, env, factory, allocator)?;
                        elements.push(ObjectLiteralField::Spread(value));
                        Ok(elements)
                    }
                }
            })?;

    let field_sets = elements
        .into_iter()
        .flat_map(|element| {
            let spread_delimiter: Option<Option<ObjectLiteralField<T>>> =
                if matches!(&element, ObjectLiteralField::Spread(_)) {
                    Some(None)
                } else {
                    None
                };
            spread_delimiter.into_iter().chain(once(Some(element)))
        })
        .chain(once(None))
        .fold((Vec::new(), Vec::new()), |results, property| {
            let (mut field_sets, mut current_set) = results;
            match property {
                Some(ObjectLiteralField::Spread(value)) => {
                    field_sets.push(value);
                    (field_sets, Vec::new())
                }
                Some(ObjectLiteralField::Property(key, value)) => {
                    current_set.push((
                        factory.create_string_term(allocator.create_string(key)),
                        value,
                    ));
                    (field_sets, current_set)
                }
                None => {
                    if !current_set.is_empty() {
                        field_sets.push(create_record(current_set, factory, allocator));
                    }
                    (field_sets, Vec::new())
                }
            }
        })
        .0;

    Ok(if field_sets.len() <= 1 {
        field_sets.into_iter().next().unwrap_or_else(|| {
            factory.create_record_term(
                allocator.create_struct_prototype(allocator.create_empty_list()),
                allocator.create_empty_list(),
            )
        })
    } else {
        factory.create_application_term(
            factory.create_builtin_term(Merge),
            // TODO: Avoid unnecessary list allocation when merging objects
            allocator.create_unit_list(factory.create_application_term(
                factory.create_builtin_term(CollectList),
                allocator.create_list(field_sets),
            )),
        )
    })
}

fn parse_array_literal<T: Expression>(
    node: &ArrayLit,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    enum ArrayLiteralFields<T> {
        Items(Vec<T>),
        Spread(T),
    }
    let elements =
        node.elems
            .iter()
            .fold(Ok(Vec::with_capacity(node.elems.len())), |results, node| {
                let mut elements = results?;
                match node {
                    None => Err(err("Missing array item", node)),
                    Some(node) => {
                        if node.spread.is_some() {
                            let value =
                                parse_expression(&node.expr, scope, env, factory, allocator)?;
                            elements.push(ArrayLiteralFields::Spread(value));
                            Ok(elements)
                        } else {
                            let value =
                                parse_expression(&node.expr, scope, env, factory, allocator)?;
                            if let Some(ArrayLiteralFields::Items(_)) = elements.last() {
                                match elements.pop() {
                                    Some(ArrayLiteralFields::Items(mut items)) => {
                                        items.push(value);
                                        elements.push(ArrayLiteralFields::Items(items));
                                    }
                                    _ => {}
                                }
                            } else {
                                elements.push(ArrayLiteralFields::Items(vec![value]))
                            }
                            Ok(elements)
                        }
                    }
                }
            })?;
    if elements.len() == 2 {
        let left = elements.first().unwrap();
        let right = elements.last().unwrap();
        match (left, right) {
            (ArrayLiteralFields::Spread(target), ArrayLiteralFields::Items(items))
                if items.len() == 1 =>
            {
                return Ok(factory.create_application_term(
                    factory.create_builtin_term(Push),
                    allocator
                        .create_pair(target.clone(), items.into_iter().next().cloned().unwrap()),
                ))
            }
            (ArrayLiteralFields::Items(items), ArrayLiteralFields::Spread(target))
                if items.len() == 1 =>
            {
                return Ok(factory.create_application_term(
                    factory.create_builtin_term(PushFront),
                    allocator
                        .create_pair(target.clone(), items.into_iter().next().cloned().unwrap()),
                ))
            }
            _ => {}
        }
    }

    let item_sets = elements.into_iter().map(|properties| match properties {
        ArrayLiteralFields::Spread(value) => value,
        ArrayLiteralFields::Items(items) => factory.create_list_term(allocator.create_list(items)),
    });
    Ok(if item_sets.len() >= 2 {
        factory.create_application_term(
            factory.create_builtin_term(Chain),
            allocator.create_list(item_sets),
        )
    } else {
        match item_sets.into_iter().next() {
            Some(value) => value,
            None => factory.create_list_term(allocator.create_empty_list()),
        }
    })
}

fn parse_unary_expression<T: Expression>(
    node: &UnaryExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    match node.op {
        UnaryOp::Minus => parse_unary_minus_expression(node, scope, env, factory, allocator),
        UnaryOp::Plus => parse_unary_plus_expression(node, scope, env, factory, allocator),
        UnaryOp::Bang => parse_unary_not_expression(node, scope, env, factory, allocator),
        _ => Err(err_unimplemented(node)),
    }
}

fn parse_binary_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    match node.op {
        BinaryOp::Add => parse_binary_add_expression(node, scope, env, factory, allocator),
        BinaryOp::Sub => parse_binary_subtract_expression(node, scope, env, factory, allocator),
        BinaryOp::Mul => parse_binary_multiply_expression(node, scope, env, factory, allocator),
        BinaryOp::Div => parse_binary_divide_expression(node, scope, env, factory, allocator),
        BinaryOp::Mod => parse_binary_remainder_expression(node, scope, env, factory, allocator),
        BinaryOp::Exp => parse_binary_pow_expression(node, scope, env, factory, allocator),
        BinaryOp::Lt => parse_binary_lt_expression(node, scope, env, factory, allocator),
        BinaryOp::Gt => parse_binary_gt_expression(node, scope, env, factory, allocator),
        BinaryOp::LtEq => parse_binary_lte_expression(node, scope, env, factory, allocator),
        BinaryOp::GtEq => parse_binary_gte_expression(node, scope, env, factory, allocator),
        BinaryOp::EqEq | BinaryOp::EqEqEq => {
            parse_binary_equal_expression(node, scope, env, factory, allocator)
        }
        BinaryOp::NotEq | BinaryOp::NotEqEq => {
            parse_binary_not_equal_expression(node, scope, env, factory, allocator)
        }
        BinaryOp::LogicalAnd => {
            parse_binary_logical_and_expression(node, scope, env, factory, allocator)
        }
        BinaryOp::LogicalOr => {
            parse_binary_logical_or_expression(node, scope, env, factory, allocator)
        }
        BinaryOp::In => parse_binary_in_expression(node, scope, env, factory, allocator),
        _ => Err(err_unimplemented(node)),
    }
}

fn parse_unary_minus_expression<T: Expression>(
    node: &UnaryExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let operand = parse_expression(&node.arg, scope, env, factory, allocator)?;
    Ok(if let Some(term) = factory.match_int_term(&operand) {
        factory.create_int_term(-term.value())
    } else if let Some(term) = factory.match_float_term(&operand) {
        factory.create_float_term(-term.value())
    } else {
        factory.create_application_term(
            factory.create_builtin_term(Subtract),
            allocator.create_pair(factory.create_float_term(0.0), operand),
        )
    })
}

fn parse_unary_plus_expression<T: Expression>(
    node: &UnaryExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let operand = parse_expression(&node.arg, scope, env, factory, allocator)?;
    Ok(if let Some(_) = factory.match_int_term(&operand) {
        operand
    } else if let Some(_) = factory.match_float_term(&operand) {
        operand
    } else {
        factory.create_application_term(
            factory.create_builtin_term(Add),
            allocator.create_pair(factory.create_float_term(0.0), operand),
        )
    })
}

fn parse_unary_not_expression<T: Expression>(
    node: &UnaryExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let operand = parse_expression(&node.arg, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Not),
        allocator.create_unit_list(operand),
    ))
}

fn parse_binary_add_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Add),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_subtract_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Subtract),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_multiply_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Multiply),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_divide_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Divide),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_remainder_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Remainder),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_pow_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Pow),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_lt_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Lt),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_gt_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Gt),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_lte_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Lte),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_gte_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Gte),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_equal_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Eq),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_not_equal_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let expression = parse_binary_equal_expression(node, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Not),
        allocator.create_unit_list(expression),
    ))
}

fn parse_binary_logical_and_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(And),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_logical_or_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Or),
        allocator.create_pair(left, right),
    ))
}

fn parse_binary_in_expression<T: Expression>(
    node: &BinExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let left = parse_expression(&node.left, scope, env, factory, allocator)?;
    let right = parse_expression(&node.right, scope, env, factory, allocator)?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Has),
        allocator.create_pair(right, left),
    ))
}

fn parse_conditional_expression<T: Expression>(
    node: &CondExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let condition = parse_expression(&node.test, scope, env, factory, allocator)?;
    let consequent = parse_expression(&node.cons, scope, env, factory, allocator)?;
    let alternate = parse_expression(&node.alt, scope, env, factory, allocator)?;
    Ok(create_if_expression(
        condition, consequent, alternate, factory, allocator,
    ))
}

fn create_if_expression<T: Expression>(
    condition: T,
    consequent: T,
    alternate: T,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: WasmParserBuiltin,
{
    factory.create_application_term(
        factory.create_builtin_term(If),
        allocator.create_triple(condition, consequent, alternate),
    )
}

fn create_try_catch_expression<'a, T: Expression>(
    node: &Stmt,
    body: impl IntoIterator<Item = &'a Stmt>,
    handler: impl IntoIterator<Item = &'a Stmt>,
    error_identifier: Option<&BindingIdent>,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let body = parse_branch(node, body, scope, env, factory, allocator)?;
    match error_identifier {
        Some(identifier) => {
            let identifier = String::from(parse_identifier(&identifier.id));
            let handler = factory.create_lambda_term(
                1,
                parse_branch(
                    node,
                    handler,
                    &scope.create_child([None, Some(identifier)]),
                    env,
                    factory,
                    allocator,
                )?,
            );
            Ok(factory.create_application_term(
                factory.create_builtin_term(IfError),
                allocator.create_pair(
                    body,
                    factory.create_lambda_term(
                        1,
                        factory.create_application_term(
                            handler,
                            allocator.create_unit_list(factory.create_application_term(
                                global_aggregate_error(factory, allocator),
                                allocator.create_unit_list(factory.create_variable_term(0)),
                            )),
                        ),
                    ),
                ),
            ))
        }
        None => {
            let handler = factory.create_lambda_term(
                1,
                parse_branch(
                    node,
                    handler,
                    &scope.create_child([None]),
                    env,
                    factory,
                    allocator,
                )?,
            );
            Ok(factory.create_application_term(
                factory.create_builtin_term(IfError),
                allocator.create_pair(body, handler),
            ))
        }
    }
}

fn global_aggregate_error<T: Expression>(
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: WasmParserBuiltin,
{
    // TODO: Add optional JS AggregateError override message
    factory.create_lambda_term(
        1,
        create_record(
            once((
                factory.create_string_term(allocator.create_static_string("name")),
                factory.create_string_term(allocator.create_static_string("AggregateError")),
            ))
            .chain(once((
                factory.create_string_term(allocator.create_static_string("message")),
                factory.create_application_term(
                    factory.create_builtin_term(FormatErrorMessage),
                    allocator.create_unit_list(factory.create_variable_term(0)),
                ),
            )))
            .chain(once((
                factory.create_string_term(allocator.create_static_string("errors")),
                factory.create_application_term(
                    factory.create_builtin_term(CollectList),
                    allocator.create_unit_list(factory.create_variable_term(0)),
                ),
            ))),
            factory,
            allocator,
        ),
    )
}

fn parse_arrow_function_expression<T: Expression>(
    node: &ArrowExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    if node.is_generator || node.is_async {
        Err(err_unimplemented(node))
    } else {
        let num_args = node.params.len();
        let arg_names = node.params.iter().map(|node| match node {
            Pat::Ident(node) => Some(String::from(parse_identifier(&node.id))),
            _ => None,
        });
        let inner_scope = scope.create_child(arg_names);
        let inner_depth = inner_scope.depth();
        let (initializers, body_scope) = node.params.iter().enumerate().fold(
            Ok((Vec::new(), inner_scope)),
            |result, (arg_index, node)| {
                let (mut combined_initializers, existing_scope) = result?;
                match node {
                    Pat::Ident(_) => Ok((combined_initializers, existing_scope)),
                    Pat::Object(pattern) => {
                        let scope_offset = existing_scope.depth() - inner_depth;
                        let arg =
                            factory.create_variable_term(num_args - arg_index - 1 + scope_offset);
                        let (initializers, child_scope) =
                            parse_object_destructuring_pattern_bindings(
                                arg,
                                &pattern.props,
                                &existing_scope,
                                env,
                                factory,
                                allocator,
                            )?;
                        let next_scope = child_scope.unwrap_or(existing_scope);
                        combined_initializers.extend(initializers);
                        Ok((combined_initializers, next_scope))
                    }
                    Pat::Array(node) => {
                        let scope_offset = existing_scope.depth() - inner_depth;
                        let arg =
                            factory.create_variable_term(num_args - arg_index - 1 + scope_offset);
                        let (initializers, child_scope) =
                            parse_array_destructuring_pattern_bindings(
                                arg,
                                &node.elems,
                                &existing_scope,
                                env,
                                factory,
                                allocator,
                            )?;
                        let next_scope = child_scope.unwrap_or(existing_scope);
                        combined_initializers.extend(initializers);
                        Ok((combined_initializers, next_scope))
                    }
                    Pat::Rest(_) => Err(err_unimplemented(node)),
                    Pat::Assign(_) => Err(err_unimplemented(node)),
                    _ => Err(err_unimplemented(node)),
                }
            },
        )?;
        let body = match &node.body {
            BlockStmtOrExpr::Expr(expression) => {
                let body = vec![Stmt::Expr(ExprStmt {
                    span: node.span,
                    expr: expression.clone(),
                })];
                parse_block(&body, &body_scope, env, factory, allocator)
            }
            BlockStmtOrExpr::BlockStmt(node) => {
                parse_block(&node.stmts, &body_scope, env, factory, allocator)
            }
        }?;
        match body {
            None => Err(err("Missing function return statement", node)),
            Some(body) => Ok(factory.create_lambda_term(
                num_args,
                initializers
                    .into_iter()
                    .rev()
                    .fold(body, |body, initializer| {
                        factory.create_let_term(initializer, body)
                    }),
            )),
        }
    }
}

fn parse_member_expression<T: Expression>(
    node: &MemberExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let target = parse_expression(&node.obj, scope, env, factory, allocator)?;
    let field = match &node.prop {
        MemberProp::Ident(name) => {
            Ok(factory.create_string_term(allocator.create_string(parse_identifier(&name))))
        }
        MemberProp::Computed(key) => parse_expression(&key.expr, scope, env, factory, allocator),
        MemberProp::PrivateName(_) => Err(err_unimplemented(&node.prop)),
    }?;
    Ok(get_dynamic_field(target, field, factory, allocator))
}

fn get_static_field<T: Expression>(
    target: T,
    field: &str,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: WasmParserBuiltin,
{
    let field = factory.create_string_term(allocator.create_string(field));
    get_dynamic_field(target, field, factory, allocator)
}

fn get_dynamic_field<T: Expression>(
    target: T,
    field: T,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: WasmParserBuiltin,
{
    factory.create_application_term(
        factory.create_builtin_term(Accessor),
        allocator.create_pair(target, field),
    )
}

fn get_indexed_field<T: Expression>(
    target: T,
    index: usize,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: WasmParserBuiltin,
{
    get_dynamic_field(
        target,
        factory.create_int_term(index as i32),
        factory,
        allocator,
    )
}

fn parse_call_expression<T: Expression>(
    node: &CallExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let target = match &node.callee {
        Callee::Expr(callee) => parse_expression(callee, scope, env, factory, allocator),
        _ => Err(err_unimplemented(&node.callee)),
    }?;
    let args = &node.args;
    let num_args = args.len();
    let (args, spread) = args.into_iter().fold(
        (Vec::with_capacity(num_args), None),
        |(mut args, spread), node| {
            if node.spread.is_some() {
                (args, Some(&*node.expr))
            } else {
                args.push(&*node.expr);
                (args, spread)
            }
        },
    );
    let args = parse_expressions(args, scope, env, factory, allocator)?;
    let spread = match spread {
        Some(spread) => parse_expression(spread, scope, env, factory, allocator).map(Some),
        None => Ok(None),
    }?;
    if let Some(spread) = spread {
        let target = if args.is_empty() {
            target
        } else {
            factory.create_partial_application_term(target, allocator.create_list(args))
        };
        Ok(factory.create_application_term(
            factory.create_builtin_term(Apply),
            allocator.create_pair(target, spread),
        ))
    } else {
        Ok(factory.create_application_term(target, allocator.create_list(args)))
    }
}

fn parse_constructor_expression<T: Expression>(
    node: &NewExpr,
    scope: &LexicalScope,
    env: &Env<T>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> ParserResult<T>
where
    T::Builtin: WasmParserBuiltin,
{
    let target = parse_expression(&node.callee, scope, env, factory, allocator)?;
    let args = (&node.args)
        .iter()
        .flat_map(|args| {
            args.iter().map(|arg| {
                if arg.spread.is_some() {
                    Err(err_unimplemented(arg))
                } else {
                    parse_expression(&arg.expr, scope, env, factory, allocator)
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(factory.create_application_term(
        factory.create_builtin_term(Construct),
        allocator.create_pair(
            target,
            factory.create_list_term(allocator.create_list(args)),
        ),
    ))
}
