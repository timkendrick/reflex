// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';
import { getListElements } from './list.mjs';

export const LIST_ITEM_DIRECTIVE = '@list_item';

export default function listItemDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, target, index, ...varArgs] = elements.filter(
    (node) => !isNonFunctionalNode(node),
  );
  const items = target && getListElements(target);
  if (
    !isNamedTermNode(LIST_ITEM_DIRECTIVE, instruction) ||
    !items ||
    !isIntegerLiteralNode(index) ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${LIST_ITEM_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const item = items[Number(index.source)];
  if (!item) {
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
  return [item];
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}

function isIntegerLiteralNode(node) {
  return node.type === NodeType.Term && Number.isInteger(Number(node.source));
}
