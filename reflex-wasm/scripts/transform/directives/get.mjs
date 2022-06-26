// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const GET_DIRECTIVE = '@get';

export default function getDirective(node, context) {
  const [instruction, identifier, ...varArgs] = node.elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  if (
    !isNamedTermNode(GET_DIRECTIVE, instruction) ||
    !isIdentifierNode(identifier) ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${GET_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const value = context.variables && context.variables[identifier.source];
  if (value === undefined) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Undeclared variable: ${formatSourceRange(source, identifier.location)}`,
    );
  }
  if (value === null) return [];
  return context.transform ? context.transform(value, context) : [value];
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}
