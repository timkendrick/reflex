// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { getListElements, LIST_DIRECTIVE } from './list.mjs';
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const CHAIN_DIRECTIVE = '@chain';

export default function chainDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, left, right, ...varArgs] = elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  const leftItems = getListElements(left);
  const rightItems = getListElements(right);
  if (
    !isNamedTermNode(CHAIN_DIRECTIVE, instruction) ||
    !leftItems ||
    !rightItems ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${CHAIN_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [
    NodeType.Instruction({
      instruction: LIST_DIRECTIVE,
      elements: [instruction, ...leftItems, ...rightItems],
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
