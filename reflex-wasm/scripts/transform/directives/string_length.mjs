// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const STRING_LENGTH_DIRECTIVE = '@string-length';

export default function stringLengthDirective(node, context) {
  const [instruction, value, ...varArgs] = node.elements
    .flatMap((node) => (context.transform ? context.transform(node, context) : [node]))
    .filter((node) => !isNonFunctionalNode(node));
  if (
    !isNamedTermNode(STRING_LENGTH_DIRECTIVE, instruction) ||
    !value ||
    !isStringNode(value) ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${STRING_LENGTH_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [
    NodeType.Term({
      source: `${JSON.parse(value.source).length}`,
      location: node.location,
    }),
  ];
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isStringNode(node) {
  return node.type === NodeType.String;
}
