// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { BLOCK_DIRECTIVE } from './block.mjs';
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const TEMPLATE_DIRECTIVE = '@template';

export default function templateDirective(node, context) {
  const elements = node.elements.filter((node) => !isNonFunctionalNode(node));
  const [instruction, ...identifiers] = elements.slice(0, -1);
  const body = elements[elements.length - 1];
  if (
    !isNamedTermNode(TEMPLATE_DIRECTIVE, instruction) ||
    !identifiers.every((node) => isIdentifierNode(node)) ||
    !body ||
    !isNamedInstructionNode(body, BLOCK_DIRECTIVE)
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${TEMPLATE_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [
    NodeType.Instruction({
      instruction: TEMPLATE_DIRECTIVE,
      elements: [...identifiers, body],
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

function isIdentifierNode(node) {
  return node.type === NodeType.Term && node.source.startsWith('$');
}

function isNamedInstructionNode(node, instruction) {
  return node.type === NodeType.Instruction && node.instruction === instruction;
}
