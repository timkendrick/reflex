// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
import { getListElements } from './list.mjs';
import { createChildScope } from '../loader.mjs';
import { NodeType } from '../parser.mjs';
import { formatSourceRange, ParseError } from '../utils.mjs';

export const FOLD_DIRECTIVE = '@fold';

export default function foldDirective(node, context) {
  const [instruction, accumulator, identifier, list, seed, template, ...varArgs] =
    node.elements.reduce((results, node) => {
      results.push(
        ...(results.length < 5 && context.transform
          ? context.transform(node, context)
          : [node]
        ).filter((node) => !isNonFunctionalNode(node)),
      );
      return results;
    }, []);
  const items = getListElements(list);
  if (
    !isNamedTermNode(FOLD_DIRECTIVE, instruction) ||
    !accumulator ||
    !isIdentifierNode(accumulator) ||
    !identifier ||
    !isIdentifierNode(identifier) ||
    !seed ||
    !items ||
    !template ||
    varArgs.length > 0
  ) {
    const source = context.sources.get(context.path);
    throw new ParseError(
      node.location,
      source,
      `Invalid ${FOLD_DIRECTIVE} directive: ${formatSourceRange(source, node.location)}`,
    );
  }
  const results = items.reduce((acc, item, index) => {
    const results = context.transform
      ? context.transform(
          template,
          createChildScope(context, {
            [accumulator.source]: acc,
            [identifier.source]: item,
            $_: NodeType.Term({ source: `${index}`, location: node.location }),
          }),
        )
      : [template];
    if (results.length !== 1) {
      const source = context.sources.get(context.path);
      throw new ParseError(
        node.location,
        source,
        `Invalid ${FOLD_DIRECTIVE} directive template: ${formatSourceRange(
          source,
          template.location,
        )}`,
      );
    }
    return results[0];
  }, seed);
  return [results];
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
