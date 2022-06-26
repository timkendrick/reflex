// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const TO_STRING_DIRECTIVE = '@to-string';

export default function toStringDirective(node, context) {
  const [instruction, target, ...varArgs] = node.elements
    .flatMap((node) => (context.transform ? context.transform(node, context) : [node]))
    .filter((node) => !isNonFunctionalNode(node));
  if (
    !isNamedTermNode(TO_STRING_DIRECTIVE, instruction) ||
    !target ||
    !isTermNode(target) ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${TO_STRING_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [
    NodeType.String({
      source: `${JSON.stringify(
        target.source.startsWith('$') ? target.source.slice(1) : target.source,
      )}`,
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
