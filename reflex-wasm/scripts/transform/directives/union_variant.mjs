// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { parseUnionDefinition, UNION_DIRECTIVE } from './union.mjs';

export const UNION_VARIANT_DIRECTIVE = '@union_variant';

export default function unionVariantDirective(node, context) {
  if (!isNamedInstructionNode(UNION_VARIANT_DIRECTIVE, node)) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${UNION_VARIANT_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, root, index, ...varArgs] = elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  const unionDefinition = isNamedInstructionNode(UNION_DIRECTIVE, root)
    ? parseUnionDefinition(root, context)
    : null;
  if (
    !isNamedTermNode(UNION_VARIANT_DIRECTIVE, instruction) ||
    !unionDefinition ||
    !isIntegerLiteralNode(index) ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${UNION_VARIANT_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const member = unionDefinition.members[Number(index.source)];
  if (!member) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${LIST_ITEM_DIRECTIVE} directive index: ${formatSourceRange(
        source,
        index.location,
      )}`,
    );
  }
  return [member];
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

function isIntegerLiteralNode(node) {
  return node.type === NodeType.Term && Number.isInteger(Number(node.source));
}
