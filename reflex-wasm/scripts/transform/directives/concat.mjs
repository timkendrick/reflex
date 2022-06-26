// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const CONCAT_DIRECTIVE = '@concat';

export default function concatDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, ...items] = elements.filter((node) => !isNonFunctionalNode(node));
  const values = items.map((node) => {
    if (isIdentifierNode(node)) {
      return node.source.slice('$'.length);
    } else if (isTermNode(node)) {
      return node.source;
    } else if (isStringNode(node)) {
      return JSON.parse(node.source);
    } else {
      return null;
    }
  });
  if (
    !isNamedTermNode(CONCAT_DIRECTIVE, instruction) ||
    values.some((value) => typeof value !== 'string')
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${CONCAT_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [
    NodeType.Term({
      source: values.join(''),
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

function isTermNode(node) {
  return node.type === NodeType.Term;
}

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}

function isStringNode(node) {
  return node.type === NodeType.String;
}
