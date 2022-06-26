// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import fs from 'fs';
import path from 'path';

import { tokenize } from './lexer.mjs';
import { parse } from './parser.mjs';

export function createLoaderContext(entryPoint, { transform = null } = {}) {
  return createContext(
    entryPoint,
    {
      sources: new Map(),
      modules: new Map(),
      transform,
    },
    {},
  );
}

export function createChildScope(context, variables) {
  return createContext(context.path, context, { ...context.variables, ...variables });
}

export function loadModule(path, context, variables) {
  const isStaticModule = Object.keys(variables).length === 0;
  const existingModule = isStaticModule ? context.modules.get(path) : undefined;
  if (existingModule === null) {
    throw new Error(`Encountered circular dependency in ${context.path}: ${path}`);
  } else if (existingModule) {
    return existingModule;
  } else {
    const source = fs.readFileSync(path, 'utf-8');
    context.sources.set(path, source);
    if (isStaticModule) context.modules.set(path, null);
    const ast = parseAst(source, path);
    const childContext = createContext(path, context, variables);
    const transformedNode = context.transform ? context.transform(ast, childContext) : [ast];
    if (!Array.isArray(transformedNode)) {
      throw new Error(`Invalid source transformation: expected Array, received ${transformedNode}`);
    } else if (transformedNode.length === 0) {
      throw new Error('Invalid source transformation: missing root node');
    } else if (transformedNode.length > 1) {
      throw new Error(
        `Invalid source transformation: expected 1 root node, received ${transformedNode.length}`,
      );
    }
    const [transformedAst] = transformedNode;
    if (isStaticModule) context.modules.set(path, transformedAst);
    return transformedAst;
  }
}

function parseAst(source, path) {
  const tokens = tokenize(source, path);
  return parse(tokens, source, path);
}

function createContext(modulePath, { sources, modules, transform }, variables) {
  const context = {
    path: modulePath,
    sources,
    modules,
    transform,
    import(path, variables) {
      const resolvedPath = path.startsWith('/') ? path : getRelativePath(modulePath, path);
      return loadModule(resolvedPath, context, variables);
    },
    variables,
  };
  return context;
}

function getRelativePath(sourcePath, targetPath) {
  return path.join(path.dirname(sourcePath), targetPath);
}
