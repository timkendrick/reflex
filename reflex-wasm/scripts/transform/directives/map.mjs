// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { getListElements } from './list.mjs';
import { createChildScope } from '../loader.mjs';
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const MAP_DIRECTIVE = '@map';

export default function mapDirective(node, context) {
  const [instruction, identifier, list, template, ...varArgs] = node.elements.reduce(
    (results, node) => {
      results.push(
        ...(results.length < 3 && context.transform
          ? context.transform(node, context)
          : [node]
        ).filter((node) => !isNonFunctionalNode(node)),
      );
      return results;
    },
    [],
  );
  const items = getListElements(list);
  if (
    !isNamedTermNode(MAP_DIRECTIVE, instruction) ||
    !identifier ||
    !isIdentifierNode(identifier) ||
    !items ||
    !template ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${MAP_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return items.flatMap((item, index) =>
    context.transform
      ? context.transform(
          template,
          createChildScope(context, {
            [identifier.source]: item,
            $_: NodeType.Term({ source: `${index}`, location: node.location }),
          }),
        )
      : [template],
  );
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
