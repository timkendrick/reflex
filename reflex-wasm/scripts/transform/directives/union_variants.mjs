// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { createListDirective } from './list.mjs';
import { parseUnionDefinition, UNION_DIRECTIVE } from './union.mjs';

export const UNION_VARIANTS_DIRECTIVE = '@union_variants';

export default function unionVariantsDirective(node, context) {
  if (!isNamedInstructionNode(UNION_VARIANTS_DIRECTIVE, node)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${UNION_VARIANTS_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, root, ...varArgs] = elements.filter((node) => !isNonFunctionalNode(node));
  const unionDefinition = isNamedInstructionNode(UNION_DIRECTIVE, root)
    ? parseUnionDefinition(root, context)
    : null;
  if (
    !isNamedTermNode(UNION_VARIANTS_DIRECTIVE, instruction) ||
    !unionDefinition ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${UNION_VARIANTS_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [
    createListDirective({
      elements: unionDefinition.variants.map(({ identifier }) => identifier),
      location: root.location,
    }),
  ];
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedInstructionNode(instruction, node) {
  return node.type === NodeType.Instruction && node.instruction === instruction;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}
