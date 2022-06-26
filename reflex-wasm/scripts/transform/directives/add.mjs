// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const ADD_DIRECTIVE = '@add';

export default function addItemDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, left, right, ...varArgs] = elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  const leftValue = parseIntegerLiteralNode(left);
  const rightValue = parseIntegerLiteralNode(right);
  if (
    !isNamedTermNode(ADD_DIRECTIVE, instruction) ||
    typeof leftValue !== 'number' ||
    typeof rightValue !== 'number' ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${ADD_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [createNumericLiteralNode(leftValue + rightValue, node.location)];
}

function parseIntegerLiteralNode(node) {
  if (!isTermNode(node)) return null;
  const value = Number(node.source);
  return Number.isInteger(value) ? value : null;
}

function createNumericLiteralNode(value, location) {
  return NodeType.Term({ source: `${value}`, location });
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isTermNode(node) {
  return node.type === NodeType.Term;
}
