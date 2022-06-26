// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const LIST_DIRECTIVE = '@list';

export default function listDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, ...items] = elements.filter((node) => !isNonFunctionalNode(node));
  if (!isNamedTermNode(LIST_DIRECTIVE, instruction)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${LIST_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [
    NodeType.Instruction({
      instruction: LIST_DIRECTIVE,
      elements: [instruction, ...items],
      location: node.location,
    }),
  ];
}

export function createListDirective({ elements, location }) {
  return NodeType.Instruction({
    instruction: LIST_DIRECTIVE,
    elements: [
      NodeType.Term({ source: LIST_DIRECTIVE, location }),
      ...elements,
    ],
    location,
  });
}

export function getListElements(node) {
  if (!isNamedInstructionNode(node, LIST_DIRECTIVE)) return null;
  return node.elements.slice(1);
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isNamedInstructionNode(node, instruction) {
  return node.type === NodeType.Instruction && node.instruction === instruction;
}
