// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const IMPORT_DIRECTIVE = '@import';

export default function importDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, identifier, importPath, ...varArgs] = elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  if (
    !isNamedTermNode(IMPORT_DIRECTIVE, instruction) ||
    !isIdentifierNode(identifier) ||
    !isStringNode(importPath) ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${IMPORT_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const importedPath = ((value) => {
    try {
      const path = JSON.parse(value);
      if (typeof path !== 'string') throw null;
      return path;
    } catch (err) {
      throw new ParseError(node.location, context.sources.get(context.path), err.message);
    }
  })(importPath.source);
  const { module: importedModule, exports: moduleExports } = ((path) => {
    try {
      return context.import(path, {});
    } catch (err) {
      throw new ParseError(node.location, context.sources.get(context.path), err.message);
    }
  })(importedPath);
  const importName = identifier.source;
  if (!moduleExports.has(importName)) {
    throw new ParseError(
      importedModule.location,
      context.sources.get(context.path),
      `Unable to import ${importName} from module ${importedPath}: ${
        moduleExports.size === 0
          ? 'Module does not declare any exports'
          : `Module exports: ${Array.from(moduleExports.keys()).join(', ')}`
      }`,
    );
  }
  return moduleExports.get(importName);
}

function isStringNode(node) {
  return node.type === NodeType.String;
}

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}
