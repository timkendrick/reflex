// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { getListElements } from './list.mjs';
import { createChildScope } from '../loader.mjs';
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const ZIP_DIRECTIVE = '@zip';

export default function zipDirective(node, context) {
  const [instruction, identifier1, identifier2, list1, list2, template, ...varArgs] =
    node.elements.reduce((results, node) => {
      results.push(
        ...(results.length < 5 && context.transform
          ? context.transform(node, context)
          : [node]
        ).filter((node) => !isNonFunctionalNode(node)),
      );
      return results;
    }, []);
  const items1 = getListElements(list1);
  const items2 = getListElements(list2);
  if (
    !isNamedTermNode(ZIP_DIRECTIVE, instruction) ||
    !identifier1 ||
    !isIdentifierNode(identifier1) ||
    !identifier2 ||
    !isIdentifierNode(identifier2) ||
    !items1 ||
    !items2 ||
    items1.length !== items2.length ||
    !template ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${ZIP_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  return items1.map((item, index) => [item, items2[index]]).flatMap(([item1, item2], index) => [
    ...(context.transform
      ? context.transform(
          template,
          createChildScope(context, {
            [identifier1.source]: item1,
            [identifier2.source]: item2,
            $_: NodeType.Term({ source: `${index}`, location: node.location }),
          }),
        )
      : [template]),
    NodeType.Whitespace({ source: '\n', location: node.location }),
  ]);
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
