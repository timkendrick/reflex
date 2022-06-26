// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { createListDirective, getListElements } from './list.mjs';
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const REVERSE_DIRECTIVE = '@reverse';

export default function lengthDirective(node, context) {
  const elements = node.elements.flatMap((node) =>
    context.transform ? context.transform(node, context) : [node],
  );
  const [instruction, list, ...varArgs] = elements.filter((node) => !isNonFunctionalNode(node));
  const items = getListElements(list);
  if (!isNamedTermNode(REVERSE_DIRECTIVE, instruction) || !items || varArgs.length > 0) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${REVERSE_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return [createListDirective({ elements: items.slice().reverse(), location: node.location })];
}

function isNonFunctionalNode(node) {
  return node.type === NodeType.Whitespace || node.type == NodeType.Comment;
}

function isNamedTermNode(source, node) {
  return node.type === NodeType.Term && node.source === source;
}
