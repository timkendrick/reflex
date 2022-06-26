// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const CHAR_DIRECTIVE = '@char';

export default function charDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, char, ...varArgs] = elements.filter((node) => !isNonFunctionalNode(node));
  if (
    !isNamedTermNode(CHAR_DIRECTIVE, instruction) ||
    !isStringNode(char) ||
    JSON.parse(char.source).length !== 1 ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${CHAR_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const charCode = JSON.parse(char.source).charCodeAt(0);
  return [
    NodeType.Instruction({
      instruction: 'i32.const',
      elements: [
        NodeType.Term({
          source: 'i32.const',
          location: char.location,
        }),
        NodeType.Whitespace({
          source: ' ',
          location: char.location,
        }),
        NodeType.Comment({
          source: `(; ${char.source} ;)`,
          location: char.location,
        }),
        NodeType.Whitespace({
          source: ' ',
          location: char.location,
        }),
        NodeType.Term({
          source: `${charCode}`,
          location: char.location,
        }),
      ],
      location: char.location,
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
